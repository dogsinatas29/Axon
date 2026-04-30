use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: ThreadStatus,
    pub author: String,
    pub milestone_id: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ThreadStatus {
    Draft,
    Working,
    SeniorReview,
    Approved,
    PatchReady,
    BossApproval,
    Completed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Ready,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub thread_id: String,
    pub author_id: String, // Agent ID or "BOSS"
    pub content: String,
    pub full_code: Option<String>,
    pub post_type: PostType,
    pub metrics: Option<RuntimeMetrics>,
    pub created_at: DateTime<Local>,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PostType {
    Proposal,
    Review,
    Patch,
    Nogari,
    System,
    Instruction,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Architect,
    Senior,
    Junior,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentPersona {
    pub name: String,
    pub gender: String,          // e.g., "Male", "Female", "Non-binary"
    pub character_core: String, // e.g., "Principles-first Architect"
    pub prefixes: Vec<String>,   // e.g., ["Cynical", "Sharp"]
    pub suffixes: Vec<String>,   // e.g., ["Coffee-addict", "Perfectionist"]
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub persona: AgentPersona,
    pub model: String,
    pub status: String, // Idle, Working, Thinking
    pub parent_id: Option<String>,
    pub dtr: f32, // Dynamic Tension/Relaxation (0.0 to 1.0)
}

impl Agent {
    pub fn description(&self) -> String {
        format!(
            "CORE: {} | PREFIXES: {:?} | SUFFIXES: {:?}",
            self.persona.character_core,
            self.persona.prefixes,
            self.persona.suffixes
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventType {
    // Thread Events
    ThreadCreated,
    ThreadAssigned,
    ThreadStarted,
    ThreadCompleted,
    
    // Message/Post Events
    PostAdded,
    MessagePosted,
    
    // Artifact/Patch Events
    PatchCreated,
    ArtifactCreated,
    
    // Approval Events
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,
    
    // Agent Events
    AgentAction,
    AgentAssigned,
    AgentResponse,
    
    // System Events
    SystemLog,
    Signal,
    QuotaExceeded,
    SystemWarning,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: String,
    pub project_id: String,
    pub thread_id: Option<String>,
    pub agent_id: Option<String>,
    pub event_type: EventType,
    pub source: String, // e.g., "daemon", "agent_id", "dispatcher"
    pub content: String,
    pub payload: Option<serde_json::Value>,
    pub timestamp: DateTime<Local>,
}

pub mod protocol;
pub mod ir;

pub mod events {
    use super::Event;
    use tokio::sync::broadcast;

    pub struct EventBus {
        tx: broadcast::Sender<Event>,
    }

    impl EventBus {
        pub fn new(capacity: usize) -> Self {
            let (tx, _) = broadcast::channel(capacity);
            Self { tx }
        }

        pub fn subscribe(&self) -> broadcast::Receiver<Event> {
            self.tx.subscribe()
        }

        pub fn publish(&self, event: Event) {
            let _ = self.tx.send(event);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentValidation {
    pub id: String,
    pub role: String,
    pub status: String, // OK, WARN, FAIL
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationResult {
    pub agents: Vec<AgentValidation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Constraints {
    pub queue_limit: usize,
    pub sampling_rate: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionContext {
    pub agents: Vec<AgentValidation>,
    pub available_agents: std::collections::HashMap<String, Vec<AgentValidation>>,
    pub constraints: Constraints,
    pub warnings: Vec<AgentValidation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionResult {
    pub status: String, // RUNNING, BLOCKED
    pub result: String,
    pub path: Vec<(String, String)>, // (role, agent_id)
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RuntimeMetrics {
    pub total_duration: Option<u64>,
    pub eval_count: Option<u64>,
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMetric {
    pub id: String,
    pub role: String,
    pub status: String,
    pub latency_ms: f64,
    pub attempts: u32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExecutionSummary {
    pub worker_id: usize,
    pub total_duration_ms: f64,
    pub steps: usize,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct QueueStatus {
    pub length: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ObservabilityReport {
    pub agents: Vec<AgentMetric>,
    pub execution_path: Vec<(String, String)>,
    pub metrics: RuntimeMetrics,
    pub summary: ExecutionSummary,
    pub queue: QueueStatus,
    pub failures: Vec<String>,
}



