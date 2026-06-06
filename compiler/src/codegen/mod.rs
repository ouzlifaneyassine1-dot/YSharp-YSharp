use crate::mir::ir::{MirModule, MirType, MirValue, MirInst, MirTerminator, MirBinOp, MirUnaryOp};
use smol_str::SmolStr;

pub mod c_backend;
pub mod game;
pub mod gpu;

#[derive(Debug, Clone)]
pub enum CodegenResult {
    Native(String),
    Object(String),
    SpirV(String),
}

/// Collected string literal: (vreg_name, string_value)
pub type StringLiteralEntry = (SmolStr, SmolStr);

pub fn generate(module: &MirModule, output: &str, link_flags: &[String], opt_level: &str, cpp_mode: bool) -> Result<CodegenResult, String> {
    match module.target {
        crate::mir::ir::Target::Native
        | crate::mir::ir::Target::Server
        | crate::mir::ir::Target::Desktop
        | crate::mir::ir::Target::Mobile
        | crate::mir::ir::Target::Wasm
        | crate::mir::ir::Target::Kernel => {
            let backend_module = convert_to_cmir(module);
            c_backend::generate(&backend_module, output, link_flags, opt_level, cpp_mode).map(CodegenResult::Native)
        }
        crate::mir::ir::Target::Game => {
            let game_module = convert_to_game_mir(module);
            game::generate(&game_module, output).map(CodegenResult::Native)
        }
        crate::mir::ir::Target::Gpu => {
            let gpu_module = convert_to_gpu_mir(module);
            gpu::generate(&gpu_module, output).map(CodegenResult::SpirV)
        }
    }
}

fn mir_to_cmir_type(ty: &MirType) -> c_backend::CMirType {
    match ty {
        MirType::Int => c_backend::CMirType::I64,
        MirType::Float => c_backend::CMirType::F64,
        MirType::Bool => c_backend::CMirType::Bool,
        MirType::String => c_backend::CMirType::Ptr(Box::new(c_backend::CMirType::I8)),
        MirType::Ptr(_) => c_backend::CMirType::Ptr(Box::new(c_backend::CMirType::I8)),
        MirType::Tensor(_, _) => c_backend::CMirType::Ptr(Box::new(c_backend::CMirType::I8)),
        MirType::Unit => c_backend::CMirType::Void,
    }
}

fn mir_to_cmir_inst(inst: &MirInst, vreg_map: &mut impl FnMut(MirValue) -> SmolStr) -> c_backend::CMirInst {
    use MirInst as MI;
    match inst {
        MI::Alloca { dest, ty, .. } => c_backend::CMirInst::Alloca {
            dest: vreg_map(*dest),
            ty: mir_to_cmir_type(ty),
        },
        MI::IntLiteral { dest, val } => c_backend::CMirInst::Binary {
            dest: vreg_map(*dest),
            op: "=".to_string(),
            lhs: SmolStr::new(format!("{}LL", val)),
            rhs: SmolStr::new("0"),
        },
        MI::FloatLiteral { dest, val } => c_backend::CMirInst::Binary {
            dest: vreg_map(*dest),
            op: "=".to_string(),
            lhs: SmolStr::new(format!("{}", val)),
            rhs: SmolStr::new("0.0"),
        },
        MI::BoolLiteral { dest, val } => c_backend::CMirInst::Binary {
            dest: vreg_map(*dest),
            op: "=".to_string(),
            lhs: if *val { SmolStr::new("1") } else { SmolStr::new("0") },
            rhs: SmolStr::new("0"),
        },
        MI::StringLiteral { dest, val: _ } => c_backend::CMirInst::Alloca {
            dest: vreg_map(*dest),
            ty: c_backend::CMirType::Ptr(Box::new(c_backend::CMirType::I8)),
        },
        MI::Load { dest, src } => c_backend::CMirInst::Load {
            dest: vreg_map(*dest),
            src: vreg_map(*src),
        },
        MI::Store { dest, src } => c_backend::CMirInst::Store {
            dest: vreg_map(*dest),
            src: vreg_map(*src),
        },
        MI::Binary { dest, op, left, right } => {
            let op_str = match op {
                MirBinOp::Add => "+", MirBinOp::Sub => "-", MirBinOp::Mul => "*",
                MirBinOp::Div => "/", MirBinOp::Mod => "%",
                MirBinOp::Eq => "==", MirBinOp::Neq => "!=",
                MirBinOp::Lt => "<", MirBinOp::Gt => ">",
                MirBinOp::Le => "<=", MirBinOp::Ge => ">=",
                MirBinOp::And => "&&", MirBinOp::Or => "||",
            };
            c_backend::CMirInst::Binary {
                dest: vreg_map(*dest),
                op: op_str.to_string(),
                lhs: vreg_map(*left),
                rhs: vreg_map(*right),
            }
        }
        MI::Call { dest, callee, args } => c_backend::CMirInst::Call {
            dest: dest.map(|d| vreg_map(d)),
            name: callee.to_string(),
            args: args.iter().map(|a| vreg_map(*a)).collect(),
        },
        MI::Print { dest, arg, arg_type, newline } => {
            let print_fn = match arg_type {
                MirType::String => "_ys_print_str",
                MirType::Int => "_ys_print_int",
                MirType::Float => "_ys_print_float",
                MirType::Bool => "_ys_print_float",
                MirType::Ptr(_) => "_ys_print_str",
                MirType::Tensor(..) => "_ys_print_str",
                MirType::Unit => "_ys_print_int",
            };
            let effective_name = if *newline {
                format!("{}_nl", print_fn)
            } else {
                print_fn.to_string()
            };
            c_backend::CMirInst::Call {
                dest: dest.map(|d| vreg_map(d)),
                name: effective_name,
                args: vec![vreg_map(*arg)],
            }
        },
        MI::Phi { dest, incoming } => c_backend::CMirInst::Load {
            dest: vreg_map(*dest),
            src: incoming.first().map(|(v, _)| vreg_map(*v)).unwrap_or_default(),
        },
        MI::Param { dest, index } => c_backend::CMirInst::Load {
            dest: vreg_map(*dest),
            src: SmolStr::new(format!("p{}", index)),
        },
        MI::Unary { dest, op, operand } => c_backend::CMirInst::Binary {
            dest: vreg_map(*dest),
            op: match op {
                MirUnaryOp::Neg => "-".to_string(),
                MirUnaryOp::Not => "!".to_string(),
            },
            lhs: vreg_map(*operand),
            rhs: SmolStr::new("0"),
        },
        MI::VectorHint { .. } => c_backend::CMirInst::Alloca {
            dest: SmolStr::new(""),
            ty: c_backend::CMirType::Void,
        },
        MI::InlineHint { .. } => c_backend::CMirInst::Alloca {
            dest: SmolStr::new(""),
            ty: c_backend::CMirType::Void,
        },
    }
}

fn mir_to_cmir_terminator(term: &MirTerminator, vreg_map: &mut impl FnMut(MirValue) -> SmolStr) -> c_backend::CTerminator {
    use MirTerminator as MT;
    match term {
        MT::Branch(id) => c_backend::CTerminator::Goto(SmolStr::new(format!("bb{}", id))),
        MT::CondBranch { cond, true_block, false_block } => {
            c_backend::CTerminator::BranchIf {
                cond: vreg_map(*cond),
                then_block: SmolStr::new(format!("bb{}", true_block)),
                else_block: SmolStr::new(format!("bb{}", false_block)),
            }
        }
        MT::Return(val) => c_backend::CTerminator::Return(val.map(|v| vreg_map(v))),
        MT::Unreachable => c_backend::CTerminator::Unreachable,
    }
}

fn collect_string_literals(module: &MirModule) -> std::collections::HashMap<MirValue, SmolStr> {
    let mut strings = std::collections::HashMap::new();
    for f in &module.functions {
        for b in &f.cfg.blocks {
            for inst in &b.instructions {
                if let MirInst::StringLiteral { dest, val } = inst {
                    strings.entry(*dest).or_insert_with(|| val.clone());
                }
            }
        }
    }
    strings
}

fn convert_to_cmir(module: &MirModule) -> c_backend::CMirModule {
    let mut vreg_counter = 0u32;
    let mut vreg_cache = std::collections::HashMap::new();
    let mut map_vreg = |v: MirValue| -> SmolStr {
        vreg_cache.entry(v).or_insert_with(|| {
            let name = SmolStr::new(format!("_v{}", vreg_counter));
            vreg_counter += 1;
            name
        }).clone()
    };

    let string_literals = collect_string_literals(module);

    c_backend::CMirModule {
        name: module.name.to_string(),
        functions: module.functions.iter().map(|f| {
            c_backend::CMirFunction {
                name: f.name.to_string(),
                params: f.params.iter().enumerate().map(|(i, ty)| {
                    (SmolStr::new(format!("p{}", i)), mir_to_cmir_type(ty))
                }).collect(),
                return_type: mir_to_cmir_type(&f.ret_type),
                blocks: f.cfg.blocks.iter().map(|b| {
                    c_backend::CBasicBlock {
                        name: SmolStr::new(format!("bb{}", b.id)),
                        insts: b.instructions.iter().map(|inst| mir_to_cmir_inst(inst, &mut map_vreg)).collect(),
                        terminator: mir_to_cmir_terminator(&b.terminator, &mut map_vreg),
                    }
                }).collect(),
                linkage: c_backend::CLinkage::External,
            }
        }).collect(),
        globals: vec![],
        string_literals: string_literals.iter().map(|(v, s)| {
            (map_vreg(*v), s.clone())
        }).collect(),
    }
}

fn convert_to_game_mir(module: &MirModule) -> crate::codegen::game::GameMirModule {
    crate::codegen::game::convert_module(module)
}

fn convert_to_gpu_mir(module: &MirModule) -> crate::codegen::gpu::GpuMirModule {
    crate::codegen::gpu::convert_module(module)
}


