use super::rule_registry::RuleState;

/// Selects the most relevant Top-K rules to inject into the agent's prompt.
/// This prevents prompt bloating while ensuring the agent follows the best practices.
pub fn select_top_k(rules: &[RuleState], k: usize) -> Vec<RuleState> {
    let mut candidates = rules.to_vec();
    
    // Sort by score (descending)
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    
    candidates.into_iter().take(k).collect()
}

/// Formats the selected rules into a structured string for prompt injection.
pub fn format_for_prompt(rules: &[RuleState]) -> String {
    if rules.is_empty() {
        return String::new();
    }
    
    let mut output = String::from("\n### EXPERIENCE-BASED CONSTRAINTS (TOP-K) ###\n");
    for (i, rule) in rules.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, rule.text));
    }
    output.push_str("\n");
    output
}
