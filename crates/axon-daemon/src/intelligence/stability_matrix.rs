use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityMetrics {
    // 1. Roundtrip Stability Score: stable_symbols / total_symbols
    pub roundtrip_stability_score: f64,
    // 2. Ownership Preservation Rate: preserved_ownership_hashes / total_ownership_regions
    pub ownership_preservation_rate: f64,
    // 3. Topology Integrity Rate: unchanged_edges / total_edges
    pub topology_integrity_rate: f64,
    // 4. Signature Stability Rate: preserved_signatures / total_signatures
    pub signature_stability_rate: f64,
    // 5. Normalization Drift Density: nonsemantic_ast_changes / total_ast_nodes
    pub normalization_drift_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedStability {
    pub topology: bool,
    pub signature: bool,
    pub ownership: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    pub name: String,
    pub language: String,
    pub source_file: String,
    pub mutation_target: String,
    pub expected_stability: ExpectedStability,
}

pub struct StabilityMatrixHarness;

impl StabilityMatrixHarness {
    pub fn evaluate_corpus(_project_root: &Path, _corpus_entry: &CorpusEntry) -> StabilityMetrics {
        // Pseudo implementation for the matrix harness
        // Reads corpus, executes shadow AST mutator, gathers metrics
        StabilityMetrics {
            roundtrip_stability_score: 1.0,
            ownership_preservation_rate: 1.0,
            topology_integrity_rate: 1.0,
            signature_stability_rate: 1.0,
            normalization_drift_density: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftFingerprint {
    pub drift_type: String,
    pub language: String,
    pub construct: String,
    pub severity: String,
    pub reproducible: bool,
    pub ownership_break: bool,
    pub topology_break: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeSubsetPolicy {
    pub language: String,
    pub allowed_constructs: Vec<String>,
    pub denied_constructs: Vec<String>,
}

impl SafeSubsetPolicy {
    pub fn is_allowed(&self, construct: &str) -> bool {
        self.allowed_constructs.contains(&construct.to_string())
    }
}
