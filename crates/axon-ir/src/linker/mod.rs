use crate::schema::ProjectIR;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: HashSet<String>,
    pub edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn from_ir(ir: &ProjectIR) -> Self {
        let mut nodes = HashSet::new();
        let mut edges = HashMap::new();

        for (key, comp) in &ir.components {
            nodes.insert(key.clone());

            let deps: Vec<String> = comp.functions.values()
                .flat_map(|f| f.dependencies.iter())
                .cloned()
                .collect();

            for dep in &deps {
                nodes.insert(dep.clone());
            }

            edges.insert(key.clone(), deps);
        }

        Self { nodes, edges }
    }

    pub fn compute_impact(&self, seed_nodes: Vec<String>) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue: Vec<String> = seed_nodes;

        while let Some(node) = queue.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());

            if let Some(deps) = self.edges.get(&node) {
                for dep in deps {
                    if !visited.contains(dep) {
                        queue.push(dep.clone());
                    }
                }
            }
        }

        visited
    }

    pub fn detect_cycles(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            graph: &DependencyGraph,
            node: &str,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(deps) = graph.edges.get(node) {
                for dep in deps {
                    if !visited.contains(dep) {
                        if let Some(cycle) = dfs(graph, dep, visited, rec_stack, path) {
                            return Some(cycle);
                        }
                    } else if rec_stack.contains(dep) {
                        let mut cycle = path[path.iter().position(|n| n == dep).unwrap()..].to_vec();
                        cycle.push(dep.clone());
                        return Some(cycle);
                    }
                }
            }

            path.pop();
            rec_stack.remove(node);
            None
        }

        for node in &self.nodes {
            if !visited.contains(node) {
                if let Some(cycle) = dfs(self, node, &mut visited, &mut rec_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }

        None
    }

    pub fn topological_sort(&self) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut result = Vec::new();

        for node in &self.nodes {
            in_degree.insert(node.clone(), 0);
        }

        for (_, deps) in &self.edges {
            for dep in deps {
                if let Some(d) = in_degree.get_mut(dep) {
                    *d += 1;
                }
            }
        }

        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();

        while let Some(node) = queue.pop() {
            result.push(node.clone());

            if let Some(deps) = self.edges.get(&node) {
                for dep in deps {
                    if let Some(d) = in_degree.get_mut(dep) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        result
    }
}

pub fn link_dependencies(ir: &ProjectIR) -> Result<DependencyGraph, String> {
    let graph = DependencyGraph::from_ir(ir);

    if let Some(cycle) = graph.detect_cycles() {
        return Err(format!("Circular dependency detected: {:?}", cycle));
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::schema::{Component, Function};

    #[test]
    fn test_dependency_graph() {
        let mut ir = ProjectIR::new();
        ir.components.insert("a.c".to_string(), Component {
            name: "A".to_string(),
            file_path: "a.c".to_string(),
            functions: std::collections::BTreeMap::new(),
            imports: ["b.c".to_string()].iter().cloned().collect(),
            associated_files: Vec::new(),
            is_entrypoint: false,
            data_models: Vec::new(),
        });
        ir.components.insert("b.c".to_string(), Component {
            name: "B".to_string(),
            file_path: "b.c".to_string(),
            functions: std::collections::BTreeMap::new(),
            imports: ["a.c".to_string()].iter().cloned().collect(),
            associated_files: Vec::new(),
            is_entrypoint: false,
            data_models: Vec::new(),
        });

        let graph = DependencyGraph::from_ir(&ir);
        assert!(graph.nodes.contains("a.c"));
        assert!(graph.nodes.contains("b.c"));
    }
}