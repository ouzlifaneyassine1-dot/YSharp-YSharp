use smol_str::SmolStr;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::ffi::OsString;

/// Backend-specific MIR types for C codegen
#[derive(Debug, Clone)]
pub enum CMirType { I8, I16, I32, I64, U8, U16, U32, U64, F32, F64, Bool, Void, Ptr(Box<CMirType>), }
#[derive(Debug, Clone)]
pub enum CTerminator { Goto(SmolStr), BranchIf { cond: SmolStr, then_block: SmolStr, else_block: SmolStr }, Return(Option<SmolStr>), Unreachable, }
#[derive(Debug, Clone)]
pub enum CMirInst {
    Alloca { dest: SmolStr, ty: CMirType },
    Load { dest: SmolStr, src: SmolStr },
    Store { dest: SmolStr, src: SmolStr },
    Binary { dest: SmolStr, op: String, lhs: SmolStr, rhs: SmolStr },
    Unary { dest: SmolStr, op: String, operand: SmolStr },
    Call { dest: Option<SmolStr>, name: String, args: Vec<SmolStr> },
    Return(Option<SmolStr>),
}
#[derive(Debug, Clone)]
pub struct CBasicBlock { pub name: SmolStr, pub insts: Vec<CMirInst>, pub terminator: CTerminator, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CLinkage { Internal, External, }
#[derive(Debug, Clone)]
pub struct CMirFunction { pub name: String, pub params: Vec<(SmolStr, CMirType)>, pub return_type: CMirType, pub blocks: Vec<CBasicBlock>, pub linkage: CLinkage, }
#[derive(Debug, Clone)]
pub struct CMirGlobal { pub name: String, pub ty: CMirType, pub init: Option<Vec<u8>>, pub mutable: bool, }
#[derive(Debug, Clone)]
pub struct CMirModule {
    pub name: String,
    pub functions: Vec<CMirFunction>,
    pub globals: Vec<CMirGlobal>,
    pub string_literals: Vec<(SmolStr, SmolStr)>,
}

pub fn generate(module: &CMirModule, output: &str, link_flags: &[String], opt_level: &str, cpp_mode: bool) -> Result<String, String> {
    let exe_path = if output.ends_with(".exe") { output.to_string() } else { format!("{}.exe", output) };
    let c_path = format!("{}.c", exe_path);
    let mut out = String::new();

    out.push_str("#define WIN32_LEAN_AND_MEAN\n#include <stdio.h>\n#include <stdlib.h>\n#include <stdint.h>\n#include <stdbool.h>\n#include <stddef.h>\n#include <string.h>\n#include <math.h>\n#include <time.h>\n#include <ctype.h>\n#include <sys/stat.h>\n#include <direct.h>\n#include <process.h>\n#include <conio.h>\n#include <winsock2.h>\n#include <ws2tcpip.h>\n#include <windows.h>\n\n");

    // Global result buffer for string-returning built-in functions
    out.push_str("int8_t _ys_retbuf[65536];\n\n");

    // Forward declarations for runtime built-ins (needed before user code)
    out.push_str("int64_t _ys_print_str(int64_t s);\n");
    out.push_str("int64_t _ys_print_int(int64_t v);\n");
    out.push_str("int64_t _ys_print_float(double v);\n");
    out.push_str("int64_t _ys_print_newline();\n\n");

    // Forward declarations for all functions
    for func in &module.functions {
        out.push_str(&format!("{} {}(", emit_c_type(&func.return_type), func.name));
        let params: Vec<String> = func.params.iter().map(|(_, t)| emit_c_type(t)).collect();
        out.push_str(&params.join(", "));
        out.push_str(");\n");
    }
    out.push('\n');

    // Emit string literal globals: const char _s0[] = "...";
    let mut str_idx = 0u32;
    for (_vreg, val) in &module.string_literals {
        let escaped = val.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
        out.push_str(&format!("const char _s{}[] = \"{}\";\n", str_idx, escaped));
        str_idx += 1;
    }
    if !module.string_literals.is_empty() {
        out.push('\n');
    }

    // Emit runtime function implementations (built-in stubs) — before user code
    out.push_str(include_str!("runtime.c"));
    out.push('\n');

    // Emit function definitions
    for func in &module.functions {
        if func.blocks.is_empty() { continue; }
        out.push_str(&emit_c_function(func, &module.string_literals));
        out.push('\n');
    }

    std::fs::write(&c_path, &out).map_err(|e| format!("failed to write C: {}", e))?;

    // Find gcc (or g++ for C++ mode) in common locations
    let compiler_name = if cpp_mode { "g++" } else { "gcc" };
    let gcc_path = find_compiler(compiler_name);
    let (gcc_cmd, gcc_dir) = match gcc_path {
        Some(ref p) => (p.to_string_lossy().to_string(), p.parent().map(|d| d.to_path_buf())),
        None => (compiler_name.to_string(), None),
    };

    // Map optimization level to GCC flags
    let opt_flag = match opt_level {
        "0" => "-O0",
        "1" => "-O1",
        "2" => "-O2",
        "3" => "-O3",
        "s" => "-Os",
        "z" => "-Oz",
        _ => "-O2",
    };

    // Determine language standard
    let std_flag = if cpp_mode { "-std=c++17" } else { "-std=c99" };

    // Compile + link to produce .exe
    let mut test_cmd = std::process::Command::new(&gcc_cmd);
    test_cmd.arg("--version");
    if let Some(ref dir) = gcc_dir {
        let mut path = OsString::from(dir);
        path.push(";");
        if let Ok(existing) = std::env::var("PATH") {
            path.push(existing);
        }
        test_cmd.env("PATH", &path);
    }
    let compiler_available = if let Ok(s) = test_cmd.output() {
        s.status.success()
    } else {
        false
    };

    if compiler_available {
        let mut build_cmd = std::process::Command::new(&gcc_cmd);
        build_cmd.args([opt_flag, "-flto", std_flag, "-o", &exe_path, &c_path, "-lws2_32"]);
        if cpp_mode { build_cmd.arg("-lstdc++"); }
        for flag in link_flags { build_cmd.arg(format!("-l{}", flag)); }
        if let Some(ref dir) = gcc_dir {
            let dir_str = dir.to_string_lossy();
            let path_val = if let Ok(existing) = std::env::var("PATH") {
                format!("{};{}", dir_str, existing)
            } else {
                dir_str.to_string()
            };
            build_cmd.env("PATH", &path_val);
        }
        let r = build_cmd.status().map_err(|e| format!("{}: {}", compiler_name, e))?;
        if r.success() {
            // let _ = std::fs::remove_file(&c_path);
            return Ok(exe_path);
        }
    }
    Err(format!("{} not found or compilation failed", compiler_name))
}

fn find_compiler(name: &str) -> Option<PathBuf> {
    // Check PATH first
    if let Ok(output) = std::process::Command::new(name).arg("--version").output() {
        if output.status.success() {
            return Some(PathBuf::from(name));
        }
    }
    // For gcc, fall back to g++ if not found
    if name == "gcc" {
        if let Ok(output) = std::process::Command::new("g++").arg("--version").output() {
            if output.status.success() {
                return Some(PathBuf::from("g++"));
            }
        }
    }
    // Common locations on Windows — try both gcc and g++ executables
    let exe_name = format!("{}.exe", name);
    let base_dirs = vec![
        r"C:\ProgramData\chocolatey\lib\mingw\tools\install\mingw64\bin",
        r"C:\ProgramData\chocolatey\lib\mingw\tools\install\mingw32\bin",
        r"C:\msys64\mingw64\bin",
        r"C:\msys64\mingw32\bin",
        r"C:\MinGW\bin",
        r"C:\MinGW-w64\bin",
    ];
    for dir in &base_dirs {
        let p = PathBuf::from(dir).join(&exe_name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn emit_c_type(ty: &CMirType) -> String {
    match ty {
        CMirType::I8 => "int8_t".to_string(),
        CMirType::I16 => "int16_t".to_string(),
        CMirType::I32 => "int32_t".to_string(),
        CMirType::I64 => "int64_t".to_string(),
        CMirType::U8 => "uint8_t".to_string(),
        CMirType::U16 => "uint16_t".to_string(),
        CMirType::U32 => "uint32_t".to_string(),
        CMirType::U64 => "uint64_t".to_string(),
        CMirType::F32 => "float".to_string(),
        CMirType::F64 => "double".to_string(),
        CMirType::Bool => "bool".to_string(),
        CMirType::Void => "void".to_string(),
        CMirType::Ptr(inner) => format!("{}*", emit_c_type(inner)),
    }
}

fn emit_c_function(func: &CMirFunction, string_literals: &[(SmolStr, SmolStr)]) -> String {
    let mut out = String::new();
    let ret_ty = emit_c_type(&func.return_type);
    out.push_str(&format!("{} {}(", &ret_ty, func.name));
    let params: Vec<String> = func.params.iter().map(|(n, t)| format!("{} {}", emit_c_type(t), n)).collect();
    out.push_str(&params.join(", "));
    out.push_str(") {\n");

    // Collect ALL dest variables used in instructions (not just Alloca)
    let mut all_vars: BTreeSet<SmolStr> = BTreeSet::new();
    let mut alloca_types: Vec<(SmolStr, CMirType)> = Vec::new();

    for block in &func.blocks {
        for inst in &block.insts {
            match inst {
                CMirInst::Alloca { dest, ty } => {
                    all_vars.insert(dest.clone());
                    alloca_types.push((dest.clone(), ty.clone()));
                }
                CMirInst::Load { dest, .. } => { all_vars.insert(dest.clone()); }
                CMirInst::Store { .. } => {}
                CMirInst::Binary { dest, .. } => { all_vars.insert(dest.clone()); }
                CMirInst::Unary { dest, .. } => { all_vars.insert(dest.clone()); }
                CMirInst::Call { dest, .. } => {
                    if let Some(d) = dest { all_vars.insert(d.clone()); }
                }
                CMirInst::Return(_) => {}
            }
        }
    }

    // Declare alloca'd variables with their explicit types
    for (dest, ty) in &alloca_types {
        // String literals are stored as int64_t (pointer cast) to avoid type mismatch
        let is_str_lit = string_literals.iter().any(|(vreg, _)| vreg == dest);
        if is_str_lit {
            out.push_str(&format!("    int64_t {};\n", dest));
        } else {
            out.push_str(&format!("    {} {};\n", emit_c_type(ty), dest));
        }
    }
    // Declare other variables (non-alloca) as int64_t
    for v in &all_vars {
        if !alloca_types.iter().any(|(d, _)| d == v) {
            out.push_str(&format!("    int64_t {};\n", v));
        }
    }

    // Emit blocks
    for (bi, block) in func.blocks.iter().enumerate() {
        if bi > 0 { out.push_str(&format!("{}:\n", block.name)); }
        for inst in &block.insts {
            out.push_str(&format!("    {}\n", emit_c_inst(inst, string_literals)));
        }
        out.push_str(&format!("    {}\n", emit_c_terminator(&block.terminator, &ret_ty)));
    }

    out.push_str("}\n");
    out
}

fn emit_c_inst(inst: &CMirInst, string_literals: &[(SmolStr, SmolStr)]) -> String {
    match inst {
        CMirInst::Alloca { dest, ty: _ } => {
            // If this alloca corresponds to a string literal, initialize with the string constant
            for (idx, (vreg, _)) in string_literals.iter().enumerate() {
                if vreg == dest {
                    return format!("{} = (int64_t)(intptr_t)_s{};", dest, idx);
                }
            }
            format!("{} = 0;", dest)
        }
        CMirInst::Load { dest, src } => format!("{} = {};", dest, src),
        CMirInst::Store { dest, src } => format!("{} = {};", dest, src),
        CMirInst::Binary { dest, op, lhs, rhs } => {
            if op == "=" {
                format!("{} = {};", dest, lhs)
            } else if op == "!" {
                format!("{} = {}{};", dest, op, lhs)
            } else {
                format!("{} = {} {} {};", dest, lhs, op, rhs)
            }
        }
        CMirInst::Unary { dest, op, operand } => {
            format!("{} = {}{};", dest, op, operand)
        }
        CMirInst::Call { dest, name, args } => {
            let a: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            // Handle PrintLine: function names ending with _nl get a newline call too
            let (base_name, add_newline) = if let Some(stripped) = name.strip_suffix("_nl") {
                (stripped.to_string(), true)
            } else {
                (name.clone(), false)
            };
            let call = match dest {
                Some(d) => format!("{} = {}({});", d, base_name, a.join(", ")),
                None => format!("{}({});", base_name, a.join(", ")),
            };
            if add_newline {
                format!("{}\n    _ys_print_newline();", call)
            } else {
                call
            }
        }
        CMirInst::Return(Some(v)) => format!("return {};", v),
        CMirInst::Return(None) => "return 0;".to_string(),
    }
}

fn emit_c_terminator(term: &CTerminator, _ret_ty: &str) -> String {
    match term {
        CTerminator::Goto(t) => format!("goto {};", t),
        CTerminator::BranchIf { cond, then_block, else_block } => format!("if ({}) {{ goto {}; }} else {{ goto {}; }}", cond, then_block, else_block),
        CTerminator::Return(Some(v)) => format!("return {};", v),
        CTerminator::Return(None) => "return 0;".to_string(),
        CTerminator::Unreachable => "__builtin_unreachable();".to_string(),
    }
}
