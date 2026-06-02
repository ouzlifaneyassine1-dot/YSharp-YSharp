use super::context::{Type, TypeEnv};

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Option<(usize, usize)>, // line, col
}

impl TypeError {
    pub fn new(message: impl Into<String>) -> Self {
        TypeError { message: message.into(), span: None }
    }

    pub fn with_span(mut self, line: usize, col: usize) -> Self {
        self.span = Some((line, col));
        self
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type error: {}", self.message)
    }
}

fn occurs_check(var_id: u32, ty: &Type, env: &TypeEnv) -> bool {
    match ty {
        Type::TypeVar(id) => *id == var_id,
        Type::Tensor { element, .. } => occurs_check(var_id, element, env),
        Type::Function { params, ret } => {
            params.iter().any(|p| occurs_check(var_id, p, env))
                || occurs_check(var_id, ret, env)
        }
        Type::Optional(inner) => occurs_check(var_id, inner, env),
        Type::Generic { params, .. } => params.iter().any(|p| occurs_check(var_id, p, env)),
        Type::Custom(_, params) => params.iter().any(|p| occurs_check(var_id, p, env)),
        _ => false,
    }
}

fn substitute(ty: &Type, var_id: u32, replacement: &Type) -> Type {
    match ty {
        Type::TypeVar(id) if *id == var_id => replacement.clone(),
        Type::TypeVar(_) => ty.clone(),
        Type::Tensor { element, dims } => Type::Tensor {
            element: Box::new(substitute(element, var_id, replacement)),
            dims: *dims,
        },
        Type::Function { params, ret } => Type::Function {
            params: params.iter().map(|p| substitute(p, var_id, replacement)).collect(),
            ret: Box::new(substitute(ret, var_id, replacement)),
        },
        Type::Optional(inner) => Type::Optional(Box::new(substitute(inner, var_id, replacement))),
        Type::Generic { name, params } => Type::Generic {
            name: name.clone(),
            params: params.iter().map(|p| substitute(p, var_id, replacement)).collect(),
        },
        Type::Custom(name, params) => Type::Custom(
            name.clone(),
            params.iter().map(|p| substitute(p, var_id, replacement)).collect(),
        ),
        _ => ty.clone(),
    }
}

pub fn unify(t1: &Type, t2: &Type, env: &mut TypeEnv) -> Result<Type, TypeError> {
    match (t1, t2) {
        (Type::TypeVar(id1), Type::TypeVar(id2)) if id1 == id2 => {
            Ok(Type::TypeVar(*id1))
        }
        (Type::TypeVar(id), ty) | (ty, Type::TypeVar(id)) => {
            if occurs_check(*id, ty, env) {
                return Err(TypeError {
                    message: format!("Occurs check failed: type variable {} appears in {:?}", id, ty),
                    span: None,
                });
            }
            Ok(ty.clone())
        }
        (Type::Int, Type::Int) => Ok(Type::Int),
        (Type::Float, Type::Float) => Ok(Type::Float),
        (Type::Bool, Type::Bool) => Ok(Type::Bool),
        (Type::String, Type::String) => Ok(Type::String),
        (Type::Null, Type::Null) => Ok(Type::Null),
        (Type::Void, Type::Void) => Ok(Type::Void),
        (Type::Infer, ty) | (ty, Type::Infer) => Ok(ty.clone()),
        (Type::Tensor { element: e1, dims: d1 }, Type::Tensor { element: e2, dims: d2 }) => {
            if d1 != d2 {
                return Err(TypeError {
                    message: format!("Tensor dimension mismatch: {} vs {}", d1, d2),
                    span: None,
                });
            }
            let unified_element = unify(e1, e2, env)?;
            Ok(Type::Tensor {
                element: Box::new(unified_element),
                dims: *d1,
            })
        }
        (Type::Function { params: p1, ret: r1 }, Type::Function { params: p2, ret: r2 }) => {
            if p1.len() != p2.len() {
                return Err(TypeError {
                    message: format!("Function parameter count mismatch: {} vs {}", p1.len(), p2.len()),
                    span: None,
                });
            }
            let unified_params: Result<Vec<Type>, TypeError> = p1.iter()
                .zip(p2.iter())
                .map(|(a, b)| unify(a, b, env))
                .collect();
            let unified_ret = unify(r1, r2, env)?;
            Ok(Type::Function {
                params: unified_params?,
                ret: Box::new(unified_ret),
            })
        }
        (Type::Optional(o1), Type::Optional(o2)) => {
            let inner = unify(o1, o2, env)?;
            Ok(Type::Optional(Box::new(inner)))
        }
        (Type::Generic { name: n1, params: p1 }, Type::Generic { name: n2, params: p2 }) => {
            if n1 != n2 {
                return Err(TypeError {
                    message: format!("Generic type mismatch: {} vs {}", n1, n2),
                    span: None,
                });
            }
            if p1.len() != p2.len() {
                return Err(TypeError {
                    message: format!("Generic parameter count mismatch"),
                    span: None,
                });
            }
            let unified_params: Result<Vec<Type>, TypeError> = p1.iter()
                .zip(p2.iter())
                .map(|(a, b)| unify(a, b, env))
                .collect();
            Ok(Type::Generic {
                name: n1.clone(),
                params: unified_params?,
            })
        }
        (Type::Custom(n1, p1), Type::Custom(n2, p2)) => {
            if n1 != n2 {
                return Err(TypeError {
                    message: format!("Custom type mismatch: {} vs {}", n1, n2),
                    span: None,
                });
            }
            if p1.len() != p2.len() {
                return Err(TypeError {
                    message: format!("Custom type parameter count mismatch"),
                    span: None,
                });
            }
            let unified_params: Result<Vec<Type>, TypeError> = p1.iter()
                .zip(p2.iter())
                .map(|(a, b)| unify(a, b, env))
                .collect();
            Ok(Type::Custom(n1.clone(), unified_params?))
        }
        _ => Err(TypeError {
            message: format!("Type mismatch: {:?} vs {:?}", t1, t2),
            span: None,
        }),
    }
}
