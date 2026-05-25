use serde::Serialize;
use std::path::Path;

/// PHASE I-3: Runtime Adjacency Weighting
/// Edges now carry weight and collapse family context to form a catastrophe probability graph.
#[derive(Debug, Serialize, Clone)]
pub struct WeightedAdjacencyEdge {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub collapse_family: String,
}

/// Defines the deterministic snapshot of the runtime callback adjacency graph.
#[derive(Debug, Serialize, Clone)]
pub struct RuntimeAdjacencyGraph {
    pub callback_edges: Vec<WeightedAdjacencyEdge>,
    pub deferred_queue_edges: Vec<WeightedAdjacencyEdge>,
    pub ownership_transfer_edges: Vec<WeightedAdjacencyEdge>,
    pub signal_recursion_edges: Vec<WeightedAdjacencyEdge>,
    pub unload_adjacency_edges: Vec<WeightedAdjacencyEdge>,
}

impl RuntimeAdjacencyGraph {
    pub fn new() -> Self {
        Self {
            callback_edges: Vec::new(),
            deferred_queue_edges: Vec::new(),
            ownership_transfer_edges: Vec::new(),
            signal_recursion_edges: Vec::new(),
            unload_adjacency_edges: Vec::new(),
        }
    }

    pub fn write_snapshot(&self, output_dir: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(output_dir.join("RUNTIME_ADJACENCY_GRAPH.json"), json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
