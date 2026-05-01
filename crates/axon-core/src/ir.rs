use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIR {
    pub components: HashMap<String, Component>, // key = component.name
    pub constraints: Vec<crate::rules::Constraint>,
    #[serde(skip)] // Not persisted in MD directly
    pub constraint_ids: std::collections::HashSet<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub file_path: String,
    pub functions: HashMap<String, Function>, // key = function.name
    pub imports: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub signature: String,
    pub dependencies: HashSet<String>,
    pub body_hash: Option<u64>, // optional (fast compare)
}

impl ProjectIR {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            constraints: Vec::new(),
            constraint_ids: std::collections::HashSet::new(),
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
                struct Components { components: Vec<RawComponent> }
                #[derive(Deserialize)]
                struct RawComponent { file: String, name: String, symbols: Vec<String>, #[serde(rename = "type")] _type: String }

                if let Ok(raw) = serde_json::from_str::<Components>(json_str) {
                    let mut components = HashMap::new();
                    for c in raw.components {
                        let mut functions = HashMap::new();
                        for s in c.symbols {
                            functions.insert(s.clone(), Function {
                                name: s.clone(),
                                signature: format!("{}()", s),
                                dependencies: HashSet::new(),
                                body_hash: None,
                            });
                        }
                        components.insert(c.name.clone(), Component {
                            name: c.name,
                            file_path: c.file,
                            functions,
                            imports: HashSet::new(),
                        });
                    }

                    // --- New: Parse Constraints ---
                    let mut constraints = Vec::new();
                    let constraint_tag = "<!-- AXON:CONSTRAINTS";
                    if let Some(c_start) = md.find(constraint_tag) {
                        let c_json_start = c_start + constraint_tag.len();
                        if let Some(c_end) = md[c_json_start..].find(end_tag) {
                            let c_json_str = md[c_json_start..c_json_start + c_end].trim();
                            if let Ok(c_list) = serde_json::from_str::<Vec<crate::rules::Constraint>>(c_json_str) {
                                constraints = c_list;
                            }
                        }
                    }

                    return Some(ProjectIR { components, constraints, constraint_ids: std::collections::HashSet::new() });
                }
            }
        }
        None
    }
}
