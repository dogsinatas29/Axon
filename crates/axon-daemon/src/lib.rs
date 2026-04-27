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
use axon_core::{events, TaskStatus};
use axon_dispatcher::{Dispatcher, Assignment};
use axon_storage::Storage;
use std::sync::Arc;
use tokio::sync::mpsc;
use std::collections::{HashMap, VecDeque};
use serde::Deserialize;

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

#[derive(Clone)]
pub struct Daemon {
    pub dispatcher: Arc<Dispatcher>,
    pub storage: Arc<Storage>,
    pub architect_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
    pub senior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
    pub junior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
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

impl Daemon {
    pub fn new(
        storage: Arc<Storage>, 
        architect_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
        senior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
        junior_models: Vec<Arc<dyn axon_model::ModelDriver + Send + Sync>>,
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
            senior_models,
            junior_models,
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

        // v0.0.16 Isolation
        let arch_guide_path = format!("{}/architecture.md", task.project_id);
        let current_arch_guide = std::fs::read_to_string(&arch_guide_path).unwrap_or_else(|_| self.architecture_guide.clone());

        let mut proposal = None;
        let mut summary = None;
        let num_juniors = self.junior_models.len();
        let mut junior_failures = Vec::new();

        'junior_fallback: for _ in 0..num_juniors {
            // PHASE_08: Adaptive Routing Selection
            let (junior_model, junior_id) = self.select_best_agent(axon_core::AgentRole::Junior);

            let mut junior_runtime = axon_agent::AgentRuntime::new(
                junior_id.clone(),
                axon_core::AgentRole::Junior,
                junior_model
            );
            junior_runtime.set_locale(&self.locale);
            junior_runtime.throttler = Some(self.throttler.clone());

            for retry_attempt in 0..=max_retries {
                let start_step = std::time::Instant::now();
                match junior_runtime.process_task(&task, &current_arch_guide, Some(self.event_bus.clone())).await {
                    Ok(p) => {
                        let latency = start_step.elapsed().as_secs_f64() * 1000.0;
                        agent_metrics.push(axon_core::AgentMetric {
                            id: junior_id.clone(),
                            role: "junior".to_string(),
                            status: "OK".to_string(),
                            latency_ms: latency,
                            attempts: (retry_attempt + 1) as u32,
                            error: None,
                        });

                        // SUCCESS: Post-processing within the same scope
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
            let (senior_model, senior_id) = self.select_best_agent(axon_core::AgentRole::Senior);

            let mut senior_runtime = axon_agent::AgentRuntime::new(
                senior_id.clone(),
                axon_core::AgentRole::Senior,
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
        if let Some(ref code) = review.full_code {
            let _ = self.sync_post_to_sandbox(&task.project_id, code);
        }

        // v0.0.16: 아키텍처 섹션 잠금 (격리 경로 적용)
        let _ = self.lock_in_architecture(&task.project_id, &task.title);

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

        tracing::info!("Starting Architect-led bootstrapping for project '{}' from specification...", project_id);

        let task = axon_core::Task {
            id: "bootstrap-task-001".to_string(),
            project_id: project_id.clone(),
            title: format!("Generate Master Hub Architecture for {}", project_id),
            description: format!(
                "YOU ARE THE SYSTEM ARCHITECT. YOUR GOAL IS TO BOOTSTRAP THE PROJECT '{}' USING THE SOVEREIGN PROTOCOL (v0.2.21+).\n\n\
                 --- LANGUAGE ENFORCEMENT ---\n\
                 YOU MUST COMMUNICATE AND GENERATE ALL CONTENT (ARCHITECTURE.MD, TASK TITLES, TASK DESCRIPTIONS) IN THE FOLLOWING LOCALE: {}.\n\n\
                 --- CRITICAL PROTOCOL ENFORCEMENT ---\n\
                 Follow the domain logic of the provided SPEC CONTENT, but the **Structure** MUST be overridden by the Sovereign Protocol v0.2.21+.\n\
                 You MUST demote the existing detailed systems (e.g., ECS, legacy architectures) to 'Node' level components, and design a new 'Hub' layer that governs them.\n\n\
                 --- STEP 1: DEEP ANALYSIS (COT) ---\n\
                 Analyze the provided specification in <thought> tags. Identify the Single Source of Truth (SSOT), authority boundaries (Hub -> Cluster -> Node), and modular specifications needed.\n\n\
                 --- STEP 2: MULTI-PERSPECTIVE EVALUATION (TOT) ---\n\
                 Evaluate at least three different architectural layouts in <evaluation> tags. Compare them based on 'Top-Down Design', 'Namespace Isolation', and 'Scalability'.\n\n\
                 --- STEP 3: MASTER HUB OUTPUT ---\n\
                 Generate the following two components:\n\
                 1. A 'Master Hub' architecture.md file content. This MUST strictly follow the 'Hub -> Cluster -> Node' hierarchical structure and define clear SSOT rules.\n\
                 2. A JSON array of initial tasks. Each task MUST include a 'title' and 'description' WRITTEN IN THE LOCALE: {}.\n\n\
                 --- SPEC CONTENT ---\n\
                 {}",
                project_id,
                self.locale,
                self.locale,
                spec_content
            ),
            status: TaskStatus::Pending,
            result: None,
            created_at: chrono::Local::now(),
        };

        let assignment = Assignment {
            task,
            agent_id: "architect-agent-001".to_string(),
        };

        let daemon = self.clone();
        let project_id_clone = project_id.clone();
        tokio::spawn(async move {
            let mut architect_runtime = axon_agent::AgentRuntime::new(
                assignment.agent_id.clone(),
                axon_core::AgentRole::Architect,
                daemon.architect_model.clone()
            ).with_timeout(600);
            architect_runtime.set_locale(&daemon.locale);

            tracing::info!("Stage 1: Architect is designing the Master Architecture...");
            match architect_runtime.process_bootstrap_step1(&assignment.task, Some(daemon.event_bus.clone())).await {
                Ok(arch_proposal) => {
                    // 1. Architecture.md Generation
                    let arch_content = &arch_proposal.content;
                    let clean_arch = if let Some(start) = arch_content.find("```markdown") {
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

                    let sandbox_path = project_id_clone.clone();
                    let _ = std::fs::create_dir_all(&sandbox_path);
                    let arch_file_path = format!("{}/architecture.md", sandbox_path);
                    let _ = std::fs::write(&arch_file_path, &clean_arch);
                    tracing::info!("✅ Architecture.md has been generated in: {}", arch_file_path);
                    let _ = daemon.storage.save_post(&arch_proposal);

                    // 2. Stage 2: Task Extraction
                    tracing::info!("Stage 2: Architect is extracting implementation tasks as RAW JSON...");
                    match architect_runtime.process_bootstrap_step2(&clean_arch, Some(daemon.event_bus.clone())).await {
                        Ok(task_proposal) => {
                            let json_str = task_proposal.content.trim();
                            
                            // Robust JSON extraction
                            let clean_json = if let Some(start) = json_str.find("```json") {
                                let end = json_str[start+7..].find("```").unwrap_or(json_str.len() - start - 7);
                                json_str[start+7..start+7+end].trim()
                            } else {
                                let start_arr = json_str.find("[");
                                let start_obj = json_str.find("{");
                                
                                match (start_arr, start_obj) {
                                    (Some(a), Some(o)) => {
                                        let start = a.min(o);
                                        let end = json_str.rfind(if a < o { "]" } else { "}" }).unwrap_or(json_str.len());
                                        json_str[start..=end].trim()
                                    }
                                    (Some(a), None) => {
                                        let end = json_str.rfind("]").unwrap_or(json_str.len());
                                        json_str[a..=end].trim()
                                    }
                                    (None, Some(o)) => {
                                        let end = json_str.rfind("}").unwrap_or(json_str.len());
                                        json_str[o..=end].trim()
                                    }
                                    _ => json_str.trim(),
                                }
                            };

                            let tasks_raw: Vec<serde_json::Value> = {
                                let mut deserializer = serde_json::Deserializer::from_str(clean_json);
                                match serde_json::Value::deserialize(&mut deserializer) {
                                    Ok(val) => {
                                        if val.is_array() {
                                            val.as_array().unwrap().clone()
                                        } else if let Some(tasks) = val.get("tasks").and_then(|t| t.as_array()) {
                                            tasks.clone()
                                        } else if val.is_object() {
                                            vec![val]
                                        } else {
                                            Vec::new()
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ [JSON_PARSE_ERROR]: {}", e);
                                        Vec::new()
                                    }
                                }
                            };

                            if !tasks_raw.is_empty() {
                                tracing::info!("🔨 Architect proposed {} tasks for project '{}'.", tasks_raw.len(), project_id_clone);
                                for t in tasks_raw {
                                    let task = axon_core::Task {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        project_id: project_id_clone.clone(), 
                                        title: t["title"].as_str().unwrap_or("Untitled").to_string(),
                                        description: t["description"].as_str().unwrap_or("").to_string(),
                                        status: TaskStatus::Pending,
                                        result: None,
                                        created_at: chrono::Local::now(),
                                    };
                                    let _ = daemon.storage.save_task(&task);

                                    let thread = axon_core::Thread {
                                        id: task.id.clone(),
                                        project_id: task.project_id.clone(),
                                        title: task.title.clone(),
                                        status: axon_core::ThreadStatus::Draft,
                                        author: "Architect".to_string(),
                                        milestone_id: None,
                                        created_at: task.created_at,
                                        updated_at: task.created_at,
                                    };
                                    let _ = daemon.storage.save_thread(&thread);

                                    let post = axon_core::Post {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        thread_id: task.id.clone(),
                                        author_id: "Architect".to_string(),
                                        content: task.description.clone(),
                                        full_code: None,
                                        post_type: axon_core::PostType::Instruction,
                                        metrics: None,
                                        created_at: task.created_at,
                                    };
                                    let _ = daemon.storage.save_post(&post);

                                    let _ = daemon.dispatcher.enqueue_task(task);
                                }
                                tracing::info!("🚀 Bootstrapping complete. AXON Factory is now OPERATIONAL.");
                            }
                        }
                        Err(e) => tracing::error!("Stage 2 Extraction failed: {}", e),
                    }
                }
                Err(e) => tracing::error!("Stage 1 Design failed: {}", e),
            }
        });

        Ok(())
    }

    pub fn lock_in_architecture(&self, project_id: &str, thread_title: &str) -> anyhow::Result<()> {
        let arch_path = format!("{}/architecture.md", project_id);
        if std::path::Path::new(&arch_path).exists() {
            let content = std::fs::read_to_string(&arch_path)?;
            let locked_marker = format!("## {} [✅ Locked]", thread_title);
            let target = format!("## {}", thread_title);
            
            if content.contains(&target) && !content.contains(&locked_marker) {
                let new_content = content.replace(&target, &locked_marker);
                std::fs::write(arch_path, new_content)?;
                tracing::info!("Locked in architecture section: {}", thread_title);
            }
        }
        Ok(())
    }

    /// v0.0.16 Isolation System: 에이전트의 결과물을 프로젝트별 샌드박스로 동기화
    fn sync_post_to_sandbox(&self, project_id: &str, content: &str) -> anyhow::Result<()> {
        let sandbox_path = project_id.to_string();
        let _ = std::fs::create_dir_all(&sandbox_path);

        // v0.0.18: Language-agnostic code extraction
        let mut found_code = false;
        let mut current_pos = 0;

        while let Some(start_idx) = content[current_pos..].find("```") {
            let real_start = current_pos + start_idx;
            let lang_end = content[real_start + 3..].find('\n').unwrap_or(0);
            let lang = content[real_start + 3..real_start + 3 + lang_end].trim();
            
            let block_start = real_start + 3 + lang_end + 1;
            if let Some(end_idx) = content[block_start..].find("```") {
                let real_end = block_start + end_idx;
                let code_part = content[block_start..real_end].trim();
                
                // Try to detect filename from comments (e.g., # filename.py or // filename.rs)
                let mut filename = format!("generated_{}.{}", uuid::Uuid::new_v4().to_string()[..4].to_string(), if lang.is_empty() { "txt" } else { lang });
                
                for line in code_part.lines().take(5) {
                    let trimmed = line.trim();
                    if (trimmed.starts_with("#") || trimmed.starts_with("//")) && (trimmed.contains(".") || trimmed.contains("/")) {
                        let detected = trimmed.trim_start_matches('#').trim_start_matches('/').trim().split_whitespace().next().unwrap_or("");
                        if detected.contains('.') {
                            filename = detected.to_string();
                            break;
                        }
                    }
                }

                let file_path = if filename.contains('/') {
                    format!("{}/{}", sandbox_path, filename)
                } else {
                    format!("{}/src/{}", sandbox_path, filename)
                };

                if let Some(parent) = std::path::Path::new(&file_path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                if let Ok(_) = std::fs::write(&file_path, code_part) {
                    tracing::info!("🔒 Isolation: Synced {} code to {}", lang, file_path);
                    found_code = true;
                }
                
                current_pos = real_end + 3;
            } else {
                break;
            }
        }

        if !found_code {
            let file_path = format!("{}/README.md", sandbox_path);
            let _ = std::fs::write(&file_path, content);
            tracing::info!("🔒 Isolation: Synced documentation to {}", file_path);
        }

        Ok(())
    }
    
    fn select_best_agent(&self, role: axon_core::AgentRole) -> (Arc<dyn axon_model::ModelDriver + Send + Sync>, String) {
        let models = match role {
            axon_core::AgentRole::Junior => &self.junior_models,
            axon_core::AgentRole::Senior => &self.senior_models,
            axon_core::AgentRole::Architect => return (self.architect_model.clone(), "architect-agent-1".to_string()),
        };

        if models.is_empty() {
            // Should not happen due to check in handle_assignment
            return (self.architect_model.clone(), "unknown".to_string());
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

        (models[best_idx].clone(), id)
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

