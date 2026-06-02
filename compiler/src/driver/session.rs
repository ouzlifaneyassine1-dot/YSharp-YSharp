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
    ) -> Result<String, Box<dyn std::error::Error>> {
        let source = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read source '{}': {}", file, e))?;

        let fpath = SmolStr::from(file);
        self.source_map.insert(fpath.clone(), source.clone());
        self.diagnostics
            .source_map
            .borrow_mut()
            .insert(fpath.clone(), source.clone());

        let target = Target::from_str(target_str)
            .map_err(|e| format!("invalid target '{}': {}", target_str, e))?;

        self.log_sender
            .try_send(format!("[info] reading '{}' ({} bytes)", file, source.len()))
            .ok();

        // --- Pipeline ---

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
        let print_param = type_env.fresh_type_var();
        type_env.bind_fn(
            SmolStr::new("Print"),
            crate::typeck::context::FunctionType {
                params: vec![print_param],
                ret: crate::typeck::context::Type::Void,
                is_differentiable: false,
            },
        );
        let printline_param = type_env.fresh_type_var();
        type_env.bind_fn(
            SmolStr::new("PrintLine"),
            crate::typeck::context::FunctionType {
                params: vec![printline_param],
                ret: crate::typeck::context::Type::Void,
                is_differentiable: false,
            },
        );
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

        // Phase 5: Lower main body to HIR nodes
        let main_hir = crate::hir::lower::lower(&conversion.arena, &conversion.body_ids)
            .map_err(|e| format!("HIR lowering failed: {}", e))?;

        // Build the main function
        let main_fn = crate::hir::HirFunction {
            name: smol_str::SmolStr::new("main"),
            params: vec![],
            ret_type: crate::hir::HirType::Int,
            body: main_hir,
            is_async: false,
            is_differentiable: false,
        };

        // Collect all functions (main + extracted definitions)
        let all_hir_fns: Vec<crate::hir::HirFunction> = {
            let mut fns = vec![main_fn];
            fns.extend(conversion.functions);
            fns
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

        let codegen_result = crate::codegen::generate(&module, &output_path)
            .map_err(|e| format!("codegen failed: {}", e))?;

        self.log_sender
            .try_send(format!(
                "[info] codegen: {:?} -> {}",
                codegen_result, output_path
            ))
            .ok();

        self.log_sender
            .try_send(format!("[info] build finished: {}", output_path))
            .ok();

        Ok(output_path)
    }
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
