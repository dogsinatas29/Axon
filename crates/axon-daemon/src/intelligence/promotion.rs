use super::rule_registry::RuleState;
use axon_core::rules::Constraint;

/// Checks if a rule has reached the confidence threshold to become a deterministic constraint.
pub fn should_promote(rule: &RuleState) -> bool {
    rule.score >= 10.0 && rule.hit_count >= 5
}

/// Converts a high-confidence rule string into a structured AXON Constraint.
pub fn to_constraint(rule: &RuleState) -> Constraint {
    let text = rule.text.to_lowercase();
    
    if text.contains("signature") && text.contains("match") {
        // For promotion, we might need to extract the function name from the rule text
        // For now, mapping to a generic placeholder or the specific one if we can parse it
        Constraint::Custom("Signature Enforcement".to_string())
    } else if text.contains("implement") && text.contains("all") {
        Constraint::MustImplementAllSymbols
    } else if text.contains("python") && text.contains("syntax") {
        Constraint::PythonOnly
    } else {
        // Fallback to custom constraint
        Constraint::Custom(rule.text.clone())
    }
}
