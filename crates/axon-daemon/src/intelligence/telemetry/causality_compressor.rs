use serde::Serialize;
use crate::intelligence::telemetry::refcount_logger::RefcountEvent;

/// Canonical Pathology Events that strip out ASLR, pointer jitter, and scheduling noise.
/// This represents the absolute 'truth' of a crash's causality.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum CanonicalPathologyEvent {
    DestroyWithoutStabilization,
    DeferredOrphanDispatch,
    RecursiveEmitStorm,
    QueueOrderingInversion,
}

/// Runtime Causality Compressor
/// Condenses thousands of raw physical telemetry bytes into normalized, 
/// deterministic causal events to prevent Lineage Explosion.
pub struct RuntimeCausalityCompressor {
    pub compressed_lineage: Vec<CanonicalPathologyEvent>,
}

impl RuntimeCausalityCompressor {
    pub fn new() -> Self {
        Self {
            compressed_lineage: Vec::new(),
        }
    }

    /// Enforces Crash Artifact Minimalism (Max Ceiling: 100 Events)
    /// Prevents Lineage Memory Explosion during Recursive Emit Storms or Infinite Redraws.
    fn push_event(&mut self, event: CanonicalPathologyEvent) {
        if self.compressed_lineage.len() < 100 {
            self.compressed_lineage.push(event);
        }
    }

    /// Evaluates raw Refcount Transitions for pathological signatures
    pub fn process_refcount_transition(&mut self, event: &RefcountEvent) {
        if *event == RefcountEvent::DestroyWithoutStabilization {
            self.push_event(CanonicalPathologyEvent::DestroyWithoutStabilization);
        }
    }

    /// Evaluates raw Signal Nesting Depths for Recursive Storms
    pub fn process_signal_depth(&mut self, depth: u32) {
        // If depth exceeds typical GTK safe depth limit (e.g. 50), it's a recursive storm.
        if depth > 50 {
            if self.compressed_lineage.last() != Some(&CanonicalPathologyEvent::RecursiveEmitStorm) {
                self.push_event(CanonicalPathologyEvent::RecursiveEmitStorm);
            }
        }
    }

    /// Evaluates deferred callback orphan states
    pub fn process_deferred_dispatch_orphan(&mut self) {
        self.push_event(CanonicalPathologyEvent::DeferredOrphanDispatch);
    }
}
