use smol_str::SmolStr;

pub type HirId = u32;

#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub entities: Vec<HirEntity>,
    pub actors: Vec<HirActor>,
    pub types: Vec<HirTypeDef>,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: SmolStr,
    pub params: Vec<HirParam>,
    pub ret_type: HirType,
    pub body: Vec<HirNode>,
    pub is_async: bool,
    pub is_differentiable: bool,
}

#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: SmolStr,
    pub ty: HirType,
}

#[derive(Debug, Clone)]
pub enum HirNode {
    If {
        cond: Box<HirNode>,
        then_body: Vec<HirNode>,
        else_body: Vec<HirNode>,
    },
    While {
        cond: Box<HirNode>,
        body: Vec<HirNode>,
    },
    Loop {
        var: SmolStr,
        from: Box<HirNode>,
        to: Box<HirNode>,
        body: Vec<HirNode>,
    },
    Return(Option<Box<HirNode>>),
    Let {
        name: SmolStr,
        ty: HirType,
        init: Option<Box<HirNode>>,
        mutable: bool,
    },
    Assign {
        target: Box<HirNode>,
        value: Box<HirNode>,
    },
    Block(Vec<HirNode>),
    Binary {
        op: HirBinOp,
        left: Box<HirNode>,
        right: Box<HirNode>,
    },
    Unary {
        op: HirUnaryOp,
        operand: Box<HirNode>,
    },
    Call {
        callee: SmolStr,
        args: Vec<HirNode>,
    },
    MethodCall {
        object: Box<HirNode>,
        method: SmolStr,
        args: Vec<HirNode>,
    },
    FieldAccess {
        object: Box<HirNode>,
        field: SmolStr,
    },
    Index {
        object: Box<HirNode>,
        index: Box<HirNode>,
    },
    Identifier(SmolStr),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(SmolStr),
    BoolLiteral(bool),
    Null,
    Array(Vec<HirNode>),
    TensorLiteral {
        dims: Vec<HirNode>,
        elements: Vec<HirNode>,
    },
    EntityDef {
        name: SmolStr,
        components: Vec<HirComponent>,
    },
    SystemDef {
        name: SmolStr,
        query: SmolStr,
        body: Vec<HirNode>,
    },
    OnHandler {
        event: SmolStr,
        body: Vec<HirNode>,
    },
    View {
        children: Vec<HirNode>,
    },
    StateDecl {
        name: SmolStr,
        ty: HirType,
        init: Option<Box<HirNode>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBinOp {
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
pub enum HirUnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirType {
    Int,
    Float,
    Bool,
    String,
    Null,
    Tensor {
        element: Box<HirType>,
        dims: usize,
    },
    Function {
        params: Vec<HirType>,
        ret: Box<HirType>,
    },
    Optional(Box<HirType>),
    Named(SmolStr),
    Infer,
}

#[derive(Debug, Clone)]
pub struct HirEntity {
    pub name: SmolStr,
    pub components: Vec<HirComponent>,
}

#[derive(Debug, Clone)]
pub struct HirActor {
    pub name: SmolStr,
    pub handlers: Vec<HirNode>,
}

#[derive(Debug, Clone)]
pub struct HirTypeDef {
    pub name: SmolStr,
    pub ty: HirType,
}

#[derive(Debug, Clone)]
pub struct HirComponent {
    pub name: SmolStr,
    pub fields: Vec<HirParam>,
}
