use serde::{Deserialize, Serialize};
use super::failure_classifier::CatastropheKind;
use crate::intelligence::replay::lineage_taxonomy::{TaxonomyMigrationManifest, RootLineage, CausalSimilarityScorer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatastropheRecord {
    pub id: String,
    pub taxonomy_version: String,
    pub root_lineage: RootLineage, // Upgraded from raw String
    pub legacy_symptoms: Vec<String>,
    pub kind: CatastropheKind,
    pub fingerprint: String,
}

pub struct PredictiveImmuneLayer;

impl PredictiveImmuneLayer {
    /// Normalizes a legacy catastrophe record into the v2.0.0 Taxonomy.
    pub fn normalize_and_archive(legacy_symptoms: Vec<String>, fingerprint: String, kind: CatastropheKind) -> Result<(), String> {
        let manifest = TaxonomyMigrationManifest::build_v2();
        
        // Pick the primary root lineage mapping
        let primary_root = legacy_symptoms.first()
            .map(|sym| manifest.map_legacy_symptom(sym))
            .unwrap_or(RootLineage::UnknownCollapse);

        let _record = CatastropheRecord {
            id: uuid::Uuid::new_v4().to_string(),
            taxonomy_version: manifest.taxonomy_version,
            root_lineage: primary_root,
            legacy_symptoms,
            kind,
            fingerprint,
        };

        // Write to a read-only historical registry
        Ok(())
    }

    /// Predictive Check: Causal Root Lineage weighted similarity.
    pub fn is_predictable_catastrophe(target_root: &RootLineage, archived_roots: &[RootLineage]) -> bool {
        // High similarity is determined if they belong to the SAME causal family
        // e.g. ZombieRetryLoop (target) vs CancellationLost (archive) = Similarity 1.0 (Same Root)
        for archived in archived_roots {
            let similarity = CausalSimilarityScorer::calculate(target_root, archived);
            if similarity > 0.8 {
                return true; 
            }
        }
        false
    }
}

