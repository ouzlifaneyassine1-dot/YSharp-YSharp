pub mod const_fold;
pub mod loop_opt;
pub mod vectorize;
pub mod inline;
pub mod reorder;

pub use const_fold::fold_constants;
pub use loop_opt::optimize_loops;
pub use vectorize::vectorize;
pub use reorder::optimize_layout;
/// Run all optimization passes on a module in order.
pub fn optimize(module: &mut crate::mir::MirModule) {
    for func in &mut module.functions {
        optimize_function(func);
    }
}

/// Run all optimization passes on a single function.
pub fn optimize_function(func: &mut crate::mir::MirFunction) {
    // Phase 1: Constant folding / propagation
    fold_constants(func);

    // Phase 2: Loop invariant code motion
    optimize_loops(func);

    // Phase 3: Auto-vectorization analysis
    vectorize(func);

    // Phase 4: Block reordering + loop rotation
    optimize_layout(func);
}
