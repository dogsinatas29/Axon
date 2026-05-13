use super::rule_registry::RuleState;
use axon_ir::schema::Constraint;

/// Checks if a rule has reached the confidence threshold to become a deterministic constraint.
pub fn should_promote(rule: &RuleState) -> bool {
    rule.score >= 10.0 && rule.hit_count >= 5
}

/// Converts a high-confidence rule string into a structured AXON Constraint.
pub fn to_constraint(rule: &RuleState) -> Constraint {
    let text = rule.text.to_lowercase();

    let kind = if text.contains("signature") && text.contains("match") {
        "SignatureMatch"
    } else if text.contains("implement") && text.contains("all") {
        "MustImplementAll"
    } else if text.contains("python") && text.contains("syntax") {
        "PythonOnly"
    } else {
        "Custom"
    };

    Constraint {
        id: 0,
        kind: kind.to_string(),
        target: rule.text.clone(),
        condition: "".to_string(),
        message: format!("Promoted from rule: {}", rule.text),
    }
}
