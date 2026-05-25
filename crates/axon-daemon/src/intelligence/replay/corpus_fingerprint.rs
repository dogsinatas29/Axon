use serde::{Deserialize, Serialize};

/// P5-8g.2: Corpus Fingerprint Registry
/// Records exact "catastrophic failure signatures" across open-source codebases (Tokio, Django, SQLite, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusFingerprint {
    /// A human-readable identifier for the catastrophic pattern.
    /// e.g. "rust.macro.where_clause.attr_shuffle.v2"
    pub fingerprint_id: String,
    
    /// Which intent triggered this?
    pub trigger: String,
    
    /// e.g. "AnchorInstability", "TopologyExplosion", "FormatterEntropySpike"
    pub failure_mode: String,
    
    pub tree_sitter_version: String,
    pub semantic_distance: f64,
    pub reproducible: bool,
}

pub struct FingerprintRegistry;

impl FingerprintRegistry {
    /// Registers a newly discovered catastrophic failure pattern so AXON remembers "where to not cut".
    pub fn register_fingerprint(fingerprint: CorpusFingerprint) {
        // Stub: Append to a global catastrophic registry
        println!("Registered catastrophic fingerprint: {:?}", fingerprint);
    }
}
