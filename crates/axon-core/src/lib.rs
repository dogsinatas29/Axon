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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
