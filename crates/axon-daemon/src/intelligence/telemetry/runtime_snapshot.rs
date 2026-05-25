use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Physical widget node capturing real C pointers and actual refcounts
#[derive(Debug, Serialize, Clone)]
pub struct PhysicalWidgetNode {
    pub ptr_address: String,
    pub widget_type: String,
    pub refcount: i32,
}

/// Actual items pushed into the GLib main loop deferred queue
#[derive(Debug, Serialize, Clone)]
pub struct PhysicalQueueItem {
    pub queue_type: String, // "IDLE" or "TIMEOUT"
    pub func_ptr: String,
    pub data_ptr: String,
}

/// STEP 2: Runtime Snapshot Serializer
/// Converts the dirty stream of physical bytes from `axon_gtk_tap.c` into a canonical JSON.
/// This generates the authoritative `runtime_object_hash` representing the true state of the process.
#[derive(Debug, Serialize, Clone)]
pub struct RuntimePhysicalSnapshot {
    pub widget_tree: Vec<PhysicalWidgetNode>,
    pub deferred_queue: Vec<PhysicalQueueItem>,
    pub runtime_object_hash: String,
}

impl RuntimePhysicalSnapshot {
    pub fn new() -> Self {
        Self {
            widget_tree: Vec::new(),
            deferred_queue: Vec::new(),
            runtime_object_hash: "UNHASHED".to_string(),
        }
    }

    /// Computes the exact `runtime_object_hash` based strictly on physical pointers and refcounts.
    /// If this hash drifts between replays, the process lacks runtime determinism.
    pub fn compute_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        
        for widget in &self.widget_tree {
            widget.ptr_address.hash(&mut hasher);
            widget.refcount.hash(&mut hasher);
        }
        for q in &self.deferred_queue {
            q.func_ptr.hash(&mut hasher);
            q.data_ptr.hash(&mut hasher);
        }
        
        self.runtime_object_hash = format!("{:016x}", hasher.finish());
    }
}
