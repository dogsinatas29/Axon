use axon_core::rules::Constraint;
use serde::{Serialize, Deserialize};

/// Priority level for a constraint. Higher = enforced first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum ConstraintPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// A constraint with an associated priority and confidence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedConstraint {
    pub constraint: Constraint,
    pub priority: ConstraintPriority,
    pub confidence: f64, // 0.0 ~ 1.0 based on source rule score
}

impl PrioritizedConstraint {
    pub fn new(constraint: Constraint, rule_score: f64) -> Self {
        let priority = if rule_score >= 50.0 {
            ConstraintPriority::Critical
        } else if rule_score >= 30.0 {
            ConstraintPriority::High
        } else if rule_score >= 15.0 {
            ConstraintPriority::Medium
        } else {
            ConstraintPriority::Low
        };

        Self {
            constraint,
            priority,
            confidence: (rule_score / 100.0).min(1.0),
        }
    }
}

/// Sorts constraints by priority (Critical first).
pub fn sort_by_priority(constraints: &mut Vec<PrioritizedConstraint>) {
    constraints.sort_by(|a, b| b.priority.cmp(&a.priority));
}
