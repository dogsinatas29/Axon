use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The Final Verdict of the Evolution Transaction
#[derive(Debug, Serialize, Deserialize)]
pub struct ProofVerdict {
    /// Schema Version (e.g., "1.0.0"). Crucial for preventing lineage breakage in the future.
    pub schema_version: String, 
    pub verdict: String,
    pub replay_identity: f64,
    pub queue_drift: bool,
    pub ownership_drift: bool,
    pub collapse_similarity: f64,
    pub runtime_regression_detected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MutationIntentLog {
    pub intent: String,
    pub target: String,
    pub mutation_scope: String,
    pub requested_by: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CausalFamily {
    pub family: String,
    pub confidence: f64,
    pub symptoms: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineageDelta {
    pub before_root_lineages: Vec<CausalFamily>,
    pub introduced_root_lineages: Vec<CausalFamily>,
    pub removed_root_lineages: Vec<CausalFamily>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueDiff {
    pub new_edges: Vec<QueueEdge>,
    pub ordering_inversions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnershipDiff {
    pub orphaned_widgets: Vec<String>,
    pub destroyed_without_stabilization: Vec<String>,
    pub new_retention_edges: Vec<String>,
}

/// A Portable Proof System representing the Runtime Evolution Certificate
pub struct ProofArtifactBundle {
    pub verdict: ProofVerdict,
    pub intent: MutationIntentLog,
    pub lineage: LineageDelta,
    pub queue: QueueDiff,
    pub ownership: OwnershipDiff,
    /// Compressed canonical replay trace. Not raw trace. 
    /// Machine readable for future `axon verify proof.axon` validation.
    pub replay_trace_bin: Vec<u8>, 
}

impl ProofArtifactBundle {
    /// Commits the proof artifact package into the `.axon-proof/` directory.
    pub fn save_to_disk(&self, base_dir: &Path) -> Result<(), String> {
        let proof_dir = base_dir.join(".axon-proof");
        if !proof_dir.exists() {
            fs::create_dir_all(&proof_dir).map_err(|e| e.to_string())?;
        }

        fn write_json<T: Serialize>(proof_dir: &Path, filename: &str, data: &T) -> Result<(), String> {
            let path = proof_dir.join(filename);
            let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
            fs::write(path, json).map_err(|e| e.to_string())
        }

        write_json(&proof_dir, "proof.verdict.json", &self.verdict)?;
        write_json(&proof_dir, "mutation.intent.json", &self.intent)?;
        write_json(&proof_dir, "lineage.delta.json", &self.lineage)?;
        write_json(&proof_dir, "queue.diff.json", &self.queue)?;
        write_json(&proof_dir, "ownership.diff.json", &self.ownership)?;
        
        fs::write(proof_dir.join("replay.trace.bin"), &self.replay_trace_bin).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_proof_artifact_serialization() {
        let bundle = ProofArtifactBundle {
            verdict: ProofVerdict {
                schema_version: "1.0.0".to_string(),
                verdict: "SAFE_TO_MERGE".to_string(),
                replay_identity: 1.0,
                queue_drift: false,
                ownership_drift: false,
                collapse_similarity: 0.02,
                runtime_regression_detected: false,
            },
            intent: MutationIntentLog {
                intent: "ADD_TIMEOUT_CALLBACK".to_string(),
                target: "ReconnectDialog".to_string(),
                mutation_scope: "bounded".to_string(),
                requested_by: "human".to_string(),
                timestamp: 178291223,
            },
            lineage: LineageDelta {
                before_family: vec![],
                after_family: vec![],
                introduced_pathologies: vec![],
                removed_pathologies: vec![],
            },
            queue: QueueDiff {
                new_edges: vec![ QueueEdge { from: "retry_timeout".to_string(), to: "idle_dispatch".to_string() } ],
                ordering_inversions: vec![],
            },
            ownership: OwnershipDiff {
                orphaned_widgets: vec![],
                destroyed_without_stabilization: vec![],
                new_retention_edges: vec!["ReconnectDialog -> RetryTimer".to_string()],
            },
            replay_trace_bin: vec![0xCA, 0xFE, 0xBA, 0xBE], // Mock compressed binary trace
        };

        let temp = tempdir().unwrap();
        let result = bundle.save_to_disk(temp.path());
        assert!(result.is_ok());

        let proof_dir = temp.path().join(".axon-proof");
        assert!(proof_dir.exists());
        assert!(proof_dir.join("proof.verdict.json").exists());
        assert!(proof_dir.join("mutation.intent.json").exists());
        assert!(proof_dir.join("queue.diff.json").exists());
        assert!(proof_dir.join("replay.trace.bin").exists());
    }
}
