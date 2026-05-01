use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use super::analysis_contract::{Analyzer, FailureEvent, AnalysisResult, ClusterInfo};

pub struct FailureClusterer;

impl Analyzer for FailureClusterer {
    fn analyze(&self, event: &FailureEvent) -> Vec<AnalysisResult> {
        let mut hasher = DefaultHasher::new();

        // Hash errors
        for err in &event.errors {
            err.hash(&mut hasher);
        }

        // Hash function names and arg counts (structural hash)
        for func in &event.functions {
            func.name.hash(&mut hasher);
            func.args.len().hash(&mut hasher);
        }

        let cluster_id = hasher.finish();

        println!("=== [ANALYZER] FailureClusterer Generated ID: {:X} ===", cluster_id);

        vec![AnalysisResult::Cluster(ClusterInfo {
            id: cluster_id,
        })]
    }
}
