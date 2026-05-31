use serde::{Deserialize, Serialize};

// From corpus/failure_classifier.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CatastropheKind {
    FormatterLocalityCollapse,
    MacroExpansionDrift,
    HiddenTopologyMutation,
    AnchorInvalidation,
    SemanticFalseEquivalence,
    OwnershipLeak,
    ReplayVariance,
    ParserNormalizationDivergence,
    UnknownDrift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureClassification {
    pub kind: CatastropheKind,
    pub determinism_rate: f32,
    pub topology_integrity: f32,
    pub semantic_drift: f32,
    pub raw_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMetrics {
    pub determinism_rate: f32,
    pub topology_integrity: f32,
    pub semantic_drift: f32,
    pub fingerprint: String,
}

impl FailureClassification {
    pub fn classify(metrics: &CampaignMetrics) -> Self {
        let kind = if metrics.determinism_rate < 1.0 {
            CatastropheKind::ReplayVariance
        } else if metrics.semantic_drift > 0.0 {
            CatastropheKind::SemanticFalseEquivalence
        } else if metrics.topology_integrity < 1.0 {
            CatastropheKind::HiddenTopologyMutation
        } else {
            CatastropheKind::UnknownDrift
        };

        Self {
            kind,
            determinism_rate: metrics.determinism_rate,
            topology_integrity: metrics.topology_integrity,
            semantic_drift: metrics.semantic_drift,
            raw_fingerprint: metrics.fingerprint.clone(),
        }
    }
}

// From corpus/corpus_fingerprint.rs (Entropy Fingerprint)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorpusFingerprint {
    pub topology_density: f64,
    pub callback_depth: usize,
    pub macro_entropy: f64,
    pub include_graph_complexity: usize,
    pub runtime_event_fanout: f64,
    pub ownership_ambiguity: f64,
}

impl CorpusFingerprint {
    pub fn new() -> Self {
        Self {
            topology_density: 0.0,
            callback_depth: 0,
            macro_entropy: 0.0,
            include_graph_complexity: 0,
            runtime_event_fanout: 0.0,
            ownership_ambiguity: 0.0,
        }
    }

    pub fn from_abandoned_gtk2() -> Self {
        Self {
            topology_density: 0.95,
            callback_depth: 12,
            macro_entropy: 0.88,
            include_graph_complexity: 1500,
            runtime_event_fanout: 0.92,
            ownership_ambiguity: 0.85,
        }
    }
}

// From replay/corpus_fingerprint.rs (Failure/Catastrophic Signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusFailureFingerprint {
    pub fingerprint_id: String,
    pub trigger: String,
    pub failure_mode: String,
    pub tree_sitter_version: String,
    pub semantic_distance: f64,
    pub reproducible: bool,
}

pub struct FingerprintRegistry;

impl FingerprintRegistry {
    pub fn register_fingerprint(fingerprint: CorpusFailureFingerprint) {
        println!("Registered catastrophic fingerprint: {:?}", fingerprint);
    }
}
