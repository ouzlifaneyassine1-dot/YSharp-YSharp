use rustc_hash::FxHashMap;

use crate::mir::ir::*;

/// Fold constant expressions in a MIR function.
///
/// This pass evaluates operations whose operands are all constant
/// at compile time, replacing the result with a constant literal.
pub fn fold_constants(func: &mut MirFunction) {
    let replaced: FxHashMap<u32, MirValue> = FxHashMap::default();
    let mut const_vals: FxHashMap<u32, ConstValue> = FxHashMap::default();

    for block in &mut func.cfg.blocks {
        let mut new_insts = Vec::with_capacity(block.instructions.len());

        for inst in block.instructions.drain(..) {
            match inst {
                MirInst::IntLiteral { dest, val } => {
                    const_vals.insert(dest.0, ConstValue::Int(val));
                    new_insts.push(MirInst::IntLiteral { dest, val });
                }
                MirInst::FloatLiteral { dest, val } => {
                    const_vals.insert(dest.0, ConstValue::Float(val));
                    new_insts.push(MirInst::FloatLiteral { dest, val });
                }
                MirInst::BoolLiteral { dest, val } => {
                    const_vals.insert(dest.0, ConstValue::Bool(val));
                    new_insts.push(MirInst::BoolLiteral { dest, val });
                }
                MirInst::StringLiteral { dest, val } => {
                    const_vals.insert(dest.0, ConstValue::String(val.clone()));
                    new_insts.push(MirInst::StringLiteral { dest, val });
                }

                MirInst::Binary {
                    dest,
                    op,
                    left,
                    right,
                } => {
                    let l = const_vals.get(&left.0);
                    let r = const_vals.get(&right.0);

                    if let (Some(lv), Some(rv)) = (l, r) {
                        if let Some(result) = fold_binary(op, lv.clone(), rv.clone()) {
                            match result {
                                ConstValue::Int(v) => {
                                    const_vals.insert(dest.0, ConstValue::Int(v));
                                    new_insts.push(MirInst::IntLiteral { dest, val: v });
                                }
                                ConstValue::Float(v) => {
                                    const_vals.insert(dest.0, ConstValue::Float(v));
                                    new_insts.push(MirInst::FloatLiteral { dest, val: v });
                                }
                                ConstValue::Bool(v) => {
                                    const_vals.insert(dest.0, ConstValue::Bool(v));
                                    new_insts.push(MirInst::BoolLiteral { dest, val: v });
                                }
                                ConstValue::String(_) => {
                                    // Binary ops don't produce strings
                                    const_vals.remove(&dest.0);
                                    new_insts.push(MirInst::Binary {
                                        dest,
                                        op,
                                        left,
                                        right,
                                    });
                                }
                            }
                            continue;
                        }
                    }
                    const_vals.remove(&dest.0);
                    new_insts.push(MirInst::Binary {
                        dest,
                        op,
                        left,
                        right,
                    });
                }

                MirInst::Unary { dest, op, operand } => {
                    if let Some(opv) = const_vals.get(&operand.0) {
                        if let Some(result) = fold_unary(op, opv.clone()) {
                            match result {
                                ConstValue::Int(v) => {
                                    const_vals.insert(dest.0, ConstValue::Int(v));
                                    new_insts.push(MirInst::IntLiteral { dest, val: v });
                                }
                                ConstValue::Float(v) => {
                                    const_vals.insert(dest.0, ConstValue::Float(v));
                                    new_insts.push(MirInst::FloatLiteral { dest, val: v });
                                }
                                ConstValue::Bool(v) => {
                                    const_vals.insert(dest.0, ConstValue::Bool(v));
                                    new_insts.push(MirInst::BoolLiteral { dest, val: v });
                                }
                                ConstValue::String(_) => {
                                    const_vals.remove(&dest.0);
                                    new_insts.push(MirInst::Unary { dest, op, operand });
                                }
                            }
                            continue;
                        }
                    }
                    const_vals.remove(&dest.0);
                    new_insts.push(MirInst::Unary { dest, op, operand });
                }

                other => {
                    // For other instructions, invalidate any dest that was being tracked
                    if let Some(dest) = inst_dest(&other) {
                        const_vals.remove(&dest.0);
                    }
                    new_insts.push(other);
                }
            }
        }

        block.instructions = new_insts;
    }

    // Fold terminators: if cond in CondBranch is constant, replace with Branch
    for i in 0..func.cfg.blocks.len() {
        let term = func.cfg.blocks[i].terminator.clone();
        if let MirTerminator::CondBranch {
            cond,
            true_block,
            false_block,
        } = term
        {
            if let Some(ConstValue::Bool(val)) = const_vals.get(&cond.0) {
                let target = if *val { true_block } else { false_block };
                func.cfg.blocks[i].terminator = MirTerminator::Branch(target);
            }
        }
    }

    // Rename uses of replaced values
    for block in &mut func.cfg.blocks {
        for inst in &mut block.instructions {
            rewrite_inst(inst, &replaced);
        }
        rewrite_terminator(&mut block.terminator, &replaced);
    }
}

/// Extract the destination value from an instruction (if any).
fn inst_dest(inst: &MirInst) -> Option<MirValue> {
    match inst {
        MirInst::Alloca { .. } => None,
        MirInst::Load { dest, .. } => Some(*dest),
        MirInst::Store { .. } => None,
        MirInst::Binary { dest, .. } => Some(*dest),
        MirInst::Unary { dest, .. } => Some(*dest),
        MirInst::Call { dest, .. } => *dest,
        MirInst::Print { dest, .. } => *dest,
        MirInst::IntLiteral { dest, .. } => Some(*dest),
        MirInst::FloatLiteral { dest, .. } => Some(*dest),
        MirInst::StringLiteral { dest, .. } => Some(*dest),
        MirInst::BoolLiteral { dest, .. } => Some(*dest),
        MirInst::Phi { dest, .. } => Some(*dest),
        MirInst::Param { dest, .. } => Some(*dest),
        MirInst::VectorHint { .. } => None,
        MirInst::InlineHint { dest, .. } => *dest,
    }
}

fn rewrite_inst(inst: &mut MirInst, replaced: &FxHashMap<u32, MirValue>) {
    match inst {
        MirInst::Binary {
            dest: _,
            op: _,
            left,
            right,
        } => {
            if let Some(&v) = replaced.get(&left.0) {
                *left = v;
            }
            if let Some(&v) = replaced.get(&right.0) {
                *right = v;
            }
        }
        MirInst::Unary {
            dest: _,
            op: _,
            operand,
        } => {
            if let Some(&v) = replaced.get(&operand.0) {
                *operand = v;
            }
        }
        MirInst::Load { dest: _, src } => {
            if let Some(&v) = replaced.get(&src.0) {
                *src = v;
            }
        }
        MirInst::Store { dest: _, src } => {
            if let Some(&v) = replaced.get(&src.0) {
                *src = v;
            }
        }
        MirInst::Call { args, .. } => {
            for arg in args.iter_mut() {
                if let Some(&v) = replaced.get(&arg.0) {
                    *arg = v;
                }
            }
        }
        MirInst::Print { arg, .. } => {
            if let Some(&v) = replaced.get(&arg.0) {
                *arg = v;
            }
        }
        MirInst::Phi { incoming, .. } => {
            for (val, _) in incoming.iter_mut() {
                if let Some(&v) = replaced.get(&val.0) {
                    *val = v;
                }
            }
        }
        MirInst::Alloca { .. }
        | MirInst::IntLiteral { .. }
        | MirInst::FloatLiteral { .. }
        | MirInst::StringLiteral { .. }
        | MirInst::BoolLiteral { .. }
        | MirInst::Param { .. }
        | MirInst::VectorHint { .. }
        | MirInst::InlineHint { .. } => {}
    }
}

fn rewrite_terminator(term: &mut MirTerminator, replaced: &FxHashMap<u32, MirValue>) {
    match term {
        MirTerminator::CondBranch { cond, .. } => {
            if let Some(&v) = replaced.get(&cond.0) {
                *cond = v;
            }
        }
        MirTerminator::Return(Some(val)) => {
            if let Some(&v) = replaced.get(&val.0) {
                *val = v;
            }
        }
        MirTerminator::Branch(_) | MirTerminator::Return(None) | MirTerminator::Unreachable => {}
    }
}

#[derive(Debug, Clone)]
enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(smol_str::SmolStr),
}

fn fold_binary(op: MirBinOp, left: ConstValue, right: ConstValue) -> Option<ConstValue> {
    match (left, right) {
        (ConstValue::Int(l), ConstValue::Int(r)) => match op {
            MirBinOp::Add => Some(ConstValue::Int(l.wrapping_add(r))),
            MirBinOp::Sub => Some(ConstValue::Int(l.wrapping_sub(r))),
            MirBinOp::Mul => Some(ConstValue::Int(l.wrapping_mul(r))),
            MirBinOp::Div if r != 0 => Some(ConstValue::Int(l / r)),
            MirBinOp::Mod if r != 0 => Some(ConstValue::Int(l % r)),
            MirBinOp::Eq => Some(ConstValue::Bool(l == r)),
            MirBinOp::Neq => Some(ConstValue::Bool(l != r)),
            MirBinOp::Lt => Some(ConstValue::Bool(l < r)),
            MirBinOp::Gt => Some(ConstValue::Bool(l > r)),
            MirBinOp::Le => Some(ConstValue::Bool(l <= r)),
            MirBinOp::Ge => Some(ConstValue::Bool(l >= r)),
            MirBinOp::And => Some(ConstValue::Int(l & r)),
            MirBinOp::Or => Some(ConstValue::Int(l | r)),
            MirBinOp::Div => None,
            MirBinOp::Mod => None,
        },
        (ConstValue::Float(l), ConstValue::Float(r)) => match op {
            MirBinOp::Add => Some(ConstValue::Float(l + r)),
            MirBinOp::Sub => Some(ConstValue::Float(l - r)),
            MirBinOp::Mul => Some(ConstValue::Float(l * r)),
            MirBinOp::Div => Some(ConstValue::Float(l / r)),
            MirBinOp::Eq => Some(ConstValue::Bool((l - r).abs() < f64::EPSILON)),
            MirBinOp::Neq => Some(ConstValue::Bool((l - r).abs() >= f64::EPSILON)),
            MirBinOp::Lt => Some(ConstValue::Bool(l < r)),
            MirBinOp::Gt => Some(ConstValue::Bool(l > r)),
            MirBinOp::Le => Some(ConstValue::Bool(l <= r)),
            MirBinOp::Ge => Some(ConstValue::Bool(l >= r)),
            _ => None,
        },
        (ConstValue::Bool(l), ConstValue::Bool(r)) => match op {
            MirBinOp::Eq => Some(ConstValue::Bool(l == r)),
            MirBinOp::Neq => Some(ConstValue::Bool(l != r)),
            MirBinOp::And => Some(ConstValue::Bool(l && r)),
            MirBinOp::Or => Some(ConstValue::Bool(l || r)),
            _ => None,
        },
        _ => None,
    }
}

fn fold_unary(op: MirUnaryOp, operand: ConstValue) -> Option<ConstValue> {
    match (op, operand) {
        (MirUnaryOp::Neg, ConstValue::Int(v)) => Some(ConstValue::Int(v.wrapping_neg())),
        (MirUnaryOp::Neg, ConstValue::Float(v)) => Some(ConstValue::Float(-v)),
        (MirUnaryOp::Not, ConstValue::Bool(v)) => Some(ConstValue::Bool(!v)),
        (MirUnaryOp::Not, ConstValue::Int(v)) => Some(ConstValue::Int(!v)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol_str::SmolStr;

    fn make_test_func() -> MirFunction {
        let mut cfg = ControlFlowGraph::new();
        let mut b0 = BasicBlock::new(0);

        let v0 = MirValue(0);
        let v1 = MirValue(1);
        let v2 = MirValue(2);
        let v3 = MirValue(3);

        b0.instructions = vec![
            MirInst::IntLiteral { dest: v0, val: 2 },
            MirInst::IntLiteral { dest: v1, val: 3 },
            MirInst::Binary {
                dest: v2,
                op: MirBinOp::Add,
                left: v0,
                right: v1,
            },
            MirInst::Binary {
                dest: v3,
                op: MirBinOp::Mul,
                left: v2,
                right: MirValue(4),
            },
        ];
        b0.terminator = MirTerminator::Return(Some(v3));

        cfg.blocks.push(b0);
        cfg.entry = 0;

        MirFunction {
            name: SmolStr::new("test"),
            params: vec![],
            ret_type: MirType::Int,
            cfg,
        }
    }

    #[test]
    fn test_const_fold_add() {
        let mut func = make_test_func();
        fold_constants(&mut func);

        let b0 = &func.cfg.blocks[0];
        // The add (v2 = 2 + 3) should be folded to IntLiteral(5)
        // Then the mul (v3 = 5 * 4) should be folded, but 4 wasn't defined in the function
        // The mul's right operand v4 doesn't exist, so it shouldn't crash but won't fold
        let has_folded_add = b0.instructions.iter().any(|inst| {
            matches!(inst, MirInst::IntLiteral { dest, val } if dest.0 == 2 && *val == 5)
        });
        assert!(has_folded_add, "Expected 2+3 to be folded to 5");
    }
}
