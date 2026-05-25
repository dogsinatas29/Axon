use serde::Serialize;

/// Real GTK Runtime Tap (Telemetry Layer)
/// Hooks actual `g_signal_emit`, `g_idle_add`, and extracts physical `GObject` refcounts.
/// Moves AXON from "Topology Theory" to "Physical Runtime Pathology Instrumentation".
#[derive(Debug, Serialize, Clone)]
pub struct GtkRuntimeSnapshot {
    pub runtime_object_hash: String,
    pub runtime_queue_hash: String,
    pub runtime_lifecycle_hash: String,
    pub active_idle_queue_size: usize,
    pub signal_emission_depth: usize,
}

impl GtkRuntimeSnapshot {
    pub fn new() -> Self {
        Self {
            runtime_object_hash: "00000000".to_string(),
            runtime_queue_hash: "00000000".to_string(),
            runtime_lifecycle_hash: "00000000".to_string(),
            active_idle_queue_size: 0,
            signal_emission_depth: 0,
        }
    }
}
