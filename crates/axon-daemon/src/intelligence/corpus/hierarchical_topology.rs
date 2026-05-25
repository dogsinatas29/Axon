use serde::Serialize;
use std::path::Path;

/// HIERARCHICAL_OWNERSHIP_GRAPH: tracks parent-child widget topology
#[derive(Debug, Serialize, Clone)]
pub struct HierarchicalOwnershipGraph {
    pub parent_child_edges: Vec<String>,
}

/// SUBTREE_LIFECYCLE_CHAIN: tracks destroy propagation lineage
#[derive(Debug, Serialize, Clone)]
pub struct SubtreeLifecycleChain {
    pub destroy_propagation_edges: Vec<String>,
}

/// SELECTION_STATE_GRAPH: tracks UI selection synchronization
#[derive(Debug, Serialize, Clone)]
pub struct SelectionStateGraph {
    pub selection_sync_edges: Vec<String>,
}

/// RECURSIVE_INVALIDATION_TRACE: tracks redraw/refresh propagation
#[derive(Debug, Serialize, Clone)]
pub struct RecursiveInvalidationTrace {
    pub redraw_propagation_edges: Vec<String>,
}

/// Defines the deterministic snapshot of deep widget topologies for ROX-Filer.
/// Shifts focus from single-callback adjacency to hierarchical subtree propagation.
#[derive(Debug, Serialize, Clone)]
pub struct RoxFilerTopologySnapshot {
    pub hierarchical_ownership: HierarchicalOwnershipGraph,
    pub subtree_lifecycle: SubtreeLifecycleChain,
    pub selection_state: SelectionStateGraph,
    pub recursive_invalidation: RecursiveInvalidationTrace,
}

impl RoxFilerTopologySnapshot {
    pub fn new() -> Self {
        Self {
            hierarchical_ownership: HierarchicalOwnershipGraph { parent_child_edges: Vec::new() },
            subtree_lifecycle: SubtreeLifecycleChain { destroy_propagation_edges: Vec::new() },
            selection_state: SelectionStateGraph { selection_sync_edges: Vec::new() },
            recursive_invalidation: RecursiveInvalidationTrace { redraw_propagation_edges: Vec::new() },
        }
    }

    pub fn write_snapshot(&self, output_dir: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(output_dir.join("ROX_FILER_TOPOLOGY_SNAPSHOT.json"), json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
