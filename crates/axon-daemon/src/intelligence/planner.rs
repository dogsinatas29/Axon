use std::collections::{HashMap, VecDeque, HashSet};
use std::path::Path;
use axon_core::{Task, TaskStatus};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraphValidation {
    pub expected_components: usize,
    pub generated_tasks: usize,
    pub missing_tasks: Vec<String>,
    pub is_valid: bool,
}

/// v0.0.29: IR-based Execution Planner
/// Converts unstructured tasks into a deterministic Directed Acyclic Graph (DAG)
pub struct ExecutionPlanner {
    // We'll store task relationships here
}

impl ExecutionPlanner {
    pub fn new() -> Self {
        Self {}
    }

    /// v0.0.28: Heuristic Dependency Discovery
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

    /// v0.0.29: Task Graph Validation
    /// Compares architecture.md components with actual generated tasks.
    /// Aborts if expected components != generated tasks.
    pub fn validate_task_graph(&self, project_root: &str, tasks: &[Task]) -> TaskGraphValidation {
        let arch_path = Path::new(project_root).join("architecture.md");
        
        if !arch_path.exists() {
            return TaskGraphValidation {
                expected_components: 0,
                generated_tasks: tasks.len(),
                missing_tasks: vec![],
                is_valid: true,
            };
        }

        let content = std::fs::read_to_string(&arch_path).unwrap_or_default();
        
        let component_count = content.matches("### Component:").count();
        
        let mut missing = Vec::new();
        for task in tasks {
            let task_file = task.title.to_lowercase();
            if task_file.contains(".c") || task_file.contains(".h") || task_file.contains(".rs") || task_file.contains(".py") {
                if task.status == TaskStatus::Failed || task.status == TaskStatus::Pending {
                    missing.push(task.title.clone());
                }
            }
        }

        let is_valid = component_count == tasks.len() && missing.is_empty();
        
        if !is_valid {
            tracing::error!("[TASK_GRAPH_VALIDATION] FAILED: Expected {} components, got {} tasks", component_count, tasks.len());
            if !missing.is_empty() {
                tracing::error!("[TASK_GRAPH_VALIDATION] Missing/Failed tasks: {:?}", missing);
            }
        }

        TaskGraphValidation {
            expected_components: component_count,
            generated_tasks: tasks.len(),
            missing_tasks: missing,
            is_valid,
        }
    }

    /// v0.0.28: Topological Order Verification
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

    /// v0.0.29: Dependency-aware Ordering Validation
    /// Enforces: Headers → Implementations → main.c (last)
    /// Returns error if main.c is generated before header/implementation tasks.
    pub fn validate_ordering(&self, tasks: &[Task]) -> Result<(), String> {
        let mut header_count = 0;
        let mut impl_count = 0;
        let mut main_position = None;
        let total = tasks.len();

        for (i, task) in tasks.iter().enumerate() {
            let name = task.title.to_lowercase();
            
            if name.ends_with(".h") {
                header_count += 1;
            } else if name.ends_with(".c") || name.ends_with(".rs") || name.ends_with(".py") {
                if name.contains("main") {
                    main_position = Some(i);
                } else {
                    impl_count += 1;
                }
            }
        }

        // Rule: main.c must be the LAST task
        if let Some(pos) = main_position {
            if pos != total - 1 {
                return Err(format!(
                    "ORDERING VIOLATION: main.c must be generated LAST. Current position: {}, Total: {}",
                    pos + 1, total
                ));
            }
        }

        // Rule: Headers should come before implementations (if both exist)
        if header_count > 0 && impl_count > 0 {
            let header_positions: Vec<usize> = tasks.iter()
                .enumerate()
                .filter(|(_, t)| t.title.to_lowercase().ends_with(".h"))
                .map(|(i, _)| i)
                .collect();
            
            let impl_positions: Vec<usize> = tasks.iter()
                .enumerate()
                .filter(|(_, t)| {
                    let n = t.title.to_lowercase();
                    (n.ends_with(".c") || n.ends_with(".rs") || n.ends_with(".py")) && !n.contains("main")
                })
                .map(|(i, _)| i)
                .collect();

            if let (Some(last_h), Some(first_i)) = (header_positions.last(), impl_positions.first()) {
                if last_h > first_i {
                    return Err(format!(
                        "ORDERING VIOLATION: Headers must be generated BEFORE implementations. Found header at position {} after implementation at position {}",
                        last_h + 1, first_i + 1
                    ));
                }
            }
        }

        Ok(())
    }
}
