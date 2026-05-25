use serde::Serialize;
use std::path::Path;

/// PHASE I-1: Corpus Sovereignty Freeze
/// Seals the exact state of the mounted corpus subsystem so future observations
/// can mathematically prove they ran against identical entropy.
#[derive(Debug, Serialize, Clone)]
pub struct CorpusSeal {
    pub repo_commit: String,
    pub subtree_digest: String,
    pub parser_version: String,
    pub grammar_sha256: String,
    pub topology_seed: u64,
    pub normalization_policy: String,
    pub ownership_anchor_version: String,
    
    // Phase I-2 Additions: Entropy-bearing topology metrics
    pub filesystem_topology_hash: String,
    pub macro_symbol_density: f64,
    pub callback_graph_density: f64,
    pub plugin_boundary_entropy: f64,
}

impl CorpusSeal {
    pub fn generate_mock_xchat_seal() -> Self {
        Self {
            repo_commit: "4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string(), // Typical abandoned commit
            subtree_digest: "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_string(),
            parser_version: "tree-sitter-c-0.20.2".to_string(),
            grammar_sha256: "sha256:5d5b09f6dcb2d53c5d8116273da928e1d0f5e3e6027c427387cc6a9db1f20387".to_string(),
            topology_seed: 0xDEAD_BEEF_CAFE,
            normalization_policy: "CRLF_TO_LF_STRIP_BOM".to_string(),
            ownership_anchor_version: "v1.2_strict_bounds".to_string(),
            filesystem_topology_hash: "sha256:4a...".to_string(),
            macro_symbol_density: 0.88,
            callback_graph_density: 0.94,
            plugin_boundary_entropy: 0.91,
        }
    }

    pub fn write_seal(&self, output_dir: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(output_dir.join("CORPUS_SEAL.json"), json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
