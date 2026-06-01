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
    pub task_kind: Option<LanguageTaskKind>,
    pub rejection_count: u32,
    pub validator_rejections: u32,
    pub senior_rejections: u32,
    pub architecture_rejections: u32,
    pub cargo_rejections: u32,
    pub lsp_rejections: u32,
    pub boss_interventions: u32,
    pub error_feedback: Option<String>,
    pub reason: Option<String>,
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
    AwaitDependency,
}

/// v0.0.31.xx: Lifecycle state for scheduler semantics (orchestration concern)
/// This separates scheduler concerns from business semantics (TaskStatus)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskLifecycleState {
    Queued,     // 대기 중 - awaiting dispatch
    Running,    // 실행 중 - currently being processed
    Completed,  // 정상 완료
    Rejected,   // validation 실패로 종료
    Superseded, // Boss intervention으로 교체됨
    Aborted,    // early abort (FROZEN_GATE, SOVEREIGN_PROTECTION 등)
    Fatal,      // v0.0.31.xx: fatal failure (e.g. max reworks reached), halting task entirely
}

impl TaskLifecycleState {
    pub fn is_active(&self) -> bool {
        matches!(self, TaskLifecycleState::Queued | TaskLifecycleState::Running)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskLifecycleState::Completed 
            | TaskLifecycleState::Rejected 
            | TaskLifecycleState::Superseded 
            | TaskLifecycleState::Aborted
            | TaskLifecycleState::Fatal
        )
    }

    pub fn is_governance_terminal(&self) -> bool {
        matches!(self, TaskLifecycleState::Superseded | TaskLifecycleState::Aborted)
    }
}

impl Default for TaskLifecycleState {
    fn default() -> Self {
        TaskLifecycleState::Queued
    }
}

/// v0.0.31.xx: Runtime events for observability and replayability
/// Append-only event log enables runtime replay and forensic debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    TaskQueued {
        task_id: String,
        project_id: String,
        target_file: Option<String>,
        timestamp: DateTime<Local>,
    },
    TaskDispatched {
        task_id: String,
        worker_id: String,
        timestamp: DateTime<Local>,
    },
    TaskTerminated {
        task_id: String,
        disposition: String,
        new_state: TaskLifecycleState,
        timestamp: DateTime<Local>,
    },
    RecoveryStarted {
        epoch: u64,
        active_tasks_from_db: usize,
        timestamp: DateTime<Local>,
    },
    RecoveryCompleted {
        epoch: u64,
        reconstructed_queues: usize,
        timestamp: DateTime<Local>,
    },
    InvariantBroken {
        invariant_name: String,
        details: String,
        timestamp: DateTime<Local>,
    },
    StageAdvanced {
        from_stage: String,
        to_stage: String,
        active_count: usize,
        timestamp: DateTime<Local>,
    },
    QueueScrub {
        removed_count: usize,
        reason: String,
        timestamp: DateTime<Local>,
    },
    ActiveFileOrphaned {
        file: String,
        timestamp: DateTime<Local>,
    },
    ActiveFileScrubbed {
        file: String,
        timestamp: DateTime<Local>,
    },
}

impl RuntimeEvent {
    /// Log the event to tracing for observability
    pub fn log(&self) {
        match self {
            RuntimeEvent::TaskQueued { task_id, project_id, target_file, timestamp } => {
                tracing::debug!("📝 [EVENT] TaskQueued task={} project={} file={:?} at {}", 
                    task_id, project_id, target_file, timestamp);
            }
            RuntimeEvent::TaskDispatched { task_id, worker_id, timestamp } => {
                tracing::debug!("🚀 [EVENT] TaskDispatched task={} worker={} at {}", 
                    task_id, worker_id, timestamp);
            }
            RuntimeEvent::TaskTerminated { task_id, disposition, new_state, timestamp } => {
                tracing::info!("🔚 [EVENT] TaskTerminated task={} disposition={} state={:?} at {}", 
                    task_id, disposition, new_state, timestamp);
            }
            RuntimeEvent::RecoveryStarted { epoch, active_tasks_from_db, timestamp } => {
                tracing::info!("🚑 [EVENT] RecoveryStarted epoch={} tasks={} at {}", 
                    epoch, active_tasks_from_db, timestamp);
            }
            RuntimeEvent::RecoveryCompleted { epoch, reconstructed_queues, timestamp } => {
                tracing::info!("✅ [EVENT] RecoveryCompleted epoch={} queues={} at {}", 
                    epoch, reconstructed_queues, timestamp);
            }
            RuntimeEvent::InvariantBroken { invariant_name, details, timestamp } => {
                tracing::error!("🚨 [EVENT] InvariantBroken name={} details={} at {}", 
                    invariant_name, details, timestamp);
            }
            RuntimeEvent::StageAdvanced { from_stage, to_stage, active_count, timestamp } => {
                tracing::info!("🚪 [EVENT] StageAdvanced from={} to={} active={} at {}", 
                    from_stage, to_stage, active_count, timestamp);
            }
            RuntimeEvent::QueueScrub { removed_count, reason, timestamp } => {
                tracing::info!("🧹 [EVENT] QueueScrub removed={} reason={} at {}", 
                    removed_count, reason, timestamp);
            }
            RuntimeEvent::ActiveFileOrphaned { file, timestamp } => {
                tracing::warn!("⚠️ [EVENT] ActiveFileOrphaned file={} at {}", file, timestamp);
            }
            RuntimeEvent::ActiveFileScrubbed { file, timestamp } => {
                tracing::info!("🧹 [EVENT] ActiveFileScrubbed file={} at {}", file, timestamp);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub lifecycle_state: TaskLifecycleState, // v0.0.31.xx: scheduler semantics
    pub dependencies: Vec<String>,
    pub result: Option<String>,
    pub target_file: Option<String>,
    pub lock_files: Vec<String>,
    pub error_feedback: Option<String>,
    pub senior_comment: Option<String>,
    pub rework_count: u32,
    pub base_hash: Option<String>,
    pub parent_task: Option<String>,
    pub reason: Option<String>,
    pub kind: String,
    pub retries: u32,
    pub assigned_worker: Option<String>,
    pub created_at: DateTime<Local>,
    pub ir_path: Option<String>,
    pub task_kind: Option<LanguageTaskKind>,
    pub signature: Option<String>,
    pub validator_rejections: u32,
    pub senior_rejections: u32,
    pub architecture_rejections: u32,
    pub cargo_rejections: u32,
    pub lsp_rejections: u32,
    pub boss_interventions: u32,
    #[serde(default)]
    pub patch_contract: Option<PatchContract>,
    #[serde(default)]
    pub repair_mode: Option<RepairMode>,
    #[serde(default)]
    pub repair_origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposedTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub component_id: Option<String>,
}

impl Task {
    pub fn from_decomposed(dt: DecomposedTask, project_id: String) -> Self {
        Self {
            id: dt.id,
            project_id,
            title: dt.title,
            description: dt.description,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            result: None,
            target_file: dt.component_id.clone(),
            lock_files: Vec::new(),
            error_feedback: None,
            senior_comment: None,
            rework_count: 0,
            base_hash: None,
            parent_task: None,
            reason: None,
            kind: if let Some(ref f) = dt.component_id {
                if f.ends_with(".c") || f.ends_with(".h") {
                    "c".to_string()
                } else if f.ends_with(".rs") {
                    "rust".to_string()
                } else if f.ends_with(".py") {
                    "python".to_string()
                } else {
                    "rust".to_string()
                }
            } else {
                "rust".to_string()
            },
            retries: 0,
            assigned_worker: None,
            created_at: Local::now(),
            ir_path: None,
            task_kind: dt.component_id.as_ref().map(|f| {
                if f.ends_with(".h") || f.ends_with(".hpp") {
                    LanguageTaskKind::C(CTaskKind::HeaderDecl)
                } else {
                    LanguageTaskKind::C(CTaskKind::SourceImpl)
                }
            }),
            signature: None,
            validator_rejections: 0,
            senior_rejections: 0,
            architecture_rejections: 0,
            cargo_rejections: 0,
            lsp_rejections: 0,
            boss_interventions: 0,
            lifecycle_state: TaskLifecycleState::Queued,
            patch_contract: None,
            repair_mode: None,
            repair_origin: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CTaskKind {
    HeaderDecl,
    SourceImpl,
    Integrator,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RustTaskKind {
    ModuleDecl,
    ModuleImpl,
    Integrator,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LanguageTaskKind {
    C(CTaskKind),
    Rust(RustTaskKind),
}

impl LanguageTaskKind {
    pub fn phase(&self) -> u32 {
        match self {
            LanguageTaskKind::C(CTaskKind::HeaderDecl) => 1,
            LanguageTaskKind::C(CTaskKind::SourceImpl) => 2,
            LanguageTaskKind::C(CTaskKind::Integrator) => 3,
            LanguageTaskKind::Rust(RustTaskKind::ModuleDecl) => 1,
            LanguageTaskKind::Rust(RustTaskKind::ModuleImpl) => 2,
            LanguageTaskKind::Rust(RustTaskKind::Integrator) => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepairMode {
    FullRewrite,
    PatchOnly,
}

impl Default for RepairMode {
    fn default() -> Self {
        RepairMode::FullRewrite
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatchOperation {
    ReplaceFunction { symbol: String },
    ModifyLines { start: usize, end: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchContract {
    pub target_file: String,
    pub symbol: Option<String>,
    pub error_line: Option<usize>,
    pub error_message: String,
    pub hard_constraints: Vec<String>,
    pub forbidden_patterns: Vec<String>,
    pub allowed_changes: Vec<String>,
    #[serde(default)]
    pub allowed_regions: Vec<(usize, usize)>,
}

/// v0.0.32: Persistent failure diagnostic metadata — paired with .failed source files.
/// Enables Failure Corpus, Root Cause Mining, and Rejection Analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedDiagnostic {
    pub stage: f32,
    pub gatekeeper: String,
    pub error_line: Option<usize>,
    pub error_message: String,
    pub reason_classification: String,
}

/// v0.0.32: Gate status within the promotion pipeline.
/// Replaces scattered bool variables (auto_reject_reason, is_approve, boss_approved, ...).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateStatus {
    Passed,
    Failed(String),
    Skipped,
}

impl GateStatus {
    pub fn is_passed(&self) -> bool {
        matches!(self, GateStatus::Passed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, GateStatus::Failed(_))
    }
}

/// v0.0.32: Explicit promotion decision object.
/// Aggregates the verdict of all 5 pipeline gates into a single auditable state.
///
/// Current implicit state (before this struct):
///   auto_reject_reason, rejection_source, is_approve, boss_approved
///
/// After:
///   PromotionDecision { validator, lsp, compilation, senior, boss }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionDecision {
    pub validator: GateStatus,
    pub lsp: GateStatus,
    pub compilation: GateStatus,
    pub senior: GateStatus,
    pub boss: GateStatus,
}

impl PromotionDecision {
    pub fn eligible(&self) -> bool {
        self.validator.is_passed()
            && self.lsp.is_passed()
            && self.compilation.is_passed()
            && self.senior.is_passed()
            && self.boss.is_passed()
    }

    pub fn summary(&self) -> String {
        let gate = |name: &str, s: &GateStatus| -> String {
            match s {
                GateStatus::Passed => format!("✅ {}", name),
                GateStatus::Failed(msg) => format!("❌ {} ({})", name, msg.chars().take(40).collect::<String>()),
                GateStatus::Skipped => format!("⏭ {}", name),
            }
        };
        format!(
            "{} | {} | {} | {} | {}",
            gate("Validator", &self.validator),
            gate("LSP", &self.lsp),
            gate("Compiler", &self.compilation),
            gate("Senior", &self.senior),
            gate("Boss", &self.boss),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Variable,
    Macro,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOwnership {
    pub symbol_name: String,
    pub owner_task_id: Option<String>,
    pub phase: LanguageTaskKind,
    pub file_path: String,
    pub symbol_kind: SymbolKind,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub brace_depth: Option<usize>,
    pub immutable: bool,
    pub last_validated_hash: Option<String>,
    #[serde(default)]
    pub validated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub validator_passed: bool,
    #[serde(default)]
    pub history: Vec<SymbolHistoryEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolHistoryEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub task_id: String,
    pub event_type: String,
    pub previous_hash: Option<String>,
    pub new_hash: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolOwnershipRegistry {
    pub files: std::collections::HashMap<String, std::collections::HashMap<String, SymbolOwnership>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Ready,
    Assigned,
    InProgress,
    Completed,
    Failed,
    AwaitDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub thread_id: String,
    pub author_id: String, // Agent ID or "BOSS"
    pub content: String,
    pub thought: Option<String>, // v0.0.25: Internal reasoning or 'vibe' from the agent
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum EventType {
    // Thread Events
    ThreadCreated,
    ThreadAssigned,
    ThreadStarted,
    ThreadCompleted,
    ThreadStatusChanged,
    
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
    AgentStreamingData,
    AgentHired,
    AgentFired,
    AgentUpdated,
    
    // System Events
    SystemLog,
    Signal,
    QuotaExceeded,
    SystemWarning,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EventLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: String,
    pub project_id: String,
    pub thread_id: Option<String>,
    pub agent_id: Option<String>,
    pub event_type: EventType,
    pub level: EventLevel,
    pub source: String, // e.g., "daemon", "agent_id", "dispatcher"
    pub content: String,
    pub payload: Option<serde_json::Value>,
    pub timestamp: DateTime<Local>,
}

pub mod protocol;
pub mod ir;
pub mod ir_change;
pub mod patch;
pub mod spec;
pub mod transformer;
pub mod validator;
pub mod rules;
pub mod profile;

pub mod events {
    use super::Event;
    use tokio::sync::broadcast;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub struct EventBus {
        tx: broadcast::Sender<Event>,
        counter: AtomicUsize,
    }

    impl EventBus {
        pub fn new(capacity: usize) -> Self {
            let (tx, _) = broadcast::channel(capacity);
            Self { tx, counter: AtomicUsize::new(0) }
        }

        pub fn subscribe(&self) -> broadcast::Receiver<Event> {
            self.tx.subscribe()
        }

        pub fn publish(&self, event: Event) {
            self.counter.fetch_add(1, Ordering::Relaxed);
            let _ = self.tx.send(event);
        }

        pub fn get_count(&self) -> usize {
            self.counter.load(Ordering::Relaxed)
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
}#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: String,
    pub tasks: Vec<Task>,
    pub dependency_closure: std::collections::HashSet<String>,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAssignment {
    pub batch: Batch,
}

impl Default for Thread {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            title: String::new(),
            status: ThreadStatus::Draft,
            author: String::new(),
            milestone_id: None,
            task_kind: None,
            rejection_count: 0,
            validator_rejections: 0,
            senior_rejections: 0,
            architecture_rejections: 0,
            cargo_rejections: 0,
            lsp_rejections: 0,
            boss_interventions: 0,
            error_feedback: None,
            reason: None,
            created_at: chrono::Local::now(),
            updated_at: chrono::Local::now(),
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            title: String::new(),
            description: String::new(),
            status: TaskStatus::Pending,
            lifecycle_state: TaskLifecycleState::Queued,
            dependencies: Vec::new(),
            result: None,
            target_file: None,
            lock_files: Vec::new(),
            error_feedback: None,
            senior_comment: None,
            rework_count: 0,
            base_hash: None,
            parent_task: None,
            reason: None,
            kind: String::new(),
            retries: 0,
            assigned_worker: None,
            created_at: chrono::Local::now(),
            ir_path: None,
            task_kind: None,
            signature: None,
            validator_rejections: 0,
            senior_rejections: 0,
            architecture_rejections: 0,
            cargo_rejections: 0,
            lsp_rejections: 0,
            boss_interventions: 0,
            patch_contract: None,
            repair_mode: None,
            repair_origin: None,
        }
    }
}
