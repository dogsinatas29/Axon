// v0.0.31: Phase 5 — Topology Formalization
// 
// Scope: runtime orchestration-level topology only.
// NOT compiler IR (no borrow graph, no MIR, no ownership analysis).
//
// Goal: give the AXON pipeline enough structural information to:
//   - Validate module legality (no circular imports)
//   - Enforce header ownership (C)
//   - Verify crate entry point exists (Rust)
//   - Detect translation unit violations (C)

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

/// Minimal topology model for a single module/file unit.
/// Language-agnostic at this level; language-specific fields are in subtypes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleTopology {
    /// The canonical file path (key in ProjectIR.components)
    pub file_path: String,

    /// Modules this unit directly imports/includes
    pub depends_on: BTreeSet<String>,

    /// Modules that import this unit (reverse dependency)
    pub depended_by: BTreeSet<String>,

    /// Language-specific topology metadata
    pub meta: TopologyMeta,
}

/// Language-specific topology metadata (discriminated via ProjectIR.language)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopologyMeta {
    // --- Rust-specific ---
    /// Whether this module is the crate entry (main.rs or lib.rs)
    pub is_crate_entry: bool,
    /// Whether this is a mod.rs (module declaration file)
    pub is_mod_decl: bool,
    /// The declared module path (e.g. "crate::utils::parser")
    pub mod_path: Option<String>,

    // --- C-specific ---
    /// Whether this is a header (.h / .hpp)
    pub is_header: bool,
    /// The translation unit that owns this header (if any)
    pub owner_translation_unit: Option<String>,
    /// Whether this is the primary compilation entry (contains main())
    pub is_compile_entry: bool,
}

/// Project-level topology: dependency graph over all modules.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectTopology {
    /// Module topology map: file_path -> ModuleTopology
    pub modules: BTreeMap<String, ModuleTopology>,

    /// Topological sort order (leaf-first, entry-last)
    pub build_order: Vec<String>,

    /// Detected cycle paths (empty = no cycles)
    pub cycles: Vec<Vec<String>>,
}

impl ProjectTopology {
    /// Build topology from a ProjectIR component map.
    /// Derives depends_on from Component.imports and Component.allowed_includes.
    pub fn from_components(
        components: &BTreeMap<String, super::types::Component>,
        language: super::types::Language,
    ) -> Self {
        let mut modules: BTreeMap<String, ModuleTopology> = BTreeMap::new();

        // First pass: create nodes
        for (path, comp) in components {
            let p = path.to_lowercase();
            let meta = match language {
                super::types::Language::Rust | super::types::Language::Cpp => TopologyMeta {
                    is_crate_entry: comp.is_entrypoint || p.contains("main.rs") || p.contains("lib.rs"),
                    is_mod_decl: p.ends_with("mod.rs"),
                    mod_path: None,
                    is_header: false,
                    owner_translation_unit: None,
                    is_compile_entry: comp.is_entrypoint,
                },
                super::types::Language::C => TopologyMeta {
                    is_crate_entry: false,
                    is_mod_decl: false,
                    mod_path: None,
                    is_header: p.ends_with(".h") || p.ends_with(".hpp"),
                    owner_translation_unit: if p.ends_with(".h") {
                        // Infer owner: same name with .c extension
                        let stem = path.trim_end_matches(".h").trim_end_matches(".hpp");
                        let c_path = format!("{}.c", stem);
                        if components.contains_key(&c_path) { Some(c_path) } else { None }
                    } else { None },
                    is_compile_entry: comp.is_entrypoint,
                },
                super::types::Language::Python => TopologyMeta {
                    is_crate_entry: comp.is_entrypoint || p.contains("main.py"),
                    is_mod_decl: p.contains("__init__.py"),
                    mod_path: None,
                    is_header: false,
                    owner_translation_unit: None,
                    is_compile_entry: comp.is_entrypoint,
                },
                super::types::Language::Lua => TopologyMeta {
                    is_crate_entry: comp.is_entrypoint || p.contains("main.lua"),
                    is_mod_decl: false,
                    mod_path: None,
                    is_header: false,
                    owner_translation_unit: None,
                    is_compile_entry: comp.is_entrypoint,
                },
            };

            modules.insert(path.clone(), ModuleTopology {
                file_path: path.clone(),
                depends_on: comp.imports.clone(),
                depended_by: BTreeSet::new(),
                meta,
            });
        }

        // Second pass: populate reverse edges (depended_by)
        let forward_edges: Vec<(String, String)> = modules
            .iter()
            .flat_map(|(from, m)| m.depends_on.iter().map(move |to| (from.clone(), to.clone())))
            .collect();
        for (from, to) in forward_edges {
            if let Some(m) = modules.get_mut(&to) {
                m.depended_by.insert(from);
            }
        }

        // Third pass: topological sort (Kahn's algorithm)
        let build_order = Self::topo_sort(&modules);
        let cycles = Self::detect_cycles(&modules);

        ProjectTopology { modules, build_order, cycles }
    }

    /// Kahn's BFS topological sort. Returns leaf-first order.
    fn topo_sort(modules: &BTreeMap<String, ModuleTopology>) -> Vec<String> {
        let mut in_degree: BTreeMap<&str, usize> = modules
            .keys()
            .map(|k| (k.as_str(), 0))
            .collect();

        for m in modules.values() {
            for dep in &m.depends_on {
                if modules.contains_key(dep) {
                    *in_degree.entry(dep.as_str()).or_insert(0) += 0; // ensure exists
                    // count reverse: who depends on dep
                }
            }
        }
        // Recount: in_degree = number of modules that THIS module depends on (which are in the set)
        for m in modules.values() {
            let count = m.depends_on.iter().filter(|d| modules.contains_key(*d)).count();
            in_degree.insert(m.file_path.as_str(), count);
        }

        let mut queue: std::collections::VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(k, _)| k.to_string())
            .collect();

        let mut result = Vec::new();
        let mut remaining = in_degree;

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            if let Some(m) = modules.get(&node) {
                for dependent in &m.depended_by {
                    if let Some(deg) = remaining.get_mut(dependent.as_str()) {
                        if *deg > 0 { *deg -= 1; }
                        if *deg == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }
        result
    }

    /// Simple cycle detection: nodes not in topo sort are part of cycles.
    fn detect_cycles(modules: &BTreeMap<String, ModuleTopology>) -> Vec<Vec<String>> {
        // Placeholder: full DFS cycle detection for future topology hardening
        // For now: any node with self-import is a trivial cycle
        let mut cycles = Vec::new();
        for (path, m) in modules {
            if m.depends_on.contains(path) {
                cycles.push(vec![path.clone()]);
            }
        }
        cycles
    }

    /// Check if Rust topology is valid: must have exactly one crate entry.
    pub fn validate_rust(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let entries: Vec<_> = self.modules.values().filter(|m| m.meta.is_crate_entry).collect();
        if entries.is_empty() {
            errors.push("Rust topology: no crate entry point (main.rs or lib.rs) found".to_string());
        } else if entries.len() > 1 {
            let paths: Vec<_> = entries.iter().map(|m| m.file_path.clone()).collect();
            errors.push(format!("Rust topology: multiple crate entries detected: {:?}", paths));
        }
        if !self.cycles.is_empty() {
            errors.push(format!("Rust topology: circular module dependencies detected: {:?}", self.cycles));
        }
        errors
    }

    /// Check if C topology is valid: translation units must not form cycles.
    pub fn validate_c(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !self.cycles.is_empty() {
            errors.push(format!("C topology: circular header dependencies detected: {:?}", self.cycles));
        }
        // Orphan headers: headers with no owner translation unit
        for m in self.modules.values() {
            if m.meta.is_header && m.meta.owner_translation_unit.is_none() && m.depended_by.is_empty() {
                errors.push(format!("C topology: orphan header with no owner or consumer: {}", m.file_path));
            }
        }
        errors
    }
}
