use serde::{Deserialize, Serialize};
use super::ast::hash_types::CompositeSymbolHash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtripMetrics {
    pub formatting_drift_detected: bool,
    pub semantic_drift_detected: bool,
    pub topology_drift_detected: bool,
    pub ownership_drift_detected: bool,
    pub parser_normalization_drift: bool,
}

pub struct StructuralRoundtripValidator;

impl StructuralRoundtripValidator {
    pub fn validate_roundtrip(
        original_hash: &CompositeSymbolHash,
        reparsed_hash: &CompositeSymbolHash,
    ) -> RoundtripMetrics {
        let formatting_drift = original_hash.raw_text_hash != reparsed_hash.raw_text_hash 
            && original_hash.normalized_ast_hash == reparsed_hash.normalized_ast_hash;
            
        let parser_normalization_drift = original_hash.normalized_ast_hash != reparsed_hash.normalized_ast_hash;
        let topology_drift = original_hash.topology_hash != reparsed_hash.topology_hash;
        let semantic_drift = original_hash.signature_hash != reparsed_hash.signature_hash;
        
        let ownership_drift = parser_normalization_drift || topology_drift || semantic_drift;

        RoundtripMetrics {
            formatting_drift_detected: formatting_drift,
            semantic_drift_detected: semantic_drift,
            topology_drift_detected: topology_drift,
            ownership_drift_detected: ownership_drift,
            parser_normalization_drift,
        }
    }
}
