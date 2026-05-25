use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationAttemptRecord {
    pub patch_id: String,
    pub symbol: String,
    pub timestamp: u64,
    pub before_hash: String,
    pub after_hash: String,
    pub topology_delta_detected: bool,
    pub signature_drift_detected: bool,
    pub formatting_drift_detected: bool,
    pub validator_decisions: Vec<String>,
}

pub struct MutationReplayObservatory;

impl MutationReplayObservatory {
    pub fn record_attempt(project_root: &std::path::Path, record: &MutationAttemptRecord) {
        use std::io::Write;
        
        // Ensure debug dir exists
        let debug_dir = project_root.join("debug");
        let _ = std::fs::create_dir_all(&debug_dir);

        let log_path = debug_dir.join("mutation_attempts.jsonl");
        if let Ok(json_line) = serde_json::to_string(record) {
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(file, "{}", json_line);
            }
        }

        if record.topology_delta_detected {
            let cat_path = debug_dir.join("topology_delta_catalog.jsonl");
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&cat_path) {
                let _ = writeln!(file, "{}", serde_json::to_string(record).unwrap());
            }
        }

        if record.signature_drift_detected {
            let cat_path = debug_dir.join("signature_drift_catalog.jsonl");
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&cat_path) {
                let _ = writeln!(file, "{}", serde_json::to_string(record).unwrap());
            }
        }

        if record.formatting_drift_detected {
            let cat_path = debug_dir.join("formatting_drift_catalog.jsonl");
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&cat_path) {
                let _ = writeln!(file, "{}", serde_json::to_string(record).unwrap());
            }
        }
    }
}
