use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use super::failure_classifier::CatastropheKind;
use crate::intelligence::replay::lineage_taxonomy::{TaxonomyMigrationManifest, RootLineage, CausalSimilarityScorer};

/// 아카이브 파일 경로 — runtime/ 하위에 영구 보존
fn archive_path() -> PathBuf {
    PathBuf::from("runtime/catastrophe_archive.jsonl")
}

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
    /// Normalizes a legacy catastrophe record into the v2.0.0 Taxonomy
    /// and appends it to the persistent JSONL archive.
    pub fn normalize_and_archive(legacy_symptoms: Vec<String>, fingerprint: String, kind: CatastropheKind) -> Result<(), String> {
        let manifest = TaxonomyMigrationManifest::build_v2();

        // Pick the primary root lineage mapping
        let primary_root = legacy_symptoms.first()
            .map(|sym| manifest.map_legacy_symptom(sym))
            .unwrap_or(RootLineage::UnknownCollapse);

        let record = CatastropheRecord {
            id: uuid::Uuid::new_v4().to_string(),
            taxonomy_version: manifest.taxonomy_version,
            root_lineage: primary_root,
            legacy_symptoms,
            kind,
            fingerprint,
        };

        // Append to read-only historical registry (JSONL format)
        let path = archive_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create archive dir: {}", e))?;
        }

        let json = serde_json::to_string(&record)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open archive: {}", e))?;

        writeln!(file, "{}", json)
            .map_err(|e| format!("Failed to write archive: {}", e))?;

        tracing::info!("📚 [CATASTROPHE_ARCHIVE] Normalized and archived: root={:?}, id={}", record.root_lineage, record.id);
        Ok(())
    }

    /// Loads all archived RootLineage entries from the persistent JSONL archive.
    pub fn load_archived_roots() -> Vec<RootLineage> {
        let path = archive_path();
        if !path.exists() {
            return Vec::new();
        }

        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("⚠️ [CATASTROPHE_ARCHIVE] Could not open archive: {}", e);
                return Vec::new();
            }
        };

        BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<CatastropheRecord>(&line).ok())
            .map(|rec| rec.root_lineage)
            .collect()
    }

    /// Predictive Check: Causal Root Lineage weighted similarity.
    /// Uses the archived historical data to detect known-dangerous patterns.
    pub fn is_predictable_catastrophe(target_root: &RootLineage, archived_roots: &[RootLineage]) -> bool {
        for archived in archived_roots {
            let similarity = CausalSimilarityScorer::calculate(target_root, archived);
            if similarity > 0.8 {
                return true;
            }
        }
        false
    }

    /// Convenience: loads archive and checks in one call.
    pub fn check_against_archive(target_root: &RootLineage) -> bool {
        let archived = Self::load_archived_roots();
        Self::is_predictable_catastrophe(target_root, &archived)
    }
}
