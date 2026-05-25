use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DeferredCallback {
    pub seq: u64,
    pub callback_ptr: usize,
    pub scheduled_by_seq: u64, // The crucial lineage anchor
    pub context_widget_ptr: usize,
}

/// 3. Deferred Callback Lineage
/// Traces `callback A -> g_idle_add(callback B) -> widget destroy -> callback B dispatch`
/// Maps the exact physical sequence of deferred ownership overlap.
pub struct DeferredLineageTracker {
    pub active_callbacks: Vec<DeferredCallback>,
}

impl DeferredLineageTracker {
    pub fn new() -> Self {
        Self { active_callbacks: Vec::new() }
    }

    pub fn track_enqueue(&mut self, seq: u64, callback_ptr: usize, scheduled_by: u64, widget_ptr: usize) {
        self.active_callbacks.push(DeferredCallback {
            seq,
            callback_ptr,
            scheduled_by_seq: scheduled_by,
            context_widget_ptr: widget_ptr,
        });
    }
}
