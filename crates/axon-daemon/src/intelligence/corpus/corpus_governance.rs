use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusProvenance {
    pub is_generated_code: bool,
    pub is_vendored_dependency: bool,
    pub is_ai_generated_sample: bool,
    pub is_minimized_benchmark: bool,
    
    pub trust_score: f64,
    pub entropy_authenticity: f64,
    pub quarantine_status: bool,
}

pub struct CorpusGovernance;

impl CorpusGovernance {
    /// Controls what AXON considers "authentic human legacy entropy".
    /// If an AI-generated mock file or heavily vendored C-binding drops the trust_score,
    /// it gets quarantined so it doesn't pollute the Promotion Metrics.
    pub fn evaluate_corpus_authenticity(_file_path: &str) -> CorpusProvenance {
        // Stub
        CorpusProvenance {
            is_generated_code: false,
            is_vendored_dependency: false,
            is_ai_generated_sample: false,
            is_minimized_benchmark: false,
            trust_score: 0.99,
            entropy_authenticity: 0.95,
            quarantine_status: false,
        }
    }
}
