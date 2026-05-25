use serde::Serialize;
use std::collections::HashSet;

/// Safe Mutation Layer: Mutation Boundary Lock
/// Defines the strict runtime boundaries (Ownership & Queue Adjacency) within which 
/// an AI or Human can safely inject new code (feature evolution).
/// This prevents arbitrary mutations from triggering a catastrophic collapse in a legacy system.
#[derive(Debug, Serialize, Clone)]
pub struct MutationBoundaryLock {
    pub locked_widget_ptrs: HashSet<usize>,
    pub allowed_queue_kinds: HashSet<String>,
    pub max_signal_depth: u32,
    pub strict_refcount_balance: bool,
}

impl MutationBoundaryLock {
    pub fn new(max_depth: u32) -> Self {
        Self {
            locked_widget_ptrs: HashSet::new(),
            allowed_queue_kinds: HashSet::new(),
            max_signal_depth: max_depth,
            strict_refcount_balance: true, // Immutable rule: Every ref must have an unref
        }
    }

    /// Locks a specific widget's ownership scope for mutation.
    pub fn lock_widget_scope(&mut self, ptr: usize) {
        self.locked_widget_ptrs.insert(ptr);
    }

    /// Whitelists specific queue topologies for the proposed mutation.
    pub fn allow_queue_kind(&mut self, kind: &str) {
        self.allowed_queue_kinds.insert(kind.to_string());
    }

    /// Verifies if a proposed mutation (e.g., adding a timeout or callback) violates
    /// the established topological boundaries.
    pub fn verify_mutation_safety(&self, proposed_widget_ptr: usize, queue_kind: &str, depth_impact: u32) -> Result<(), String> {
        if !self.locked_widget_ptrs.contains(&proposed_widget_ptr) {
            return Err(format!("BOUNDARY_VIOLATION: Attempting to mutate an unlocked widget ownership scope (ptr: {}).", proposed_widget_ptr));
        }
        if !self.allowed_queue_kinds.contains(queue_kind) {
            return Err(format!("BOUNDARY_VIOLATION: Attempting to inject a disallowed queue kind: {}.", queue_kind));
        }
        if depth_impact > self.max_signal_depth {
            return Err("BOUNDARY_VIOLATION: Proposed mutation exceeds safe signal nesting depth.".to_string());
        }
        // If strict refcount balance is violated, it would be checked during replay evaluation.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_boundary_enforcement() {
        let mut lock = MutationBoundaryLock::new(5);
        lock.lock_widget_scope(0x7FFA0001);
        lock.allow_queue_kind("IDLE_ADD");

        // Safe Mutation
        assert!(lock.verify_mutation_safety(0x7FFA0001, "IDLE_ADD", 2).is_ok());

        // Boundary Violation: Out-of-scope widget
        assert!(lock.verify_mutation_safety(0x7FFB9999, "IDLE_ADD", 2).is_err());

        // Boundary Violation: Disallowed Queue (e.g. timeout recursive loop)
        assert!(lock.verify_mutation_safety(0x7FFA0001, "TIMEOUT_ADD", 2).is_err());

        // Boundary Violation: Excessive Nesting
        assert!(lock.verify_mutation_safety(0x7FFA0001, "IDLE_ADD", 10).is_err());
    }
}
