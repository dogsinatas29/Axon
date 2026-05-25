use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompositeSymbolHash {
    pub raw_text_hash: String,
    pub normalized_ast_hash: String,
    pub topology_hash: String,
    pub signature_hash: String,
}

impl CompositeSymbolHash {
    pub fn new(raw: &str, ast: &str, topo: &str, sig: &str) -> Self {
        Self {
            raw_text_hash: raw.to_string(),
            normalized_ast_hash: ast.to_string(),
            topology_hash: topo.to_string(),
            signature_hash: sig.to_string(),
        }
    }
}
