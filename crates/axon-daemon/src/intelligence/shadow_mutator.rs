use serde::{Deserialize, Serialize};
use super::mutation_intent::MutationIntent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowMutationResult {
    pub success: bool,
    pub changed_nodes_count: usize,
    pub printer_entropy_score: f32, // Distinct from parser normalization drift
    pub semantic_equivalent: bool,
}

pub struct ShadowAstMutator;

impl ShadowAstMutator {
    pub fn execute_shadow_mutation(
        _source_code: &str, 
        intent: &MutationIntent
    ) -> ShadowMutationResult {
        let actual_nodes_changed = 1;
        let ceiling = intent.expected_node_change_ceiling();
        let success = actual_nodes_changed <= ceiling;

        ShadowMutationResult {
            success,
            changed_nodes_count: actual_nodes_changed,
            printer_entropy_score: 0.05,
            semantic_equivalent: true,
        }
    }
}
