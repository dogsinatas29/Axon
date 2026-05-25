use super::symbol_graph::SymbolDependencyGraph;
use super::failure_attribution::FailureAttribution;

#[derive(Debug, Clone)]
pub struct RepairTask {
    pub task_id: String,
    pub priority: f32,
    pub target_symbol: String,
}

pub struct TopologyAwareScheduler {
    pub graph: SymbolDependencyGraph,
}

impl TopologyAwareScheduler {
    pub fn new(graph: SymbolDependencyGraph) -> Self {
        Self { graph }
    }

    pub fn calculate_priority(&self, attribution: &FailureAttribution, symbol: &str) -> f32 {
        let node = self.graph.nodes.get(symbol);
        
        let dependency_centrality = match node {
            Some(n) => n.dependents.len() as f32,
            None => 0.0,
        };

        // ownership_confidence, failure_locality, retry_probability are simplified here
        let ownership_confidence = if node.is_some() && node.unwrap().owner_task_id.is_some() { 1.0 } else { 0.5 };
        let failure_locality = match attribution.repair_radius {
            super::repair_radius::RepairRadius::SymbolOnly => 1.0,
            super::repair_radius::RepairRadius::DirectDependents => 0.8,
            super::repair_radius::RepairRadius::FileWide => 0.5,
            super::repair_radius::RepairRadius::PhaseWide => 0.2,
        };
        let retry_probability = 1.0; // Assume 1st retry for now

        // priority = ownership_confidence * dependency_centrality * failure_locality * retry_probability
        let base_centrality = 1.0 + dependency_centrality;

        ownership_confidence * base_centrality * failure_locality * retry_probability
    }

    pub fn schedule_repair(&self, attribution: &FailureAttribution) -> Vec<RepairTask> {
        let mut tasks = Vec::new();

        if let Some(ref sym) = attribution.failed_symbol {
            if let Some(node) = self.graph.nodes.get(sym) {
                if let Some(ref owner) = node.owner_task_id {
                    tasks.push(RepairTask {
                        task_id: owner.clone(),
                        priority: self.calculate_priority(attribution, sym),
                        target_symbol: sym.clone(),
                    });
                }

                // If radius is DirectDependents, also schedule dependents
                if attribution.repair_radius == super::repair_radius::RepairRadius::DirectDependents {
                    for dep in &node.dependents {
                        if let Some(dep_node) = self.graph.nodes.get(dep) {
                            if let Some(ref dep_owner) = dep_node.owner_task_id {
                                if !tasks.iter().any(|t| t.task_id == *dep_owner) {
                                    tasks.push(RepairTask {
                                        task_id: dep_owner.clone(),
                                        priority: self.calculate_priority(attribution, dep),
                                        target_symbol: dep.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by priority descending
        tasks.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        tasks
    }
}
