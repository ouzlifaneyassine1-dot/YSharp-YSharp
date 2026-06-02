use rustc_hash::FxHashMap;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Null,
    Void,
    Tensor { element: Box<Type>, dims: usize },
    Function { params: Vec<Type>, ret: Box<Type> },
    TypeVar(u32),
    Generic { name: SmolStr, params: Vec<Type> },
    Optional(Box<Type>),
    Entity(SmolStr),
    Custom(SmolStr, Vec<Type>),
    Infer,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub ret: Type,
    pub is_differentiable: bool,
}

pub struct TypeEnv {
    vars: FxHashMap<SmolStr, Type>,
    functions: FxHashMap<SmolStr, FunctionType>,
    next_type_var: u32,
    custom_types: FxHashMap<SmolStr, Vec<SmolStr>>, // type name -> field names
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            vars: FxHashMap::default(),
            functions: FxHashMap::default(),
            next_type_var: 0,
            custom_types: FxHashMap::default(),
        }
    }

    pub fn lookup_var(&self, name: &SmolStr) -> Option<&Type> {
        self.vars.get(name)
    }

    pub fn lookup_fn(&self, name: &SmolStr) -> Option<&FunctionType> {
        self.functions.get(name)
    }

    pub fn bind_var(&mut self, name: SmolStr, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn bind_fn(&mut self, name: SmolStr, ty: FunctionType) {
        self.functions.insert(name, ty);
    }

    pub fn register_custom_type(&mut self, name: SmolStr, fields: Vec<SmolStr>) {
        self.custom_types.insert(name, fields);
    }

    pub fn get_custom_fields(&self, name: &SmolStr) -> Option<&Vec<SmolStr>> {
        self.custom_types.get(name)
    }

    pub fn fresh_type_var(&mut self) -> Type {
        let id = self.next_type_var;
        self.next_type_var += 1;
        Type::TypeVar(id)
    }

    pub fn new_function_type(&mut self) -> Type {
        let fresh_ret = self.fresh_type_var();
        Type::Function {
            params: Vec::new(),
            ret: Box::new(fresh_ret),
        }
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}
