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
    pub is_blocking: bool, // v0.0.29.25: Criticality
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

    pub fn add_node(&mut self, id: &str, node_type: NodeType, role: NodeRole, is_blocking: bool) {
        self.nodes.insert(id.to_string(), Node { id: id.to_string(), node_type, role, is_blocking });
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
        let components_opt = ir.get("components");
        if let Some(components) = components_opt {
            if let Some(arr) = components.as_array() {
                for comp in arr {
                    self.process_component_json(comp);
                }
            } else if let Some(map) = components.as_object() {
                for comp in map.values() {
                    self.process_component_json(comp);
                }
            }
        }

        // Second pass: Add edges between components (Dependencies)
        if let Some(components) = components_opt {
            if let Some(arr) = components.as_array() {
                for comp in arr { self.process_component_deps(comp); }
            } else if let Some(map) = components.as_object() {
                for comp in map.values() { self.process_component_deps(comp); }
            }
        }
    }

    fn process_component_json(&mut self, comp: &serde_json::Value) {
        let comp_name = comp.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
        let comp_role = match comp.get("role").and_then(|v| v.as_str()) {
            Some("entry") => NodeRole::Entry,
            Some("boundary") => NodeRole::Boundary,
            _ => NodeRole::Pure,
        };
        // v0.0.29.25: Criticality
        let is_blocking = comp.get("is_blocking").and_then(|v| v.as_bool()).unwrap_or(true);

        let comp_id = format!("comp:{}", comp_name);
        self.add_node(&comp_id, NodeType::Component, comp_role, is_blocking);

        // v0.0.29: Support both 'file' (legacy/RawComponent) and 'file_path' (ProjectIR/Component)
        let file_path_opt = comp.get("file").or_else(|| comp.get("file_path")).and_then(|f| f.as_str());
        
        if let Some(file_path) = file_path_opt {
            let file_id = format!("file:{}", file_path);
            self.add_node(&file_id, NodeType::File, comp_role, is_blocking);
            self.add_edge(&comp_id, &file_id);

            if let Some(funcs) = comp.get("functions").and_then(|f| f.as_array()) {
                for func in funcs {
                    let func_name = func.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let func_id = format!("func:{}::{}", file_path, func_name);
                    self.add_node(&func_id, NodeType::Function, comp_role, is_blocking);
                    self.add_edge(&file_id, &func_id);
                }
            } else if let Some(funcs_map) = comp.get("functions").and_then(|f| f.as_object()) {
                // v0.0.29: Support BTreeMap structure from ProjectIR
                for (func_name, _func_obj) in funcs_map {
                    let func_id = format!("func:{}::{}", file_path, func_name);
                    self.add_node(&func_id, NodeType::Function, comp_role, is_blocking);
                    self.add_edge(&file_id, &func_id);
                }
            }
        }
    }

    fn process_component_deps(&mut self, comp: &serde_json::Value) {
        let comp_name = comp.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
        let comp_id = format!("comp:{}", comp_name);
        
        // Support both 'dependencies' (Vec) and 'imports' (BTreeSet)
        let deps_opt = comp.get("dependencies").or_else(|| comp.get("imports")).and_then(|d| d.as_array());
        if let Some(deps) = deps_opt {
            for dep in deps {
                if let Some(dep_name) = dep.as_str() {
                    let dep_id = format!("comp:{}", dep_name);
                    self.add_edge(&comp_id, &dep_id);
                }
            }
        }
    }

    pub fn generate_cmake(&self, project_name: &str, locale: &str, sandbox_root: &std::path::Path) -> String {
        let mut out = String::new();
        let header = if locale == "ko_KR" {
            "# AXON v0.0.29.25 SOVEREIGN 빌드 스크립트 (PRUNING_ENABLED)\n"
        } else if locale == "ja_JP" {
            "# AXON v0.0.29.25 SOVEREIGN ビルドスクリプト (PRUNING_ENABLED)\n"
        } else {
            "# AXON v0.0.29.25 SOVEREIGN BUILD SCRIPT (PRUNING_ENABLED)\n"
        };
        out.push_str(header);
        out.push_str("cmake_minimum_required(VERSION 3.10)\n");
        out.push_str(&format!("project({})\n\n", project_name));
        out.push_str("set(CMAKE_C_STANDARD 99)\n");
        out.push_str("set(CMAKE_CXX_STANDARD 17)\n\n");

        let mut source_files = HashSet::new();
        let mut has_sqlite = false;

        for (node_id, node) in &self.nodes {
            if let NodeType::File = node.node_type {
                let file_path = node_id.replace("file:", "");
                if file_path.ends_with(".c") || file_path.ends_with(".cpp") {
                    // v0.0.29.25: Physical Pruning
                    let full_path = sandbox_root.join(&file_path);
                    if node.is_blocking || full_path.exists() {
                        source_files.insert(file_path.clone());
                    } else {
                        out.push_str(&format!("# [PRUNED] Optional file missing: {}\n", file_path));
                    }
                }
                if file_path.contains("database") || file_path.contains("sqlite") {
                    has_sqlite = true;
                }
            }
        }

        let mut sources: Vec<String> = source_files.into_iter().collect();
        sources.sort();

        out.push_str("include_directories(include)\n\n");

        if has_sqlite {
            out.push_str("find_package(PkgConfig REQUIRED)\n");
            out.push_str("pkg_check_modules(SQLITE3 REQUIRED sqlite3)\n");
            out.push_str("include_directories(${SQLITE3_INCLUDE_DIRS})\n\n");
        }

        if !sources.is_empty() {
            out.push_str(&format!("add_executable({} {})\n", project_name, sources.join(" ")));
        } else {
            let err_msg = if locale == "ko_KR" {
                "# 오류: 아키텍처 명세에 소스 파일이 정의되지 않았거나 모두 Pruning 되었습니다.\n"
            } else if locale == "ja_JP" {
                "# エラー: アーキテクチャ仕様にソースファイルが定義されていないか、すべて剪定されました。\n"
            } else {
                "# ERROR: Architectural Spec defines no source files or all have been pruned.\n"
            };
            out.push_str(err_msg);
            out.push_str(&format!("add_executable({} src/main.c)\n", project_name));
        }

        if has_sqlite {
            out.push_str(&format!("target_link_libraries({} PRIVATE ${{SQLITE3_LIBRARIES}})\n", project_name));
        }

        out
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
