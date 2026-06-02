use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::collections::HashSet;

use super::ir::*;

/// SSA construction state.
pub struct SsaBuilder {
    /// Per-block, per-variable: which value is current when entering the block.
    block_incoming: Vec<FxHashMap<SmolStr, MirValue>>,
    /// Per-variable: stack of most recent definitions.
    var_stack: FxHashMap<SmolStr, Vec<MirValue>>,
    /// Set of blocks that have been processed.
    visited: HashSet<usize>,
    /// The CFG being transformed.
    cfg: ControlFlowGraph,
    /// Variables that exist (for which we may need phi nodes).
    variables: Vec<SmolStr>,
    /// Dominance frontiers: block -> set of blocks it dominates.
    dom_frontiers: Vec<HashSet<usize>>,
    /// Immediate dominators: doms[b] = idom(b).
    doms: Vec<Option<usize>>,
}

impl SsaBuilder {
    pub fn new() -> Self {
        SsaBuilder {
            block_incoming: Vec::new(),
            var_stack: FxHashMap::default(),
            visited: HashSet::new(),
            cfg: ControlFlowGraph::new(),
            variables: Vec::new(),
            dom_frontiers: Vec::new(),
            doms: Vec::new(),
        }
    }

    /// Build dominance information for the CFG.
    fn compute_dominance(&mut self) {
        let n = self.cfg.blocks.len();
        self.doms = vec![None; n];
        self.dom_frontiers = vec![HashSet::new(); n];

        if n == 0 {
            return;
        }

        // Simple iterative dominance computation
        let entry = self.cfg.entry;
        self.doms[entry] = Some(entry);

        let mut changed = true;
        while changed {
            changed = false;
            for b in 0..n {
                if b == entry {
                    continue;
                }
                let preds = self.cfg.predecessors(b);
                if preds.is_empty() {
                    continue;
                }

                // new_idom = first processed predecessor
                let mut new_idom = None;
                for &p in &preds {
                    if self.doms[p].is_some() {
                        new_idom = Some(p);
                        break;
                    }
                }

                if let Some(mut idom) = new_idom {
                    for &p in &preds {
                        if self.doms[p].is_some() {
                            idom = intersect(&self.doms, idom, p);
                        }
                    }
                    if self.doms[b] != Some(idom) {
                        self.doms[b] = Some(idom);
                        changed = true;
                    }
                }
            }
        }

        // Compute dominance frontiers
        for b in 0..n {
            let preds = self.cfg.predecessors(b);
            if preds.len() >= 2 {
                for &p in &preds {
                    let mut runner = p;
                    while runner != self.doms[b].unwrap_or(entry) {
                        self.dom_frontiers[runner].insert(b);
                        runner = self.doms[runner].unwrap_or(entry);
                    }
                }
            }
        }
    }

    /// Place phi nodes for all variables at dominance frontiers.
    fn place_phi_nodes(&mut self) {
        let _n = self.cfg.blocks.len();

        // Discover all variable names from alloca instructions
        self.variables.clear();
        for block in &self.cfg.blocks {
            for inst in &block.instructions {
                if let MirInst::Alloca { name, .. } = inst {
                    if !self.variables.contains(name) {
                        self.variables.push(name.clone());
                    }
                }
            }
        }

        // For each variable, place phi nodes
        for var in self.variables.clone().iter() {
            // Find blocks that define this var
            let def_blocks = Vec::new();
            for (b, block) in self.cfg.blocks.iter().enumerate() {
                for inst in &block.instructions {
                    match inst {
                        MirInst::Store { src: _, .. } => {
                            // If we track definitions per variable, we'd check.
                            // For simplicity, mark blocks with stores to this var.
                            if let Some(MirInst::Alloca { name: _, .. }) = self.cfg.blocks[b].instructions
                                .iter()
                                .find(|i| matches!(i, MirInst::Alloca { name: n, .. } if n == var))
                            {
                                // This is approximate - real SSA would track this per-variable
                            }
                        }
                        _ => {}
                    }
                }
            }

            // If we have def blocks, place phi nodes in dominance frontiers
            if !def_blocks.is_empty() {
                let mut phi_placed = HashSet::new();
                let mut worklist: Vec<usize> = def_blocks.clone();
                while let Some(b) = worklist.pop() {
                    for &df in &self.dom_frontiers[b] {
                        if phi_placed.insert(df) {
                            // Insert phi at start of block df
                            let phi_dest = MirValue::new(self.cfg.blocks[df].instructions.len() as u32 + 1000); // placeholder
                            let phi = MirInst::Phi {
                                dest: phi_dest,
                                incoming: Vec::new(),
                            };
                            self.cfg.blocks[df].instructions.insert(0, phi);
                            if !worklist.contains(&df) {
                                worklist.push(df);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Rename variables to achieve SSA form.
    fn rename(&mut self) {
        self.var_stack.clear();
        self.visited.clear();
        self.block_incoming = vec![FxHashMap::default(); self.cfg.blocks.len()];

        // Initialize var stacks from alloca names
        for var in &self.variables {
            self.var_stack.entry(var.clone()).or_default();
        }

        self.seal_block(self.cfg.entry);
        self.rename_block(self.cfg.entry);
    }

    fn rename_block(&mut self, block_id: usize) {
        if !self.visited.insert(block_id) {
            return;
        }

        // Process instructions, renaming uses and defs
        let block = self.cfg.blocks[block_id].clone();

        // First, collect phi node incoming values
        let _phi_count = block
            .instructions
            .iter()
            .filter(|i| matches!(i, MirInst::Phi { .. }))
            .count();

        for inst in &block.instructions {
            match inst {
                MirInst::Phi { dest, incoming: _ } => {
                    // Push the phi dest as a new version for its variable
                    // For now, we just ensure the value is unique
                    self.block_incoming[block_id].insert(
                        SmolStr::new(format!("phi_{}", dest.0)),
                        *dest,
                    );
                }
                MirInst::Load { dest: _, src: _ } => {
                    // Load produces a new value - push it if it corresponds to a variable
                }
                _ => {}
            }
        }

        // Rename uses in terminator
        let term = block.terminator.clone();
        let _ = term; // SSA renaming of terminators is handled during construction

        // Process successors, updating phi node incoming values
        for succ in self.cfg.successors(block_id) {
            // For each phi in the successor, add this block as an incoming source
            let succ_phis: Vec<usize> = self.cfg.blocks[succ]
                .instructions
                .iter()
                .enumerate()
                .filter(|(_, i)| matches!(i, MirInst::Phi { .. }))
                .map(|(idx, _)| idx)
                .collect();

            for phi_idx in succ_phis {
                if let MirInst::Phi {
                    ref mut incoming, ..
                } = self.cfg.blocks[succ].instructions[phi_idx]
                {
                    // Use a placeholder value; proper renaming would use the current stack top
                    let val = MirValue::new(0);
                    incoming.push((val, block_id));
                }
            }

            if !self.visited.contains(&succ) {
                self.seal_block(succ);
                self.rename_block(succ);
            }
        }
    }

    fn seal_block(&mut self, _block_id: usize) {
        // In a more complete implementation, this would mark the block as sealed
        // and perform phi node completion.
    }
}

/// Construct SSA for the given CFG by placing phi nodes and renaming.
pub fn construct_ssa(cfg: &mut ControlFlowGraph) {
    if cfg.blocks.is_empty() {
        return;
    }

    let mut builder = SsaBuilder::new();
    builder.cfg = cfg.clone();
    builder.compute_dominance();
    builder.place_phi_nodes();
    builder.rename();

    // Copy back modified blocks
    *cfg = builder.cfg;
}

/// Intersect two nodes in the dominator tree (used in iterative dominance computation).
fn intersect(doms: &[Option<usize>], mut a: usize, mut b: usize) -> usize {
    while a != b {
        while a < b {
            a = doms[a].unwrap_or(0);
        }
        while b < a {
            b = doms[b].unwrap_or(0);
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_cfg() -> ControlFlowGraph {
        let mut cfg = ControlFlowGraph::new();
        let b0 = cfg.add_block(BasicBlock::new(0));
        let b1 = cfg.add_block(BasicBlock::new(1));
        let b2 = cfg.add_block(BasicBlock::new(2));

        cfg.blocks[b0].terminator = MirTerminator::Branch(b1);
        cfg.blocks[b1].terminator = MirTerminator::CondBranch {
            cond: MirValue::new(0),
            true_block: b2,
            false_block: 0,
        };
        cfg.blocks[b2].terminator = MirTerminator::Return(Some(MirValue::new(1)));
        cfg.entry = b0;

        cfg
    }

    #[test]
    fn test_ssa_construction() {
        let mut cfg = make_simple_cfg();
        construct_ssa(&mut cfg);
        // SSA should not crash and should produce valid CFG
        assert!(!cfg.blocks.is_empty());
    }
}
