use super::atomic_io::{write_json_atomic, append_jsonl_atomic};
use crate::intelligence::causality::StateTransitionRecord;

/// Single IO Gateway for all authoritative state modifications.
pub struct GovernanceStore {
    base_dir: std::path::PathBuf,
}

impl GovernanceStore {
    pub fn new(base_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Safely writes the ownership snapshot.
    pub fn write_ownership_snapshot<T: serde::Serialize>(&self, snapshot: &T) -> std::io::Result<()> {
        let path = self.base_dir.join("ownership_snapshot.json");
        write_json_atomic(&path, snapshot)
    }

    /// Safely writes the symbol registry.
    pub fn write_symbol_registry<T: serde::Serialize>(&self, registry: &T) -> std::io::Result<()> {
        let path = self.base_dir.join("symbol_registry.json");
        write_json_atomic(&path, registry)
    }

    /// Appends to the state transition ledger (provenance).
    pub fn append_provenance(&self, record: &StateTransitionRecord) -> std::io::Result<()> {
        let path = self.base_dir.join("mutation_attempts.jsonl");
        append_jsonl_atomic(&path, record)
    }
}
