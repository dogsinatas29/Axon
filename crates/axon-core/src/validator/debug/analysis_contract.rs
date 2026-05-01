use crate::validator::error::ValidationError;
use crate::validator::types::FunctionSig;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEvent {
    pub code: String,
    pub errors: Vec<ValidationError>,
    pub functions: Vec<FunctionSig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisResult {
    Cluster(ClusterInfo),
    Rule(RuleCandidate),
    Coverage(CoverageDelta),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCandidate {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageDelta {
    pub pattern: String,
}

pub trait Analyzer: Send + Sync {
    fn analyze(&self, event: &FailureEvent) -> Vec<AnalysisResult>;
}
