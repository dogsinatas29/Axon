use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Path}, response::IntoResponse, http::StatusCode,
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
};
use serde::{Serialize, Deserialize};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::Semaphore;
use tower_http::services::ServeDir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::sync::RwLock as AsyncRwLock;
use crate::AxonConfig;
use crate::bootstrap::{BootstrapManager, create_model_driver};
use crate::events::EventBus;
use crate::pipeline::ExecutionPipeline;
use crate::{PendingApproval, PipelineReview, AgentConfig, PersonaConfig};
use axon_storage::Storage;

pub struct AgentPool {
    pub juniors: Vec<AgentConfig>,
    pub seniors: Vec<AgentConfig>,
    pub architect: AgentConfig,
}

pub struct PersonaRegistry {
    pub personas: std::collections::HashMap<String, PersonaConfig>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub axon_config: AxonConfig,
    pub bootstrap_status: Arc<AsyncMutex<BootstrapStatus>>,
    pub pending_approval: Arc<Mutex<Option<PendingApproval>>>,
    pub pending_reviews: Arc<Mutex<std::collections::HashMap<String, PipelineReview>>>,
    pub pipeline_running: Arc<AtomicBool>,
    pub storage: Arc<Storage>,
    pub event_bus: Arc<EventBus>,
    pub task_semaphore: Arc<Semaphore>,
    pub agent_pool: Arc<AsyncRwLock<AgentPool>>,
    pub persona_registry: Arc<AsyncRwLock<PersonaRegistry>>,
    pub project_folder: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BootstrapStatus {
    stage: String,
    message: String,
    is_running: bool,
    is_complete: bool,
    error: Option<String>,
    project_id: Option<String>,
}

impl Default for BootstrapStatus {
    fn default() -> Self {
        Self {
            stage: "idle".to_string(),
            message: "Awaiting specification submission.".to_string(),
            is_running: false,
            is_complete: false,
            error: None,
            project_id: None,
        }
    }
}

#[derive(Serialize)]
struct StatusResponse {
    is_running: bool,
    active_threads: usize,
    total_signals: usize,
    nogari_count: usize,
    bootstrap_stage: String,
    locale: String,
    bootstrap: BootstrapStatus,
    project_folder: String,
}

#[derive(Deserialize)]
struct SpecSubmission {
    content: String,
}

#[derive(Serialize)]
struct SpecSubmissionResponse {
    status: String,
    message: String,
    project_id: Option<String>,
}

async fn get_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    let bs = state.bootstrap_status.lock().await;
    // active_threads = 현재 InProgress 상태인 태스크 수 = 실제 작업 중인 워커(주니어) 수
    let _ = state.storage.flush().await;
    let active_workers = state.storage.list_all_tasks()
        .map(|tasks| tasks.iter()
            .filter(|t| t.status == axon_core::TaskStatus::InProgress)
            .count())
        .unwrap_or(0);
    let event_count = state.event_bus.get_count();
    let nogari_count = state.storage.list_posts_by_thread("lounge")
        .map(|p| p.len())
        .unwrap_or(0);
    let bootstrap_stage = if bs.is_complete {
        "Complete".to_string()
    } else if bs.is_running {
        format!("Running: {}", bs.stage)
    } else {
        bs.stage.clone()
    };
    Json(StatusResponse {
        is_running: bs.is_running || bs.is_complete,
        active_threads: active_workers,
        total_signals: event_count,
        nogari_count,
        bootstrap_stage,
        locale: state.axon_config.locale.clone(),
        bootstrap: bs.clone(),
        project_folder: state.project_folder.clone(),
    })
}

async fn list_tasks(State(state): State<Arc<AppState>>) -> Json<Vec<serde_json::Value>> {
    let tasks = state.storage.list_runnable_threads()
        .unwrap_or_default();
    Json(tasks.into_iter().map(|t| serde_json::to_value(t).unwrap_or_default()).collect())
}

async fn list_threads(State(state): State<Arc<AppState>>) -> Json<Vec<serde_json::Value>> {
    let threads = state.storage.list_all_threads()
        .unwrap_or_default();
    Json(threads.into_iter().map(|t| serde_json::to_value(t).unwrap_or_default()).collect())
}

async fn list_posts(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> Json<Vec<serde_json::Value>> {
    let posts = state.storage.list_posts_by_thread(&thread_id)
        .unwrap_or_default();
    Json(posts.into_iter().map(|p| serde_json::to_value(p).unwrap_or_default()).collect())
}

async fn list_agents_api(State(state): State<Arc<AppState>>) -> Json<Vec<serde_json::Value>> {
    let agents = state.storage.list_agents()
        .unwrap_or_default();
    Json(agents.into_iter().map(|a| serde_json::to_value(a).unwrap_or_default()).collect())
}

async fn list_events(State(state): State<Arc<AppState>>) -> Json<Vec<serde_json::Value>> {
    let events = state.storage.list_events(200)
        .unwrap_or_default();
    Json(events.into_iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect())
}

async fn get_specs_status(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Json<serde_json::Value> {
    let bs = state.bootstrap_status.lock().await;
    let progress = match bs.stage.as_str() {
        "idle" => 0.0,
        "queued" => 0.05,
        "SpecAnalysis" => 0.25,
        "Skeleton" => 0.5,
        "ImplGen" => 0.75,
        "complete" => 1.0,
        "failed" => 0.0,
        _ => 0.0,
    };
    Json(serde_json::json!({
        "project_id": bs.project_id,
        "stage": bs.stage,
        "state": if bs.is_complete { "Completed" } else if bs.error.is_some() { "Failed" } else { "Running" },
        "message": bs.message,
        "progress": progress,
        "error": bs.error,
    }))
}

async fn approve_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("Approving thread: {}", thread_id);

    let review = {
        let mut reviews = state.pending_reviews.lock().unwrap();
        reviews.remove(&thread_id)
    };

    match review {
        Some(r) => {
            if let Some(ref proposal) = r.proposal {
                if let Some(ref code) = proposal.full_code {
                    if let Some(ref target) = r.task.target_file {
                        let sandbox_root = std::path::Path::new(&r.task.project_id);
                        let fpath = sandbox_root.join(target);
                        if let Some(parent) = fpath.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(&fpath, code);
                    }
                }
            }

            let mut updated = r.task.clone();
            updated.status = axon_core::TaskStatus::Completed;
            updated.lifecycle_state = axon_core::TaskLifecycleState::Completed;
            updated.boss_interventions += 1;
            let _ = state.storage.save_task(updated.clone()).await;

            if let Ok(Some(mut thread)) = state.storage.get_thread(&thread_id) {
                thread.status = axon_core::ThreadStatus::Completed;
                thread.updated_at = chrono::Local::now();
                thread.boss_interventions = updated.boss_interventions;
                thread.senior_rejections = updated.senior_rejections;
                thread.validator_rejections = updated.validator_rejections;
                thread.architecture_rejections = updated.architecture_rejections;
                thread.cargo_rejections = updated.cargo_rejections;
                thread.lsp_rejections = updated.lsp_rejections;
                let _ = state.storage.save_thread(thread).await;
            }

            if let Some(proposal) = r.proposal {
                let _ = state.storage.save_post(proposal).await;
            }
            if let Some(review) = r.review {
                let _ = state.storage.save_post(review).await;
            }
            let _ = state.storage.flush().await;

            state.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: updated.project_id.clone(),
                thread_id: Some(thread_id.clone()),
                agent_id: Some("boss".to_string()),
                event_type: axon_core::EventType::ApprovalGranted,
                level: axon_core::EventLevel::Info,
                source: "boss".to_string(),
                content: format!("Boss approved task '{}'", updated.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            // Boss Approve 후 Phase Gating 재체크: 모든 Phase 1 task가 Completed면 Phase 2 자동 재시작
            let project_id = updated.project_id.clone();
            let state_for_spawn = state.clone();
            tokio::spawn(async move {
                // Phase 1 header task들이 모두 Completed인지 확인
                let all_tasks = state_for_spawn.storage.list_all_tasks().unwrap_or_default();
                let phase1_tasks: Vec<_> = all_tasks.iter().filter(|t| {
                    t.project_id == project_id &&
                    t.target_file.as_deref().map(|f| f.ends_with(".h") || f.ends_with(".hpp")).unwrap_or(false)
                }).collect();

                if !phase1_tasks.is_empty() {
                    let all_completed = phase1_tasks.iter().all(|t| t.status == axon_core::TaskStatus::Completed);
                    if all_completed {
                        // Phase 2에 pending task가 있는지 확인
                        let phase2_pending: Vec<_> = all_tasks.iter().filter(|t| {
                            t.project_id == project_id &&
                            t.status != axon_core::TaskStatus::Completed &&
                            t.lifecycle_state != axon_core::TaskLifecycleState::Rejected &&
                            t.lifecycle_state != axon_core::TaskLifecycleState::Fatal &&
                            !t.target_file.as_deref().map(|f| f.ends_with(".h") || f.ends_with(".hpp")).unwrap_or(false)
                        }).collect();

                        if !phase2_pending.is_empty() && !state_for_spawn.pipeline_running.load(Ordering::SeqCst) {
                            tracing::info!("🔄 Boss approved all Phase 1 tasks. Auto-resuming pipeline for Phase 2...");

                            let sandbox_root = std::path::PathBuf::from(&project_id);
                            let mut pipeline = ExecutionPipeline::new(
                                state_for_spawn.axon_config.clone(),
                                state_for_spawn.storage.clone(),
                                state_for_spawn.event_bus.clone(),
                                project_id.clone(),
                                sandbox_root,
                                state_for_spawn.agent_pool.clone(),
                            )
                            .with_pending_reviews(state_for_spawn.pending_reviews.clone())
                            .with_running(state_for_spawn.pipeline_running.clone())
                            .with_task_semaphore(state_for_spawn.task_semaphore.clone());
                            pipeline.run_background();
                        }
                    }
                }
            });

            (StatusCode::OK, Json(serde_json::json!({"status": "approved", "thread_id": thread_id}))).into_response()
        }
        None => {
            let project_folder = if let Ok(Some(th)) = state.storage.get_thread(&thread_id) {
                th.project_id.clone()
            } else {
                "spec".to_string()
            };
            let sandbox_root = std::path::Path::new(&project_folder);
            let approval_file = sandbox_root.join(format!(".axon_approval_pending_{}", thread_id));
            let legacy_approval_file = sandbox_root.join(".axon_approval_pending");

            let mut updated_file = false;

            if approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        approval["approved"] = serde_json::Value::Bool(true);
                        approval["status"] = serde_json::Value::String("APPROVED".to_string());
                        if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                            if std::fs::write(&approval_file, updated_content).is_ok() {
                                tracing::info!("✅ Boss approved thread via high-level file trigger: {}", thread_id);
                                updated_file = true;
                            }
                        }
                    }
                }
            }

            if !updated_file && legacy_approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&legacy_approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        if approval["task_id"].as_str() == Some(&thread_id) {
                            approval["approved"] = serde_json::Value::Bool(true);
                            approval["status"] = serde_json::Value::String("APPROVED".to_string());
                            if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                                if std::fs::write(&legacy_approval_file, updated_content).is_ok() {
                                    tracing::info!("✅ Boss approved thread via legacy file trigger: {}", thread_id);
                                    updated_file = true;
                                }
                            }
                        }
                    }
                }
            }

            if updated_file {
                (StatusCode::OK, Json(serde_json::json!({"status": "approved", "thread_id": thread_id}))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review or approval file found for this thread"}))).into_response()
            }
        }
    }
}

async fn reject_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("Rejecting thread: {}", thread_id);

    let review = {
        let mut reviews = state.pending_reviews.lock().unwrap();
        reviews.remove(&thread_id)
    };

    match review {
        Some(r) => {
            let mut updated = r.task.clone();
            updated.status = axon_core::TaskStatus::Failed;
            updated.lifecycle_state = axon_core::TaskLifecycleState::Rejected;
            updated.boss_interventions += 1;
            let _ = state.storage.save_task(updated.clone()).await;

            if let Ok(Some(mut thread)) = state.storage.get_thread(&thread_id) {
                thread.status = axon_core::ThreadStatus::Completed;
                thread.updated_at = chrono::Local::now();
                thread.boss_interventions = updated.boss_interventions;
                thread.senior_rejections = updated.senior_rejections;
                thread.validator_rejections = updated.validator_rejections;
                thread.architecture_rejections = updated.architecture_rejections;
                thread.cargo_rejections = updated.cargo_rejections;
                thread.lsp_rejections = updated.lsp_rejections;
                let _ = state.storage.save_thread(thread).await;
            }

            state.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: updated.project_id.clone(),
                thread_id: Some(thread_id.clone()),
                agent_id: Some("boss".to_string()),
                event_type: axon_core::EventType::ApprovalRejected,
                level: axon_core::EventLevel::Warning,
                source: "boss".to_string(),
                content: format!("Boss rejected task '{}'", updated.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            (StatusCode::OK, Json(serde_json::json!({"status": "rejected", "thread_id": thread_id}))).into_response()
        }
        None => {
            let project_folder = if let Ok(Some(th)) = state.storage.get_thread(&thread_id) {
                th.project_id.clone()
            } else {
                "spec".to_string()
            };
            let sandbox_root = std::path::Path::new(&project_folder);
            let approval_file = sandbox_root.join(format!(".axon_approval_pending_{}", thread_id));
            let legacy_approval_file = sandbox_root.join(".axon_approval_pending");

            let mut updated_file = false;

            if approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        approval["approved"] = serde_json::Value::Bool(false);
                        approval["status"] = serde_json::Value::String("REJECTED".to_string());
                        if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                            if std::fs::write(&approval_file, updated_content).is_ok() {
                                tracing::info!("❌ Boss rejected thread via high-level file trigger: {}", thread_id);
                                updated_file = true;
                            }
                        }
                    }
                }
            }

            if !updated_file && legacy_approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&legacy_approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        if approval["task_id"].as_str() == Some(&thread_id) {
                            approval["approved"] = serde_json::Value::Bool(false);
                            approval["status"] = serde_json::Value::String("REJECTED".to_string());
                            if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                                if std::fs::write(&legacy_approval_file, updated_content).is_ok() {
                                    tracing::info!("❌ Boss rejected thread via legacy file trigger: {}", thread_id);
                                    updated_file = true;
                                }
                            }
                        }
                    }
                }
            }

            if updated_file {
                (StatusCode::OK, Json(serde_json::json!({"status": "rejected", "thread_id": thread_id}))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review or approval file found for this thread"}))).into_response()
            }
        }
    }
}

async fn retry_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Json(body): Json<BossReviewAction>,
) -> impl IntoResponse {
    tracing::info!("Retrying thread: {}", thread_id);

    let review = {
        let mut reviews = state.pending_reviews.lock().unwrap();
        reviews.remove(&thread_id)
    };

    match review {
        Some(r) => {
            let feedback = body.feedback.unwrap_or_else(|| r.senior_feedback.clone());

            let mut updated = r.task.clone();
            updated.status = axon_core::TaskStatus::Pending;
            updated.lifecycle_state = axon_core::TaskLifecycleState::Queued;
            updated.error_feedback = Some(feedback);
            updated.rework_count = 0;
            updated.senior_rejections = 0;
            updated.boss_interventions += 1;
            let _ = state.storage.save_task(updated.clone()).await;

            if let Ok(Some(mut thread)) = state.storage.get_thread(&thread_id) {
                thread.status = axon_core::ThreadStatus::Draft;
                thread.updated_at = chrono::Local::now();
                thread.boss_interventions = updated.boss_interventions;
                thread.senior_rejections = updated.senior_rejections;
                thread.validator_rejections = updated.validator_rejections;
                thread.architecture_rejections = updated.architecture_rejections;
                thread.cargo_rejections = updated.cargo_rejections;
                thread.lsp_rejections = updated.lsp_rejections;
                let _ = state.storage.save_thread(thread).await;
            }

            state.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: updated.project_id.clone(),
                thread_id: Some(thread_id.clone()),
                agent_id: Some("boss".to_string()),
                event_type: axon_core::EventType::SystemLog,
                level: axon_core::EventLevel::Info,
                source: "boss".to_string(),
                content: format!("Boss retried task '{}' with feedback", updated.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            (StatusCode::OK, Json(serde_json::json!({"status": "retrying", "thread_id": thread_id}))).into_response()
        }
        None => {
            let project_folder = if let Ok(Some(th)) = state.storage.get_thread(&thread_id) {
                th.project_id.clone()
            } else {
                "spec".to_string()
            };
            let sandbox_root = std::path::Path::new(&project_folder);
            let approval_file = sandbox_root.join(format!(".axon_approval_pending_{}", thread_id));
            let legacy_approval_file = sandbox_root.join(".axon_approval_pending");

            let mut updated_file = false;

            if approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        approval["approved"] = serde_json::Value::Bool(false);
                        approval["status"] = serde_json::Value::String("REJECTED".to_string());
                        if let Some(ref feedback) = body.feedback {
                            approval["senior_feedback"] = serde_json::Value::String(feedback.clone());
                        }
                        if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                            if std::fs::write(&approval_file, updated_content).is_ok() {
                                tracing::info!("🔄 Boss retried thread via high-level file trigger: {}", thread_id);
                                updated_file = true;
                            }
                        }
                    }
                }
            }

            if !updated_file && legacy_approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&legacy_approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        if approval["task_id"].as_str() == Some(&thread_id) {
                            approval["approved"] = serde_json::Value::Bool(false);
                            approval["status"] = serde_json::Value::String("REJECTED".to_string());
                            if let Some(ref feedback) = body.feedback {
                                approval["senior_feedback"] = serde_json::Value::String(feedback.clone());
                            }
                            if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                                if std::fs::write(&legacy_approval_file, updated_content).is_ok() {
                                    tracing::info!("🔄 Boss retried thread via legacy file trigger: {}", thread_id);
                                    updated_file = true;
                                }
                            }
                        }
                    }
                }
            }

            if updated_file {
                (StatusCode::OK, Json(serde_json::json!({"status": "retrying", "thread_id": thread_id}))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review or approval file found for this thread"}))).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
struct HireRequest {
    role: String,
    model: String,
    runtime: String,
    provider: Option<String>,
    endpoint: Option<String>,
    persona: Option<PersonaConfig>,
}

async fn hire_agent(
    State(state): State<Arc<AppState>>,
    Json(body): Json<HireRequest>,
) -> impl IntoResponse {
    let new_id = format!("{}-agent-{}", body.role.to_lowercase(), uuid::Uuid::new_v4().simple());
    
    let new_config = AgentConfig {
        id: Some(new_id.clone()),
        runtime: body.runtime,
        provider: body.provider,
        endpoint: body.endpoint,
        model: body.model,
        provider_type: None,
    };
    
    match body.role.as_str() {
        "Junior" | "junior" => {
            let mut pool = state.agent_pool.write().await;
            pool.juniors.push(new_config);
            state.task_semaphore.add_permits(1);
            tracing::info!("Junior hired: {} (semaphore +1, total: {})", new_id, pool.juniors.len());
        }
        "Senior" | "senior" => {
            let mut pool = state.agent_pool.write().await;
            pool.seniors.push(new_config);
            tracing::info!("Senior hired: {} (total: {})", new_id, pool.seniors.len());
        }
        "Architect" | "architect" => {
            let mut pool = state.agent_pool.write().await;
            pool.architect = new_config;
            tracing::info!("Architect replaced: {}", new_id);
        }
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid role"}))),
    }
    
    if let Some(persona) = body.persona {
        let mut registry = state.persona_registry.write().await;
        registry.personas.insert(new_id.clone(), persona);
    }
    
    state.event_bus.publish(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: String::new(),
        thread_id: None,
        agent_id: Some(new_id.clone()),
        event_type: axon_core::EventType::AgentHired,
        level: axon_core::EventLevel::Info,
        source: "hr_board".to_string(),
        content: format!("Agent {} hired as {}", new_id, body.role),
        payload: None,
        timestamp: chrono::Local::now(),
    });
    
    (StatusCode::OK, Json(serde_json::json!({
        "status": "hired",
        "id": new_id,
        "role": body.role
    })))
}

async fn fire_agent(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let role = body.get("role").and_then(|v| v.as_str()).unwrap_or("junior");
    
    match role {
        "Junior" | "junior" => {
            let mut pool = state.agent_pool.write().await;
            if pool.juniors.len() <= 1 {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Cannot fire last junior worker"
                })));
            }
            
            let before_len = pool.juniors.len();
            pool.juniors.retain(|a| a.id.as_deref() != Some(&agent_id));
            
            if pool.juniors.len() < before_len {
                state.task_semaphore.forget_permits(1);
                tracing::info!("Junior fired: {} (semaphore -1, remaining: {})", agent_id, pool.juniors.len());
                
                state.event_bus.publish(axon_core::Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: String::new(),
                    thread_id: None,
                    agent_id: Some(agent_id.clone()),
                    event_type: axon_core::EventType::AgentFired,
                    level: axon_core::EventLevel::Warning,
                    source: "hr_board".to_string(),
                    content: format!("Agent {} fired. Graceful eviction: current task completes, no new assignments.", agent_id),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
                
                (StatusCode::OK, Json(serde_json::json!({
                    "status": "fired",
                    "id": agent_id,
                    "eviction": "graceful"
                })))
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Agent not found"})))
            }
        }
        "Senior" | "senior" => {
            let mut pool = state.agent_pool.write().await;
            if pool.seniors.is_empty() {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Cannot fire last senior reviewer"
                })));
            }
            
            let before_len = pool.seniors.len();
            pool.seniors.retain(|a| a.id.as_deref() != Some(&agent_id));
            
            if pool.seniors.len() < before_len {
                tracing::info!("Senior fired: {} (remaining: {})", agent_id, pool.seniors.len());
                (StatusCode::OK, Json(serde_json::json!({
                    "status": "fired",
                    "id": agent_id
                })))
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Agent not found"})))
            }
        }
        _ => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid role"}))),
    }
}

#[derive(Deserialize)]
struct SwapProviderRequest {
    runtime: String,
    provider: String,
    model: String,
}

async fn swap_provider(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
    Json(body): Json<SwapProviderRequest>,
) -> impl IntoResponse {
    let mut pool = state.agent_pool.write().await;
    
    let swap_runtime = body.runtime.clone();
    let swap_provider = body.provider.clone();
    let swap_model = body.model.clone();
    
    let mut found = false;
    for j in &mut pool.juniors {
        if j.id.as_deref() == Some(&agent_id) {
            j.runtime = body.runtime.clone();
            j.provider = Some(body.provider.clone());
            j.model = body.model.clone();
            found = true;
            break;
        }
    }
    if !found {
        for s in &mut pool.seniors {
            if s.id.as_deref() == Some(&agent_id) {
                s.runtime = body.runtime.clone();
                s.provider = Some(body.provider.clone());
                s.model = body.model.clone();
                found = true;
                break;
            }
        }
    }
    if !found && pool.architect.id.as_deref() == Some(&agent_id) {
        pool.architect.runtime = body.runtime.clone();
        pool.architect.provider = Some(body.provider.clone());
        pool.architect.model = body.model.clone();
        found = true;
    }
    
    if found {
        tracing::info!("Provider swapped for {}: runtime={}, provider={}, model={}", agent_id, swap_runtime, swap_provider, swap_model);
        state.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: String::new(),
            thread_id: None,
            agent_id: Some(agent_id.clone()),
            event_type: axon_core::EventType::AgentUpdated,
            level: axon_core::EventLevel::Info,
            source: "hr_board".to_string(),
            content: format!("Agent {} provider hot-swapped to {}/{}", agent_id, swap_runtime, swap_model),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        (StatusCode::OK, Json(serde_json::json!({"status": "swapped", "id": agent_id})))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Agent not found"})))
    }
}

async fn get_semantics_risks() -> Json<serde_json::Value> {
    Json(serde_json::json!({"risks": []}))
}

async fn post_semantics_decide(
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("Semantic decision: {:?}", body);
    (StatusCode::OK, Json(serde_json::json!({"status": "decided"})))
}

async fn get_pending_approval(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let approval = state.pending_approval.lock().unwrap();
    match approval.as_ref() {
        Some(p) => (StatusCode::OK, Json(serde_json::to_value(p).unwrap())).into_response(),
        None => (StatusCode::NOT_FOUND, "No pending approval").into_response(),
    }
}

async fn approve_spec_analysis(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("📡 Spec analysis approved via API POST /api/specs/approve");
    let mut approval = state.pending_approval.lock().unwrap();
    match approval.as_mut() {
        Some(ref mut p) => {
            p.approved = true;
            let content = serde_json::json!({"status": "APPROVED", "approved": true});
            let _ = std::fs::write(&p.approval_file_path, serde_json::to_string(&content).unwrap());
            (StatusCode::OK, Json(serde_json::json!({"status": "approved"}))).into_response()
        }
        None => (StatusCode::CONFLICT, "No pending approval").into_response(),
    }
}

async fn reject_spec_analysis(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("📡 Spec analysis rejected via API POST /api/specs/reject");
    let mut approval = state.pending_approval.lock().unwrap();
    match approval.as_mut() {
        Some(ref mut p) => {
            p.rejected = true;
            let content = serde_json::json!({"status": "REJECTED", "approved": false});
            let _ = std::fs::write(&p.approval_file_path, serde_json::to_string(&content).unwrap());
            (StatusCode::OK, Json(serde_json::json!({"status": "rejected"}))).into_response()
        }
        None => (StatusCode::CONFLICT, "No pending approval").into_response(),
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, _receiver) = socket.split();
    let mut rx = state.event_bus.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => {
                if let Ok(json) = serde_json::to_string(&event) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("WebSocket event bus lagged by {} messages", n);
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn list_pipeline_reviews(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<serde_json::Value>> {
    let reviews = state.pending_reviews.lock().unwrap();
    Json(reviews.values().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[derive(Deserialize)]
struct BossReviewAction {
    feedback: Option<String>,
}

async fn approve_pipeline_review(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let review = {
        let mut reviews = state.pending_reviews.lock().unwrap();
        reviews.remove(&task_id)
    };

    match review {
        Some(r) => {
            if let Some(ref proposal) = r.proposal {
                if let Some(ref code) = proposal.full_code {
                    if let Some(ref target) = r.task.target_file {
                        let sandbox_root = std::path::Path::new(&r.task.project_id);
                        let fpath = sandbox_root.join(target);
                        if let Some(parent) = fpath.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match std::fs::write(&fpath, code) {
                            Ok(_) => tracing::info!("✅ Boss approved: wrote {}", fpath.display()),
                            Err(e) => tracing::error!("❌ Boss approve write failed {}: {}", fpath.display(), e),
                        }
                    }
                }
            }

            let mut updated = r.task.clone();
            updated.status = axon_core::TaskStatus::Completed;
            updated.lifecycle_state = axon_core::TaskLifecycleState::Completed;
            updated.boss_interventions += 1;
            let _ = state.storage.save_task(updated.clone()).await;

            if let Ok(Some(mut thread)) = state.storage.get_thread(&task_id) {
                thread.status = axon_core::ThreadStatus::Completed;
                thread.updated_at = chrono::Local::now();
                thread.boss_interventions = updated.boss_interventions;
                thread.senior_rejections = updated.senior_rejections;
                thread.validator_rejections = updated.validator_rejections;
                thread.architecture_rejections = updated.architecture_rejections;
                thread.cargo_rejections = updated.cargo_rejections;
                thread.lsp_rejections = updated.lsp_rejections;
                let _ = state.storage.save_thread(thread).await;
            }

            if let Some(proposal) = r.proposal {
                let _ = state.storage.save_post(proposal).await;
            }
            if let Some(review) = r.review {
                let _ = state.storage.save_post(review).await;
            }
            let _ = state.storage.flush().await;

            state.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: updated.project_id.clone(),
                thread_id: Some(task_id.clone()),
                agent_id: Some("boss".to_string()),
                event_type: axon_core::EventType::ApprovalGranted,
                level: axon_core::EventLevel::Info,
                source: "boss".to_string(),
                content: format!("Boss approved task '{}'", updated.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            // Auto-resume pipeline after Boss approval
            let state_clone = state.clone();
            let project_id = updated.project_id.clone();
            tokio::spawn(async move {
                state_clone.pipeline_running.store(false, Ordering::SeqCst);
                if let Ok(tasks) = state_clone.storage.list_all_tasks() {
                    let has_pending = tasks.iter().any(|t| {
                        t.project_id == project_id &&
                        t.status != axon_core::TaskStatus::Completed &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Rejected &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Fatal &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Superseded
                    });
                    if has_pending {
                        tracing::info!("🔄 Boss approved task. Auto-resuming pipeline for remaining tasks...");
                        let sandbox_root = std::path::PathBuf::from(&project_id);
                        let mut pipeline = ExecutionPipeline::new(
                            state_clone.axon_config.clone(),
                            state_clone.storage.clone(),
                            state_clone.event_bus.clone(),
                            project_id.clone(),
                            sandbox_root,
                            state_clone.agent_pool.clone(),
                        )
                        .with_pending_reviews(state_clone.pending_reviews.clone())
                        .with_running(state_clone.pipeline_running.clone())
                        .with_task_semaphore(state_clone.task_semaphore.clone());
                        pipeline.run_background();
                    }
                }
            });

            (StatusCode::OK, Json(serde_json::json!({"status": "approved", "task_id": task_id}))).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this task"}))).into_response(),
    }
}

async fn reject_pipeline_review(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let review = {
        let mut reviews = state.pending_reviews.lock().unwrap();
        reviews.remove(&task_id)
    };

    match review {
        Some(r) => {
            let mut updated = r.task.clone();
            updated.status = axon_core::TaskStatus::Failed;
            updated.lifecycle_state = axon_core::TaskLifecycleState::Rejected;
            updated.boss_interventions += 1;
            let _ = state.storage.save_task(updated.clone()).await;

            if let Ok(Some(mut thread)) = state.storage.get_thread(&task_id) {
                thread.status = axon_core::ThreadStatus::Completed;
                thread.updated_at = chrono::Local::now();
                thread.boss_interventions = updated.boss_interventions;
                thread.senior_rejections = updated.senior_rejections;
                thread.validator_rejections = updated.validator_rejections;
                thread.architecture_rejections = updated.architecture_rejections;
                thread.cargo_rejections = updated.cargo_rejections;
                thread.lsp_rejections = updated.lsp_rejections;
                let _ = state.storage.save_thread(thread).await;
            }

            state.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: updated.project_id.clone(),
                thread_id: Some(task_id.clone()),
                agent_id: Some("boss".to_string()),
                event_type: axon_core::EventType::ApprovalRejected,
                level: axon_core::EventLevel::Warning,
                source: "boss".to_string(),
                content: format!("Boss rejected task '{}'", updated.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            // Auto-resume pipeline after Boss reject
            let state_clone = state.clone();
            let project_id = updated.project_id.clone();
            tokio::spawn(async move {
                state_clone.pipeline_running.store(false, Ordering::SeqCst);
                if let Ok(tasks) = state_clone.storage.list_all_tasks() {
                    let has_pending = tasks.iter().any(|t| {
                        t.project_id == project_id &&
                        t.status != axon_core::TaskStatus::Completed &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Rejected &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Fatal &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Superseded
                    });
                    if has_pending {
                        tracing::info!("🔄 Boss rejected task. Auto-resuming pipeline for remaining tasks...");
                        let sandbox_root = std::path::PathBuf::from(&project_id);
                        let mut pipeline = ExecutionPipeline::new(
                            state_clone.axon_config.clone(),
                            state_clone.storage.clone(),
                            state_clone.event_bus.clone(),
                            project_id.clone(),
                            sandbox_root,
                            state_clone.agent_pool.clone(),
                        )
                        .with_pending_reviews(state_clone.pending_reviews.clone())
                        .with_running(state_clone.pipeline_running.clone())
                        .with_task_semaphore(state_clone.task_semaphore.clone());
                        pipeline.run_background();
                    }
                }
            });

            (StatusCode::OK, Json(serde_json::json!({"status": "rejected", "task_id": task_id}))).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this task"}))).into_response(),
    }
}

async fn retry_pipeline_review(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
    Json(body): Json<BossReviewAction>,
) -> impl IntoResponse {
    let review = {
        let mut reviews = state.pending_reviews.lock().unwrap();
        reviews.remove(&task_id)
    };

    match review {
        Some(r) => {
            let feedback = body.feedback.unwrap_or_else(|| r.senior_feedback.clone());

            let mut updated = r.task.clone();
            updated.status = axon_core::TaskStatus::Pending;
            updated.lifecycle_state = axon_core::TaskLifecycleState::Queued;
            updated.error_feedback = Some(feedback);
            updated.rework_count = 0;
            updated.senior_rejections = 0;
            updated.boss_interventions += 1;
            let _ = state.storage.save_task(updated.clone()).await;

            if let Ok(Some(mut thread)) = state.storage.get_thread(&task_id) {
                thread.status = axon_core::ThreadStatus::Draft;
                thread.updated_at = chrono::Local::now();
                thread.boss_interventions = updated.boss_interventions;
                thread.senior_rejections = updated.senior_rejections;
                thread.validator_rejections = updated.validator_rejections;
                thread.architecture_rejections = updated.architecture_rejections;
                thread.cargo_rejections = updated.cargo_rejections;
                thread.lsp_rejections = updated.lsp_rejections;
                let _ = state.storage.save_thread(thread).await;
            }

            state.event_bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: updated.project_id.clone(),
                thread_id: Some(task_id.clone()),
                agent_id: Some("boss".to_string()),
                event_type: axon_core::EventType::SystemLog,
                level: axon_core::EventLevel::Info,
                source: "boss".to_string(),
                content: format!("Boss retried task '{}' with feedback", updated.title),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            // [FIX_RETRY_HANG] Retry 시 기존 파이프라인 스레드가 파일 폴링(1시간)에 갇히는 것을 방지
            let sandbox_root = std::path::PathBuf::from(&updated.project_id);
            let approval_file = sandbox_root.join(format!(".axon_approval_pending_{}", task_id));
            if approval_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&approval_file) {
                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {
                        approval["approved"] = serde_json::Value::Bool(false);
                        approval["status"] = serde_json::Value::String("REJECTED".to_string());
                        if let Ok(updated_content) = serde_json::to_string_pretty(&approval) {
                            let _ = std::fs::write(&approval_file, updated_content);
                        }
                    }
                }
            }

            // Auto-resume pipeline after Boss retry
            let state_clone = state.clone();
            let project_id = updated.project_id.clone();
            tokio::spawn(async move {
                state_clone.pipeline_running.store(false, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                
                if let Ok(tasks) = state_clone.storage.list_all_tasks() {
                    let has_pending = tasks.iter().any(|t| {
                        t.project_id == project_id &&
                        t.status != axon_core::TaskStatus::Completed &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Rejected &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Fatal &&
                        t.lifecycle_state != axon_core::TaskLifecycleState::Superseded
                    });
                    if has_pending {
                        tracing::info!("🔄 Boss retried task. Auto-resuming pipeline for remaining tasks...");
                        let sandbox_root = std::path::PathBuf::from(&project_id);
                        let mut pipeline = ExecutionPipeline::new(
                            state_clone.axon_config.clone(),
                            state_clone.storage.clone(),
                            state_clone.event_bus.clone(),
                            project_id.clone(),
                            sandbox_root,
                            state_clone.agent_pool.clone(),
                        )
                        .with_pending_reviews(state_clone.pending_reviews.clone())
                        .with_running(state_clone.pipeline_running.clone())
                        .with_task_semaphore(state_clone.task_semaphore.clone());
                        pipeline.run_background();
                    }
                }
            });

            (StatusCode::OK, Json(serde_json::json!({"status": "retrying", "task_id": task_id}))).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this task"}))).into_response(),
    }
}

async fn resume_pipeline(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if state.pipeline_running.load(Ordering::SeqCst) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({
            "status": "already_running",
            "message": "Pipeline is already running"
        }))).into_response();
    }

    let tasks = state.storage.list_all_tasks().unwrap_or_default();
    let pending: Vec<_> = tasks.iter().filter(|t| {
        t.status != axon_core::TaskStatus::Completed
            && t.lifecycle_state != axon_core::TaskLifecycleState::Rejected
            && t.lifecycle_state != axon_core::TaskLifecycleState::Superseded
            && t.lifecycle_state != axon_core::TaskLifecycleState::Fatal
            && t.lifecycle_state != axon_core::TaskLifecycleState::Aborted
    }).collect();

    if pending.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!({"status": "idle", "message": "No pending tasks"}))).into_response();
    }

    let project_id = pending[0].project_id.clone();
    let sandbox_root = std::path::PathBuf::from(&project_id);

    let mut pipeline = ExecutionPipeline::new(
        state.axon_config.clone(),
        state.storage.clone(),
        state.event_bus.clone(),
        project_id.clone(),
        sandbox_root,
        state.agent_pool.clone(),
    )
    .with_pending_reviews(state.pending_reviews.clone())
    .with_running(state.pipeline_running.clone())
    .with_task_semaphore(state.task_semaphore.clone());
    pipeline.run_background();

    (StatusCode::OK, Json(serde_json::json!({
        "status": "resumed",
        "project_id": project_id,
        "pending_tasks": pending.len()
    }))).into_response()
}

async fn pause_pipeline(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.pipeline_running.store(false, Ordering::SeqCst);
    tracing::info!("⏸️ Pipeline pause requested via API.");
    (StatusCode::OK, Json(serde_json::json!({"status": "paused"})))
}

async fn submit_spec(
    State(state): State<Arc<AppState>>,
    Json(submission): Json<SpecSubmission>,
) -> impl IntoResponse {
    let mut bs = state.bootstrap_status.lock().await;
    if bs.is_running {
        return (StatusCode::CONFLICT, Json(SpecSubmissionResponse {
            status: "conflict".to_string(),
            message: "A bootstrap process is already running.".to_string(),
            project_id: None,
        })).into_response();
    }

    let project_id = format!("spec-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0000"));
    bs.is_running = true;
    bs.stage = "queued".to_string();
    bs.message = "Spec received. Starting bootstrap pipeline...".to_string();
    bs.is_complete = false;
    bs.error = None;
    bs.project_id = Some(project_id.clone());
    drop(bs);

    let config = state.axon_config.clone();
    let status = state.bootstrap_status.clone();
    let storage = state.storage.clone();
    let event_bus = state.event_bus.clone();
    let pending_approval = state.pending_approval.clone();
    let pending_reviews = state.pending_reviews.clone();

    let pipeline_config = config.clone();
    let pipeline_storage = storage.clone();
    let pipeline_eb = event_bus.clone();
    let pipeline_running = state.pipeline_running.clone();
    let pipeline_pool = state.agent_pool.clone();

    tokio::spawn(async move {
        let spec_path = "submitted_spec.md";
        if let Err(e) = std::fs::write(spec_path, &submission.content) {
            let mut bs = status.lock().await;
            bs.is_running = false;
            bs.error = Some(format!("Failed to write spec file: {}", e));
            return;
        }

        let manager = match BootstrapManager::with_shared_state(config, spec_path, storage, event_bus, Some(pending_approval)) {
            Ok(m) => m,
            Err(e) => {
                let mut bs = status.lock().await;
                bs.is_running = false;
                bs.error = Some(format!("Failed to initialize BootstrapManager: {}", e));
                return;
            }
        };

        {
            let mut bs = status.lock().await;
            bs.stage = "SpecAnalysis".to_string();
            bs.message = "Running specification analysis...".to_string();
        }

        match manager.run_v3(submission.content).await {
            Ok(()) => {
                let mut bs = status.lock().await;
                bs.stage = "complete".to_string();
                bs.message = "Bootstrap completed successfully.".to_string();
                bs.is_running = false;
                bs.is_complete = true;
                drop(bs);

                // ⏳ Flush all pending WAL ops to storage before pipeline reads tasks
                tracing::info!("⏳ Synchronizing bootstrap state with durable disk storage...");
                if let Err(e) = pipeline_storage.flush().await {
                    tracing::error!("❌ Critical Storage Flush Failed: {}", e);
                    let mut bs = status.lock().await;
                    bs.is_running = false;
                    bs.error = Some(format!("Storage flush failed: {}", e));
                    return;
                }
                tracing::info!("✅ WAL queue flushed. Launching pipeline safely.");

                // Start execution pipeline with shared pending_reviews
                let mut pipeline = ExecutionPipeline::new(
                    pipeline_config,
                    pipeline_storage,
                    pipeline_eb,
                    manager.project_id.clone(),
                    manager.sandbox_root.clone(),
                    pipeline_pool,
                )
                .with_pending_reviews(pending_reviews)
                .with_running(pipeline_running);
                pipeline.run_background();
            }
            Err(e) => {
                let mut bs = status.lock().await;
                bs.stage = "failed".to_string();
                bs.message = "Bootstrap failed.".to_string();
                bs.is_running = false;
                bs.error = Some(e);
            }
        }
    });

    (StatusCode::ACCEPTED, Json(SpecSubmissionResponse {
        status: "accepted".to_string(),
        message: "Spec received and bootstrap pipeline started.".to_string(),
        project_id: Some(project_id),
    })).into_response()
}

pub(crate) async fn setup_ingress(
    axon_config: AxonConfig,
    storage: Arc<Storage>,
    event_bus: Arc<EventBus>,
    spec_path: &str,
) -> Result<Arc<AppState>, String> {
    tracing::info!("HTTP Ingress starting...");

    let junior_count = axon_config.agents.juniors.len().max(1);
    
    let project_folder = std::path::Path::new(spec_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project")
        .to_string();
    
    let agent_pool = AgentPool {
        juniors: axon_config.agents.juniors.clone(),
        seniors: axon_config.agents.seniors.clone(),
        architect: axon_config.agents.architect.clone(),
    };
    
    let persona_registry = PersonaRegistry {
        personas: axon_config.agents.personas.clone(),
    };
    
    let state = Arc::new(AppState {
        axon_config,
        bootstrap_status: Arc::new(AsyncMutex::new(BootstrapStatus::default())),
        pending_approval: Arc::new(Mutex::new(None)),
        pending_reviews: Arc::new(Mutex::new(std::collections::HashMap::new())),
        pipeline_running: Arc::new(AtomicBool::new(false)),
        storage,
        event_bus,
        task_semaphore: Arc::new(Semaphore::new(junior_count)),
        agent_pool: Arc::new(AsyncRwLock::new(agent_pool)),
        persona_registry: Arc::new(AsyncRwLock::new(persona_registry)),
        project_folder,
    });

    // Recover pending_reviews from DB on startup
    {
        let storage = state.storage.clone();
        let reviews = state.pending_reviews.clone();
        if let Ok(tasks) = storage.list_all_tasks() {
            for task in tasks {
                if task.status == axon_core::TaskStatus::Failed 
                    && task.lifecycle_state == axon_core::TaskLifecycleState::Aborted
                    && task.boss_interventions > 0 
                {
                    if let Ok(thread) = storage.get_thread(&task.id) {
                        if let Some(th) = thread {
                            if th.status == axon_core::ThreadStatus::BossApproval {
                                let posts = storage.list_posts_by_thread(&task.id).unwrap_or_default();
                                let proposal = posts.iter().find(|p| matches!(p.post_type, axon_core::PostType::Proposal)).cloned();
                                let review = posts.iter().find(|p| matches!(p.post_type, axon_core::PostType::Review)).cloned();
                                let senior_feedback = review.as_ref()
                                    .and_then(|r| if r.content.is_empty() { None } else { Some(r.content.clone()) })
                                    .unwrap_or_else(|| task.error_feedback.clone().unwrap_or_default());
                                
                                let review_entry = crate::PipelineReview {
                                    task_id: task.id.clone(),
                                    task: task.clone(),
                                    proposal,
                                    review,
                                    senior_feedback,
                                };
                                reviews.lock().unwrap().insert(task.id.clone(), review_entry);
                                tracing::info!("🔄 Recovered pending review for task: {} ({})", task.title, task.id);
                            }
                        }
                    }
                }
            }
        }
        let recovered_count = reviews.lock().unwrap().len();
        if recovered_count > 0 {
            tracing::info!("✅ Recovered {} pending reviews from DB", recovered_count);
        }
    }

    // Recover pending_approval from file on startup
    {
        let spec_approval_file = std::path::PathBuf::from("spec/.axon_approval_pending");
        if spec_approval_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&spec_approval_file) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let approved = val.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
                    let rejected = val.get("rejected").and_then(|v| v.as_bool()).unwrap_or(false);
                    if !approved && !rejected {
                        let project_id = val.get("project_id").and_then(|v| v.as_str()).unwrap_or("spec").to_string();
                        let constraints_path = val.get("constraints_path").and_then(|v| v.as_str()).unwrap_or("spec/immutable_constraints.json").to_string();
                        let ambiguity = val.get("ambiguity_detected").and_then(|v| v.as_bool()).unwrap_or(false);
                        let components: Vec<String> = val.get("components").and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|c| c.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        let message = val.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        
                        let spec_approval = crate::PendingApproval {
                            project_id,
                            constraints_path,
                            approval_file_path: spec_approval_file.to_string_lossy().to_string(),
                            ambiguity_detected: ambiguity,
                            components,
                            approved: false,
                            rejected: false,
                        };
                        *state.pending_approval.lock().unwrap() = Some(spec_approval);
                        tracing::info!("🔄 Recovered pending spec approval from file: {}", message);
                    }
                }
            }
        }
    }

    // Auto-persist all EventBus events to storage
    let store = state.storage.clone();
    let mut rx = state.event_bus.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Err(e) = store.save_event(event).await {
                tracing::error!("Failed to persist event: {}", e);
            }
        }
    });

    let studio_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
        .join("../../studio/dist")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("studio/dist"));

    let app = Router::new()
        .route("/health", get(|| async { "AXON Daemon is alive and running!" }))
        .route("/api/status", get(get_status))
        .route("/api/tasks", get(list_tasks))
        .route("/api/threads", get(list_threads))
        .route("/api/threads/:thread_id/posts", get(list_posts))
        .route("/api/threads/:thread_id/approve", post(approve_thread))
        .route("/api/threads/:thread_id/reject", post(reject_thread))
        .route("/api/threads/:thread_id/retry", post(retry_thread))
        .route("/api/agents", get(list_agents_api))
        .route("/api/agents/hire", post(hire_agent))
        .route("/api/agents/:agent_id/fire", post(fire_agent))
        .route("/api/agents/:agent_id/swap-provider", post(swap_provider))
        .route("/api/events", get(list_events))
        .route("/api/specs", post(submit_spec))
        .route("/api/specs/status/:project_id", get(get_specs_status))
        .route("/api/specs/approval", get(get_pending_approval))
        .route("/api/specs/approve", post(approve_spec_analysis))
        .route("/api/specs/reject", post(reject_spec_analysis))
        .route("/api/semantics/risks", get(get_semantics_risks))
        .route("/api/semantics/decide", post(post_semantics_decide))
        .route("/api/pipeline/reviews", get(list_pipeline_reviews))
        .route("/api/pipeline/reviews/:task_id/approve", post(approve_pipeline_review))
        .route("/api/pipeline/reviews/:task_id/reject", post(reject_pipeline_review))
        .route("/api/pipeline/reviews/:task_id/retry", post(retry_pipeline_review))
        .route("/api/pause", post(pause_pipeline))
        .route("/api/resume", post(resume_pipeline))
        .route("/ws", get(ws_handler))
        .nest_service("/", ServeDir::new(studio_path))
        .with_state(state.clone());

    let listener = TcpListener::bind("0.0.0.0:8080").await.map_err(|e| e.to_string())?;

    tracing::info!("AXON Boss Board listening on http://localhost:8080");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Server error: {}", e);
        }
    });

    Ok(state)
}
