use std::collections::{HashMap, VecDeque, HashSet};
use axon_core::{Task, Batch};

const MAX_TASK_RETRIES: u32 = 3;

pub struct Coordinator {
    /// v0.0.28: Quarantined tasks (permanently failed)
    pub quarantined_tasks: HashSet<String>,
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
            quarantined_tasks: HashSet::new(),
            file_priorities: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        let file = task.target_file.clone().unwrap_or_else(|| "logic".to_string());

        // v0.0.31.xx: [ALR] Lineage-based deduplication
        // Before adding new task, remove ALL pending tasks for same file
        // to prevent stale accumulation (e.g., database.c: 14 tasks bug)
        let q = self.per_file_queues.entry(file.clone()).or_default();
        let prev_count = q.len();
        let mut final_task = task;
        if !q.is_empty() {
            tracing::info!(
                "🧹 [QUEUE_DEDUP] File '{}' had {} pending tasks - removing before add with state preservation",
                file, prev_count
            );
            // v0.0.31.21: Preserve cumulative rework counts from existing queue to prevent resets
            for existing in q.iter() {
                if existing.id == final_task.id {
                    final_task.rework_count = std::cmp::max(final_task.rework_count, existing.rework_count);
                    final_task.validator_rejections = std::cmp::max(final_task.validator_rejections, existing.validator_rejections);
                    final_task.senior_rejections = std::cmp::max(final_task.senior_rejections, existing.senior_rejections);
                    final_task.architecture_rejections = std::cmp::max(final_task.architecture_rejections, existing.architecture_rejections);
                    final_task.cargo_rejections = std::cmp::max(final_task.cargo_rejections, existing.cargo_rejections);
                    final_task.lsp_rejections = std::cmp::max(final_task.lsp_rejections, existing.lsp_rejections);
                    final_task.boss_interventions = std::cmp::max(final_task.boss_interventions, existing.boss_interventions);
                    if final_task.error_feedback.is_none() && existing.error_feedback.is_some() {
                        final_task.error_feedback = existing.error_feedback.clone();
                    }
                    if final_task.senior_comment.is_none() && existing.senior_comment.is_some() {
                        final_task.senior_comment = existing.senior_comment.clone();
                    }
                }
            }
            q.clear(); // Remove all stale pending tasks for this file
        }

        // v0.0.31.xx: Quarantine check after state preservation
        if self.quarantined_tasks.contains(&final_task.id) || final_task.rework_count >= MAX_TASK_RETRIES {
            tracing::warn!(
                "⚠️ [COORD_ADD_TASK_REJECTED] Skipping adding quarantined/limit-reached task: {} (rework_count: {})",
                final_task.id, final_task.rework_count
            );
            self.quarantined_tasks.insert(final_task.id.clone());
            return;
        }

        // v0.0.28: Visibility logging
        tracing::debug!("[COORD_ADD_TASK] Adding task {} to file queue '{}'", final_task.id, file);

        self.per_file_queues.entry(file.clone()).or_default().push_back(final_task);

        // v0.0.28: Log queue state after add
        let queue_len = self.per_file_queues.get(&file).map(|q| q.len()).unwrap_or(0);
        tracing::debug!("[COORD_QUEUE] File '{}' now has {} tasks", file, queue_len);
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
                    // v0.0.28: Retry Fuse - Check if task is quarantined
                    if self.quarantined_tasks.contains(&task.id) {
                        tracing::warn!("⚠️ [QUARANTINE_SKIP] Skipping quarantined task: {}", task.id);
                        continue;
                    }
                    // Check retry limit
                    if task.rework_count >= MAX_TASK_RETRIES {
                        tracing::error!(
                            "🛑 [TASK_QUARANTINED] task={} rework_count={} target_file={:?}",
                            task.id, task.rework_count, task.target_file
                        );
                        self.quarantined_tasks.insert(task.id.clone());
                        continue;
                    }
                    self.active_files.insert(file);
                    return Some(task);
                }
            }
        }
        None
    }

    pub fn build_batch(
        &mut self,
        dep_graph: &std::sync::MutexGuard<crate::dep_graph::DepGraph>,
        completed_titles: &HashSet<String>,
    ) -> Option<Batch> {
        // 1. Pick the highest priority task as seed that has all its dependencies satisfied
        let next_file = self.per_file_queues.keys()
            .filter(|f| {
                if self.active_files.contains(*f) { return false; }
                if let Some(q) = self.per_file_queues.get(*f) {
                    if let Some(first_task) = q.front() {
                        // Dependency Readiness Gate: 부모 의존성들이 모두 Completed 완료 상태인지 체크!
                        return first_task.dependencies.iter().all(|dep| completed_titles.contains(dep));
                    }
                }
                false
            })
            .max_by_key(|f| self.file_priorities.get(*f).map(|p| p.score()).unwrap_or(0))
            .cloned()?;

        let seed_task = loop {
            let t = self.per_file_queues.get_mut(&next_file)?.pop_front()?;
            if t.rework_count >= MAX_TASK_RETRIES {
                tracing::error!("🛑 [TASK_QUANRANTINED] task={} rework_count={} target_file={:?}", t.id, t.rework_count, t.target_file);
                self.quarantined_tasks.insert(t.id.clone());
                if self.per_file_queues.get(&next_file).unwrap().is_empty() {
                    return None;
                }
                continue;
            }
            break t;
        };
        
        let mut batch_tasks = vec![seed_task.clone()];
        let mut closure = HashSet::new();
        
        if let Some(target) = &seed_task.target_file {
            let file_node = format!("file:{}", target);
            closure = dep_graph.compute_impact(vec![file_node.clone()]);
            closure.insert(file_node);
        }

        // 2. Coalesce other ready tasks that fall within this closure
        let other_files: Vec<String> = self.per_file_queues.keys()
            .filter(|f| {
                if !closure.contains(&format!("file:{}", f)) || **f == next_file { return false; }
                if let Some(q) = self.per_file_queues.get(*f) {
                    if let Some(first_task) = q.front() {
                        // Dependency Readiness Gate
                        return first_task.dependencies.iter().all(|dep| completed_titles.contains(dep));
                    }
                }
                false
            })
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

    pub fn complete_task(&mut self, task: &Task, success: bool) {
        let file = task.target_file.clone().unwrap_or_else(|| "logic".to_string());
        self.active_files.remove(&file);

        // v0.0.29: [QUEUE_POP_AND_BREAK]
        // If the task was successful, clear any redundant tasks for this file from the queue.
        if success {
            if let Some(q) = self.per_file_queues.get_mut(&file) {
                if !q.is_empty() {
                    tracing::info!("✂️ [COORD_CLEANUP] Clearing {} redundant tasks for file '{}'", q.len(), file);
                    q.clear();
                }
            }
        }
    }

    // v0.0.31.xx: Invariant assertion - total tasks in queues
    pub fn queued_task_count(&self) -> usize {
        self.per_file_queues.values().map(|q| q.len()).sum()
    }

    // v0.0.31.xx: Invariant assertion - active files count
    pub fn active_files_count(&self) -> usize {
        self.active_files.len()
    }

    // v0.0.31.xx: Invariant assertion - all file keys that have pending tasks
    pub fn files_with_pending_tasks(&self) -> Vec<String> {
        self.per_file_queues
            .iter()
            .filter(|(_, q)| !q.is_empty())
            .map(|(f, _)| f.clone())
            .collect()
    }
}
