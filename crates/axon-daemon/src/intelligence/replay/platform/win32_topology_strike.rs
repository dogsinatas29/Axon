use std::collections::{HashMap, HashSet};
use crate::intelligence::replay::platform::PlatformStrikeSim;
use crate::intelligence::common::types::CorpusFingerprint;
use crate::intelligence::replay::trace_layering::TraceLayering;

#[derive(Debug, PartialEq)]
pub enum Win32TopologyViolation {
    Win32AbiViolation(String),
    MessageLoopTopologyCollapse(String),
    PaintLifecycleViolation(String),
    HwndLifecycleViolation(String),
    SubclassChainCorruption(String),
}

/// Win32 Runtime Topology Gate
/// Enforces order, pairs, and lifecycles strictly without deep semantic interpretation.
pub struct Win32TopologyGate {
    pub hwnd_owners: HashMap<String, String>,
    pub active_paint_contexts: HashSet<String>,
    pub subclass_chains: HashMap<String, Vec<String>>, // hwnd -> ordered hooks
}

impl Win32TopologyGate {
    pub fn new() -> Self {
        Self {
            hwnd_owners: HashMap::new(),
            active_paint_contexts: HashSet::new(),
            subclass_chains: HashMap::new(),
        }
    }

    pub fn register_hwnd(&mut self, hwnd: &str, task_id: &str) {
        self.hwnd_owners.insert(hwnd.to_string(), task_id.to_string());
    }

    /// 1. WndProc Signature Sovereignty
    pub fn attempt_wndproc_mutation(&self, new_signature: &str) -> Result<(), Win32TopologyViolation> {
        let expected = "LRESULT CALLBACK WndProc(HWND, UINT, WPARAM, LPARAM)";
        // Simplified check representing strict ABI topology
        if new_signature != expected {
            return Err(Win32TopologyViolation::Win32AbiViolation(
                format!("WIN32_ABI_VIOLATION: Attempted to mutate WndProc signature to '{}'", new_signature)
            ));
        }
        Ok(())
    }

    /// 2. Message Loop Pair Integrity
    pub fn verify_message_loop(&self, loop_sequence: &[&str]) -> Result<(), Win32TopologyViolation> {
        // Minimal trio required: GetMessage, TranslateMessage, DispatchMessage
        let required = ["GetMessage", "TranslateMessage", "DispatchMessage"];
        let mut found = 0;
        for step in loop_sequence {
            if required.contains(step) {
                found += 1;
            }
        }
        if found < 3 {
            return Err(Win32TopologyViolation::MessageLoopTopologyCollapse(
                "MESSAGE_LOOP_TOPOLOGY_COLLAPSE: Missing core message dispatch sequence.".to_string()
            ));
        }
        Ok(())
    }

    /// 3. BeginPaint / EndPaint Pair
    pub fn execute_paint_action(&mut self, action: &str, hwnd: &str) -> Result<(), Win32TopologyViolation> {
        match action {
            "BeginPaint" => {
                if !self.active_paint_contexts.insert(hwnd.to_string()) {
                    return Err(Win32TopologyViolation::PaintLifecycleViolation(
                        format!("PAINT_LIFECYCLE_VIOLATION: Double BeginPaint on {}", hwnd)
                    ));
                }
            },
            "EndPaint" => {
                if !self.active_paint_contexts.remove(hwnd) {
                    return Err(Win32TopologyViolation::PaintLifecycleViolation(
                        format!("PAINT_LIFECYCLE_VIOLATION: EndPaint without BeginPaint on {}", hwnd)
                    ));
                }
            },
            _ => {}
        }
        Ok(())
    }

    /// 4. HWND Lifecycle Ownership
    pub fn attempt_destroy_window(&self, hwnd: &str, task_id: &str) -> Result<(), Win32TopologyViolation> {
        if let Some(owner) = self.hwnd_owners.get(hwnd) {
            if owner != task_id {
                return Err(Win32TopologyViolation::HwndLifecycleViolation(
                    format!("HWND_LIFECYCLE_VIOLATION: Task '{}' attempted to destroy HWND owned by '{}'", task_id, owner)
                ));
            }
        } else {
            return Err(Win32TopologyViolation::HwndLifecycleViolation(
                "HWND_LIFECYCLE_VIOLATION: Attempted to destroy unknown HWND".to_string()
            ));
        }
        Ok(())
    }

    /// 5. Subclass Chain Corruption
    pub fn verify_subclass_call(&self, hwnd: &str, calling_hook: &str) -> Result<(), Win32TopologyViolation> {
        // Enforces that CallWindowProc must follow the explicit lineage
        if let Some(chain) = self.subclass_chains.get(hwnd) {
            if !chain.contains(&calling_hook.to_string()) {
                return Err(Win32TopologyViolation::SubclassChainCorruption(
                    format!("SUBCLASS_CHAIN_CORRUPTION: Hook '{}' is not registered in the subclass lineage of HWND '{}'", calling_hook, hwnd)
                ));
            }
        }
        Ok(())
    }
}

impl PlatformStrikeSim for Win32TopologyGate {
    fn name(&self) -> &'static str {
        "win32"
    }

    fn run_strike(&self, fingerprint: &CorpusFingerprint, seed: u64) -> Result<TraceLayering, String> {
        let mut topo_events = Vec::new();
        let mut run_events = Vec::new();

        if fingerprint.ownership_ambiguity > 0.5 {
            topo_events.push(format!("WIN32_HWND_MUTATION_{}", seed % 100));
        }
        Ok(TraceLayering::compute(&topo_events, &run_events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win32_wndproc_sovereignty() {
        let gate = Win32TopologyGate::new();
        let result = gate.attempt_wndproc_mutation("int WndProc(HWND, UINT, WPARAM, LPARAM)");
        assert!(matches!(result, Err(Win32TopologyViolation::Win32AbiViolation(_))));
    }

    #[test]
    fn test_win32_message_loop_integrity() {
        let gate = Win32TopologyGate::new();
        // Missing TranslateMessage
        let broken_sequence = vec!["GetMessage", "DispatchMessage"];
        let result = gate.verify_message_loop(&broken_sequence);
        assert!(matches!(result, Err(Win32TopologyViolation::MessageLoopTopologyCollapse(_))));
    }

    #[test]
    fn test_win32_paint_lifecycle() {
        let mut gate = Win32TopologyGate::new();
        gate.execute_paint_action("BeginPaint", "hwnd_1").unwrap();
        // EndPaint omitted, or called on wrong HWND
        let result = gate.execute_paint_action("EndPaint", "hwnd_2");
        assert!(matches!(result, Err(Win32TopologyViolation::PaintLifecycleViolation(_))));
    }

    #[test]
    fn test_win32_hwnd_ownership() {
        let mut gate = Win32TopologyGate::new();
        gate.register_hwnd("hwnd_main", "Task_A");
        // Task B tries to destroy Task A's window
        let result = gate.attempt_destroy_window("hwnd_main", "Task_B");
        assert!(matches!(result, Err(Win32TopologyViolation::HwndLifecycleViolation(_))));
    }

    #[test]
    fn test_win32_subclass_chain() {
        let mut gate = Win32TopologyGate::new();
        gate.subclass_chains.insert("hwnd_main".to_string(), vec!["OriginalWndProc".to_string(), "Hook_A".to_string()]);
        // Unregistered hook attempts CallWindowProc
        let result = gate.verify_subclass_call("hwnd_main", "RogueHook");
        assert!(matches!(result, Err(Win32TopologyViolation::SubclassChainCorruption(_))));
    }
}
