use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleState {
    pub text: String,
    pub score: f64,
    pub last_updated: u64, // Unix timestamp
    pub hit_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleRegistry {
    pub rules: HashMap<String, RuleState>, // Key: hash(text)
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_rule(&self, key: &str) -> Option<&RuleState> {
        self.rules.get(key)
    }

    pub fn get_all_rules(&self) -> Vec<RuleState> {
        self.rules.values().cloned().collect()
    }
}
