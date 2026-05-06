/*
 * AXON - The Automated Software Factory
 * Copyright (C) 2026 dogsinatas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

pub mod server;
pub mod controller;
pub mod admin;
pub mod debug_hook;
pub mod intelligence;
pub mod dep_graph;
pub mod execution_validator;
use crate::dep_graph::DepGraph;
use rusqlite::params;
use axon_core::events;
use axon_dispatcher::Dispatcher;
use axon_core::BatchAssignment;
use axon_storage::Storage;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::mpsc;
use std::collections::HashMap;
use crate::intelligence::decision::*;

// Legacy routing types removed in v0.0.25

pub struct BootstrapManager {
    pub project_id: String,
    pub sandbox_root: PathBuf,
}


#[derive(Clone)]
pub struct Daemon {
    pub dispatcher: Arc<Dispatcher>,
    pub storage: Arc<Storage>,
    pub architect_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
    pub architect_model_name: String,
    pub senior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
    pub senior_model_names: Vec<String>,
    pub junior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
    pub junior_model_names: Vec<String>,
    pub event_bus: Arc<events::EventBus>,
    pub architecture_guide: String,
    pub pause_tx: Arc<tokio::sync::watch::Sender<bool>>,
    pub pause_rx: tokio::sync::watch::Receiver<bool>,
    pub locale: String,
    pub controller: Arc<controller::ControlSystem>,
    pub lounge: Arc<axon_agent::lounge::LoungeManager>,
    pub admin: Arc<admin::AdminSystem>,
    pub rr_indices: Arc<std::sync::Mutex<std::collections::HashMap<axon_core::AgentRole, usize>>>,
    pub throttler: Arc<tokio::sync::Semaphore>,
    pub sampling_rate: f64,
    pub task_counter: Arc<std::sync::atomic::AtomicUsize>,
    pub validation_counter: Arc<std::sync::atomic::AtomicUsize>, // v0.0.25: Track cycles for periodic full check
    pub coordinator: Arc<std::sync::Mutex<intelligence::coordinator::Coordinator>>,
    pub final_gate_lock: Arc<tokio::sync::Mutex<()>>,
    pub dep_graph: Arc<std::sync::Mutex<DepGraph>>,
}


#[allow(dead_code)]
impl Daemon {
    pub fn publish_event(&self, event: axon_core::Event) {
        if let Err(e) = self.storage.save_event(&event) {
            tracing::error!("❌ [DB_EVENT_FAIL] Failed to save event to database: {}", e);
        }
        self.event_bus.publish(event);
    }

    fn resolve_tool_path(name: &str) -> String {
        if let Ok(mut curr) = std::env::current_dir() {
            for _ in 0..10 {
                let path = curr.join("tools").join(name);
                if path.exists() {
                    return path.to_string_lossy().to_string();
                }
                if !curr.pop() { break; }
            }
        }
        format!("tools/{}", name)
    }

    fn get_current_project_state(&self, project_id: &str) -> String {
        let mut state = std::collections::HashMap::new();
        let project_path = std::path::Path::new(project_id);
        if project_path.exists() {
            let mut stack = vec![project_path.to_path_buf()];
            while let Some(dir) = stack.pop() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if name.starts_with('.') || name == "target" || name == "crates" || name == "tools" || name == "mile_stone" {
                                continue;
                            }
                            stack.push(path);
                        } else {
                            let rel_path = path.strip_prefix(project_path).unwrap_or(&path);
                            let fname = rel_path.to_string_lossy();
                            if fname.starts_with('.') {
                                continue;
                            }
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                state.insert(fname.to_string(), content);
                            }
                        }
                    }
                }
            }
        }
        serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string())
    }

    fn record_failure_trace(&self, task_id: &str, error: &str, file: &str, symbol: &str, stage: &str) {
        let trace_dir = ".axon_trace";
        let _ = std::fs::create_dir_all(trace_dir);
        let path = format!("{}/traces.ndjson", trace_dir);
        
        let trace = serde_json::json!({
            "ts": chrono::Local::now().to_rfc3339(),
            "task_id": task_id,
            "error": error,
            "file": if file.is_empty() { None } else { Some(file) },
            "symbol": if symbol.is_empty() { None } else { Some(symbol) },
            "stage": stage
        });

        if let Ok(content) = serde_json::to_string(&trace) {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path) 
            {
                let _ = writeln!(file, "{}", content);
            }
        }
    }

    pub fn new(
        storage: Arc<Storage>, 
        architect_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
        architect_model_name: String,
        senior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
        senior_model_names: Vec<String>,
        junior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
        junior_model_names: Vec<String>,
        worker_tx: mpsc::Sender<BatchAssignment>,
        architecture_guide: String,
        sampling_rate: f64,
        locale: String,
    ) -> Self {
        let event_bus = Arc::new(events::EventBus::new(100));
        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        
        tracing::info!("🌐 Active Factory Locale: {}", locale);

        Self {
            dispatcher: Arc::new(Dispatcher::new(worker_tx).with_limit(100)), // v0.0.25: Expanded queue for heavy workloads
            storage: storage.clone(),
            architect_model,
            architect_model_name,
            senior_models,
            senior_model_names,
            junior_models,
            junior_model_names,
            event_bus: event_bus.clone(),
            architecture_guide,
            pause_tx: Arc::new(pause_tx),
            pause_rx,
            locale,
            controller: Arc::new(controller::ControlSystem::new()),
            lounge: Arc::new(axon_agent::lounge::LoungeManager::new(".").with_event_bus(event_bus.clone())),
            admin: Arc::new(admin::AdminSystem::new(storage.clone())),
            rr_indices: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            throttler: Arc::new(tokio::sync::Semaphore::new(1)),
            sampling_rate,
            task_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            validation_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            coordinator: Arc::new(std::sync::Mutex::new(intelligence::coordinator::Coordinator::new())),
            final_gate_lock: Arc::new(tokio::sync::Mutex::new(())),
            dep_graph: Arc::new(std::sync::Mutex::new(dep_graph::DepGraph::new())),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("AXON Daemon starting (Multi-Worker Mode - Phase 07)...");
        
        // v0.0.25: Ensure Lounge thread exists for UI visibility on resume
        let _ = self.setup_lounge();
        
        // RECOVERY (v0.0.15): DB에서 처리되지 않은 태스크들을 불러와 스케줄러 큐에 재진입시킵니다.
        // v0.0.23: Use ExecutionPlanner to build the DAG before enqueuing
        if let Ok(tasks) = self.storage.list_all_tasks() {
            let planner = intelligence::planner::ExecutionPlanner::new();
            
            // Filter only pending/in-progress tasks
            let mut ready_tasks: Vec<_> = tasks.into_iter()
                .filter(|t| t.status == axon_core::TaskStatus::Pending || t.status == axon_core::TaskStatus::InProgress)
                .collect();

            if !ready_tasks.is_empty() {
                planner.plan_dependencies(&mut ready_tasks);
                
                let mut recovered_count = 0;
                let mut coordinator_tasks = Vec::new();
                for mut task in ready_tasks {
                    task.status = axon_core::TaskStatus::Pending;
                    let _ = self.storage.save_task(&task);
                    coordinator_tasks.push(task);
                    recovered_count += 1;
                }
                
                // v0.0.25: Load tasks into Coordinator SSOT
                {
                    let mut coord = self.coordinator.lock().unwrap();
                    *coord = intelligence::coordinator::Coordinator::new();
                    for t in coordinator_tasks {
                        coord.add_task(t);
                    }
                    
                    // v0.0.25: [ALR] Initialize Priorities from DepGraph
                    let graph = self.dep_graph.lock().unwrap();
                    for (node_id, _) in &graph.nodes {
                        if node_id.starts_with("file:") {
                            let fname = node_id.replace("file:", "");
                            let deps = graph.edges_in.get(node_id).map(|s| s.len() as u32).unwrap_or(0);
                            coord.update_priority(&fname, false, false, deps);
                        }
                    }
                }
                
                tracing::info!("♻️ Recovered {} unfinished tasks with Coordinator Per-file Queue mapping.", recovered_count);
            }
        }
        
        // v0.0.25: [ALR] Multi-worker scale based on available junior agents
        let worker_count = self.junior_models.len().max(1); 
        let mut worker_handles = Vec::new();
        
        for i in 0..worker_count {
            let daemon = self.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = daemon.worker_loop(i).await {
                    tracing::error!("❌ Worker {} crashed: {}", i, e);
                }
            });
            worker_handles.push(handle);
        }
        
        tracing::info!("👷 {} workers activated and ready.", worker_count);

        // Keep the main run task alive until all workers exit (which they shouldn't)
        for h in worker_handles {
            let _ = h.await;
        }

        Ok(())
    }


    pub fn submit_task(&self, task: axon_core::Task) {
        let mut coord = self.coordinator.lock().unwrap();
        coord.add_task(task);
    }

    async fn worker_loop(&self, id: usize) -> anyhow::Result<()> {
        let mut pause_rx = self.pause_rx.clone();
        
        loop {
            // Check pause status
            if *pause_rx.borrow() {
                if pause_rx.changed().await.is_err() {
                    break;
                }
                continue;
            }

            // v0.0.25: Request BATCH from Coordinator (Dependency Clustering)
            let ready_batch = {
                let mut coord = self.coordinator.lock().unwrap();
                let graph = self.dep_graph.lock().unwrap();
                coord.build_batch(&graph)
            };

            if let Some(batch) = ready_batch {
                tracing::info!("👷 [Worker {}] Coordinator DISPATCHED batch {} with {} tasks", id, batch.id, batch.tasks.len());
                self.publish_event(axon_core::Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: "system".to_string(),
                    thread_id: None,
                    agent_id: Some(format!("WORKER-{}", id)),
                    event_type: axon_core::EventType::SystemLog,
                    source: format!("WORKER-{}", id),
                    content: format!("👷 [Worker {}] DISPATCHED batch {} ({} tasks)", id, batch.id, batch.tasks.len()),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
                
                let result = self.handle_assignment(BatchAssignment { batch }).await;
                
                if let Err(e) = result {
                    tracing::error!("❌ [Worker {}] Task execution failed: {}", id, e);
                }
                
                // Physical cooldown to avoid API burst on multi-worker
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            } else {
                // Wait for new tasks or dependencies to be satisfied
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        }

        Ok(())
    }

    pub async fn handle_assignment(&self, assignment: BatchAssignment) -> anyhow::Result<()> {
        let batch = assignment.batch;
        let batch_id = batch.id.clone();
        let _start_total = std::time::Instant::now();
        
        tracing::info!("⚙️ [BATCH_START] Processing batch {} with {} tasks.", batch.id, batch.tasks.len());
        self.publish_event(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "system".to_string(),
            thread_id: None,
            agent_id: None,
            event_type: axon_core::EventType::SystemLog,
            source: "BATCH_PROCESSOR".to_string(),
            content: format!("⚙️ [BATCH_START] Processing batch {} ({} tasks)", batch.id, batch.tasks.len()),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        // Phase 1: Generation (Parallel or Sequential within worker context)
        let mut results = Vec::new();
        let mut backups = HashMap::new();
        let mut all_metrics = Vec::new();

        for task in &batch.tasks {
            // Backup files before modification
            if let Some(target) = &task.target_file {
                let fpath = std::path::Path::new(&task.project_id).join(target);
                if let Ok(content) = std::fs::read(&fpath) {
                    backups.insert(target.clone(), content);
                }
            }

            // Execute task through Junior agent
            let result = self.execute_junior_task(task).await;
            results.push(result);
        }

        // Phase 2: Senior Review (Batch context)
        let mut all_approved = true;
        let mut failures = Vec::new();
        let mut senior_comments = std::collections::HashMap::new();

        for (i, res) in results.into_iter().enumerate() {
            let task = &batch.tasks[i];
            match res {
                Ok((patch, metrics)) => {
                    all_metrics.push(metrics);
                    // Senior review
                    match self.verify_with_senior(task, &patch).await {
                        Err(err) => {
                            all_approved = false;
                            failures.push(format!("Task {} failed senior review: {}", task.id, err));
                        }
                        Ok(comment) => {
                            senior_comments.insert(task.id.clone(), comment);
                            // Apply patch dry-run
                                // v0.0.25: Final materialization of the code to disk
                                if let Some(target) = &task.target_file {
                                    let fpath = std::path::Path::new(&task.project_id).join(target);
                                    let _ = std::fs::write(&fpath, &patch);
                                    
                                    self.publish_event(axon_core::Event {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        project_id: task.project_id.clone(),
                                        thread_id: Some(task.id.clone()),
                                        agent_id: None,
                                        event_type: axon_core::EventType::ArtifactCreated,
                                        source: "daemon".to_string(),
                                        content: format!("💾 Code materialized: {} -> {:?}", task.title, fpath),
                                        payload: None,
                                        timestamp: chrono::Local::now(),
                                    });
                                }
                        }
                    }
                }
                Err(err) => {
                    all_approved = false;
                    failures.push(format!("Task {} junior execution failed: {}", task.id, err));
                }
            }
        }
        
        if !all_approved {
            tracing::error!("❌ [BATCH_REJECT] Senior rejected batch {}. Rolling back.", batch.id);
            
            // v0.0.25: Strategic Visibility - Alert UI of batch failure
            self.publish_event(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: batch.tasks[0].project_id.clone(),
                thread_id: None,
                agent_id: None,
                event_type: axon_core::EventType::Signal,
                source: "daemon".to_string(),
                content: format!("🚨 [BATCH_REJECT] Senior rejected batch {}. Check logs for details.", batch.id),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            for failure in &failures {
                tracing::error!("   -> {}", failure);
                // Also publish individual task failures as signals
                self.publish_event(axon_core::Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: batch.tasks[0].project_id.clone(),
                    thread_id: None,
                    agent_id: None,
                    event_type: axon_core::EventType::Signal,
                    source: "daemon".to_string(),
                    content: format!("🚩 [TASK_FAIL] {}", failure),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
            }
            for (fname, content) in backups {
                let fpath = std::path::Path::new(&batch.tasks[0].project_id).join(fname);
                let _ = std::fs::write(fpath, content);
            }
            // v0.0.26: Stage 4 - Self-Healing Loop (Re-queue with Feedback)
            let mut coord = self.coordinator.lock().unwrap();
            for task in &batch.tasks {
                let mut rework_task = task.clone();
                rework_task.status = axon_core::TaskStatus::Failed;
                rework_task.rework_count += 1;
                
                // Combine all batch failures into the task feedback
                let combined_failure = failures.join("\n---\n");
                rework_task.error_feedback = Some(combined_failure);
                
                // Persistence & Re-queue
                let _ = self.storage.save_task(&rework_task);
                coord.complete_task(task); // Release current lock
                coord.add_task(rework_task); // Re-queue for next attempt
            }
            return Ok(());
        }

        // =========================================================================
        // v0.0.25: [FINAL_GATE] Atomic Batch Integrity
        // =========================================================================
        
        let _gate = self.final_gate_lock.lock().await;

        let mut validation_success = true;
        let cycle = self.validation_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mode = if cycle > 0 && cycle % 10 == 0 {
            execution_validator::ValidationMode::Full
        } else {
            execution_validator::ValidationMode::Incremental
        };

        // Validate batch as a whole
        if let Some(rep_task) = batch.tasks.first() {
            if let Some(target) = &rep_task.target_file {
                let project_root = format!("./{}", rep_task.project_id);
                
                // 1. Compile Gate
                if let Err(err) = execution_validator::validate(&project_root, target, mode) {
                    validation_success = false;
                    failures.push(format!("[BATCH_COMPILE_FAIL] {}", err));
                }

                // 2. Selective Run Gate
                if validation_success {
                    let affected_set: std::collections::HashSet<String> = batch.dependency_closure.clone();
                    let run_targets = self.dep_graph.lock().unwrap().run_targets(&affected_set);
                    
                    if let Err(err) = execution_validator::selective_run(&project_root, target, run_targets) {
                        validation_success = false;
                        failures.push(format!("[BATCH_RUN_FAIL] {}", err));
                    }
                }
            }
        }

        if validation_success {
            tracing::info!("🚀 [BATCH_PROMOTION] Batch {} passed all gates. Committing to SSOT.", batch_id);
            if let Some(task) = batch.tasks.first() {
                self.publish_event(axon_core::Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: task.project_id.clone(),
                    thread_id: Some(task.id.clone()),
                    agent_id: None,
                    event_type: axon_core::EventType::SystemLog,
                    source: "FACTORY_ENGINE".to_string(),
                    content: format!("🚀 [BATCH_PROMOTION] Batch {} passed all gates. Committing to SSOT.", batch_id),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
            }
            let mut coord = self.coordinator.lock().unwrap();
            for task in &batch.tasks {
                let mut t = task.clone();
                t.status = axon_core::TaskStatus::Completed;
                if let Some(comment) = senior_comments.get(&t.id) {
                    t.senior_comment = Some(comment.clone());
                    
                    // v0.0.25: Post Senior review to the Thread for UI visibility
                    let _ = self.storage.save_post(&axon_core::Post {
                        id: uuid::Uuid::new_v4().to_string(),
                        thread_id: t.id.clone(),
                        author_id: "SENIOR".to_string(),
                        content: comment.clone(),
                        post_type: axon_core::PostType::Review,
                        thought: None,
                        full_code: None,
                        metrics: None,
                        created_at: chrono::Local::now(),
                    });
                }
                let _ = self.storage.save_task(&t);
                
                // Release locks and notify UI
                coord.complete_task(&t);
                
                if let Ok(Some(mut thread)) = self.storage.get_thread(&t.id) {
                    thread.status = axon_core::ThreadStatus::Completed;
                    let _ = self.storage.save_thread(&thread);
                }
            }
        } else {
            tracing::error!("🛑 [BATCH_GATE_REJECT] Integrity check failed for batch {}.", batch.id);
            for (fname, content) in backups {
                let fpath = std::path::Path::new(&batch.tasks[0].project_id).join(fname);
                let _ = std::fs::write(fpath, content);
            }
            // Spawn reworks for all affected files (Fan-out)
            let affected: Vec<String> = batch.dependency_closure.iter()
                .filter(|id| id.starts_with("file:"))
                .map(|id| id.replace("file:", ""))
                .collect();

            for task in &batch.tasks {
                let _ = self.spawn_rework_task(task, "BATCH_FAIL", &affected).await;
                let mut coord = self.coordinator.lock().unwrap();
                coord.complete_task(task);
            }
        }

        Ok(())
    }

    pub async fn bootstrap_from_spec(&self, spec_path: String) -> anyhow::Result<()> {
        let spec_content = std::fs::read_to_string(&spec_path)?;
        let project_id = std::path::Path::new(&spec_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default-project")
            .to_string();

        // v0.0.25: Ensure Lounge thread exists
        let _ = self.setup_lounge();

        let mut sandbox_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        sandbox_path.push(&project_id);

        let manager = BootstrapManager {
            project_id: project_id.clone(),
            sandbox_root: sandbox_path,
        };

        let daemon = self.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.run_v3(&daemon, spec_content).await {
                tracing::error!("❌ [BOOTSTRAP V3 FAILED]: {}", e);
            }
        });

        Ok(())
    }
}

impl BootstrapManager {
    pub async fn run_v3(&self, daemon: &Daemon, spec_content: String) -> anyhow::Result<()> {
        tracing::info!("🚀 Starting State Machine Pipeline (L-DDP Phase 1.4) for project '{}'...", self.project_id);

        let mut architect_runtime = axon_agent::AgentRuntime::new(
            "architect-agent-001".to_string(),
            axon_core::AgentRole::Architect,
            daemon.architect_model_name.clone(),
            daemon.architect_model.clone()
        ).with_timeout(1200).with_project(self.project_id.clone());
        architect_runtime.set_locale(&daemon.locale);

        let mut stage = Stage::Skeleton;
        let mut ir_opt: Option<axon_core::ir::ProjectIR> = None;
        let mut attempts = 0;
        let max_retries = 5;
        let mut current_hint: Option<String> = None;

        loop {
            tracing::info!("🏭 [FACTORY_STAGE] Currently running: {:?}", stage);
            
            match stage {
                Stage::Skeleton => {
                    tracing::info!("📐 [STAGE:Skeleton] Designing Architecture IR...");
                    let res = architect_runtime.generate_ir(&spec_content, current_hint.clone(), Some(daemon.event_bus.clone())).await;
                    match res {
                        Ok(ir) => {
                            // Validation Gate
                            let mut errors = Vec::new();
                            let is_c = ir.components.values().any(|c| c.file_path.ends_with(".c") || c.file_path.ends_with(".h"));
                            if is_c {
                                if !ir.components.values().any(|c| c.file_path.ends_with(".h")) {
                                    errors.push("Missing .h headers for C project.".to_string());
                                }
                            }

                            if errors.is_empty() {
                                ir_opt = Some(ir);
                                stage = StageRouter::next_stage(&stage);
                                attempts = 0;
                                current_hint = None;
                            } else {
                                let diag = Diagnostic { code: "SKELETON_ERR".into(), message: errors.join(", ") };
                                let cause = infer_cause(&diag);
                                let scope = determine_scope(&cause);
                                tracing::warn!("❌ [SKELETON_FAIL] cause={:?}, scope={:?}", cause, scope);
                                if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in Skeleton stage.")); }
                                stage = StageRouter::route_retry(&scope, &stage);
                                attempts += 1;
                            }
                        },
                        Err(e) => {
                            let diag = Diagnostic { code: "SKELETON_LLM_ERR".into(), message: e.to_string() };
                            let cause = infer_cause(&diag);
                            let scope = determine_scope(&cause);
                            let hint = intelligence::decision::generate_hint(&cause);
                            
                            tracing::warn!("❌ [SKELETON_LLM_FAIL] cause={:?}, scope={:?}, hint={}", cause, scope, hint);
                            
                            if attempts >= max_retries { 
                                tracing::error!("🔥 [SKELETON_CRITICAL] Max retries reached: {}", e);
                                return Err(e); 
                            }
                            
                            current_hint = Some(hint.to_string());
                            stage = StageRouter::route_retry(&scope, &stage);
                            attempts += 1;
                        }
                    }
                },
                Stage::HeaderGen => {
                    tracing::info!("📜 [STAGE:HeaderGen] Decomposing & Materializing Headers...");
                    if let Some(ref ir) = ir_opt {
                        // v0.0.26: Extract Header-only tasks
                        let res = architect_runtime.process_bootstrap_step2(&serde_json::to_string(ir).unwrap(), Some(daemon.event_bus.clone())).await;
                        
                        match res {
                            Ok(post) => {
                                match serde_json::from_str::<Vec<axon_core::Task>>(&post.content) {
                                    Ok(tasks) => {
                                        // Filter for headers (L-DDP Isolation)
                                        let header_tasks: Vec<axon_core::Task> = tasks.into_iter().filter(|t| t.title.contains(".h") || t.description.contains(".h")).collect();
                                        
                                        if header_tasks.is_empty() {
                                            tracing::warn!("⚠️ No header tasks extracted, skipping to ImplGen.");
                                            stage = Stage::ImplGen;
                                        } else {
                                            for mut task in header_tasks {
                                                task.project_id = self.project_id.clone();
                                                let _ = daemon.storage.save_task(&task);
                                            }
                                            stage = StageRouter::next_stage(&stage);
                                        }
                                        attempts = 0;
                                    },
                                    Err(e) => {
                                        let diag = Diagnostic { code: "HEADER_PARSE_ERR".into(), message: e.to_string() };
                                        let cause = infer_cause(&diag);
                                        let scope = determine_scope(&cause);
                                        tracing::warn!("❌ [HEADER_PARSE_FAIL] cause={:?}, scope={:?}, error=\"{}\"", cause, scope, e);
                                        if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in HeaderGen stage.")); }
                                        stage = StageRouter::route_retry(&scope, &stage);
                                        attempts += 1;
                                    }
                                }
                            },
                            Err(e) => {
                                let diag = Diagnostic { code: "HEADER_LLM_ERR".into(), message: e.to_string() };
                                let cause = infer_cause(&diag);
                                let scope = determine_scope(&cause);
                                tracing::warn!("❌ [HEADER_LLM_FAIL] cause={:?}, scope={:?}", cause, scope);
                                if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in HeaderGen stage.")); }
                                stage = StageRouter::route_retry(&scope, &stage);
                                attempts += 1;
                            }
                        }
                    } else {
                        stage = Stage::Skeleton;
                    }
                },
                Stage::ImplGen => {
                    tracing::info!("🔨 [STAGE:ImplGen] Materializing Implementation Tasks...");
                    if let Some(ref ir) = ir_opt {
                        let res = architect_runtime.process_bootstrap_step2(&serde_json::to_string(ir).unwrap(), Some(daemon.event_bus.clone())).await;
                        match res {
                            Ok(post) => {
                                match serde_json::from_str::<Vec<axon_core::Task>>(&post.content) {
                                    Ok(tasks) => {
                                        // Filter for .c/.cpp
                                        for mut task in tasks.into_iter().filter(|t| t.title.contains(".c") || t.title.contains(".cpp")) {
                                            task.project_id = self.project_id.clone();
                                            let _ = daemon.storage.save_task(&task);
                                        }
                                        stage = StageRouter::next_stage(&stage);
                                        attempts = 0;
                                    },
                                    Err(e) => {
                                        let diag = Diagnostic { code: "IMPL_PARSE_ERR".into(), message: e.to_string() };
                                        let cause = infer_cause(&diag);
                                        let scope = determine_scope(&cause);
                                        tracing::warn!("❌ [IMPL_PARSE_FAIL] cause={:?}, scope={:?}, error=\"{}\"", cause, scope, e);
                                        if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in ImplGen stage.")); }
                                        stage = StageRouter::route_retry(&scope, &stage);
                                        attempts += 1;
                                    }
                                }
                            },
                            Err(e) => {
                                let diag = Diagnostic { code: "IMPL_LLM_ERR".into(), message: e.to_string() };
                                let cause = infer_cause(&diag);
                                let scope = determine_scope(&cause);
                                tracing::warn!("❌ [IMPL_LLM_FAIL] cause={:?}, scope={:?}", cause, scope);
                                if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in ImplGen stage.")); }
                                stage = StageRouter::route_retry(&scope, &stage);
                                attempts += 1;
                            }
                        }
                    } else {
                        stage = Stage::Skeleton;
                    }
                },
                Stage::Build => {
                    tracing::info!("📦 [STAGE:Build] Executing CMake Build...");
                    let build_dir = self.sandbox_root.join("build");
                    let _ = std::fs::create_dir_all(&build_dir);

                    // 1. CMake Configure
                    let configure = std::process::Command::new("cmake")
                        .current_dir(&build_dir)
                        .arg("..")
                        .output();

                    match configure {
                        Ok(output) if !output.status.success() => {
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            let diag = Diagnostic { code: "CMAKE_ERROR".into(), message: stderr };
                            let cause = infer_cause(&diag);
                            let scope = determine_scope(&cause);
                            tracing::warn!("❌ [BUILD_FAIL:Configure] cause={:?}, scope={:?}", cause, scope);
                            if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in Build stage.")); }
                            stage = StageRouter::route_retry(&scope, &stage);
                            attempts += 1;
                            continue;
                        },
                        Err(e) => return Err(anyhow::anyhow!("Failed to run cmake: {}", e)),
                        _ => {}
                    }

                    // 2. CMake Build
                    let build = std::process::Command::new("cmake")
                        .current_dir(&build_dir)
                        .args(["--build", "."])
                        .output();

                    match build {
                        Ok(output) if !output.status.success() => {
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            let diag = Diagnostic { code: "BUILD_ERROR".into(), message: stderr };
                            let cause = infer_cause(&diag);
                            let scope = determine_scope(&cause);
                            tracing::warn!("❌ [BUILD_FAIL:Make] cause={:?}, scope={:?}", cause, scope);
                            if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in Build stage.")); }
                            stage = StageRouter::route_retry(&scope, &stage);
                            attempts += 1;
                            continue;
                        },
                        Err(e) => return Err(anyhow::anyhow!("Failed to run build: {}", e)),
                        _ => {
                            tracing::info!("✅ Build successful.");
                            stage = StageRouter::next_stage(&stage);
                            attempts = 0;
                        }
                    }
                },
                Stage::Runtime => {
                    tracing::info!("🏃 [STAGE:Runtime] Executing Binary...");
                    let build_dir = self.sandbox_root.join("build");
                    // Detect binary name (default 'app' from CMakeLists.txt)
                    let bin_path = build_dir.join("app");
                    
                    if !bin_path.exists() {
                        tracing::error!("❌ Binary not found at {:?}", bin_path);
                        stage = Stage::Build;
                        continue;
                    }

                    let run = std::process::Command::new(&bin_path)
                        .current_dir(&build_dir)
                        .output();

                    match run {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            
                            if output.status.success() {
                                tracing::info!("✅ [RUNTIME_SUCCESS]\nStdout: {}", stdout);
                                stage = StageRouter::next_stage(&stage);
                                attempts = 0;
                            } else {
                                let diag = Diagnostic { code: "RUNTIME_ERROR".into(), message: format!("Stdout: {}\nStderr: {}", stdout, stderr) };
                                let cause = infer_cause(&diag);
                                let scope = determine_scope(&cause);
                                tracing::warn!("❌ [RUNTIME_FAIL] cause={:?}, scope={:?}", cause, scope);
                                if attempts >= max_retries { return Err(anyhow::anyhow!("Max retries reached in Runtime stage.")); }
                                stage = StageRouter::route_retry(&scope, &stage);
                                attempts += 1;
                            }
                        },
                        Err(e) => return Err(anyhow::anyhow!("Failed to execute binary: {}", e)),
                    }
                },
                Stage::Sync => {
                    tracing::info!("🔄 [STAGE:Sync] Syncing to Architecture.md & CMakeLists.txt...");
                    if let Some(ref ir) = ir_opt {
                        let arch_md = architect_runtime.generate_architecture_from_ir(ir, Some(daemon.event_bus.clone())).await?;
                        let _ = std::fs::create_dir_all(&self.sandbox_root);
                        std::fs::write(self.sandbox_root.join("architecture.md"), arch_md)?;
                        
                        let mut graph = crate::dep_graph::DepGraph::new();
                        graph.build_from_ir(&serde_json::to_value(&ir).unwrap_or_default());
                        let cmake_content = graph.generate_cmake(&self.project_id);
                        std::fs::write(self.sandbox_root.join("CMakeLists.txt"), cmake_content)?;
                        
                        stage = StageRouter::next_stage(&stage);
                        attempts = 0;
                    } else { stage = Stage::Skeleton; }
                },
                Stage::Complete => {
                    tracing::info!("✅ [STAGE:Complete] Factory Pipeline Finished Successfully.");
                    return Ok(());
                }
            }
        }
    }
    pub async fn run_v2(&self, daemon: &Daemon, spec_content: String) -> anyhow::Result<()> {
        tracing::info!("Starting Bootstrap V2 for project '{}'...", self.project_id);

        let spec_truncated = if spec_content.len() > 8000 {
            format!("{}... [TRUNCATED DUE TO SIZE LIMIT]", &spec_content[..8000])
        } else {
            spec_content.clone()
        };

        let task = axon_core::Task {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            title: format!("Generate Master Hub Architecture for {}", self.project_id),
            description: format!(
                "OBJECTIVE: Generate architecture.md for project '{}'.\n\n\
                 --- SPEC CONTENT ---\n\
                 {}",
                self.project_id,
                spec_truncated
            ),
            status: axon_core::TaskStatus::Pending,
            dependencies: Vec::new(),
            result: None,
            target_file: None,
            lock_files: Vec::new(),
            error_feedback: None,
            senior_comment: None,
            rework_count: 0,
            base_hash: None,
            parent_task: None,
            reason: None,
            kind: "ir".to_string(),
            retries: 0,
            assigned_worker: None,
            created_at: chrono::Local::now(),
        };

        let mut architect_runtime = axon_agent::AgentRuntime::new(
            "architect-agent-001".to_string(),
            axon_core::AgentRole::Architect,
            daemon.architect_model_name.clone(),
            daemon.architect_model.clone()
        ).with_timeout(600);
        architect_runtime.set_locale(&daemon.locale);

        tracing::info!("Stage 1: Deterministic Convergence Loop (Architecture -> Validator -> Repair)");
        let mut error_feedback: Option<String> = None;
        let mut architecture_ready = false;
        let mut clean_arch = String::new();

        for attempt in 1..=5 {
            tracing::info!("🔄 Bootstrap Loop Attempt {}/5...", attempt);
            match architect_runtime.process_bootstrap_step1(&task, error_feedback.clone(), Some(daemon.event_bus.clone())).await {
                Ok(arch_proposal) => {
                    let arch_content = &arch_proposal.content;
                    let current_clean_arch = if let Some(start) = arch_content.find("```markdown") {
                        let remaining = &arch_content[start + 11..];
                        let end = remaining.find("```").unwrap_or(remaining.len());
                        remaining[..end].trim().to_string()
                    } else if let Some(start) = arch_content.find("```") {
                        let remaining = &arch_content[start + 3..];
                        let end = remaining.find("```").unwrap_or(remaining.len());
                        remaining[..end].trim().to_string()
                    } else {
                        arch_content.trim().to_string()
                    };

                    if current_clean_arch.len() < 20 {
                        error_feedback = Some("Architect generated an empty or invalid architecture. Provide more detail.".to_string());
                        continue;
                    }

                    let _ = std::fs::create_dir_all(&self.sandbox_root);
                    let arch_file_path = self.sandbox_root.join("architecture.md");
                    let _ = std::fs::write(&arch_file_path, &current_clean_arch);

                    // VALIDATOR: IR Compilation & Validation
                    let constraints_path = self.sandbox_root.join("constraints.json");
                    let compiler_res = std::process::Command::new("python3")
                        .arg(Daemon::resolve_tool_path("axon_ir_compiler.py"))
                        .arg(&arch_file_path)
                        .arg("--output")
                        .arg(&constraints_path)
                        .output();

                    match compiler_res {
                        Ok(output) if output.status.success() => {
                            tracing::info!("✅ [Attempt {}] Convergence reached. IR Constraints compiled.", attempt);
                            clean_arch = current_clean_arch;
                            architecture_ready = true;
                            let _ = daemon.storage.save_post(&arch_proposal);
                            break;
                        },
                        Ok(output) => {
                            let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                            tracing::warn!("⚠️ [Attempt {}] IR Validation Failed: {}", attempt, err_msg);
                            error_feedback = Some(format!("IR Validation Error (SSOT Violation):\n{}", err_msg));
                        },
                        Err(e) => return Err(anyhow::anyhow!("IR Compiler not found: {}", e)),
                    }
                },
                Err(e) => {
                    tracing::error!("❌ Architect design failed on attempt {}: {}", attempt, e);
                    error_feedback = Some(e.to_string());
                }
            }
        }

        if !architecture_ready {
            return Err(anyhow::anyhow!("Convergence not reached after 5 attempts."));
        }

        // Stage 2: Task Extraction
        tracing::info!("Stage 2: Extracting implementation tasks from converged architecture...");
        match architect_runtime.process_bootstrap_step2(&clean_arch, Some(daemon.event_bus.clone())).await {
            Ok(task_proposal) => {
                let clean_json = self.extract_json(&task_proposal.content);
                let tasks_raw: Vec<serde_json::Value> = serde_json::from_str(&clean_json).unwrap_or_default();

                if !tasks_raw.is_empty() {
                    for t in tasks_raw {
                        let task = axon_core::Task {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: self.project_id.clone(), 
                            title: t["title"].as_str().unwrap_or("Untitled").to_string(),
                            description: t["description"].as_str().unwrap_or("").to_string(),
                            status: axon_core::TaskStatus::Pending,
                            dependencies: Vec::new(),
                            result: None,
                            target_file: t["target_file"].as_str().map(|s| s.to_string()),
                            lock_files: Vec::new(),
                            error_feedback: None,
                            senior_comment: None,
                            rework_count: 0,
                            base_hash: None,
                            parent_task: None,
                            reason: None,
                            kind: "rust".to_string(),
                            retries: 0,
                            assigned_worker: None,
                            created_at: chrono::Local::now(),
                        };
                        let _ = daemon.storage.save_task(&task);
                        let _ = daemon.submit_task(task);
                    }
                    tracing::info!("🚀 Bootstrap complete. AXON Factory is now OPERATIONAL.");
                }
            },
            Err(e) => tracing::error!("Stage 2 extraction failed: {}", e),
        }

        Ok(())
    }

    fn extract_json(&self, content: &str) -> String {
        if let Some(start) = content.find("```json") {
            let end = content[start+7..].find("```").unwrap_or(content.len() - start - 7);
            content[start+7..start+7+end].trim().to_string()
        } else {
            content.trim().to_string()
        }
    }
}

#[allow(dead_code)]
#[allow(dead_code)]
impl Daemon {
    pub fn lock_in_architecture(&self, project_id: &str, thread_title: &str) -> anyhow::Result<()> {
        let mut arch_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        arch_path.push(project_id);
        arch_path.push("architecture.md");
        if std::path::Path::new(&arch_path).exists() {
            let content = std::fs::read_to_string(&arch_path)?;
            let locked_marker = format!("## {} [✅ Locked]", thread_title);
            let target = format!("## {}", thread_title);
            
            if content.contains(&target) && !content.contains(&locked_marker) {
                let new_content = content.replace(&target, &locked_marker);
                std::fs::write(&arch_path, new_content)?;
                tracing::info!("Locked in architecture section: {} at {}", thread_title, arch_path.display());
            }
        }
        Ok(())
    }

    /// v0.0.18: Output Contract & Hardening Order Implementation
    /// Tier 1 (Strict) -> Tier 2 (Relaxed) -> Tier 3 (Heuristic) -> Code Validation -> Atomic Commit
    #[allow(dead_code)]
    fn sync_post_to_sandbox(&self, project_id: &str, content: &str) -> anyhow::Result<()> {
        let mut sandbox_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        sandbox_path.push(project_id);
        let _ = std::fs::create_dir_all(&sandbox_path);

        let mut files_to_commit: Vec<(String, String)> = Vec::new();

        // Tier 1 & 2: Output Contract Extraction
        // Look for [OUTPUT] block
        if let Some(output_start) = content.find("[OUTPUT]") {
            let output_block = if let Some(end) = content[output_start..].find("END_FILE") {
                &content[output_start..output_start+end]
            } else {
                &content[output_start..]
            };

            // Relaxed / Strict: extract FILE: <filename> and code blocks
            let mut current_pos = 0;
            while let Some(file_start) = output_block[current_pos..].find("FILE:") {
                let real_start = current_pos + file_start;
                let line_end = output_block[real_start..].find('\n').unwrap_or(output_block.len() - real_start);
                let filename = output_block[real_start + 5..real_start + line_end].trim().to_string();
                
                if filename.is_empty() || filename.contains("..") {
                    current_pos = real_start + line_end;
                    continue;
                }

                current_pos = real_start + line_end;
                
                if let Some(code_start) = output_block[current_pos..].find("```") {
                    let real_code_start = current_pos + code_start;
                    let lang_end = output_block[real_code_start + 3..].find('\n').unwrap_or(0);
                    let block_content_start = real_code_start + 3 + lang_end + 1;
                    
                    if let Some(code_end) = output_block[block_content_start..].find("```") {
                        let code_content = output_block[block_content_start..block_content_start + code_end].trim().to_string();
                        if !code_content.is_empty() {
                            files_to_commit.push((filename, code_content));
                        }
                        current_pos = block_content_start + code_end + 3;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Tier 3: Heuristic Extraction (Fallback)
        if files_to_commit.is_empty() {
            // v0.0.19: Heuristic parse is NOT allowed for commit
            tracing::error!("❌ Parser Tier 1/2 failed. Heuristic fallback is NOT allowed for commit.");
            return Err(anyhow::anyhow!("Heuristic parse is not allowed for commit"));
        }
                    

        if files_to_commit.is_empty() {
            tracing::warn!("❌ Parser failed completely. No code blocks found.");
            let file_path = sandbox_path.join("README.md");
            let _ = std::fs::write(&file_path, content);
            return Err(anyhow::anyhow!("FORMAT VIOLATION: No valid code blocks or FILE contract found."));
        }

        // v0.0.18: Dependency Intelligence - Filename Inference and Correction
        Self::fix_filenames(&mut files_to_commit);

        // Hardening Phase 1: 3-Way Merge
        let current_map = Self::load_current_files(&sandbox_path.to_string_lossy());
        let snapshot_map = Self::load_snapshot(&sandbox_path.to_string_lossy());
        
        let mut new_map = std::collections::HashMap::new();
        for (filename, code) in files_to_commit {
            new_map.insert(filename, code);
        }

        let merged_map_opt = Self::merge_all(&snapshot_map, &current_map, &new_map);
        if merged_map_opt.is_none() {
            tracing::error!("❌ 3-Way Merge Conflict detected.");
            return Err(anyhow::anyhow!("MERGE_CONFLICT: Manual resolution or LLM Retry required."));
        }
        
        let merged_map = merged_map_opt.unwrap();
        let merged_files: Vec<(String, String)> = merged_map.into_iter().collect();

        // Hardening Phase 2: Atomic Commit with Temp Dir
        let tmp_dir = sandbox_path.join(format!(".tmp_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir)?;

        let mut validated = true;
        for (filename, code) in &merged_files {
            let tmp_file_path = tmp_dir.join(filename);
            if let Some(parent) = tmp_file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&tmp_file_path, code) {
                tracing::error!("❌ Failed to write temp file {}: {}", tmp_file_path.display(), e);
                validated = false;
                break;
            }

            // Hardening Phase 3: Formatting & Code Validation (Python)
            if filename.ends_with(".py") {
                // v0.0.18: Stabilization (Formatter)
            }
        }

        if validated {
            // v0.0.19: Stage 4 --- Execution Harness
            // Prepare file map for harness
            let mut final_entry_point = "main.py".to_string();
            let mut file_map = std::collections::HashMap::new();
            
            for (filename, code) in &merged_files {
                file_map.insert(filename, code);
                let n = filename.to_lowercase();
                if n.ends_with("main.py") || n.ends_with("app.py") {
                    final_entry_point = filename.clone();
                }
            }

            let file_map_json = serde_json::to_string(&file_map).unwrap_or_default();
            let tmp_json_path = sandbox_path.join(format!(".files_{}.json", uuid::Uuid::new_v4()));
            let _ = std::fs::write(&tmp_json_path, file_map_json);

            tracing::info!("🚀 [Stage 4] Launching Execution Harness in Sandbox: {}", sandbox_path.display());
            
            let harness_output = std::process::Command::new("python3")
                .arg(Daemon::resolve_tool_path("axon_execution_harness.py"))
                .arg("--project-root")
                .arg(&sandbox_path)
                .arg("--files-json")
                .arg(&tmp_json_path)
                .arg("--entry")
                .arg(&final_entry_point)
                .output();

            let _ = std::fs::remove_file(&tmp_json_path);

            match harness_output {
                Ok(out) => {
                    if !out.status.success() {
                        let err_msg = String::from_utf8_lossy(&out.stderr).into_owned();
                        tracing::error!("❌ [Stage 4] Execution Harness Failed: {}", err_msg);
                        let _ = std::fs::remove_dir_all(&tmp_dir);
                        return Err(anyhow::anyhow!("RUNTIME_ERROR: {}", err_msg));
                    }
                    tracing::info!("✅ [Stage 4] Execution Harness Passed.");
                    let stdout_msg = String::from_utf8_lossy(&out.stdout);
                    tracing::debug!("Harness Output:\n{}", stdout_msg);
                }
                Err(e) => {
                    tracing::error!("❌ [Stage 4] Failed to execute harness script: {}", e);
                    let _ = std::fs::remove_dir_all(&tmp_dir);
                    return Err(anyhow::anyhow!("HARNESS_EXEC_FAIL: {}", e));
                }
            }

            // v0.0.18: Incremental Diff Commit
            if let Err(e) = Self::apply_diff(&sandbox_path.to_string_lossy(), &merged_files, &tmp_dir.to_string_lossy()) {
                let _ = std::fs::remove_dir_all(&tmp_dir);
                tracing::error!("❌ Incremental Commit failed: {}", e);
                return Err(anyhow::anyhow!("COMMIT_FAILED: Error during incremental diff commit."));
            }

            // v0.0.18: Save Snapshot
            Self::save_snapshot(&sandbox_path.to_string_lossy(), &merged_files);

            let _ = std::fs::remove_dir_all(&tmp_dir);
            Ok(())
        } else {
            // Rollback
            let _ = std::fs::remove_dir_all(&tmp_dir);
            tracing::error!("🔙 Atomic Rollback: Temp directory destroyed due to validation failure.");
            Err(anyhow::anyhow!("CODE INVALID: Failed syntax validation or I/O error."))
        }
    }

    fn extract_imports(code: &str) -> Vec<(String, usize)> {
        let mut imports = Vec::new();
        for line in code.lines() {
            let line = line.trim();
            if line.starts_with("import ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    let imp = parts[1].split('.').next().unwrap_or("");
                    imports.push((imp.to_string(), 0));
                }
            } else if line.starts_with("from ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 3 && parts[2] == "import" {
                    let module_str = parts[1];
                    let mut level = 0;
                    let mut mod_name = module_str;
                    while mod_name.starts_with('.') {
                        level += 1;
                        mod_name = &mod_name[1..];
                    }
                    imports.push((mod_name.to_string(), level));
                }
            }
        }
        imports
    }

    fn resolve_import(module: &str, level: usize, current_path: &str) -> String {
        if level == 0 {
            return format!("{}.py", module.replace(".", "/"));
        }
        let mut parts: Vec<&str> = current_path.split('/').collect();
        parts.pop();
        
        let keep_len = parts.len().saturating_sub(level - 1);
        let mut base: Vec<&str> = parts.into_iter().take(keep_len).collect();
        
        if !module.is_empty() {
            let mod_parts: Vec<&str> = module.split('.').collect();
            base.extend(mod_parts);
        }
        
        format!("{}.py", base.join("/"))
    }

    fn match_module_to_code(module: &str, code: &str) -> bool {
        let cap_module = {
            let mut c = module.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };
        let class_decl = format!("class {}", cap_module);
        let def_decl = format!("def {}", module);
        
        code.contains(&class_decl) || code.contains(&def_decl)
    }

    fn fix_filenames(files: &mut Vec<(String, String)>) {
        let filenames: std::collections::HashSet<String> = files.iter()
            .map(|(n, _)| n.clone())
            .collect();
            
        let mut missing = std::collections::HashSet::new();
        for (name, code) in files.iter() {
            let imports = Self::extract_imports(code);
            for (imp, level) in imports {
                let target = Self::resolve_import(&imp, level, name);
                if !filenames.contains(&target) {
                    let base_mod = target.split('/').last().unwrap_or("").replace(".py", "");
                    if !base_mod.is_empty() {
                        missing.insert(base_mod);
                    }
                }
            }
        }
        
        for (name, code) in files.iter_mut() {
            for mod_name in &missing {
                if Self::match_module_to_code(mod_name, code) {
                    *name = format!("{}.py", mod_name);
                }
            }
        }
    }

    fn _validate_dependencies(files: &[(String, String)]) -> bool {
        let paths: std::collections::HashSet<String> = files.iter()
            .map(|(n, _)| n.clone())
            .collect();

        for (path, code) in files {
            let imports = Self::extract_imports(code);
            for (imp, level) in imports {
                if level > 0 {
                    let target = Self::resolve_import(&imp, level, path);
                    if !paths.contains(&target) {
                        tracing::error!("❌ Invalid relative import: from {} in {}", imp, path);
                        return false;
                    }
                }
            }
        }
        true
    }

    fn _has_entry_point(files: &[(String, String)]) -> bool {
        let mut has_main_file = false;
        let mut has_entry_code = false;
        
        for (name, code) in files {
            let n = name.to_lowercase();
            if n.ends_with("main.py") || n.ends_with("app.py") || n.ends_with("main.rs") {
                has_main_file = true;
            }
            if code.contains("if __name__ == '__main__'") 
                || code.contains("if __name__ == \"__main__\"") 
                || code.contains("fn main()") 
                || code.contains("def main()") 
            {
                has_entry_code = true;
            }
        }
        
        if !has_main_file {
            return true;
        }
        has_entry_code
    }

    fn apply_diff(base_dir: &str, new_files: &[(String, String)], tmp_dir: &str) -> anyhow::Result<()> {
        let mut old_files = std::collections::HashMap::new();
        if let Ok(entries) = std::fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            old_files.insert(name, content);
                        }
                    }
                }
            }
        }

        let mut new_map = std::collections::HashMap::new();
        for (name, code) in new_files {
            new_map.insert(name.clone(), code.clone());
        }

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();

        for (name, code) in &new_map {
            match old_files.get(name) {
                None => added.push(name.clone()),
                Some(old_code) if old_code != code => modified.push(name.clone()),
                _ => {}
            }
        }

        for name in old_files.keys() {
            if !new_map.contains_key(name) {
                deleted.push(name.clone());
            }
        }

        for name in deleted {
            let path = std::path::Path::new(base_dir).join(&name);
            let _ = std::fs::remove_file(&path);
            tracing::info!("🗑️ Deleted old file: {}", path.display());
        }

        for name in added.into_iter().chain(modified.into_iter()) {
            let src_path = std::path::Path::new(tmp_dir).join(&name);
            let dest_path = std::path::Path::new(base_dir).join(&name);
            if let Some(parent) = dest_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(_) = std::fs::rename(&src_path, &dest_path) {
                let _ = std::fs::copy(&src_path, &dest_path);
                let _ = std::fs::remove_file(&src_path);
            }
            tracing::info!("📝 Applied diff to: {}", dest_path.display());
        }

        Ok(())
    }

    fn load_current_files(base_dir: &str) -> std::collections::HashMap<String, String> {
        let mut files = std::collections::HashMap::new();
        if let Ok(entries) = std::fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.starts_with('.') {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                files.insert(name, content);
                            }
                        }
                    }
                }
            }
        }
        files
    }

    fn load_snapshot(base_dir: &str) -> std::collections::HashMap<String, String> {
        let path = std::path::Path::new(base_dir).join(".axon").join("snapshot.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str(&content) {
                return map;
            }
        }
        std::collections::HashMap::new()
    }

    fn save_snapshot(base_dir: &str, files: &[(String, String)]) {
        let path = std::path::Path::new(base_dir).join(".axon").join("snapshot.json");
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let mut map = std::collections::HashMap::new();
        for (name, code) in files {
            map.insert(name.clone(), code.clone());
        }
        
        if let Ok(json) = serde_json::to_string_pretty(&map) {
            let _ = std::fs::write(path, json);
        }
    }

    fn sign_code(code: &str, agent_name: &str, task_id: &str, file_name: &str) -> String {
        format!(
            "// === AXON GENERATED CODE ===\n\
             // Agent: {}\n\
             // Task : {}\n\
             // File : {}\n\
             // ===========================\n\n\
             {}",
            agent_name, task_id, file_name, code
        )
    }

    fn verify_contract(&self, project_id: &str, target_file: &str, code: &str) -> Vec<String> {
        let mut violations = Vec::new();
        let arch_path = std::path::Path::new(project_id).join("architecture.md");
        
        if !arch_path.exists() {
            tracing::warn!("⚠️ [CONTRACT_SKIP] architecture.md not found in project '{}'. skipping contract verification.", project_id);
            return violations;
        }

        let arch_content = match std::fs::read_to_string(&arch_path) {
            Ok(c) => c,
            Err(_) => return violations,
        };

        // Extract JSON block
        let json_start = "<!-- AXON:SPEC:COMPONENTS";
        let json_end = "-->";
        
        let json_str = if let Some(start_idx) = arch_content.find(json_start) {
            let offset = start_idx + json_start.len();
            if let Some(end_idx) = arch_content[offset..].find(json_end) {
                &arch_content[offset..offset + end_idx]
            } else { "" }
        } else { "" };

        if json_str.trim().is_empty() {
            tracing::warn!("⚠️ [CONTRACT_SKIP] No AXON:SPEC:COMPONENTS found in architecture.md");
            return violations;
        }

        let spec: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("🚨 [CONTRACT_ERROR] Failed to parse architecture JSON: {}", e);
                return violations;
            }
        };

        if let Some(components) = spec.get("components").and_then(|c| c.as_array()) {
            for comp in components {
                if comp.get("file").and_then(|f| f.as_str()) == Some(target_file) {
                    if let Some(funcs) = comp.get("functions").and_then(|f| f.as_array()) {
                        for func in funcs {
                            let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("");
                            let signature = func.get("signature").and_then(|s| s.as_str()).unwrap_or("");
                            
                            // 1. Basic Existence Check
                            let pattern_rust = format!("fn {}", name);
                            let pattern_python = format!("def {}", name);
                            
                            if !code.contains(&pattern_rust) && !code.contains(&pattern_python) {
                                violations.push(format!("[F_SIGNATURE_MISMATCH] Missing required function: {}", name));
                                continue;
                            }

                            // 2. Argument Count & Name Check (Naive but effective)
                            // Extract arguments from signature: "calculate_age(year, month, day)" -> ["year", "month", "day"]
                            if let Some(arg_start) = signature.find('(') {
                                if let Some(arg_end) = signature.find(')') {
                                    let args_str = &signature[arg_start + 1..arg_end];
                                    let expected_args: Vec<&str> = if args_str.trim().is_empty() {
                                        Vec::new()
                                    } else {
                                        args_str.split(',').map(|s| s.trim()).collect()
                                    };

                                    // Find the function definition in the code to check its arguments
                                    let lines: Vec<&str> = code.lines().collect();
                                    let mut found_def = false;
                                    for line in lines {
                                        let t = line.trim();
                                        if (t.starts_with("pub fn ") || t.starts_with("fn ") || t.starts_with("def ")) && t.contains(&format!("{}(", name)) {
                                            found_def = true;
                                            // Check argument count by counting commas in the definition line
                                            // (Caveat: Multi-line definitions or complex types might need a better parser)
                                            if let Some(def_start) = t.find('(') {
                                                if let Some(def_end) = t.find(')') {
                                                    let actual_args_str = &t[def_start + 1..def_end];
                                                    let actual_args: Vec<&str> = if actual_args_str.trim().is_empty() {
                                                        Vec::new()
                                                    } else {
                                                        actual_args_str.split(',').map(|s| s.trim()).collect()
                                                    };

                                                    if actual_args.len() != expected_args.len() {
                                                        violations.push(format!("[F_SIGNATURE_MISMATCH] Argument count mismatch for {}. Expected {}, found {}.", name, expected_args.len(), actual_args.len()));
                                                    } else {
                                                        // v0.0.25 [Priority 2]: Strict Type Verification
                                                        let allowed_types = vec![
                                                            "u8", "u16", "u32", "u64", "u128", "usize",
                                                            "i8", "i16", "i32", "i64", "i128", "isize",
                                                            "f32", "f64",
                                                            "String", "&str", "bool",
                                                            "Result", "Option", "Vec", "Self"
                                                        ];

                                                        for arg_def in &actual_args {
                                                            if let Some(colon_idx) = arg_def.find(':') {
                                                                let type_part = arg_def[colon_idx+1..].trim();
                                                                // Extract base type (handle Result<T, E> -> Result)
                                                                let base_type = type_part.split('<').next().unwrap_or(type_part).trim();
                                                                
                                                                if !allowed_types.iter().any(|&at| base_type.contains(at)) {
                                                                    violations.push(format!("[F_SIGNATURE_MISMATCH] Unauthorized or hallucinated type '{}' detected in function {}.", base_type, name));
                                                                }
                                                            }
                                                        }

                                                        // Check if each expected argument name is present in the actual argument
                                                        for expected in &expected_args {
                                                            if !actual_args.iter().any(|a| a.contains(expected)) {
                                                                violations.push(format!("[F_SIGNATURE_MISMATCH] Missing expected argument '{}' in function {}.", expected, name));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            break;
                                        }
                                    }
                                    if !found_def {
                                        violations.push(format!("[F_SIGNATURE_MISMATCH] Could not verify signature for {} (possibly multi-line or complex).", name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        violations
    }

    pub fn verify_ir_completeness_static(base_path: &std::path::Path, spec_content: &str) -> Vec<String> {
        let mut violations = Vec::new();
        let arch_path = base_path.join("architecture.md");
        
        if !arch_path.exists() {
            return violations;
        }

        let arch_content = std::fs::read_to_string(arch_path).unwrap_or_default();

        // 1. Extract Mandatory Nodes from spec.md content (looking for | Node | tables or Node Definitions)
        let mut expected_nodes = std::collections::HashSet::new();
        let mut in_node_table = false;
        for line in spec_content.lines() {
            let t = line.trim();
            if t.contains("| Node |") || t.contains("|Node|") { in_node_table = true; continue; }
            if in_node_table {
                if t.starts_with('|') {
                    let parts: Vec<&str> = t.split('|').map(|s| s.trim()).collect();
                    if parts.len() > 2 {
                        let node_name = parts[1];
                        if !node_name.is_empty() && node_name != "Node" && !node_name.starts_with('-') {
                            let upper = node_name.to_uppercase();
                            if upper != "START" && upper != "END" && upper != "BYPASS" && upper != "OUTPUT" && !upper.contains('/') {
                                expected_nodes.insert(upper);
                            }
                        }
                    }
                } else if !t.is_empty() {
                    in_node_table = false;
                }
            }
        }

        if expected_nodes.is_empty() {
            // Fallback: look for lines starting with "- Node:" or similar
            for line in spec_content.lines() {
                if line.trim().to_uppercase().starts_with("- NODE:") {
                    let node_name = line.split(':').nth(1).unwrap_or("").trim().split_whitespace().next().unwrap_or("").to_uppercase();
                    if !node_name.is_empty() { expected_nodes.insert(node_name); }
                }
            }
        }

        // 2. Extract Materialized Components/Functions from architecture.md
        let arch_upper = arch_content.to_uppercase();
        let mut missing_nodes = Vec::new();

        for node in expected_nodes {
            // Check if node name exists in architecture.md (as component, function, or in JSON)
            // We use a flexible check because DB_CHECK might become query_db in the architecture.
            // But usually the name or a strong derivative should exist.
            if !arch_upper.contains(&node) {
                // Special mappings
                let mut found_alias = false;
                if node == "DB_CHECK" && (arch_upper.contains("QUERY_DB") || arch_upper.contains("LOAD_FROM_DB") || arch_upper.contains("PERSISTENCE")) { found_alias = true; }
                if node == "SAVE_DB" && (arch_upper.contains("PERSISTENCE") || arch_upper.contains("SAVE_TO_DB")) { found_alias = true; }
                if node == "CALCULATE" && arch_upper.contains("COMPUTE") { found_alias = true; }
                
                if !found_alias {
                    missing_nodes.push(node);
                }
            }
        }

        if !missing_nodes.is_empty() {
            violations.push(format!("[F_IR_INCOMPLETE] The following nodes from spec.md were LOST during architecture materialization: {}", missing_nodes.join(", ")));
        }

        violations
    }

    /// v0.0.25: Check IR for completeness (stub detection)
    fn check_ir_completeness(ir: &axon_core::ir::ProjectIR, spec_content: &str) -> Vec<String> {
        // Simplified check first
        if ir.components.is_empty() {
             return vec!["IR contains no components.".to_string()];
        }
        let mut errors = Vec::new();
        
        // 1. Extract Mandatory Nodes from spec.md content
        let mut expected_nodes = std::collections::HashSet::new();
        let mut in_node_table = false;
        for line in spec_content.lines() {
            let t = line.trim();
            if t.contains("| Node |") || t.contains("|Node|") { in_node_table = true; continue; }
            if in_node_table {
                if t.starts_with('|') {
                    let parts: Vec<&str> = t.split('|').map(|s| s.trim()).collect();
                    if parts.len() > 2 {
                        let node_name = parts[1];
                        if !node_name.is_empty() && node_name != "Node" && !node_name.starts_with('-') {
                            let upper = node_name.to_uppercase();
                            if upper != "START" && upper != "END" && upper != "BYPASS" && upper != "OUTPUT" && !upper.contains('/') {
                                expected_nodes.insert(upper);
                            }
                        }
                    }
                } else if !t.is_empty() {
                    in_node_table = false;
                }
            }
        }

        if expected_nodes.is_empty() {
            for line in spec_content.lines() {
                if line.trim().to_uppercase().starts_with("- NODE:") {
                    let node_name = line.split(':').nth(1).unwrap_or("").trim().split_whitespace().next().unwrap_or("").to_uppercase();
                    if !node_name.is_empty() { expected_nodes.insert(node_name); }
                }
            }
        }

        // 2. Check if all expected nodes are represented in IR components or functions
        let mut materialized_logic = std::collections::HashSet::new();
        for comp in ir.components.values() {
            materialized_logic.insert(comp.name.to_uppercase());
            for func in &comp.functions {
                materialized_logic.insert(func.1.name.to_uppercase());
            }
        }

        for node in expected_nodes {
            let target_node = ir.node_mapping.get(&node).unwrap_or(&node).to_uppercase();
            let mut found = false;
            for mat in &materialized_logic {
                if mat.contains(&node) || node.contains(mat) || mat.contains(&target_node) || target_node.contains(mat) {
                    found = true;
                    break;
                }
            }
            if !found {
                // Aliases
                if node == "DB_CHECK" && (materialized_logic.contains("QUERY_DB") || materialized_logic.contains("LOAD_FROM_DB")) { found = true; }
                if node == "SAVE_DB" && (materialized_logic.contains("SAVE_TO_DB") || materialized_logic.contains("PERSISTENCE")) { found = true; }
                if node == "CALCULATE" && (materialized_logic.contains("COMPUTE") || materialized_logic.contains("CALCULATE_AGE")) { found = true; }
                
                if !found {
                    errors.push(format!("[F_IR_INCOMPLETE] Required Node '{}' is missing from IR.", node));
                }
            }
        }

        errors
    }

    fn extract_required_functions(description: &str) -> Vec<String> {
        let mut functions = Vec::new();
        let mut in_functions_block = false;
        
        for line in description.lines() {
            let trimmed = line.trim();
            if trimmed.to_uppercase().starts_with("FUNCTIONS:") {
                in_functions_block = true;
                continue;
            }
            if in_functions_block {
                if trimmed.is_empty() { continue; }
                if trimmed.starts_with('-') {
                    let func = trimmed[1..].trim();
                    if !func.is_empty() {
                        functions.push(func.to_string());
                    }
                } else if trimmed.starts_with('#') || (trimmed.contains(':') && !trimmed.starts_with('-')) {
                    // Stop if we hit another header or section
                    break;
                }
            }
        }
        functions
    }

    fn extract_functions(code: &str) -> std::collections::HashMap<String, (usize, usize, String)> {
        let mut funcs = std::collections::HashMap::new();
        let lines: Vec<&str> = code.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let start_idx = i;
            
            // 1. Decorator Handling
            let mut has_decorator = false;
            while i < lines.len() && lines[i].trim().starts_with('@') {
                has_decorator = true;
                i += 1;
            }
            if i >= lines.len() { break; }
            
            let line = lines[i];
            let trimmed = line.trim();
            
            // 2. Function / Class Detection
            if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                let is_class = trimmed.starts_with("class ");
                let prefix_len = if is_class { 6 } else { 4 };
                
                if let Some(name_part) = trimmed[prefix_len..].split('(').next() {
                    let name = name_part.split(':').next().unwrap_or("").trim().to_string();
                    let base_indent = line.len() - line.trim_start().len();
                    
                    i += 1; // Move past the def/class line
                    
                    // 3. Indent Check (Find End)
                    while i < lines.len() {
                        let l = lines[i];
                        let t = l.trim();
                        if t.is_empty() || t.starts_with('#') {
                            i += 1;
                            continue;
                        }
                        let current_indent = l.len() - l.trim_start().len();
                        if current_indent <= base_indent {
                            break;
                        }
                        i += 1;
                    }
                    
                    // 4. Trailing Comment Exclusion
                    let mut end_idx = i;
                    while end_idx > start_idx {
                        let prev_line = lines[end_idx - 1].trim();
                        if prev_line.is_empty() || prev_line.starts_with('#') {
                            end_idx -= 1;
                        } else {
                            break;
                        }
                    }
                    
                    let body = lines[start_idx..end_idx].join("\n");
                    let key = if is_class { format!("class:{}", name) } else { name };
                    funcs.insert(key, (start_idx, end_idx, body));
                    
                    // Continue from end_idx to avoid inner nested overlaps
                    i = end_idx;
                    continue;
                }
            } else if has_decorator {
                // False alarm (e.g. commented out decorator or invalid syntax)
                i += 1;
            } else {
                i += 1;
            }
        }
        funcs
    }

    fn merge_semantic(base: &str, current: &str, new: &str) -> Option<String> {
        // If the task didn't touch this file, preserve the current state.
        if new.is_empty() { return Some(current.to_string()); }
        
        if current == base { return Some(new.to_string()); }
        if new == base { return Some(current.to_string()); }
        if current == new { return Some(current.to_string()); }

        let base_funcs = Self::extract_functions(base);
        let current_funcs = Self::extract_functions(current);
        let new_funcs = Self::extract_functions(new);

        let mut merged_code = current.to_string();
        
        for (name, (_, _, new_body)) in &new_funcs {
            let base_body = base_funcs.get(name).map(|(_, _, b)| b.as_str()).unwrap_or("");
            let current_body = current_funcs.get(name).map(|(_, _, b)| b.as_str()).unwrap_or("");
            
            if new_body == base_body {
                continue;
            }
            
            if current_body != base_body && current_body != new_body {
                return None; // CONFLICT at function level
            }
            
            if let Some((_, _, c_body)) = current_funcs.get(name) {
                merged_code = merged_code.replace(c_body, new_body);
            } else {
                merged_code.push_str("\n\n");
                merged_code.push_str(new_body);
            }
        }
        
        Some(merged_code)
    }

    fn merge_all(
        base_map: &std::collections::HashMap<String, String>,
        current_map: &std::collections::HashMap<String, String>,
        new_map: &std::collections::HashMap<String, String>
    ) -> Option<std::collections::HashMap<String, String>> {
        let mut merged = std::collections::HashMap::new();
        let mut all_files = std::collections::HashSet::new();

        for k in base_map.keys() { all_files.insert(k.clone()); }
        for k in current_map.keys() { all_files.insert(k.clone()); }
        for k in new_map.keys() { all_files.insert(k.clone()); }

        for f in all_files {
            let base = base_map.get(&f).map(|s| s.as_str()).unwrap_or("");
            let current = current_map.get(&f).map(|s| s.as_str()).unwrap_or("");
            let new = new_map.get(&f).map(|s| s.as_str()).unwrap_or("");

            if let Some(result) = Self::merge_semantic(base, current, new) {
                if !result.is_empty() || new_map.contains_key(&f) || current_map.contains_key(&f) {
                    merged.insert(f, result);
                }
            } else {
                return None; // CONFLICT
            }
        }
        Some(merged)
    }
    
    fn select_best_agent(&self, role: axon_core::AgentRole, kind: &str) -> (Arc<dyn axon_model::ModelDriver + Send + Sync>, String, String) {
        let (models, names) = match role {
            axon_core::AgentRole::Junior => (&self.junior_models, &self.junior_model_names),
            axon_core::AgentRole::Senior => (&self.senior_models, &self.senior_model_names),
            axon_core::AgentRole::Architect => return (self.architect_model.clone(), format!("architect-agent-1({})", self.architect_model_name), self.architect_model_name.clone()),
        };

        if models.is_empty() {
            return (self.architect_model.clone(), "unknown".to_string(), self.architect_model_name.clone());
        }

        // v0.0.25: [ALR] Step 2 - Adaptive Routing based on DB Stats (Refined Formula)
        let db_stats = self.storage.get_worker_stats().unwrap_or_default();
        
        let mut best_idx = 0;
        let mut best_score = -100.0; 

        for i in 0..models.len() {
            let id = format!("{}-agent-{}({})", match role {
                axon_core::AgentRole::Junior => "junior",
                axon_core::AgentRole::Senior => "senior",
                _ => "agent"
            }, i + 1, &names[i]);

            let score = if let Some((success_rate, avg_retries, samples, specialization)) = db_stats.get(&id) {
                if *samples < 5 {
                    // Cold start fallback: random exploration
                    use rand::Rng;
                    rand::thread_rng().gen_range(0.0..1.0)
                } else {
                    let skill = specialization.get(kind).cloned().unwrap_or(0.5);
                    let success = *success_rate as f32;
                    let retries = (*avg_retries as f32 / 3.0).min(1.0);
                    
                    // User Formula: skill * 0.5 + success * 0.4 - retries * 0.1
                    (skill * 0.5 + success * 0.4 - retries * 0.1) as f64
                }
            } else {
                0.5 // Default score
            };

            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        (models[best_idx].clone(), format!("{}-agent-{}({})", match role {
            axon_core::AgentRole::Junior => "junior",
            axon_core::AgentRole::Senior => "senior",
            _ => "agent"
        }, best_idx + 1, &names[best_idx]), names[best_idx].clone())
    }
    
    async fn abort_with_failure(&self, task: &mut axon_core::Task, failures: Vec<String>, path: Vec<(String, String)>, metrics_list: Vec<axon_core::RuntimeMetrics>, agent_metrics: Vec<axon_core::AgentMetric>, start_total: std::time::Instant, worker_id: usize) -> anyhow::Result<()> {
        // v0.0.25: [ALR] Capture Failure Metrics (Step 1)
        if let Some(w_id) = &task.assigned_worker {
            let _ = self.storage.update_worker_stats(w_id, false, task.rework_count, &task.kind);
        }
        
        // v0.0.25: [ALR] Update Hotspot Priority and Release Queue lock
        {
            let mut coord = self.coordinator.lock().unwrap();
            if let Some(target) = &task.target_file {
                coord.update_priority(target, false, true, 0);
            }
            coord.complete_task(&task);
        }

        task.status = axon_core::TaskStatus::Failed;
        let failure_reason = failures.join("\n");
        task.result = Some(failure_reason.clone());
        task.error_feedback = Some(failure_reason);
        let _ = self.storage.save_task(task);
        
        // v0.0.25: Release all locks on failure
        let _ = self.storage.release_all_locks_for_task(&task.id);

        // v0.0.23: Reset Work Board UI on failure
        if let Ok(Some(mut thread)) = self.storage.get_thread(&task.id) {
            thread.status = axon_core::ThreadStatus::Draft;
            let _ = self.storage.save_thread(&thread);
            
            // v0.0.23: Record failure reason as a Post for UI visibility
            let failure_msg = format!("### ❌ [PIPELINE_FAILED]\n\n{}", failures.join("\n"));
            let _ = self.storage.save_post(&axon_core::Post {
                id: uuid::Uuid::new_v4().to_string(),
                thread_id: task.id.clone(),
                author_id: "system-harness".to_string(),
                content: failure_msg,
                thought: None,
                full_code: None,
                post_type: axon_core::PostType::System,
                metrics: None,
                created_at: chrono::Local::now(),
            });

            // v0.0.23: Auto-Requeue (Self-Correction Loop)
            // Put the task back into the dispatcher so a worker can try again with the new feedback
            task.status = axon_core::TaskStatus::Pending; // Set back to pending for retry
            let _ = self.submit_task(task.clone());
            tracing::info!("♻️ [AUTO_REQUEUE] Task {} sent back to dispatcher for self-correction retry.", task.id);
        }

        let last_metrics = metrics_list.last().cloned().unwrap_or_default();
        let total_duration_ms = start_total.elapsed().as_secs_f64() * 1000.0;

        let report = axon_core::ObservabilityReport {
            agents: agent_metrics,
            execution_path: path,
            metrics: last_metrics,
            summary: axon_core::ExecutionSummary {
                worker_id,
                total_duration_ms,
                steps: metrics_list.len(),
                status: "FAILED".to_string(),
            },
            queue: axon_core::QueueStatus {
                length: self.dispatcher.len(),
                limit: self.dispatcher.limit(),
            },
            failures: failures.clone(),
        };

        self.publish_event(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: task.project_id.clone(),
            thread_id: Some(task.id.clone()),
            agent_id: None,
            event_type: axon_core::EventType::SystemLog,
            source: "pipeline_failure".to_string(),
            content: serde_json::to_string(&report).unwrap_or_default(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        tracing::error!("❌ [PIPELINE_FAILED] Task {}: {:?}", task.id, failures);
        Ok(())
    }

    pub async fn execute_junior_task(&self, task: &axon_core::Task) -> anyhow::Result<(String, axon_core::RuntimeMetrics)> {
        // v0.0.26: Stage 2 - Context Intelligence (Header-first)
        let mut existing_code = String::new();
        
        // 1. Gather code from dependencies (e.g., .h contents for a .c task)
        for dep_name in &task.dependencies {
            match self.storage.get_task_by_title(&task.project_id, dep_name) {
                Ok(Some(dep_task)) => {
                    if let Some(fname) = &dep_task.target_file {
                        let fpath = std::path::Path::new(&task.project_id).join(fname);
                        if let Ok(content) = std::fs::read_to_string(&fpath) {
                            existing_code.push_str(&format!("### DEPENDENCY: {} ({}) ###\n{}\n\n", dep_name, fname, content));
                        }
                    }
                },
                _ => tracing::warn!("⚠️ [CONTEXT_FAIL] Could not find dependency task {} for task {}", dep_name, task.id),
            }
        }

        // 2. Initialize Junior Runtime (v0.0.26 Pattern)
        let junior_name = task.assigned_worker.as_deref().unwrap_or("junior-agent-001");
        let mut junior_runtime = axon_agent::AgentRuntime::new(
            junior_name.to_string(),
            axon_core::AgentRole::Junior,
            self.junior_model_names[0].clone(),
            self.junior_models[0].clone()
        ).with_project(task.project_id.clone());
        junior_runtime.set_locale(&self.locale);

        // 3. Execute implementation through axon-agent (Enforcing Stage 1 Policy)
        let post = junior_runtime.run_implementation_task(
            task,
            self.event_bus.clone(),
            &task.kind, // lang_name
            "",         // lang_instruction (legacy, handled by system prompt)
            &self.architecture_guide,
            &existing_code
        ).await?;

        let full_code = post.full_code.unwrap_or_default();
        let thought_opt = post.thought;
        let parsed_success = !full_code.is_empty();
        let metrics = post.metrics.unwrap_or(axon_core::RuntimeMetrics { total_duration: Some(0), eval_count: Some(0), eval_duration: Some(0) });

        // v0.0.25: Post the Junior's thought to the Lounge BEFORE parser checks
        // Even if they failed to write valid code, their attempt/excuse should be logged!
        if let Some(ref thought) = thought_opt {
            let _ = self.storage.save_post(&axon_core::Post {
                id: uuid::Uuid::new_v4().to_string(),
                thread_id: "lounge".to_string(),
                author_id: format!("JUNIOR-{}", task.assigned_worker.as_deref().unwrap_or("unknown")),
                content: format!("**[Task: {}]**\n{}", task.title, thought),
                post_type: axon_core::PostType::Nogari,
                thought: None,
                full_code: None,
                metrics: None,
                created_at: chrono::Local::now(),
            });

            // Broadcast to Lounge UI
            let event = axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: task.project_id.clone(), // v0.0.25: Use actual project_id for visibility
                thread_id: Some("lounge".to_string()),
                agent_id: task.assigned_worker.clone(),
                event_type: axon_core::EventType::MessagePosted,
                source: format!("JUNIOR-{}", task.assigned_worker.as_deref().unwrap_or("unknown")),
                content: format!("💬 {}: {}", task.assigned_worker.as_deref().unwrap_or("Junior"), thought),
                payload: None,
                timestamp: chrono::Local::now(),
            };
            self.publish_event(event);
        }

        if !parsed_success {
            tracing::error!("❌ [PARSER_FAIL] Junior produced a response but it could not be parsed into AXON Patch V2.");
            anyhow::bail!("Code Extraction Failed: Junior response did not follow AXON Patch Protocol V2.");
        }

        // v0.0.25: ALWAYS post to the specific task thread for context, even if no thought
        let thought_text = thought_opt.as_deref().unwrap_or("Implementation ready.");
        let content_with_code = format!("{}\n\n```{}\n{}\n```", thought_text, task.kind, full_code);
        let _ = self.storage.save_post(&axon_core::Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: format!("JUNIOR-{}", task.assigned_worker.as_deref().unwrap_or("unknown")),
            content: content_with_code,
            post_type: axon_core::PostType::Proposal,
            thought: thought_opt,
            full_code: Some(full_code.clone()),
            metrics: Some(metrics.clone()),
            created_at: chrono::Local::now(),
        });
        
        Ok((full_code, metrics))
    }

    pub async fn verify_with_senior(&self, task: &axon_core::Task, patch: &str) -> anyhow::Result<String> {
        let senior = self.senior_models[0].clone();
        let prompt = format!(
            "REVIEW THIS IMPLEMENTATION:\n\n\
             Task: {}\n\n\
             Code:\n{}\n\n\
             If the code is acceptable and implements the logic correctly, you MUST reply with 'APPROVE' followed by a short encouraging comment or review.\n\
             If there are critical bugs or it fails to follow the task, explain why and DO NOT say 'APPROVE'.",
            task.description, patch
        );
        
        let resp = senior.generate(prompt).await.map_err(|e| anyhow::anyhow!(e))?;
        let resp_upper = resp.text.to_uppercase();
        if resp_upper.contains("APPROVE") || resp_upper.contains("OK") || resp_upper.contains("LOOKS GOOD") {
            Ok(resp.text)
        } else {
            anyhow::bail!("Senior rejected: {}", resp.text)
        }
    }


    /// v0.0.25: Phase 0 - Function Signature Extraction (Rust/Python)
    fn extract_actual_functions(content: &str) -> Vec<String> {
        let mut functions = Vec::new();
        // Regex for Rust: pub fn name, fn name
        // Regex for Python: def name
        let re_rust = regex::Regex::new(r"(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        let re_py = regex::Regex::new(r"def\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        for cap in re_rust.captures_iter(content) {
            functions.push(cap[1].to_string());
        }
        for cap in re_py.captures_iter(content) {
            functions.push(cap[1].to_string());
        }
        functions
    }

    /// v0.0.25: Dependency-Aware Impact Analysis
    pub async fn analyze_impact_and_schedule_rework(&self, project_id: &str, changed_file: &str) -> anyhow::Result<Vec<String>> {
        let mut graph = dep_graph::DepGraph::new();
        
        // 1. Build from Architecture IR
        let arch_path = std::path::Path::new(project_id).join("architecture.md");
        if let Ok(arch_content) = std::fs::read_to_string(&arch_path) {
            let json_start = "<!-- AXON:SPEC:COMPONENTS";
            let json_end = "-->";
            if let Some(start_idx) = arch_content.find(json_start) {
                let offset = start_idx + json_start.len();
                if let Some(end_idx) = arch_content[offset..].find(json_end) {
                    let json_str = &arch_content[offset..offset + end_idx];
                    if let Ok(spec) = serde_json::from_str::<serde_json::Value>(json_str) {
                        let mut graph = self.dep_graph.lock().unwrap();
                        graph.build_from_ir(&spec);
                    }
                }
            }
        }

        // 2. Enrich from existing files
        if let Ok(entries) = std::fs::read_dir(project_id) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(code) = std::fs::read_to_string(&path) {
                        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        graph.enrich_from_code(fname, &code);
                    }
                }
            }
        }

        // 3. Compute Impact
        let changed_node = format!("file:{}", changed_file);
        let impact_set = graph.compute_impact(vec![changed_node]);
        
        let mut scheduled_files = Vec::new();
        for node_id in impact_set {
            if node_id.starts_with("file:") && !node_id.contains(changed_file) {
                let impacted_file = node_id.replace("file:", "");
                
                // Create a Rework Task
                let rework_task = axon_core::Task {
                    id: format!("rework-{}-{}", impacted_file.replace(".", "-"), uuid::Uuid::new_v4().to_string()[..8].to_string()),
                    project_id: project_id.to_string(),
                    title: format!("Dependency Rework: {}", impacted_file),
                    description: format!("Upstream dependency '{}' changed. Verify and update this file to maintain consistency.", changed_file),
                    status: axon_core::TaskStatus::Pending,
                    dependencies: Vec::new(),
                    result: None,
                    target_file: Some(impacted_file.clone()),
                    lock_files: vec![impacted_file.clone(), changed_file.to_string()],
                    error_feedback: None,
                    senior_comment: None,
                    rework_count: 1,
                    base_hash: None,
                    parent_task: None,
                    reason: Some(format!("Dependency '{}' changed", changed_file)),
                    kind: "rust".to_string(),
                    retries: 0,
                    assigned_worker: None,
                    created_at: chrono::Local::now(),
                };
                
                let _ = self.submit_task(rework_task);
                scheduled_files.push(impacted_file);
            }
        }

        Ok(scheduled_files)
    }

    async fn analyze_conflicts_and_propose_rules(&self, _task_id: &str, file_path: &str) -> anyhow::Result<()> {
        // v0.0.25: [ALR] Step 3 - Rule Candidate Generation (Sandbox)
        let _db_stats = self.storage.get_worker_stats().unwrap_or_default(); // Not needed here, but for future logic
        
        // Count recent conflicts for this file/pattern
        let conn = self.storage.conn.lock().unwrap();
        let occurrences: i64 = conn.query_row(
            "SELECT COUNT(*) FROM conflict_events WHERE file_path = ?1 AND created_at > datetime('now', '-24 hours')",
            params![file_path],
            |r| r.get(0)
        ).unwrap_or(0);

        if occurrences >= 3 {
            tracing::warn!("🔥 [RULE_PROPOSAL] Recurring conflict detected for {}. Proposing rule candidate.", file_path);
            let rule_id = format!("rule-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
            let pattern = format!("Recurring conflict on {}", file_path);
            let fix = "Enforce strict dependency sync or increase lease duration".to_string();
            
            let _ = conn.execute(
                "INSERT OR IGNORE INTO rule_candidates (id, pattern, fix_strategy, confidence, occurrences, state, created_at)
                 VALUES (?1, ?2, ?3, 0.5, ?4, 'Candidate', ?5)",
                params![rule_id, pattern, fix, occurrences, chrono::Local::now().to_rfc3339()],
            );
        }
        Ok(())
    }

    async fn spawn_rework_task(&self, original_task: &axon_core::Task, reason: &str, lock_files: &Vec<String>) -> anyhow::Result<()> {
        if original_task.rework_count >= 3 {
            tracing::error!("🛑 [REWORK_LIMIT] Task {} reached max retries. Escalating to human.", original_task.id);
            return Ok(());
        }

        // v0.0.25: [ALR] Step 4 - Rework Propagation (Dependency Expansion)
        let mut impacted_ids = std::collections::HashSet::new();
        for f in lock_files {
            let file_id = format!("file:{}", f);
            impacted_ids.insert(file_id.clone());
            let dependents = self.dep_graph.lock().unwrap().compute_impact(vec![file_id]);
            for d in dependents {
                impacted_ids.insert(d);
            }
        }
        
        // Convert node IDs (file:...) back to file paths
        let final_lock_set: Vec<String> = impacted_ids.into_iter()
            .filter(|id| id.starts_with("file:"))
            .map(|id| id.replace("file:", ""))
            .collect();

        tracing::info!("♻️ [REWORK_EXPANSION] Expanding rework for {} due to {}. Impacted: {:?}", original_task.id, reason, final_lock_set);

        let _rework_id = format!("rework-{}-{}", original_task.id, uuid::Uuid::new_v4().to_string().chars().take(4).collect::<String>());

        let mut rework_task = original_task.clone();
        rework_task.id = format!("rework-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        rework_task.status = axon_core::TaskStatus::Ready;
        rework_task.parent_task = Some(original_task.id.clone());
        rework_task.reason = Some(reason.to_string());
        rework_task.rework_count += 1;
        rework_task.lock_files = lock_files.clone();
        rework_task.description = format!("{} (Original failed due to: {})", original_task.description, reason);
        rework_task.created_at = chrono::Local::now();

        // v0.0.25: [C2R] Explicit instruction for LLM
        let rework_instruction = format!("\n\n[REWORK CONTEXT]\nPrevious attempt failed due to {}. You MUST use the latest version of all affected files and perform a FULL rewrite if necessary to ensure consistency.", reason);
        rework_task.description.push_str(&rework_instruction);

        let _ = self.storage.save_task(&rework_task);
        
        tracing::info!("🔄 [C2R_SPAWNED] Rework task {} created for original {}", rework_task.id, original_task.id);

        // v0.0.25: [ALR] Update Hotspot Priority for Rework
        {
            let mut coord = self.coordinator.lock().unwrap();
            if let Some(target) = &original_task.target_file {
                coord.update_priority(target, true, false, 0);
            }
            coord.add_task(rework_task);
        }

        Ok(())
    }

    fn compute_hash_map(&self, project_id: &str, files: &Vec<String>) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        for f in files {
            let path = std::path::Path::new(project_id).join(f);
            if let Some(h) = Self::calculate_file_hash(&path) {
                map.insert(f.clone(), h);
            }
        }
        map
    }

    /// v0.0.25: Version Gate - Calculate file hash for optimistic concurrency control
    fn calculate_file_hash(path: &std::path::Path) -> Option<String> {
        if let Ok(content) = std::fs::read(&path) {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(content);
            Some(hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect())
        } else {
            None
        }
    }

    pub fn setup_lounge(&self) -> anyhow::Result<()> {
        let lounge_thread = axon_core::Thread {
            id: "lounge".to_string(),
            project_id: "system".to_string(),
            title: "Lounge (#nogari)".to_string(),
            status: axon_core::ThreadStatus::Working,
            author: "SYSTEM".to_string(),
            milestone_id: None,
            created_at: chrono::Local::now(),
            updated_at: chrono::Local::now(),
        };
        let _ = self.storage.save_thread(&lounge_thread);
        Ok(())
    }
}

pub async fn validate_agent(driver: &dyn axon_model::ModelDriver, model_name: &str) -> String {
    if let Ok(models) = driver.list_available_models().await {
        // Partial match for flexibility (e.g., "gemini-2.5-flash" vs "models/gemini-2.5-flash")
        if models.iter().any(|m| m.contains(model_name) || model_name.contains(m)) {
            return "OK".to_string();
        }
    }
    "FAIL".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_functions() {
        let code = r#"
import os

@cache
def my_func():
    return 1

# comment
def my_func2():
    pass
# trailing
"#;
        let funcs = Daemon::extract_functions(code);
        assert!(funcs.contains_key("my_func"));
        assert!(funcs.contains_key("my_func2"));
        let (_, _, b) = funcs.get("my_func").unwrap();
        assert!(b.contains("@cache"));
        assert!(!b.contains("# comment"));
        
        let (_, _, b2) = funcs.get("my_func2").unwrap();
        assert!(!b2.contains("# trailing"));
    }

    #[test]
    fn test_resolve_import() {
        let path = "src/services/user.py";
        assert_eq!(Daemon::resolve_import("database", 1, path), "src/services/database.py");
        assert_eq!(Daemon::resolve_import("utils", 2, path), "src/utils.py");
        assert_eq!(Daemon::resolve_import("os", 0, path), "os.py");
    }

    #[test]
    fn test_merge_semantic() {
        let base = r#"
def func1():
    pass

def func2():
    pass
"#;
        let current = r#"
def func1():
    # user comment
    pass

def func2():
    pass
"#;
        let new_code = r#"
def func1():
    pass

def func2():
    return 42
"#;
        let merged = Daemon::merge_semantic(base, current, new_code).unwrap();
        assert!(merged.contains("# user comment"));
        assert!(merged.contains("return 42"));
    }

    // =========================================================================
    // v0.0.25: Phase 0 - Data Integrity Tests (No LLM required)
    // =========================================================================

    /// Simulates the destructive rewrite guard logic for a given old/new content pair.
    /// Returns a list of violations found.
    fn simulate_destructive_rewrite_check(old: &str, new: &str) -> Vec<String> {
        let old_funcs = Daemon::extract_actual_functions(old);
        let new_funcs = Daemon::extract_actual_functions(new);
        let mut violations = Vec::new();
        for f in &old_funcs {
            if !new_funcs.contains(f) {
                violations.push(format!(
                    "Destructive Rewrite Violation: You deleted an existing function '{}'.",
                    f
                ));
            }
        }
        violations
    }

    /// TC1: Partial rewrite attack (LLM returns only get_year)
    /// Expected: REJECT — Destructive rewrite detected
    #[test]
    fn tc1_partial_rewrite_must_reject() {
        let old_file = r#"
pub fn get_year() -> i32 { 2023 }
pub fn get_name() -> String { "Original".to_string() }
pub fn get_month() -> u8 { 1 }
pub fn get_day() -> u8 { 1 }
"#;
        // LLM only returned get_year
        let new_file = r#"
pub fn get_year() -> i32 { 2025 }
"#;

        let violations = simulate_destructive_rewrite_check(old_file, new_file);

        println!("[TC1] Violations: {:?}", violations);

        assert!(!violations.is_empty(), "TC1 FAILED: Partial rewrite was not rejected!");
        assert!(violations.iter().any(|v| v.contains("get_name")), "TC1: get_name not flagged");
        assert!(violations.iter().any(|v| v.contains("get_month")), "TC1: get_month not flagged");
        assert!(violations.iter().any(|v| v.contains("get_day")), "TC1: get_day not flagged");
        println!("✅ TC1 PASSED: Partial rewrite correctly REJECTED.");
    }

    /// TC2: Normal full rewrite (LLM returns all functions)
    /// Expected: PASS — commit allowed
    #[test]
    fn tc2_full_rewrite_must_pass() {
        let old_file = r#"
pub fn get_year() -> i32 { 2023 }
pub fn get_name() -> String { "Original".to_string() }
pub fn get_month() -> u8 { 1 }
pub fn get_day() -> u8 { 1 }
"#;
        // LLM returned all functions (correct rewrite)
        let new_file = r#"
pub fn get_year() -> i32 { 2025 }
pub fn get_name() -> String { "x".to_string() }
pub fn get_month() -> u8 { 1 }
pub fn get_day() -> u8 { 1 }
"#;

        let violations = simulate_destructive_rewrite_check(old_file, new_file);

        println!("[TC2] Violations: {:?}", violations);

        assert!(violations.is_empty(), "TC2 FAILED: Valid full rewrite was incorrectly rejected! Violations: {:?}", violations);
        println!("✅ TC2 PASSED: Full rewrite correctly ACCEPTED.");
    }

    /// TC3: Single function deleted (get_day missing)
    /// Expected: REJECT — missing function
    #[test]
    fn tc3_missing_function_must_reject() {
        let old_file = r#"
pub fn get_year() -> i32 { 2023 }
pub fn get_name() -> String { "Original".to_string() }
pub fn get_month() -> u8 { 1 }
pub fn get_day() -> u8 { 1 }
"#;
        // LLM forgot get_day
        let new_file = r#"
pub fn get_year() -> i32 { 2025 }
pub fn get_name() -> String { "x".to_string() }
pub fn get_month() -> u8 { 1 }
"#;

        let violations = simulate_destructive_rewrite_check(old_file, new_file);

        println!("[TC3] Violations: {:?}", violations);

        assert!(!violations.is_empty(), "TC3 FAILED: Missing get_day was not detected!");
        assert!(violations.iter().any(|v| v.contains("get_day")), "TC3: get_day not flagged");
        assert!(!violations.iter().any(|v| v.contains("get_year")), "TC3: get_year should NOT be flagged");
        assert!(!violations.iter().any(|v| v.contains("get_name")), "TC3: get_name should NOT be flagged");
        assert!(!violations.iter().any(|v| v.contains("get_month")), "TC3: get_month should NOT be flagged");
        println!("✅ TC3 PASSED: Missing function correctly REJECTED.");
    }
    pub fn simulate_write_gate_check(target_path: &str, state_map: &std::collections::HashMap<String, String>, initial_state_map: &std::collections::HashMap<String, String>) -> Vec<String> {
        let mut violations = Vec::new();
        for (fname, new_content) in state_map {
            let is_modified = if let Some(old_content) = initial_state_map.get(fname) {
                old_content.trim() != new_content.trim()
            } else {
                !new_content.trim().is_empty()
            };

            if is_modified && fname != target_path {
                violations.push(format!("Write Gate Violation: {}", fname));
            }
        }
        violations
    }

    #[test]
    fn tc4_unauthorized_file_modification_must_reject() {
        let target_path = "target.rs";
        let mut state_map = std::collections::HashMap::new();
        state_map.insert("target.rs".to_string(), "pub fn main() {}".to_string());
        state_map.insert("malicious.rs".to_string(), "hack()".to_string());
        
        let initial_state_map = std::collections::HashMap::new();
        
        let violations = simulate_write_gate_check(target_path, &state_map, &initial_state_map);
        
        println!("[TC4] Violations: {:?}", violations);
        assert!(!violations.is_empty(), "TC4 FAILED: Unauthorized write was not rejected!");
        assert!(violations.iter().any(|v| v.contains("malicious.rs")), "TC4: malicious.rs not flagged");
        println!("✅ TC4 PASSED: Unauthorized write correctly REJECTED.");
    }

    #[test]
    fn tc5_authorized_file_modification_must_pass() {
        let target_path = "target.rs";
        let mut state_map = std::collections::HashMap::new();
        state_map.insert("target.rs".to_string(), "pub fn main() {}".to_string());
        
        let initial_state_map = std::collections::HashMap::new();
        
        let violations = simulate_write_gate_check(target_path, &state_map, &initial_state_map);
        
        println!("[TC5] Violations: {:?}", violations);
        assert!(violations.is_empty(), "TC5 FAILED: Authorized write was incorrectly rejected! {:?}", violations);
        println!("✅ TC5 PASSED: Authorized write correctly ACCEPTED.");
    }

    pub fn simulate_static_validation(content: &str) -> Vec<String> {
        let mut violations = Vec::new();
        if content.trim().len() < 60 {
            violations.push("F_STUB".to_string());
        }
        if content.contains("```") {
            violations.push("F_MARKDOWN".to_string());
        }
        if content.contains("2023") {
            violations.push("F_HARDCODE".to_string());
        }
        violations
    }

    #[test]
    fn tc6_stub_detection_must_reject() {
        let content = "pub fn main() {}";
        let violations = simulate_static_validation(content);
        assert!(violations.contains(&"F_STUB".to_string()));
    }

    #[test]
    fn tc7_markdown_pollution_must_reject() {
        let content = "```rust\npub fn main() {}\n```";
        let violations = simulate_static_validation(content);
        assert!(violations.contains(&"F_MARKDOWN".to_string()));
    }

    #[test]
    fn tc8_hardcode_detection_must_reject() {
        let content = "pub fn get_year() -> i32 { 2023 }\n// Sufficiently long content to pass stub check. 1234567890 1234567890 1234567890";
        let violations = simulate_static_validation(content);
        assert!(violations.contains(&"F_HARDCODE".to_string()));
    }

    pub fn simulate_execution_validation(stderr: &str, success: bool) -> Vec<String> {
        let mut violations = Vec::new();
        if !success {
            if stderr.contains("SyntaxError") || stderr.contains("IndentationError") {
                violations.push("F_COMPILE_FAIL".to_string());
            } else {
                violations.push("F_RUNTIME_FAIL".to_string());
            }
        }
        violations
    }

    #[test]
    fn tc9_compile_fail_must_reject() {
        let stderr = "  File \"main.py\", line 1\n    pub fn main()\n                ^\nSyntaxError: invalid syntax";
        let violations = simulate_execution_validation(stderr, false);
        assert!(violations.contains(&"F_COMPILE_FAIL".to_string()));
    }

    #[test]
    fn tc10_runtime_fail_must_reject() {
        let stderr = "Traceback (most recent call last):\n  File \"main.py\", line 2, in <module>\n    1/0\nZeroDivisionError: division by zero";
        let violations = simulate_execution_validation(stderr, false);
        assert!(violations.contains(&"F_RUNTIME_FAIL".to_string()));
    }

    #[test]
    fn tc11_execution_pass_must_pass() {
        let violations = simulate_execution_validation("", true);
        assert!(violations.is_empty());
    }
}
