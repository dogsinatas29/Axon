#[cfg(test)]
mod tests {
    use axon_daemon::governance::circuit_breaker::{FailureBudget, CircuitState, PatchFingerprint};

    #[test]
    fn test_convergence_governor_escalation() {
        let mut budget = FailureBudget::new("parse_ast");
        let fingerprint = PatchFingerprint {
            topology_delta_hash: "A".to_string(),
            signature_delta_hash: "B".to_string(),
            ownership_delta_hash: "C".to_string(),
        };

        for i in 1..5 { // N-1 failures
            let state = budget.record_attempt(&fingerprint);
            assert_eq!(state, CircuitState::Healthy);
        }

        // Nth failure (5)
        let final_state = budget.record_attempt(&fingerprint);
        assert_eq!(final_state, CircuitState::EscalatedToHuman);
    }
}
