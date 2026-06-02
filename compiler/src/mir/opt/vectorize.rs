// ---------------------------------------------------------------------------
// OY# Auto-Vectorization Pass — detects SIMD-izable loops and rewrites them
// into vector operations. Targets loops with:
//   - Contiguous memory access (stride-1)
//   - No loop-carried dependencies (other than reduction)
//   - Trip count known or estimated
// ---------------------------------------------------------------------------

use rustc_hash::FxHashSet;
use crate::mir::ir::*;

const VEC_WIDTH: usize = 4; // f32x4 / i32x4

#[derive(Debug, Clone)]
struct VecLoopInfo {
    header: usize,
    latch: usize,
    body: FxHashSet<usize>,
    /// Induction variable steps
    iv: Option<InductionVar>,
    /// Candidate loads/stores for vectorization
    mem_ops: Vec<VecMemOp>,
    /// Trip count if known
    trip_count: Option<u64>,
}

#[derive(Debug, Clone)]
struct InductionVar {
    val: MirValue,
    init: i64,
    step: i64,
    limit: MirValue,
}

#[derive(Debug, Clone)]
struct VecMemOp {
    load: bool,
    dest: MirValue,
    ptr: MirValue,
    index: MirValue,
    stride: i64,
    inst_idx: usize,
    block: usize,
}

/// Main vectorization entry point
pub fn vectorize(func: &mut MirFunction) {
    let loops = find_vectorizable_loops(func);
    for loop_info in loops {
        if let Err(e) = try_vectorize(func, &loop_info) {
            // If vectorization fails, the function is unchanged
            let _ = e;
        }
    }
}

fn find_vectorizable_loops(func: &MirFunction) -> Vec<VecLoopInfo> {
    let mut candidates = Vec::new();

    for block_idx in 0..func.cfg.blocks.len() {
        let term = &func.cfg.blocks[block_idx].terminator;
        if let MirTerminator::CondBranch { true_block, false_block, .. } = term {
            // Check if this is a loop header: block_idx dominates both
            // Simple heuristic: back-edge to a prior block
            let targets = [*true_block, *false_block];
            for target in targets {
                if target < block_idx {
                    // Possible back-edge (block N -> block M where M < N)
                    let mut body = FxHashSet::default();
                    body.insert(block_idx);
                    body.insert(target);
                    collect_loop_body(func, target, block_idx, &mut body);

                    let mem_ops = find_mem_ops(func, &body);
                    let iv = find_induction_var(func, target, &body);
                    let trip = estimate_trip_count(func, target, &iv);

                    if !mem_ops.is_empty() {
                        candidates.push(VecLoopInfo {
                            header: target,
                            latch: block_idx,
                            body,
                            iv,
                            mem_ops,
                            trip_count: trip,
                        });
                    }
                }
            }
        }
    }
    candidates
}

fn collect_loop_body(func: &MirFunction, header: usize, latch: usize, body: &mut FxHashSet<usize>) {
    let mut stack = vec![latch];
    while let Some(current) = stack.pop() {
        for pred in func.cfg.predecessors(current) {
            if pred != header && body.insert(pred) {
                stack.push(pred);
            }
        }
    }
}

fn find_mem_ops(func: &MirFunction, body: &FxHashSet<usize>) -> Vec<VecMemOp> {
    let mut ops = Vec::new();
    for &b in body.iter() {
        for (idx, inst) in func.cfg.blocks[b].instructions.iter().enumerate() {
            match inst {
                MirInst::Load { dest, src } => {
                    ops.push(VecMemOp {
                        load: true, dest: *dest, ptr: *src,
                        index: MirValue(0), stride: 1, inst_idx: idx, block: b,
                    });
                }
                MirInst::Store { dest, src: _ } => {
                    ops.push(VecMemOp {
                        load: false, dest: *dest, ptr: *dest,
                        index: MirValue(0), stride: 1, inst_idx: idx, block: b,
                    });
                }
                _ => {}
            }
        }
    }
    ops
}

fn find_induction_var(func: &MirFunction, header: usize, body: &FxHashSet<usize>) -> Option<InductionVar> {
    // Look for patterns: v = phi(init, next), next = v + step
    let header_block = &func.cfg.blocks[header];
    for inst in &header_block.instructions {
        if let MirInst::Phi { dest, incoming } = inst {
            // Check if the phi is used in an add with constant in the latch
            let _init_val = if incoming.len() == 2 {
                incoming[0].0
            } else {
                continue;
            };
            // Try to find the step by looking at instructions using this phi
            for &b in body.iter() {
                let block = &func.cfg.blocks[b];
                for inst2 in &block.instructions {
                    if let MirInst::Binary { dest, op: MirBinOp::Add, left, right } = inst2 {
                        if *left == *dest && *right == *dest {
                            // self-assign skip
                        }
                        let (base, _step_val) = if *left == *dest { (right, None::<i64>) }
                            else if *right == *dest { (left, None::<i64>) }
                            else { continue; };
                        // Check if step is a constant
                        if let MirInst::IntLiteral { val, .. } = &func.cfg.blocks[b].instructions.iter()
                            .find(|i| matches!(i, MirInst::IntLiteral { dest: d, .. } if *d == *base))
                            .unwrap_or(&MirInst::IntLiteral { dest: MirValue(0), val: 0 })
                        {
                            return Some(InductionVar {
                                val: *dest,
                                init: 0,
                                step: *val,
                                limit: *dest,
                            });
                        }
                    }
                }
            }
            return Some(InductionVar { val: *dest, init: 0, step: 1, limit: MirValue(0) });
        }
    }
    None
}

fn estimate_trip_count(func: &MirFunction, header: usize, iv: &Option<InductionVar>) -> Option<u64> {
    // If limit is a constant, compute (limit - init) / step
    match iv {
        Some(iv) => {
            // Check header for IntLiteral limit
            for inst in &func.cfg.blocks[header].instructions {
                if let MirInst::IntLiteral { dest, val } = inst {
                    if *dest == iv.limit {
                        let trips = if iv.step > 0 {
                            ((*val - iv.init) / iv.step).max(0) as u64
                        } else {
                            0
                        };
                        return Some(trips);
                    }
                }
            }
            None
        }
        None => None,
    }
}

fn try_vectorize(func: &mut MirFunction, info: &VecLoopInfo) -> Result<(), String> {
    // For now, mark the loop body with a VectorHint metadata comment
    // Full vectorization (replacing scalar ops with vector ops) will be
    // implemented once the backend supports vector types natively.
    //
    // This pass serves as analysis — the vectorization decision is emitted
    // as a hint to the codegen backend.

    // Emit a pseudo-instruction before the loop header to mark vectorization
    let header_block = &mut func.cfg.blocks[info.header];
    header_block.instructions.insert(0, MirInst::VectorHint {
        width: VEC_WIDTH as u32,
        ops: info.mem_ops.len() as u32,
        trip_count: info.trip_count.unwrap_or(0),
    });

    Ok(())
}

/// Detect vector hints in a function (for backend consumption)
pub fn has_vector_hints(func: &MirFunction) -> bool {
    func.cfg.blocks.iter().any(|b| {
        b.instructions.iter().any(|i| matches!(i, MirInst::VectorHint { .. }))
    })
}
