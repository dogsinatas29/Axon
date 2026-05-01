use crate::validator::error::ValidationError;
use crate::validator::types::FunctionSig;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionArtifact {
    pub code: String,
    pub spec: Option<Vec<FunctionSig>>,
    pub stage: Stage,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stage {
    Parse,
    Extract,
    Match,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub code: String,
    pub expected: Vec<FunctionSig>,
}
