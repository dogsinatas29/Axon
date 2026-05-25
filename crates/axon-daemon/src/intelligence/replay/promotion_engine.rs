use super::metrics_aggregator::AggregatedMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionStatus {
    Approved,
    ShadowOnly,
    Quarantine,
    Reject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromotionDecision {
    pub status: PromotionStatus,
    pub violated_constraint: Option<String>,
    pub observed_metric: Option<f64>,
    pub expected_threshold: Option<f64>,
    pub replay_trace_id: Option<String>,
    pub corpus_case: Option<String>,
    pub semantic_diff_summary: Option<String>,
}

pub struct PromotionEngine;

impl PromotionEngine {
    /// The "Mutation Constitutional Law".
    /// Evaluates statistical integrity and provides full explainability on failures.
    pub fn evaluate(metrics: &AggregatedMetrics, trace_id: &str) -> PromotionDecision {
        if metrics.determinism_rate < 1.0 {
            return Self::fail(PromotionStatus::Reject, "Determinism < 100%", metrics.determinism_rate, 1.0, trace_id);
        }
        if metrics.topology_preservation_rate < 0.999 {
            return Self::fail(PromotionStatus::Reject, "Topology Integrity Failed", metrics.topology_preservation_rate, 0.999, trace_id);
        }
        if metrics.signature_preservation_rate < 1.0 {
            return Self::fail(PromotionStatus::Reject, "Signature Drift Detected", metrics.signature_preservation_rate, 1.0, trace_id);
        }
        if metrics.replay_variance > 0.0 {
            return Self::fail(PromotionStatus::ShadowOnly, "Replay Variance > 0", metrics.replay_variance, 0.0, trace_id);
        }
        if metrics.locality_ratio_p95 > 1.05 { // Unexpected formatting drift
            return Self::fail(PromotionStatus::ShadowOnly, "Locality Drift > 5%", metrics.locality_ratio_p95, 1.05, trace_id);
        }
        if metrics.printer_entropy_p95 > 0.05 { // E.g., formatting completely rewritten
            return Self::fail(PromotionStatus::ShadowOnly, "High Formatting Entropy", metrics.printer_entropy_p95, 0.05, trace_id);
        }
        if metrics.mutation_entropy_score > 0.5 { // Overly aggressive mutations
            return Self::fail(PromotionStatus::Quarantine, "Unstable Mutation Entropy", metrics.mutation_entropy_score, 0.5, trace_id);
        }
        
        PromotionDecision {
            status: PromotionStatus::Approved,
            violated_constraint: None,
            observed_metric: None,
            expected_threshold: None,
            replay_trace_id: Some(trace_id.to_string()),
            corpus_case: None,
            semantic_diff_summary: None,
        }
    }

    fn fail(status: PromotionStatus, reason: &str, observed: f64, expected: f64, trace: &str) -> PromotionDecision {
        PromotionDecision {
            status,
            violated_constraint: Some(reason.to_string()),
            observed_metric: Some(observed),
            expected_threshold: Some(expected),
            replay_trace_id: Some(trace.to_string()),
            corpus_case: Some("Unknown (Aggregated)".to_string()), // Will be enriched by Orchestrator
            semantic_diff_summary: None,
        }
    }
}
