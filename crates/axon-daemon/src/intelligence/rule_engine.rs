use super::rule_registry::{RuleRegistry, RuleState};
use axon_core::validator::debug::analysis_contract::RuleCandidate;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct RuleEngine {
    pub registry: RuleRegistry,
}

impl RuleEngine {
    pub fn new(registry: RuleRegistry) -> Self {
        Self { registry }
    }

    /// Ingests new rule candidates and updates their state.
    pub fn ingest(&mut self, candidates: Vec<RuleCandidate>, now: u64) {
        for candidate in candidates {
            let key = self.generate_key(&candidate.text);
            
            let state = self.registry.rules.entry(key).or_insert(RuleState {
                text: candidate.text.clone(),
                score: 0.0,
                last_updated: now,
                hit_count: 0,
            });

            // 1. Scoring
            Self::update_score(state);

            // 2. Decay (Time-based)
            Self::apply_time_decay(state, now);

            state.last_updated = now;
        }
    }

    fn update_score(state: &mut RuleState) {
        state.hit_count += 1;
        state.score += 1.0;
        println!("=== [ENGINE] Rule Score Updated: '{}' -> {}", state.text, state.score);
    }

    fn apply_time_decay(state: &mut RuleState, now: u64) {
        let dt = now.saturating_sub(state.last_updated);
        // Decay by 0.001 per second of inactivity
        let penalty = (dt as f64) * 0.001;
        state.score = (state.score - penalty).max(0.0);
        
        if penalty > 0.0 {
            println!("=== [ENGINE] Time Decay Applied: -{} to '{}'", penalty, state.text);
        }
    }

    pub fn apply_failure_decay(&mut self, key: &str) {
        if let Some(state) = self.registry.rules.get_mut(key) {
            state.score *= 0.9; // 10% penalty
            println!("=== [ENGINE] Failure Decay Applied: '{}' -> {}", state.text, state.score);
        }
    }

    fn generate_key(&self, text: &str) -> String {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:X}", hasher.finish())
    }

    /// Produces constraint proposals from promoted rules.
    /// This is a pure output function — no side effects on IR.
    pub fn produce_constraints(&self) -> Vec<super::staging::ConstraintProposal> {
        self.registry.rules.values()
            .filter(|r| super::promotion::should_promote(r))
            .map(|r| super::staging::ConstraintProposal {
                constraint: super::promotion::to_constraint(r),
                source_rule: r.text.clone(),
            })
            .collect()
    }
}
