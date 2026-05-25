use crate::intelligence::mutation::intent_dsl::TopologyMutationIntent;
use crate::intelligence::mutation::boundary_lock::MutationBoundaryLock;
use crate::intelligence::mutation::safe_envelope::SafeMutationEnvelope;

/// 4. Replay-backed Construction & Transaction Management
/// Treats every code change as a Topology Transaction.
pub struct MutationTransaction {
    pub intent: TopologyMutationIntent,
    pub boundary_lock: MutationBoundaryLock,
}

impl MutationTransaction {
    pub fn begin(intent: TopologyMutationIntent, lock: MutationBoundaryLock) -> Self {
        Self { intent, boundary_lock: lock }
    }

    /// Step 1 & 2: Evaluate Intent against the Lock, issue Envelope if safe.
    pub fn evaluate_and_issue_envelope(&self) -> Result<SafeMutationEnvelope, String> {
        match &self.intent {
            TopologyMutationIntent::AddIdleCallback { owner_widget_ptr, queue_class, .. } => {
                self.boundary_lock.verify_mutation_safety(*owner_widget_ptr, queue_class, 1)?;
            },
            TopologyMutationIntent::AttachSignal { from_widget_ptr, .. } => {
                // Signals typically increase nesting depth significantly in GTK
                self.boundary_lock.verify_mutation_safety(*from_widget_ptr, "SIGNAL_EMIT", 2)?;
            },
            TopologyMutationIntent::AddTimeout { owner_widget_ptr, .. } => {
                self.boundary_lock.verify_mutation_safety(*owner_widget_ptr, "TIMEOUT_ADD", 1)?;
            }
        }

        Ok(SafeMutationEnvelope::new(
            self.intent.clone(),
            "TX_MUTATION_001".to_string(), // In reality, generated uuid
            "LOCKED_TOPOLOGY_HASH".to_string()
        ))
    }

    /// Step 4: Validate post-mutation replay
    /// The Observability Layer acts as the ultimate Safety Oracle here.
    pub fn validate_post_mutation_replay(&self, compressed_lineage_before: &str, compressed_lineage_after: &str) -> Result<(), String> {
        if compressed_lineage_before != compressed_lineage_after {
            return Err("REPLAY_VALIDATION_FAILED: The mutation injected topology drift or a new pathology.".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_transaction_flow() {
        let mut lock = MutationBoundaryLock::new(5);
        lock.lock_widget_scope(0x1000);
        lock.allow_queue_kind("IDLE_ADD");

        let intent = TopologyMutationIntent::AddIdleCallback {
            target_flow: "Reconnect".to_string(),
            owner_widget_ptr: 0x1000,
            queue_class: "IDLE_ADD".to_string(),
        };

        let tx = MutationTransaction::begin(intent, lock);
        
        // 1. Issue Envelope
        let envelope = tx.evaluate_and_issue_envelope();
        assert!(envelope.is_ok());

        // 2. Validate Replay (Safety Oracle)
        let before_lineage = "CanonicalPathologyEvent::None";
        let after_lineage = "CanonicalPathologyEvent::None";
        assert!(tx.validate_post_mutation_replay(before_lineage, after_lineage).is_ok());

        // 3. Detect Drift
        let bad_after_lineage = "CanonicalPathologyEvent::DeferredOrphanDispatch";
        assert!(tx.validate_post_mutation_replay(before_lineage, bad_after_lineage).is_err());
    }
}
