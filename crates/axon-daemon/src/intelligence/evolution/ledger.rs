use crate::intelligence::evolution::workflow::{EvolutionVerdict, TopologyMutationContract};
use serde::Serialize;
use std::sync::Mutex;
use std::collections::HashMap;

lazy_static::lazy_static! {
    // In-memory ledger for demonstration. In production, this goes to axon-storage (SQLite).
    static ref LEDGER_DB: Mutex<HashMap<u64, MutationTransactionRecord>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Clone)]
pub struct MutationTransactionRecord {
    pub transaction_id: u64,
    pub intent_summary: String,
    pub verdict: EvolutionVerdict,
    pub replay_identity_score: f64,
    pub approved_by: Option<String>,
}

/// Mutation Transaction Ledger
/// The central immutable ledger tracking all topology evolution attempts.
pub struct MutationTransactionLedger;

impl MutationTransactionLedger {
    /// Records the execution of an evolution loop into the Ledger
    pub fn record_transaction(contract: &TopologyMutationContract) -> u64 {
        // Simple monotonically increasing ID based on timestamp
        let id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64; 
        
        let identity_score = if contract.verdict == EvolutionVerdict::SafeToMerge { 1.0 } else { 0.0 };

        let record = MutationTransactionRecord {
            transaction_id: id,
            intent_summary: format!("{:?}", contract.intent),
            verdict: contract.verdict.clone(),
            replay_identity_score: identity_score,
            approved_by: None, // Human has not yet signed the contract
        };

        let mut db = LEDGER_DB.lock().unwrap();
        db.insert(id, record);
        
        id
    }
    
    /// The Human Approval action: signing the TopologyMutationContract
    pub fn approve_transaction(transaction_id: u64, approver_name: &str) -> Result<(), String> {
        let mut db = LEDGER_DB.lock().unwrap();
        if let Some(record) = db.get_mut(&transaction_id) {
            if record.verdict == EvolutionVerdict::SafeToMerge {
                record.approved_by = Some(approver_name.to_string());
                Ok(())
            } else {
                Err("AXON HARD REJECT: Cannot approve a topology drift. Mutation is unsafe.".to_string())
            }
        } else {
            Err("Transaction not found in ledger.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::evolution::workflow::{ReplayEvidence};
    use crate::intelligence::mutation::intent_dsl::TopologyMutationIntent;

    #[test]
    fn test_ledger_records_and_approves_safe_transaction() {
        let contract = TopologyMutationContract {
            intent: TopologyMutationIntent::AddTimeout { target_flow: "Reconnect".to_string(), owner_widget_ptr: 0x1000, interval_ms: 5000 },
            risk_forecast: "Safe".to_string(),
            replay_evidence: ReplayEvidence { replay_runs: 1000, observed_drift: 0, adjacency_variance_pct: 0.0, queue_ordering_variance_pct: 0.0 },
            forensic_report: "Safe".to_string(),
            verdict: EvolutionVerdict::SafeToMerge,
        };

        let tx_id = MutationTransactionLedger::record_transaction(&contract);
        
        // Human approves the safe transaction
        let result = MutationTransactionLedger::approve_transaction(tx_id, "HumanBoss");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ledger_blocks_human_from_approving_unsafe_transaction() {
        let contract = TopologyMutationContract {
            intent: TopologyMutationIntent::AddTimeout { target_flow: "Reconnect".to_string(), owner_widget_ptr: 0x1000, interval_ms: 5000 },
            risk_forecast: "High".to_string(),
            replay_evidence: ReplayEvidence { replay_runs: 1000, observed_drift: 1, adjacency_variance_pct: 100.0, queue_ordering_variance_pct: 100.0 },
            forensic_report: "Drift".to_string(),
            verdict: EvolutionVerdict::TopologyDriftRejected,
        };

        let tx_id = MutationTransactionLedger::record_transaction(&contract);
        
        // Human attempts to override and approve a broken topology
        let result = MutationTransactionLedger::approve_transaction(tx_id, "HumanBoss");
        
        // AXON Kernel blocks the human: "Cannot approve a topology drift."
        assert!(result.is_err());
    }
}
