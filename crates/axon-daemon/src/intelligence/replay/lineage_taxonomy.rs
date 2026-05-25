use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RootLineage {
    OwnershipCollapse,
    QueueOrderingCollapse,
    CancellationCollapse,
    RecursivePropagationCollapse,
    SafeBenignPropagation, 
    UnknownCollapse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymptomLineage {
    ZombieRetryLoop,
    DeferredOrphanDispatch,
    StaleHWNDDispatch,
    OrphanedAsyncTask,
    CancellationLost,
    UnboundedEmitGrowth,
    RecursiveEmitStorm,
    DestroyBeforeCallback,
    QueueInversionDrift,
    BoundedSelectionSync,
    SafeRedrawCascade,
    UnknownSymptom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxonomyMigrationManifest {
    pub taxonomy_version: String,
    pub migration_map: HashMap<String, RootLineage>,
}

impl TaxonomyMigrationManifest {
    pub fn build_v2() -> Self {
        let mut map = HashMap::new();
        // Ownership Collapse Symptoms
        map.insert("DeferredOrphanDispatch".into(), RootLineage::OwnershipCollapse);
        map.insert("StaleHWNDDispatch".into(), RootLineage::OwnershipCollapse);
        
        // Cancellation Collapse Symptoms
        map.insert("ZombieRetryLoop".into(), RootLineage::CancellationCollapse);
        map.insert("OrphanedAsyncTask".into(), RootLineage::CancellationCollapse);
        map.insert("CancellationLost".into(), RootLineage::CancellationCollapse);

        // Recursive Propagation Collapse
        map.insert("RecursiveEmitStorm".into(), RootLineage::RecursivePropagationCollapse);
        map.insert("UnboundedEmitGrowth".into(), RootLineage::RecursivePropagationCollapse);

        // Queue Ordering
        map.insert("QueueInversionDrift".into(), RootLineage::QueueOrderingCollapse);
        map.insert("DestroyBeforeCallback".into(), RootLineage::QueueOrderingCollapse);

        // Benign Corpus
        map.insert("BoundedSelectionSync".into(), RootLineage::SafeBenignPropagation);
        map.insert("SafeRedrawCascade".into(), RootLineage::SafeBenignPropagation);

        TaxonomyMigrationManifest {
            taxonomy_version: "2.0.0".to_string(),
            migration_map: map,
        }
    }

    pub fn map_legacy_symptom(&self, legacy_symptom: &str) -> RootLineage {
        self.migration_map.get(legacy_symptom).cloned().unwrap_or(RootLineage::UnknownCollapse)
    }
}

pub struct CausalSimilarityScorer;

impl CausalSimilarityScorer {
    /// Calculates similarity based on Root Lineage causality rather than string distance.
    /// Returns 1.0 for identical root families, 0.0 for unrelated families.
    pub fn calculate(root_a: &RootLineage, root_b: &RootLineage) -> f64 {
        if root_a == root_b {
            1.0 // Identical causal structure (e.g., ZombieRetryLoop vs CancellationLost)
        } else {
            0.0 // Orthogonal causal structure
        }
    }
}
