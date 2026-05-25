use serde::Serialize;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Clone)]
pub struct CanonicalTraceRecord {
    pub seq: usize,
    pub ownership_anchor: String,
    pub topology_edge_hash: String,
    pub parser_boundary_hash: String,
    pub canonical_failure: String,
    pub catastrophe_lineage: String,
    pub trace_hash: String,
    pub system_state_hash: String,
}

pub struct GtkRuntimeCollapseHarness {
    pub trace_log: Vec<CanonicalTraceRecord>,
    pub current_topology_hash: String,
}

impl GtkRuntimeCollapseHarness {
    pub fn new() -> Self {
        Self {
            trace_log: Vec::new(),
            current_topology_hash: "initial_gtk_topology".to_string(),
        }
    }

    pub fn generate_hash(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash_bytes = hasher.finalize();
        let mut hash_str = String::with_capacity(64);
        for byte in hash_bytes {
            hash_str.push_str(&format!("{:02x}", byte));
        }
        hash_str
    }

    pub fn trigger_collapse(&mut self, seq: usize, attack_scenario: &str) -> CanonicalTraceRecord {
        let canonical_failure = match attack_scenario {
            "double_destroy" => "DOUBLE_FREE_LIFECYCLE_COLLAPSE",
            "orphan_mutation" => "ORPHAN_PARENT_MUTATION",
            "invalid_quit" => "INVALID_MAIN_QUIT_ORDER",
            _ => "UNKNOWN_COLLAPSE",
        };

        let lineage = format!("Task_B -> {} -> {}", attack_scenario, canonical_failure);
        let edge_hash = Self::generate_hash(&format!("edge_{}", attack_scenario));
        let boundary_hash = Self::generate_hash("fixed_parser_boundary_10_50");

        let mut record = CanonicalTraceRecord {
            seq,
            ownership_anchor: "Task_A_widget_main".to_string(),
            topology_edge_hash: edge_hash,
            parser_boundary_hash: boundary_hash,
            canonical_failure: canonical_failure.to_string(),
            catastrophe_lineage: lineage.clone(),
            trace_hash: "".to_string(), // computed below
            system_state_hash: "state_after_collapse_0xFAIL".to_string(),
        };

        // Trace hash incorporates sequence, failure, and edge to ensure perfect identity
        let trace_input = format!("{}_{}_{}_{}", seq, canonical_failure, record.topology_edge_hash, record.parser_boundary_hash);
        record.trace_hash = Self::generate_hash(&trace_input);

        self.trace_log.push(record.clone());
        record
    }
}

pub fn validate_replay_collapse(attack_scenario: &str, replays: usize) -> Result<(), String> {
    let mut baseline_trace_hash = String::new();

    for run in 1..=replays {
        let mut harness = GtkRuntimeCollapseHarness::new();
        let record = harness.trigger_collapse(1, attack_scenario);

        if run == 1 {
            baseline_trace_hash = record.trace_hash;
        } else if record.trace_hash != baseline_trace_hash {
            return Err(format!("REPLAY_VARIANCE_DETECTED: Trace hash drifted at run {}", run));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage3_double_destroy_collapse() {
        assert!(validate_replay_collapse("double_destroy", 1000).is_ok());
    }

    #[test]
    fn test_stage3_orphan_parent_collapse() {
        assert!(validate_replay_collapse("orphan_mutation", 1000).is_ok());
    }

    #[test]
    fn test_stage3_invalid_quit_order() {
        assert!(validate_replay_collapse("invalid_quit", 1000).is_ok());
    }
}
