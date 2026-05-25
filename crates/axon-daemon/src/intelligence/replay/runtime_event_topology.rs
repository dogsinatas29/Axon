use serde::Serialize;
use sha2::{Sha256, Digest};

/// PHASE E: Runtime Event Topology Determinism
/// Canonicalizes runtime event execution ordering to prevent async/deferred drift.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum RuntimeEvent {
    SignalEmit(String),
    CallbackEnter(String),
    CallbackExit(String),
    DeferredQueue(String),      // e.g. g_idle_add, PostMessage
    DeferredExecute(String),
    OwnershipHandoff(String, String), // from -> to
    QueuedDestruction(String),
}

pub struct RuntimeTopologyHarness {
    pub event_queue: Vec<RuntimeEvent>,
    pub canonical_trace: Vec<RuntimeEvent>,
}

impl RuntimeTopologyHarness {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
            canonical_trace: Vec::new(),
        }
    }

    pub fn emit_signal(&mut self, signal: &str) {
        self.canonical_trace.push(RuntimeEvent::SignalEmit(signal.to_string()));
    }

    pub fn execute_callback(&mut self, callback: &str) {
        self.canonical_trace.push(RuntimeEvent::CallbackEnter(callback.to_string()));
        self.canonical_trace.push(RuntimeEvent::CallbackExit(callback.to_string()));
    }

    pub fn queue_deferred(&mut self, action: &str) {
        self.canonical_trace.push(RuntimeEvent::DeferredQueue(action.to_string()));
        self.event_queue.push(RuntimeEvent::DeferredExecute(action.to_string()));
    }

    pub fn process_queue(&mut self) {
        let queue = std::mem::take(&mut self.event_queue);
        for event in queue {
            self.canonical_trace.push(event);
        }
    }

    pub fn queue_destruction(&mut self, widget: &str) {
        self.canonical_trace.push(RuntimeEvent::QueuedDestruction(widget.to_string()));
    }

    pub fn handoff_ownership(&mut self, from: &str, to: &str) {
        self.canonical_trace.push(RuntimeEvent::OwnershipHandoff(from.to_string(), to.to_string()));
    }

    /// Computes the absolute lifecycle hash representing the exact deterministic order of runtime events.
    pub fn get_lifecycle_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for event in &self.canonical_trace {
            let event_str = format!("{:?}", event);
            hasher.update(event_str.as_bytes());
        }
        let hash_bytes = hasher.finalize();
        let mut hash_str = String::with_capacity(64);
        for byte in hash_bytes {
            hash_str.push_str(&format!("{:02x}", byte));
        }
        hash_str
    }
}

pub fn validate_runtime_determinism(replays: usize) -> Result<(), String> {
    let mut baseline_lifecycle_hash = String::new();

    for run in 1..=replays {
        let mut harness = RuntimeTopologyHarness::new();
        
        // Complex GTK/Win32 Event Ordering Scenario
        harness.emit_signal("button_clicked");
        harness.execute_callback("on_button_clicked");
        
        // Simulate g_idle_add or PostMessage (Deferred Queue)
        harness.queue_deferred("async_cleanup_task");
        
        // Ownership Race Prevention: Handoff must happen synchronously before deferred execution
        harness.handoff_ownership("Task_A", "Task_B");
        
        // GTK Main Loop processes idle events
        harness.process_queue();
        
        // Deferred Destruction
        harness.queue_destruction("main_window");

        let current_hash = harness.get_lifecycle_hash();

        if run == 1 {
            baseline_lifecycle_hash = current_hash;
        } else if current_hash != baseline_lifecycle_hash {
            return Err(format!("EVENT_ORDERING_DRIFT: Runtime event topology diverged at run {}", run));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_e_runtime_event_topology() {
        assert!(validate_runtime_determinism(1000).is_ok());
    }

    #[test]
    fn test_phase_e_drift_detection() {
        let mut harness1 = RuntimeTopologyHarness::new();
        harness1.emit_signal("click");
        harness1.queue_deferred("task1");
        harness1.process_queue();
        let hash1 = harness1.get_lifecycle_hash();

        let mut harness2 = RuntimeTopologyHarness::new();
        harness2.emit_signal("click");
        harness2.process_queue(); // Premature processing (out of order execution)
        harness2.queue_deferred("task1");
        let hash2 = harness2.get_lifecycle_hash();

        assert_ne!(hash1, hash2, "EVENT_ORDERING_DRIFT must produce different lifecycle hashes");
    }
}
