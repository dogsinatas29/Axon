use serde::{Deserialize, Serialize};

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
