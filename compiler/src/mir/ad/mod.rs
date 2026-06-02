use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::mir::ir::*;

/// Auto-differentiation pass for MIR functions.
///
/// This is a structural placeholder that prepares the infrastructure
/// for automatic differentiation of Y# functions.
#[derive(Debug, Clone)]
pub struct AutodiffPass {
    /// Seed derivatives for inputs.
    seeds: FxHashMap<SmolStr, MirValue>,
    /// Maximum number of derivatives to track.
    max_derivatives: usize,
}

impl AutodiffPass {
    pub fn new() -> Self {
        AutodiffPass {
            seeds: FxHashMap::default(),
            max_derivatives: 1,
        }
    }

    /// Configure a seed derivative for a parameter.
    pub fn with_seed(mut self, param: SmolStr, seed: MirValue) -> Self {
        self.seeds.insert(param, seed);
        self
    }

    /// Set the maximum number of derivatives to track.
    pub fn with_max_derivatives(mut self, max: usize) -> Self {
        self.max_derivatives = max;
        self
    }

    /// Run the auto-diff pass on a MIR function.
    ///
    /// Currently returns a stub function that records the differentiation
    /// request. Actual derivative computation will be implemented in future
    /// versions.
    pub fn run(&self, func: &MirFunction) -> MirFunction {
        // For each differentiable parameter, we would:
        // 1. Create a shadow MIR function for the adjoint (reverse-mode)
        // 2. Walk the CFG backward, accumulating gradients
        // 3. Generate gradient update code
        //
        // For now, emit a differentiated function with a note in the name.

        let diff_name = SmolStr::new(format!("{}_grad", func.name));

        let mut cfg = ControlFlowGraph::new();
        let mut entry = BasicBlock::new(0);

        // Create a placeholder result value
        let result = MirValue(0);
        entry
            .instructions
            .push(MirInst::FloatLiteral {
                dest: result,
                val: 0.0,
            });
        entry.terminator = MirTerminator::Return(Some(result));
        cfg.blocks.push(entry);
        cfg.entry = 0;

        MirFunction {
            name: diff_name,
            params: func.params.clone(),
            ret_type: MirType::Float, // Gradients are typically floats
            cfg,
        }
    }
}

impl Default for AutodiffPass {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a differentiated version of a function.
pub fn differentiate(func: &MirFunction) -> MirFunction {
    let pass = AutodiffPass::new();
    pass.run(func)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol_str::SmolStr;

    fn make_differentiable_func() -> MirFunction {
        let mut cfg = ControlFlowGraph::new();
        let mut b0 = BasicBlock::new(0);

        let x = MirValue(0);
        let y = MirValue(1);
        let z = MirValue(2);

        b0.instructions = vec![
            MirInst::IntLiteral { dest: x, val: 3 },
            MirInst::IntLiteral { dest: y, val: 4 },
            MirInst::Binary {
                dest: z,
                op: MirBinOp::Mul,
                left: x,
                right: y,
            },
        ];
        b0.terminator = MirTerminator::Return(Some(z));

        cfg.blocks.push(b0);
        cfg.entry = 0;

        MirFunction {
            name: SmolStr::new("f"),
            params: vec![MirType::Int, MirType::Int],
            ret_type: MirType::Int,
            cfg,
        }
    }

    #[test]
    fn test_autodiff_pass() {
        let func = make_differentiable_func();
        let grad = differentiate(&func);
        assert!(grad.name.as_str().contains("_grad"));
        assert_eq!(grad.params.len(), 2);
    }
}
