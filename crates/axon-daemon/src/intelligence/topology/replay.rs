use serde::{Deserialize, Serialize};
use super::symbol_graph::SymbolDependencyGraph;
use super::failure_attribution::FailureAttribution;
use super::scheduler::TopologyAwareScheduler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairSimulationInput {
    pub failure_message: String,
    pub ast_snapshot: Vec<crate::intelligence::ast::OwnedSymbol>,
    pub graph: SymbolDependencyGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairSimulationOutput {
    pub repair_order: Vec<String>,
    pub reopened_tasks: Vec<String>,
    pub radius: String,
    pub convergence_steps: usize,
    pub ripple_count: usize,
}

pub struct RepairReplaySimulator;

impl RepairReplaySimulator {
    pub fn simulate_failure(input: &RepairSimulationInput) -> RepairSimulationOutput {
        // 1. Attribution
        let attribution = FailureAttribution::attribute(
            &input.failure_message,
            "dummy_path",
            &input.graph,
            &input.ast_snapshot,
        );

        // 2. Scheduling
        let scheduler = TopologyAwareScheduler::new(input.graph.clone());
        let repair_tasks = scheduler.schedule_repair(&attribution);

        // 3. Metrics calculation
        let repair_order: Vec<String> = repair_tasks.iter().map(|t| t.target_symbol.clone()).collect();
        
        let mut reopened_tasks: Vec<String> = repair_tasks.iter().map(|t| t.task_id.clone()).collect();
        reopened_tasks.sort();
        reopened_tasks.dedup();

        // The ripple count is how many OTHER tasks were opened apart from the original failing owner.
        // If there's 1 task opened, ripple is 0. If 2 tasks opened, ripple is 1.
        let ripple_count = if reopened_tasks.is_empty() { 0 } else { reopened_tasks.len() - 1 };
        
        // Convergence steps (naive estimation based on graph depth/task count from failure)
        // Here we just use the number of repair tasks as a proxy for convergence steps
        let convergence_steps = repair_tasks.len();

        RepairSimulationOutput {
            repair_order,
            reopened_tasks,
            radius: format!("{:?}", attribution.repair_radius),
            convergence_steps,
            ripple_count,
        }
    }
}
