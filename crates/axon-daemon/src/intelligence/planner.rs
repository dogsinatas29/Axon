use std::collections::{HashMap, VecDeque, HashSet};
use axon_core::Task;

/// v0.0.23: IR-based Execution Planner
/// Converts unstructured tasks into a deterministic Directed Acyclic Graph (DAG)
pub struct ExecutionPlanner {
    // We'll store task relationships here
}

impl ExecutionPlanner {
    pub fn new() -> Self {
        Self {}
    }

    /// v0.0.23: Heuristic Dependency Discovery
    /// Analyzes tasks and automatically links them based on file references.
    /// e.g., if 'calculate.rs' task mentions 'validation.rs' in its description.
    pub fn plan_dependencies(&self, tasks: &mut [Task]) {
        let mut file_to_task_id = HashMap::new();
        
        // Map files to their task IDs
        for task in tasks.iter() {
            let file_name = task.title.split_whitespace().last().unwrap_or(&task.title);
            file_to_task_id.insert(file_name.to_lowercase(), task.id.clone());
        }

        // Discovery phase
        for i in 0..tasks.len() {
            let mut deps = Vec::new();
            let desc = tasks[i].description.to_lowercase();
            let title = tasks[i].title.to_lowercase();

            for (file, id) in &file_to_task_id {
                if *id == tasks[i].id { continue; }
                
                // If this task's description or title mentions another task's file, 
                // it's likely a dependency (heuristic).
                if desc.contains(file) || title.contains(file) {
                    deps.push(id.clone());
                }
            }
            tasks[i].dependencies = deps;
        }
    }

    /// v0.0.23: Topological Order Verification
    /// Checks if the current task set has cycles.
    pub fn verify_dag(&self, tasks: &[Task]) -> bool {
        let mut graph = HashMap::new();
        let mut indegree = HashMap::new();
        let mut nodes = HashSet::new();

        for task in tasks {
            nodes.insert(task.id.clone());
            indegree.entry(task.id.clone()).or_insert(0);
            for dep in &task.dependencies {
                graph.entry(dep.clone()).or_insert_with(Vec::new).push(task.id.clone());
                *indegree.entry(task.id.clone()).or_insert(0) += 1;
            }
        }

        let mut q = VecDeque::new();
        for node in &nodes {
            if *indegree.get(node).unwrap_or(&0) == 0 {
                q.push_back(node.clone());
            }
        }

        let mut count = 0;
        while let Some(u) = q.pop_front() {
            count += 1;
            if let Some(neighbors) = graph.get(&u) {
                for v in neighbors {
                    let deg = indegree.get_mut(v).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        q.push_back(v.clone());
                    }
                }
            }
        }

        count == nodes.len()
    }
}
