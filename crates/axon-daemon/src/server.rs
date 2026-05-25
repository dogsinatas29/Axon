use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Path}, response::IntoResponse, http::StatusCode,
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
};
use serde::{Serialize, Deserialize};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex as AsyncMutex;
use tower_http::services::ServeDir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use crate::AxonConfig;
use crate::bootstrap::BootstrapManager;
use crate::events::EventBus;
use crate::pipeline::ExecutionPipeline;
use crate::{PendingApproval, PipelineReview};
use axon_storage::Storage;

#[derive(Clone)]
pub(crate) struct AppState {
    pub axon_config: AxonConfig,
    pub bootstrap_status: Arc<AsyncMutex<BootstrapStatus>>,
    pub pending_approval: Arc<Mutex<Option<PendingApproval>>>,
    pub pending_reviews: Arc<Mutex<std::collections::HashMap<String, PipelineReview>>>,
    pub pipeline_running: Arc<AtomicBool>,
    pub storage: Arc<Storage>,
    pub event_bus: Arc<EventBus>,
}

#[derive(Serialize, Deserialize, Clone)]
struct BootstrapStatus {
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

            (StatusCode::OK, Json(serde_json::json!({"status": "approved", "thread_id": thread_id}))).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this thread"}))).into_response(),
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
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this thread"}))).into_response(),
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
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this thread"}))).into_response(),
    }
}

async fn hire_agent(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("Hiring agent: {:?}", body);
    (StatusCode::OK, Json(serde_json::json!({"status": "hired", "id": uuid::Uuid::new_v4().to_string()})))
}

async fn fire_agent(
    State(_state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("Firing agent: {}", agent_id);
    (StatusCode::OK, Json(serde_json::json!({"status": "fired"})))
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
    let mut approval = state.pending_approval.lock().unwrap();
    match approval.as_mut() {
        Some(ref mut p) => {
            p.approved = true;
            // Also write to the file in case bootstrap polls it
            let path = format!("{}/.axon_approval_pending", p.project_id);
            let content = serde_json::json!({"status": "APPROVED", "approved": true});
            let _ = std::fs::write(&path, serde_json::to_string(&content).unwrap());
            (StatusCode::OK, Json(serde_json::json!({"status": "approved"}))).into_response()
        }
        None => (StatusCode::CONFLICT, "No pending approval").into_response(),
    }
}

async fn reject_spec_analysis(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut approval = state.pending_approval.lock().unwrap();
    match approval.as_mut() {
        Some(ref mut p) => {
            p.rejected = true;
            let path = format!("{}/.axon_approval_pending", p.project_id);
            let content = serde_json::json!({"status": "REJECTED", "approved": false});
            let _ = std::fs::write(&path, serde_json::to_string(&content).unwrap());
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

            (StatusCode::OK, Json(serde_json::json!({"status": "retrying", "task_id": task_id}))).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No pending review for this task"}))).into_response(),
    }
}

async fn resume_pipeline(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let tasks = state.storage.list_all_tasks().unwrap_or_default();
    let pending: Vec<_> = tasks.iter().filter(|t| {
        t.status != axon_core::TaskStatus::Completed
            && t.lifecycle_state != axon_core::TaskLifecycleState::Rejected
            && t.lifecycle_state != axon_core::TaskLifecycleState::Fatal
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
    )
    .with_pending_reviews(state.pending_reviews.clone())
    .with_running(state.pipeline_running.clone());
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

pub async fn setup_ingress(
    axon_config: AxonConfig,
    storage: Arc<Storage>,
    event_bus: Arc<EventBus>,
) -> Result<Arc<AppState>, String> {
    tracing::info!("HTTP Ingress starting...");

    let state = Arc::new(AppState {
        axon_config,
        bootstrap_status: Arc::new(AsyncMutex::new(BootstrapStatus::default())),
        pending_approval: Arc::new(Mutex::new(None)),
        pending_reviews: Arc::new(Mutex::new(std::collections::HashMap::new())),
        pipeline_running: Arc::new(AtomicBool::new(false)),
        storage,
        event_bus,
    });

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
