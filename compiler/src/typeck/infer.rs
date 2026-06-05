use smol_str::SmolStr;

use super::context::{FunctionType, Type, TypeEnv};
use super::unify::{unify, TypeError};
use crate::parser::ast::{AstArena, AstNode, BinOp, UnaryOp};

pub fn infer_expr(
    arena: &AstArena,
    id: u32,
    env: &mut TypeEnv,
) -> Result<Type, TypeError> {
    let node = arena.get(id);
    match node {
        AstNode::IntLiteral(_) => Ok(Type::Int),
        AstNode::FloatLiteral(_) => Ok(Type::Float),
        AstNode::BoolLiteral(_) => Ok(Type::Bool),
        AstNode::StringLiteral(_) => Ok(Type::String),
        AstNode::NullLiteral => Ok(Type::Null),

        AstNode::Identifier(name) => {
            // Check built-in functions first (they're not variables)
            if let Some(fn_type) = env.lookup_fn(name).cloned() {
                Ok(Type::Function {
                    params: fn_type.params.clone(),
                    ret: Box::new(fn_type.ret.clone()),
                })
            } else {
                env.lookup_var(name).cloned().ok_or_else(|| TypeError::new(format!("Undefined variable: {}", name)))
            }
        }

        AstNode::Binary { op, left, right } => {
            let left_ty = infer_expr(arena, *left, env)?;
            let right_ty = infer_expr(arena, *right, env)?;

            match op {
                BinOp::Add => {
                    // Addition: numeric OR string concatenation
                    let left_is_str = matches!(&left_ty, Type::String);
                    let right_is_str = matches!(&right_ty, Type::String);
                    if left_is_str || right_is_str {
                        unify(&left_ty, &Type::String, env)?;
                        unify(&right_ty, &Type::String, env)?;
                        Ok(Type::String)
                    } else {
                        let numeric = |ty: &Type| -> bool {
                            matches!(ty, Type::Int | Type::Float | Type::TypeVar(_))
                        };
                        if !numeric(&left_ty) && !matches!(left_ty, Type::TypeVar(_)) {
                            return Err(TypeError::new(format!("Left operand must be numeric or string, got {:?}", left_ty)));
                        }
                        if !numeric(&right_ty) && !matches!(right_ty, Type::TypeVar(_)) {
                            return Err(TypeError::new(format!("Right operand must be numeric or string, got {:?}", right_ty)));
                        }
                        match (&left_ty, &right_ty) {
                            (Type::Float, _) | (_, Type::Float) => Ok(Type::Float),
                            _ => Ok(Type::Int),
                        }
                    }
                }
                BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                    let numeric = |ty: &Type| -> bool {
                        matches!(ty, Type::Int | Type::Float | Type::TypeVar(_))
                    };
                    if !numeric(&left_ty) && !matches!(left_ty, Type::TypeVar(_)) {
                        return Err(TypeError::new(format!("Left operand must be numeric, got {:?}", left_ty)));
                    }
                    if !numeric(&right_ty) && !matches!(right_ty, Type::TypeVar(_)) {
                        return Err(TypeError::new(format!("Right operand must be numeric, got {:?}", right_ty)));
                    }
                    match (&left_ty, &right_ty) {
                        (Type::Float, _) | (_, Type::Float) => Ok(Type::Float),
                        _ => Ok(Type::Int),
                    }
                }
                BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                    unify(&left_ty, &right_ty, env)?;
                    Ok(Type::Bool)
                }
                BinOp::And | BinOp::Or => {
                    unify(&left_ty, &Type::Bool, env)?;
                    unify(&right_ty, &Type::Bool, env)?;
                    Ok(Type::Bool)
                }
            }
        }

        AstNode::Unary { op, operand } => {
            let op_ty = infer_expr(arena, *operand, env)?;
            match op {
                UnaryOp::Neg => {
                    if matches!(op_ty, Type::Int | Type::Float | Type::TypeVar(_)) {
                        Ok(op_ty)
                    } else {
                        Err(TypeError::new(format!("Cannot negate {:?}", op_ty)))
                    }
                }
                UnaryOp::Not => {
                    unify(&op_ty, &Type::Bool, env)?;
                    Ok(Type::Bool)
                }
            }
        }

        AstNode::Call { callee, args } => {
            // Check if it's a known built-in function BEFORE variable lookup
            let callee_is_builtin = if let AstNode::Identifier(name) = arena.get(*callee) {
                env.lookup_fn(name).is_some()
            } else {
                false
            };

            if callee_is_builtin {
                if let AstNode::Identifier(name) = arena.get(*callee) {
                    if let Some(fn_type) = env.lookup_fn(name) {
                        let fn_type = fn_type.clone();
                        let param_count = fn_type.params.len();
                        let arg_tys: Result<Vec<Type>, TypeError> = args
                            .iter()
                            .map(|arg_id| infer_expr(arena, *arg_id, env))
                            .collect();
                        let arg_tys = arg_tys?;

                        if arg_tys.len() != param_count {
                            return Err(TypeError::new(format!(
                                "Function {} expects {} arguments, got {}",
                                name,
                                param_count,
                                arg_tys.len()
                            )));
                        }

                        for (arg, param) in arg_tys.iter().zip(fn_type.params.iter()) {
                            unify(arg, param, env)?;
                        }

                        return Ok(fn_type.ret.clone());
                    }
                }
            }

            // Generic function call: infer from callee type
            let callee_ty = infer_expr(arena, *callee, env)?;
            match &callee_ty {
                Type::Function { params, ret } => {
                    let arg_tys: Result<Vec<Type>, TypeError> = args
                        .iter()
                        .map(|arg_id| infer_expr(arena, *arg_id, env))
                        .collect();
                    let arg_tys = arg_tys?;

                    if arg_tys.len() != params.len() {
                        return Err(TypeError::new(format!(
                            "Function expects {} arguments, got {}",
                            params.len(),
                            arg_tys.len()
                        )));
                    }

                    for (arg, param) in arg_tys.iter().zip(params.iter()) {
                        unify(arg, param, env)?;
                    }

                    Ok(ret.as_ref().clone())
                }
                _ => Err(TypeError::new(format!("{:?} is not callable", callee_ty))),
            }
        }

        AstNode::Block(stmts) => {
            let mut last_ty = Type::Null;
            for stmt_id in stmts {
                last_ty = infer_expr(arena, *stmt_id, env)?;
            }
            Ok(last_ty)
        }

        AstNode::If {
            cond,
            then_block,
            else_block,
        } => {
            let cond_ty = infer_expr(arena, *cond, env)?;
            unify(&cond_ty, &Type::Bool, env)?;

            let then_ty = infer_expr(arena, *then_block, env)?;
            match else_block {
                Some(else_id) => {
                    let else_ty = infer_expr(arena, *else_id, env)?;
                    unify(&then_ty, &else_ty, env)
                }
                None => Ok(Type::Null),
            }
        }

        AstNode::While { cond, body } => {
            let cond_ty = infer_expr(arena, *cond, env)?;
            unify(&cond_ty, &Type::Bool, env)?;
            infer_expr(arena, *body, env)?;
            Ok(Type::Null)
        }

        AstNode::Loop {
            var,
            from,
            to,
            body,
        } => {
            let from_ty = infer_expr(arena, *from, env)?;
            let to_ty = infer_expr(arena, *to, env)?;
            unify(&from_ty, &Type::Int, env)?;
            unify(&to_ty, &Type::Int, env)?;
            // Bind loop variable as Int within the loop body
            env.bind_var(var.clone(), Type::Int);
            infer_expr(arena, *body, env)?;
            Ok(Type::Null)
        }

        AstNode::VarDecl {
            name,
            type_expr: _,
            init,
            mutable: _,
        } => {
            let ty = if let Some(init_id) = init {
                let inferred = infer_expr(arena, *init_id, env)?;
                env.bind_var(name.clone(), inferred.clone());
                inferred
            } else {
                let tv = env.fresh_type_var();
                env.bind_var(name.clone(), tv.clone());
                tv
            };
            Ok(ty)
        }

        AstNode::Assign { target, value } => {
            let target_ty = infer_expr(arena, *target, env)?;
            let value_ty = infer_expr(arena, *value, env)?;
            unify(&target_ty, &value_ty, env)
        }

        AstNode::Return(opt_expr) => {
            match opt_expr {
                Some(expr_id) => infer_expr(arena, *expr_id, env),
                None => Ok(Type::Null),
            }
        }

        AstNode::Array(elements) => {
            if elements.is_empty() {
                Ok(Type::Generic {
                    name: SmolStr::new("Array"),
                    params: vec![env.fresh_type_var()],
                })
            } else {
                let elem_ty = infer_expr(arena, elements[0], env)?;
                for elem_id in &elements[1..] {
                    let ety = infer_expr(arena, *elem_id, env)?;
                    unify(&elem_ty, &ety, env)?;
                }
                Ok(Type::Generic {
                    name: SmolStr::new("Array"),
                    params: vec![elem_ty],
                })
            }
        }

        AstNode::Lambda { params, body } => {
            let mut param_tys = Vec::new();
            let fresh_ret = env.fresh_type_var();

            for param in params {
                let pt = param
                    .type_expr
                    .as_ref()
                    .map(|te| type_expr_to_type(te, env))
                    .unwrap_or_else(|| env.fresh_type_var());
                env.bind_var(param.name.clone(), pt.clone());
                param_tys.push(pt);
            }

            let ret_ty = infer_expr(arena, *body, env)?;
            unify(&fresh_ret, &ret_ty, env)?;

            Ok(Type::Function {
                params: param_tys,
                ret: Box::new(ret_ty),
            })
        }

        AstNode::Program { body, .. } => {
            let mut last_ty = Type::Null;
            for stmt_id in body {
                last_ty = infer_expr(arena, *stmt_id, env)?;
            }
            Ok(last_ty)
        }

        AstNode::Function {
            name,
            params,
            ret_type,
            body,
            is_differentiable,
            ..
        } => {
            let mut param_tys = Vec::new();
            let mut param_names = Vec::new();

            for param in params {
                let pt = param
                    .type_expr
                    .as_ref()
                    .map(|te| type_expr_to_type(te, env))
                    .unwrap_or_else(|| env.fresh_type_var());
                param_tys.push(pt.clone());
                param_names.push(param.name.clone());
            }

            let ret_ty = ret_type
                .as_ref()
                .map(|te| type_expr_to_type(te, env))
                .unwrap_or_else(|| env.fresh_type_var());

            // Bind params in new scope
            for (name, ty) in param_names.iter().zip(param_tys.iter()) {
                env.bind_var(name.clone(), ty.clone());
            }

            let body_ty = if body.is_empty() {
                Type::Null
            } else {
                let mut last = Type::Null;
                for stmt_id in body {
                    last = infer_expr(arena, *stmt_id, env)?;
                }
                last
            };

            // If body doesn't return explicitly, use its block result type
            if !matches!(ret_ty, Type::TypeVar(_)) {
                if !matches!(body_ty, Type::Null) {
                    unify(&ret_ty, &body_ty, env)?;
                }
            }

            let fn_ty = Type::Function {
                params: param_tys.clone(),
                ret: Box::new(ret_ty.clone()),
            };

            env.bind_fn(name.clone(), FunctionType {
                params: param_tys,
                ret: ret_ty,
                is_differentiable: *is_differentiable,
            });

            Ok(fn_ty)
        }

        _other => {
            // Default: try to infer, or return fresh type var
            Ok(env.fresh_type_var())
        }
    }
}

fn type_expr_to_type(te: &crate::parser::ast::TypeExpr, env: &mut TypeEnv) -> Type {
    match te {
        crate::parser::ast::TypeExpr::Named(name) => Type::Custom(name.clone(), vec![]),
        crate::parser::ast::TypeExpr::Infer => env.fresh_type_var(),
        crate::parser::ast::TypeExpr::Tensor { element, dims: _ } => Type::Tensor {
            element: Box::new(type_expr_to_type(element, env)),
            dims: 0, // simplified
        },
        crate::parser::ast::TypeExpr::Function { params, ret } => Type::Function {
            params: params.iter().map(|p| type_expr_to_type(p, env)).collect(),
            ret: Box::new(type_expr_to_type(ret, env)),
        },
        crate::parser::ast::TypeExpr::Optional(inner) => {
            Type::Optional(Box::new(type_expr_to_type(inner, env)))
        }
    }
}
