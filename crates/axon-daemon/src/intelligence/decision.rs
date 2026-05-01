use super::rule_registry::RuleRegistry;
use super::{promotion, selection, global_registry};

pub struct DecisionLayer {
    pub global: global_registry::GlobalRegistry,
}

impl DecisionLayer {
    pub fn new() -> Self {
        Self {
            global: global_registry::GlobalRegistry::default(),
        }
    }

    /// Processes the current rule registry to perform promotion, selection, and sync.
    pub fn process(&mut self, local_registry: &mut RuleRegistry) -> String {
        println!("=== [DECISION] Processing Intelligence Layer ===");

        // 1. Cross-Project Sync
        self.global.pull_to_local(local_registry);
        self.global.push_from_local(local_registry);

        // 2. Rule Promotion
        for rule in local_registry.rules.values() {
            if promotion::should_promote(rule) {
                let constraint = promotion::to_constraint(rule);
                println!("   -> PROMOTED to Constraint: {:?}", constraint);
                // In a real implementation, this would be registered in the project's constraint set
            }
        }

        // 3. Top-K Selection for Prompt
        let top_rules = selection::select_top_k(
            &local_registry.rules.values().cloned().collect::<Vec<_>>(),
            5
        );

        selection::format_for_prompt(&top_rules)
    }
}
