use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::hir::{
    HirBinOp, HirFunction, HirNode, HirType, HirUnaryOp,
};

use super::ir::*;

/// Stateful builder that converts a HIR function into MIR (CFG form).
pub struct MirBuilder {
    /// Next available value id.
    next_val: u32,
    /// Variable name → current SSA value (stack of versions).
    vars: FxHashMap<SmolStr, Vec<MirValue>>,
    /// Variable name → declared MIR type.
    var_types: FxHashMap<SmolStr, MirType>,
    /// Current basic block being built.
    current_block: usize,
    /// The CFG under construction.
    cfg: ControlFlowGraph,
    /// Block stack for control flow (loops, if-else).
    break_targets: Vec<usize>,
    continue_targets: Vec<usize>,
    /// Known function return types for local functions.
    fn_ret_types: FxHashMap<SmolStr, MirType>,
}

impl MirBuilder {
    pub fn new() -> Self {
        MirBuilder {
            next_val: 0,
            vars: FxHashMap::default(),
            var_types: FxHashMap::default(),
            current_block: 0,
            cfg: ControlFlowGraph::new(),
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            fn_ret_types: FxHashMap::default(),
        }
    }

    fn fresh_val(&mut self) -> MirValue {
        let v = self.next_val;
        self.next_val += 1;
        MirValue(v)
    }

    fn emit(&mut self, inst: MirInst) {
        self.cfg.blocks[self.current_block]
            .instructions
            .push(inst);
    }

    fn set_terminator(&mut self, term: MirTerminator) {
        self.cfg.blocks[self.current_block].terminator = term;
    }

    fn new_block(&mut self) -> usize {
        let id = self.cfg.blocks.len();
        self.cfg.blocks.push(BasicBlock::new(id));
        id
    }

    fn switch_to_block(&mut self, id: usize) {
        self.current_block = id;
    }

    fn declare_var(&mut self, name: &SmolStr, ty: &MirType) -> MirValue {
        let ptr = self.fresh_val();
        self.emit(MirInst::Alloca {
            name: name.clone(),
            ty: ty.clone(),
            dest: ptr,
        });
        self.var_types.insert(name.clone(), ty.clone());
        // alloca produces a pointer value; we store it in the var map
        self.vars.entry(name.clone()).or_default().push(ptr);
        ptr
    }

    fn write_var(&mut self, name: &SmolStr, val: MirValue) {
        // Store to the alloca'd pointer
        let ptr = self
            .vars
            .get(name)
            .and_then(|v| v.last().copied())
            .expect("variable not declared");
        self.emit(MirInst::Store {
            dest: ptr,
            src: val,
        });
    }

    fn read_var(&mut self, name: &SmolStr) -> MirValue {
        let ptr = self
            .vars
            .get(name)
            .and_then(|v| v.last().copied())
            .expect("variable not declared");
        let val = self.fresh_val();
        self.emit(MirInst::Load {
            dest: val,
            src: ptr,
        });
        val
    }

    /// Infer the MIR type of a HirNode (for built-in dispatch).
    fn infer_hir_type(&self, node: &HirNode) -> MirType {
        match node {
            HirNode::StringLiteral(_) => MirType::String,
            HirNode::IntLiteral(_) => MirType::Int,
            HirNode::FloatLiteral(_) => MirType::Float,
            HirNode::BoolLiteral(_) => MirType::Bool,
            HirNode::Identifier(name) => {
                self.var_types.get(name).cloned().or_else(|| {
                    self.fn_ret_types.get(name).cloned()
                }).unwrap_or(MirType::Int)
            }
            HirNode::Call { callee, args: _ } => {
                self.fn_ret_types.get(callee).cloned().unwrap_or(MirType::Int)
            }
            HirNode::Binary { op, left, right } => {
                let left_type = self.infer_hir_type(left);
                let right_type = self.infer_hir_type(right);
                match op {
                    HirBinOp::Add => {
                        if left_type == MirType::String || right_type == MirType::String {
                            MirType::String
                        } else if left_type == MirType::Float || right_type == MirType::Float {
                            MirType::Float
                        } else {
                            MirType::Int
                        }
                    }
                    HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod => {
                        if left_type == MirType::Float || right_type == MirType::Float {
                            MirType::Float
                        } else {
                            MirType::Int
                        }
                    }
                    HirBinOp::Eq | HirBinOp::Neq | HirBinOp::Lt | HirBinOp::Gt
                    | HirBinOp::Le | HirBinOp::Ge | HirBinOp::And | HirBinOp::Or => MirType::Bool,
                }
            }
            HirNode::Unary { op, operand } => {
                let operand_type = self.infer_hir_type(operand);
                match op {
                    HirUnaryOp::Neg => {
                        if operand_type == MirType::Float { MirType::Float } else { MirType::Int }
                    }
                    HirUnaryOp::Not => MirType::Bool,
                }
            }
            _ => MirType::Int,
        }
    }

    /// Lower a full HirFunction to a MirFunction.
    pub fn lower(&mut self, hir_fn: &HirFunction) -> MirFunction {
        // Register this function's return type (for recursive calls)
        self.fn_ret_types.insert(hir_fn.name.clone(), lower_type(&hir_fn.ret_type));

        // Create entry block
        let entry = self.new_block();
        self.switch_to_block(entry);
        self.cfg.entry = entry;

        // Declare parameters
        let mut mir_params = Vec::with_capacity(hir_fn.params.len());
        for (i, param) in hir_fn.params.iter().enumerate() {
            let mir_ty = lower_type(&param.ty);
            let ptr = self.declare_var(&param.name, &mir_ty);
            // Load the actual parameter value from the C function param pN
            let param_val = self.fresh_val();
            self.emit(MirInst::Param {
                dest: param_val,
                index: i as u32,
            });
            self.emit(MirInst::Store {
                dest: ptr,
                src: param_val,
            });
            mir_params.push(mir_ty);
        }

        // Lower body
        self.lower_nodes(&hir_fn.body);

        // If the last block doesn't have a terminator, add Return
        {
            let last_block = self.current_block;
            if matches!(
                self.cfg.blocks[last_block].terminator,
                MirTerminator::Unreachable
            ) {
                self.set_terminator(MirTerminator::Return(None));
            }
        }

        let mir_ret = lower_type(&hir_fn.ret_type);

        MirFunction {
            name: hir_fn.name.clone(),
            params: mir_params,
            ret_type: mir_ret,
            cfg: self.cfg.clone(),
        }
    }

    fn lower_nodes(&mut self, nodes: &[HirNode]) {
        for node in nodes {
            self.lower_node(node);
        }
    }

    fn lower_node(&mut self, node: &HirNode) -> Option<MirValue> {
        match node {
            HirNode::Block(stmts) => {
                let mut last_val = None;
                for stmt in stmts {
                    last_val = self.lower_node(stmt);
                }
                last_val
            }

            HirNode::IntLiteral(val) => {
                let dest = self.fresh_val();
                self.emit(MirInst::IntLiteral {
                    dest,
                    val: *val,
                });
                Some(dest)
            }

            HirNode::FloatLiteral(val) => {
                let dest = self.fresh_val();
                self.emit(MirInst::FloatLiteral {
                    dest,
                    val: *val,
                });
                Some(dest)
            }

            HirNode::StringLiteral(val) => {
                let dest = self.fresh_val();
                self.emit(MirInst::StringLiteral {
                    dest,
                    val: val.clone(),
                });
                Some(dest)
            }

            HirNode::BoolLiteral(val) => {
                let dest = self.fresh_val();
                self.emit(MirInst::BoolLiteral {
                    dest,
                    val: *val,
                });
                Some(dest)
            }

            HirNode::Identifier(name) => Some(self.read_var(name)),

            HirNode::Null => None,

            HirNode::Binary {
                op,
                left,
                right,
            } => {
                let l = self.lower_node(left).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });
                let r = self.lower_node(right).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });
                let dest = self.fresh_val();
                self.emit(MirInst::Binary {
                    dest,
                    op: lower_binop(*op),
                    left: l,
                    right: r,
                });
                Some(dest)
            }

            HirNode::Unary { op, operand } => {
                let opnd = self.lower_node(operand).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });
                let dest = self.fresh_val();
                self.emit(MirInst::Unary {
                    dest,
                    op: lower_unaryop(*op),
                    operand: opnd,
                });
                Some(dest)
            }

            HirNode::Call { callee, args } => {
                let mir_args: Vec<MirValue> = args
                    .iter()
                    .map(|a| self.lower_node(a).unwrap_or_else(|| {
                        let d = self.fresh_val();
                        self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                        d
                    }))
                    .collect();
                let dest = self.fresh_val();
                if callee == "Print" || callee == "PrintLine" {
                    let arg_type = self.infer_hir_type(&args[0]);
                    self.emit(MirInst::Print {
                        dest: Some(dest),
                        arg: mir_args[0],
                        arg_type,
                        newline: callee == "PrintLine",
                    });
                } else {
                    self.emit(MirInst::Call {
                        dest: Some(dest),
                        callee: callee.clone(),
                        args: mir_args,
                    });
                }
                Some(dest)
            }

            HirNode::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.lower_node(object).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });
                let mut mir_args = vec![obj];
                for a in args {
                    mir_args.push(self.lower_node(a).unwrap_or_else(|| {
                        let d = self.fresh_val();
                        self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                        d
                    }));
                }
                let dest = self.fresh_val();
                self.emit(MirInst::Call {
                    dest: Some(dest),
                    callee: method.clone(),
                    args: mir_args,
                });
                Some(dest)
            }

            HirNode::FieldAccess { object, field: _ } => {
                self.lower_node(object)
            }

            HirNode::Index { object, index } => {
                let obj = self.lower_node(object);
                self.lower_node(index);
                obj
            }

            HirNode::Let {
                name,
                ty,
                init,
                mutable: _,
            } => {
                let mir_ty = if *ty == HirType::Infer {
                    if let Some(init_node) = init {
                        self.infer_hir_type(init_node)
                    } else {
                        MirType::Int
                    }
                } else {
                    lower_type(ty)
                };
                self.declare_var(name, &mir_ty);
                if let Some(init) = init {
                    let val = self.lower_node(init).unwrap_or_else(|| {
                        let d = self.fresh_val();
                        self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                        d
                    });
                    self.write_var(name, val);
                }
                None
            }

            HirNode::Assign { target, value } => {
                // Target must be an identifier for now
                let val = self.lower_node(value).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });
                if let HirNode::Identifier(name) = target.as_ref() {
                    self.write_var(name, val);
                }
                None
            }

            HirNode::If {
                cond,
                then_body,
                else_body,
            } => {
                let cond_val = self.lower_node(cond).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::BoolLiteral {
                        dest: d,
                        val: false,
                    });
                    d
                });

                let then_block = self.new_block();
                let else_block = self.new_block();
                let merge_block = self.new_block();

                self.set_terminator(MirTerminator::CondBranch {
                    cond: cond_val,
                    true_block: then_block,
                    false_block: else_block,
                });

                // Then branch
                self.switch_to_block(then_block);
                self.lower_nodes(then_body);
                if matches!(
                    self.cfg.blocks[self.current_block].terminator,
                    MirTerminator::Unreachable
                ) {
                    self.set_terminator(MirTerminator::Branch(merge_block));
                }

                // Else branch
                self.switch_to_block(else_block);
                self.lower_nodes(else_body);
                if matches!(
                    self.cfg.blocks[self.current_block].terminator,
                    MirTerminator::Unreachable
                ) {
                    self.set_terminator(MirTerminator::Branch(merge_block));
                }

                // Merge block
                self.switch_to_block(merge_block);
                None
            }

            HirNode::While { cond, body } => {
                let header_block = self.new_block();
                let body_block = self.new_block();
                let exit_block = self.new_block();

                // Branch from current block to header
                self.set_terminator(MirTerminator::Branch(header_block));

                // Header: evaluate condition
                self.switch_to_block(header_block);
                let cond_val = self.lower_node(cond).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::BoolLiteral {
                        dest: d,
                        val: false,
                    });
                    d
                });
                self.set_terminator(MirTerminator::CondBranch {
                    cond: cond_val,
                    true_block: body_block,
                    false_block: exit_block,
                });

                // Body
                self.break_targets.push(exit_block);
                self.continue_targets.push(header_block);
                self.switch_to_block(body_block);
                self.lower_nodes(body);
                // Branch back to header
                if matches!(
                    self.cfg.blocks[self.current_block].terminator,
                    MirTerminator::Unreachable
                ) {
                    self.set_terminator(MirTerminator::Branch(header_block));
                }
                self.break_targets.pop();
                self.continue_targets.pop();

                // Exit
                self.switch_to_block(exit_block);
                None
            }

            HirNode::Loop {
                var,
                from,
                to,
                body,
            } => {
                // for var = from to to { body }
                // Initial: var = from
                let from_val = self.lower_node(from).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });
                let to_val = self.lower_node(to).unwrap_or_else(|| {
                    let d = self.fresh_val();
                    self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                    d
                });

                let mir_ty = MirType::Int;
                self.declare_var(var, &mir_ty);
                self.write_var(var, from_val);

                let test_block = self.new_block();
                let body_block = self.new_block();
                let exit_block = self.new_block();

                self.set_terminator(MirTerminator::Branch(test_block));

                // Test: var <= to
                self.switch_to_block(test_block);
                let var_val = self.read_var(var);
                let cond = self.fresh_val();
                self.emit(MirInst::Binary {
                    dest: cond,
                    op: MirBinOp::Le,
                    left: var_val,
                    right: to_val,
                });
                self.set_terminator(MirTerminator::CondBranch {
                    cond,
                    true_block: body_block,
                    false_block: exit_block,
                });

                // Body
                self.break_targets.push(exit_block);
                self.continue_targets.push(test_block);
                self.switch_to_block(body_block);
                self.lower_nodes(body);

                // Increment var and go back to test
                let var_cur = self.read_var(var);
                let one = self.fresh_val();
                self.emit(MirInst::IntLiteral { dest: one, val: 1 });
                let var_next = self.fresh_val();
                self.emit(MirInst::Binary {
                    dest: var_next,
                    op: MirBinOp::Add,
                    left: var_cur,
                    right: one,
                });
                self.write_var(var, var_next);

                if matches!(
                    self.cfg.blocks[self.current_block].terminator,
                    MirTerminator::Unreachable
                ) {
                    self.set_terminator(MirTerminator::Branch(test_block));
                }
                self.break_targets.pop();
                self.continue_targets.pop();

                self.switch_to_block(exit_block);
                None
            }

            HirNode::Return(expr) => {
                let val = expr
                    .as_ref()
                    .and_then(|e| self.lower_node(e));
                self.set_terminator(MirTerminator::Return(val));

                // Create a new unreachable block for subsequent code
                let dead_block = self.new_block();
                self.switch_to_block(dead_block);
                None
            }

            HirNode::Array(elements) => {
                // Lower all elements; the last value is the "result"
                let mut last = None;
                for el in elements {
                    last = self.lower_node(el);
                }
                last
            }

            HirNode::TensorLiteral { dims, elements } => {
                for d in dims {
                    self.lower_node(d);
                }
                for el in elements {
                    self.lower_node(el);
                }
                None
            }

            HirNode::EntityDef { .. } | HirNode::SystemDef { .. } => {
                // For systems, lower the body
                if let HirNode::SystemDef { body, .. } = node {
                    self.lower_nodes(body);
                }
                // EntityDef is a declaration; no runtime code
                None
            }

            HirNode::OnHandler { body, .. } => {
                self.lower_nodes(body);
                None
            }

            HirNode::View { children } => {
                for child in children {
                    self.lower_node(child);
                }
                None
            }

            HirNode::StateDecl { name, ty, init } => {
                let mir_ty = if *ty == HirType::Infer {
                    init.as_ref().map(|n| self.infer_hir_type(n)).unwrap_or(MirType::Int)
                } else {
                    lower_type(ty)
                };
                self.declare_var(name, &mir_ty);
                if let Some(init) = init {
                    let val = self.lower_node(init).unwrap_or_else(|| {
                        let d = self.fresh_val();
                        self.emit(MirInst::IntLiteral { dest: d, val: 0 });
                        d
                    });
                    self.write_var(name, val);
                }
                None
            }
        }
    }
}

/// Lower a HirType to MirType.
fn lower_type(ty: &HirType) -> MirType {
    match ty {
        HirType::Int => MirType::Int,
        HirType::Float => MirType::Float,
        HirType::Bool => MirType::Bool,
        HirType::String => MirType::String,
        HirType::Null => MirType::Int,
        HirType::Tensor { element, dims } => {
            MirType::Tensor(Box::new(lower_type(element)), *dims)
        }
        HirType::Function { .. } => MirType::Int,
        HirType::Optional(inner) => MirType::Ptr(Box::new(lower_type(inner))),
        HirType::Named(_) => MirType::Int,
        HirType::Infer => MirType::Int,
    }
}

fn lower_binop(op: HirBinOp) -> MirBinOp {
    match op {
        HirBinOp::Add => MirBinOp::Add,
        HirBinOp::Sub => MirBinOp::Sub,
        HirBinOp::Mul => MirBinOp::Mul,
        HirBinOp::Div => MirBinOp::Div,
        HirBinOp::Eq => MirBinOp::Eq,
        HirBinOp::Neq => MirBinOp::Neq,
        HirBinOp::Lt => MirBinOp::Lt,
        HirBinOp::Gt => MirBinOp::Gt,
        HirBinOp::Le => MirBinOp::Le,
        HirBinOp::Ge => MirBinOp::Ge,
        HirBinOp::And => MirBinOp::And,
        HirBinOp::Or => MirBinOp::Or,
        HirBinOp::Mod => MirBinOp::Mod,
    }
}

fn lower_unaryop(op: HirUnaryOp) -> MirUnaryOp {
    match op {
        HirUnaryOp::Neg => MirUnaryOp::Neg,
        HirUnaryOp::Not => MirUnaryOp::Not,
    }
}

/// Lower a complete HIR function to MIR.
pub fn lower_function(hir_fn: &HirFunction, fn_ret_map: &FxHashMap<SmolStr, MirType>) -> MirFunction {
    let mut builder = MirBuilder::new();
    builder.fn_ret_types = fn_ret_map.clone();
    builder.lower(hir_fn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::ir::HirType;

    fn make_simple_fn() -> HirFunction {
        HirFunction {
            name: SmolStr::new("test"),
            params: vec![],
            ret_type: HirType::Int,
            body: vec![
                HirNode::IntLiteral(42),
            ],
            is_async: false,
            is_differentiable: false,
        }
    }

    #[test]
    fn test_lower_simple_function() {
        let hir_fn = make_simple_fn();
        let mut map = FxHashMap::default();
        map.insert(hir_fn.name.clone(), MirType::Int);
        let mir_fn = lower_function(&hir_fn, &map);
        assert_eq!(mir_fn.name.as_str(), "test");
        assert!(mir_fn.cfg.entry == 0);
        assert!(!mir_fn.cfg.blocks.is_empty());
    }

    #[test]
    fn test_lower_if_else() {
        let hir_fn = HirFunction {
            name: SmolStr::new("test_if"),
            params: vec![],
            ret_type: HirType::Int,
            body: vec![HirNode::If {
                cond: Box::new(HirNode::BoolLiteral(true)),
                then_body: vec![HirNode::IntLiteral(1)],
                else_body: vec![HirNode::IntLiteral(2)],
            }],
            is_async: false,
            is_differentiable: false,
        };
        let mut map = FxHashMap::default();
        map.insert(hir_fn.name.clone(), MirType::Int);
        let mir_fn = lower_function(&hir_fn, &map);
        assert!(mir_fn.cfg.blocks.len() >= 4); // entry, then, else, merge, plus entry body
    }
}
