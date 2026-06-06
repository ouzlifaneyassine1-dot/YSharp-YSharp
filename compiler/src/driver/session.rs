use std::io;
use std::collections::HashMap;
use crossbeam_channel::{bounded, Sender, Receiver, TrySendError};
use smol_str::SmolStr;
use tracing_subscriber::EnvFilter;
use crate::error::Diagnostics;
use crate::driver::build::BuildGraph;
use crate::driver::target::Target;

struct ChannelWriter {
    sender: Sender<String>,
}

impl io::Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();
        match self.sender.try_send(s) {
            Ok(()) => Ok(buf.len()),
            Err(TrySendError::Full(_)) => Ok(buf.len()),
            Err(TrySendError::Disconnected(_)) => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "log channel disconnected",
            )),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct Session<'a> {
    pub diagnostics: &'a Diagnostics,
    pub log_sender: Sender<String>,
    pub log_receiver: Receiver<String>,
    pub source_map: HashMap<SmolStr, String>,
}

impl<'a> Session<'a> {
    pub fn new(diagnostics: &'a Diagnostics, log_level: &str) -> Self {
        let (sender, receiver) = bounded::<String>(10_000);

        let log_sender = sender.clone();
        let make_writer = move || ChannelWriter {
            sender: log_sender.clone(),
        };

        let filter = EnvFilter::try_new(log_level)
            .unwrap_or_else(|_| EnvFilter::new("warn"));

        let _ = tracing_subscriber::fmt()
            .with_writer(make_writer)
            .with_env_filter(filter)
            .try_init();

        Session {
            diagnostics,
            log_sender: sender,
            log_receiver: receiver,
            source_map: HashMap::new(),
        }
    }

    pub fn build(
        &mut self,
        file: &str,
        target_str: &str,
        output: Option<&str>,
        easy: bool,
        link_flags: &[String],
        opt_level: &str,
        cpp_mode: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut raw_source = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read source '{}': {}", file, e))?;

        // Strip Unix shebang (#!/path/to/interpreter) before lexing/parsing
        if raw_source.starts_with("#!") {
            if let Some(end) = raw_source.find('\n') {
                raw_source = raw_source[end + 1..].to_string();
            } else {
                raw_source = String::new();
            }
        }

        // Auto-detect Y# Easy by extension or --easy flag
        let is_easy = easy || file.ends_with(".yse");

        let source = if is_easy {
            let transpiled = crate::easy::transpile(&raw_source);
            self.log_sender
                .try_send(format!("[info] transpiled Y# Easy -> Y# ({} -> {} bytes)", raw_source.len(), transpiled.len()))
                .ok();
            transpiled
        } else {
            raw_source.clone()
        };

        let fpath = SmolStr::from(file);
        self.source_map.insert(fpath.clone(), source.clone());
        self.diagnostics
            .source_map
            .borrow_mut()
            .insert(fpath.clone(), source.clone());

        self.log_sender
            .try_send(format!("[info] reading '{}' ({} bytes)", file, source.len()))
            .ok();

        // --- Pipeline ---

        let target = Target::from_str(target_str)
            .map_err(|e| format!("invalid target '{}': {}", target_str, e))?;

        let mut bg = BuildGraph::new();
        bg.add_file(fpath.clone(), source);

        // Phase 1: Lexical analysis (diagnostic only — parser has its own lexer)
        let _tokens = {
            let src = bg.get_source(&fpath).unwrap();
            let mut lexer = crate::lexer::Lexer::new(src);
            match lexer.tokenize() {
                Ok(t) => {
                    self.log_sender
                        .try_send(format!("[info] lexer: {} tokens produced", t.len()))
                        .ok();
                    t
                }
                Err(diag) => {
                    self.diagnostics
                        .add(diag.level, &diag.message, diag.span);
                    return Err("lexical analysis failed".into());
                }
            }
        };

        // Phase 2: Parsing
        let src_str = bg.get_source(&fpath).unwrap();
        let parser_result = crate::parser::parse_program(src_str);
        let (parser_arena, root_id) = parser_result.map_err(|e| {
            // Try to extract line/col from parse error
            let span = extract_span_from_source(src_str, &e, &fpath);
            self.diagnostics
                .add(crate::error::DiagnosticLevel::Error, &e, span);
            format!("parsing failed: {}", e)
        })?;

        self.log_sender
            .try_send(format!("[info] parser: {} nodes produced", parser_arena.len()))
            .ok();

        // Phase 3: Type checking — strict, errors block the build
        let mut type_env = crate::typeck::TypeEnv::new();
        // Register built-in runtime functions
        register_builtins(&mut type_env);
        if let Err(type_err) = crate::typeck::infer_expr(&parser_arena, root_id, &mut type_env) {
            let err_span = type_err.span.map(|(line, col)| {
                crate::error::Span::new(fpath.clone(), line, col, line, col + 1)
            });
            self.diagnostics.add(
                crate::error::DiagnosticLevel::Error,
                format!("type error: {}", type_err.message),
                err_span,
            );
            return Err("type checking failed".into());
        }

        // Phase 4: Convert parser AST → HIR lowerer AST
        let conversion = crate::hir::lower::from_parser_ast(&parser_arena)
            .map_err(|e| format!("AST conversion failed: {}", e))?;

        // Build the main function (from program body or extracted definition)
        let all_hir_fns: Vec<crate::hir::HirFunction> = {
            // Check if any extracted function is named "main" (from `Function main()` inside Program)
            let existing_main = conversion.functions.iter().position(|f| f.name == "main");

            if let Some(idx) = existing_main {
                // Use the extracted `main` function directly, skip body-based main
                let mut fns: Vec<crate::hir::HirFunction> = conversion.functions;
                let main_fn = fns.remove(idx);
                vec![main_fn].into_iter().chain(fns).collect()
            } else {
                // Lower program body as main()
                let main_hir = crate::hir::lower::lower(&conversion.arena, &conversion.body_ids)
                    .map_err(|e| format!("HIR lowering failed: {}", e))?;
                let main_fn = crate::hir::HirFunction {
                    name: smol_str::SmolStr::new("main"),
                    params: vec![],
                    ret_type: crate::hir::HirType::Int,
                    body: main_hir,
                    is_async: false,
                    is_differentiable: false,
                };
                let mut fns = vec![main_fn];
                fns.extend(conversion.functions);
                fns
            }
        };

        self.log_sender
            .try_send(format!("[info] HIR: {} functions", all_hir_fns.len()))
            .ok();

        // Phase 6: Lower each function to MIR
        let mir_fns: Result<Vec<_>, _> = all_hir_fns
            .iter()
            .map(|hf| Ok::<_, Box<dyn std::error::Error>>(crate::mir::lower::lower_function(hf)))
            .collect();
        let mut mir_fns = mir_fns?;

        // Phase 7: MIR optimization passes
        let mut module = crate::mir::MirModule {
            name: smol_str::SmolStr::new("main"),
            functions: Vec::new(),
            target: target.to_mir_target(),
        };
        for mir_fn in &mut mir_fns {
            crate::mir::opt::optimize_function(mir_fn);
        }
        module.functions = mir_fns;

        self.log_sender
            .try_send(format!("[info] MIR optimizer: {} functions optimized", module.functions.len()))
            .ok();

        // Phase 8: Code generation
        let output_path = output
            .map(|s| s.to_string())
            .unwrap_or_else(|| target.default_output_name());

        let result = crate::codegen::generate(&module, &output_path, link_flags, opt_level, cpp_mode)
            .map_err(|e| format!("codegen failed: {}", e))?;

        let actual_path = match &result {
            crate::codegen::CodegenResult::Native(p)
            | crate::codegen::CodegenResult::Object(p) => p.clone(),
            crate::codegen::CodegenResult::SpirV(p) => p.clone(),
        };

        self.log_sender
            .try_send(format!(
                "[info] codegen: {:?} -> {}",
                result, output_path
            ))
            .ok();

        self.log_sender
            .try_send(format!("[info] build finished: {}", actual_path))
            .ok();

        Ok(actual_path)
    }
}

/// Register all built-in runtime functions with their type signatures.
fn register_builtins(type_env: &mut crate::typeck::TypeEnv) {
    use crate::typeck::context::{FunctionType, Type};
    macro_rules! builtin {
        ($name:expr, [$($p:expr),*], $ret:expr) => {
            type_env.bind_fn(SmolStr::new($name), FunctionType {
                params: vec![$($p),*],
                ret: $ret,
                is_differentiable: false,
            });
        };
        (poly $name:expr, $ret:expr) => {
            let p = type_env.fresh_type_var();
            type_env.bind_fn(SmolStr::new($name), FunctionType {
                params: vec![p],
                ret: $ret,
                is_differentiable: false,
            });
        };
    }
    use Type::*;

    // --- I/O (polymorphic: accepts any single arg) ---
    let print_p = type_env.fresh_type_var();
    type_env.bind_fn(SmolStr::new("Print"), FunctionType { params: vec![print_p], ret: Void, is_differentiable: false });
    let printline_p = type_env.fresh_type_var();
    type_env.bind_fn(SmolStr::new("PrintLine"), FunctionType { params: vec![printline_p], ret: Void, is_differentiable: false });
    builtin!("ReadLine", [], String);
    builtin!("ReadInt", [], Int);
    builtin!("ReadFloat", [], Float);
    builtin!("ReadAllText", [String], String);
    builtin!("WriteAllText", [String, String], Void);
    builtin!("AppendAllText", [String, String], Void);
    builtin!("FileExists", [String], Bool);
    builtin!("FileDelete", [String], Void);
    builtin!("FileCopy", [String, String], Void);
    builtin!("FileMove", [String, String], Void);
    builtin!("FileSize", [String], Int);

    // --- Directory ---
    builtin!("DirCreate", [String], Void);
    builtin!("DirDelete", [String], Void);
    builtin!("DirExists", [String], Bool);
    builtin!("DirList", [String], String);
    builtin!("GetCurrentDir", [], String);
    builtin!("SetCurrentDir", [String], Void);

    // --- Console ---
    builtin!("ClearScreen", [], Void);
    builtin!("CursorPos", [Int, Int], Void);
    builtin!("GetCursorX", [], Int);
    builtin!("GetCursorY", [], Int);
    builtin!("SetColor", [Int], Void);
    builtin!("ReadKey", [], Int);

    // --- System ---
    builtin!("ExitF", [Int], Void);
    builtin!("SleepF", [Int], Void);
    builtin!("Exec", [String], Int);
    builtin!("ExecOutput", [String], String);
    builtin!("GetEnv", [String], String);
    builtin!("SetEnv", [String, String], Void);
    builtin!("GetOS", [], String);
    builtin!("GetPID", [], Int);
    builtin!("GetUserName", [], String);
    builtin!("GetHostName", [], String);
    builtin!("GetCPUCount", [], Int);

    // --- Time ---
    builtin!("NowUnix", [], Int);
    builtin!("NowMillis", [], Int);
    builtin!("NowString", [], String);
    builtin!("DateString", [], String);
    builtin!("TimeString", [], String);
    builtin!("Year", [], Int);
    builtin!("Month", [], Int);
    builtin!("Day", [], Int);
    builtin!("Hour", [], Int);
    builtin!("Minute", [], Int);
    builtin!("Second", [], Int);

    // --- String ---
    builtin!("StringLen", [String], Int);
    builtin!("StringSub", [String, Int, Int], String);
    builtin!("StringSplit", [String, String], String);
    builtin!("StringContains", [String, String], Bool);
    builtin!("StringReplace", [String, String, String], String);
    builtin!("StringTrim", [String], String);
    builtin!("StringTrimLeft", [String], String);
    builtin!("StringTrimRight", [String], String);
    builtin!("StringToUpper", [String], String);
    builtin!("StringToLower", [String], String);
    builtin!("StringStartsWith", [String, String], Bool);
    builtin!("StringEndsWith", [String, String], Bool);
    builtin!("StringAt", [String, Int], String);
    builtin!("StringPadLeft", [String, Int, String], String);
    builtin!("StringPadRight", [String, Int, String], String);

    // --- Conversion ---
    builtin!("ToInt", [Int], Int);
    builtin!("ToFloat", [Float], Float);
    builtin!("ToString", [String], String);
    builtin!("ParseInt", [String], Int);
    builtin!("ParseFloat", [String], Float);
    builtin!("Format", [String, String], String);
    builtin!("IntToStr", [Int], String);
    builtin!("FloatToStr", [Float], String);
    builtin!("BoolToStr", [Bool], String);
    builtin!("StrToInt", [String], Int);
    builtin!("StrToFloat", [String], Float);
    builtin!("CharCode", [String], Int);
    builtin!("CodeChar", [Int], String);

    // --- Math ---
    builtin!("Abs", [Int], Int);
    builtin!("AbsF", [Float], Float);
    builtin!("Min", [Int, Int], Int);
    builtin!("MinF", [Float, Float], Float);
    builtin!("Max", [Int, Int], Int);
    builtin!("MaxF", [Float, Float], Float);
    builtin!("Clamp", [Int, Int, Int], Int);
    builtin!("ClampF", [Float, Float, Float], Float);
    builtin!("Sin", [Float], Float);
    builtin!("Cos", [Float], Float);
    builtin!("Tan", [Float], Float);
    builtin!("Asin", [Float], Float);
    builtin!("Acos", [Float], Float);
    builtin!("Atan", [Float], Float);
    builtin!("Atan2", [Float, Float], Float);
    builtin!("Sqrt", [Float], Float);
    builtin!("Pow", [Float, Float], Float);
    builtin!("Exp", [Float], Float);
    builtin!("Log", [Float], Float);
    builtin!("Log2", [Float], Float);
    builtin!("Log10", [Float], Float);
    builtin!("Floor", [Float], Int);
    builtin!("Ceil", [Float], Int);
    builtin!("Round", [Float], Int);
    builtin!("Trunc", [Float], Int);
    builtin!("Frac", [Float], Float);
    builtin!("Sign", [Int], Int);
    builtin!("SignF", [Float], Float);
    builtin!("Lerp", [Float, Float, Float], Float);
    builtin!("Random", [], Float);
    builtin!("RandomRange", [Float, Float], Float);
    builtin!("RandomInt", [Int, Int], Int);
    builtin!("SeedRandom", [Int], Void);
    builtin!("DegToRad", [Float], Float);
    builtin!("RadToDeg", [Float], Float);
    builtin!("Hypot", [Float, Float], Float);

    // --- Memory / Advanced ---
    builtin!("MemoryAddress", [Int], Int);
    builtin!("MemorySize", [], Int);
    builtin!("StackAlloc", [Int], Int);
    builtin!("StackFree", [Int], Void);
    builtin!("CopyMem_", [Int, Int, Int], Void);
    builtin!("CompareMemory", [Int, Int, Int], Bool);
    builtin!("SetMemory", [Int, Int, Int], Void);

    // --- Process ---
    builtin!("RunProcess", [String, String], Int);
    builtin!("KillProcess", [Int], Void);
    builtin!("ProcessExists", [Int], Bool);
    builtin!("WaitProcess", [Int], Int);

    // --- Network ---
    builtin!("HttpGet", [String], String);
    builtin!("HttpPost", [String, String], String);
    builtin!("HttpGetJson", [String], String);
    builtin!("HttpPostJson", [String, String], String);
    builtin!("DownloadFile", [String, String], Bool);
    builtin!("PingHost", [String], Bool);
    builtin!("ResolveHost", [String], String);

    // --- Type ---
    builtin!("TypeOf", [Int], String);
    builtin!("IsInt", [Int], Bool);
    builtin!("IsFloat", [Float], Bool);
    builtin!("IsString", [String], Bool);
    builtin!("IsBool", [Bool], Bool);
}

/// Try to extract a source span from an error message that might contain
/// "at line N" or "(L:C)" patterns.
fn extract_span_from_source(
    _src: &str,
    err_msg: &str,
    file: &SmolStr,
) -> Option<crate::error::Span> {
    // Pattern: "at line N" or "line N" or "on line N" in the error message
    let bytes = err_msg.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        // Find "line" keyword
        if i + 4 < len
            && (bytes[i] == b'l' || bytes[i] == b'L')
            && (bytes[i+1] == b'i' || bytes[i+1] == b'I')
            && (bytes[i+2] == b'n' || bytes[i+2] == b'N')
            && (bytes[i+3] == b'e' || bytes[i+3] == b'E')
            && i + 4 < len
            && bytes[i+4] == b' '
        {
            let start = i + 5;
            let mut end = start;
            while end < len && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start {
                let line: usize = err_msg[start..end].parse().ok()?;
                // Optional colon + column
                let col = if end < len && bytes[end] == b':' {
                    let col_start = end + 1;
                    let mut col_end = col_start;
                    while col_end < len && bytes[col_end].is_ascii_digit() {
                        col_end += 1;
                    }
                    if col_end > col_start {
                        err_msg[col_start..col_end].parse().ok().unwrap_or(1)
                    } else {
                        1
                    }
                } else {
                    1
                };
                return Some(crate::error::Span::new(
                    file.clone(), line, col, line, col + 1,
                ));
            }
        }
        i += 1;
    }
    None
}
