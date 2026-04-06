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

#[derive(Clone)]
pub struct Daemon {
    pub dispatcher: Arc<Dispatcher>,
    pub storage: Arc<Storage>,
    pub architect_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
    pub senior_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
    pub junior_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
    pub event_bus: Arc<events::EventBus>,
    pub architecture_guide: String,
    pub pause_tx: Arc<tokio::sync::watch::Sender<bool>>,
    pub pause_rx: tokio::sync::watch::Receiver<bool>,
    pub locale: String, // v0.0.15: OS Locale (e.g., "ko_KR", "en_US")
}

impl Daemon {
    pub fn new(
        storage: Arc<Storage>, 
        architect_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
        senior_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
        junior_model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
        worker_tx: mpsc::Sender<Assignment>,
        architecture_guide: String
    ) -> Self {
        let event_bus = Arc::new(events::EventBus::new(100));
        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        
        // LOCALE DETECTION (v0.0.15): 시스템 언어 설정을 파악하여 에이전트 페르소나에 주입
        let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
        tracing::info!("🌐 Detected System Locale: {}", locale);

        Self {
            dispatcher: Arc::new(Dispatcher::new(worker_tx)),
            storage,
            architect_model,
            senior_model,
            junior_model,
            event_bus,
            architecture_guide,
            pause_tx: Arc::new(pause_tx),
            pause_rx,
            locale,
        }
    }

    pub async fn run(&self, mut worker_rx: mpsc::Receiver<Assignment>) -> anyhow::Result<()> {
        tracing::info!("AXON Daemon starting...");
        
        // RECOVERY (v0.0.15): DB에서 처리되지 않은 태스크들을 불러와 스케줄러 큐에 재진입시킵니다.
        if let Ok(tasks) = self.storage.list_all_tasks() {
            let mut recovered_count = 0;
            for mut task in tasks {
                if task.status == TaskStatus::Pending || task.status == TaskStatus::InProgress {
                    // InProgress였던 것도 다시 Pending으로 돌려서 재할당 가능하게 함
                    task.status = TaskStatus::Pending;
                    let _ = self.storage.save_task(&task);
                    self.dispatcher.enqueue_task(task);
                    recovered_count += 1;
                }
            }
            if recovered_count > 0 {
                tracing::info!("♻️ Recovered {} unfinished tasks from database.", recovered_count);
            }
        }
        
        // Main orchestration loop
        let daemon = self.clone();
        let mut pause_rx = self.pause_rx.clone();
        
        loop {
            // Check if paused
            if *pause_rx.borrow() {
                tracing::debug!("Daemon is paused, waiting for resume...");
                if pause_rx.changed().await.is_err() {
                    break;
                }
                continue;
            }

            tokio::select! {
                Some(assignment) = worker_rx.recv() => {
                    let d = daemon.clone();
                    tokio::spawn(async move {
                        if let Err(e) = d.handle_assignment(assignment).await {
                            tracing::error!("Failed to handle assignment: {}", e);
                        }
                    });
                    
                    // THROTTLE (Phase 2): 4초 대기 (15 RPM 제한을 위한 전역 분배 간격 조율)
                    // 각 태스크 스폰 사이에 최소 물리적 지연을 두어 병렬 API 급발진 방지
                    tokio::time::sleep(tokio::time::Duration::from_millis(4100)).await;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    // Periodic scheduling check
                    // For now, assume fixed agents available
                    let available_agents = vec!["agent-gemini-1".to_string()];
                    let _ = daemon.dispatcher.schedule(available_agents).await;
                }
                Ok(_) = pause_rx.changed() => {
                    // Pause state changed, loop will restart and check at the top
                }
            }
        }
        Ok(())
    }

    async fn handle_assignment(&self, assignment: Assignment) -> anyhow::Result<()> {
        let mut task = assignment.task;
        let junior_id = assignment.agent_id;
        
        // 1. JUNIOR IMPLEMENTATION
        let mut junior_runtime = axon_agent::AgentRuntime::new(
            junior_id.clone(),
            axon_core::AgentRole::Junior,
            self.junior_model.clone()
        );
        // LOCALE INJECTION: 주니어에게 사장님의 언어로 보고할 것을 강제함
        junior_runtime.set_locale(&self.locale);

        let proposal = junior_runtime.process_task(&task, &self.architecture_guide, Some(self.event_bus.clone())).await?;
        let _ = self.storage.save_post(&proposal);
        
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

        // 2. SYSTEM SUMMARY (Intermediate step)
        let summary = junior_runtime.generate_system_summary(&proposal, Some(self.event_bus.clone())).await?;
        let _ = self.storage.save_post(&summary);

        self.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: task.project_id.clone(),
            thread_id: Some(task.id.clone()),
            agent_id: None,
            event_type: axon_core::EventType::SystemLog,
            source: "SYSTEM_SUMMARY".to_string(),
            content: "System generated objective summary for proposal".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        // 3. SENIOR REVIEW
        let mut senior_runtime = axon_agent::AgentRuntime::new(
            "senior-agent-1".to_string(),
            axon_core::AgentRole::Senior,
            self.senior_model.clone()
        );
        senior_runtime.set_locale(&self.locale);

        let review = senior_runtime.review_proposal(&task, &proposal, Some(&summary), Some(self.event_bus.clone())).await?;
        let _ = self.storage.save_post(&review);

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

        // 3. ARCHITECT VALIDATION
        let mut architect_runtime = axon_agent::AgentRuntime::new(
            "architect-agent-1".to_string(),
            axon_core::AgentRole::Architect,
            self.architect_model.clone()
        );
        architect_runtime.set_locale(&self.locale);

        let validation = architect_runtime.validate_architecture(&task, &review, &self.architecture_guide, Some(self.event_bus.clone())).await?;
        let _ = self.storage.save_post(&validation);

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

        // 4. FINALIZE TASK
        task.status = TaskStatus::Completed;
        let _ = self.storage.save_task(&task);

        self.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: task.project_id.clone(),
            thread_id: Some(task.id.clone()),
            agent_id: None,
            event_type: axon_core::EventType::ThreadCompleted,
            source: "daemon".to_string(),
            content: format!("Task {} successfully passed all reviews", task.id),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        Ok(())
    }

    pub async fn bootstrap_from_spec(&self, spec_content: String) -> anyhow::Result<()> {
        tracing::info!("Starting Architect-led bootstrapping from specification...");

        let task = axon_core::Task {
            id: "bootstrap-task-001".to_string(),
            project_id: "system".to_string(),
            title: "Generate Master Hub Architecture (Sovereign Protocol v0.2.21+)".to_string(),
            description: format!(
                "YOU ARE THE SYSTEM ARCHITECT. YOUR GOAL IS TO BOOTSTRAP THE PROJECT USING THE SOVEREIGN PROTOCOL (v0.2.21+).\n\n\
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
                self.locale,
                self.locale,
                spec_content
            ),
            status: TaskStatus::Pending,
            created_at: chrono::Local::now(),
        };

        let assignment = Assignment {
            task,
            agent_id: "architect-agent-001".to_string(),
        };

        let daemon = self.clone();
        tokio::spawn(async move {
            let mut architect_runtime = axon_agent::AgentRuntime::new(
                assignment.agent_id.clone(),
                axon_core::AgentRole::Architect,
                daemon.architect_model.clone()
            );
            architect_runtime.set_locale(&daemon.locale);

            tracing::info!("Architect is analyzing spec and breaking down tasks...");
            match architect_runtime.process_task(&assignment.task, "SYSTEM_BOOTSTRAP_PROTOCOL", Some(daemon.event_bus.clone())).await {
                Ok(proposal) => {
                    // 1. Architecture.md Generation
                    if let Some(ref arch_content) = proposal.full_code {
                        // Further refine: if there's a markdown block, extract it
                        let clean_arch = if let Some(start) = arch_content.find("```markdown") {
                            let remaining = &arch_content[start + 11..];
                            let end = remaining.find("```").unwrap_or(remaining.len());
                            let content = remaining[..end].trim().to_string();
                            let full_code = {
                                // Strip reasoning tags to get clean content
                                let mut clean = content.clone();
                                for tag in ["thought", "analysis", "reasoning", "evaluation"] {
                                    let start_tag = format!("<{}>", tag);
                                    let end_tag = format!("</{}>", tag);
                                    while let (Some(s), Some(e)) = (clean.find(&start_tag), clean.find(&end_tag)) {
                                        clean.replace_range(s..e + end_tag.len(), "");
                                    }
                                }
                                Some(clean.trim().to_string())
                            };
                            full_code.unwrap_or(content)
                        } else if let Some(start) = arch_content.find("# ") {
                           arch_content[start..].to_string()
                        } else {
                            arch_content.clone()
                        };

                        let _ = std::fs::write("architecture.md", clean_arch);
                        tracing::info!("✅ Architecture.md has been generated (Master Hub).");
                    }

                    // 2. Intelligent Spec Breakdown (Look for JSON block)
                    let content = &proposal.content;
                    let json_str = if let Some(start) = content.find("```json") {
                        let end = content[start+7..].find("```").unwrap_or(content.len() - start - 7);
                        Some(&content[start+7..start+7+end])
                    } else if let Some(start) = content.find("[") {
                        let end = content.rfind("]").unwrap_or(0);
                        if end > start { Some(&content[start..=end]) } else { None }
                    } else {
                        None
                    };

                    if let Some(json_str) = json_str {
                        if let Ok(tasks_raw) = serde_json::from_str::<Vec<serde_json::Value>>(json_str.trim()) {
                            tracing::info!("🔨 Architect proposed {} tasks from spec.", tasks_raw.len());
                            for t in tasks_raw {
                                let task = axon_core::Task {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    project_id: "default-project".to_string(),
                                    title: t["title"].as_str().unwrap_or("Untitled").to_string(),
                                    description: t["description"].as_str().unwrap_or("").to_string(),
                                    status: TaskStatus::Pending,
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
                                    created_at: task.created_at,
                                };
                                let _ = daemon.storage.save_post(&post);

                                daemon.dispatcher.enqueue_task(task);
                            }
                        }
                    }
                    let _ = daemon.storage.save_post(&proposal);
                    tracing::info!("🚀 Bootstrapping complete. AXON Factory is now OPERATIONAL.");
                }
                Err(e) => {
                    tracing::error!("Architect failed to bootstrap: {}", e);
                }
            }
        });

        Ok(())
    }

    pub fn lock_in_architecture(&self, thread_title: &str) -> anyhow::Result<()> {
        let arch_path = "architecture.md";
        if std::path::Path::new(arch_path).exists() {
            let content = std::fs::read_to_string(arch_path)?;
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
}
