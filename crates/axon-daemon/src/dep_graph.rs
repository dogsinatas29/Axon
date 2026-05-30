use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use regex::Regex;

// v0.0.31.xx: AXON:SPEC:CMAKE block parser — reads structured CMake config from spec.md
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CMakeSpec {
    pub cmake_minimum_required: Option<String>,
    pub project_name: Option<String>,
    pub cxx_standard: Option<u32>,
    pub cxx_compiler: Option<String>,
    pub pkg_config_modules: Vec<PkgConfigModule>,
    pub find_packages: Vec<FindPackage>,
    pub link_libraries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgConfigModule {
    pub name: String,
    pub version: String,
    pub link_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindPackage {
    pub name: String,
    pub version: String,
    pub required: bool,
}

// v0.0.31.xx: AXON:SPEC:LUA block parser
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LuaSpec {
    pub lua_version: String,
    pub linking: String,
    pub scripts: Vec<LuaScript>,
    pub c_bindings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaScript {
    pub file: String,
    pub role: String,
}

/// Parses <!-- AXON:SPEC:CMAKE { ... } --> block from spec.md
pub fn parse_cmake_spec(spec: &str) -> Option<CMakeSpec> {
    let start_marker = "<!-- AXON:SPEC:CMAKE";
    let end_marker = "-->";
    let start = spec.find(start_marker)?;
    let end = spec[start..].find(end_marker)?;
    let json_str = spec[start + start_marker.len()..start + end].trim();
    serde_json::from_str(json_str).ok()
}

/// Parses <!-- AXON:SPEC:LUA { ... } --> block from spec.md
pub fn parse_lua_spec(spec: &str) -> Option<LuaSpec> {
    let start_marker = "<!-- AXON:SPEC:LUA";
    let end_marker = "-->";
    let start = spec.find(start_marker)?;
    let end = spec[start..].find(end_marker)?;
    let json_str = spec[start + start_marker.len()..start + end].trim();
    serde_json::from_str(json_str).ok()
}

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
    pub component_type: axon_ir::schema::types::ComponentType, // v0.0.31.20: Semantic classification
}

pub struct DepGraph {
    pub nodes: HashMap<String, Node>,
    pub edges_out: HashMap<String, HashSet<String>>, // Forward: A -> B (A uses B)
    pub edges_in: HashMap<String, HashSet<String>>,  // Reverse: B -> A (A is used by B)
    pub platform: Option<String>,
    pub runtime_model: Option<String>,
}

impl DepGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges_out: HashMap::new(),
            edges_in: HashMap::new(),
            platform: None,
            runtime_model: None,
        }
    }

    pub fn add_node(&mut self, id: &str, node_type: NodeType, role: NodeRole, is_blocking: bool, component_type: axon_ir::schema::types::ComponentType) {
        self.nodes.insert(id.to_string(), Node { id: id.to_string(), node_type, role, is_blocking, component_type });
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
        if let Some(p) = ir.get("platform").and_then(|v| v.as_str()) {
            self.platform = Some(p.to_string());
        }
        if let Some(r) = ir.get("runtime_model").and_then(|v| v.as_str()) {
            self.runtime_model = Some(r.to_string());
        }

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

        // v0.0.31.20: Extract or auto-classify component type
        let component_type = match comp.get("component_type")
            .or_else(|| comp.get("type"))
            .and_then(|v| v.as_str()) 
        {
            Some("system_library") => axon_ir::schema::types::ComponentType::SystemLibrary,
            Some("external_runtime") => axon_ir::schema::types::ComponentType::ExternalRuntime,
            _ => {
                let file_path = comp.get("file").or_else(|| comp.get("file_path")).and_then(|f| f.as_str()).unwrap_or("");
                let file_lower = file_path.to_lowercase();
                let base_name = std::path::Path::new(&file_lower)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let name_lower = comp_name.to_lowercase();
                let system_libs = ["user32", "gdi32", "kernel32", "shell32", "comdlg32", "gdi"];
                if system_libs.contains(&base_name) || system_libs.contains(&name_lower.as_str()) {
                    axon_ir::schema::types::ComponentType::SystemLibrary
                } else {
                    axon_ir::schema::types::ComponentType::ProjectModule
                }
            }
        };

        let comp_id = format!("comp:{}", comp_name);
        self.add_node(&comp_id, NodeType::Component, comp_role, is_blocking, component_type);

        // v0.0.29: Support both 'file' (legacy/RawComponent) and 'file_path' (ProjectIR/Component)
        let file_path_opt = comp.get("file").or_else(|| comp.get("file_path")).and_then(|f| f.as_str());
        
        if let Some(file_path) = file_path_opt {
            let file_id = format!("file:{}", file_path);
            self.add_node(&file_id, NodeType::File, comp_role, is_blocking, component_type);
            self.add_edge(&comp_id, &file_id);

            if let Some(funcs) = comp.get("functions").and_then(|f| f.as_array()) {
                for func in funcs {
                    let func_name = func.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let func_id = format!("func:{}::{}", file_path, func_name);
                    self.add_node(&func_id, NodeType::Function, comp_role, is_blocking, component_type);
                    self.add_edge(&file_id, &func_id);
                }
            } else if let Some(funcs_map) = comp.get("functions").and_then(|f| f.as_object()) {
                // v0.0.29: Support BTreeMap structure from ProjectIR
                for (func_name, _func_obj) in funcs_map {
                    let func_id = format!("func:{}::{}", file_path, func_name);
                    self.add_node(&func_id, NodeType::Function, comp_role, is_blocking, component_type);
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

    pub fn generate_cmake(&self, project_name: &str, locale: &str, sandbox_root: &std::path::Path, cmake_spec: Option<&CMakeSpec>) -> String {
        let mut out = String::new();
        let header = if locale == "ko_KR" {
            "# AXON v0.0.31 SOVEREIGN 빌드 스크립트 (SPEC-DRIVEN)\n"
        } else if locale == "ja_JP" {
            "# AXON v0.0.31 SOVEREIGN ビルドスクリプト (SPEC-DRIVEN)\n"
        } else {
            "# AXON v0.0.31 SOVEREIGN BUILD SCRIPT (SPEC-DRIVEN)\n"
        };
        out.push_str(header);

        // Use spec-driven values if available, otherwise fallback to defaults
        let cmake_ver = cmake_spec
            .and_then(|s| s.cmake_minimum_required.as_deref())
            .unwrap_or("3.16");
        out.push_str(&format!("cmake_minimum_required(VERSION {})\n", cmake_ver));

        let proj = cmake_spec
            .and_then(|s| s.project_name.as_deref())
            .unwrap_or(project_name);
        out.push_str(&format!("project({})\n\n", proj));

        if let Some(cxx_std) = cmake_spec.and_then(|s| s.cxx_standard) {
            out.push_str(&format!("set(CMAKE_CXX_STANDARD {})\n", cxx_std));
        } else {
            out.push_str("set(CMAKE_C_STANDARD 99)\n");
            out.push_str("set(CMAKE_CXX_STANDARD 17)\n");
        }
        if let Some(ref compiler) = cmake_spec.and_then(|s| s.cxx_compiler.clone()) {
            out.push_str(&format!("set(CMAKE_CXX_COMPILER {})\n", compiler));
        }
        out.push('\n');

        let mut source_files = HashSet::new();
        let mut link_libraries = HashSet::new();

        for (node_id, node) in &self.nodes {
            if let NodeType::File = node.node_type {
                let file_path = node_id.replace("file:", "");
                
                if node.component_type == axon_ir::schema::types::ComponentType::SystemLibrary {
                    let base_name = std::path::Path::new(&file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    link_libraries.insert(base_name);
                    continue;
                } else if node.component_type == axon_ir::schema::types::ComponentType::ExternalRuntime {
                    continue;
                }

                if file_path.ends_with(".c") || file_path.ends_with(".cpp") {
                    let full_path = sandbox_root.join(&file_path);
                    if node.is_blocking || full_path.exists() {
                        source_files.insert(file_path.clone());
                    } else {
                        out.push_str(&format!("# [PRUNED] Optional file missing: {}\n", file_path));
                    }
                }
            }

            if let NodeType::Component = node.node_type {
                if node.component_type == axon_ir::schema::types::ComponentType::SystemLibrary {
                    let comp_name = node_id.replace("comp:", "");
                    link_libraries.insert(comp_name);
                }
            }
        }

        // v0.0.31: Spec-driven pkg-config and find_package
        if let Some(spec) = cmake_spec {
            if !spec.pkg_config_modules.is_empty() {
                out.push_str("find_package(PkgConfig REQUIRED)\n");
                for pkg in &spec.pkg_config_modules {
                    out.push_str(&format!(
                        "pkg_check_modules({} REQUIRED IMPORTED_TARGET \"{} {}\")\n",
                        pkg.name, pkg.name.to_lowercase(), pkg.version
                    ));
                }
                out.push('\n');
            }
            for fp in &spec.find_packages {
                let req = if fp.required { " REQUIRED" } else { "" };
                out.push_str(&format!("find_package({} {}{})\n", fp.name, fp.version, req));
            }
            if !spec.find_packages.is_empty() {
                out.push('\n');
            }
        }

        // v0.0.31: Spec-driven include directories
        out.push_str("include_directories(include)\n\n");

        // v0.0.31: Build Personality Layer
        let is_win32 = self.platform.as_deref() == Some("win32")
            || self.runtime_model.as_deref() == Some("win32_gui")
            || proj.to_lowercase().contains("win32");
        let win32_flag = if is_win32 { " WIN32" } else { "" };

        let mut sources: Vec<String> = source_files.into_iter().collect();
        sources.sort();

        if !sources.is_empty() {
            out.push_str(&format!("add_executable({}{} {})\n", proj, win32_flag, sources.join(" ")));
        } else {
            out.push_str(&format!("add_executable({}{} src/main.c)\n", proj, win32_flag));
        }

        // v0.0.31: Spec-driven link libraries
        if let Some(spec) = cmake_spec {
            // Add spec-defined link libraries
            for lib in &spec.link_libraries {
                link_libraries.insert(lib.clone());
            }
        }

        // v0.0.31: Win32 subsystem libraries
        if is_win32 {
            link_libraries.insert("user32".to_string());
            link_libraries.insert("gdi32".to_string());
            link_libraries.insert("kernel32".to_string());
            out.push_str("if(MSVC)\n");
            out.push_str(&format!("    set_target_properties({} PROPERTIES WIN32_EXECUTABLE TRUE)\n", proj));
            out.push_str("else()\n");
            out.push_str("    set(CMAKE_EXE_LINKER_FLAGS \"${CMAKE_EXE_LINKER_FLAGS} -mwindows\")\n");
            out.push_str("endif()\n\n");
        }

        if !link_libraries.is_empty() {
            let mut libs: Vec<String> = link_libraries.into_iter().collect();
            libs.sort();
            out.push_str(&format!("target_link_libraries({} PRIVATE {})\n", proj, libs.join(" ")));
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
