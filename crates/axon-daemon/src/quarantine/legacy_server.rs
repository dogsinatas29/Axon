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

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::{IntoResponse, Json},
    routing::{get, post},
    http::StatusCode,
    Router,
};
use futures_util::StreamExt;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::Daemon;
use axon_core::{Task, TaskStatus};

pub async fn start_server(daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Robust path resolution for Studio assets
    let possible_paths = vec![
        "studio/dist".to_string(),
        "../studio/dist".to_string(),
        "../../studio/dist".to_string(),
        "./studio/dist".to_string(),
    ];

    let mut studio_path = "studio/dist".to_string();
    let mut found = false;
    for path in possible_paths {
        if std::path::Path::new(&path).join("index.html").exists() {
            studio_path = path;
            found = true;
            break;
        }
    }

    if !found {
        let cwd = std::env::current_dir().unwrap_or_default();
        tracing::warn!("studio/dist/index.html not found! (CWD: {:?}). Studio UI will likely fail.", cwd);
    }

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/threads", get(list_threads)) 
        .route("/api/project/:project_id/threads", get(list_threads_by_project))
        .route("/api/threads/:id/posts", get(list_posts))
        .route("/api/threads/:id/approve", post(approve_thread))
        .route("/api/threads/:id/clarify", post(clarify_thread))
        .route("/api/specs", post(submit_spec))
        .route("/api/project/:project_id/specs", post(submit_spec_by_project))
        .route("/api/tasks", get(list_tasks))
        .route("/api/pause", post(pause_daemon))
        .route("/api/resume", post(resume_daemon))
        .route("/api/posts", get(list_posts))
        .route("/api/events", get(get_events))
        .route("/api/agents", get(list_agents_api))
        .route("/api/agents/hire", post(hire_agent))
        .route("/api/agents/:id/fire", post(fire_agent))
        .route("/submit", post(submit_task))
        .route("/api/tasks/:id", get(get_task_api))
        .route("/api/status", get(get_status))
        .route("/health", get(health_check))
        .route("/queue", get(get_queue_api))
        .route("/api/semantics/risks", get(get_semantic_risks))
        .route("/api/semantics/decide", post(submit_semantic_decision))
        .route("/api/approval/pending", get(get_pending_approval))
        .route("/api/approval/respond", post(respond_approval))
        .nest_service("/", ServeDir::new(studio_path))
        .layer(cors)
        .with_state(daemon);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    
    // v0.0.30: [PORT_HARDENING] Ensure immediate port recovery on restart
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    #[cfg(not(windows))]
    socket.set_reuse_port(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;
    socket.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(socket.into())?;
    
    tracing::info!("Studio UI available at http://localhost:9000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_threads(
    State(daemon): State<Arc<Daemon>>,
) -> Json<Vec<axon_core::Thread>> {
    let threads = daemon.storage.list_all_threads().unwrap_or_default();
    Json(threads)
}

async fn list_threads_by_project(
    Path(project_id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
) -> Json<Vec<axon_core::Thread>> {
    let threads = daemon.storage.list_all_threads().unwrap_or_default();
    let filtered = threads.into_iter().filter(|t| t.project_id == project_id).collect();
    Json(filtered)
}

async fn list_posts(
    Path(id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
) -> Json<Vec<axon_core::Post>> {
    let posts = daemon.storage.list_posts_by_thread(&id).unwrap_or_default();
    Json(posts)
}

async fn get_events(
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    let events = daemon.storage.list_events(100).unwrap_or_default();
    Json(events)
}

async fn list_tasks(
    State(daemon): State<Arc<Daemon>>,
) -> Json<Vec<Task>> {
    tracing::info!("🔍 [RADAR] Scanning global task database for semantic risks...");
    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();
    tracing::info!("📊 [RADAR] Total tasks found: {}", tasks.len());
    Json(tasks)
}

#[derive(serde::Deserialize)]
struct SpecSubmission {
    content: String,
}

async fn submit_spec(
    State(daemon): State<Arc<Daemon>>,
    Json(submission): Json<SpecSubmission>,
) -> impl IntoResponse {
    submit_spec_internal(daemon, "default-project".to_string(), submission).await
}

async fn submit_spec_by_project(
    Path(project_id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
    Json(submission): Json<SpecSubmission>,
) -> impl IntoResponse {
    submit_spec_internal(daemon, project_id, submission).await
}

async fn submit_spec_internal(
    daemon: Arc<Daemon>,
    project_id: String,
    submission: SpecSubmission,
) -> impl IntoResponse {
    tracing::info!("Received new spec submission for project: {}", project_id);
    
    let prompt = format!(
        "PARSE THIS SPEC INTO TASKS (JSON ARRAY with fields: title, description, target_file):\n\n{}",
        submission.content
    );

    match daemon.architect_model.generate(prompt).await {
        Ok(resp) => {
            let tasks_json = resp.text;
            if let Ok(tasks_raw) = serde_json::from_str::<Vec<serde_json::Value>>(&tasks_json) {
                for t in tasks_raw {
                    let task = Task {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: project_id.clone(),
                        title: t["title"].as_str().unwrap_or("Untitled").into(),
                        description: t["description"].as_str().unwrap_or("").into(),
                        status: TaskStatus::Pending,
                        dependencies: Vec::new(),
                        result: None,
                        target_file: t["target_file"].as_str().map(|s| s.to_string()),
                        lock_files: Vec::new(),
                        error_feedback: None,
                        senior_comment: None,
                        rework_count: 0,
                        base_hash: None,
                        parent_task: None,
                        reason: None,
                        kind: "rust".to_string(),
                        retries: 0,
                        assigned_worker: None,
                        created_at: chrono::Local::now(),
                        ir_path: None,
                        task_kind: if let Some(f) = t["target_file"].as_str() {
                            if f.ends_with(".h") { Some(axon_core::TaskKind::HeaderDecl) }
                            else if f.contains("main") { Some(axon_core::TaskKind::Integrator) }
                            else { Some(axon_core::TaskKind::SourceImpl) }
                        } else { None },
                        signature: None,
                    };
                    let _ = daemon.storage.save_task(task.clone()).await;
                    
                    let thread = axon_core::Thread {
                        id: task.id.clone(),
                        project_id: task.project_id.clone(),
                        title: task.title.clone(),
                        status: axon_core::ThreadStatus::Draft,
                        author: "Architect".to_string(),
                        milestone_id: None,
                        task_kind: task.task_kind.clone(),
                        rejection_count: 0,
                        created_at: task.created_at,
                        updated_at: task.created_at,
                    };
                    let _ = daemon.storage.save_thread(thread).await;

                    let new_post = axon_core::Post {
                        id: uuid::Uuid::new_v4().to_string(),
                        thread_id: task.id.clone(),
                        author_id: "Architect".to_string(),
                        content: task.description.clone(),
                        thought: None,
                        full_code: None,
                        post_type: axon_core::PostType::Instruction,
                        metrics: None,
                        created_at: task.created_at,
                    };
                    let _ = daemon.storage.save_post(new_post).await;
                    
                    daemon.publish_event(axon_core::Event {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: task.project_id.clone(),
                        thread_id: Some(task.id.clone()),
                        agent_id: None,
                        event_type: axon_core::EventType::ThreadCreated,
                        level: axon_core::EventLevel::Info,
                        source: "architect".to_string(),
                        content: format!("🆕 New strategic thread created: {}", task.title),
                        payload: None,
                        timestamp: chrono::Local::now(),
                    });

                    if let Err(e) = daemon.dispatcher.enqueue_task(task) {
                        tracing::error!("❌ [QUEUE_REJECTED] via API: {}", e);
                    }
                }
                "Spec processed and tasks queued".into_response()
            } else {
                let task = Task {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id.clone(),
                    title: "Parsed Task".into(),
                    description: submission.content,
                    status: TaskStatus::Pending,
                    dependencies: Vec::new(),
                    result: None,
                    target_file: Some("pending_recovery.rs".to_string()),
                    lock_files: Vec::new(),
                    error_feedback: None,
                    senior_comment: None,
                    rework_count: 0,
                    base_hash: None,
                    parent_task: None,
                    reason: None,
                    kind: "rust".to_string(),
                    retries: 0,
                    assigned_worker: None,
                    created_at: chrono::Local::now(),
                    ir_path: None,
                    task_kind: None,
                    signature: None,
                };
                let _ = daemon.storage.save_task(task.clone()).await;

                let thread = axon_core::Thread {
                    id: task.id.clone(),
                    project_id: task.project_id.clone(),
                    title: task.title.clone(),
                    status: axon_core::ThreadStatus::Draft,
                    author: "Architect".to_string(),
                    milestone_id: None,
                    task_kind: None,
                    rejection_count: 0,
                    created_at: task.created_at,
                    updated_at: task.created_at,
                };
                let _ = daemon.storage.save_thread(thread).await;

                let new_post = axon_core::Post {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: task.id.clone(),
                    author_id: "Architect".to_string(),
                    content: task.description.clone(),
                    thought: None,
                    full_code: None,
                    post_type: axon_core::PostType::Instruction,
                    metrics: None,
                    created_at: task.created_at,
                };
                let _ = daemon.storage.save_post(new_post).await;

                if let Err(e) = daemon.dispatcher.enqueue_task(task) {
                    tracing::error!("❌ [QUEUE_REJECTED] via API: {}", e);
                }
                "Spec partially processed as a single task".into_response()
            }
        }
        Err(e) => {
            tracing::error!("LLM Error during spec parsing: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(serde::Deserialize)]
struct ClarifyRequest {
    command: String,
}

async fn clarify_thread(
    Path(id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
    Json(payload): Json<ClarifyRequest>,
) -> impl IntoResponse {
    tracing::info!("🚨 [BOSS_INTERRUPT] Received clarification for thread: {}", id);
    if let Ok(Some(mut task)) = daemon.storage.get_task(&id) {
        let clarification_msg = format!("\n\n--- 🚨 BOSS CLARIFICATION ---\n{}\n----------------------------\n", payload.command);
        let mut feedback = task.error_feedback.unwrap_or_default();
        feedback.push_str(&clarification_msg);
        task.error_feedback = Some(feedback);
        task.status = TaskStatus::Pending;
        task.rework_count = 0;
        if let Err(e) = daemon.storage.save_task(task.clone()).await {
            tracing::error!("Failed to save clarified task: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        if let Ok(Some(mut thread)) = daemon.storage.get_thread(&id) {
            thread.status = axon_core::ThreadStatus::Working;
            let _ = daemon.storage.save_thread(thread).await;
        }
        if let Err(e) = daemon.dispatcher.enqueue_task(task) {
            tracing::error!("Failed to re-enqueue clarified task: {}", e);
        }
        daemon.publish_event(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "system".to_string(),
            thread_id: Some(id),
            agent_id: None,
            event_type: axon_core::EventType::AgentAction,
            level: axon_core::EventLevel::Warning,
            source: "BOSS".to_string(),
            content: format!("Boss issued a direct clarification: {}", payload.command),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        "Clarified".into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn approve_thread(
    Path(id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    if let Ok(Some(mut thread)) = daemon.storage.get_thread(&id) {
        thread.status = axon_core::ThreadStatus::Completed;
        if let Err(e) = daemon.storage.save_thread(thread.clone()).await {
             tracing::error!("❌ Failed to save thread: {}", e);
             return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        let _ = daemon.lock_in_architecture(&thread.project_id, &thread.title);
        let _ = daemon.seal_semantic_risks(&thread.id).await;
        daemon.publish_event(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: thread.project_id.clone(),
            thread_id: Some(id),
            agent_id: None,
            event_type: axon_core::EventType::ApprovalGranted,
            level: axon_core::EventLevel::Info,
            source: "BOSS".to_string(),
            content: "BOSS가 스레드를 승인했습니다.".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        "Approved".into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

async fn pause_daemon(State(daemon): State<Arc<Daemon>>) -> impl IntoResponse {
    let _ = daemon.pause_tx.send(true);
    daemon.publish_event(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: "system".to_string(),
        thread_id: None,
        agent_id: None,
        event_type: axon_core::EventType::SystemLog,
        level: axon_core::EventLevel::Info,
        source: "daemon".to_string(),
        content: "Daemon PAUSED by BOSS".to_string(),
        payload: None,
        timestamp: chrono::Local::now(),
    });
    "Paused".into_response()
}

async fn resume_daemon(State(daemon): State<Arc<Daemon>>) -> impl IntoResponse {
    let _ = daemon.pause_tx.send(false);
    daemon.publish_event(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: "system".to_string(),
        thread_id: None,
        agent_id: None,
        event_type: axon_core::EventType::SystemLog,
        level: axon_core::EventLevel::Info,
        source: "daemon".to_string(),
        content: "Daemon RESUMED by BOSS".to_string(),
        payload: None,
        timestamp: chrono::Local::now(),
    });
    "Resumed".into_response()
}

#[derive(serde::Serialize)]
struct StatusResponse {
    is_running: bool,
    active_threads: usize,
    total_signals: usize,
    locale: String,
}

async fn get_status(State(daemon): State<Arc<Daemon>>) -> Json<StatusResponse> {
    let is_paused = *daemon.pause_rx.borrow();
    let threads = daemon.storage.list_runnable_threads().unwrap_or_default();
    let strategic_threads = threads.iter().filter(|t| {
        let is_lounge = t.id == "lounge" || t.title.contains("Lounge");
        let is_system = t.project_id == "system" || t.project_id == "AXON-SYSTEM";
        !is_lounge && !is_system
    }).count();
    let events = daemon.storage.list_events(1000).unwrap_or_default();
    let process_signals = events.iter().filter(|e| e.event_type != axon_core::EventType::MessagePosted).count();
    Json(StatusResponse {
        is_running: !is_paused,
        active_threads: strategic_threads,
        total_signals: process_signals,
        locale: daemon.locale.clone(),
    })
}

async fn list_agents_api(State(daemon): State<Arc<Daemon>>) -> Json<Vec<axon_core::Agent>> {
    let agents = daemon.storage.list_agents().unwrap_or_default();
    Json(agents)
}

#[derive(serde::Deserialize)]
struct HireRequest {
    role: axon_core::AgentRole,
    parent_id: Option<String>,
}

async fn hire_agent(State(daemon): State<Arc<Daemon>>, Json(req): Json<HireRequest>) -> impl IntoResponse {
    let agent_id = format!("agent-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (model, model_name) = match req.role {
        axon_core::AgentRole::Architect => (daemon.architect_model.clone(), daemon.architect_model_name.clone()),
        axon_core::AgentRole::Senior => (
            daemon.senior_models.first().cloned().unwrap_or_else(|| daemon.architect_model.clone()),
            daemon.senior_model_names.first().cloned().unwrap_or_else(|| daemon.architect_model_name.clone())
        ),
        axon_core::AgentRole::Junior => (
            daemon.junior_models.first().cloned().unwrap_or_else(|| daemon.architect_model.clone()),
            daemon.junior_model_names.first().cloned().unwrap_or_else(|| daemon.architect_model_name.clone())
        ),
    };
    let mut runtime = axon_agent::AgentRuntime::new(agent_id.clone(), req.role, model_name, model);
    runtime.agent.parent_id = req.parent_id;
    let _ = daemon.storage.save_agent(runtime.agent.clone()).await;
    daemon.publish_event(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: "system".to_string(),
        thread_id: None,
        agent_id: Some(agent_id.clone()),
        event_type: axon_core::EventType::AgentAssigned,
        level: axon_core::EventLevel::Info,
        source: "daemon".to_string(),
        content: format!("New agent {} hired as {:?}", runtime.agent.name, runtime.agent.role),
        payload: None,
        timestamp: chrono::Local::now(),
    });
    Json(runtime.agent)
}

async fn fire_agent(Path(id): Path<String>, State(daemon): State<Arc<Daemon>>) -> impl IntoResponse {
    let agents = daemon.storage.list_agents().unwrap_or_default();
    let fired_agent = agents.iter().find(|a| a.id == id);
    if let Some(agent) = fired_agent {
        let same_role_count = agents.iter().filter(|a| a.role == agent.role).count();
        if same_role_count <= 1 && agent.role != axon_core::AgentRole::Architect {
            return (axum::http::StatusCode::BAD_REQUEST, "Cannot fire last agent of role.").into_response();
        }
        let children_to_fire: Vec<String> = agents.iter().filter(|a| a.parent_id.as_deref() == Some(&id)).map(|a| a.id.clone()).collect();
        for child_id in children_to_fire { let _ = daemon.storage.delete_agent(child_id).await; }
        let _ = daemon.storage.delete_agent(id.clone()).await;
        daemon.publish_event(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "system".to_string(),
            thread_id: None,
            agent_id: Some(id.clone()),
            event_type: axon_core::EventType::SystemLog,
            level: axon_core::EventLevel::Info,
            source: "daemon".to_string(),
            content: format!("Agent {} fired.", agent.name),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        axum::http::StatusCode::OK.into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(daemon): State<Arc<Daemon>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, daemon))
}

async fn handle_socket(mut socket: WebSocket, daemon: Arc<Daemon>) {
    let mut rx = daemon.event_bus.subscribe();
    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                if let Ok(text) = serde_json::to_string(&event) {
                    if socket.send(Message::Text(text)).await.is_err() { break; }
                }
            }
            msg = socket.next() => {
                match msg { Some(Ok(Message::Close(_))) | None => break, _ => {} }
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct TaskRequest {
    pub task: String,
    pub project_id: Option<String>,
    pub target_file: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SubmitResponse {
    pub status: String,
    pub task_id: String,
    pub queue_size: usize,
}

async fn submit_task(State(daemon): State<Arc<Daemon>>, Json(req): Json<TaskRequest>) -> Json<SubmitResponse> {
    let project_id = req.project_id.unwrap_or_else(|| "default-project".to_string());
    let task_id = uuid::Uuid::new_v4().to_string();
    let task = Task {
        id: task_id.clone(),
        project_id,
        title: "API Task".to_string(),
        description: req.task,
        status: TaskStatus::Pending,
        dependencies: Vec::new(),
        result: None,
        target_file: Some(req.target_file.unwrap_or_else(|| "manual_task.rs".to_string())),
        lock_files: Vec::new(),
        error_feedback: None,
        senior_comment: None,
        rework_count: 0,
        base_hash: None,
        parent_task: None,
        reason: None,
        kind: "rust".to_string(),
        retries: 0,
        assigned_worker: None,
        created_at: chrono::Local::now(),
        ir_path: None,
        task_kind: None,
        signature: None,
    };
    let _ = daemon.storage.save_task(task.clone()).await;
    let _ = daemon.dispatcher.enqueue_task(task);
    Json(SubmitResponse { status: "ACCEPTED".to_string(), task_id, queue_size: daemon.dispatcher.len() })
}

async fn get_task_api(Path(id): Path<String>, State(daemon): State<Arc<Daemon>>) -> impl IntoResponse {
    match daemon.storage.get_task(&id) {
        Ok(Some(task)) => Json(task).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn health_check() -> Json<serde_json::Value> { Json(serde_json::json!({ "status": "OK" })) }

#[derive(serde::Serialize)]
pub struct QueueResponse { pub length: usize, pub limit: usize }
async fn get_queue_api(State(daemon): State<Arc<Daemon>>) -> Json<QueueResponse> {
    Json(QueueResponse { length: daemon.dispatcher.len(), limit: daemon.dispatcher.limit() })
}


async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {
    let mut risks: Vec<serde_json::Value> = Vec::new();
    let root_path = std::env::current_dir().unwrap_or_default();
    let mut stack = vec![root_path];
    let mut visited_count = 0;
    let mut global_ir: Option<axon_ir::ProjectIR> = None;

    while let Some(path) = stack.pop() {
        visited_count += 1;
        if visited_count > 100 { break; }
        let approval_file = path.join(".axon_approval_pending");
        if approval_file.exists() {
             let (actor, cause, expected, detected, recommend) = if daemon.locale == "ko_KR" {
                 ("Sovereign Gatekeeper", "새로운 명세가 감지되었습니다. 제조 공정 시작을 위해 보스의 승인이 필요합니다.", "승인된 명세 (Authorized Specification)", "설계 초안 대기 중 (New Design Draft)", "[승인 & 봉인] 버튼을 눌러 공정 시작을 승인하십시오.")
             } else if daemon.locale == "ja_JP" {
                 ("統治官 (Sovereign Gatekeeper)", "新しい仕様が検出されました。製造工程を開始するには社長の承認が必要です。", "承認された仕様 (Authorized Specification)", "設計ドラ프트待機中 (New Design Draft)", "[承認 & 封印] ボタンを押して工程の開始を承認してください。")
             } else {
                 ("Sovereign Gatekeeper", "New specification detected. Boss approval required to start manufacturing.", "Authorized Specification", "New Design Draft awaiting approval", "Click [OVERRIDE & SEAL] to approve.")
             };

             risks.push(serde_json::json!({
                 "risk_id": "pending_approval",
                 "kind": "Bootstrap",
                 "level": "Critical",
                 "target": "Factory Gateway",
                 "failed_stage": "SpecAnalysis",
                 "actor": actor,
                 "cause": cause,
                 "expected": expected,
                 "detected": detected,
                 "recommendation": recommend,
                 "component": "GATEWAY",
             }));
        }
        if let Some(ir) = crate::intelligence::decision::load_project_ir(&path.to_string_lossy()) {
            global_ir = Some(ir.clone());
            let extractor = crate::intelligence::semantic_debugger::SemanticRiskExtractor::new(&path.to_string_lossy());
            let extracted = extractor.extract_risks(&ir).await;
            for risk in extracted.risks { risks.push(serde_json::to_value(risk).unwrap()); }
        }
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() && !entry.file_name().to_string_lossy().starts_with('.') && entry.file_name() != "target" { stack.push(entry.path()); }
            }
        }
    }

    tracing::info!("🔍 [RADAR] Scanning global task database for semantic risks...");
    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();
    tracing::info!("📊 [RADAR] Total tasks found: {}", tasks.len());
    for task in tasks {
        let thread_rejection = daemon.storage.get_thread(&task.id).ok().flatten().map(|t| t.rejection_count).unwrap_or(0);
        if (task.rework_count >= 3 || thread_rejection >= 3) && task.status != axon_core::TaskStatus::Completed {
            let posts = daemon.storage.list_posts_by_thread(&task.id).unwrap_or_default();
            let error_post = posts.iter().rev().find(|p| p.author_id != "BOSS" && (p.content.to_lowercase().contains("error") || p.content.to_lowercase().contains("reject") || p.content.to_lowercase().contains("fail")));
            let raw_log = error_post.map(|p| p.content.clone()).unwrap_or_else(|| task.error_feedback.clone().unwrap_or_else(|| "Unknown Failure".to_string()));
            let last_code = posts.iter().rev().find(|p| p.full_code.is_some()).and_then(|p| p.full_code.clone());
            
            let mut actor = if daemon.locale == "ja_JP" { "契約検証官" } else { "Contract Verifier" };
            let mut failed_stage = if daemon.locale == "ja_JP" { "契約検証" } else { "Contract Verification" };
            if raw_log.contains("error:") || raw_log.contains("cmake") {
                actor = if daemon.locale == "ja_JP" { "コンパイラ (Clang/GCC)" } else { "Compiler (Clang/GCC)" };
                failed_stage = if daemon.locale == "ja_JP" { "ビル드/링크" } else { "Build/Linking" };
            } else if raw_log.contains("SENIOR_REJECT") || raw_log.contains("Review") || raw_log.contains("rejected") {
                actor = if daemon.locale == "ko_KR" { "시니어 AI 감사관" } else if daemon.locale == "ja_JP" { "シニアAI監査役" } else { "Senior AI Auditor" };
                failed_stage = if daemon.locale == "ko_KR" { "의미론적 리뷰" } else if daemon.locale == "ja_JP" { "意味論적レビュー" } else { "Semantic Review" };
            }

            // v0.0.30: Consolidated rejection info
            let max_rejection = if thread_rejection > task.rework_count { thread_rejection } else { task.rework_count };
            let rejection_summary = if daemon.locale == "ko_KR" {
                format!("🚨 {}회 반려됨 (임계값 초과)", max_rejection)
            } else {
                format!("🚨 {} REJECTIONS (Threshold Exceeded)", max_rejection)
            };

            let mut target_line = -1;
            if let Some(caps) = regex::Regex::new(r"[:\s](\d+)[:\s]").ok().and_then(|re| re.captures(&raw_log)) {
                target_line = caps.get(1).and_then(|m| m.as_str().parse::<i32>().ok()).unwrap_or(-1);
            }

            // v0.0.30: Deep Contract Mining from IR
            let mut expected = if daemon.locale == "ko_KR" { "설계 규약을 찾을 수 없음" } else { "Design Contract Not Found" }.to_string();
            if let Some(ref ir) = global_ir {
                if let Some(target_file) = &task.target_file {
                    for comp in ir.components.values() {
                        let target_name = std::path::Path::new(target_file).file_name().and_then(|n| n.to_str()).unwrap_or(target_file);
                        let comp_name_path = std::path::Path::new(&comp.file_path).file_name().and_then(|n| n.to_str()).unwrap_or(&comp.file_path);
                        if &comp.file_path == target_file || comp_name_path == target_name {
                            expected = format!("Component: {}\nTier: {:?}\nContracts: {}", 
                                comp.name, comp.tier, 
                                comp.functions.keys().cloned().collect::<Vec<_>>().join(", "));
                            break;
                        }
                    }
                }
            }

            let detected = if raw_log.len() > 150 { format!("{}...", &raw_log[..150]) } else { raw_log.clone() };

            risks.push(serde_json::json!({
                "risk_id": format!("rejection_limit_{}", task.id),
                "kind": "ImplementationFail",
                "level": "Critical",
                "target": task.title,
                "actor": actor,
                "failed_stage": failed_stage,
                "cause": format!("{} | {}", rejection_summary, raw_log),
                "expected": expected,
                "detected": detected,
                "target_line": target_line,
                "component": task.target_file.clone().unwrap_or_else(|| "unknown".to_string()),
                "full_code": last_code,
                "task_id": task.id,
            }));
        }
    }
    Json(serde_json::json!({ "risks": risks, "locale": daemon.locale }))
}

async fn submit_semantic_decision(
    State(daemon): State<Arc<Daemon>>,
    Json(decision): Json<serde_json::Value>,
) -> impl IntoResponse {
    let risk_id = decision["risk_id"].as_str().unwrap_or_default();
    let action = decision["action"].as_str().unwrap_or("SEAL");
    let comment = decision["comment"].as_str().unwrap_or("");

    tracing::info!("⚖️ [BOSS_DECISION] Risk: {}, Action: {}", risk_id, action);

    if risk_id == "pending_approval" {
        let root = std::env::current_dir().unwrap_or_default();
        let mut stack = vec![root];
        while let Some(path) = stack.pop() {
            let approval_file = path.join(".axon_approval_pending");
            if approval_file.exists() {
                if action == "SEAL" || action == "Approve" {
                    if let Ok(content) = std::fs::read_to_string(&approval_file) {
                        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                            val["approved"] = serde_json::json!(true);
                            if let Ok(json) = serde_json::to_string_pretty(&val) {
                                let _ = std::fs::write(&approval_file, json);
                                tracing::info!("✅ [SOVEREIGN_SEAL] Seal applied to approval file. Factory proceeding...");
                            }
                        }
                    }
                } else {
                    let _ = std::fs::write(path.join(".axon_rejected"), "Boss rejected bootstrap");
                }
            }
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() && !entry.file_name().to_string_lossy().starts_with('.') { stack.push(entry.path()); }
                }
            }
        }
        return StatusCode::OK.into_response();
    }

    if risk_id.starts_with("rejection_limit_") {
        let task_id = &risk_id["rejection_limit_".len()..];
        let boss_code = decision["code"].as_str();

        if let Ok(Some(mut task)) = daemon.storage.get_task(task_id) {
            if action == "SEAL" {
                // Direct Boss Intervention: If code is provided, write it to file
                if let (Some(code), Some(rel_path)) = (boss_code, &task.target_file) {
                    let target_path = std::path::Path::new(&task.project_id).join(rel_path);
                    
                    tracing::info!("✍️ [BOSS_OVERRIDE] Writing direct code override to {:?} (Length: {} bytes)", target_path, code.len());
                    if let Err(e) = std::fs::write(&target_path, code) {
                        tracing::error!("❌ [BOSS_OVERRIDE_FAIL] Failed to write file {:?}: {}", target_path, e);
                    }
                }

                task.status = axon_core::TaskStatus::Completed;
                task.error_feedback = Some(format!("[BOSS_OVERRIDE]: {}", comment));
                let _ = daemon.storage.save_task(task).await;
                tracing::info!("✅ [BOSS_SEALED] Task {} force-completed.", task_id);
            } else if action == "REWORK" {
                task.rework_count = 0;
                task.status = axon_core::TaskStatus::Pending;
                task.senior_comment = Some(format!("[BOSS_HINT]: {}", comment));
                let _ = daemon.storage.save_task(task.clone()).await;
                let _ = daemon.dispatcher.enqueue_task(task);
                tracing::info!("🔄 [BOSS_REWORK] Task {} re-queued with hint.", task_id);
            } else {
                task.status = axon_core::TaskStatus::Failed;
                let _ = daemon.storage.save_task(task).await;
                tracing::info!("🛑 [BOSS_CANCELLED] Task {} marked as failed.", task_id);
            }
            return StatusCode::OK.into_response();
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn get_pending_approval(State(_daemon): State<Arc<Daemon>>) -> impl IntoResponse {
    let root_path = std::env::current_dir().unwrap_or_default();
    let mut stack = vec![root_path];
    while let Some(path) = stack.pop() {
        let approval_file = path.join(".axon_approval_pending");
        if approval_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&approval_file) {
                if let Ok(approval) = serde_json::from_str::<serde_json::Value>(&content) {
                    if approval["approved"].as_bool() == Some(false) { return Json(approval).into_response(); }
                }
            }
        }
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() && !entry.file_name().to_string_lossy().starts_with('.') { stack.push(entry.path()); }
            }
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn respond_approval(State(daemon): State<Arc<Daemon>>, Json(decision): Json<serde_json::Value>) -> impl IntoResponse {
    submit_semantic_decision(State(daemon), Json(decision)).await
}
