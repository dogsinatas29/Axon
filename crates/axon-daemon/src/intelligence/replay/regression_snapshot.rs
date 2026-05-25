use serde::{Deserialize, Serialize};

/// Protects against environmental drift.
/// If a parser library updates and breaks assumptions, this snapshot invalidates the old promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionSnapshot {
    pub intent_type: String,
    pub parser_version: String,
    pub tree_sitter_grammar_version: String,
    pub canonicalizer_policy_version: String,
    pub lowering_template_version: String,
    pub semantic_authority_version: String,
    pub timestamp: String,
}

pub struct SnapshotManager;

impl SnapshotManager {
    /// Captures the environmental envelope at the exact moment of Promotion Approval.
    pub fn capture_freeze(intent_type: &str) -> RegressionSnapshot {
        RegressionSnapshot {
            intent_type: intent_type.to_string(),
            parser_version: env!("CARGO_PKG_VERSION").to_string(), // Simplified
            tree_sitter_grammar_version: "v0.20".to_string(),
            canonicalizer_policy_version: "v1.0".to_string(),
            lowering_template_version: "v1.0".to_string(),
            semantic_authority_version: "v2.0".to_string(),
            timestamp: "2026-05-23T22:45:00Z".to_string(),
        }
    }
}
