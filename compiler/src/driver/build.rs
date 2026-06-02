use std::collections::HashMap;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub struct BuildGraph {
    files: HashMap<SmolStr, String>,
    dependencies: HashMap<SmolStr, Vec<SmolStr>>,
}

impl BuildGraph {
    pub fn new() -> Self {
        BuildGraph {
            files: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: SmolStr, source: String) {
        self.files.entry(path.clone()).or_insert(source);
        self.dependencies.entry(path).or_default();
    }

    pub fn add_dependency(&mut self, from: SmolStr, to: SmolStr) {
        self.dependencies.entry(from).or_default().push(to);
    }

    pub fn get_source(&self, path: &SmolStr) -> Option<&str> {
        self.files.get(path).map(|s| s.as_str())
    }

    pub fn resolve_order(&self) -> Vec<SmolStr> {
        let mut visited = HashMap::new();
        let mut order = Vec::new();

        for path in self.files.keys() {
            if !visited.contains_key(path) {
                self.visit(path.clone(), &mut visited, &mut order);
            }
        }

        order.reverse();
        order
    }

    fn visit(
        &self,
        node: SmolStr,
        visited: &mut HashMap<SmolStr, bool>,
        order: &mut Vec<SmolStr>,
    ) {
        match visited.get(&node) {
            Some(true) => return,
            Some(false) => {
                panic!("circular dependency detected for '{}'", node);
            }
            None => {
                visited.insert(node.clone(), false);
                if let Some(deps) = self.dependencies.get(&node) {
                    for dep in deps {
                        self.visit(dep.clone(), visited, order);
                    }
                }
                visited.insert(node.clone(), true);
                order.push(node);
            }
        }
    }
}

impl Default for BuildGraph {
    fn default() -> Self {
        Self::new()
    }
}
