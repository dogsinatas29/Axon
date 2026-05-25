use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProvenance {
    pub patch_id: String,
    pub source_failure: Option<String>,
    pub owner_task: String,
    pub validated_by: Vec<String>,
    pub timestamp: u64,
}

impl PatchProvenance {
    pub fn new(patch_id: &str, owner_task: &str, source_failure: Option<String>) -> Self {
        Self {
            patch_id: patch_id.to_string(),
            source_failure,
            owner_task: owner_task.to_string(),
            validated_by: Vec::new(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        }
    }

    pub fn add_validation(&mut self, gate_name: &str) {
        self.validated_by.push(gate_name.to_string());
    }
}
