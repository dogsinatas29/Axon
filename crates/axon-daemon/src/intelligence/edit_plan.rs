use serde::{Deserialize, Serialize};

/// Represents a surgical text modification bypassing the AST Printer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteEdit {
    pub start_byte: usize,
    pub end_byte: usize,
    pub new_content: String,
}

/// The lowered execution plan from a SemanticIntent.
/// Maps intent to exact byte ranges and mandates strict hash expectations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableEditPlan {
    pub target_symbol: String,
    pub edits: Vec<ByteEdit>,
    pub expected_semantic_hash: String,
    pub expected_topology_hash: String,
}
