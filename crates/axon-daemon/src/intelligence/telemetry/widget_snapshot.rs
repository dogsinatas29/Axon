use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct WidgetNodeState {
    pub ptr: usize,
    pub refcount: i32,
    pub parent_ptr: Option<usize>,
    pub is_orphan: bool,
}

/// 4. Widget Tree Snapshot Diff
/// Tracks the actual physical mutation of the widget tree in memory.
/// Used to capture detached renderers, orphan subtrees, and stale child pointers.
pub struct WidgetTreeDiff {
    pub snapshots: Vec<HashMap<usize, WidgetNodeState>>,
}

impl WidgetTreeDiff {
    pub fn new() -> Self {
        Self { snapshots: Vec::new() }
    }

    pub fn capture_snapshot(&mut self, state: HashMap<usize, WidgetNodeState>) {
        self.snapshots.push(state);
    }
    
    pub fn calculate_orphan_nodes(&self, from_idx: usize, to_idx: usize) -> Vec<usize> {
        let mut orphans = Vec::new();
        if to_idx < self.snapshots.len() && from_idx < self.snapshots.len() {
            let to_state = &self.snapshots[to_idx];
            for (ptr, node) in to_state {
                if node.is_orphan {
                    orphans.push(*ptr);
                }
            }
        }
        orphans
    }
}
