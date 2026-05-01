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
use axon_core::{events, TaskStatus};
use axon_dispatcher::{Dispatcher, Assignment};
use axon_storage::Storage;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::mpsc;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
struct RoutingParams {
    latency_weight: f64,
    fail_penalty: f64,
}

impl Default for RoutingParams {
    fn default() -> Self {
        Self {
            latency_weight: 1.0,
            fail_penalty: 1000.0,
        }
    }
}

#[derive(Debug, Default)]
struct AgentStats {
    latencies: VecDeque<f64>,
    success_count: usize,
    fail_count: usize,
}

impl AgentStats {
    fn score(&self, params: &RoutingParams) -> f64 {
        if self.success_count == 0 {
            return f64::INFINITY;
        }
        let avg_latency = self.latencies.iter().sum::<f64>() / self.latencies.len() as f64;
        let fail_penalty = self.fail_count as f64 * params.fail_penalty;
        (avg_latency * params.latency_weight) + fail_penalty
    }

    fn record_success(&mut self, latency: f64) {
        self.latencies.push_back(latency);
        if self.latencies.len() > 50 {
            self.latencies.pop_front();
        }
        self.success_count += 1;
    }

    fn record_fail(&mut self) {
        self.fail_count += 1;
    }
}

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
    agent_stats: Arc<std::sync::Mutex<HashMap<String, AgentStats>>>,
    routing_params: Arc<std::sync::Mutex<RoutingParams>>,
    pub sampling_rate: f64,
    task_counter: Arc<std::sync::atomic::AtomicUsize>,
}

#[allow(dead_code)]
impl Daemon {
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
        worker_tx: mpsc::Sender<Assignment>,
        architecture_guide: String,
        sampling_rate: f64,
        locale: String,
    ) -> Self {
        let event_bus = Arc::new(events::EventBus::new(100));
        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        
        tracing::info!("🌐 Active Factory Locale: {}", locale);

        Self {
            dispatcher: Arc::new(Dispatcher::new(worker_tx)),
            storage: storage.clone(),
            architect_model,
            architect_model_name,
            senior_models,
            senior_model_names,
            junior_models,
            junior_model_names,
            event_bus,
            architecture_guide,
            pause_tx: Arc::new(pause_tx),
            pause_rx,
            locale,
            controller: Arc::new(controller::ControlSystem::new()),
            lounge: Arc::new(axon_agent::lounge::LoungeManager::new(".")),
            admin: Arc::new(admin::AdminSystem::new(storage.clone())),
            rr_indices: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            throttler: Arc::new(tokio::sync::Semaphore::new(2)),
            agent_stats: Arc::new(std::sync::Mutex::new({
                let mut map = HashMap::new();
                if let Ok(stats) = storage.load_all_agent_stats() {
                    for (id, success, fail, latencies_json) in stats {
                        let latencies: VecDeque<f64> = serde_json::from_str(&latencies_json).unwrap_or_default();
                        map.insert(id, AgentStats { latencies, success_count: success, fail_count: fail });
                    }
                }
                map
            })),
            routing_params: Arc::new(std::sync::Mutex::new(RoutingParams::default())),
            sampling_rate,
            task_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("AXON Daemon starting (Multi-Worker Mode - Phase 07)...");
        
        // RECOVERY (v0.0.15): DB에서 처리되지 않은 태스크들을 불러와 스케줄러 큐에 재진입시킵니다.
        if let Ok(tasks) = self.storage.list_all_tasks() {
            let mut recovered_count = 0;
            for mut task in tasks {
                if task.status == axon_core::TaskStatus::Pending || task.status == axon_core::TaskStatus::InProgress {
                    task.status = axon_core::TaskStatus::Pending;
                    let _ = self.storage.save_task(&task);
                    let _ = self.dispatcher.enqueue_task(task);
                    recovered_count += 1;
                }
            }
            if recovered_count > 0 {
                tracing::info!("♻️ Recovered {} unfinished tasks from database.", recovered_count);
            }
        }
        
        let worker_count = 2; // PHASE_07: Default worker count
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

            // PHASE_07: Pop task from shared dispatcher queue
            if let Some(task) = self.dispatcher.pop_task() {
                tracing::info!("👷 [Worker {}] Popped task {}: {}", id, task.id, task.title);
                
                let mut task_in_progress = task.clone();
                task_in_progress.status = axon_core::TaskStatus::InProgress;
                let _ = self.storage.save_task(&task_in_progress);

                if let Err(e) = self.handle_assignment(task_in_progress, id).await {
                    tracing::error!("❌ [Worker {}] Task execution failed: {}", id, e);
                }
                
                // Physical cooldown to avoid API burst on multi-worker
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            } else {
                // Wait for new tasks
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        }

        Ok(())
    }

    async fn handle_assignment(&self, mut task: axon_core::Task, worker_id: usize) -> anyhow::Result<()> {
        
        // PHASE_04: Check minimum layer requirements
        if self.junior_models.is_empty() || self.senior_models.is_empty() {
            tracing::error!("❌ [EXECUTION BLOCKED]: Minimum layer requirement not satisfied.");
            return Ok(());
        }

        let mut execution_path = Vec::new();
        let mut all_metrics = Vec::new();
        let mut failures = Vec::new();
        let mut agent_metrics = Vec::new();
        let start_total = std::time::Instant::now();
        let max_retries = 2;

        // v0.0.16 Isolation (Absolute Pathing)
        let mut sandbox_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        sandbox_root.push(&task.project_id);
        
        let arch_guide_path = sandbox_root.join("architecture.md");
        let current_arch_guide = std::fs::read_to_string(&arch_guide_path).unwrap_or_else(|_| {
            tracing::warn!("⚠️  Architecture guide not found at {}, falling back to default.", arch_guide_path.display());
            self.architecture_guide.clone()
        });

        let mut final_entry_point = "main.py".to_string();
        let mut proposal = None;
        let mut summary = None;
        let mut final_simulated_state = String::new();
        let num_juniors = self.junior_models.len();
        let mut junior_failures = Vec::new();
        let mut junior_error_feedback: Option<String> = None;

        'junior_fallback: for _ in 0..num_juniors {
            // PHASE_08: Adaptive Routing Selection
            let (junior_model, junior_id, junior_name) = self.select_best_agent(axon_core::AgentRole::Junior);

            let mut junior_runtime = axon_agent::AgentRuntime::new(
                junior_id.clone(),
                axon_core::AgentRole::Junior,
                junior_name,
                junior_model
            );
            junior_runtime.set_locale(&self.locale);
            junior_runtime.throttler = Some(self.throttler.clone());

            for retry_attempt in 0..=max_retries {
                let start_step = std::time::Instant::now();
                match junior_runtime.process_task(&task, &current_arch_guide, junior_error_feedback.clone(), Some(self.event_bus.clone())).await {
                    Ok(p) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        
                        // v0.0.19: Hard Fail Conditions (Strict Output Contract)
                        // v0.0.22: Hardened JSON Contract Validation
                        let is_hard_fail = p.full_code.is_none() || p.full_code.as_ref().unwrap().trim().is_empty();

                        if is_hard_fail {
                            self.record_failure_trace(&task.id, "HARD_FAIL_CONTRACT", "junior_output", "json_missing", "JuniorOutput");
                            tracing::warn!("⚠️ Junior {} retry {} Hard Fail: Missing structured JSON in response", junior_runtime.agent.name, retry_attempt + 1);
                            let _ = std::fs::write(format!("{}_rejected.txt", junior_runtime.agent.id), &p.content);
                            if retry_attempt == max_retries {
                                self.record_failure_trace(&task.id, "TASK_SUMMARY", "FAIL", "all_retries_failed", "Final");
                                junior_failures.push(format!("Junior {} failed to provide a valid JSON proposal after {} retries.", junior_id, max_retries + 1));
                            }
                            continue;
                        }

                        // PHASE_06: Hardened Stage 6 - IR-First Validation
                        let mut ir_validation_success = true;
                        let mut ir_validation_err = String::new();
                        let mut simulated_state_json = String::new();

                        // 1. Get structured JSON/Patch from Junior's response
                        let junior_json = p.full_code.clone().unwrap_or_default();

                        // 1.5. IR Mapping Validation (v0.0.23: IR-First)
                        if let Some(ir) = axon_core::ir::ProjectIR::from_md(&current_arch_guide) {
                            let ir_json = serde_json::to_string(&ir).unwrap_or_default();
                            let tmp_ir_path = format!("{}/.ir_{}.json", task.project_id, uuid::Uuid::new_v4());
                            let _ = std::fs::write(&tmp_ir_path, &ir_json);
                            
                            let tmp_junior_json = format!("{}/.junior_{}.json", task.project_id, uuid::Uuid::new_v4());
                            let _ = std::fs::write(&tmp_junior_json, &junior_json);

                            tracing::info!("🏛️ [Stage 6.1] IR Mapping Validation for Junior {}", junior_runtime.agent.name);
                            let mapper_output = std::process::Command::new("python3")
                                .arg(Self::resolve_tool_path("axon_ir_mapper.py"))
                                .arg(&tmp_ir_path)
                                .arg(&tmp_junior_json)
                                .output();
                            
                            let _ = std::fs::remove_file(&tmp_ir_path);

                            match mapper_output {
                                Ok(out) if !out.status.success() => {
                                    ir_validation_success = false;
                                    ir_validation_err = format!("IR Contract Violation: {}", String::from_utf8_lossy(&out.stdout));
                                },
                                Err(e) => {
                                    tracing::error!("❌ [IR MAPPER ERROR]: {}", e);
                                },
                                _ => {
                                    tracing::info!("✅ [Stage 6.1] IR Mapping Passed.");
                                }
                            }
                            let _ = std::fs::remove_file(&tmp_junior_json);
                        }

                        if !ir_validation_success {
                            self.record_failure_trace(&task.id, "IR_CONTRACT_FAIL", "architecture", "structure", "Stage6.1");
                            tracing::warn!("⚠️ [Stage 6.1] IR Fail for {}: {}", junior_runtime.agent.name, ir_validation_err);
                            junior_error_feedback = Some(ir_validation_err.clone());
                            if retry_attempt == max_retries {
                                junior_failures.push(format!("Junior {}: IR contract failed: {}", junior_runtime.agent.name, ir_validation_err));
                            }
                            continue;
                        }

                        // 2. Patch Simulation (Virtual FS)
                        let tmp_junior_json = format!("{}/.junior_sim_{}.json", task.project_id, uuid::Uuid::new_v4());
                        let _ = std::fs::write(&tmp_junior_json, &junior_json);
                        let simulation_output = std::process::Command::new("python3")
                            .arg(Self::resolve_tool_path("axon_patch_simulator.py"))
                            .arg(&task.project_id)
                            .arg(&tmp_junior_json)
                            .output();

                        match simulation_output {
                            Ok(out) if out.status.success() => {
                                simulated_state_json = String::from_utf8_lossy(&out.stdout).into_owned();
                            },
                            Ok(out) => {
                                ir_validation_success = false;
                                ir_validation_err = format!("Patch Simulation Failed: {}", String::from_utf8_lossy(&out.stderr));
                            },
                            Err(e) => {
                                ir_validation_success = false;
                                ir_validation_err = format!("Patch Simulator Error: {}", e);
                            }
                        }

                        // 3. Semantic IR Validation (AST & Schema)
                        if ir_validation_success {
                            let constraints_path = format!("{}/constraints.json", task.project_id);
                            let tmp_state_path = format!("{}/.state_{}.json", task.project_id, uuid::Uuid::new_v4());
                            let _ = std::fs::write(&tmp_state_path, &simulated_state_json);

                            let validator_output = std::process::Command::new("python3")
                                .arg(Self::resolve_tool_path("axon_ir_validator.py"))
                                .arg(&constraints_path)
                                .arg(&tmp_state_path)
                                .arg(&task.project_id)
                                .output();

                            let _ = std::fs::remove_file(&tmp_state_path);

                            match validator_output {
                                Ok(out) if out.status.success() => {
                                    tracing::info!("✅ [Stage 6.2] Semantic IR Validation Passed for {}", junior_runtime.agent.name);
                                },
                                Ok(out) => {
                                    ir_validation_success = false;
                                    ir_validation_err = format!("Semantic Validation Failed:\n{}", String::from_utf8_lossy(&out.stderr));
                                },
                                Err(e) => {
                                    ir_validation_success = false;
                                    ir_validation_err = format!("IR Validator Error: {}", e);
                                }
                            }
                        }

                        let _ = std::fs::remove_file(&tmp_junior_json);

                        if !ir_validation_success {
                            self.record_failure_trace(&task.id, "VALIDATION_FAIL", "semantic", "unknown", "Stage6");
                            tracing::warn!("⚠️ [Stage 6] Validation Fail for {}: {}", junior_runtime.agent.name, ir_validation_err);
                            junior_error_feedback = Some(ir_validation_err.clone());
                            if retry_attempt == max_retries {
                                junior_failures.push(format!("Junior {}: Semantic validation failed: {}", junior_runtime.agent.name, ir_validation_err));
                            }
                            continue;
                        }

                        // v0.0.19: Stage 5 --- Autonomous Feedback Loop (Pre-review Execution Check)
                        if ir_validation_success {
                            // Extract simulated files for harness
                            let file_map: std::collections::HashMap<String, String> = serde_json::from_str(&simulated_state_json).unwrap_or_default();
                            
                            for fname in file_map.keys() {
                                if fname.to_lowercase().ends_with("main.py") {
                                    final_entry_point = fname.clone();
                                }
                            }

                            if !file_map.is_empty() {
                                let tmp_json_path = format!("{}/.harness_{}.json", task.project_id, uuid::Uuid::new_v4());
                                let _ = std::fs::write(&tmp_json_path, &simulated_state_json);

                                // v0.0.19: Architecture Mapping Validation (Before execution)
                                let arch_file_path = format!("{}/architecture.md", task.project_id);
                                tracing::info!("🗺️ [Stage 4.5] Architecture Mapping Validation for Junior {}", junior_runtime.agent.name);
                                
                                let mapping_output = std::process::Command::new("python3")
                                    .arg(Self::resolve_tool_path("axon_mapping_validator.py"))
                                    .arg(&arch_file_path)
                                    .arg(&task.project_id)
                                    .arg("--state-json")
                                    .arg(&tmp_json_path)
                                    .output();

                                match mapping_output {
                                    Ok(out) if !out.status.success() => {
                                        let err_msg = String::from_utf8_lossy(&out.stdout).into_owned();
                                        self.record_failure_trace(&task.id, "MAPPING_DRIFT", "architecture", "structure", "Stage4.5");
                                        // v0.0.19: Observation Mode - WARN only, do not block
                                        tracing::warn!("🗺️ [Stage 4.5] [OBSERVATION] Architecture Drift Detected for {}:\n{}", junior_runtime.agent.name, err_msg);
                                        // junior_error_feedback = Some(format!("Architecture Drift Detected (Warning):\n{}", err_msg));
                                    },
                                    Err(e) => {
                                        tracing::error!("❌ [MAPPING VALIDATOR ERROR]: {}", e);
                                    },
                                    _ => {
                                        tracing::info!("✅ [Stage 4.5] Architecture Mapping Passed for Junior {}", junior_runtime.agent.name);
                                    }
                                }

                                // v0.0.19: Stage 4.6 --- Soft Rule Enforcement (Judge Training)
                                let suggestions_path = ".axon_trace/suggested_rules.json";
                                if std::path::Path::new(suggestions_path).exists() {
                                    // Stage 4.8.8: Dependency-aware filtering
                                    let changed_files = if let Ok(state_map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&simulated_state_json) {
                                        state_map.keys().cloned().collect::<Vec<String>>().join(",")
                                    } else {
                                        "".to_string()
                                    };

                                    let soft_res = std::process::Command::new("python3")
                                        .arg(Self::resolve_tool_path("axon_soft_rule_engine.py"))
                                        .arg(&tmp_json_path)
                                        .arg(suggestions_path)
                                        .arg(changed_files)
                                        .output();
                                    
                                    if let Ok(out) = soft_res {
                                        let stdout = String::from_utf8_lossy(&out.stdout);
                                        if stdout.contains("<<<<SOFT_RULES_VIOLATION>>>>") {
                                            for line in stdout.lines() {
                                                if line.trim().starts_with('{') {
                                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                                                        let rule_type = v["rule"]["type"].as_str().unwrap_or("unknown");
                                                        let rule_file = v["rule"]["file"].as_str().unwrap_or("unknown");
                                                        let rule_symbol = v["rule"]["symbol"].as_str().unwrap_or("unknown");
                                                        self.record_failure_trace(&task.id, "RULE_VIOLATION", rule_file, rule_symbol, "Stage4.6");
                                                        tracing::warn!("⚖️ [Stage 4.6] [SOFT VIOLATION] Rule '{}' failed for {}: {}", rule_type, rule_file, rule_symbol);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                tracing::info!("🧪 [Stage 5] Pre-review Execution Check for Junior {}", junior_runtime.agent.name);
                                
                                let harness_output = std::process::Command::new("python3")
                                    .arg(Self::resolve_tool_path("axon_execution_harness.py"))
                                    .arg("--project-root")
                                    .arg(&task.project_id)
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
                                            self.record_failure_trace(&task.id, "RUNTIME_CRASH", &final_entry_point, "main", "Stage5");
                                            tracing::warn!("⚠️ [Stage 5] Execution Fail for {}: {}", junior_runtime.agent.name, err_msg);
                                            junior_error_feedback = Some(err_msg.clone());
                                            
                                            if retry_attempt == max_retries {
                                                junior_failures.push(format!("Junior {}: Runtime failure: {}", junior_runtime.agent.name, err_msg));
                                            }
                                            continue; // RETRY with feedback
                                        }
                                        tracing::info!("✅ [Stage 5] Execution Pass for Junior {}", junior_runtime.agent.name);
                                        
                                        // PHASE_06: Operational Integrity - Golden Test (Regression)
                                        let constraints_path = format!("{}/constraints.json", task.project_id);
                                        let tmp_state_path = format!("{}/.state_final_{}.json", task.project_id, uuid::Uuid::new_v4());
                                        let _ = std::fs::write(&tmp_state_path, &simulated_state_json);

                                        let golden_output = std::process::Command::new("python3")
                                            .arg(Self::resolve_tool_path("axon_golden_tester.py"))
                                            .arg(&constraints_path)
                                            .arg(&tmp_state_path)
                                            .output();


                                        match golden_output {
                                            Ok(out) if out.status.success() => {
                                                tracing::info!("🏆 [Stage 6] Golden Test (Regression) Passed for Junior {}", junior_runtime.agent.name);
                                                
                                                // PHASE_07: Operational Integrity - Property Test (Fuzzing)
                                                let property_output = std::process::Command::new("python3")
                                                    .arg(Self::resolve_tool_path("axon_property_tester.py"))
                                                    .arg(&constraints_path)
                                                    .arg(&tmp_state_path)
                                                    .output();
                                                 // v0.0.22: Clean up state file AFTER all integrity tests
                                                 let _ = std::fs::remove_file(&tmp_state_path);

                                                match property_output {
                                                    Ok(out) if out.status.success() => {
                                                        tracing::info!("🎲 [Stage 7] Property Test (Fuzzing) Passed for Junior {}", junior_runtime.agent.name);
                                                    },
                                                    Ok(out) => {
                                                        let fuzz_msg = String::from_utf8_lossy(&out.stdout).into_owned();
                                                        tracing::warn!("❌ [Stage 7] Property Test Failed (Edge Case Found) for {}:\n{}", junior_runtime.agent.name, fuzz_msg);
                                                         junior_error_feedback = Some(format!("PROPERTY FAILURE: Randomized edge-case check failed. DETAILS: {}", fuzz_msg.replace("<<<<PROPERTY_TEST_FAILED>>>>", "").trim()));
                                                        if retry_attempt == max_retries {
                                                            junior_failures.push(format!("Junior {}: Property test failed.", junior_runtime.agent.name));
                                                        }
                                                        continue; // RETRY with fuzzing feedback
                                                    },
                                                    Err(e) => {
                                                        tracing::error!("❌ Failed to execute property tester: {}", e);
                                                    }
                                                }
                                            },
                                            Ok(out) => {
                                                let trace_msg = String::from_utf8_lossy(&out.stdout).into_owned();
                                                tracing::warn!("❌ [Stage 6] Golden Test (Regression) Failed for {}:\n{}", junior_runtime.agent.name, trace_msg);
                                                 junior_error_feedback = Some(format!("REGRESSION FAILURE: Business logic invariants violated. DETAILS: {}", trace_msg.replace("<<<<GOLDEN_TEST_FAILED>>>>", "").trim()));
                                                if retry_attempt == max_retries {
                                                    junior_failures.push(format!("Junior {}: Golden test failed.", junior_runtime.agent.name));
                                                }
                                                continue; // RETRY with regression feedback
                                            },
                                            Err(e) => {
                                                tracing::error!("❌ Failed to execute golden tester: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ Failed to execute harness: {}", e);
                                        ir_validation_success = false;
                                        ir_validation_err = format!("Harness execution failed: {}", e);
                                    }
                                }
                            }
                        }


                        if !ir_validation_success {
                            tracing::warn!("⚠️ Junior {} retry {} Semantic Validation Fail: {}", junior_runtime.agent.name, retry_attempt + 1, ir_validation_err);
                            junior_error_feedback = Some(ir_validation_err.clone());
                            if retry_attempt == max_retries {
                                junior_failures.push(format!("Junior {}: Semantic validation failed: {}", junior_runtime.agent.name, ir_validation_err));
                            }
                            continue;
                        }

                        tracing::info!("✅ [Stage 5] Autonomous Loop Complete for Junior {}", junior_runtime.agent.name);

                        agent_metrics.push(axon_core::AgentMetric {
                            id: junior_id.clone(),
                            role: "junior".to_string(),
                            status: "OK".to_string(),
                            latency_ms: latency,
                            attempts: (retry_attempt + 1) as u32,
                            error: None,
                        });

                        // SUCCESS: Post-processing within the same scope
                        self.record_failure_trace(&task.id, "TASK_SUMMARY", "SUCCESS", "none", "Final");
                        if let Some(m) = &p.metrics {
                            all_metrics.push(m.clone());
                        }
                        execution_path.push(("junior".to_string(), junior_id.clone()));
                        let _ = self.storage.save_post(&p);
                        
                        self.event_bus.publish(axon_core::Event {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: task.project_id.clone(),
                            thread_id: Some(task.id.clone()),
                            agent_id: Some(junior_id.clone()),
                            event_type: axon_core::EventType::AgentResponse,
                            source: junior_id.clone(),
                            content: format!("Junior {} proposed a solution", junior_runtime.agent.name),
                            payload: None,
                            timestamp: chrono::Local::now(),
                        });

                        let _ = self.lounge.log_vibe(&junior_runtime.agent, axon_agent::lounge::Vibe::Focus);
                        
                        let summary_post = junior_runtime.generate_system_summary(&p, Some(self.event_bus.clone())).await?;
                        if let Some(m) = &summary_post.metrics {
                            all_metrics.push(m.clone());
                        }
                        let _ = self.storage.save_post(&summary_post);
                        
                        summary = Some(summary_post);
                        proposal = Some(p);
                        final_simulated_state = simulated_state_json.clone();
                        final_entry_point = final_entry_point.clone();
                        break 'junior_fallback;
                    }
                    Err(e) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        self.record_agent_fail(&junior_id); // PHASE_08
                        tracing::warn!("⚠️ Junior {} retry {} failed: {}", junior_runtime.agent.name, retry_attempt + 1, e);
                        if retry_attempt == max_retries {
                            agent_metrics.push(axon_core::AgentMetric {
                                id: junior_id.clone(),
                                role: "junior".to_string(),
                                status: "FAIL".to_string(),
                                latency_ms: latency,
                                attempts: (retry_attempt + 1) as u32,
                                error: Some(e.to_string()),
                            });
                            junior_failures.push(format!("Junior {}: {}", junior_runtime.agent.name, e));
                        }
                    }
                }
            }
        }

        if proposal.is_none() || summary.is_none() {
            failures.extend(junior_failures);
            return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
        }
        let proposal = proposal.unwrap();
        let summary = summary.unwrap();

        // 3. SENIOR SELECTION (with Fallback & Retry)
        let mut review = None;
        let num_seniors = self.senior_models.len();
        let mut senior_failures = Vec::new();

        'senior_fallback: for _ in 0..num_seniors {
            // PHASE_08: Adaptive Routing
            let (senior_model, senior_id, senior_name) = self.select_best_agent(axon_core::AgentRole::Senior);

            let mut senior_runtime = axon_agent::AgentRuntime::new(
                senior_id.clone(),
                axon_core::AgentRole::Senior,
                senior_name,
                senior_model
            );
            senior_runtime.set_locale(&self.locale);
            senior_runtime.throttler = Some(self.throttler.clone());

            for retry_attempt in 0..=max_retries {
                let start_step = std::time::Instant::now();
                match senior_runtime.review_proposal(&task, &proposal, Some(&summary), Some(self.event_bus.clone())).await {
                    Ok(r) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        self.record_agent_success(&senior_id, latency); // PHASE_08
                        agent_metrics.push(axon_core::AgentMetric {
                            id: senior_runtime.agent.id.clone(),
                            role: "senior".to_string(),
                            status: "OK".to_string(),
                            latency_ms: latency,
                            attempts: (retry_attempt + 1) as u32,
                            error: None,
                        });

                        if let Some(m) = &r.metrics {
                            all_metrics.push(m.clone());
                        }
                        execution_path.push(("senior".to_string(), "senior-agent-1".to_string()));
                        let _ = self.storage.save_post(&r);
                        
                        self.event_bus.publish(axon_core::Event {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: task.project_id.clone(),
                            thread_id: Some(task.id.clone()),
                            agent_id: Some(senior_runtime.agent.id.clone()),
                            event_type: axon_core::EventType::AgentAction,
                            source: senior_runtime.agent.id.clone(),
                            content: format!("Senior {} reviewed the proposal", senior_runtime.agent.name),
                            payload: None,
                            timestamp: chrono::Local::now(),
                        });

                        review = Some(r);
                        break 'senior_fallback;
                    }
                    Err(e) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        self.record_agent_fail(&senior_id); // PHASE_08
                        tracing::warn!("⚠️ Senior {} retry {} failed: {}", senior_runtime.agent.name, retry_attempt + 1, e);
                        if retry_attempt == max_retries {
                            agent_metrics.push(axon_core::AgentMetric {
                                id: senior_runtime.agent.id.clone(),
                                role: "senior".to_string(),
                                status: "FAIL".to_string(),
                                latency_ms: latency,
                                attempts: (retry_attempt + 1) as u32,
                                error: Some(e.to_string()),
                            });
                            senior_failures.push(format!("Senior {}: {}", senior_runtime.agent.name, e));
                        }
                    }
                }
            }
        }

        if review.is_none() {
            failures.extend(senior_failures);
            return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
        }
        let review = review.unwrap();

        // 4. ARCHITECT VALIDATION (with Probabilistic Bypass - v0.0.17)
        let mut validation = None;
        let mut arch_failures = Vec::new();

        use rand::Rng;
        let roll = rand::thread_rng().gen_range(0.0..1.0);
        
        if roll <= self.sampling_rate {
            tracing::info!("🔍 [SAMPLING]: Architect selected for high-fidelity validation (roll: {:.2}/{:.2})", roll, self.sampling_rate);
            
            // Architect usually has 1 model, but we follow the fallback pattern for consistency
            for retry_attempt in 0..=max_retries {
                let start_step = std::time::Instant::now();
                let mut architect_runtime = axon_agent::AgentRuntime::new(
                    "architect-agent-1".to_string(),
                    axon_core::AgentRole::Architect,
                    self.architect_model_name.clone(),
                    self.architect_model.clone()
                );
                architect_runtime.set_locale(&self.locale);
                architect_runtime.throttler = Some(self.throttler.clone());

                match architect_runtime.validate_architecture(&task, &review, &self.architecture_guide, Some(self.event_bus.clone())).await {
                    Ok(v) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        self.record_agent_success("architect-agent-1", latency);
                        agent_metrics.push(axon_core::AgentMetric {
                            id: architect_runtime.agent.id.clone(),
                            role: "architect".to_string(),
                            status: "OK".to_string(),
                            latency_ms: latency,
                            attempts: (retry_attempt + 1) as u32,
                            error: None,
                        });

                        if let Some(m) = &v.metrics {
                            all_metrics.push(m.clone());
                        }
                        execution_path.push(("architect".to_string(), "architect-agent-1".to_string()));
                        let _ = self.storage.save_post(&v);
                        
                        self.event_bus.publish(axon_core::Event {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: task.project_id.clone(),
                            thread_id: Some(task.id.clone()),
                            agent_id: Some(architect_runtime.agent.id.clone()),
                            event_type: axon_core::EventType::AgentAction,
                            source: architect_runtime.agent.id.clone(),
                            content: format!("Architect {} validated the proposal", architect_runtime.agent.name),
                            payload: None,
                            timestamp: chrono::Local::now(),
                        });

                        validation = Some(v);
                        break;
                    }
                    Err(e) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        self.record_agent_fail("architect-agent-1");
                        tracing::warn!("⚠️ Architect retry {} failed: {}", retry_attempt + 1, e);
                        if retry_attempt == max_retries {
                            agent_metrics.push(axon_core::AgentMetric {
                                id: architect_runtime.agent.id.clone(),
                                role: "architect".to_string(),
                                status: "FAIL".to_string(),
                                latency_ms: latency,
                                attempts: (retry_attempt + 1) as u32,
                                error: Some(e.to_string()),
                            });
                            arch_failures.push(format!("Architect failure: {}", e));
                        }
                    }
                }
            }

            if validation.is_none() {
                failures.extend(arch_failures);
                return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
            }
        } else {
            tracing::info!("⚡ [BYPASS]: Architect skipped via sampling rate ({:.2} > {:.2}). Promoting Senior review.", roll, self.sampling_rate);
            // v0.0.17: When bypassed, the Senior's review is promoted to the final validation
            validation = Some(review.clone());
        }

        let validation = validation.unwrap();

        // 5. ISOLATION SYNC (v0.0.16): 최종 승인된 주니어의 코드를 프로젝트 샌드박스에 물리적 반영
        // v0.0.22: Removed redundant sync_post_to_sandbox call as 'promote' handles SSOT updates via JSON state.
        if validation.content.contains("APPROVE") || review.content.contains("APPROVE") {
            // v0.0.22: Official SSOT Promotion after Senior/Architect Approval
            let tmp_files_json = format!("{}/.promote_final_{}.json", task.project_id, uuid::Uuid::new_v4());
            let _ = std::fs::write(&tmp_files_json, &final_simulated_state);
            
            let registry_res = std::process::Command::new("python3")
                .arg(Self::resolve_tool_path("axon_registry.py"))
                .arg("promote")
                .arg("--root")
                .arg(&task.project_id)
                .arg("--files-json")
                .arg(&tmp_files_json)
                .arg("--task-id")
                .arg(&task.id)
                .output();
            
            let _ = std::fs::remove_file(&tmp_files_json);

            if let Ok(rout) = registry_res {
                if rout.status.success() {
                    tracing::info!("🚀 [SSOT PROMOTED] Versioned snapshot created for task {} after Senior Review", task.id);
                    
                    // v0.0.23: Strict Simulation Error Check
                    if let Ok(state_map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&final_simulated_state) {
                        for (k, v) in &state_map {
                            if k.starts_with("error_") {
                                tracing::error!("❌ [SIMULATION_FAILED] {}: {}", k, v);
                                failures.push(format!("Simulation Error: {}", v));
                                return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
                            }
                        }
                        
                        // 1. Snapshot/Backup before commit (for Rollback)
                        let mut backups = std::collections::HashMap::new();
                        for (fname, _) in &state_map {
                            let fpath = std::path::Path::new(&task.project_id).join(fname);
                            if fpath.exists() {
                                if let Ok(content) = std::fs::read_to_string(&fpath) {
                                    backups.insert(fname.clone(), content);
                                }
                            }
                        }

                        for (fname, code) in &state_map {
                            let fpath = std::path::Path::new(&task.project_id).join(fname);
                            if let Some(parent) = fpath.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            if let Err(e) = std::fs::write(&fpath, code) {
                                tracing::error!("❌ [COMMIT FAILED] Critical IO error at {}: {}", fpath.display(), e);
                                failures.push(format!("Physical Commit Failed: {}", e));
                                return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
                            }
                        }

                        // 2. Final Physical Harness (on the REAL project root)
                        tracing::info!("🧪 [FINAL VERIFICATION] Running harness on physical files for task {}...", task.id);
                        
                        // We need a dummy empty json for the harness to skip virtual patching
                        let dummy_json_path = format!("{}/.harness_dummy_{}.json", task.project_id, uuid::Uuid::new_v4());
                        let _ = std::fs::write(&dummy_json_path, "{}");

                        let final_harness = std::process::Command::new("python3")
                            .arg(Self::resolve_tool_path("axon_execution_harness.py"))
                            .arg("--project-root")
                            .arg(&task.project_id)
                            .arg("--files-json")
                            .arg(&dummy_json_path)
                            .arg("--entry")
                            .arg(&final_entry_point)
                            .output();
                        
                        let _ = std::fs::remove_file(&dummy_json_path);
                        
                        match final_harness {
                            Ok(out) if out.status.success() => {
                                tracing::info!("✅ [COMMIT_SUCCESS] Physical validation passed for task {}. Factory proceeding...", task.id);
                            },
                            _ => {
                                // v0.0.23: PESSIMISTIC INTERVENTION & AUTO-ROLLBACK
                                let err_msg = if let Ok(out) = final_harness {
                                    String::from_utf8_lossy(&out.stderr).into_owned()
                                } else {
                                    "Execution failure".to_string()
                                };
                                
                                tracing::error!("🚨 [COMMIT_FAILED] Physical validation failed for task {}: {}", task.id, err_msg);
                                tracing::warn!("📢 [AUTO-ROLLBACK] Reverting files to previous state to maintain factory integrity.");
                                
                                // Perform Rollback
                                for (fname, content) in backups {
                                    let fpath = std::path::Path::new(&task.project_id).join(fname);
                                    let _ = std::fs::write(fpath, content);
                                }

                                tracing::warn!("📢 [SENIOR INTERRUPT] Physical environment mismatch. Manual intervention required.");
                                failures.push(format!("COMMIT_PENDING Failure: Physical execution failed after commit. Details: {}", err_msg));
                                return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
                            }
                        }
                    }
                } else {
                    tracing::error!("❌ [REGISTRY ERROR] Promotion failed at final stage.");
                }
            }

            // v0.0.16: 아키텍처 섹션 잠금 (격리 경로 적용)
            let _ = self.lock_in_architecture(&task.project_id, &task.title);
        } else {
            // v0.0.22: Hard-fail if not approved by Senior
            tracing::warn!("⛔ [REJECTED]: Senior/Architect refused to approve task {}. Marking as FAILED.", task.id);
            failures.push(format!("Review Refusal: The Senior agent did not find the solution satisfactory."));
            return self.abort_with_failure(&mut task, failures, execution_path, all_metrics, agent_metrics, start_total, worker_id).await;
        }

        // Final Status Update (v0.0.17: Mark as Completed)
        task.status = axon_core::TaskStatus::Completed;
        let _ = self.storage.save_task(&task);
        
        if let Ok(Some(mut thread)) = self.storage.get_thread(&task.id) {
            thread.status = axon_core::ThreadStatus::Completed;
            thread.updated_at = chrono::Local::now();
            let _ = self.storage.save_thread(&thread);
            
            // Notify event bus of thread completion
            self.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: task.project_id.clone(),
                thread_id: Some(task.id.clone()),
                agent_id: None,
                event_type: axon_core::EventType::ThreadCompleted,
                source: "system".to_string(),
                content: format!("Thread '{}' completed successfully", task.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });
        }

        // PHASE_10: Trigger Feedback Loop every 10 tasks
        let count = self.task_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        if count % 10 == 0 {
            self.update_routing_params();
        }

        // PHASE_05: Observability Report
        let last_metrics = all_metrics.last().cloned().unwrap_or_default();
        let total_duration_ms = start_total.elapsed().as_secs_f64() * 1000.0;

        let report = axon_core::ObservabilityReport {
            agents: agent_metrics,
            execution_path,
            metrics: last_metrics,
            summary: axon_core::ExecutionSummary {
                worker_id,
                total_duration_ms,
                steps: all_metrics.len(),
                status: "RUNNING".to_string(),
            },
            queue: axon_core::QueueStatus {
                length: self.dispatcher.len(),
                limit: self.dispatcher.limit(),
            },
            failures,
        };

        tracing::info!("📊 Observability Report: {:?}", report);
        
        // Publish to event bus
        self.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: task.project_id.clone(),
            thread_id: Some(task.id.clone()),
            agent_id: None,
            event_type: axon_core::EventType::SystemLog,
            source: "observability".to_string(),
            content: serde_json::to_string(&report).unwrap_or_default(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        task.status = TaskStatus::Completed;
        task.result = Some(validation.content.clone());
        let _ = self.storage.save_task(&task);

        Ok(())
    }

    pub async fn bootstrap_from_spec(&self, spec_path: String) -> anyhow::Result<()> {
        let spec_content = std::fs::read_to_string(&spec_path)?;
        let project_id = std::path::Path::new(&spec_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default-project")
            .to_string();

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
        tracing::info!("Starting Bootstrap V3 (Pure Rust Deterministic Pipeline) for project '{}'...", self.project_id);

        let mut architect_runtime = axon_agent::AgentRuntime::new(
            "architect-agent-001".to_string(),
            axon_core::AgentRole::Architect,
            daemon.architect_model_name.clone(),
            daemon.architect_model.clone()
        ).with_timeout(600).with_project(self.project_id.clone());
        architect_runtime.set_locale(&daemon.locale);

        // 1. Initial IR Fill
        tracing::info!("Stage 1: Initial IR generation...");
        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            thread_id: None,
            agent_id: Some("architect-agent-001".to_string()),
            event_type: axon_core::EventType::SystemLog,
            source: "bootstrap".to_string(),
            content: "Stage 1: Initial IR generation started...".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        let mut ir = architect_runtime.generate_ir(&spec_content, Some(daemon.event_bus.clone())).await?;
        let mut prev_hash = String::new();

        // 2. Deterministic Convergence Loop
        tracing::info!("Stage 2: Deterministic Convergence Loop (JSON IR -> Validator -> Repair)");
        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            thread_id: None,
            agent_id: Some("architect-agent-001".to_string()),
            event_type: axon_core::EventType::SystemLog,
            source: "bootstrap".to_string(),
            content: "Stage 2: Convergence Loop (IR Validation & Repair) started...".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        for attempt in 1..=10 {
            // v0.0.23: Simplified validation for bootstrap phase
            let mut errors = Vec::new();
            if ir.components.is_empty() {
                errors.push("IR is empty. No components defined.".to_string());
            }

            if errors.is_empty() {
                tracing::info!("✅ IR Converged on attempt {}.", attempt);
                daemon.event_bus.publish(axon_core::Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: self.project_id.clone(),
                    thread_id: None,
                    agent_id: Some("architect-agent-001".to_string()),
                    event_type: axon_core::EventType::SystemLog,
                    source: "bootstrap".to_string(),
                    content: format!("✅ IR Converged on attempt {}.", attempt),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
                break;
            }

            tracing::warn!("⚠️ Attempt {}: Found {} validation errors. Repairing...", attempt, errors.len());
            let new_ir = architect_runtime.repair_ir(&ir, &errors, Some(daemon.event_bus.clone())).await?;
            
            let hash = format!("{:?}", new_ir);
            if hash == prev_hash {
                tracing::warn!("⏸️ IR state stabilized but errors remain. Breaking loop.");
                break;
            }
            prev_hash = hash;
            ir = new_ir;

            if attempt == 10 {
                return Err(anyhow::anyhow!("Failed to converge IR after 10 attempts."));
            }
        }

        // 3. Sync to Markdown (Architecture.md)
        tracing::info!("Stage 3: Generating architecture.md from converged IR...");
        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            thread_id: None,
            agent_id: Some("architect-agent-001".to_string()),
            event_type: axon_core::EventType::SystemLog,
            source: "bootstrap".to_string(),
            content: "Stage 3: Generating architecture.md from IR...".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        let arch_md = architect_runtime.generate_architecture_from_ir(&ir, Some(daemon.event_bus.clone())).await?;
        
        let _ = std::fs::create_dir_all(&self.sandbox_root);
        let arch_file_path = self.sandbox_root.join("architecture.md");
        std::fs::write(&arch_file_path, &arch_md)?;
        
        // 4. Save IR to constraints.json (for legacy tools compatibility)
        let constraints_path = self.sandbox_root.join("constraints.json");
        let ir_json = serde_json::to_string_pretty(&ir)?;
        std::fs::write(&constraints_path, &ir_json)?;
        
        // 4.5 Stage 3.5: Stub Generation (Physical File Materialization)
        // v0.0.22: Fix for cross-file import errors during parallel bootstrapping.
        // Pre-create all files defined in the IR as empty stubs so that 'import' statements pass validation.
        tracing::info!("Stage 3.5: Generating physical file stubs to satisfy import dependencies...");
        for comp in ir.components.values() {
            let file_path = self.sandbox_root.join(&comp.file_path);
            if !file_path.exists() {
                if let Some(parent) = file_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                // Write a minimal stub or empty file
                let _ = std::fs::write(&file_path, format!("# AXON STUB: {}\n# Implementation pending...\n", comp.name));
            }
        }

        // v0.0.23: Guarantee main.py entry point always exists.
        // The execution harness (Stage 5) requires main.py.
        // If the IR has a 'main' component, ensure its stub is named main.py.
        // If not, auto-generate a minimal main.py that imports all components.
        let main_py_path = self.sandbox_root.join("main.py");
        if !main_py_path.exists() {
            let has_main_comp = ir.components.contains_key("main");
            let main_content = if has_main_comp {
                // Delegate to the existing main component's stub (already created above)
                // Just create a thin main.py wrapper
                "# AXON: Auto-generated entry point\nif __name__ == '__main__':\n    print('AXON project initialized.')\n".to_string()
            } else {
                // Synthesize a minimal main.py that imports all components
                let imports: Vec<String> = ir.components.values()
                    .filter_map(|c| {
                        // Only include Python files (.py)
                        if c.file_path.ends_with(".py") {
                            let module = c.file_path.trim_end_matches(".py").replace(['/', '\\'], ".");
                            Some(format!("# from {} import *", module))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                format!(
                    "# AXON: Auto-generated entry point\n# Project: {}\n{}\n\nif __name__ == '__main__':\n    print('AXON project initialized.')\n",
                    self.project_id,
                    imports.join("\n")
                )
            };
            
            tracing::info!("Stage 3.5: Auto-generating main.py entry point (IR had no 'main' component).");
            std::fs::write(&main_py_path, main_content)?;
        }

        // 5. Extraction of Tasks (from IR)
        tracing::info!("Stage 4: Extracting implementation tasks from IR...");
        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            thread_id: None,
            agent_id: Some("architect-agent-001".to_string()),
            event_type: axon_core::EventType::SystemLog,
            source: "bootstrap".to_string(),
            content: "Stage 4: Extracting tasks and creating work threads...".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        for comp in ir.components.values() {
            let task_id = uuid::Uuid::new_v4().to_string();
            let description = format!(
                "RESPONSIBILITY: implementation of {} component\n\nFUNCTIONS:\n{}",
                comp.name,
                comp.functions.values().map(|f| format!("- {}", f.signature)).collect::<Vec<_>>().join("\n")
            );

            let task = axon_core::Task {
                id: task_id.clone(),
                project_id: self.project_id.clone(),
                title: format!("Implement {}", comp.name),
                description: description.clone(),
                status: TaskStatus::Pending,
                result: None,
                created_at: chrono::Local::now(),
            };

            // v0.0.22: Also create a Thread so it shows up in the Work Board
            let thread = axon_core::Thread {
                id: task_id.clone(),
                project_id: self.project_id.clone(),
                title: task.title.clone(),
                status: axon_core::ThreadStatus::Working,
                author: "Architect".to_string(),
                milestone_id: None,
                created_at: task.created_at,
                updated_at: task.created_at,
            };

            // v0.0.22: Add an initial instruction post
            let post = axon_core::Post {
                id: uuid::Uuid::new_v4().to_string(),
                thread_id: task_id.clone(),
                author_id: "Architect".to_string(),
                content: description,
                full_code: None,
                post_type: axon_core::PostType::Instruction,
                metrics: None,
                created_at: task.created_at,
            };

            let _ = daemon.storage.save_task(&task);
            let _ = daemon.storage.save_thread(&thread);
            let _ = daemon.storage.save_post(&post);
            
            let _ = daemon.dispatcher.enqueue_task(task);

            // Signal thread creation
            daemon.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: self.project_id.clone(),
                thread_id: Some(task_id),
                agent_id: None,
                event_type: axon_core::EventType::ThreadCreated,
                source: "bootstrap".to_string(),
                content: format!("New work thread created for {}", comp.name),
                payload: None,
                timestamp: chrono::Local::now(),
            });
        }

        tracing::info!("🚀 Bootstrap V3 complete. Factory is OPERATIONAL.");
        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            thread_id: None,
            agent_id: None,
            event_type: axon_core::EventType::SystemLog,
            source: "bootstrap".to_string(),
            content: "🚀 Bootstrap complete. Factory is now OPERATIONAL.".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        Ok(())
    }

    pub async fn run_v2(&self, daemon: &Daemon, spec_content: String) -> anyhow::Result<()> {
        tracing::info!("Starting Bootstrap V2 for project '{}'...", self.project_id);

        let spec_truncated = if spec_content.len() > 8000 {
            format!("{}... [TRUNCATED DUE TO SIZE LIMIT]", &spec_content[..8000])
        } else {
            spec_content.clone()
        };

        let task = axon_core::Task {
            id: "bootstrap-task-001".to_string(),
            project_id: self.project_id.clone(),
            title: format!("Generate Master Hub Architecture for {}", self.project_id),
            description: format!(
                "OBJECTIVE: Generate architecture.md for project '{}'.\n\n\
                 --- SPEC CONTENT ---\n\
                 {}",
                self.project_id,
                spec_truncated
            ),
            status: TaskStatus::Pending,
            result: None,
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
                            status: TaskStatus::Pending,
                            result: None,
                            created_at: chrono::Local::now(),
                        };
                        let _ = daemon.storage.save_task(&task);
                        let _ = daemon.dispatcher.enqueue_task(task);
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
    
    fn select_best_agent(&self, role: axon_core::AgentRole) -> (Arc<dyn axon_model::ModelDriver + Send + Sync>, String, String) {
        let (models, names) = match role {
            axon_core::AgentRole::Junior => (&self.junior_models, &self.junior_model_names),
            axon_core::AgentRole::Senior => (&self.senior_models, &self.senior_model_names),
            axon_core::AgentRole::Architect => return (self.architect_model.clone(), "architect-agent-1".to_string(), self.architect_model_name.clone()),
        };

        if models.is_empty() {
            // Should not happen due to check in handle_assignment
            return (self.architect_model.clone(), "unknown".to_string(), self.architect_model_name.clone());
        }

        let stats_lock = self.agent_stats.lock().unwrap();
        let params_lock = self.routing_params.lock().unwrap();
        
        let best_idx = (0..models.len())
            .min_by(|&a, &b| {
                let id_a = format!("{}-agent-{}", match role {
                    axon_core::AgentRole::Junior => "junior",
                    axon_core::AgentRole::Senior => "senior",
                    _ => "agent"
                }, a + 1);
                let id_b = format!("{}-agent-{}", match role {
                    axon_core::AgentRole::Junior => "junior",
                    axon_core::AgentRole::Senior => "senior",
                    _ => "agent"
                }, b + 1);
                
                let score_a = stats_lock.get(&id_a).map(|s| s.score(&params_lock)).unwrap_or(f64::INFINITY);
                let score_b = stats_lock.get(&id_b).map(|s| s.score(&params_lock)).unwrap_or(f64::INFINITY);
                
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(0);

        let id = format!("{}-agent-{}", match role {
            axon_core::AgentRole::Junior => "junior",
            axon_core::AgentRole::Senior => "senior",
            _ => "agent"
        }, best_idx + 1);

        (models[best_idx].clone(), id, names[best_idx].clone())
    }

    fn record_agent_success(&self, id: &str, latency: f64) {
        let mut stats_lock = self.agent_stats.lock().unwrap();
        let stats = stats_lock.entry(id.to_string()).or_default();
        stats.record_success(latency);
        
        // PHASE_09: Persist to storage
        let latencies_json = serde_json::to_string(&stats.latencies).unwrap_or_default();
        let _ = self.storage.save_agent_stats(id, stats.success_count, stats.fail_count, &latencies_json);
    }

    fn record_agent_fail(&self, id: &str) {
        let mut stats_lock = self.agent_stats.lock().unwrap();
        let stats = stats_lock.entry(id.to_string()).or_default();
        stats.record_fail();
        
        // PHASE_09: Persist to storage
        let latencies_json = serde_json::to_string(&stats.latencies).unwrap_or_default();
        let _ = self.storage.save_agent_stats(id, stats.success_count, stats.fail_count, &latencies_json);
    }

    fn update_routing_params(&self) {
        let stats_lock = self.agent_stats.lock().unwrap();
        let mut params_lock = self.routing_params.lock().unwrap();

        let total_fail: usize = stats_lock.values().map(|s| s.fail_count).sum();
        let total_success: usize = stats_lock.values().map(|s| s.success_count).sum();

        if total_success == 0 {
            return;
        }

        let fail_ratio = total_fail as f64 / total_success as f64;
        
        tracing::info!("🔄 [FEEDBACK LOOP] Analysis: Success={}, Fail={}, Ratio={:.2}", total_success, total_fail, fail_ratio);

        // PHASE_10: Adaptive Scaling of Penalty
        if fail_ratio > 0.3 {
            params_lock.fail_penalty *= 1.2;
            tracing::warn!("🛡️ [FEEDBACK LOOP] High failure ratio! Increasing fail_penalty to {:.0}", params_lock.fail_penalty);
        } else if fail_ratio < 0.1 {
            params_lock.fail_penalty *= 0.9;
            tracing::info!("🍀 [FEEDBACK LOOP] System stable. Relaxing fail_penalty to {:.0}", params_lock.fail_penalty);
        }

        // Clamp values to prevent runaway inflation/deflation
        params_lock.fail_penalty = params_lock.fail_penalty.clamp(500.0, 5000.0);
    }
    
    async fn abort_with_failure(&self, task: &mut axon_core::Task, failures: Vec<String>, path: Vec<(String, String)>, metrics_list: Vec<axon_core::RuntimeMetrics>, agent_metrics: Vec<axon_core::AgentMetric>, start_total: std::time::Instant, worker_id: usize) -> anyhow::Result<()> {
        task.status = axon_core::TaskStatus::Failed;
        task.result = Some(failures.join("\n"));
        let _ = self.storage.save_task(task);

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

        self.event_bus.publish(axon_core::Event {
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
}
