use serde::Serialize;

/// Identifies the "Entropy Density" of a legacy codebase.
/// Used to explain why a specific corpus generates particular collapse lineages.
#[derive(Debug, Serialize, Clone)]
pub struct CorpusFingerprint {
    pub topology_density: f64,
    pub callback_depth: usize,
    pub macro_entropy: f64,
    pub include_graph_complexity: usize,
    pub runtime_event_fanout: f64,
    pub ownership_ambiguity: f64,
}

impl CorpusFingerprint {
    pub fn new() -> Self {
        Self {
            topology_density: 0.0,
            callback_depth: 0,
            macro_entropy: 0.0,
            include_graph_complexity: 0,
            runtime_event_fanout: 0.0,
            ownership_ambiguity: 0.0,
        }
    }

    /// Mock calculation for a highly hostile GTK2 codebase
    pub fn from_abandoned_gtk2() -> Self {
        Self {
            topology_density: 0.95,
            callback_depth: 12, // Deep nested callbacks
            macro_entropy: 0.88, // GObject MACRO HELL
            include_graph_complexity: 1500, // Giant headers
            runtime_event_fanout: 0.92, // Signals triggering signals
            ownership_ambiguity: 0.85, // Floating references
        }
    }
}
