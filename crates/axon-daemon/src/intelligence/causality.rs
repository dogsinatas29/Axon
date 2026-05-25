use serde::{Deserialize, Serialize};
use super::provenance::PatchProvenance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerDecision {
    pub selected_task: String,
    pub target_symbol: String,
    pub priority: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyDeltaReport {
    pub added_edges: usize,
    pub removed_edges: usize,
    pub affected_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransitionOutcome {
    Success,
    RejectedByOwnership,
    RejectedBySignatureDrift,
    RejectedByTopologyDelta,
    ShadowModeSimulated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionRecord {
    pub transition_id: String,
    pub timestamp: u64,

    pub failure_id: Option<String>,
    pub triggering_symbol: Option<String>,

    pub scheduler_decision: Option<SchedulerDecision>,

    pub patch_provenance: Option<PatchProvenance>,

    pub topology_delta: Option<TopologyDeltaReport>,

    pub final_outcome: TransitionOutcome,
}

impl StateTransitionRecord {
    pub fn new(transition_id: &str) -> Self {
        Self {
            transition_id: transition_id.to_string(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            failure_id: None,
            triggering_symbol: None,
            scheduler_decision: None,
            patch_provenance: None,
            topology_delta: None,
            final_outcome: TransitionOutcome::ShadowModeSimulated,
        }
    }
}
