/*
 * AXON - The Deterministic Governance Kernel
 */

// Legacy modules moved to quarantine, commented out for baseline
// pub mod admin;
// pub mod controller;
// pub mod debug_hook;

pub mod dep_graph;
pub mod execution_validator;
pub mod intelligence;
pub mod observability;
pub mod server;
pub mod governance;
pub mod events;
pub mod bootstrap;
pub mod pipeline;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::future::pending;
use serde::{Deserialize, Serialize};
use crate::events::EventBus;
use axon_storage::Storage;

#[derive(Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub project_id: String,
    pub constraints_path: String,
    pub ambiguity_detected: bool,
    pub components: Vec<String>,
    pub approved: bool,
    pub rejected: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PipelineReview {
    pub task_id: String,
    pub task: axon_core::Task,
    pub proposal: Option<axon_core::Post>,
    pub review: Option<axon_core::Post>,
    pub senior_feedback: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LspConfig {
    pub language: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    pub runtime: String,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentsConfig {
    pub architect: AgentConfig,
    pub seniors: Vec<AgentConfig>,
    pub juniors: Vec<AgentConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecutionConfig {
    pub review_queue_limit: usize,
    pub sampling_rate: f64,
    pub fallback_enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AxonConfig {
    pub locale: String,
    pub lsps: Option<Vec<LspConfig>>,
    pub agents: AgentsConfig,
    pub execution: ExecutionConfig,
}

impl AxonConfig {
    pub fn load(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: AxonConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}

pub struct KernelConfig {
    pub temp_dir: PathBuf,
    pub thread_count: usize,
    pub replay_seed: u64,
    pub feature_flags: BTreeSet<String>,
    pub axon_config: AxonConfig,
}

pub struct DeterministicKernel {
    pub config: KernelConfig,
    pub storage: Arc<Storage>,
    pub event_bus: Arc<EventBus>,
}

impl DeterministicKernel {
    pub fn new(config: KernelConfig, storage: Arc<Storage>, event_bus: Arc<EventBus>) -> Self {
        Self { config, storage, event_bus }
    }

    pub async fn run(&self) -> Result<(), String> {
        tracing::info!("==================================================");
        tracing::info!("🚀 AXON Deterministic Kernel Booting...");
        tracing::info!("==================================================");
        tracing::info!("📡 Boss Board: http://localhost:8080");
        tracing::info!("🌐 Language: {}", self.config.axon_config.locale);
        tracing::info!("🚀 Architect Model : {}", self.config.axon_config.agents.architect.model);
        for (i, s) in self.config.axon_config.agents.seniors.iter().enumerate() {
            tracing::info!("🚀 Senior {} Model : {}", i + 1, s.model);
        }
        for (i, j) in self.config.axon_config.agents.juniors.iter().enumerate() {
            tracing::info!("🚀 Junior {} Model : {}", i + 1, j.model);
        }
        tracing::info!("👥 Total Employed Junior Workers (Thread Pool): {}", self.config.axon_config.agents.juniors.len());
        tracing::info!("==================================================");

        crate::server::setup_ingress(
            self.config.axon_config.clone(),
            self.storage.clone(),
            self.event_bus.clone(),
        ).await?;

        pending::<()>().await;
        Ok(())
    }

    pub async fn start_with_spec(&self, spec_path: &str) -> Result<(), String> {
        tracing::info!("==================================================");
        tracing::info!("🚀 AXON Deterministic Kernel Booting with Spec: {}", spec_path);
        tracing::info!("==================================================");
        tracing::info!("📡 Boss Board: http://localhost:8080");
        tracing::info!("🌐 Language: {}", self.config.axon_config.locale);
        tracing::info!("🚀 Architect Model : {}", self.config.axon_config.agents.architect.model);
        for (i, s) in self.config.axon_config.agents.seniors.iter().enumerate() {
            tracing::info!("🚀 Senior {} Model : {}", i + 1, s.model);
        }
        for (i, j) in self.config.axon_config.agents.juniors.iter().enumerate() {
            tracing::info!("🚀 Junior {} Model : {}", i + 1, j.model);
        }
        tracing::info!("👥 Total Employed Junior Workers (Thread Pool): {}", self.config.axon_config.agents.juniors.len());
        tracing::info!("==================================================");

        let spec_content = std::fs::read_to_string(spec_path)
            .map_err(|e| format!("Failed to read spec file '{}': {}", spec_path, e))?;

        // Start HTTP server FIRST so web UI is available during bootstrap
        tracing::info!("Starting HTTP server before bootstrap pipeline...");
        let app_state = crate::server::setup_ingress(
            self.config.axon_config.clone(),
            self.storage.clone(),
            self.event_bus.clone(),
        ).await?;

        let manager = bootstrap::BootstrapManager::with_shared_state(
            self.config.axon_config.clone(),
            spec_path,
            self.storage.clone(),
            self.event_bus.clone(),
            Some(app_state.pending_approval.clone()),
        )?;

        manager.run_v3(spec_content).await?;
        tracing::info!("Bootstrap complete. Server continues serving.");

        // ⏳ Flush all pending WAL ops to storage before pipeline reads tasks
        tracing::info!("⏳ Synchronizing bootstrap state with durable disk storage...");
        self.storage.flush().await.map_err(|e| format!("Critical Storage Flush Failed: {}", e))?;
        tracing::info!("✅ WAL queue flushed. Launching pipeline safely.");

        // Start execution pipeline with shared pending_reviews
        let mut pipeline = crate::pipeline::ExecutionPipeline::new(
            self.config.axon_config.clone(),
            self.storage.clone(),
            self.event_bus.clone(),
            manager.project_id.clone(),
            manager.sandbox_root.clone(),
        )
        .with_pending_reviews(app_state.pending_reviews.clone());
        pipeline.run_background();

        pending::<()>().await;
        Ok(())
    }
}
