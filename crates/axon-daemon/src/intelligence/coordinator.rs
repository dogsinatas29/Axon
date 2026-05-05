use std::collections::{HashMap, VecDeque, HashSet};
use axon_core::{Task, Batch};

pub struct Coordinator {
    /// file_path -> tasks (Per-file Queuing)
    pub per_file_queues: HashMap<String, VecDeque<Task>>,
    /// Current active files being modified by workers
    pub active_files: HashSet<String>,
    /// File priorities for bottleneck resolution
    pub file_priorities: HashMap<String, FilePriority>,
}

#[derive(Default, Clone)]
pub struct FilePriority {
    pub fan_in: u32,
    pub rework_count: u32,
    pub failure_count: u32,
}

impl FilePriority {
    pub fn score(&self) -> u32 {
        (self.fan_in * 3) + (self.rework_count * 2) + self.failure_count
    }
}

impl Coordinator {
    pub fn new() -> Self {
        Self {
            per_file_queues: HashMap::new(),
            active_files: HashSet::new(),
            file_priorities: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        let file = task.target_file.clone().unwrap_or_else(|| "logic".to_string());
        
        // v0.0.25: [ALR] Task Coalescing for Reworks
        if task.id.starts_with("rework") {
            let q = self.per_file_queues.entry(file.clone()).or_default();
            q.retain(|t| !t.id.starts_with("rework")); // Drop stale reworks
        }

        self.per_file_queues.entry(file).or_default().push_back(task);
    }

    pub fn update_priority(&mut self, file: &str, rework: bool, failure: bool, fan_in: u32) {
        let p = self.file_priorities.entry(file.to_string()).or_default();
        if rework { p.rework_count += 1; }
        if failure { p.failure_count += 1; }
        if fan_in > 0 { p.fan_in = fan_in; }
    }

    pub fn next_task(&mut self) -> Option<Task> {
        // 1. Pick highest priority file that is NOT active
        let mut available_files: Vec<String> = self.per_file_queues.keys()
            .filter(|f| !self.active_files.contains(*f) && !self.per_file_queues.get(*f).unwrap().is_empty())
            .cloned()
            .collect();
        
        if available_files.is_empty() { return None; }

        // Sort by score descending
        available_files.sort_by_key(|f| std::cmp::Reverse(self.file_priorities.get(f).cloned().unwrap_or_default().score()));

        for file in available_files {
            if let Some(q) = self.per_file_queues.get_mut(&file) {
                if let Some(task) = q.pop_front() {
                    self.active_files.insert(file);
                    return Some(task);
                }
            }
        }
        None
    }

    pub fn build_batch(&mut self, dep_graph: &std::sync::MutexGuard<crate::dep_graph::DepGraph>) -> Option<Batch> {
        // 1. Pick the highest priority task as seed
        let next_file = self.per_file_queues.keys()
            .filter(|f| !self.active_files.contains(*f) && !self.per_file_queues.get(*f).unwrap().is_empty())
            .max_by_key(|f| self.file_priorities.get(*f).map(|p| p.score()).unwrap_or(0))
            .cloned()?;

        let seed_task = self.per_file_queues.get_mut(&next_file)?.pop_front()?;
        
        let mut batch_tasks = vec![seed_task.clone()];
        let mut closure = HashSet::new();
        
        if let Some(target) = &seed_task.target_file {
            let file_node = format!("file:{}", target);
            closure = dep_graph.compute_impact(vec![file_node.clone()]);
            closure.insert(file_node);
        }

        // 2. Coalesce other ready tasks that fall within this closure
        let other_files: Vec<String> = self.per_file_queues.keys()
            .filter(|f| closure.contains(&format!("file:{}", f)) && **f != next_file)
            .cloned()
            .collect();

        for f in other_files {
            if let Some(q) = self.per_file_queues.get_mut(&f) {
                if let Some(t) = q.pop_front() {
                    batch_tasks.push(t);
                }
            }
        }

        let priority = self.file_priorities.get(&next_file).map(|p| p.score()).unwrap_or(0);

        // Mark all files in batch as active
        for t in &batch_tasks {
            if let Some(target) = &t.target_file {
                self.active_files.insert(target.clone());
            }
        }

        Some(Batch {
            id: uuid::Uuid::new_v4().to_string(),
            tasks: batch_tasks,
            dependency_closure: closure,
            priority,
        })
    }

    pub fn complete_task(&mut self, task: &Task) {
        let file = task.target_file.clone().unwrap_or_else(|| "logic".to_string());
        self.active_files.remove(&file);
    }
}
