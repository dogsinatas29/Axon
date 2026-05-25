use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromotionStatus {
    ALLOW,
    DENY,
    ShadowOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructMetrics {
    pub stability: f32,
    pub topology_risk: f32,
    pub normalization_drift: f32,
    pub replay_count: usize,
    pub promotion: PromotionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageHeatmap {
    pub constructs: HashMap<String, ConstructMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationHeatmap {
    pub languages: HashMap<String, LanguageHeatmap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionGate {
    pub min_stability_score: f32,
    pub max_drift_density: f32,
    pub required_replay_count: usize,
    pub topology_integrity_threshold: f32,
}

impl PromotionGate {
    pub fn default_strict() -> Self {
        Self {
            min_stability_score: 0.99,
            max_drift_density: 0.01,
            required_replay_count: 10000,
            topology_integrity_threshold: 0.999,
        }
    }

    pub fn evaluate_promotion(&self, replays: usize, stability: f32, drift: f32, topology: f32) -> PromotionStatus {
        if replays >= self.required_replay_count 
            && stability >= self.min_stability_score 
            && drift <= self.max_drift_density 
            && topology >= self.topology_integrity_threshold 
        {
            PromotionStatus::ALLOW
        } else {
            PromotionStatus::DENY
        }
    }
}
