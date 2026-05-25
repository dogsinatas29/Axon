use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use axon_core::{AgentRole, Event, EventType, EventLevel, Task, Thread, Post, PostType, DecomposedTask};
use axon_core::events::EventBus;
use axon_agent::AgentRuntime;
use axon_model::{ModelDriver, MockDriver, OllamaDriver, GeminiDriver, ClaudeDriver, OpenAIDriver};
use axon_storage::Storage;
use crate::{AxonConfig, PendingApproval};

pub(crate) fn create_model_driver(cfg: &crate::AgentConfig) -> Arc<dyn ModelDriver + Send + Sync> {
    match cfg.runtime.as_str() {
        "cloud" => {
            let provider = cfg.provider.as_deref().unwrap_or("gemini");
            let key = match provider {
                "gemini" => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
                "claude" => std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                "openai" => std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                _ => String::new(),
            };
            match provider {
                "gemini" => Arc::new(GeminiDriver::new(key, cfg.model.clone())),
                "claude" => Arc::new(ClaudeDriver::new(key, cfg.model.clone())),
                "openai" => Arc::new(OpenAIDriver::new(key, cfg.model.clone())),
                _ => Arc::new(MockDriver),
            }
        }
        "local" => {
            let endpoint = cfg.endpoint.as_deref().unwrap_or("http://127.0.0.1:11434");
            Arc::new(OllamaDriver::new(endpoint.to_string(), cfg.model.clone()))
        }
        _ => Arc::new(MockDriver),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootstrapStage {
    SpecAnalysis,
    Skeleton,
    ImplGen,
    Complete,
}

pub struct BootstrapManager {
    pub project_id: String,
    pub sandbox_root: PathBuf,
    pub config: AxonConfig,
    pub storage: Arc<Storage>,
    pub event_bus: Arc<EventBus>,
    pub architect_runtime: AgentRuntime,
    pub pending_approval: Option<Arc<Mutex<Option<PendingApproval>>>>,
}

impl BootstrapManager {
    pub fn new(config: AxonConfig, spec_path: &str) -> Result<Self, String> {
        let storage = Arc::new(Storage::new("runtime/state.db").map_err(|e| e.to_string())?);
        let event_bus = Arc::new(EventBus::new(256));
        Self::with_shared_state(config, spec_path, storage, event_bus, None)
    }

    pub fn with_shared_state(
        config: AxonConfig,
        spec_path: &str,
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        pending_approval: Option<Arc<Mutex<Option<PendingApproval>>>>,
    ) -> Result<Self, String> {
        let project_id = std::path::Path::new(spec_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default-project")
            .to_string();

        let sandbox_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&project_id);
        std::fs::create_dir_all(&sandbox_root).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(".axon/personas").map_err(|e| e.to_string())?;

        let architect_driver = create_model_driver(&config.agents.architect);
        let architect_runtime = AgentRuntime::new(
            "architect-agent-001".to_string(),
            AgentRole::Architect,
            config.agents.architect.model.clone(),
            architect_driver,
        )
        .with_timeout(1200)
        .with_project(project_id.clone());

        Ok(Self {
            project_id,
            sandbox_root,
            config,
            storage,
            event_bus,
            architect_runtime,
            pending_approval,
        })
    }

    pub async fn run_v3(&self, spec_content: String) -> Result<(), String> {
        tracing::info!("🚀 BootstrapManager state machine started for project '{}'", self.project_id);

        let mut stage = BootstrapStage::SpecAnalysis;
        let mut ir_opt: Option<axon_core::ir::ProjectIR> = None;
        let mut attempts = 0;
        let max_retries = 3;

        loop {
            tracing::info!("🏭 Stage: {:?}", stage);

            match stage {
                BootstrapStage::SpecAnalysis => {
                    match self.run_spec_analysis(&spec_content).await {
                        Ok(()) => {
                            stage = BootstrapStage::Skeleton;
                            attempts = 0;
                        }
                        Err(e) => {
                            attempts += 1;
                            if attempts >= max_retries {
                                return Err(format!("SpecAnalysis failed after {} retries: {}", max_retries, e));
                            }
                            tracing::warn!("⚠️ SpecAnalysis failed (attempt {}/{}): {}", attempts, max_retries, e);
                        }
                    }
                }

                BootstrapStage::Skeleton => {
                    match self.run_skeleton(&spec_content).await {
                        Ok(ir) => {
                            ir_opt = Some(ir);
                            stage = BootstrapStage::ImplGen;
                            attempts = 0;
                        }
                        Err(e) => {
                            attempts += 1;
                            if attempts >= max_retries {
                                return Err(format!("Skeleton failed after {} retries: {}", max_retries, e));
                            }
                            tracing::warn!("⚠️ Skeleton failed (attempt {}/{}): {}", attempts, max_retries, e);
                        }
                    }
                }

                BootstrapStage::ImplGen => {
                    if let Some(ref ir) = ir_opt {
                        match self.run_impl_gen(ir).await {
                            Ok(()) => {
                                stage = BootstrapStage::Complete;
                            }
                            Err(e) => {
                                attempts += 1;
                                if attempts >= max_retries {
                                    return Err(format!("ImplGen failed after {} retries: {}", max_retries, e));
                                }
                                tracing::warn!("⚠️ ImplGen failed (attempt {}/{}): {}", attempts, max_retries, e);
                            }
                        }
                    } else {
                        return Err("ImplGen: No IR available from Skeleton stage".to_string());
                    }
                }

                BootstrapStage::Complete => {
                    tracing::info!("✅ Bootstrap complete for project '{}'", self.project_id);
                    self.publish_event(EventType::SystemLog, "Bootstrap pipeline completed successfully.");
                    return Ok(());
                }
            }
        }
    }

    async fn run_spec_analysis(&self, spec_content: &str) -> Result<(), String> {
        tracing::info!("🔍 SpecAnalysis: Extracting immutable constraints...");
        self.publish_event(EventType::SystemLog, "SpecAnalysis: Analyzing specification...");

        let constraints = self.architect_runtime
            .process_spec_analysis(spec_content, Some(self.event_bus.clone()))
            .await
            .map_err(|e| format!("LLM SpecAnalysis failed: {}", e))?;

        let constraints_path = self.sandbox_root.join("immutable_constraints.json");
        let json = serde_json::to_string_pretty(&constraints)
            .map_err(|e| format!("Failed to serialize constraints: {}", e))?;
        std::fs::write(&constraints_path, &json)
            .map_err(|e| format!("Failed to write constraints: {}", e))?;
        tracing::info!("✅ Immutable constraints written to {:?}", constraints_path);

        // Boss approval gate — file + shared state (no stdin)
        let approval_file = self.sandbox_root.join(".axon_approval_pending");
        let approval_info = serde_json::json!({
            "status": "PENDING_BOSS_APPROVAL",
            "message": "Spec analysis complete. Review constraints and approve to continue.",
            "ambiguity_detected": constraints.ambiguity_detected,
            "approved": false
        });
        std::fs::write(&approval_file, serde_json::to_string_pretty(&approval_info).unwrap_or_default())
            .map_err(|e| format!("Failed to write approval file: {}", e))?;
        tracing::info!("⏳ Boss approval pending via file: {:?}", approval_file);

        // Share pending approval state with HTTP API via Arc<Mutex>
        if let Some(ref approval_mutex) = self.pending_approval {
            let mut approval = approval_mutex.lock().unwrap();
            *approval = Some(PendingApproval {
                project_id: self.project_id.clone(),
                constraints_path: constraints_path.to_string_lossy().to_string(),
                ambiguity_detected: constraints.ambiguity_detected,
                components: constraints.components.iter().map(|c| c.name.clone()).collect(),
                approved: false,
                rejected: false,
            });
        }

        let approved = poll_approval_file(&approval_file, self.pending_approval.clone()).await?;

        if !approved {
            if let Some(ref approval_mutex) = self.pending_approval {
                *approval_mutex.lock().unwrap() = None;
            }
            return Err("Boss rejected the specification analysis.".to_string());
        }

        // Clear pending approval after approval
        if let Some(ref approval_mutex) = self.pending_approval {
            *approval_mutex.lock().unwrap() = None;
        }

        tracing::info!("✅ SpecAnalysis approved.");
        self.publish_event(EventType::ApprovalGranted, "Spec analysis constraints approved by Boss.");
        Ok(())
    }

    async fn run_skeleton(&self, spec_content: &str) -> Result<axon_core::ir::ProjectIR, String> {
        tracing::info!("🏗️ Skeleton: Generating ProjectIR from spec...");
        self.publish_event(EventType::SystemLog, "Skeleton: Generating architecture IR...");

        let constraints_path = self.sandbox_root.join("immutable_constraints.json");
        let constraints: Option<axon_core::spec::ImmutableConstraints> = if constraints_path.exists() {
            std::fs::read_to_string(&constraints_path).ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        } else {
            None
        };

        let ir = self.architect_runtime
            .generate_ir_with_context(spec_content, None, constraints.as_ref(), 16384, Some(self.event_bus.clone()))
            .await
            .map_err(|e| format!("LLM Skeleton generation failed: {}", e))?;

        // Write architecture.md
        let arch_md = self.architect_runtime
            .generate_architecture_from_ir(&ir, Some(self.event_bus.clone()))
            .await
            .map_err(|e| format!("Failed to generate architecture.md: {}", e))?;

        let arch_path = self.sandbox_root.join("architecture.md");
        std::fs::write(&arch_path, &arch_md)
            .map_err(|e| format!("Failed to write architecture.md: {}", e))?;

        // Generate CMakeLists.txt from dep graph
        let mut graph = crate::dep_graph::DepGraph::new();
        graph.build_from_ir(&serde_json::to_value(&ir).unwrap_or_default());
        let cmake_content = graph.generate_cmake(&self.project_id, &self.config.locale, &self.sandbox_root);
        let cmake_path = self.sandbox_root.join("CMakeLists.txt");
        std::fs::write(&cmake_path, &cmake_content)
            .map_err(|e| format!("Failed to write CMakeLists.txt: {}", e))?;

        tracing::info!("✅ Architecture written to {:?}, CMakeLists.txt to {:?}", arch_path, cmake_path);
        self.publish_event(EventType::ArtifactCreated, &format!("Architecture IR generated: {:?}", arch_path));

        Ok(ir)
    }

    async fn run_impl_gen(&self, ir: &axon_core::ir::ProjectIR) -> Result<(), String> {
        tracing::info!("📝 ImplGen: Decomposing IR into tasks...");
        self.publish_event(EventType::SystemLog, "ImplGen: Decomposing architecture into implementation tasks...");

        let comps: Vec<&axon_core::ir::Component> = ir.components.values().collect();

        if comps.is_empty() {
            let ir_json = serde_json::to_string(ir)
                .map_err(|e| format!("Failed to serialize IR: {}", e))?;

            let post = self.architect_runtime
                .process_bootstrap_step2_with_context(&ir_json, 8192, Some(self.event_bus.clone()))
                .await
                .map_err(|e| format!("LLM task decomposition failed: {}", e))?;

            let content = post.content.trim().to_string();
            let tasks_json = extract_json_block(&content).unwrap_or(content);

            let d_tasks: Vec<DecomposedTask> = if let Ok(tasks) = serde_json::from_str::<Vec<DecomposedTask>>(&tasks_json) {
                tasks
            } else if let Ok(single) = serde_json::from_str::<DecomposedTask>(&tasks_json) {
                vec![single]
            } else {
                tracing::warn!("⚠️ Could not parse decomposed tasks from LLM response. Raw: {}", &tasks_json[..tasks_json.len().min(200)]);
                return Err("Failed to parse task decomposition from LLM response.".to_string());
            };

            Self::save_tasks_from_decomposed(&self.storage, &self.project_id, &self.event_bus, d_tasks).await?;
        } else {
            tracing::info!("🏗️ Deterministic task extraction from {} IR components", comps.len());
            let mut created_count = 0;

            for comp in comps {
                let target_file = &comp.file_path;
                if target_file.is_empty() {
                    continue;
                }

                let task_id = format!("task_{:03}", created_count + 1);
                let is_header = target_file.ends_with(".h") || target_file.ends_with(".hpp");
                let title = format!("Implement {}", target_file);
                let description = format!("Implement the {} module", target_file);

                let task = Task {
                    id: task_id.clone(),
                    project_id: self.project_id.clone(),
                    title,
                    description,
                    status: axon_core::TaskStatus::Pending,
                    dependencies: Vec::new(),
                    result: None,
                    target_file: Some(target_file.clone()),
                    lock_files: Vec::new(),
                    error_feedback: None,
                    senior_comment: None,
                    rework_count: 0,
                    base_hash: None,
                    parent_task: None,
                    reason: None,
                    kind: if target_file.ends_with(".c") || target_file.ends_with(".h") { "c".to_string() }
                          else if target_file.ends_with(".rs") { "rust".to_string() }
                          else if target_file.ends_with(".py") { "python".to_string() }
                          else { "c".to_string() },
                    retries: 0,
                    assigned_worker: None,
                    created_at: chrono::Local::now(),
                    ir_path: None,
                    task_kind: if is_header { Some(axon_core::LanguageTaskKind::C(axon_core::CTaskKind::HeaderDecl)) }
                               else { Some(axon_core::LanguageTaskKind::C(axon_core::CTaskKind::SourceImpl)) },
                    signature: None,
                    validator_rejections: 0,
                    senior_rejections: 0,
                    architecture_rejections: 0,
                    cargo_rejections: 0,
                    lsp_rejections: 0,
                    boss_interventions: 0,
                    lifecycle_state: axon_core::TaskLifecycleState::Queued,
                    patch_contract: None,
                    repair_mode: None,
                    repair_origin: None,
                };

                let thread = Thread {
                    id: task_id.clone(),
                    project_id: self.project_id.clone(),
                    title: task.title.clone(),
                    status: axon_core::ThreadStatus::Draft,
                    author: "Architect".to_string(),
                    milestone_id: None,
                    task_kind: task.task_kind,
                    rejection_count: 0,
                    validator_rejections: 0,
                    senior_rejections: 0,
                    architecture_rejections: 0,
                    cargo_rejections: 0,
                    lsp_rejections: 0,
                    error_feedback: None,
                    reason: None,
                    created_at: chrono::Local::now(),
                    updated_at: chrono::Local::now(),
                };

                let new_post = Post {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: task_id.clone(),
                    author_id: "Architect".to_string(),
                    content: task.description.clone(),
                    thought: None,
                    full_code: None,
                    post_type: PostType::Instruction,
                    metrics: None,
                    created_at: chrono::Local::now(),
                };

                self.storage.save_task(task).await
                    .map_err(|e| format!("Failed to save task: {}", e))?;
                self.storage.save_thread(thread).await
                    .map_err(|e| format!("Failed to save thread: {}", e))?;
                self.storage.save_post(new_post).await
                    .map_err(|e| format!("Failed to save post: {}", e))?;

                created_count += 1;
            }

            tracing::info!("✅ Created {} tasks from IR component decomposition.", created_count);
            self.publish_event(EventType::SystemLog, &format!("Created {} tasks from IR components", created_count));
        }

        self.publish_event(EventType::SystemLog, &format!("ImplGen complete. Tasks saved to storage."));
        Ok(())
    }

    async fn save_tasks_from_decomposed(
        storage: &Arc<Storage>,
        project_id: &str,
        event_bus: &Arc<EventBus>,
        d_tasks: Vec<DecomposedTask>,
    ) -> Result<(), String> {
        let mut created_count = 0;
        for dt in d_tasks {
            let is_header = dt.component_id.as_ref()
                .map(|f| f.ends_with(".h"))
                .unwrap_or(false);

            let task = Task::from_decomposed(dt.clone(), project_id.to_string());
            let thread = Thread {
                id: task.id.clone(),
                project_id: task.project_id.clone(),
                title: task.title.clone(),
                status: axon_core::ThreadStatus::Draft,
                author: "Architect".to_string(),
                milestone_id: None,
                task_kind: if is_header { Some(axon_core::LanguageTaskKind::C(axon_core::CTaskKind::HeaderDecl)) }
                           else { Some(axon_core::LanguageTaskKind::C(axon_core::CTaskKind::SourceImpl)) },
                rejection_count: 0,
                validator_rejections: 0,
                senior_rejections: 0,
                architecture_rejections: 0,
                cargo_rejections: 0,
                lsp_rejections: 0,
                error_feedback: None,
                reason: None,
                created_at: chrono::Local::now(),
                updated_at: chrono::Local::now(),
            };
            let new_post = Post {
                id: uuid::Uuid::new_v4().to_string(),
                thread_id: task.id.clone(),
                author_id: "Architect".to_string(),
                content: task.description.clone(),
                thought: None,
                full_code: None,
                post_type: PostType::Instruction,
                metrics: None,
                created_at: chrono::Local::now(),
            };

            storage.save_task(task.clone()).await
                .map_err(|e| format!("Failed to save task: {}", e))?;
            storage.save_thread(thread).await
                .map_err(|e| format!("Failed to save thread: {}", e))?;
            storage.save_post(new_post).await
                .map_err(|e| format!("Failed to save post: {}", e))?;

            event_bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                thread_id: None,
                agent_id: Some("architect-agent-001".to_string()),
                event_type: EventType::ThreadCreated,
                level: EventLevel::Info,
                source: "bootstrap".to_string(),
                content: format!("Task '{}' created for target: {:?}", task.title, task.target_file),
                payload: None,
                timestamp: chrono::Local::now(),
            });
            created_count += 1;
        }
        tracing::info!("✅ Created {} tasks from spec decomposition.", created_count);
        Ok(())
    }

    fn publish_event(&self, event_type: EventType, content: &str) {
        self.event_bus.publish(Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            thread_id: None,
            agent_id: Some("architect-agent-001".to_string()),
            event_type,
            level: EventLevel::Info,
            source: "bootstrap".to_string(),
            content: content.to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });
    }
}

async fn poll_approval_file(
    approval_file: &std::path::Path,
    pending_approval: Option<Arc<Mutex<Option<PendingApproval>>>>,
) -> Result<bool, String> {
    let mut poll_interval = tokio::time::interval(std::time::Duration::from_millis(500));
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(3600);

    loop {
        poll_interval.tick().await;
        if start.elapsed() > timeout {
            return Err("Boss approval timed out after 1 hour.".to_string());
        }

        // Check shared state first (faster path via API)
        if let Some(ref approval_mutex) = pending_approval {
            let approval = approval_mutex.lock().unwrap();
            if let Some(ref p) = *approval {
                if p.approved {
                    let _ = std::fs::remove_file(approval_file);
                    return Ok(true);
                }
                if p.rejected {
                    let _ = std::fs::remove_file(approval_file);
                    return Ok(false);
                }
            }
        }

        // Fallback: check file directly
        if let Ok(content) = std::fs::read_to_string(approval_file) {
            if let Ok(approval) = serde_json::from_str::<serde_json::Value>(&content) {
                if approval["approved"].as_bool().unwrap_or(false) {
                    let _ = std::fs::remove_file(approval_file);
                    return Ok(true);
                }
                if approval["status"].as_str() == Some("REJECTED") {
                    let _ = std::fs::remove_file(approval_file);
                    return Ok(false);
                }
            }
        }
    }
}

fn extract_json_block(text: &str) -> Option<String> {
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return Some(text[start..=end].to_string());
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Some(text[start..=end].to_string());
        }
    }
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_string());
        }
    }
    None
}
