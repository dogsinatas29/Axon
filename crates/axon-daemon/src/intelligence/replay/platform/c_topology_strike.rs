use std::collections::HashMap;
use crate::intelligence::replay::platform::PlatformStrikeSim;
use crate::intelligence::common::types::CorpusFingerprint;
use crate::intelligence::replay::trace_layering::TraceLayering;

#[derive(Debug, PartialEq)]
pub enum CTopologyViolation {
    AbiTopologyViolation(String),
    TopologyImportViolation(String),
    MacroContamination(String),
}

/// Topology Ownership Gate for C/C++ environments
/// Extends ownership beyond symbols to includes, macros, and ABI contracts.
pub struct CTopologyGate {
    pub symbol_owners: HashMap<String, String>,
    pub include_owners: HashMap<String, String>,
    pub macro_owners: HashMap<String, String>,
}

impl CTopologyGate {
    pub fn new() -> Self {
        Self {
            symbol_owners: HashMap::new(),
            include_owners: HashMap::new(),
            macro_owners: HashMap::new(),
        }
    }

    pub fn register_symbol(&mut self, symbol: &str, owner: &str) {
        self.symbol_owners.insert(symbol.to_string(), owner.to_string());
    }

    pub fn register_include(&mut self, include_edge: &str, owner: &str) {
        self.include_owners.insert(include_edge.to_string(), owner.to_string());
    }

    pub fn register_macro(&mut self, macro_name: &str, owner: &str) {
        self.macro_owners.insert(macro_name.to_string(), owner.to_string());
    }

    /// Validates a proposed topology mutation against the kernel's registry
    pub fn attempt_mutation(&self, entity_type: &str, entity_name: &str, task_id: &str) -> Result<(), CTopologyViolation> {
        match entity_type {
            "symbol" => {
                if let Some(owner) = self.symbol_owners.get(entity_name) {
                    if owner != task_id {
                        return Err(CTopologyViolation::AbiTopologyViolation(
                            format!("ABI_TOPOLOGY_VIOLATION: Symbol '{}' is owned by '{}'. Task '{}' forbidden.", entity_name, owner, task_id)
                        ));
                    }
                }
            },
            "include" => {
                if let Some(owner) = self.include_owners.get(entity_name) {
                    if owner != task_id {
                        return Err(CTopologyViolation::TopologyImportViolation(
                            format!("TOPOLOGY_IMPORT_VIOLATION: Include edge '{}' is owned by '{}'. Task '{}' forbidden.", entity_name, owner, task_id)
                        ));
                    }
                } else {
                    // Changing an include edge to an unregistered/unknown topology
                    return Err(CTopologyViolation::TopologyImportViolation(
                        format!("TOPOLOGY_IMPORT_VIOLATION: Unregistered or modified include edge '{}' attempted by Task '{}'.", entity_name, task_id)
                    ));
                }
            },
            "macro" => {
                if let Some(owner) = self.macro_owners.get(entity_name) {
                    if owner != task_id {
                        return Err(CTopologyViolation::MacroContamination(
                            format!("MACRO_CONTAMINATION: Macro scope '{}' is owned by '{}'. Task '{}' forbidden.", entity_name, owner, task_id)
                        ));
                    }
                } else {
                    // Injecting a new global macro is semantic namespace corruption unless explicitly granted
                    return Err(CTopologyViolation::MacroContamination(
                        format!("MACRO_CONTAMINATION: Unauthorized injection of global macro '{}' by Task '{}'.", entity_name, task_id)
                    ));
                }
            },
            _ => {}
        }
        Ok(())
    }
}

impl PlatformStrikeSim for CTopologyGate {
    fn name(&self) -> &'static str {
        "c_topology"
    }

    fn run_strike(&self, fingerprint: &CorpusFingerprint, seed: u64) -> Result<TraceLayering, String> {
        let mut topo_events = Vec::new();
        let mut run_events = Vec::new();

        if fingerprint.topology_density > 0.5 {
            topo_events.push(format!("C_ABI_MUTATION_{}", seed % 100));
        }
        Ok(TraceLayering::compute(&topo_events, &run_events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_header_abi_topology_strike() {
        let mut gate = CTopologyGate::new();
        gate.register_symbol("parse_user", "Task_A");
        gate.register_symbol("render_user", "Task_B");

        // Experiment 1: Task_B attempts to modify parse_user signature
        let result = gate.attempt_mutation("symbol", "parse_user", "Task_B");
        match result {
            Err(CTopologyViolation::AbiTopologyViolation(_)) => { /* PASS */ },
            _ => panic!("Expected ABI_TOPOLOGY_VIOLATION"),
        }
    }

    #[test]
    fn test_c_include_topology_mutation() {
        let mut gate = CTopologyGate::new();
        gate.register_include("user_parser.h", "Task_A");

        // Experiment 2: Task_A tries to change dependency to "experimental_parser.h"
        // This is a new edge not registered/allowed for Task_A in current topology
        let result = gate.attempt_mutation("include", "experimental_parser.h", "Task_A");
        match result {
            Err(CTopologyViolation::TopologyImportViolation(_)) => { /* PASS */ },
            _ => panic!("Expected TOPOLOGY_IMPORT_VIOLATION"),
        }
    }

    #[test]
    fn test_c_macro_contamination() {
        let mut gate = CTopologyGate::new();
        gate.register_macro("USER_MAX", "Task_A");

        // Experiment 3: Task_B tries to define a new macro `#define User void` (Semantic Namespace Corruption)
        let result = gate.attempt_mutation("macro", "User", "Task_B");
        match result {
            Err(CTopologyViolation::MacroContamination(_)) => { /* PASS */ },
            _ => panic!("Expected MACRO_CONTAMINATION"),
        }
    }
}
