use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use axon_core::{
    AgentRole, Event, EventLevel, EventType, Task, TaskLifecycleState,
    TaskStatus, ThreadStatus,
};
use axon_core::events::EventBus;
use axon_agent::AgentRuntime;
use axon_storage::Storage;
use crate::bootstrap::create_model_driver;
use crate::{AxonConfig, PipelineReview};

// Phase 7-C: Sandbox State Machine — tracks file lifecycle to prevent memory-loss retry loops
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    Clean,          // No sandbox file exists yet
    Proposed,       // Junior generated code, awaiting Senior review
    Rejected,       // Senior rejected → renamed to .failed, original_code preserved
    Contaminated,   // Parser failed → renamed to .failed, original_code preserved
    Promoted,       // Senior approved → moved to real_path
}

impl SandboxState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SandboxState::Clean => "CLEAN",
            SandboxState::Proposed => "PROPOSED",
            SandboxState::Rejected => "REJECTED",
            SandboxState::Contaminated => "CONTAMINATED",
            SandboxState::Promoted => "PROMOTED",
        }
    }
}

// Phase 7-B: Output Normalization Layer — single convergence point for all LLM output formats
#[derive(Debug, Clone)]
pub struct NormalizedOutput {
    pub decision: Option<String>,    // "APPROVE" or "REJECT" (Senior only)
    pub feedback: Option<String>,    // Senior feedback text
    pub code: Option<String>,        // Junior generated code
}

impl NormalizedOutput {
    pub fn is_approve(&self) -> bool {
        self.decision.as_deref() == Some("APPROVE")
    }
}

// normalize_output: scans raw LLM output and converges to NormalizedOutput
// Handles: JSON wrapper, markdown code blocks, C/C++ raw patterns, raw text fallback
pub fn normalize_output(raw: &str, is_senior_review: bool) -> NormalizedOutput {
    let trimmed = raw.trim();

    if is_senior_review {
        // Senior output: extract decision + feedback
        normalize_senior_output(trimmed)
    } else {
        // Junior output: extract code
        normalize_junior_output(trimmed)
    }
}

fn normalize_senior_output(raw: &str) -> NormalizedOutput {
    let trimmed = raw.trim();

    // Phase 8: Empty validator detection — silent failure is unsafe
    if trimmed.is_empty() || trimmed == "{}" || trimmed == "{ }" {
        tracing::error!("❌ [EMPTY_VALIDATOR] Senior returned empty response — hard reject");
        return NormalizedOutput {
            decision: Some("REJECT".to_string()),
            feedback: Some("[PATCH_TRUNCATED] Senior validator returned empty response — context collapse or malformed patch detected".to_string()),
            code: None,
        };
    }

    // Phase 8: Detect causal rejection JSON from Senior
    if trimmed.starts_with("{") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(status) = json.get("status").and_then(|v| v.as_str()) {
                if status == "PATCH_TRUNCATED" || status == "CONTEXT_COLLAPSE" {
                    let reason = json.get("root_cause").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let detected_by = json.get("detected_by").and_then(|v| v.as_str()).unwrap_or("senior-normalizer");
                    tracing::warn!("⚠️ [CAUSAL_REJECTION] status={} reason={} by={}", status, reason, detected_by);
                    return NormalizedOutput {
                        decision: Some("REJECT".to_string()),
                        feedback: Some(format!("[{}] {}", status, reason)),
                        code: None,
                    };
                }
            }
        }
    }

    // Tier 1: [APPROVE]/[REJECT] line matching
    let first_line = trimmed.lines().next().unwrap_or("").trim();
    if first_line.starts_with("[APPROVE]") {
        let feedback = trimmed.lines().skip(1).collect::<Vec<_>>().join("\n").trim().to_string();
        return NormalizedOutput {
            decision: Some("APPROVE".to_string()),
            feedback: if feedback.is_empty() { None } else { Some(feedback) },
            code: None,
        };
    }
    if first_line.starts_with("[REJECT]") {
        let mut fb = trimmed.lines().skip(1).collect::<Vec<_>>().join("\n").trim().to_string();
        if fb.is_empty() && first_line.len() > "[REJECT]".len() {
            fb = first_line["[REJECT]".len()..].trim().to_string();
        }
        return NormalizedOutput {
            decision: Some("REJECT".to_string()),
            feedback: if fb.is_empty() { None } else { Some(fb) },
            code: None,
        };
    }

    // Tier 2: JSON decision parsing
    if first_line.starts_with("{") || trimmed.starts_with("{") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let decision = json.get("decision").and_then(|v| v.as_str()).unwrap_or("REJECT");
            let fb = json.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
            // Also check for "response" field (legacy format)
            let fb = if fb.is_empty() {
                json.get("response").and_then(|v| v.as_str()).unwrap_or("")
            } else { fb };
            return NormalizedOutput {
                decision: Some(decision.to_string()),
                feedback: if fb.is_empty() { None } else { Some(fb.to_string()) },
                code: None,
            };
        }
    }

    // Tier 3: Raw text fallback — treat as REJECT with full content as feedback
    tracing::warn!("⚠️ [SENIOR_UNKNOWN_FORMAT] first_line='{}', treating as REJECT", first_line.chars().take(60).collect::<String>());
    NormalizedOutput {
        decision: Some("REJECT".to_string()),
        feedback: Some(raw.to_string()),
        code: None,
    }
}

fn normalize_junior_output(raw: &str) -> NormalizedOutput {
    // Tier 1: Markdown code block extraction
    if let Some(code) = extract_code_block(raw) {
        return NormalizedOutput {
            decision: None,
            feedback: None,
            code: Some(code),
        };
    }

    // Tier 2: C/C++ raw pattern extraction
    if let Some(code) = extract_cpp_c_code(raw) {
        return NormalizedOutput {
            decision: None,
            feedback: None,
            code: Some(code),
        };
    }

    // Tier 3: Raw text fallback
    tracing::warn!("⚠️ [JUNIOR_NO_CODE_BLOCK] Using raw output as-is.");
    NormalizedOutput {
        decision: None,
        feedback: None,
        code: Some(raw.to_string()),
    }
}

// extract_code_block: extracts content from ```cpp ... ``` or similar markdown blocks
fn extract_code_block(raw: &str) -> Option<String> {
    let mut result = String::new();
    let mut in_code = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code {
                if !result.is_empty() {
                    return Some(result);
                }
                in_code = false;
            } else {
                in_code = true;
                result.clear();
            }
        } else if in_code {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    if !result.is_empty() { Some(result) } else { None }
}

// extract_cpp_c_code: heuristic extraction for C/C++ code without markdown blocks
fn extract_cpp_c_code(raw: &str) -> Option<String> {
    let patterns = ["#include", "struct ", "class ", "void ", "int ", "bool ", "extern ", "typedef ", "enum ", "#define", "#ifndef", "#pragma"];
    let lines: Vec<&str> = raw.lines().collect();
    let mut first_idx = None;
    let mut last_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if patterns.iter().any(|p| trimmed.starts_with(p)) {
            if first_idx.is_none() { first_idx = Some(i); }
            last_idx = Some(i);
        }
    }
    if let (Some(start), Some(end)) = (first_idx, last_idx) {
        if end >= start {
            let code = lines[start..=end].join("\n");
            if code.len() >= 20 {
                return Some(code);
            }
        }
    }
    None
}

pub struct ExecutionPipeline {
    config: AxonConfig,
    storage: Arc<Storage>,
    event_bus: Arc<EventBus>,
    project_id: String,
    sandbox_root: PathBuf,
    pipeline_handle: Option<JoinHandle<()>>,
    pub running: Arc<AtomicBool>,
    pub pending_reviews: Arc<Mutex<HashMap<String, PipelineReview>>>,
}

impl ExecutionPipeline {
    pub fn new(
        config: AxonConfig,
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        project_id: String,
        sandbox_root: PathBuf,
    ) -> Self {
        Self {
            config,
            storage,
            event_bus,
            project_id,
            sandbox_root,
            pipeline_handle: None,
            running: Arc::new(AtomicBool::new(false)),
            pending_reviews: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn pending_reviews_handle(&self) -> Arc<Mutex<HashMap<String, PipelineReview>>> {
        self.pending_reviews.clone()
    }

    pub fn with_pending_reviews(mut self, reviews: Arc<Mutex<HashMap<String, PipelineReview>>>) -> Self {
        self.pending_reviews = reviews;
        self
    }

    pub fn with_running(mut self, running: Arc<AtomicBool>) -> Self {
        self.running = running;
        self
    }

    pub fn run_background(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            tracing::warn!("Pipeline already running for project '{}'", self.project_id);
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let config = self.config.clone();
        let storage = self.storage.clone();
        let event_bus = self.event_bus.clone();
        let project_id = self.project_id.clone();
        let sandbox_root = self.sandbox_root.clone();
        let running = self.running.clone();
        let pending_reviews = self.pending_reviews.clone();

        let handle = tokio::spawn(async move {
            let senior_driver = create_model_driver(&config.agents.seniors[0]);

            let mut juniors: Vec<AgentRuntime> = Vec::new();
            for (i, jconf) in config.agents.juniors.iter().enumerate() {
                let driver = create_model_driver(jconf);
                let mut agent = AgentRuntime::new(
                    format!("junior-agent-{:03}", i + 1),
                    AgentRole::Junior,
                    jconf.model.clone(),
                    driver,
                )
                .with_timeout(600)
                .with_project(project_id.clone());
                agent.set_locale(&config.locale);
                juniors.push(agent);
            }

            let mut senior = AgentRuntime::new(
                "senior-agent-001".to_string(),
                AgentRole::Senior,
                config.agents.seniors[0].model.clone(),
                senior_driver,
            )
            .with_timeout(300)
            .with_project(project_id.clone());
            senior.set_locale(&config.locale);

            Self::run_pipeline(
                storage,
                event_bus,
                juniors,
                senior,
                &project_id,
                &sandbox_root,
                &pending_reviews,
                &config,
                running.clone(),
            )
            .await;

            running.store(false, Ordering::SeqCst);
            tracing::info!("✅ Execution pipeline completed for project '{}'", project_id);
        });

        self.pipeline_handle = Some(handle);
    }

    #[allow(dead_code)]
    fn is_paused(running: &Arc<AtomicBool>) -> bool {
        !running.load(Ordering::SeqCst)
    }

    async fn run_pipeline(
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        juniors: Vec<AgentRuntime>,
        senior: AgentRuntime,
        project_id: &str,
        sandbox_root: &PathBuf,
        pending_reviews: &Arc<Mutex<HashMap<String, PipelineReview>>>,
        config: &AxonConfig,
        running: Arc<AtomicBool>,
    ) {
        tracing::info!("🚀 Execution pipeline started for project '{}'", project_id);

        event_bus.publish(Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            thread_id: None,
            agent_id: Some("pipeline".to_string()),
            event_type: EventType::SystemLog,
            level: EventLevel::Info,
            source: "pipeline".to_string(),
            content: format!("Execution pipeline started for project '{}'", project_id),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        let arch_path = sandbox_root.join("architecture.md");
        let architecture_guide = std::fs::read_to_string(&arch_path).unwrap_or_else(|_| {
            tracing::warn!(
                "⚠️ Architecture guide not found at {:?}, using empty guide",
                arch_path
            );
            String::new()
        });

        let all_tasks = storage.list_all_tasks().unwrap_or_default();
        let project_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|t| {
                t.project_id == project_id
                    && t.status != TaskStatus::Completed
                    && t.lifecycle_state != TaskLifecycleState::Rejected
                    && t.lifecycle_state != TaskLifecycleState::Superseded
                    && t.lifecycle_state != TaskLifecycleState::Fatal
            })
            .collect();

        if project_tasks.is_empty() {
            tracing::info!("No pending tasks for project '{}'", project_id);
            return;
        }

        // Phase 7-D: Resolve target_file from title when None (same logic as execute_one_task)
        let resolve_target = |t: &Task| -> Option<String> {
            t.target_file.clone().or_else(|| {
                let raw = t.title.split_whitespace().last().unwrap_or("unknown");
                let cleaned = raw
                    .trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')' || c == '`' || c == '*')
                    .split(']')
                    .next()
                    .unwrap_or(raw)
                    .split('(')
                    .next()
                    .unwrap_or(raw)
                    .to_string();
                if cleaned.is_empty() || cleaned == "unknown" { None } else { Some(cleaned) }
            })
        };

        let phase1: Vec<&Task> = project_tasks
            .iter()
            .filter(|t| resolve_target(t).as_deref().map(|f| f.ends_with(".h") || f.ends_with(".hpp")).unwrap_or(false))
            .collect();
        let phase2: Vec<&Task> = project_tasks
            .iter()
            .filter(|t| {
                let target = resolve_target(t);
                let is_h = target.as_deref().map(|f| f.ends_with(".h") || f.ends_with(".hpp")).unwrap_or(false);
                let is_int = t.task_kind
                    .as_ref()
                    .map(|k| matches!(k, axon_core::LanguageTaskKind::C(axon_core::CTaskKind::Integrator)))
                    .unwrap_or(false);
                !is_h && !is_int
            })
            .collect();
        let phase3: Vec<&Task> = project_tasks
            .iter()
            .filter(|t| {
                t.task_kind
                    .as_ref()
                    .map(|k| matches!(k, axon_core::LanguageTaskKind::C(axon_core::CTaskKind::Integrator)))
                    .unwrap_or(false)
            })
            .collect();

        if !phase1.is_empty() {
            tracing::info!("🏗️ Phase 1: Header declarations ({} tasks)", phase1.len());
            Self::execute_phase(
                storage.clone(), event_bus.clone(), juniors.clone(), senior.clone(), &phase1, &architecture_guide,
                sandbox_root, project_id, pending_reviews, config, running.clone(),
            )
            .await;
            if Self::is_paused(&running) { return; }

            // Phase gating: Phase 2 must not start until ALL Phase 1 tasks are Completed
            let phase1_all_completed = phase1.iter().all(|t| {
                storage
                    .get_task(&t.id)
                    .ok()
                    .flatten()
                    .map(|t| t.status == TaskStatus::Completed)
                    .unwrap_or(false)
            });
            if !phase1_all_completed {
                let failed_ids: Vec<&str> = phase1.iter()
                    .filter(|t| {
                        storage.get_task(&t.id).ok().flatten()
                            .map(|t| t.status != TaskStatus::Completed)
                            .unwrap_or(true)
                    })
                    .map(|t| t.id.as_str())
                    .collect();
                tracing::warn!(
                    "⛔ Phase 1 NOT fully completed. Skipping Phase 2/3 until Boss resolves: {:?}",
                    failed_ids
                );
                return;
            }
        }

        if !phase2.is_empty() {
            tracing::info!("🏗️ Phase 2: Source implementations ({} tasks)", phase2.len());
            Self::execute_phase(
                storage.clone(), event_bus.clone(), juniors.clone(), senior.clone(), &phase2, &architecture_guide,
                sandbox_root, project_id, pending_reviews, config, running.clone(),
            )
            .await;
            if Self::is_paused(&running) { return; }

            // Phase gating: Phase 3 must not start until ALL Phase 2 tasks are Completed
            let phase2_all_completed = phase2.iter().all(|t| {
                storage
                    .get_task(&t.id)
                    .ok()
                    .flatten()
                    .map(|t| t.status == TaskStatus::Completed)
                    .unwrap_or(false)
            });
            if !phase2_all_completed {
                let failed_ids: Vec<&str> = phase2.iter()
                    .filter(|t| {
                        storage.get_task(&t.id).ok().flatten()
                            .map(|t| t.status != TaskStatus::Completed)
                            .unwrap_or(true)
                    })
                    .map(|t| t.id.as_str())
                    .collect();
                tracing::warn!(
                    "⛔ Phase 2 NOT fully completed. Skipping Phase 3 until Boss resolves: {:?}",
                    failed_ids
                );
                return;
            }
        }

        if !phase3.is_empty() {
            tracing::info!("🏗️ Phase 3: Integrators ({} tasks)", phase3.len());
            Self::execute_phase(
                storage.clone(), event_bus.clone(), juniors.clone(), senior.clone(), &phase3, &architecture_guide,
                sandbox_root, project_id, pending_reviews, config, running.clone(),
            )
            .await;
            if Self::is_paused(&running) { return; }
        }

        let phase1_completed = phase1.iter().all(|t| {
            storage
                .get_task(&t.id)
                .ok()
                .flatten()
                .map(|t| t.status == TaskStatus::Completed)
                .unwrap_or(false)
        });
        let phase2_completed = phase2.iter().all(|t| {
            storage
                .get_task(&t.id)
                .ok()
                .flatten()
                .map(|t| t.status == TaskStatus::Completed)
                .unwrap_or(false)
        });

        if phase1_completed && phase2_completed {
            tracing::info!("🏁 All phases completed for project '{}'", project_id);
            event_bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                thread_id: None,
                agent_id: Some("pipeline".to_string()),
                event_type: EventType::ThreadCompleted,
                level: EventLevel::Info,
                source: "pipeline".to_string(),
                content: format!("Execution pipeline completed for project '{}'", project_id),
                payload: None,
                timestamp: chrono::Local::now(),
            });
        } else {
            tracing::warn!(
                "⚠️ Pipeline finished with some tasks pending review for project '{}'",
                project_id
            );
        }
    }

    async fn execute_phase(
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        juniors: Vec<AgentRuntime>,
        senior: AgentRuntime,
        tasks: &[&Task],
        architecture_guide: &str,
        sandbox_root: &PathBuf,
        project_id: &str,
        pending_reviews: &Arc<Mutex<HashMap<String, PipelineReview>>>,
        config: &AxonConfig,
        running: Arc<AtomicBool>,
    ) {
        let semaphore = Arc::new(Semaphore::new(juniors.len()));
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for (idx, task) in tasks.iter().enumerate() {
            if Self::is_paused(&running) {
                tracing::info!("⏸️ Pipeline paused mid-phase.");
                break;
            }
            if task.status == TaskStatus::Completed {
                continue;
            }

            let junior = juniors[idx % juniors.len()].clone();
            let senior = senior.clone();
            let permit = semaphore.clone();
            let storage = storage.clone();
            let event_bus = event_bus.clone();
            let task = (*task).clone();
            let architecture_guide = architecture_guide.to_string();
            let sandbox_root = sandbox_root.clone();
            let project_id = project_id.to_string();
            let pending_reviews = pending_reviews.clone();
            let config = config.clone();
            let running = running.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                Self::execute_one_task(
                    &storage, event_bus, &junior, &senior, &task, &architecture_guide,
                    &sandbox_root, &project_id, &pending_reviews, &config, running,
                )
                .await;
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    fn sandbox_path(sandbox_root: &PathBuf, target: &str) -> PathBuf {
        sandbox_root.join(".axon/sandbox").join(target)
    }

    // Phase 7-C: .failed path for state machine — preserves original_code on parser failure/rejection
    fn failed_path(sandbox_path: &PathBuf) -> PathBuf {
        let ext = sandbox_path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        if ext.is_empty() {
            sandbox_path.with_extension("failed")
        } else {
            sandbox_path.with_extension(format!("{}.failed", ext))
        }
    }

    // Phase 7-C: State transition logger
    fn log_sandbox_transition(from: SandboxState, to: SandboxState, target: &str, detail: &str) {
        tracing::info!("📦 Sandbox [{}] → [{}] | {} | {}", from.as_str(), to.as_str(), target, detail);
    }

    fn log_active_workers(storage: &Storage, label: &str) {
        let count = storage.list_all_tasks()
            .map(|tasks| tasks.iter()
                .filter(|t| t.status == TaskStatus::InProgress)
                .count())
            .unwrap_or(0);
        tracing::info!("👷 Active Workers: {} ({})", count, label);
    }

    async fn execute_one_task(
        storage: &Storage,
        event_bus: Arc<EventBus>,
        junior: &AgentRuntime,
        senior: &AgentRuntime,
        task: &Task,
        architecture_guide: &str,
        sandbox_root: &PathBuf,
        project_id: &str,
        pending_reviews: &Arc<Mutex<HashMap<String, PipelineReview>>>,
        _config: &AxonConfig,
        pipeline_running: Arc<AtomicBool>,
    ) {
        Self::log_active_workers(storage, &format!("starting task '{}'", task.title));

        event_bus.publish(Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            thread_id: Some(task.id.clone()),
            agent_id: Some("pipeline".to_string()),
            event_type: EventType::ThreadStarted,
            level: EventLevel::Info,
            source: "pipeline".to_string(),
            content: format!("Starting task: {}", task.title),
            payload: None,
            timestamp: chrono::Local::now(),
        });

        Self::update_thread_status(storage, &task.id, ThreadStatus::Working).await;

        let mut running = task.clone();
        running.status = TaskStatus::InProgress;
        running.lifecycle_state = TaskLifecycleState::Running;
        let _ = storage.save_task(running).await;
        let _ = storage.flush().await;

        let mut retries = 0u32;
        let max_retries = 3u32;
        let mut error_feedback: Option<String> = task.error_feedback.clone();

        loop {
            if Self::is_paused(&pipeline_running) {
                tracing::info!("⏸️ Pipeline paused, stopping task '{}'", task.title);
                return;
            }
            if retries >= max_retries {
                tracing::warn!(
                    "⚠️ Task '{}' failed {} times. Sending to Boss Board for review.",
                    task.title,
                    max_retries
                );

                Self::update_thread_status(storage, &task.id, ThreadStatus::BossApproval).await;

                let mut updated = task.clone();
                updated.status = TaskStatus::Failed;
                updated.lifecycle_state = TaskLifecycleState::Aborted;
                updated.boss_interventions += 1;
                let _ = storage.save_task(updated).await;

                event_bus.publish(Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id.to_string(),
                    thread_id: Some(task.id.clone()),
                    agent_id: Some("pipeline".to_string()),
                    event_type: EventType::ApprovalRequested,
                    level: EventLevel::Warning,
                    source: "pipeline".to_string(),
                    content: format!(
                        "Task '{}' requires Boss intervention (failed {} retries)",
                        task.title, max_retries
                    ),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
                Self::log_active_workers(storage, &format!("task '{}' → Boss Board (max retries)", task.title));
                return;
            }

            // Phase 7-A fix: Parse target_file from title when not set (matches Junior agent fallback)
            let resolved_target = task.target_file.clone().or_else(|| {
                let raw_target = task.title.split_whitespace().last().unwrap_or("unknown");
                let cleaned = raw_target
                    .trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')' || c == '`' || c == '*')
                    .split(']')
                    .next()
                    .unwrap_or(raw_target)
                    .split('(')
                    .next()
                    .unwrap_or(raw_target)
                    .to_string();
                if cleaned.is_empty() || cleaned == "unknown" { None } else { Some(cleaned) }
            });

            let existing_code = if retries > 0 {
                if let Some(ref target) = resolved_target {
                    let sandbox_path = Self::sandbox_path(sandbox_root, target);
                    // Phase 7-C: Check .failed file for original_code preservation
                    let failed_path = Self::failed_path(&sandbox_path);
                    std::fs::read_to_string(&sandbox_path)
                        .or_else(|_| std::fs::read_to_string(&failed_path))
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let proposal = match junior
                .process_task(
                    task,
                    architecture_guide,
                    error_feedback.clone(),
                    Some(event_bus.clone()),
                    &existing_code,
                )
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!(
                        "❌ Junior LLM error for task '{}' (attempt {}): {}",
                        task.title,
                        retries + 1,
                        e
                    );
                    retries += 1;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            // Write proposal to sandbox BEFORE Senior review (ensures target_original_code exists for retry)
            if let Some(ref code) = proposal.full_code {
                if let Some(ref target) = task.target_file {
                    let sandbox_path = Self::sandbox_path(sandbox_root, target);
                    if let Some(parent) = sandbox_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&sandbox_path, code);
                    Self::log_sandbox_transition(SandboxState::Clean, SandboxState::Proposed, target, "Junior proposal written");
                }
            }

            let review = match senior
                .review_proposal(task, &proposal, None, Some(event_bus.clone()))
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(
                        "❌ Senior LLM error for task '{}' (attempt {}): {}",
                        task.title,
                        retries + 1,
                        e
                    );
                    retries += 1;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            let _ = storage.save_post(proposal.clone()).await;
            let _ = storage.save_post(review.clone()).await;
            let _ = storage.flush().await;

            // Phase 7-B: Output Normalization Layer — single convergence point for all Senior output formats
            let normalized = normalize_output(&review.content, true);
            let is_approve = normalized.is_approve();
            let feedback = normalized.feedback.unwrap_or_default();

            if is_approve {
                if let Some(ref target) = task.target_file {
                    let sandbox_path = Self::sandbox_path(sandbox_root, target);
                    let real_path = sandbox_root.join(target);

                    // Phase 7-C: Atomic promotion with state machine logging
                    let promote_result = if sandbox_path.exists() {
                        match std::fs::rename(&sandbox_path, &real_path) {
                            Ok(_) => {
                                Self::log_sandbox_transition(SandboxState::Proposed, SandboxState::Promoted, target, "renamed to real_path");
                                Ok(())
                            }
                            Err(e) => {
                                tracing::warn!("⚠️ rename failed ({}), falling back to copy+remove", e);
                                (|| -> Result<(), std::io::Error> {
                                    if let Some(parent) = real_path.parent() {
                                        std::fs::create_dir_all(parent)?;
                                    }
                                    std::fs::copy(&sandbox_path, &real_path)?;
                                    std::fs::remove_file(&sandbox_path)?;
                                    Ok(())
                                })()
                                .map(|_| Self::log_sandbox_transition(SandboxState::Proposed, SandboxState::Promoted, target, "copy fallback"))
                                .map_err(|e| tracing::error!("❌ Failed to promote {}: {}", real_path.display(), e))
                            }
                        }
                    } else {
                        if let Some(ref code) = proposal.full_code {
                            if let Some(parent) = real_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            match std::fs::write(&real_path, code) {
                                Ok(_) => tracing::info!("✅ Wrote {} (direct, no sandbox)", real_path.display()),
                                Err(e) => tracing::error!("❌ Failed to write {}: {}", real_path.display(), e),
                            }
                        }
                        Ok(())
                    };

                    if promote_result.is_ok() {
                        let _ = std::fs::remove_file(&sandbox_path);
                    }
                }

                let mut updated = task.clone();
                updated.status = TaskStatus::Completed;
                updated.lifecycle_state = TaskLifecycleState::Completed;
                updated.rework_count = retries;
                updated.error_feedback = None;
                let _ = storage.save_task(updated).await;

                Self::update_thread_status(storage, &task.id, ThreadStatus::Completed).await;

                event_bus.publish(Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id.to_string(),
                    thread_id: Some(task.id.clone()),
                    agent_id: Some("pipeline".to_string()),
                    event_type: EventType::ThreadCompleted,
                    level: EventLevel::Info,
                    source: "pipeline".to_string(),
                    content: format!("Task '{}' completed successfully", task.title),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });

                Self::log_active_workers(storage, &format!("task '{}' completed/approved", task.title));
                tracing::info!("✅ Task '{}' approved and completed.", task.title);
                return;
            } else {
                retries += 1;

                // Phase 7-C: Transition sandbox to REJECTED state — rename to .failed to preserve original_code
                if let Some(ref target) = task.target_file {
                    let sandbox_path = Self::sandbox_path(sandbox_root, target);
                    if sandbox_path.exists() {
                        let fpath = Self::failed_path(&sandbox_path);
                        if let Err(e) = std::fs::rename(&sandbox_path, &fpath) {
                            tracing::warn!("⚠️ Failed to rename sandbox to .failed: {}", e);
                        } else {
                            Self::log_sandbox_transition(SandboxState::Proposed, SandboxState::Rejected, target, "renamed to .failed for retry preservation");
                        }
                    }
                }

                tracing::warn!(
                    "⚠️ Senior rejected task '{}' (attempt {}/{}). Feedback (first 120): {}",
                    task.title,
                    retries,
                    max_retries,
                    feedback.chars().take(120).collect::<String>()
                );

                error_feedback = Some(feedback.clone());

                let mut updated = task.clone();
                updated.error_feedback = error_feedback.clone();
                updated.rework_count = retries;
                updated.senior_rejections = task.senior_rejections + retries;

                if retries >= max_retries {
                    let review_entry = PipelineReview {
                        task_id: updated.id.clone(),
                        task: updated.clone(),
                        proposal: Some(proposal),
                        review: Some(review),
                        senior_feedback: feedback.clone(),
                    };

                    {
                        let mut reviews = pending_reviews.lock().unwrap();
                        reviews.insert(task.id.clone(), review_entry);
                    }

                    Self::update_thread_status(storage, &task.id, ThreadStatus::BossApproval).await;

                    updated.status = TaskStatus::Failed;
                    updated.lifecycle_state = TaskLifecycleState::Aborted;
                    updated.boss_interventions += 1;
                    let _ = storage.save_task(updated).await;

                    event_bus.publish(Event {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: project_id.to_string(),
                        thread_id: Some(task.id.clone()),
                        agent_id: Some("pipeline".to_string()),
                        event_type: EventType::ApprovalRequested,
                        level: EventLevel::Warning,
                        source: "pipeline".to_string(),
                        content: format!(
                            "Task '{}' requires Boss intervention",
                            task.title
                        ),
                        payload: None,
                        timestamp: chrono::Local::now(),
                    });

                    // Phase 7-C: Final failure cleanup — remove both sandbox and .failed files
                    if let Some(ref target) = task.target_file {
                        let sandbox_path = Self::sandbox_path(sandbox_root, target);
                        let fpath = Self::failed_path(&sandbox_path);
                        let _ = std::fs::remove_file(&sandbox_path);
                        let _ = std::fs::remove_file(&fpath);
                        Self::log_sandbox_transition(SandboxState::Rejected, SandboxState::Clean, target, "Boss Board — all files cleaned");
                    }

                    Self::log_active_workers(storage, &format!("task '{}' → Boss Board (3× reject)", task.title));
                    tracing::warn!(
                        "⚠️ Task '{}' sent to Boss Board after {} rejections.",
                        task.title,
                        max_retries
                    );
                    return;
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    async fn update_thread_status(storage: &Storage, thread_id: &str, status: ThreadStatus) {
        if let Ok(Some(mut thread)) = storage.get_thread(thread_id) {
            thread.status = status;
            thread.updated_at = chrono::Local::now();
            let _ = storage.save_thread(thread).await;
        }
    }
}
