use crate::validator::error::ValidationError;
use super::analysis_contract::{Analyzer, FailureEvent, AnalysisResult, RuleCandidate};

pub struct RuleGenerator;

impl Analyzer for RuleGenerator {
    fn analyze(&self, event: &FailureEvent) -> Vec<AnalysisResult> {
        let mut results = Vec::new();

        for err in &event.errors {
            match err {
                ValidationError::SyntaxError(msg) => {
                    results.push(AnalysisResult::Rule(RuleCandidate {
                        text: format!("Fix Python syntax error: {}", msg),
                    }));
                }
                ValidationError::MissingFunction(name) => {
                    results.push(AnalysisResult::Rule(RuleCandidate {
                        text: format!("Missing required function: {}", name),
                    }));
                }
                ValidationError::SignatureMismatch { name, expected, actual } => {
                    results.push(AnalysisResult::Rule(RuleCandidate {
                        text: format!("Correct function signature for '{}'. Expected {:?}, found {:?}", name, expected, actual),
                    }));
                }
                ValidationError::MissingComponent(name) => {
                    results.push(AnalysisResult::Rule(RuleCandidate {
                        text: format!("Implement missing component: {}", name),
                    }));
                }
                _ => {}
            }
        }

        if !results.is_empty() {
            println!("=== [ANALYZER] RuleGenerator Proposed {} Candidates ===", results.len());
        }

        results
    }
}
