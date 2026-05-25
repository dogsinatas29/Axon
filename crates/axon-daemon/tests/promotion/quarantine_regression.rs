use axon_daemon::intelligence::replay::promotion_engine::{PromotionEngine, PromotionStatus};
use axon_daemon::intelligence::replay::metrics_aggregator::AggregatedMetrics;

#[test]
fn test_explainability_on_quarantine() {
    let metrics = AggregatedMetrics {
        determinism_rate: 1.0,
        semantic_integrity_rate: 1.0,
        topology_preservation_rate: 1.0,
        signature_preservation_rate: 1.0,
        anchor_survivability_p95: 0.999,
        locality_ratio_p95: 1.0, 
        printer_entropy_p95: 0.0,
        rollback_recovery_success_rate: 1.0,
        replay_variance: 0.0,
        mutation_entropy_score: 0.8, // Fails here: > 0.5 triggers Quarantine
    };
    
    let decision = PromotionEngine::evaluate(&metrics, "trace_quarantine_001");
    
    assert_eq!(decision.status, PromotionStatus::Quarantine);
    assert_eq!(decision.violated_constraint.unwrap(), "Unstable Mutation Entropy");
    assert_eq!(decision.observed_metric.unwrap(), 0.8);
    assert_eq!(decision.expected_threshold.unwrap(), 0.5);
    // This explainability allows the human Boss to reverse-engineer why the subset failed.
}
