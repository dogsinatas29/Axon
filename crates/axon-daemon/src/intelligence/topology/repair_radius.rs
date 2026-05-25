use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairRadius {
    SymbolOnly,
    DirectDependents,
    FileWide,
    PhaseWide,
}

impl RepairRadius {
    pub fn determine_from_error(error_msg: &str) -> Self {
        let msg = error_msg.to_lowercase();
        if msg.contains("expected") && (msg.contains(";") || msg.contains("{") || msg.contains("}")) {
            // typo / syntax error
            RepairRadius::SymbolOnly
        } else if msg.contains("mismatched types") || msg.contains("wrong number of arguments") || msg.contains("not found in") {
            // signature mismatch
            RepairRadius::DirectDependents
        } else if msg.contains("not satisfy trait bound") || msg.contains("trait") {
            // trait contract break
            RepairRadius::FileWide
        } else if msg.contains("ir corruption") || msg.contains("critical") {
            // IR corruption
            RepairRadius::PhaseWide
        } else {
            // safe default
            RepairRadius::DirectDependents
        }
    }
}
