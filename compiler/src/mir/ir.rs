use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Native,
    Wasm,
    Gpu,
    Game,
    Kernel,
    Server,
    Desktop,
    Mobile,
}

#[derive(Debug, Clone)]
pub struct MirModule {
    pub name: SmolStr,
    pub functions: Vec<MirFunction>,
    pub target: Target,
}

#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: SmolStr,
    pub params: Vec<MirType>,
    pub ret_type: MirType,
    pub cfg: ControlFlowGraph,
}

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub entry: usize,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        ControlFlowGraph {
            blocks: Vec::new(),
            entry: 0,
        }
    }

    pub fn add_block(&mut self, block: BasicBlock) -> usize {
        let id = self.blocks.len();
        self.blocks.push(block);
        id
    }

    pub fn block(&self, id: usize) -> &BasicBlock {
        &self.blocks[id]
    }

    pub fn block_mut(&mut self, id: usize) -> &mut BasicBlock {
        &mut self.blocks[id]
    }

    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn predecessors(&self, block_id: usize) -> Vec<usize> {
        let mut preds = Vec::new();
        for (i, block) in self.blocks.iter().enumerate() {
            match &block.terminator {
                MirTerminator::Branch(target) => {
                    if *target == block_id {
                        preds.push(i);
                    }
                }
                MirTerminator::CondBranch {
                    true_block,
                    false_block,
                    ..
                } => {
                    if *true_block == block_id || *false_block == block_id {
                        preds.push(i);
                    }
                }
                MirTerminator::Return(_) | MirTerminator::Unreachable => {}
            }
        }
        preds
    }

    pub fn successors(&self, block_id: usize) -> Vec<usize> {
        let block = &self.blocks[block_id];
        match &block.terminator {
            MirTerminator::Branch(target) => vec![*target],
            MirTerminator::CondBranch {
                true_block,
                false_block,
                ..
            } => vec![*true_block, *false_block],
            MirTerminator::Return(_) | MirTerminator::Unreachable => vec![],
        }
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<MirInst>,
    pub terminator: MirTerminator,
}

impl BasicBlock {
    pub fn new(id: usize) -> Self {
        BasicBlock {
            id,
            instructions: Vec::new(),
            terminator: MirTerminator::Unreachable,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MirInst {
    Alloca {
        name: SmolStr,
        ty: MirType,
        dest: MirValue,
    },
    Load {
        dest: MirValue,
        src: MirValue,
    },
    Store {
        dest: MirValue,
        src: MirValue,
    },
    Binary {
        dest: MirValue,
        op: MirBinOp,
        left: MirValue,
        right: MirValue,
    },
    Unary {
        dest: MirValue,
        op: MirUnaryOp,
        operand: MirValue,
    },
    Call {
        dest: Option<MirValue>,
        callee: SmolStr,
        args: Vec<MirValue>,
    },
    IntLiteral {
        dest: MirValue,
        val: i64,
    },
    FloatLiteral {
        dest: MirValue,
        val: f64,
    },
    StringLiteral {
        dest: MirValue,
        val: SmolStr,
    },
    BoolLiteral {
        dest: MirValue,
        val: bool,
    },
    Phi {
        dest: MirValue,
        incoming: Vec<(MirValue, usize)>,
    },
    /// Vectorization hint: marks a loop for SIMD codegen
    VectorHint {
        width: u32,
        ops: u32,
        trip_count: u64,
    },
    /// Print intrinsic: dispatches to the correct print function based on arg_type
    Print {
        dest: Option<MirValue>,
        arg: MirValue,
        arg_type: MirType,
        newline: bool,
    },
    /// Inline hint: marks a call site for inlining in the backend
    InlineHint {
        dest: Option<MirValue>,
        args: Vec<MirValue>,
    },
}

#[derive(Debug, Clone)]
pub enum MirTerminator {
    Branch(usize),
    CondBranch {
        cond: MirValue,
        true_block: usize,
        false_block: usize,
    },
    Return(Option<MirValue>),
    Unreachable,
}

/// SSA value reference: a unique index within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirValue(pub u32);

impl MirValue {
    pub fn new(id: u32) -> Self {
        MirValue(id)
    }
}

impl std::fmt::Display for MirValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirType {
    Int,
    Float,
    Bool,
    String,
    Ptr(Box<MirType>),
    Tensor(Box<MirType>, usize),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnaryOp {
    Neg,
    Not,
}
