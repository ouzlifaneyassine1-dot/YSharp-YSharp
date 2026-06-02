use smol_str::SmolStr;

pub type AstId = u32;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: SmolStr,
    pub type_expr: Option<TypeExpr>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Program { name: SmolStr, body: Vec<AstId> },
    Function {
        name: SmolStr,
        params: Vec<Param>,
        ret_type: Option<TypeExpr>,
        body: Vec<AstId>,
        is_async: bool,
        is_differentiable: bool,
    },
    Param { name: SmolStr, type_expr: Option<TypeExpr> },
    Block(Vec<AstId>),
    If { cond: AstId, then_block: AstId, else_block: Option<AstId> },
    While { cond: AstId, body: AstId },
    Loop { var: SmolStr, from: AstId, to: AstId, body: AstId },
    For { var: SmolStr, iterable: AstId, body: AstId },
    Return(Option<AstId>),
    VarDecl { name: SmolStr, type_expr: Option<TypeExpr>, init: Option<AstId>, mutable: bool },
    Assign { target: AstId, value: AstId },
    Binary { op: BinOp, left: AstId, right: AstId },
    Unary { op: UnaryOp, operand: AstId },
    Call { callee: AstId, args: Vec<AstId> },
    MethodCall { object: AstId, method: SmolStr, args: Vec<AstId> },
    FieldAccess { object: AstId, field: SmolStr },
    Index { object: AstId, index: AstId },
    Identifier(SmolStr),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(SmolStr),
    BoolLiteral(bool),
    NullLiteral,
    Array(Vec<AstId>),
    Lambda { params: Vec<Param>, body: AstId },
    TensorLiteral { dims: Vec<AstId>, elements: Vec<AstId> },
    EntityDef { name: SmolStr, components: Vec<AstId> },
    ComponentDef { name: SmolStr, fields: Vec<Param> },
    SystemDef { name: SmolStr, query: SmolStr, body: AstId },
    ActorDef { name: SmolStr, handlers: Vec<AstId> },
    OnHandler { event: SmolStr, body: AstId },
    View { children: Vec<AstId> },
    StateDecl { name: SmolStr, type_expr: Option<TypeExpr>, init: Option<AstId> },
    TypePath(Vec<SmolStr>),
    TypeTensor { element: Box<TypeExpr>, dims: Vec<TypeExpr> },
    TypeFunction { params: Vec<TypeExpr>, ret: Box<TypeExpr> },
    TypeOptional(Box<TypeExpr>),
    TypeInfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp { Add, Sub, Mul, Div, Eq, Neq, Lt, Gt, Le, Ge, And, Or, Mod }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Neg, Not }

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(SmolStr),
    Tensor { element: Box<TypeExpr>, dims: Vec<TypeExpr> },
    Function { params: Vec<TypeExpr>, ret: Box<TypeExpr> },
    Optional(Box<TypeExpr>),
    Infer,
}

pub struct AstArena {
    nodes: Vec<AstNode>,
}

impl AstArena {
    pub fn new() -> Self {
        AstArena { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, node: AstNode) -> AstId {
        let id = self.nodes.len() as AstId;
        self.nodes.push(node);
        id
    }

    pub fn get(&self, id: AstId) -> &AstNode {
        &self.nodes[id as usize]
    }

    pub fn get_mut(&mut self, id: AstId) -> &mut AstNode {
        &mut self.nodes[id as usize]
    }

    pub fn into_nodes(self) -> Vec<AstNode> {
        self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
