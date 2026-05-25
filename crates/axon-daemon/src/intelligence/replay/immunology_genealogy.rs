use serde::Serialize;
use std::collections::HashMap;

/// Refined Adjacency Patterns for Signal Reentrancy
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum SignalReentrancySubtype {
    ReconnectEmitReentry,
    NestedIdleEmitReentry,
    UnloadCallbackReentry,
    DeferredDestroyReentry,
}

/// Refined Adjacency Patterns for Destroy Order Drift
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum DestroyOrderSubtype {
    PluginOwnershipOrphan,
    StalePointerLineage,
}

/// Refined Adjacency Patterns for Deep Widget Topology
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum HierarchicalCollapseSubtype {
    ParentChildDrift,
    StaleSubtreeOwnership,
    OrphanedWidgetBranch,
    RecursiveDestroyPropagation,
    SelectionSynchronizationRace,
}

/// PHASE G-1 & I-3: Collapse Family Canonicalization
/// Refined by adjacency pattern, ownership timing, deferred overlap, and callback lineage.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum CanonicalCollapseFamily {
    GtkSignalReentrancy(SignalReentrancySubtype),
    GtkFloatingRefCollapse,
    GtkDestroyOrderDrift(DestroyOrderSubtype),
    GtkIdleQueueDivergence,
    CMacroNamespaceContamination,
    Win32SubclassChainCollapse,
    HierarchicalOwnershipCollapse(HierarchicalCollapseSubtype),
}

/// PHASE G-2: Collapse Genealogy Node
/// Tracks the mutation lineage and parent-child relationship of topological catastrophes.
#[derive(Debug, Serialize, Clone)]
pub struct CollapseNode {
    pub family: CanonicalCollapseFamily,
    pub version: usize, // e.g. V1 -> V2 -> V3 lineage
    pub parent_hash: Option<String>,
    pub topology_similarity_score: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImmunologyGenealogy {
    pub lineage_tree: HashMap<String, CollapseNode>,
}

impl ImmunologyGenealogy {
    pub fn new() -> Self {
        Self {
            lineage_tree: HashMap::new(),
        }
    }

    /// Registers a new catastrophe, establishing its place in the genealogy tree.
    pub fn register_collapse(&mut self, state_hash: &str, family: CanonicalCollapseFamily, parent: Option<String>, similarity: f64) {
        let version = self.calculate_depth(&parent) + 1;
        self.lineage_tree.insert(state_hash.to_string(), CollapseNode {
            family,
            version,
            parent_hash: parent,
            topology_similarity_score: similarity,
        });
    }

    fn calculate_depth(&self, parent: &Option<String>) -> usize {
        let mut depth = 0;
        let mut current = parent.clone();
        while let Some(hash) = current {
            if let Some(node) = self.lineage_tree.get(&hash) {
                depth += 1;
                current = node.parent_hash.clone();
            } else {
                break;
            }
        }
        depth
    }

    /// PHASE G-3 & I-3: Prediction Drift Replay
    /// Predicts catastrophe probability with strict determinism.
    pub fn predict_catastrophe(&self, target_topology_hash: &str) -> Option<(CanonicalCollapseFamily, f64, usize)> {
        if let Some(node) = self.lineage_tree.get(target_topology_hash) {
            Some((node.family.clone(), node.topology_similarity_score, node.version))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immunity_regression_prediction_determinism() {
        let mut immunology = ImmunologyGenealogy::new();
        immunology.register_collapse(
            "hash_drift_1", 
            CanonicalCollapseFamily::GtkDestroyOrderDrift(DestroyOrderSubtype::PluginOwnershipOrphan), 
            None, 
            0.93
        );

        // Phase I-3: The prediction must be deterministically identical over 1,000 replays
        for _ in 0..1000 {
            let prediction = immunology.predict_catastrophe("hash_drift_1");
            assert!(prediction.is_some());
            let (family, probability, version) = prediction.unwrap();
            assert_eq!(family, CanonicalCollapseFamily::GtkDestroyOrderDrift(DestroyOrderSubtype::PluginOwnershipOrphan));
            assert_eq!(probability, 0.93);
            assert_eq!(version, 1);
        }
    }
}
