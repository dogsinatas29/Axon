use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: u64,
    pub kind: String,
    pub target: String,
    pub condition: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIR {
    #[serde(default)]
    pub node_mapping: BTreeMap<String, String>,
    pub components: BTreeMap<String, Component>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    #[serde(skip)]
    pub constraint_ids: std::collections::HashSet<u64>,
    #[serde(default)]
    pub thought: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub file_path: String,
    pub functions: BTreeMap<String, Function>,
    pub imports: BTreeSet<String>,
    #[serde(default)]
    pub associated_files: Vec<String>,
    #[serde(default)]
    pub is_entrypoint: bool,
    #[serde(default)]
    pub data_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub signature: String,
    pub dependencies: BTreeSet<String>,
    pub body_hash: Option<u64>,
}

impl ProjectIR {
    pub fn new() -> Self {
        Self {
            node_mapping: BTreeMap::new(),
            components: BTreeMap::new(),
            constraints: Vec::new(),
            constraint_ids: std::collections::HashSet::new(),
            thought: None,
        }
    }

    pub fn from_md(md: &str) -> Option<Self> {
        let start_tag = "<!-- AXON:SPEC:COMPONENTS";
        let end_tag = "-->";

        if let Some(start_idx) = md.find(start_tag) {
            let json_start = start_idx + start_tag.len();
            if let Some(end_idx) = md[json_start..].find(end_tag) {
                let json_str = md[json_start..json_start + end_idx].trim();

                #[derive(Deserialize)]
                struct Components {
                    #[serde(default)]
                    node_mapping: BTreeMap<String, String>,
                    components: Vec<RawComponent>
                }
                #[derive(Deserialize)]
                struct RawComponent {
                    file: String,
                    name: String,
                    symbols: Vec<String>,
                    #[serde(rename = "type")] _type: String
                }

                if let Ok(raw) = serde_json::from_str::<Components>(json_str) {
                    let mut components = BTreeMap::new();
                    for c in raw.components {
                        let mut functions = BTreeMap::new();
                        for s in c.symbols {
                            functions.insert(s.clone(), Function {
                                name: s.clone(),
                                signature: format!("{}()", s),
                                dependencies: BTreeSet::new(),
                                body_hash: None,
                            });
                        }
                        let canonical_key = crate::canonicalizer::canonicalize_path(&c.file);
                        let comp_name = c.name.clone();
                        components.insert(canonical_key.clone(), Component {
                            name: comp_name.clone(),
                            file_path: c.file,
                            functions,
                            imports: BTreeSet::new(),
                            associated_files: Vec::new(),
                            is_entrypoint: false,
                            data_models: Vec::new(),
                        });
                        tracing::debug!("[IR_REGISTER] key={} name={}", canonical_key, comp_name);
                    }

                    let mut constraints = Vec::new();
                    let constraint_tag = "<!-- AXON:CONSTRAINTS";
                    if let Some(c_start) = md.find(constraint_tag) {
                        let c_json_start = c_start + constraint_tag.len();
                        if let Some(c_end) = md[c_json_start..].find(end_tag) {
                            let c_json_str = md[c_json_start..c_json_start + c_end].trim();
                            if let Ok(c_list) = serde_json::from_str::<Vec<Constraint>>(c_json_str) {
                                constraints = c_list;
                            }
                        }
                    }

                    return Some(ProjectIR {
                        node_mapping: raw.node_mapping,
                        components,
                        constraints,
                        constraint_ids: std::collections::HashSet::new(),
                        thought: None,
                    });
                }
            }
        }
        None
    }

    pub fn get_component(&self, canonical_path: &str) -> Option<&Component> {
        let key = crate::canonicalizer::canonicalize_path(canonical_path);
        self.components.get(&key)
    }

    pub fn get_all_keys(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }
}

impl Default for ProjectIR {
    fn default() -> Self {
        Self::new()
    }
}