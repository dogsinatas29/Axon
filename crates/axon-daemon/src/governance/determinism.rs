use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct SystemStateHasher;

impl SystemStateHasher {
    /// Computes a canonical hash representing the entire state of the AXON repair kernel.
    /// If two deterministic replays result in a different hash here, the kernel has a nondeterminism bug.
    pub fn compute_canonical_hash(
        ownership_snapshot: &str,
        dependency_graph: &str,
        leases: &str,
        provenance_head: &str
    ) -> String {
        let mut hasher = DefaultHasher::new();
        
        ownership_snapshot.hash(&mut hasher);
        dependency_graph.hash(&mut hasher);
        leases.hash(&mut hasher);
        provenance_head.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
}
