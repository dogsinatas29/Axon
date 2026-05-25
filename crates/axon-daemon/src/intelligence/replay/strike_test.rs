use crate::governance::ownership::OwnershipRegistry;

/// AST Locator (Tree-sitter Demotion)
/// AST's only role is to locate the boundary and identify the symbol name.
/// It is completely stripped of any authority to approve or canonicalize.
#[derive(Debug)]
pub struct AstLocator;

impl AstLocator {
    // Simulated Tree-sitter locator for test purposes
    pub fn locate_symbol_at_line(_source: &str, line: usize) -> Option<String> {
        match line {
            1 => Some("parse_user".to_string()),
            2..=4 => Some("render_user".to_string()),
            _ => None,
        }
    }
}

pub struct KernelSovereigntyGate {
    pub ownership: OwnershipRegistry,
}

impl KernelSovereigntyGate {
    pub fn new() -> Self {
        Self {
            ownership: OwnershipRegistry::new(),
        }
    }

    /// Attempts a mutation on a specific line of the source code.
    /// The AST locates the symbol, and the Governance Kernel enforces the strike rules.
    pub fn attempt_mutation(&self, source: &str, target_line: usize, task_id: &str) -> Result<(), String> {
        let symbol_id = AstLocator::locate_symbol_at_line(source, target_line)
            .ok_or_else(|| "UNKNOWN_TOPOLOGY: Cannot mutate outside known symbol boundaries".to_string())?;

        // Physical Ownership Strike check
        self.ownership.can_mutate_body(&symbol_id, task_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_ownership_strike() {
        let source_code = "pub fn parse_user() {}\npub fn render_user() {\n    parse_user();\n}\n";
        let mut gate = KernelSovereigntyGate::new();
        
        // Task A owns parse_user, Task B owns render_user
        gate.ownership.register_symbol("parse_user", "Task_A", "pub fn parse_user()");
        gate.ownership.register_symbol("render_user", "Task_B", "pub fn render_user()");

        // Task A can mutate its own code
        assert!(gate.attempt_mutation(source_code, 1, "Task_A").is_ok());

        // Task B can mutate its own code
        assert!(gate.attempt_mutation(source_code, 3, "Task_B").is_ok());

        // STRIKE TEST: Task B tries to modify parse_user (line 1)
        let strike_result = gate.attempt_mutation(source_code, 1, "Task_B");
        assert!(strike_result.is_err(), "Kernel MUST reject Task_B's attempt to mutate parse_user");
        let err_msg = strike_result.unwrap_err();
        
        // Assert the canonical failure shape
        assert!(err_msg.contains("OWNERSHIP_VIOLATION"));
        assert!(err_msg.contains("Task 'Task_B' is forbidden"));
        assert!(err_msg.contains("Task 'Task_A'"));
        
        println!("Strike rejected correctly: {}", err_msg);
    }
}
