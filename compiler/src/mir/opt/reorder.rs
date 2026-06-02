// ---------------------------------------------------------------------------
// OY# Basic Block Reordering + Loop Rotation Pass
// Optimizes branch layout for better CPU frontend throughput.
// - Hot blocks are contiguous (reduces I-cache misses)
// - Loop rotation converts while-loops to do-while where profitable
// - Cold blocks (unlikely branches) are moved to the end
// ---------------------------------------------------------------------------

use rustc_hash::{FxHashMap, FxHashSet};
use crate::mir::ir::*;

pub fn optimize_layout(func: &mut MirFunction) {
    // Loop rotation disabled — it incorrectly shares condition vreg
    // across blocks without moving computation.
    // rotate_loops(func);
    reorder_blocks(func);
}

/// Rotate loops: convert "branch to header, header cond-branch to body"
/// into "cond-branch to body or exit, body falls through to test".
/// This reduces branch mispredictions by making the common path linear.
fn rotate_loops(func: &mut MirFunction) {
    let n = func.cfg.blocks.len();
    if n < 2 { return; }

    // Find loops (back edges)
    let mut rotated = FxHashSet::default();
    for latch in 0..n {
        let term = func.cfg.blocks[latch].terminator.clone();
        match term {
            MirTerminator::Branch(header) if header <= latch && header != latch => {
                if rotated.contains(&header) { continue; }
                // Check: header has CondBranch to body and exit
                let header_term = func.cfg.blocks[header].terminator.clone();
                if let MirTerminator::CondBranch { cond, true_block, false_block } = header_term {
                    // Insert a copy of the condition at the end of the latch
                    // and make the latch branch directly to body or exit.
                    // For simplicity: if latch ends with Branch(header), change
                    // the latch's terminator to mirror the header's condition.
                    if header != func.cfg.entry {
                        func.cfg.blocks[latch].terminator = MirTerminator::CondBranch {
                            cond,
                            true_block,
                            false_block,
                        };
                        // The header now unconditionally branches to the body
                        func.cfg.blocks[header].terminator = MirTerminator::Branch(true_block);
                        rotated.insert(header);
                    }
                }
            }
            MirTerminator::CondBranch { true_block: _tb, false_block: _fb, .. } => {
                // Self-loop: latch branches back to itself
                // Nothing to rotate
            }
            _ => {}
        }
    }
}

/// Reorder blocks: place blocks in DFS post-order for better spatial locality.
/// Unlikely blocks (those ending in Branch to exit) are pushed to the end.
/// The entry block is always kept at index 0.
fn reorder_blocks(func: &mut MirFunction) {
    let n = func.cfg.blocks.len();
    if n <= 1 { return; }

    let entry = func.cfg.entry;

    // Compute block frequencies (simplified: use branch probabilities)
    let mut freq = FxHashMap::default();
    compute_freq(func, &mut freq);

    // Sort non-entry blocks by frequency (stable: prefer original order for equal freq)
    let mut non_entry: Vec<usize> = (0..n).filter(|&i| i != entry).collect();
    non_entry.sort_by(|&a, &b| {
        let fa = freq.get(&a).copied().unwrap_or(0.0);
        let fb = freq.get(&b).copied().unwrap_or(0.0);
        fb.partial_cmp(&fa).unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });

    // Build new block order: entry first, then sorted non-entry
    let mut old_blocks = Vec::new();
    core::mem::swap(&mut old_blocks, &mut func.cfg.blocks);

    let mut new_blocks = Vec::with_capacity(n);
    let mut indices = vec![entry];
    indices.extend(&non_entry);
    for &idx in &indices {
        new_blocks.push(old_blocks[idx].clone());
    }

    // Remap block references
    let mut remap = FxHashMap::default();
    for (new_idx, &old_idx) in indices.iter().enumerate() {
        remap.insert(old_idx, new_idx);
    }

    func.cfg.blocks = new_blocks;
    func.cfg.entry = remap.get(&entry).copied().unwrap_or(0);

    // Update block IDs to match new positions
    for (new_idx, block) in func.cfg.blocks.iter_mut().enumerate() {
        block.id = new_idx;
    }

    // Remap terminators
    for block in &mut func.cfg.blocks {
        remap_terminator(&mut block.terminator, &remap);
    }
}

fn compute_freq(func: &MirFunction, freq: &mut FxHashMap<usize, f32>) {
    let n = func.cfg.blocks.len();
    if n == 0 { return; }

    // Simple frequency estimation: BFS from entry, assume equal branch probability
    let mut work = vec![func.cfg.entry];
    let mut visited = FxHashSet::default();
    freq.insert(func.cfg.entry, 1.0);
    visited.insert(func.cfg.entry);

    while let Some(idx) = work.pop() {
        let cur_freq = *freq.get(&idx).unwrap_or(&0.0);
        let term = &func.cfg.blocks[idx].terminator;
        let succs = match term {
            MirTerminator::Branch(target) => vec![*target],
            MirTerminator::CondBranch { true_block, false_block, .. } => {
                vec![*true_block, *false_block]
            }
            MirTerminator::Return(_) | MirTerminator::Unreachable => vec![],
        };
        let edge_freq = if succs.is_empty() { 0.0 } else { cur_freq / succs.len() as f32 };
        for succ in succs {
            let entry = freq.entry(succ).or_insert(0.0);
            *entry += edge_freq;
            if visited.insert(succ) {
                work.push(succ);
            }
        }
    }
}

fn remap_terminator(term: &mut MirTerminator, remap: &FxHashMap<usize, usize>) {
    match term {
        MirTerminator::Branch(target) => {
            if let Some(&new) = remap.get(target) { *target = new; }
        }
        MirTerminator::CondBranch { true_block, false_block, .. } => {
            if let Some(&new) = remap.get(true_block) { *true_block = new; }
            if let Some(&new) = remap.get(false_block) { *false_block = new; }
        }
        MirTerminator::Return(_) | MirTerminator::Unreachable => {}
    }
}
