use smol_str::SmolStr;

use super::ir::*;

// ---------------------------------------------------------------------------
// AST types (minimal — the parser would normally produce these)
// ---------------------------------------------------------------------------

pub type AstId = usize;

/// Arena-allocated AST storage with stable indices.
#[derive(Debug, Default)]
pub struct AstArena {
    nodes: Vec<AstNode>,
}

impl AstArena {
    pub fn alloc(&mut self, node: AstNode) -> AstId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn get(&self, id: AstId) -> &AstNode {
        &self.nodes[id]
    }

    pub fn get_mut(&mut self, id: AstId) -> &mut AstNode {
        &mut self.nodes[id]
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Module {
        functions: Vec<AstId>,
        entities: Vec<AstId>,
        actors: Vec<AstId>,
        types: Vec<AstId>,
    },
    Function {
        name: SmolStr,
        params: Vec<AstId>,
        ret_type: AstId,
        body: AstId,
        is_async: bool,
        is_differentiable: bool,
    },
    Param {
        name: SmolStr,
        ty: AstId,
    },
    If {
        cond: AstId,
        then_body: AstId,
        else_body: AstId,
    },
    While {
        cond: AstId,
        body: AstId,
    },
    Loop {
        var: SmolStr,
        from: AstId,
        to: AstId,
        body: AstId,
    },
    Return(Option<AstId>),
    Let {
        name: SmolStr,
        ty: AstId,
        init: Option<AstId>,
        mutable: bool,
    },
    Assign {
        target: AstId,
        value: AstId,
    },
    Block(Vec<AstId>),
    Binary {
        op: HirBinOp,
        left: AstId,
        right: AstId,
    },
    Unary {
        op: HirUnaryOp,
        operand: AstId,
    },
    Call {
        callee: SmolStr,
        args: Vec<AstId>,
    },
    MethodCall {
        object: AstId,
        method: SmolStr,
        args: Vec<AstId>,
    },
    FieldAccess {
        object: AstId,
        field: SmolStr,
    },
    Index {
        object: AstId,
        index: AstId,
    },
    Identifier(SmolStr),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(SmolStr),
    BoolLiteral(bool),
    Null,
    Array(Vec<AstId>),
    TensorLiteral {
        dims: Vec<AstId>,
        elements: Vec<AstId>,
    },
    EntityDef {
        name: SmolStr,
        components: Vec<AstId>,
    },
    SystemDef {
        name: SmolStr,
        query: SmolStr,
        body: Vec<AstId>,
    },
    OnHandler {
        event: SmolStr,
        body: Vec<AstId>,
    },
    View {
        children: Vec<AstId>,
    },
    StateDecl {
        name: SmolStr,
        ty: AstId,
        init: Option<AstId>,
    },
    TypeRef(HirType),
    Component {
        name: SmolStr,
        fields: Vec<AstId>,
    },
}

// ---------------------------------------------------------------------------
// AST → HIR lowering
// ---------------------------------------------------------------------------

/// Lower a sequence of AST statements into HIR nodes.
pub fn lower(arena: &AstArena, ast_ids: &[AstId]) -> Result<Vec<HirNode>, String> {
    let mut nodes = Vec::with_capacity(ast_ids.len());
    for &id in ast_ids {
        nodes.push(lower_node(arena, id)?);
    }
    Ok(nodes)
}

fn lower_node(arena: &AstArena, id: AstId) -> Result<HirNode, String> {
    let node = arena.get(id).clone();
    match node {
        AstNode::Module { .. } => Err("unexpected Module node in statement position".into()),
        AstNode::Function { .. } => Err("unexpected Function node in statement position".into()),
        AstNode::Param { .. } => Err("unexpected Param node in expression position".into()),
        AstNode::Component { .. } => Err("unexpected Component node in expression position".into()),
        AstNode::TypeRef(_ty) => Ok(HirNode::IntLiteral(0)), // type references lower to nothing meaningful at runtime

        AstNode::If {
            cond,
            then_body,
            else_body,
        } => {
            let cond = lower_node(arena, cond)?;
            let then_body = lower_block(arena, then_body)?;
            let else_body = lower_block(arena, else_body)?;
            Ok(HirNode::If {
                cond: Box::new(cond),
                then_body,
                else_body,
            })
        }
        AstNode::While { cond, body } => {
            let cond = lower_node(arena, cond)?;
            let body = lower_block(arena, body)?;
            Ok(HirNode::While {
                cond: Box::new(cond),
                body,
            })
        }
        AstNode::Loop {
            var,
            from,
            to,
            body,
        } => {
            let from = lower_node(arena, from)?;
            let to = lower_node(arena, to)?;
            let body = lower_block(arena, body)?;
            Ok(HirNode::Loop {
                var,
                from: Box::new(from),
                to: Box::new(to),
                body,
            })
        }
        AstNode::Return(expr) => {
            let expr = expr.map(|e| lower_node(arena, e)).transpose()?;
            Ok(HirNode::Return(expr.map(Box::new)))
        }
        AstNode::Let {
            name,
            ty,
            init,
            mutable,
        } => {
            let hir_ty = lower_type(arena, ty);
            let init = init.map(|e| lower_node(arena, e)).transpose()?;
            Ok(HirNode::Let {
                name,
                ty: hir_ty,
                init: init.map(Box::new),
                mutable,
            })
        }
        AstNode::Assign { target, value } => {
            let target = lower_node(arena, target)?;
            let value = lower_node(arena, value)?;
            Ok(HirNode::Assign {
                target: Box::new(target),
                value: Box::new(value),
            })
        }
        AstNode::Block(stmts) => {
            let nodes = lower_all(arena, &stmts)?;
            Ok(HirNode::Block(nodes))
        }
        AstNode::Binary { op, left, right } => {
            let left = lower_node(arena, left)?;
            let right = lower_node(arena, right)?;
            Ok(HirNode::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            })
        }
        AstNode::Unary { op, operand } => {
            let operand = lower_node(arena, operand)?;
            Ok(HirNode::Unary {
                op,
                operand: Box::new(operand),
            })
        }
        AstNode::Call { callee, args } => {
            let args = lower_all(arena, &args)?;
            Ok(HirNode::Call { callee, args })
        }
        AstNode::MethodCall {
            object,
            method,
            args,
        } => {
            let object = lower_node(arena, object)?;
            let args = lower_all(arena, &args)?;
            Ok(HirNode::MethodCall {
                object: Box::new(object),
                method,
                args,
            })
        }
        AstNode::FieldAccess { object, field } => {
            let object = lower_node(arena, object)?;
            Ok(HirNode::FieldAccess {
                object: Box::new(object),
                field,
            })
        }
        AstNode::Index { object, index } => {
            let object = lower_node(arena, object)?;
            let index = lower_node(arena, index)?;
            Ok(HirNode::Index {
                object: Box::new(object),
                index: Box::new(index),
            })
        }
        AstNode::Identifier(name) => Ok(HirNode::Identifier(name)),
        AstNode::IntLiteral(val) => Ok(HirNode::IntLiteral(val)),
        AstNode::FloatLiteral(val) => Ok(HirNode::FloatLiteral(val)),
        AstNode::StringLiteral(val) => Ok(HirNode::StringLiteral(val)),
        AstNode::BoolLiteral(val) => Ok(HirNode::BoolLiteral(val)),
        AstNode::Null => Ok(HirNode::Null),
        AstNode::Array(elements) => {
            let elements = lower_all(arena, &elements)?;
            Ok(HirNode::Array(elements))
        }
        AstNode::TensorLiteral { dims, elements } => {
            let dims = lower_all(arena, &dims)?;
            let elements = lower_all(arena, &elements)?;
            Ok(HirNode::TensorLiteral { dims, elements })
        }
        AstNode::EntityDef { name, components } => {
            let mut hir_components = Vec::with_capacity(components.len());
            for &comp_id in &components {
                if let AstNode::Component { name: cname, fields } = arena.get(comp_id) {
                    let mut hir_fields = Vec::with_capacity(fields.len());
                    for &fid in fields {
                        if let AstNode::Param { name: pname, ty } = arena.get(fid) {
                            hir_fields.push(HirParam {
                                name: pname.clone(),
                                ty: lower_type(arena, *ty),
                            });
                        }
                    }
                    hir_components.push(HirComponent {
                        name: cname.clone(),
                        fields: hir_fields,
                    });
                }
            }
            Ok(HirNode::EntityDef {
                name,
                components: hir_components,
            })
        }
        AstNode::SystemDef {
            name: _name,
            query,
            body,
        } => {
            let body = lower_all(arena, &body)?;
            Ok(HirNode::SystemDef {
                name: _name,
                query,
                body,
            })
        }
        AstNode::OnHandler { event, body } => {
            let body = lower_all(arena, &body)?;
            Ok(HirNode::OnHandler { event, body })
        }
        AstNode::View { children } => {
            let children = lower_all(arena, &children)?;
            Ok(HirNode::View { children })
        }
        AstNode::StateDecl { name, ty, init } => {
            let hir_ty = lower_type(arena, ty);
            let init = init.map(|e| lower_node(arena, e)).transpose()?;
            Ok(HirNode::StateDecl {
                name,
                ty: hir_ty,
                init: init.map(Box::new),
            })
        }
    }
}

fn lower_block(arena: &AstArena, block_id: AstId) -> Result<Vec<HirNode>, String> {
    match arena.get(block_id) {
        AstNode::Block(stmts) => lower_all(arena, stmts),
        other => Err(format!("expected Block node, got {other:?}")),
    }
}

fn lower_all(arena: &AstArena, ids: &[AstId]) -> Result<Vec<HirNode>, String> {
    let mut nodes = Vec::with_capacity(ids.len());
    for &id in ids {
        nodes.push(lower_node(arena, id)?);
    }
    Ok(nodes)
}

fn lower_type(arena: &AstArena, type_id: AstId) -> HirType {
    match arena.get(type_id) {
        AstNode::TypeRef(ty) => ty.clone(),
        other => {
            // Fallback: try to interpret named types
            HirType::Named(SmolStr::new(format!("{other:?}")))
        }
    }
}

// ---------------------------------------------------------------------------
// Parser AST → HIR lower AST converter
// ---------------------------------------------------------------------------

use std::collections::HashMap;

/// Result of converting a parser AST: the HIR lowerer's arena/IDs, plus any function definitions.
pub struct ParserConversion {
    pub arena: AstArena,
    pub body_ids: Vec<AstId>,
    pub functions: Vec<HirFunction>,
}

/// Convert a parser arena into the HIR lowerer's arena.
/// Returns arena + root node IDs for the main body + extracted function definitions.
pub fn from_parser_ast(parser_arena: &crate::parser::ast::AstArena) -> Result<ParserConversion, String> {
    let mut target = AstArena::default();
    let mut mapping = HashMap::new();
    let mut functions = Vec::new();

    // Find the Program node
    let mut body_ids = Vec::new();
    for i in 0..parser_arena.len() as u32 {
        if matches!(parser_arena.get(i), crate::parser::ast::AstNode::Program { .. }) {
            if let crate::parser::ast::AstNode::Program { body, .. } = parser_arena.get(i) {
                for &bid in body {
                    // Check if this is a function definition — extract it directly
                    if let crate::parser::ast::AstNode::Function { .. } = parser_arena.get(bid) {
                        functions.push(extract_function(parser_arena, bid, &mut target, &mut mapping)?);
                    } else {
                        body_ids.push(convert_node(parser_arena, bid, &mut target, &mut mapping)?);
                    }
                }
            }
            break;
        }
    }

    Ok(ParserConversion { arena: target, body_ids, functions })
}

/// Extract a parser Function node into a HirFunction, also building HIR lower nodes for its body.
fn extract_function(
    parser: &crate::parser::ast::AstArena,
    fn_id: u32,
    target: &mut AstArena,
    mapping: &mut HashMap<u32, AstId>,
) -> Result<HirFunction, String> {
    use crate::parser::ast::AstNode as P;

    let node = parser.get(fn_id);
    let (name, params, ret_type, body_ids, is_async, is_differentiable) = match node {
        P::Function { name, params, ret_type, body, is_async, is_differentiable } => {
            (name.clone(), params, ret_type.clone(), body.clone(), *is_async, *is_differentiable)
        }
        _ => return Err("expected Function node".into()),
    };

    // Convert params
    let mut hir_params = Vec::with_capacity(params.len());
    for param in params {
        let param_ty = match &param.type_expr {
            Some(te) => convert_type_expr_to_hir(te),
            None => HirType::Infer,
        };
        hir_params.push(HirParam {
            name: param.name.clone(),
            ty: param_ty,
        });
    }

    // Convert return type
    let ret_type = match &ret_type {
        Some(te) => convert_type_expr_to_hir(te),
        None => HirType::Infer,
    };

    // Convert body statements into a Block node, then lower to HirNodes
    let mut converted_body = Vec::new();
    for &bid in &body_ids {
        let tid = convert_node(parser, bid, target, mapping)?;
        converted_body.push(tid);
    }
    let block_id = target.alloc(AstNode::Block(converted_body));
    let body_hir = lower(target, &[block_id])?;

    Ok(HirFunction {
        name,
        params: hir_params,
        ret_type,
        body: body_hir,
        is_async,
        is_differentiable,
    })
}

fn convert_node(
    parser: &crate::parser::ast::AstArena,
    id: u32,
    target: &mut AstArena,
    mapping: &mut HashMap<u32, AstId>,
) -> Result<AstId, String> {
    use crate::parser::ast::AstNode as P;

    // Cache check
    if let Some(&tid) = mapping.get(&id) {
        return Ok(tid);
    }

    let node = parser.get(id);
    let tid = match node {
        P::Block(stmts) => {
            let converted: Result<Vec<_>, _> = stmts.iter().map(|&sid| convert_node(parser, sid, target, mapping)).collect();
            target.alloc(AstNode::Block(converted?))
        }
        P::If { cond, then_block, else_block } => {
            let c = convert_node(parser, *cond, target, mapping)?;
            let t = convert_node(parser, *then_block, target, mapping)?;
            let e = else_block.map(|eb| convert_node(parser, eb, target, mapping)).transpose()?.unwrap_or_else(|| {
                target.alloc(AstNode::Block(vec![]))
            });
            target.alloc(AstNode::If { cond: c, then_body: t, else_body: e })
        }
        P::While { cond, body } => {
            let c = convert_node(parser, *cond, target, mapping)?;
            let b = convert_node(parser, *body, target, mapping)?;
            target.alloc(AstNode::While { cond: c, body: b })
        }
        P::Loop { var, from, to, body } => {
            let f = convert_node(parser, *from, target, mapping)?;
            let t = convert_node(parser, *to, target, mapping)?;
            let b = convert_node(parser, *body, target, mapping)?;
            target.alloc(AstNode::Loop { var: var.clone(), from: f, to: t, body: b })
        }
        P::For { var, iterable, body } => {
            // Lower for-loop as a while loop over an iterator (simplified)
            let iter = convert_node(parser, *iterable, target, mapping)?;
            let b = convert_node(parser, *body, target, mapping)?;
            target.alloc(AstNode::Loop { var: var.clone(), from: iter, to: iter, body: b })
        }
        P::Return(opt) => {
            let inner = opt.map(|e| convert_node(parser, e, target, mapping)).transpose()?;
            target.alloc(AstNode::Return(inner))
        }
        P::VarDecl { name, type_expr, init, mutable } => {
            let ty_id = convert_type_expr(type_expr.as_ref(), target);
            let init_id = init.map(|e| convert_node(parser, e, target, mapping)).transpose()?;
            target.alloc(AstNode::Let { name: name.clone(), ty: ty_id, init: init_id, mutable: *mutable })
        }
        P::Assign { target: t_id, value } => {
            let t = convert_node(parser, *t_id, target, mapping)?;
            let v = convert_node(parser, *value, target, mapping)?;
            target.alloc(AstNode::Assign { target: t, value: v })
        }
        P::Binary { op, left, right } => {
            let l = convert_node(parser, *left, target, mapping)?;
            let r = convert_node(parser, *right, target, mapping)?;
            target.alloc(AstNode::Binary { op: convert_binop(*op), left: l, right: r })
        }
        P::Unary { op, operand } => {
            let o = convert_node(parser, *operand, target, mapping)?;
            target.alloc(AstNode::Unary { op: convert_unaryop(*op), operand: o })
        }
        P::Call { callee, args } => {
            let _callee_id = convert_node(parser, *callee, target, mapping)?;
            let callee_name = match parser.get(*callee) {
                P::Identifier(name) => name.clone(),
                _ => SmolStr::new("__unknown"),
            };
            let converted: Result<Vec<_>, _> = args.iter().map(|&a| convert_node(parser, a, target, mapping)).collect();
            target.alloc(AstNode::Call { callee: callee_name, args: converted? })
        }
        P::MethodCall { object, method, args } => {
            let o = convert_node(parser, *object, target, mapping)?;
            let converted: Result<Vec<_>, _> = args.iter().map(|&a| convert_node(parser, a, target, mapping)).collect();
            target.alloc(AstNode::MethodCall { object: o, method: method.clone(), args: converted? })
        }
        P::FieldAccess { object, field } => {
            let o = convert_node(parser, *object, target, mapping)?;
            target.alloc(AstNode::FieldAccess { object: o, field: field.clone() })
        }
        P::Index { object, index } => {
            let o = convert_node(parser, *object, target, mapping)?;
            let i = convert_node(parser, *index, target, mapping)?;
            target.alloc(AstNode::Index { object: o, index: i })
        }
        P::Identifier(name) => target.alloc(AstNode::Identifier(name.clone())),
        P::IntLiteral(v) => target.alloc(AstNode::IntLiteral(*v)),
        P::FloatLiteral(v) => target.alloc(AstNode::FloatLiteral(*v)),
        P::StringLiteral(v) => target.alloc(AstNode::StringLiteral(v.clone())),
        P::BoolLiteral(v) => target.alloc(AstNode::BoolLiteral(*v)),
        P::NullLiteral => target.alloc(AstNode::Null),
        P::Array(elements) => {
            let converted: Result<Vec<_>, _> = elements.iter().map(|&e| convert_node(parser, e, target, mapping)).collect();
            target.alloc(AstNode::Array(converted?))
        }
        P::Lambda { params, body } => {
            // Lambda as a block with param declarations (simplified)
            let mut stmts = Vec::new();
            for param in params {
                let ty_id = convert_type_expr(param.type_expr.as_ref(), target);
                let init = target.alloc(AstNode::Null);
                stmts.push(target.alloc(AstNode::Let {
                    name: param.name.clone(),
                    ty: ty_id,
                    init: Some(init),
                    mutable: false,
                }));
            }
            let body_id = convert_node(parser, *body, target, mapping)?;
            stmts.push(body_id);
            target.alloc(AstNode::Block(stmts))
        }
        P::TensorLiteral { dims, elements } => {
            let converted_dims: Result<Vec<_>, _> = dims.iter().map(|&d| convert_node(parser, d, target, mapping)).collect();
            let converted_els: Result<Vec<_>, _> = elements.iter().map(|&e| convert_node(parser, e, target, mapping)).collect();
            target.alloc(AstNode::TensorLiteral { dims: converted_dims?, elements: converted_els? })
        }
        P::EntityDef { name, components } => {
            let mut comp_ids = Vec::new();
            for &cid in components {
                let cid = convert_node(parser, cid, target, mapping)?;
                comp_ids.push(cid);
            }
            target.alloc(AstNode::EntityDef { name: name.clone(), components: comp_ids })
        }
        P::ComponentDef { name, fields } => {
            let mut field_ids = Vec::new();
            for f in fields {
                let ty_id = convert_type_expr(f.type_expr.as_ref(), target);
                field_ids.push(target.alloc(AstNode::Param { name: f.name.clone(), ty: ty_id }));
            }
            target.alloc(AstNode::Component { name: name.clone(), fields: field_ids })
        }
        P::SystemDef { name, query, body } => {
            let body_id = convert_node(parser, *body, target, mapping)?;
            target.alloc(AstNode::SystemDef { name: name.clone(), query: query.clone(), body: vec![body_id] })
        }
        P::ActorDef { name: _, handlers } => {
            let handler_ids: Result<Vec<_>, _> = handlers.iter().map(|&h| convert_node(parser, h, target, mapping)).collect();
            target.alloc(AstNode::Block(handler_ids?))
        }
        P::OnHandler { event, body } => {
            let body_id = convert_node(parser, *body, target, mapping)?;
            target.alloc(AstNode::OnHandler { event: event.clone(), body: vec![body_id] })
        }
        P::View { children } => {
            let child_ids: Result<Vec<_>, _> = children.iter().map(|&c| convert_node(parser, c, target, mapping)).collect();
            target.alloc(AstNode::View { children: child_ids? })
        }
        P::StateDecl { name, type_expr, init } => {
            let ty_id = convert_type_expr(type_expr.as_ref(), target);
            let init_id = init.map(|e| convert_node(parser, e, target, mapping)).transpose()?;
            target.alloc(AstNode::StateDecl { name: name.clone(), ty: ty_id, init: init_id })
        }
        P::TypePath(parts) => {
            let name = parts.join("::");
            target.alloc(AstNode::TypeRef(HirType::Named(SmolStr::new(name))))
        }
        P::TypeTensor { element, dims: _ } => {
            let el_ty = convert_type_expr_to_hir(element.as_ref());
            target.alloc(AstNode::TypeRef(HirType::Tensor { element: Box::new(el_ty), dims: 0 }))
        }
        P::TypeFunction { params, ret } => {
            let hir_params: Vec<HirType> = params.iter().map(convert_type_expr_to_hir).collect();
            let hir_ret = convert_type_expr_to_hir(ret.as_ref());
            target.alloc(AstNode::TypeRef(HirType::Function { params: hir_params, ret: Box::new(hir_ret) }))
        }
        P::TypeOptional(inner) => {
            let hir_inner = convert_type_expr_to_hir(inner.as_ref());
            target.alloc(AstNode::TypeRef(HirType::Optional(Box::new(hir_inner))))
        }
        P::TypeInfer => target.alloc(AstNode::TypeRef(HirType::Infer)),
        P::Program { .. } | P::Function { .. } | P::Param { .. } => {
            return Err(format!("unexpected parser node {:?} in expression position", node));
        }
    };

    mapping.insert(id, tid);
    Ok(tid)
}

/// Convert a parser TypeExpr reference to a HIR lower AstId (TypeRef).
fn convert_type_expr(type_expr: Option<&crate::parser::ast::TypeExpr>, target: &mut AstArena) -> AstId {
    let hir_ty = match type_expr {
        Some(te) => convert_type_expr_to_hir(te),
        None => HirType::Infer,
    };
    target.alloc(AstNode::TypeRef(hir_ty))
}

fn convert_type_expr_to_hir(te: &crate::parser::ast::TypeExpr) -> HirType {
    use crate::parser::ast::TypeExpr as TE;
    match te {
        TE::Named(name) => {
            // Resolve common type names to concrete HirType values
            match name.as_str() {
                "Int" | "Integer" | "i64" | "i32" => HirType::Int,
                "Float" | "f64" | "f32" | "Double" => HirType::Float,
                "Bool" | "Boolean" => HirType::Bool,
                "String" | "Str" => HirType::String,
                "Null" | "Void" => HirType::Null,
                _ => HirType::Named(name.clone()),
            }
        }
        TE::Tensor { element, dims: _ } => HirType::Tensor {
            element: Box::new(convert_type_expr_to_hir(element)),
            dims: 0,
        },
        TE::Function { params, ret } => HirType::Function {
            params: params.iter().map(convert_type_expr_to_hir).collect(),
            ret: Box::new(convert_type_expr_to_hir(ret)),
        },
        TE::Optional(inner) => HirType::Optional(Box::new(convert_type_expr_to_hir(inner))),
        TE::Infer => HirType::Infer,
    }
}

fn convert_binop(op: crate::parser::ast::BinOp) -> HirBinOp {
    use crate::parser::ast::BinOp as P;
    match op {
        P::Add => HirBinOp::Add,
        P::Sub => HirBinOp::Sub,
        P::Mul => HirBinOp::Mul,
        P::Div => HirBinOp::Div,
        P::Eq => HirBinOp::Eq,
        P::Neq => HirBinOp::Neq,
        P::Lt => HirBinOp::Lt,
        P::Gt => HirBinOp::Gt,
        P::Le => HirBinOp::Le,
        P::Ge => HirBinOp::Ge,
        P::And => HirBinOp::And,
        P::Or => HirBinOp::Or,
        P::Mod => HirBinOp::Mod,
    }
}

fn convert_unaryop(op: crate::parser::ast::UnaryOp) -> HirUnaryOp {
    use crate::parser::ast::UnaryOp as P;
    match op {
        P::Neg => HirUnaryOp::Neg,
        P::Not => HirUnaryOp::Not,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_int_literal() {
        let mut arena = AstArena::default();
        let id = arena.alloc(AstNode::IntLiteral(42));
        let nodes = lower(&arena, &[id]).unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0], HirNode::IntLiteral(42)));
    }

    #[test]
    fn test_lower_binary_expr() {
        let mut arena = AstArena::default();
        let lhs = arena.alloc(AstNode::IntLiteral(1));
        let rhs = arena.alloc(AstNode::IntLiteral(2));
        let expr = arena.alloc(AstNode::Binary {
            op: HirBinOp::Add,
            left: lhs,
            right: rhs,
        });
        let nodes = lower(&arena, &[expr]).unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            HirNode::Binary { op, left, right } => {
                assert_eq!(*op, HirBinOp::Add);
                assert!(matches!(**left, HirNode::IntLiteral(1)));
                assert!(matches!(**right, HirNode::IntLiteral(2)));
            }
            _ => panic!("expected Binary node"),
        }
    }

    #[test]
    fn test_lower_if() {
        let mut arena = AstArena::default();
        let cond = arena.alloc(AstNode::BoolLiteral(true));
        let then_block = arena.alloc(AstNode::Block(vec![]));
        let else_block = arena.alloc(AstNode::Block(vec![]));
        let if_node = arena.alloc(AstNode::If {
            cond,
            then_body: then_block,
            else_body: else_block,
        });
        let nodes = lower(&arena, &[if_node]).unwrap();
        assert!(matches!(nodes[0], HirNode::If { .. }));
    }
}
