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
    /// Calculates causal similarity using a weighted cross-lineage matrix.
    ///
    /// - 동일 계통(Same Root)       = 1.0 (확실한 동일 원인)
    /// - 연관 계통(Related Root)    = 0.2~0.4 (부분 연관)
    /// - 무관 계통(Unrelated Root)  = 0.0
    /// - SafeBenignPropagation 은 위험 계통과 항상 0.0
    pub fn calculate(root_a: &RootLineage, root_b: &RootLineage) -> f64 {
        if root_a == root_b {
            return 1.0;
        }

        // SafeBenignPropagation은 다른 위험 계통과 교차 유사도 없음
        if matches!(root_a, RootLineage::SafeBenignPropagation)
            || matches!(root_b, RootLineage::SafeBenignPropagation)
        {
            return 0.0;
        }

        // UnknownCollapse는 계통 불명으로 0.0
        if matches!(root_a, RootLineage::UnknownCollapse)
            || matches!(root_b, RootLineage::UnknownCollapse)
        {
            return 0.0;
        }

        // 계통 간 부분 유사도 매트릭스
        // 설계 근거: 소유권 붕괴와 취소 붕괴는 생명주기(lifecycle) 오류로 연관,
        //           큐 순서 붕괴와 재귀 전파 붕괴는 이벤트 흐름(flow) 오류로 연관
        match (root_a, root_b) {
            // Lifecycle 계열 연관
            (RootLineage::OwnershipCollapse, RootLineage::CancellationCollapse)
            | (RootLineage::CancellationCollapse, RootLineage::OwnershipCollapse) => 0.4,

            // Flow 계열 연관
            (RootLineage::QueueOrderingCollapse, RootLineage::RecursivePropagationCollapse)
            | (RootLineage::RecursivePropagationCollapse, RootLineage::QueueOrderingCollapse) => 0.3,

            // Lifecycle ↔ Flow 간 약한 연관
            (RootLineage::OwnershipCollapse, RootLineage::QueueOrderingCollapse)
            | (RootLineage::QueueOrderingCollapse, RootLineage::OwnershipCollapse) => 0.2,

            (RootLineage::CancellationCollapse, RootLineage::RecursivePropagationCollapse)
            | (RootLineage::RecursivePropagationCollapse, RootLineage::CancellationCollapse) => 0.2,

            _ => 0.0,
        }
    }
}
