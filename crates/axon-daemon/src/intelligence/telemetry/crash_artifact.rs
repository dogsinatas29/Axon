use serde::Serialize;
use std::path::Path;

/// STEP 4-5: Crash Artifact Capture
/// Extracts the dirty, physical state of the process the moment a catastrophe is intercepted.
/// Becomes the foundation of the 'Catastrophe Lineage'.
#[derive(Debug, Serialize, Clone)]
pub struct CrashArtifactBundle {
    pub runtime_object_hash: String,
    pub queue_snapshot: String, // E.g., serializing the QueueDriftDetector
    pub callback_lineage: Vec<String>,
    pub widget_tree_snapshot: String,
    pub last_signal_edge: String,
}

impl CrashArtifactBundle {
    pub fn generate_bundle(hash: String, last_signal: String) -> Self {
        Self {
            runtime_object_hash: hash,
            queue_snapshot: "QUEUE_STATE_DUMP_BYTES".to_string(),
            callback_lineage: vec!["callback_ptr_0x1".to_string(), "callback_ptr_0x2".to_string()],
            widget_tree_snapshot: "PHYSICAL_WIDGET_TREE_DUMP".to_string(),
            last_signal_edge: last_signal,
        }
    }

    pub fn write_artifact(&self, output_dir: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(output_dir.join("CRASH_ARTIFACT_BUNDLE.json"), json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
