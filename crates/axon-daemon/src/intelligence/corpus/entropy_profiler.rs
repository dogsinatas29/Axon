use serde::{Deserialize, Serialize};

/// Identifies "why this file is dangerous" before any mutation happens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyProfile {
    pub file_path: String,
    
    // Metrics
    pub macro_density: f64,
    pub generic_nesting_depth: usize,
    pub cfg_fragmentation_score: f64,
    pub decorator_stack_depth: usize,
    pub preprocessor_branch_factor: f64,
    pub comment_locality_fragility: f64,
    pub formatting_irregularity_score: f64,
    pub anchor_instability_probability: f64,
    
    // Aggregation
    pub total_entropy_score: f64,
    pub risk_class: String, // "LOW", "MEDIUM", "HIGH", "CRITICAL"
}

pub struct EntropyProfiler;

impl EntropyProfiler {
    pub fn profile_file(file_path: &str) -> EntropyProfile {
        // Stub: Structural scanner extracts these metrics
        EntropyProfile {
            file_path: file_path.to_string(),
            macro_density: 0.15,
            generic_nesting_depth: 3,
            cfg_fragmentation_score: 0.4,
            decorator_stack_depth: 0,
            preprocessor_branch_factor: 0.2,
            comment_locality_fragility: 0.6,
            formatting_irregularity_score: 0.8,
            anchor_instability_probability: 0.3,
            total_entropy_score: 0.82,
            risk_class: "HIGH".into(),
        }
    }
}
