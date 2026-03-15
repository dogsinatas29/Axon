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
    pub model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
    pub event_bus: Arc<events::EventBus>,
    pub architecture_guide: String,
    pub pause_tx: Arc<tokio::sync::watch::Sender<bool>>,
    pub pause_rx: tokio::sync::watch::Receiver<bool>,
}

impl Daemon {
    pub fn new(
        storage: Arc<Storage>, 
        model: Arc<dyn axon_model::ModelDriver + Send + Sync>,
        worker_tx: mpsc::Sender<Assignment>,
        architecture_guide: String
    ) -> Self {
        let event_bus = Arc::new(events::EventBus::new(100));
        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        Self {
            dispatcher: Arc::new(Dispatcher::new(worker_tx)),
            storage,
            model,
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
            self.model.clone()
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

        // 2. SENIOR REVIEW (Mocked selection or use existing senior)
        let senior_runtime = axon_agent::AgentRuntime::new(
            "senior-agent-1".to_string(),
            axon_core::AgentRole::Senior,
            self.model.clone()
        );

        let review = senior_runtime.review_proposal(&task, &proposal).await?;
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
            self.model.clone()
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
