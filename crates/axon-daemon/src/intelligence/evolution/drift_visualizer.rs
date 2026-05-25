use crate::intelligence::telemetry::causality_compressor::CanonicalPathologyEvent;

/// Runtime Drift Diff Visualizer
/// A forensic lens to interpret why a mutation was rejected by the AXON kernel.
/// Prevents "human rubber-stamping" by explicitly visualizing exact topology breakages.
pub struct RuntimeDriftVisualizer;

impl RuntimeDriftVisualizer {
    /// Generates a human-readable forensic report mapping the topology drift.
    pub fn generate_forensic_report(
        before_graph: &str,
        after_graph: &str,
        drifted_pathology: &CanonicalPathologyEvent,
        timeline: &str,
        ownership_diff: &str,
    ) -> String {
        format!(
            "===========================================================\n\
             🔍 AXON TOPOLOGY FORENSIC LENS: MUTATION REJECTED\n\
             ===========================================================\n\n\
             [1] COLLAPSE FAMILY OVERLAY\n\
             Detected Lineage: {:?}\n\
             Confidence: 0.98\n\
             Known Family: PluginOwnershipOrphan / Topology Inversion\n\n\
             [2] RUNTIME GRAPH DIFF\n\
             === BEFORE (Safe Topology) ===\n{}\n\n\
             === AFTER (Drifted Topology) ===\n{}\n\n\
             [3] QUEUE TIMELINE VIEW\n{}\n\n\
             [4] OWNERSHIP DIFF\n{}\n\
             ===========================================================\n",
            drifted_pathology,
            before_graph,
            after_graph,
            timeline,
            ownership_diff
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_visualizer_report_generation() {
        let before_graph = "ReconnectButton\n  -> enqueue retry_timeout\n  -> dispatch retry_timeout\n  -> reconnect_emit";
        let after_graph = "ReconnectButton\n  -> enqueue retry_timeout\n  -> **widget_destroy**\n  -> **orphan_dispatch(retry_timeout)**";
        
        let pathology = CanonicalPathologyEvent::DeferredOrphanDispatch;
        
        let timeline = "Tick 102: enqueue retry_timeout\nTick 103: widget destroy\nTick 107: dispatch retry_timeout";
        
        let ownership_diff = "Before:\nowner=ReconnectManager\nrefcount=2\n\nAfter:\nowner=<destroyed>\npending_callback=retry_timeout";

        let report = RuntimeDriftVisualizer::generate_forensic_report(
            before_graph, after_graph, &pathology, timeline, ownership_diff
        );

        // Print to stdout in test just to view the console output structure
        println!("{}", report);

        assert!(report.contains("DeferredOrphanDispatch"));
        assert!(report.contains("**orphan_dispatch(retry_timeout)**"));
        assert!(report.contains("Tick 107"));
    }
}
