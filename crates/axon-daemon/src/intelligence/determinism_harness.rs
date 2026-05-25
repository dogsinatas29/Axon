use super::causality::StateTransitionRecord;

pub struct DeterminismVerificationHarness;

impl DeterminismVerificationHarness {
    /// Compares two transition records derived from the same inputs.
    /// If they differ in critical fields, it indicates nondeterminism 
    /// (e.g., HashMap iteration order, async races, timestamp contamination).
    pub fn verify_determinism(run_a: &StateTransitionRecord, run_b: &StateTransitionRecord) -> Result<(), String> {
        // Compare everything except the timestamp
        let mut clone_a = run_a.clone();
        let mut clone_b = run_b.clone();
        
        clone_a.timestamp = 0;
        clone_b.timestamp = 0;

        let a_json = serde_json::to_string(&clone_a).unwrap();
        let b_json = serde_json::to_string(&clone_b).unwrap();

        if a_json != b_json {
            Err(format!(
                "Nondeterminism detected!\nRun A: {}\nRun B: {}", 
                a_json, b_json
            ))
        } else {
            Ok(())
        }
    }
}
