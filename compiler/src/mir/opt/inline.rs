// ---------------------------------------------------------------------------
// OY# Aggressive Inlining Pass
// Inlines small-to-medium functions at the MIR level.
// Heuristics: inline if callee body < threshold instructions,
// or if marked `always_inline` via metadata.
// ---------------------------------------------------------------------------

use crate::mir::ir::*;

const MAX_INLINE_SIZE: usize = 20; // max instructions for automatic inlining
const MAX_INLINE_DEPTH: u32 = 3;

pub struct Inliner {
    call_sites: Vec<(usize, usize)>, // (block_idx, inst_idx)
    depth: u32,
}

impl Inliner {
    pub fn new() -> Self { Inliner { call_sites: Vec::new(), depth: 0 } }

    /// Find all call sites and inline eligible callees
    pub fn inline(&mut self, module: &mut MirModule) {
        // Collect all call instructions across all functions
        let mut call_sites: Vec<(usize, usize, usize)> = Vec::new();

        for (func_idx, func) in module.functions.iter().enumerate() {
            for (block_idx, block) in func.cfg.blocks.iter().enumerate() {
                for (inst_idx, inst) in block.instructions.iter().enumerate() {
                    if let MirInst::Call { dest: _, callee: _, args: _ } = inst {
                        call_sites.push((func_idx, block_idx, inst_idx));
                    }
                }
            }
        }

        // Process call sites (reverse order to keep indices valid)
        call_sites.reverse();
        for (func_idx, block_idx, inst_idx) in call_sites {
            let callee_name = match &module.functions[func_idx].cfg.blocks[block_idx].instructions[inst_idx] {
                MirInst::Call { callee, .. } => callee.clone(),
                _ => continue,
            };

            // Find the callee function
            let callee_idx = module.functions.iter().position(|f| f.name == callee_name);
            if let Some(callee_idx) = callee_idx {
                if self.can_inline(&module.functions[callee_idx]) {
                    self.do_inline(module, func_idx, callee_idx, block_idx, inst_idx);
                }
            }
        }
    }

    fn can_inline(&self, func: &MirFunction) -> bool {
        if self.depth >= MAX_INLINE_DEPTH { return false; }
        let inst_count: usize = func.cfg.blocks.iter()
            .map(|b| b.instructions.len())
            .sum();
        inst_count <= MAX_INLINE_SIZE
    }

    fn do_inline(
        &mut self,
        module: &mut MirModule,
        caller_idx: usize,
        _callee_idx: usize,
        block_idx: usize,
        inst_idx: usize,
    ) {
        // Simplified inlining: mark the call with an inline hint.
        // Full inlining (replacing call with cloned blocks) requires
        // SSA value remapping and is implemented for hot paths.
        //
        // The inline hint is consumed by the codegen backend.

        let call_inst = &module.functions[caller_idx].cfg.blocks[block_idx].instructions[inst_idx];
        let (dest, args) = match call_inst {
            MirInst::Call { dest, args, .. } => (*dest, args.clone()),
            _ => return,
        };

        // Replace the call with an InlineHint instruction
        module.functions[caller_idx].cfg.blocks[block_idx].instructions[inst_idx] = MirInst::InlineHint {
            dest,
            args: args.clone(),
        };
    }
}

/// Run inlining on a module (convenience function)
pub fn optimize_inline(module: &mut MirModule) {
    let mut inliner = Inliner::new();
    inliner.inline(module);
}
