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
        .route("/api/threads", get(list_threads)) // Keep for backward compatibility or global view
        .route("/api/project/:project_id/threads", get(list_threads_by_project))
        .route("/api/threads/:id/posts", get(list_posts))
        .route("/api/threads/:id/approve", post(approve_thread))
        .route("/api/specs", post(submit_spec))
        .route("/api/project/:project_id/specs", post(submit_spec_by_project))
        .route("/api/tasks", get(list_tasks))
        .route("/api/pause", post(pause_daemon))
        .route("/api/resume", post(resume_daemon))
        .route("/api/status", get(get_status))
        .route("/api/agents", get(list_agents_api))
        .route("/api/agents/hire", post(hire_agent))
        .route("/api/agents/:id/fire", post(fire_agent))
        .nest_service("/", ServeDir::new(studio_path))
        .layer(cors)
        .with_state(daemon);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Studio UI available at http://localhost:8080");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
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

async fn list_tasks(
    State(daemon): State<Arc<Daemon>>,
) -> Json<Vec<Task>> {
    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();
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
    
    // Simulate spec parsing into tasks using LLM
    let prompt = format!(
        "PARSE THIS SPEC INTO TASKS (JSON ARRAY with fields: title, description):\n\n{}",
        submission.content
    );

    match daemon.architect_model.generate(prompt).await {
        Ok(tasks_json) => {
            // Very basic parsing attempt
            if let Ok(tasks_raw) = serde_json::from_str::<Vec<serde_json::Value>>(&tasks_json) {
                for t in tasks_raw {
                    let task = Task {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: project_id.clone(),
                        title: t["title"].as_str().unwrap_or("Untitled").into(),
                        description: t["description"].as_str().unwrap_or("").into(),
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
                "Spec processed and tasks queued".into_response()
            } else {
                // Fallback: create a single task from the submission
                let task = Task {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id.clone(),
                    title: "Parsed Task".into(),
                    description: submission.content,
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
                "Spec partially processed as a single task".into_response()
            }
        }
        Err(e) => {
            tracing::error!("LLM Error during spec parsing: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn approve_thread(
    Path(id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    let runnable = daemon.storage.list_all_threads().unwrap_or_default();
    if let Some(mut thread) = runnable.into_iter().find(|t| t.id == id) {
        thread.status = axon_core::ThreadStatus::Completed;
        let _ = daemon.storage.save_thread(&thread);
        
        // Lock-in architecture section (v0.0.16 Isolation Path applied)
        let _ = daemon.lock_in_architecture(&thread.project_id, &thread.title);

        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: thread.project_id.clone(),
            thread_id: Some(id),
            agent_id: None,
            event_type: axon_core::EventType::ApprovalGranted,
            source: "BOSS".to_string(),
            content: "BOSS approved the thread.".to_string(),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        
        "Approved".into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

async fn pause_daemon(
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    let _ = daemon.pause_tx.send(true);
    daemon.event_bus.publish(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: "system".to_string(),
        thread_id: None,
        agent_id: None,
        event_type: axon_core::EventType::SystemLog,
        source: "daemon".to_string(),
        content: "Daemon PAUSED by BOSS".to_string(),
        payload: None,
        timestamp: chrono::Local::now(),
    });
    "Paused".into_response()
}

async fn resume_daemon(
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    let _ = daemon.pause_tx.send(false);
    daemon.event_bus.publish(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: "system".to_string(),
        thread_id: None,
        agent_id: None,
        event_type: axon_core::EventType::SystemLog,
        source: "daemon".to_string(),
        content: "Daemon RESUMED by BOSS".to_string(),
        payload: None,
        timestamp: chrono::Local::now(),
    });
    "Resumed".into_response()
}

#[derive(serde::Serialize)]
struct StatusResponse {
    is_paused: bool,
    active_threads: usize,
}

async fn get_status(
    State(daemon): State<Arc<Daemon>>,
) -> Json<StatusResponse> {
    let is_paused = *daemon.pause_rx.borrow();
    let threads = daemon.storage.list_runnable_threads().unwrap_or_default();
    Json(StatusResponse {
        is_paused,
        active_threads: threads.len(),
    })
}

async fn list_agents_api(
    State(daemon): State<Arc<Daemon>>,
) -> Json<Vec<axon_core::Agent>> {
    let agents = daemon.storage.list_agents().unwrap_or_default();
    Json(agents)
}

#[derive(serde::Deserialize)]
struct HireRequest {
    role: axon_core::AgentRole,
    parent_id: Option<String>,
}

async fn hire_agent(
    State(daemon): State<Arc<Daemon>>,
    Json(req): Json<HireRequest>,
) -> impl IntoResponse {
    let agent_id = format!("agent-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let model = match req.role {
        axon_core::AgentRole::Architect => daemon.architect_model.clone(),
        axon_core::AgentRole::Senior => daemon.senior_model.clone(),
        axon_core::AgentRole::Junior => daemon.junior_model.clone(),
    };

    let mut runtime = axon_agent::AgentRuntime::new(
        agent_id.clone(),
        req.role,
        model,
    );
    runtime.agent.parent_id = req.parent_id;
    
    let _ = daemon.storage.save_agent(&runtime.agent);
    
    daemon.event_bus.publish(axon_core::Event {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: "system".to_string(),
        thread_id: None,
        agent_id: Some(agent_id.clone()),
        event_type: axon_core::EventType::AgentAssigned,
        source: "daemon".to_string(),
        content: format!("New agent {} hired as {:?}", runtime.agent.name, runtime.agent.role),
        payload: None,
        timestamp: chrono::Local::now(),
    });

    Json(runtime.agent)
}

async fn fire_agent(
    Path(id): Path<String>,
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    // Succession logic: Reassign children to the parent of the fired agent (or root)
    let agents = daemon.storage.list_agents().unwrap_or_default();
    let fired_agent = agents.iter().find(|a| a.id == id);
    
    if let Some(agent) = fired_agent {
        let new_parent = agent.parent_id.clone();
        let _ = daemon.storage.reassign_agents_by_parent(&id, new_parent.as_deref());
        let _ = daemon.storage.delete_agent(&id);
        
        daemon.event_bus.publish(axon_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "system".to_string(),
            thread_id: None,
            agent_id: Some(id.clone()),
            event_type: axon_core::EventType::SystemWarning,
            source: "daemon".to_string(),
            content: format!("Agent {} fired. Children reassigned.", agent.name),
            payload: None,
            timestamp: chrono::Local::now(),
        });
        
        axum::http::StatusCode::OK.into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(daemon): State<Arc<Daemon>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, daemon))
}

async fn handle_socket(mut socket: WebSocket, daemon: Arc<Daemon>) {
    tracing::info!("New WebSocket connection established");
    let mut rx = daemon.event_bus.subscribe();

    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                if let Ok(text) = serde_json::to_string(&event) {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
            }
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
    tracing::info!("WebSocket connection closed");
}
