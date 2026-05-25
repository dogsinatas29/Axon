use serde::Serialize;
use sha2::{Sha256, Digest};

/// F-5 & PHASE G: 2-Layer Trace Determinism
/// Decouples static parsing/topology drifts from dynamic runtime execution drifts.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct TraceLayering {
    pub topology_trace_hash: String,
    pub runtime_trace_hash: String,
    pub combined_state_hash: String,
}

impl TraceLayering {
    /// Computes the exact failure lineage separating the topology and runtime layers.
    pub fn compute(topology_events: &[String], runtime_events: &[String]) -> Self {
        let topo_hash = Self::hash_events(topology_events);
        let runtime_hash = Self::hash_events(runtime_events);
        
        let mut hasher = Sha256::new();
        hasher.update(format!("{}_{}", topo_hash, runtime_hash).as_bytes());
        let hash_bytes = hasher.finalize();
        
        let mut combined = String::with_capacity(64);
        for byte in hash_bytes {
            combined.push_str(&format!("{:02x}", byte));
        }

        Self {
            topology_trace_hash: topo_hash,
            runtime_trace_hash: runtime_hash,
            combined_state_hash: combined,
        }
    }

    fn hash_events(events: &[String]) -> String {
        let mut hasher = Sha256::new();
        for e in events {
            hasher.update(e.as_bytes());
        }
        let hash_bytes = hasher.finalize();
        let mut hash_str = String::with_capacity(64);
        for byte in hash_bytes {
            hash_str.push_str(&format!("{:02x}", byte));
        }
        hash_str
    }
}

/// PHASE G: Catastrophe Immunology
/// Identifies and clusters failure genealogies.
pub struct ImmunologyObservatory {
    pub known_collapse_families: std::collections::HashMap<String, String>, // Combined Hash -> Genealogy Name
}

impl ImmunologyObservatory {
    pub fn new() -> Self {
        Self {
            known_collapse_families: std::collections::HashMap::new(),
        }
    }

    pub fn register_catastrophe(&mut self, trace: &TraceLayering, lineage_name: &str) {
        self.known_collapse_families.insert(trace.combined_state_hash.clone(), lineage_name.to_string());
    }

    pub fn diagnose_collapse(&self, trace: &TraceLayering) -> String {
        if let Some(lineage) = self.known_collapse_families.get(&trace.combined_state_hash) {
            format!("MATCHED_FAMILY: {}", lineage)
        } else {
            "UNKNOWN_MUTATION_ORPHAN".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_layer_separation() {
        // Even if topology is identical, runtime drift changes the combined state.
        let topo_events = vec!["ownership_widget_A".to_string(), "signal_click_B".to_string()];
        let run_events_1 = vec!["idle_add_cleanup".to_string(), "execute_cleanup".to_string()];
        let run_events_2 = vec!["execute_cleanup".to_string(), "idle_add_cleanup".to_string()]; // Out of order!

        let trace1 = TraceLayering::compute(&topo_events, &run_events_1);
        let trace2 = TraceLayering::compute(&topo_events, &run_events_2);

        assert_eq!(trace1.topology_trace_hash, trace2.topology_trace_hash, "Topology is identical");
        assert_ne!(trace1.runtime_trace_hash, trace2.runtime_trace_hash, "Runtime ordering drifted");
        assert_ne!(trace1.combined_state_hash, trace2.combined_state_hash, "Combined hash must diverge");
    }

    #[test]
    fn test_immunology_observatory_clustering() {
        let mut observatory = ImmunologyObservatory::new();
        let trace = TraceLayering::compute(&["topo1".to_string()], &["run1".to_string()]);
        
        observatory.register_catastrophe(&trace, "GTK2_REENTRANT_DOUBLE_FREE_V1");
        
        let diagnostic = observatory.diagnose_collapse(&trace);
        assert_eq!(diagnostic, "MATCHED_FAMILY: GTK2_REENTRANT_DOUBLE_FREE_V1");
    }
}
