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
    pub approval_file_path: String,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum LlmProvider {
    Local { endpoint: String },
    Cloud { api_key_env: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    pub id: Option<String>,
    pub runtime: String,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    #[serde(default)]
    pub provider_type: Option<LlmProvider>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PersonaConfig {
    pub name: String,
    pub age: Option<u8>,
    pub gender: String,
    pub personality: String,
    pub speech_style: String,
    pub catchphrase: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentsConfig {
    pub architect: AgentConfig,
    pub seniors: Vec<AgentConfig>,
    pub juniors: Vec<AgentConfig>,
    #[serde(default)]
    pub personas: std::collections::HashMap<String, PersonaConfig>,
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
            ".",
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
            spec_path,
        ).await?;

        // v0.0.31.21: [RESUME_PHASE] Check project_state for completed phases
        let project_id = std::path::Path::new(spec_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default-project")
            .to_string();

        let resume_phase = match self.storage.get_project_state(&project_id) {
            Ok(Some((stage, status))) if status == "completed" => {
                if stage == "Phase3_Completed" {
                    tracing::info!("🔄 [RESUME] Phase 3 already completed for project '{}'. Starting pipeline from existing tasks...", project_id);
                    let mut pipeline = crate::pipeline::ExecutionPipeline::new(
                        self.config.axon_config.clone(),
                        self.storage.clone(),
                        self.event_bus.clone(),
                        project_id.clone(),
                        std::path::Path::new(spec_path).parent().unwrap_or_else(|| std::path::Path::new(".")).join(&project_id),
                        app_state.agent_pool.clone(),
                    )
                    .with_pending_reviews(app_state.pending_reviews.clone())
                    .with_task_semaphore(app_state.task_semaphore.clone());
                    pipeline.run_background();

                    pending::<()>().await;
                    return Ok(());
                } else if stage == "Phase2_Completed" {
                    tracing::info!("🔄 [RESUME] Phase 2 completed for project '{}'. Resuming from Phase 3...", project_id);
                    2
                } else if stage == "Phase1_Completed" {
                    tracing::info!("🔄 [RESUME] Phase 1 completed for project '{}'. Resuming from Phase 2...", project_id);
                    1
                } else {
                    tracing::info!("🔄 [RESUME] Unknown stage '{}', starting full bootstrap", stage);
                    0
                }
            }
            _ => 0,
        };

        let manager = bootstrap::BootstrapManager::with_shared_state(
            self.config.axon_config.clone(),
            spec_path,
            self.storage.clone(),
            self.event_bus.clone(),
            Some(app_state.pending_approval.clone()),
        )?
        .with_resume_phase(resume_phase);

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
            app_state.agent_pool.clone(),
        )
        .with_pending_reviews(app_state.pending_reviews.clone())
        .with_task_semaphore(app_state.task_semaphore.clone());
        pipeline.run_background();

        pending::<()>().await;
        Ok(())
    }
}
