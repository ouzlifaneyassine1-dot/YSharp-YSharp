
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use super::ir::*;

/// A scope tree tracking identifier bindings with parent links.
#[derive(Debug, Clone)]
pub struct ScopeTree {
    /// Maps each binding name to its unique scope-given id (index into the bindings vec).
    bindings: FxHashMap<SmolStr, Vec<usize>>,
    /// Stack representing the current scope chain.
    scopes: Vec<Scope>,
}

#[derive(Debug, Clone)]
struct Scope {
    /// The bindings introduced in this scope.
    local_bindings: Vec<SmolStr>,
    parent: Option<usize>,
}

impl ScopeTree {
    pub fn new() -> Self {
        ScopeTree {
            bindings: FxHashMap::default(),
            scopes: Vec::new(),
        }
    }

    /// Enter a new scope. Returns the scope index.
    pub fn enter_scope(&mut self, parent: Option<usize>) -> usize {
        let idx = self.scopes.len();
        self.scopes.push(Scope {
            local_bindings: Vec::new(),
            parent,
        });
        idx
    }

    /// Exit the current scope (pop it).
    pub fn exit_scope(&mut self) {
        let scope = self.scopes.pop().expect("scope stack underflow");
        for name in &scope.local_bindings {
            if let Some(entries) = self.bindings.get_mut(name) {
                entries.pop();
                if entries.is_empty() {
                    self.bindings.remove(name);
                }
            }
        }
    }

    /// Declare a new binding in the current scope. Returns the binding index.
    /// Returns `Err` if the binding shadows another in the same scope.
    pub fn declare(&mut self, name: &SmolStr) -> Result<usize, String> {
        let scope_idx = self.scopes.len() - 1;
        let scope = &self.scopes[scope_idx];
        if scope.local_bindings.contains(name) {
            return Err(format!("shadowing not allowed in same scope: `{name}`"));
        }
        let binding_id = self.bindings.len();
        self.bindings
            .entry(name.clone())
            .or_default()
            .push(binding_id);
        self.scopes[scope_idx].local_bindings.push(name.clone());
        Ok(binding_id)
    }

    /// Resolve a name to its binding index.
    pub fn resolve(&self, name: &SmolStr) -> Option<usize> {
        self.bindings.get(name).and_then(|entries| entries.last().copied())
    }

    /// Check if the given name is currently bound (i.e., visible).
    pub fn is_bound(&self, name: &SmolStr) -> bool {
        self.resolve(name).is_some()
    }

    /// The number of unique bindings declared so far.
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Iterate over all bindings (name, id).
    pub fn all_bindings(&self) -> impl Iterator<Item = (&SmolStr, &Vec<usize>)> {
        self.bindings.iter()
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve all identifiers in the module, assigning unique binding indices.
/// Reports shadowing violations.
pub fn resolve(module: &mut HirModule) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut tree = ScopeTree::new();

    // Global scope for type/entity/actor names
    tree.enter_scope(None);

    for typedef in &module.types {
        if let Err(e) = tree.declare(&typedef.name) {
            errors.push(e);
        }
    }
    for entity in &module.entities {
        if let Err(e) = tree.declare(&entity.name) {
            errors.push(e);
        }
    }
    for actor in &module.actors {
        if let Err(e) = tree.declare(&actor.name) {
            errors.push(e);
        }
    }

    for func in &mut module.functions {
        resolve_function(func, &mut tree, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn resolve_function(
    func: &mut HirFunction,
    tree: &mut ScopeTree,
    errors: &mut Vec<String>,
) {
    tree.enter_scope(None);

    // Declare parameters
    for param in &func.params {
        if let Err(e) = tree.declare(&param.name) {
            errors.push(e);
        }
    }

    // Resolve body
    resolve_nodes(&mut func.body, tree, errors);

    tree.exit_scope();
}

fn resolve_nodes(
    nodes: &mut [HirNode],
    tree: &mut ScopeTree,
    errors: &mut Vec<String>,
) {
    let mut i = 0;
    while i < nodes.len() {
        resolve_node(&mut nodes[i], tree, errors);
        i += 1;
    }
}

fn resolve_node(
    node: &mut HirNode,
    tree: &mut ScopeTree,
    errors: &mut Vec<String>,
) {
    match node {
        HirNode::Block(stmts) => {
            tree.enter_scope(None);
            resolve_nodes(stmts, tree, errors);
            tree.exit_scope();
        }
        HirNode::If {
            cond,
            then_body,
            else_body,
        } => {
            resolve_node(cond, tree, errors);
            tree.enter_scope(None);
            resolve_nodes(then_body, tree, errors);
            tree.exit_scope();
            tree.enter_scope(None);
            resolve_nodes(else_body, tree, errors);
            tree.exit_scope();
        }
        HirNode::While { cond, body } => {
            resolve_node(cond, tree, errors);
            tree.enter_scope(None);
            resolve_nodes(body, tree, errors);
            tree.exit_scope();
        }
        HirNode::Loop {
            var: _var,
            from,
            to,
            body,
        } => {
            resolve_node(from, tree, errors);
            resolve_node(to, tree, errors);
            tree.enter_scope(None);
            // Loop variable is declared in the loop body scope
            resolve_nodes(body, tree, errors);
            tree.exit_scope();
        }
        HirNode::Let {
            name,
            init,
            mutable: _,
            ty: _,
        } => {
            if let Some(init) = init {
                resolve_node(init, tree, errors);
            }
            if let Err(e) = tree.declare(name) {
                errors.push(e);
            }
        }
        HirNode::Assign { target, value } => {
            resolve_node(target, tree, errors);
            resolve_node(value, tree, errors);
        }
        HirNode::Return(expr) => {
            if let Some(expr) = expr {
                resolve_node(expr, tree, errors);
            }
        }
        HirNode::Binary { left, right, .. } => {
            resolve_node(left, tree, errors);
            resolve_node(right, tree, errors);
        }
        HirNode::Unary { operand, .. } => {
            resolve_node(operand, tree, errors);
        }
        HirNode::Call { args, .. } => {
            for arg in args.iter_mut() {
                resolve_node(arg, tree, errors);
            }
        }
        HirNode::MethodCall {
            object, args, ..
        } => {
            resolve_node(object, tree, errors);
            for arg in args.iter_mut() {
                resolve_node(arg, tree, errors);
            }
        }
        HirNode::FieldAccess { object, .. } => {
            resolve_node(object, tree, errors);
        }
        HirNode::Index { object, index } => {
            resolve_node(object, tree, errors);
            resolve_node(index, tree, errors);
        }
        HirNode::Identifier(name) => {
            if !tree.is_bound(name) {
                errors.push(format!("unresolved identifier `{name}`"));
            }
        }
        HirNode::Array(elements) => {
            for el in elements.iter_mut() {
                resolve_node(el, tree, errors);
            }
        }
        HirNode::TensorLiteral { dims, elements } => {
            for d in dims.iter_mut() {
                resolve_node(d, tree, errors);
            }
            for el in elements.iter_mut() {
                resolve_node(el, tree, errors);
            }
        }
        HirNode::EntityDef { components, .. } => {
            for comp in components {
                for field in &mut comp.fields {
                    if let Err(e) = tree.declare(&field.name) {
                        errors.push(e);
                    }
                }
            }
        }
        HirNode::SystemDef { body, .. } => {
            tree.enter_scope(None);
            resolve_nodes(body, tree, errors);
            tree.exit_scope();
        }
        HirNode::OnHandler { body, .. } => {
            tree.enter_scope(None);
            resolve_nodes(body, tree, errors);
            tree.exit_scope();
        }
        HirNode::View { children } => {
            tree.enter_scope(None);
            resolve_nodes(children, tree, errors);
            tree.exit_scope();
        }
        HirNode::StateDecl { name, init, .. } => {
            if let Some(init) = init {
                resolve_node(init, tree, errors);
            }
            if let Err(e) = tree.declare(name) {
                errors.push(e);
            }
        }
        HirNode::IntLiteral(_)
        | HirNode::FloatLiteral(_)
        | HirNode::StringLiteral(_)
        | HirNode::BoolLiteral(_)
        | HirNode::Null => {}
    }
}

/// Analysis of captured variables for closures/lambdas.
#[derive(Debug, Clone)]
pub struct CaptureInfo {
    /// Set of variable names captured from enclosing scopes.
    pub captures: Vec<SmolStr>,
}

#[derive(Debug, Clone)]
pub struct CaptureAnalysis {
    /// Per-function capture information.
    pub captures: FxHashMap<SmolStr, CaptureInfo>,
}

impl CaptureAnalysis {
    pub fn new() -> Self {
        CaptureAnalysis {
            captures: FxHashMap::default(),
        }
    }

    /// Analyze a module for captured variables.
    pub fn analyze(&mut self, module: &HirModule) -> Result<(), String> {
        let mut tree = ScopeTree::new();

        // Global type/entity/actor names
        tree.enter_scope(None);
        for t in &module.types {
            let _ = tree.declare(&t.name);
        }
        for e in &module.entities {
            let _ = tree.declare(&e.name);
        }
        for a in &module.actors {
            let _ = tree.declare(&a.name);
        }

        for func in &module.functions {
            let captures = analyze_function_captures(func, &mut tree)?;
            self.captures.insert(func.name.clone(), captures);
        }

        Ok(())
    }

    /// Return the captures for a given function.
    pub fn captures_for(&self, name: &SmolStr) -> Option<&CaptureInfo> {
        self.captures.get(name)
    }
}

impl Default for CaptureAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

fn analyze_function_captures(
    func: &HirFunction,
    tree: &mut ScopeTree,
) -> Result<CaptureInfo, String> {
    tree.enter_scope(None);

    for param in &func.params {
        tree.declare(&param.name).ok();
    }

    let mut captured = Vec::new();
    let mut visitor = CaptureVisitor {
        tree,
        captured: &mut captured,
        current_scope_bindings: Vec::new(),
    };
    visitor.visit_nodes(&func.body);

    visitor.tree.exit_scope();

    Ok(CaptureInfo {
        captures: visitor.captured.clone(),
    })
}

struct CaptureVisitor<'a> {
    tree: &'a mut ScopeTree,
    captured: &'a mut Vec<SmolStr>,
    current_scope_bindings: Vec<SmolStr>,
}

impl<'a> CaptureVisitor<'a> {
    fn visit_nodes(&mut self, nodes: &[HirNode]) {
        for node in nodes {
            self.visit_node(node);
        }
    }

    fn visit_node(&mut self, node: &HirNode) {
        match node {
            HirNode::Block(stmts) => {
                self.enter_scope();
                self.visit_nodes(stmts);
                self.exit_scope();
            }
            HirNode::If {
                cond,
                then_body,
                else_body,
            } => {
                self.visit_node(cond);
                self.enter_scope();
                self.visit_nodes(then_body);
                self.exit_scope();
                self.enter_scope();
                self.visit_nodes(else_body);
                self.exit_scope();
            }
            HirNode::While { cond, body } => {
                self.visit_node(cond);
                self.enter_scope();
                self.visit_nodes(body);
                self.exit_scope();
            }
            HirNode::Loop {
                var: _var,
                from,
                to,
                body,
            } => {
                self.visit_node(from);
                self.visit_node(to);
                self.enter_scope();
                self.visit_nodes(body);
                self.exit_scope();
            }
            HirNode::Let { name, init, .. } => {
                if let Some(init) = init {
                    self.visit_node(init);
                }
                self.current_scope_bindings.push(name.clone());
                self.tree.declare(name).ok();
            }
            HirNode::Assign { target, value } => {
                self.visit_node(target);
                self.visit_node(value);
            }
            HirNode::Return(expr) => {
                if let Some(expr) = expr {
                    self.visit_node(expr);
                }
            }
            HirNode::Binary { left, right, .. } => {
                self.visit_node(left);
                self.visit_node(right);
            }
            HirNode::Unary { operand, .. } => {
                self.visit_node(operand);
            }
            HirNode::Call { args, .. } => {
                for arg in args {
                    self.visit_node(arg);
                }
            }
            HirNode::MethodCall { object, args, .. } => {
                self.visit_node(object);
                for arg in args {
                    self.visit_node(arg);
                }
            }
            HirNode::FieldAccess { object, .. } => {
                self.visit_node(object);
            }
            HirNode::Index { object, index } => {
                self.visit_node(object);
                self.visit_node(index);
            }
            HirNode::Identifier(name) => {
                if self.tree.resolve(name).is_some()
                    && !self.current_scope_bindings.contains(name)
                    && !self.captured.contains(name)
                {
                    self.captured.push(name.clone());
                }
            }
            HirNode::Array(elements) => {
                for el in elements {
                    self.visit_node(el);
                }
            }
            HirNode::TensorLiteral { dims, elements } => {
                for d in dims {
                    self.visit_node(d);
                }
                for el in elements {
                    self.visit_node(el);
                }
            }
            HirNode::EntityDef { .. } | HirNode::SystemDef { .. } => {
                self.enter_scope();
                if let HirNode::SystemDef { body, .. } = node {
                    self.visit_nodes(body);
                }
                self.exit_scope();
            }
            HirNode::OnHandler { body, .. } => {
                self.enter_scope();
                self.visit_nodes(body);
                self.exit_scope();
            }
            HirNode::View { children } => {
                self.enter_scope();
                self.visit_nodes(children);
                self.exit_scope();
            }
            HirNode::StateDecl { name, init, .. } => {
                if let Some(init) = init {
                    self.visit_node(init);
                }
                self.current_scope_bindings.push(name.clone());
                self.tree.declare(name).ok();
            }
            HirNode::IntLiteral(_)
            | HirNode::FloatLiteral(_)
            | HirNode::StringLiteral(_)
            | HirNode::BoolLiteral(_)
            | HirNode::Null => {}
        }
    }

    fn enter_scope(&mut self) {
        self.tree.enter_scope(None);
    }

    fn exit_scope(&mut self) {
        self.tree.exit_scope();
    }
}
