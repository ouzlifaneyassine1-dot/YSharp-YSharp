pub mod target;

use std::collections::HashMap;
use smol_str::SmolStr;

/// Backend-specific MIR types for LLVM codegen
#[derive(Debug, Clone)]
pub enum LlvmMirType { I8, I16, I32, I64, U8, U16, U32, U64, F32, F64, Bool, Void, Ptr(Box<LlvmMirType>), }

#[derive(Debug, Clone)]
pub enum LlvmBinaryOp { Add, Sub, Mul, Div, Rem, And, Or, Xor, Shl, Shr, Eq, Ne, Lt, Le, Gt, Ge, }

#[derive(Debug, Clone)]
pub enum LlvmTerminator { Goto(SmolStr), BranchIf { cond: SmolStr, then_block: SmolStr, else_block: SmolStr }, Return(Option<SmolStr>), Unreachable, }

#[derive(Debug, Clone)]
pub enum LlvmMirInst {
    Alloca { dest: SmolStr, ty: LlvmMirType },
    Load { dest: crate::mir::ir::MirValue, src: crate::mir::ir::MirValue },
    Store { dest: crate::mir::ir::MirValue, src: crate::mir::ir::MirValue },
    Binary { dest: crate::mir::ir::MirValue, op: LlvmBinaryOp, lhs: crate::mir::ir::MirValue, rhs: crate::mir::ir::MirValue },
    Call { dest: Option<SmolStr>, name: String, args: Vec<SmolStr> },
    Phi { dest: crate::mir::ir::MirValue, incoming: Vec<(SmolStr, SmolStr)> },
    Cast { dest: SmolStr, from: SmolStr, to: LlvmMirType },
    Gep { dest: SmolStr, ptr: SmolStr, indices: Vec<SmolStr> },
    Memcpy { dest: SmolStr, src: SmolStr, size: u64 },
    PtrToInt { dest: SmolStr, src: SmolStr },
    IntToPtr { dest: SmolStr, src: SmolStr, to: LlvmMirType },
}

#[derive(Debug, Clone)]
pub struct LlvmBasicBlock { pub name: SmolStr, pub insts: Vec<LlvmMirInst>, pub terminator: LlvmTerminator, }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlvmLinkage { Internal, External, Kernel, }

#[derive(Debug, Clone)]
pub struct LlvmMirFunction {
    pub name: String, pub params: Vec<(SmolStr, LlvmMirType)>,
    pub return_type: LlvmMirType, pub blocks: Vec<LlvmBasicBlock>, pub linkage: LlvmLinkage,
}

#[derive(Debug, Clone)]
pub struct LlvmMirGlobal { pub name: String, pub ty: LlvmMirType, pub init: Option<Vec<u8>>, pub mutable: bool, }

#[derive(Debug, Clone)]
pub struct LlvmMirModule { pub name: String, pub functions: Vec<LlvmMirFunction>, pub globals: Vec<LlvmMirGlobal>, }

pub fn generate(module: &LlvmMirModule, output: &str) -> Result<String, String> {
    // Without inkwell, emit .ll textual IR as a fallback
    let mut ir = String::new();
    ir.push_str(&format!("; Y# LLVM IR — Module: {}\n\n", module.name));

    for func in &module.functions {
        ir.push_str(&emit_llvm_function(func));
        ir.push('\n');
    }

    let ir_path = if output.ends_with(".ll") { output.to_string() } else { format!("{}.ll", output) };
    std::fs::write(&ir_path, &ir).map_err(|e| format!("failed to write IR: {}", e))?;
    Ok(ir_path)
}

fn emit_llvm_type(ty: &LlvmMirType) -> String {
    match ty {
        LlvmMirType::I8 => "i8", LlvmMirType::I16 => "i16", LlvmMirType::I32 => "i32",
        LlvmMirType::I64 => "i64", LlvmMirType::U8 => "i8", LlvmMirType::U16 => "i16",
        LlvmMirType::U32 => "i32", LlvmMirType::U64 => "i64", LlvmMirType::F32 => "float",
        LlvmMirType::F64 => "double", LlvmMirType::Bool => "i1", LlvmMirType::Void => "void",
        LlvmMirType::Ptr(_) => "ptr",
    }.to_string()
}

fn emit_llvm_function(func: &LlvmMirFunction) -> String {
    let mut out = String::new();
    let ret_ty = emit_llvm_type(&func.return_type);
    let params: Vec<String> = func.params.iter().map(|(n, t)| format!("{} %{}", emit_llvm_type(t), n)).collect();
    out.push_str(&format!("define {} @{}({}) {{\n", ret_ty, func.name, params.join(", ")));

    for block in &func.blocks {
        out.push_str(&format!("{}:\n", block.name));
        for inst in &block.insts {
            out.push_str(&format!("  {}\n", emit_llvm_inst(inst)));
        }
        out.push_str(&format!("  {}\n", emit_llvm_terminator(&block.terminator)));
    }

    out.push_str("}\n");
    out
}

fn emit_llvm_inst(inst: &LlvmMirInst) -> String {
    match inst {
        LlvmMirInst::Alloca { dest, ty } => format!("%{} = alloca {}", dest, emit_llvm_type(ty)),
        LlvmMirInst::Load { dest, src } => format!("%{} = load i64, ptr %{}", dest.0, src.0),
        LlvmMirInst::Store { dest, src } => format!("store i64 %{}, ptr %{}", src.0, dest.0),
        LlvmMirInst::Binary { dest, op, lhs, rhs } => {
            let op_str = match op {
                LlvmBinaryOp::Add => "add i64", LlvmBinaryOp::Sub => "sub i64",
                LlvmBinaryOp::Mul => "mul i64", LlvmBinaryOp::Div => "sdiv i64",
                LlvmBinaryOp::Rem => "srem i64", LlvmBinaryOp::And => "and i64",
                LlvmBinaryOp::Or => "or i64", LlvmBinaryOp::Xor => "xor i64",
                LlvmBinaryOp::Eq => "icmp eq i64", LlvmBinaryOp::Ne => "icmp ne i64",
                LlvmBinaryOp::Lt => "icmp slt i64", LlvmBinaryOp::Le => "icmp sle i64",
                LlvmBinaryOp::Gt => "icmp sgt i64", LlvmBinaryOp::Ge => "icmp sge i64",
            };
            format!("%{} = {} %{}, %{}", dest.0, op_str, lhs.0, rhs.0)
        }
        LlvmMirInst::Call { dest, name, args } => {
            let arg_strs: Vec<String> = args.iter().map(|a| format!("i64 %{}", a)).collect();
            match dest {
                Some(d) => format!("%{} = call i64 @{}({})", d, name, arg_strs.join(", ")),
                None => format!("call void @{}({})", name, arg_strs.join(", ")),
            }
        }
        LlvmMirInst::Phi { dest, incoming } => {
            let inc_strs: Vec<String> = incoming.iter().map(|(v, b)| format!("[i64 %{}, %{}]", v, b)).collect();
            format!("%{} = phi i64 {}", dest.0, inc_strs.join(", "))
        }
        LlvmMirInst::Cast { dest, from, to } => {
            let to_str = emit_llvm_type(to);
            format!("%{} = sext i64 %{} to {}", dest, from, to_str)
        }
        LlvmMirInst::Gep { dest, ptr, indices } => {
            let idx_strs: Vec<String> = indices.iter().map(|i| format!("i64 %{}", i)).collect();
            format!("%{} = getelementptr i8, ptr %{}, {}", dest, ptr, idx_strs.join(", "))
        }
        LlvmMirInst::Memcpy { dest, src, size } => {
            format!("call void @llvm.memcpy.p0.p0.i64(ptr %{}, ptr %{}, i64 {}, i1 false)", dest, src, size)
        }
        LlvmMirInst::PtrToInt { dest, src } => {
            format!("%{} = ptrtoint ptr %{} to i64", dest, src)
        }
        LlvmMirInst::IntToPtr { dest, src, to: _ } => {
            format!("%{} = inttoptr i64 %{} to ptr", dest, src)
        }
    }
}

fn emit_llvm_terminator(term: &LlvmTerminator) -> String {
    match term {
        LlvmTerminator::Goto(target) => format!("br label %{}", target),
        LlvmTerminator::BranchIf { cond, then_block, else_block } => {
            format!("br i1 %{}, label %{}, label %{}", cond, then_block, else_block)
        }
        LlvmTerminator::Return(Some(v)) => format!("ret i64 %{}", v),
        LlvmTerminator::Return(None) => "ret void".to_string(),
        LlvmTerminator::Unreachable => "unreachable".to_string(),
    }
}
