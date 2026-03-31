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
        }
    }

    pub async fn run(&self, mut worker_rx: mpsc::Receiver<Assignment>) -> anyhow::Result<()> {
        tracing::info!("AXON Daemon starting...");
        
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
        let junior_runtime = axon_agent::AgentRuntime::new(
            junior_id.clone(),
            axon_core::AgentRole::Junior,
            self.junior_model.clone()
        );

        let proposal = junior_runtime.process_task(&task, &self.architecture_guide).await?;
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
        let summary = junior_runtime.generate_system_summary(&proposal).await?;
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
        let senior_runtime = axon_agent::AgentRuntime::new(
            "senior-agent-1".to_string(),
            axon_core::AgentRole::Senior,
            self.senior_model.clone()
        );

        let review = senior_runtime.review_proposal(&task, &proposal, Some(&summary)).await?;
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
        let architect_runtime = axon_agent::AgentRuntime::new(
            "architect-agent-1".to_string(),
            axon_core::AgentRole::Architect,
            self.architect_model.clone()
        );

        let validation = architect_runtime.validate_architecture(&task, &review, &self.architecture_guide).await?;
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
            title: "Generate Core Architecture & Task Breakdown".to_string(),
            description: format!(
                "Read the following specification and generate two outputs:\n\
                 1. A comprehensive 'architecture.md' file content.\n\
                 2. A JSON array of initial tasks to be executed by Juniors and Seniors. Each task must have: 'title', 'description'.\n\n\
                 --- SPEC CONTENT ---\n\
                 {}",
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
            let architect_runtime = axon_agent::AgentRuntime::new(
                assignment.agent_id.clone(),
                axon_core::AgentRole::Architect,
                daemon.architect_model.clone()
            );

            tracing::info!("Architect is analyzing spec and breaking down tasks...");
            match architect_runtime.process_task(&assignment.task, "SYSTEM_BOOTSTRAP_PROTOCOL").await {
                Ok(proposal) => {
                    // 1. Architecture.md Generation
                    if let Some(ref arch_content) = proposal.full_code {
                        let _ = std::fs::write("architecture.md", arch_content);
                        tracing::info!("✅ Architecture.md has been generated.");
                    }

                    // 2. Intelligent Spec Breakdown (JSON extraction from content)
                    let content = &proposal.content;
                    if let Some(json_start) = content.find("[") {
                        if let Some(json_end) = content.rfind("]") {
                            let json_str = &content[json_start..=json_end];
                            if let Ok(tasks_raw) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
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
                                    daemon.dispatcher.enqueue_task(task);
                                }
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
