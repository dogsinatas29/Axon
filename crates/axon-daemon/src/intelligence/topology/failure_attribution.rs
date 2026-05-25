use serde::{Deserialize, Serialize};
use super::repair_radius::RepairRadius;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAttribution {
    pub failed_symbol: Option<String>,
    pub owner_task: Option<String>,
    pub repair_radius: RepairRadius,
    pub original_error: String,
}

impl FailureAttribution {
    pub fn attribute(
        error_msg: &str,
        _file_path: &str, // Could be used to filter snapshot
        _graph: &super::symbol_graph::SymbolDependencyGraph,
        snapshot: &[crate::intelligence::ast::OwnedSymbol],
    ) -> Self {
        let mut failed_symbol = None;
        let mut owner_task = None;

        // rudimentary heuristic to find symbol in error string
        for sym in snapshot {
            if error_msg.contains(&sym.symbol) {
                failed_symbol = Some(sym.symbol.clone());
                owner_task = sym.owner_task.clone();
                break; // Take the first one we find for now
            }
        }

        let radius = RepairRadius::determine_from_error(error_msg);

        Self {
            failed_symbol,
            owner_task,
            repair_radius: radius,
            original_error: error_msg.to_string(),
        }
    }
}
