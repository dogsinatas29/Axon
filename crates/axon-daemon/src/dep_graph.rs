use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    File,
    Component,
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum NodeRole {
    Entry,
    Boundary,
    Pure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub role: NodeRole,
}

pub struct DepGraph {
    pub nodes: HashMap<String, Node>,
    pub edges_out: HashMap<String, HashSet<String>>, // Forward: A -> B (A uses B)
    pub edges_in: HashMap<String, HashSet<String>>,  // Reverse: B -> A (A is used by B)
}

impl DepGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges_out: HashMap::new(),
            edges_in: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: &str, node_type: NodeType, role: NodeRole) {
        self.nodes.insert(id.to_string(), Node { id: id.to_string(), node_type, role });
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges_out.entry(from.to_string()).or_default().insert(to.to_string());
        self.edges_in.entry(to.to_string()).or_default().insert(from.to_string());
    }

    /// BFS through reverse edges to find all impacted parents
    pub fn compute_impact(&self, changed_nodes: Vec<String>) -> HashSet<String> {
        let mut impacted = HashSet::new();
        let mut queue: Vec<String> = changed_nodes;

        while let Some(n) = queue.pop() {
            if impacted.contains(&n) {
                continue;
            }
            impacted.insert(n.clone());

            if let Some(parents) = self.edges_in.get(&n) {
                for parent in parents {
                    queue.push(parent.clone());
                }
            }
        }

        impacted
    }

    /// Build initial graph from architecture.md JSON components
    pub fn build_from_ir(&mut self, ir: &serde_json::Value) {
        if let Some(components) = ir.get("components").and_then(|c| c.as_array()) {
            for comp in components {
                let comp_name = comp.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                let comp_role = match comp.get("role").and_then(|v| v.as_str()) {
                    Some("entry") => NodeRole::Entry,
                    Some("boundary") => NodeRole::Boundary,
                    _ => NodeRole::Pure,
                };
                let comp_id = format!("comp:{}", comp_name);
                self.add_node(&comp_id, NodeType::Component, comp_role);

                if let Some(file_path) = comp.get("file").and_then(|f| f.as_str()) {
                    let file_id = format!("file:{}", file_path);
                    self.add_node(&file_id, NodeType::File, comp_role); // File inherits component role
                    self.add_edge(&comp_id, &file_id); // Component owns file

                    if let Some(funcs) = comp.get("functions").and_then(|f| f.as_array()) {
                        for func in funcs {
                            let func_name = func.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let func_id = format!("func:{}::{}", file_path, func_name);
                            self.add_node(&func_id, NodeType::Function, comp_role);
                            self.add_edge(&file_id, &func_id); // File defines function
                        }
                    }
                }
            }
        }
    }

    /// Parse code to find explicit dependencies (e.g., 'use' statements in Rust)
    pub fn enrich_from_code(&mut self, file_path: &str, code: &str) {
        let file_id = format!("file:{}", file_path);
        
        // Rust 'use' pattern: use crate::module::item;
        let use_re = Regex::new(r"use\s+([a-zA-Z0-9_:]+)").unwrap();
        for cap in use_re.captures_iter(code) {
            let used_path = &cap[1];
            // Naive mapping: take the last or second to last part as potential file name
            let parts: Vec<&str> = used_path.split("::").collect();
            if parts.len() > 1 {
                let target_file = format!("{}.rs", parts[parts.len() - 2]); // Very simple heuristic
                let target_id = format!("file:{}", target_file);
                if self.nodes.contains_key(&target_id) {
                    self.add_edge(&file_id, &target_id);
                }
            }
        }

        // Call pattern: name(args)
        let call_re = Regex::new(r"([a-zA-Z0-9_]+)\s*\(").unwrap();
        let mut target_ids = Vec::new();
        for cap in call_re.captures_iter(code) {
            let call_name = &cap[1];
            // Look for any function with this name in the graph
            for node_id in self.nodes.keys() {
                if node_id.starts_with("func:") && node_id.ends_with(&format!("::{}", call_name)) {
                    target_ids.push(node_id.clone());
                }
            }
        }
        for tid in target_ids {
            self.add_edge(&file_id, &tid);
        }
    }

    /// Identify which nodes in the closure require runtime validation (Entry or Boundary)
    pub fn run_targets(&self, closure: &HashSet<String>) -> Vec<String> {
        closure.iter()
            .filter(|id| {
                if let Some(node) = self.nodes.get(*id) {
                    matches!(node.role, NodeRole::Entry | NodeRole::Boundary)
                } else {
                    true // Default to safety
                }
            })
            .cloned()
            .collect()
    }
}
