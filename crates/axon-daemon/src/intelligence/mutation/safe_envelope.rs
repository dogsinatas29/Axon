use serde::Serialize;
use crate::intelligence::mutation::intent_dsl::TopologyMutationIntent;

/// 3. Safe Mutation Envelope
/// Issued only if the mutation intent clears the Boundary Lock.
/// The LLM / Human is restricted to writing code explicitly bounded by this envelope.
#[derive(Debug, Serialize, Clone)]
pub struct SafeMutationEnvelope {
    pub intent: TopologyMutationIntent,
    pub transaction_id: String,
    pub permitted_lines_of_code: usize, // Enforce localized patches
    pub bounds_hash: String,            // Cryptographic lock on the topology boundary
}

impl SafeMutationEnvelope {
    pub fn new(intent: TopologyMutationIntent, tx_id: String, bounds: String) -> Self {
        Self {
            intent,
            transaction_id: tx_id,
            permitted_lines_of_code: 50, // Strict localization
            bounds_hash: bounds,
        }
    }
}
