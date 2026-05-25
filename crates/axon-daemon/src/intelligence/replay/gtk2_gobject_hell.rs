use serde::Serialize;
use sha2::{Sha256, Digest};

/// PHASE F: GTK2 + GObject Macro Hell Harness
/// Validates deterministic topology governance in the face of pseudo-OOP macro expansions, 
/// floating references, and nested signal graphs.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Gtk2CollapseKind {
    TypeRegistrationOrderDrift,
    ReentrantSignalCollapse,
    DestroyDuringEmit,
    FloatingRefLeak,
    OwnershipTransferDrift,
    IdleQueueOrderingDrift,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum GObjectEvent {
    // F-1: Type Registration
    MacroDefineType(String),
    ClassInit(String),
    
    // F-2: Signal Graph
    SignalConnect(String, String),
    SignalEmitEnter(String),
    SignalEmitExit(String),
    
    // F-3: Floating Reference Hell
    ObjectNewFloating(String),
    RefSink(String),
    ContainerAddTransfer(String, String), // child -> parent
    
    // F-4: Idle Queue
    IdleAdd(String),
    IdleExecute(String),
}

pub struct GObjectHellHarness {
    pub canonical_trace: Vec<GObjectEvent>,
    pub active_emissions: std::collections::HashSet<String>,
}

impl GObjectHellHarness {
    pub fn new() -> Self {
        Self {
            canonical_trace: Vec::new(),
            active_emissions: std::collections::HashSet::new(),
        }
    }

    /// F-1: Macro-driven type registration topology
    pub fn register_type(&mut self, type_name: &str) {
        self.canonical_trace.push(GObjectEvent::MacroDefineType(type_name.to_string()));
        self.canonical_trace.push(GObjectEvent::ClassInit(type_name.to_string()));
    }

    /// F-3: Floating Reference Sink Simulation
    pub fn create_floating_widget(&mut self, widget_id: &str) {
        self.canonical_trace.push(GObjectEvent::ObjectNewFloating(widget_id.to_string()));
    }

    pub fn container_add(&mut self, parent: &str, child: &str) {
        // Simulates sinking a floating reference automatically on container_add
        self.canonical_trace.push(GObjectEvent::RefSink(child.to_string()));
        self.canonical_trace.push(GObjectEvent::ContainerAddTransfer(child.to_string(), parent.to_string()));
    }

    /// F-2: Signal emit and reentrancy check
    pub fn emit_signal(&mut self, widget: &str, signal: &str) -> Result<(), Gtk2CollapseKind> {
        let emission_id = format!("{}:{}", widget, signal);
        
        // Check for Reentrant Signal Collapse
        if self.active_emissions.contains(&emission_id) {
            return Err(Gtk2CollapseKind::ReentrantSignalCollapse);
        }

        self.active_emissions.insert(emission_id.clone());
        self.canonical_trace.push(GObjectEvent::SignalEmitEnter(emission_id.clone()));
        
        // Nested logic happens here in real life...
        
        self.canonical_trace.push(GObjectEvent::SignalEmitExit(emission_id.clone()));
        self.active_emissions.remove(&emission_id);
        
        Ok(())
    }

    /// F-2: Destroy during emit
    pub fn destroy_widget(&mut self, widget: &str) -> Result<(), Gtk2CollapseKind> {
        // If a widget is destroyed while ANY of its signals are currently emitting
        for active in &self.active_emissions {
            if active.starts_with(&format!("{}:", widget)) {
                return Err(Gtk2CollapseKind::DestroyDuringEmit);
            }
        }
        Ok(())
    }

    pub fn get_topology_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for event in &self.canonical_trace {
            hasher.update(format!("{:?}", event).as_bytes());
        }
        let hash_bytes = hasher.finalize();
        let mut hash_str = String::with_capacity(64);
        for byte in hash_bytes {
            hash_str.push_str(&format!("{:02x}", byte));
        }
        hash_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f1_type_registration_topology() {
        let mut harness = GObjectHellHarness::new();
        harness.register_type("MyCustomWidget");
        assert_eq!(harness.canonical_trace.len(), 2);
    }

    #[test]
    fn test_f2_reentrant_signal_collapse() {
        let mut harness = GObjectHellHarness::new();
        harness.active_emissions.insert("button:clicked".to_string());
        
        // Simulating a callback that re-emits the same signal recursively
        let result = harness.emit_signal("button", "clicked");
        assert!(matches!(result, Err(Gtk2CollapseKind::ReentrantSignalCollapse)));
    }

    #[test]
    fn test_f2_destroy_during_emit() {
        let mut harness = GObjectHellHarness::new();
        harness.active_emissions.insert("window:destroy".to_string());
        
        // Simulating gtk_widget_destroy being called while already inside a destroy handler
        let result = harness.destroy_widget("window");
        assert!(matches!(result, Err(Gtk2CollapseKind::DestroyDuringEmit)));
    }

    #[test]
    fn test_f3_floating_ref_ownership_transfer() {
        let mut harness = GObjectHellHarness::new();
        harness.create_floating_widget("child_btn");
        harness.container_add("main_box", "child_btn");

        assert!(matches!(harness.canonical_trace[1], GObjectEvent::RefSink(_)));
        assert!(matches!(harness.canonical_trace[2], GObjectEvent::ContainerAddTransfer(_, _)));
    }
}
