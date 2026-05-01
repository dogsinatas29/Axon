use axon_core::rules::Rule;
use axon_core::Task;

pub struct PromptComposer;

impl PromptComposer {
    pub fn compose(
        base_prompt: &str,
        rules: &[Rule],
        task: &Task,
    ) -> String {
        let mut prompt = String::new();

        // 1. System base
        prompt.push_str(base_prompt);
        prompt.push_str("\n\n");

        // 2. Rules (Constraints)
        if !rules.is_empty() {
            prompt.push_str("### MANDATORY SYSTEM CONSTRAINTS (EXPERIENCE-BASED) ###\n");
            for (i, rule) in rules.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, rule.constraint));
            }
            prompt.push_str("\n");
        }

        // 3. Task details
        prompt.push_str("### TARGET TASK ###\n");
        prompt.push_str(&task.description);

        prompt
    }
}
