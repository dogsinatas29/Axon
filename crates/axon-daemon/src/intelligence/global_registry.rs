use std::collections::HashMap;
use super::rule_registry::{RuleRegistry, RuleState};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalRegistry {
    pub rules: HashMap<String, RuleState>,
}

impl GlobalRegistry {
    /// Pushes high-scoring local rules to the global registry for cross-project sharing.
    pub fn push_from_local(&mut self, local: &RuleRegistry) {
        for (key, rule) in &local.rules {
            if rule.score >= 20.0 {
                // Only push proven rules
                self.rules.entry(key.clone())
                    .and_modify(|r| {
                        if rule.score > r.score {
                            *r = rule.clone();
                        }
                    })
                    .or_insert(rule.clone());
            }
        }
    }

    /// Pulls global knowledge into the local project registry.
    pub fn pull_to_local(&self, local: &mut RuleRegistry) {
        for (key, rule) in &self.rules {
            local.rules.entry(key.clone()).or_insert(rule.clone());
        }
    }
}
