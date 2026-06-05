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
    custom_types: FxHashMap<SmolStr, Vec<SmolStr>>,
    type_var_bindings: FxHashMap<u32, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            vars: FxHashMap::default(),
            functions: FxHashMap::default(),
            next_type_var: 0,
            custom_types: FxHashMap::default(),
            type_var_bindings: FxHashMap::default(),
        }
    }

    /// Resolve a type by following type variable chains.
    pub fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeVar(id) => {
                match self.type_var_bindings.get(id) {
                    Some(bound) => self.resolve(bound),
                    None => ty.clone(),
                }
            }
            Type::Tensor { element, dims } => Type::Tensor {
                element: Box::new(self.resolve(element)),
                dims: *dims,
            },
            Type::Function { params, ret } => Type::Function {
                params: params.iter().map(|p| self.resolve(p)).collect(),
                ret: Box::new(self.resolve(ret)),
            },
            Type::Optional(inner) => Type::Optional(Box::new(self.resolve(inner))),
            Type::Generic { name, params } => Type::Generic {
                name: name.clone(),
                params: params.iter().map(|p| self.resolve(p)).collect(),
            },
            Type::Custom(name, params) => Type::Custom(
                name.clone(),
                params.iter().map(|p| self.resolve(p)).collect(),
            ),
            _ => ty.clone(),
        }
    }

    /// Bind a type variable to a resolved type.
    pub fn bind_type_var(&mut self, id: u32, ty: Type) {
        // Follow the existing binding chain first
        let resolved = self.resolve(&ty);
        self.type_var_bindings.insert(id, resolved);
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
