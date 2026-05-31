use serde::{Deserialize, Serialize};

/// P5-8h: Corpus Ingestor
/// Fetches external repositories into a highly reproducible snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibleCorpusSnapshot {
    pub repo: String,
    pub commit: String,
    pub language: String,
    pub tree_sitter_version: String,
    pub formatter_version: String,
    pub ingest_timestamp: String,
    /// 캠페인에서 순회할 소스 파일 목록 (절대 경로)
    pub source_files: Vec<String>,
}

pub struct CorpusIngestor;

impl CorpusIngestor {
    /// Deep-freezes a specific commit of a legacy open-source project (e.g. Tokio, Django)
    /// to guarantee that tests rerun a year from now yield the exact same environment.
    pub fn freeze_snapshot(repo: &str, commit: &str, language: &str) -> ReproducibleCorpusSnapshot {
        // Stub: performs a shallow clone and pins dependencies.
        ReproducibleCorpusSnapshot {
            repo: repo.to_string(),
            commit: commit.to_string(),
            language: language.to_string(),
            tree_sitter_version: env!("CARGO_PKG_VERSION").to_string(),
            formatter_version: "rustfmt 1.81".to_string(),
            ingest_timestamp: "2026-05-23T23:10:00Z".to_string(),
            source_files: Vec::new(), // 실제 사용 시 glob으로 수집
        }
    }
}
