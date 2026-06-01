use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::sync::RwLock as AsyncRwLock;
use tokio::task::JoinHandle;
use axon_core::{
    AgentRole, Event, EventLevel, EventType, Task, TaskLifecycleState,
    TaskStatus, ThreadStatus, Post, PostType, FailedDiagnostic,
    PromotionDecision, GateStatus,
};
use axon_core::events::EventBus;
use axon_agent::AgentRuntime;
use axon_ir::schema::ProjectIR;
use axon_storage::Storage;
use crate::bootstrap::create_model_driver;
use crate::{AxonConfig, PipelineReview, AgentConfig};
use crate::server::AgentPool;
use crate::intelligence::corpus::catastrophe_archive::PredictiveImmuneLayer;
use crate::intelligence::replay::lineage_taxonomy::{TaxonomyMigrationManifest, RootLineage};
use crate::intelligence::corpus::corpus_executor::CorpusExecutor;
// v0.0.31.40: [LSP_GATEKEEPER] LSP semantic firewall between Junior and Senior
use crate::intelligence::lsp::{LspSupervisor, LspVerdict, LspDiagnostic};
use axon_ir::Language;
// v0.0.31.41: [COMPILATION_GATE] Physical compiler harness between LSP and Senior
use crate::execution_validator::{self, ValidationMode, extract_error_locations};

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

// Phase CoT/ToT: Strip <REASONING>...</REASONING> blocks from Senior output
// This isolates reasoning from the decision — only [APPROVE]/[REJECT] reaches the state machine
fn strip_reasoning_block(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut skip = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<REASONING>") {
            skip = true;
            continue;
        }
        if trimmed.starts_with("</REASONING>") {
            skip = false;
            continue;
        }
        if !skip {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
    }

    result
}

/// v0.0.31.37: [HALLUCINATION_GUARD] Senior Header Review Validation
/// When Senior rejects a header file, verify if the proposed code actually contains
/// function bodies ({ ... } patterns). If the code only has declarations, flag it as
/// Senior hallucination. Pure logic — no hardcoded keywords.
pub fn detect_senior_header_hallucination(
    target_file: &str,
    _feedback: &str,
    proposed_code: &str,
) -> bool {
    // Only applies to header files
    if !target_file.ends_with(".h") && !target_file.ends_with(".hpp") {
        return false;
    }

    // Verify: does the proposed code actually contain function bodies?
    // A function body in C/C++ is identified by { ... } at the top level.
    // Declarations end with ; — definitions have { ... }.
    let has_function_body = proposed_code.lines().any(|line| {
        let trimmed = line.trim();
        // Skip preprocessor directives, comments, and empty lines
        if trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") || trimmed.is_empty() {
            return false;
        }
        // A line that contains both a function-like pattern AND an opening brace
        if trimmed.contains('{') {
            let before_brace: String = trimmed.split('{').next().unwrap_or("").to_string();
            // Function definitions typically have ) before {
            if before_brace.contains(')') && !before_brace.trim().starts_with("struct") && !before_brace.trim().starts_with("enum") && !before_brace.trim().starts_with("union") && !before_brace.trim().starts_with("class") {
                return true;
            }
        }
        false
    });

    if !has_function_body {
        tracing::warn!(
            "🛡️ [HALLUCINATION_GUARD] Senior rejected header {}, but proposed code has NO function bodies (declarations only). Auto-approving.",
            target_file
        );
        return true;
    }

    false
}

/// v0.0.31.38: [AUTONOMOUS_SLICER] Header Contamination Pre-Senior Review
/// When a Junior submits a header file (.h/.hpp) containing function bodies ({ ... }),
/// automatically extract the function bodies into a companion .cpp file,
/// leaving only declarations in the header.
/// This prevents the "Junior writes implementation in header → Senior REJECT → deadlock" loop.
/// Returns: Some((cleaned_header, cpp_content, cpp_path)) if contamination detected, None otherwise.
pub fn autonomous_header_slicer(
    target_file: &str,
    proposed_code: &str,
    sandbox_root: &std::path::Path,
) -> Option<(String, String, std::path::PathBuf)> {
    // Only applies to header files
    if !target_file.ends_with(".h") && !target_file.ends_with(".hpp") {
        return None;
    }

    // Parse the code to detect and extract function bodies
    let lines: Vec<&str> = proposed_code.lines().collect();
    let mut header_lines: Vec<String> = Vec::new();
    let mut cpp_body_lines: Vec<String> = Vec::new();
    let mut in_function_body = false;
    let mut brace_depth: i32 = 0;
    let mut current_function_sig: Option<String> = None;
    let mut function_buffer: Vec<String> = Vec::new();
    let mut has_cpp_import = false;

    // Determine the corresponding .cpp file path
    let cpp_path = target_file
        .replace("include/", "src/")
        .replace(".h", ".cpp")
        .replace(".hpp", ".cpp");
    let cpp_full_path = sandbox_root.join(&cpp_path);

    for line in &lines {
        let trimmed = line.trim();

        // Skip preprocessor and empty lines outside function bodies
        if !in_function_body {
            if trimmed.starts_with('#') || trimmed.is_empty() {
                header_lines.push(line.to_string());
                continue;
            }

            // Detect function definition: line has ) and { on same line (or { on next)
            // e.g., "char* safe_read_line(int max_length) {"
            let looks_like_func_def = trimmed.contains(')') && (trimmed.contains('{') || {
                // Check if next non-empty line starts with {
                let idx = lines.iter().position(|l| l == line).unwrap_or(0);
                lines.iter().skip(idx + 1).find(|l| !l.trim().is_empty() && !l.trim().starts_with("//"))
                    .map(|next| next.trim().starts_with('{')).unwrap_or(false)
            });

            if looks_like_func_def {
                // Extract the function signature (everything before {)
                let sig: String = if let Some(pos) = trimmed.find('{') {
                    trimmed[..pos].trim().to_string()
                } else {
                    trimmed.trim().to_string()
                };

                // Add declaration to header: replace { body } with ;
                let decl_line = if sig.ends_with(')') {
                    format!("{};", sig)
                } else {
                    format!("{};", sig.trim_end_matches('{').trim())
                };
                header_lines.push(decl_line);

                // Start capturing function body
                in_function_body = true;
                brace_depth = 0;
                current_function_sig = Some(sig.clone());
                function_buffer.clear();

                // If the opening brace is on this line, capture it
                if trimmed.contains('{') {
                    function_buffer.push(line.to_string());
                    brace_depth += 1;
                    // Check for closing brace on same line
                    for ch in trimmed.chars() {
                        if ch == '{' { brace_depth += 1; }
                        if ch == '}' { brace_depth -= 1; }
                    }
                    // Adjust: we counted { twice above, fix
                    brace_depth = 0;
                    let open_count = trimmed.matches('{').count();
                    let close_count = trimmed.matches('}').count();
                    brace_depth = (open_count as i32) - (close_count as i32);

                    if brace_depth <= 0 {
                        // Single-line function body (rare but possible)
                        in_function_body = false;
                        cpp_body_lines.push(format!("{} {{", sig));
                        // Extract body between braces
                        if let Some(start) = trimmed.find('{') {
                            if let Some(end) = trimmed[start..].rfind('}') {
                                let body = &trimmed[start + 1..start + end];
                                if !body.trim().is_empty() {
                                    cpp_body_lines.push(format!("    {}", body.trim()));
                                }
                            }
                        }
                        cpp_body_lines.push("}".to_string());
                        current_function_sig = None;
                    }
                }
                continue;
            }

            // Regular header line (declaration, comment, etc.)
            header_lines.push(line.to_string());
        } else {
            // Inside function body
            function_buffer.push(line.to_string());
            let open_count = trimmed.matches('{').count();
            let close_count = trimmed.matches('}').count();
            brace_depth += (open_count as i32) - (close_count as i32);

            if brace_depth <= 0 {
                // Function body complete
                in_function_body = false;
                if let Some(ref sig) = current_function_sig {
                    cpp_body_lines.push(format!("{} {{", sig));
                    // Add body lines (skip the opening/closing brace lines we already handled)
                    for buf_line in &function_buffer {
                        let bt = buf_line.trim();
                        if !bt.starts_with('#') && !bt.is_empty() {
                            cpp_body_lines.push(buf_line.to_string());
                        }
                    }
                    cpp_body_lines.push("}".to_string());
                }
                current_function_sig = None;
            }
        }
    }

    // If no function bodies were extracted, no contamination
    if cpp_body_lines.is_empty() {
        return None;
    }

    // Build the cleaned header
    let cleaned_header = header_lines.join("\n");

    // Build the companion .cpp file
    let header_include = std::path::Path::new(target_file)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| target_file.to_string());
    let mut cpp_content = String::new();
    cpp_content.push_str(&format!("#include \"{}\"\n", header_include));
    if !has_cpp_import {
        cpp_content.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n");
    }
    cpp_content.push_str("\n");
    cpp_content.push_str(&cpp_body_lines.join("\n"));
    cpp_content.push('\n');

    tracing::warn!(
        "🔪 [AUTONOMOUS_SLICER] Header contamination detected in {}: extracted {} function body(ies) → {}",
        target_file,
        cpp_body_lines.iter().filter(|l| l.trim() == "{").count(),
        cpp_path
    );

    // Write the .cpp file to sandbox
    if let Some(parent) = cpp_full_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cpp_full_path, &cpp_content);

    Some((cleaned_header, cpp_content, cpp_full_path))
}

fn normalize_senior_output(raw: &str) -> NormalizedOutput {
    // Phase CoT/ToT: Strip <REASONING>...</REASONING> blocks before parsing decision
    let stripped = strip_reasoning_block(raw);
    let trimmed = stripped.trim();

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
    pub task_semaphore: Arc<Semaphore>,
    pub agent_pool: Arc<AsyncRwLock<AgentPool>>,
}

impl ExecutionPipeline {
    pub fn new(
        config: AxonConfig,
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        project_id: String,
        sandbox_root: PathBuf,
        agent_pool: Arc<AsyncRwLock<AgentPool>>,
    ) -> Self {
     let junior_count = config.agents.juniors.len();
     Self {
         config,
         storage,
         event_bus,
         project_id,
         sandbox_root,
         pipeline_handle: None,
         running: Arc::new(AtomicBool::new(false)),
         pending_reviews: Arc::new(Mutex::new(HashMap::new())),
         task_semaphore: Arc::new(Semaphore::new(junior_count)),
         agent_pool,
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

    pub fn with_task_semaphore(mut self, semaphore: Arc<Semaphore>) -> Self {
        self.task_semaphore = semaphore;
        self
    }

    pub fn with_agent_pool(mut self, pool: Arc<AsyncRwLock<AgentPool>>) -> Self {
        self.agent_pool = pool;
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
        let task_semaphore = self.task_semaphore.clone();
        let agent_pool = self.agent_pool.clone();

        let handle = tokio::spawn(async move {
            let pool = agent_pool.read().await;
            
            let senior_config = &pool.seniors[0];
            let senior_driver = create_model_driver(senior_config);

            // [HALLUCINATION_FIX] architecture.md를 ProjectIR로 파싱 (에이전트 생성 전)
            let arch_path_early = sandbox_root.join("architecture.md");
            let mut arch_text_early = std::fs::read_to_string(&arch_path_early).unwrap_or_default();
            let mut project_ir_early = ProjectIR::from_md(&arch_text_early);
            if project_ir_early.is_none() {
                tracing::error!("🚨 [IR_PARSE_FAIL] 에이전트 생성 단계에서 ProjectIR 파싱 실패. Boss 개입을 대기합니다.");
                event_bus.publish(Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id.to_string(),
                    thread_id: None,
                    agent_id: Some("pipeline".to_string()),
                    event_type: EventType::SystemLog,
                    level: EventLevel::Error,
                    source: "pipeline".to_string(),
                    content: "🚨 [IR_PARSE_FAIL] 에이전트 생성 단계에서 ProjectIR 파싱 실패. architecture.md 구조 검토 필요.".to_string(),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });
                
                let approval_file = sandbox_root.join(".axon_approval_pending");
                let pending_json = serde_json::json!({
                    "status": "SUSPENDED_BY_IR_FAIL",
                    "reason": "ProjectIR::from_md returned None during agent creation. Fix architecture.md and approve.",
                    "project_id": project_id,
                    "approved": false
                });
                let _ = std::fs::write(&approval_file, serde_json::to_string_pretty(&pending_json).unwrap());

                match Self::wait_for_boss_approval(&approval_file).await {
                    Ok(true) => {
                        tracing::info!("✅ [IR_PARSE_FAIL] Boss approved. Resuming agent creation.");
                        arch_text_early = std::fs::read_to_string(&arch_path_early).unwrap_or_default();
                        project_ir_early = ProjectIR::from_md(&arch_text_early);
                        if project_ir_early.is_none() {
                            tracing::error!("❌ [IR_PARSE_FAIL] ProjectIR parsing still failing. Aborting.");
                            running.store(false, Ordering::SeqCst);
                            return;
                        }
                    }
                    Ok(false) | Err(_) => {
                        tracing::error!("❌ [IR_PARSE_FAIL] Boss rejected or timed out. Aborting.");
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                }
            } else {
                tracing::info!("✅ [IR_INJECTED] 에이전트에 ProjectIR 주입 준비 완료 — {} 컴포넌트", project_ir_early.as_ref().unwrap().components.len());
            }

            let mut juniors: Vec<AgentRuntime> = Vec::new();
            for (i, jconf) in pool.juniors.iter().enumerate() {
                let driver = create_model_driver(jconf);
                let default_id = format!("junior-agent-{:03}", i + 1);
                let agent_id = jconf.id.as_deref().unwrap_or(&default_id);
                let mut agent = AgentRuntime::new(
                    agent_id.to_string(),
                    AgentRole::Junior,
                    jconf.model.clone(),
                    driver,
                )
                .with_timeout(600)
                .with_project(project_id.clone());
                // [HALLUCINATION_FIX] IR 계약 주입 — constraint_block 활성화
                if let Some(ref ir) = project_ir_early {
                    agent = agent.with_ir(ir.clone());
                }
                agent.set_locale(&config.locale);
                juniors.push(agent);
            }

            let mut senior = AgentRuntime::new(
                senior_config.id.as_deref().unwrap_or("senior-agent-001").to_string(),
                AgentRole::Senior,
                senior_config.model.clone(),
                senior_driver,
            )
            .with_timeout(300)
            .with_project(project_id.clone());
            // [HALLUCINATION_FIX] 시니어에게도 IR 계약 주입
            if let Some(ref ir) = project_ir_early {
                senior = senior.with_ir(ir.clone());
            }
            senior.set_locale(&config.locale);
            
            drop(pool);

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
                task_semaphore,
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
        task_semaphore: Arc<Semaphore>,
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

        // [HALLUCINATION_FIX] architecture.md → ProjectIR 파싱 (constraint_block 활성화)
        let mut project_ir = ProjectIR::from_md(&architecture_guide);
        if project_ir.is_none() {
            tracing::error!("🚨 [IR_PARSE_FAIL] architecture.md에서 ProjectIR 파싱 실패. 파이프라인 진행을 차단하고 Boss 개입을 대기합니다.");
            
            event_bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                thread_id: None,
                agent_id: Some("pipeline".to_string()),
                event_type: EventType::SystemLog,
                level: EventLevel::Error,
                source: "pipeline".to_string(),
                content: "🚨 [IR_PARSE_FAIL] ProjectIR 파싱 실패. architecture.md 구조 검토 필요.".to_string(),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            let approval_file = sandbox_root.join(".axon_approval_pending");
            let pending_json = serde_json::json!({
                "status": "SUSPENDED_BY_IR_FAIL",
                "reason": "ProjectIR::from_md returned None. Fix architecture.md and approve.",
                "project_id": project_id,
                "approved": false
            });
            let _ = std::fs::write(&approval_file, serde_json::to_string_pretty(&pending_json).unwrap());

            match Self::wait_for_boss_approval(&approval_file).await {
                Ok(true) => {
                    tracing::info!("✅ [IR_PARSE_FAIL] Boss approved. Resuming pipeline.");
                    let new_architecture_guide = std::fs::read_to_string(&arch_path).unwrap_or_default();
                    project_ir = ProjectIR::from_md(&new_architecture_guide);
                    if project_ir.is_none() {
                        tracing::error!("❌ [IR_PARSE_FAIL] ProjectIR parsing still failing after Boss approval. Aborting.");
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                }
                Ok(false) | Err(_) => {
                    tracing::error!("❌ [IR_PARSE_FAIL] Boss rejected or timed out. Aborting pipeline.");
                    running.store(false, Ordering::SeqCst);
                    return;
                }
            }
        } else {
            tracing::info!("✅ [IR_PARSED] ProjectIR 파싱 성공 — {} 컴포넌트 계약 활성화", project_ir.as_ref().unwrap().components.len());
        }

        let all_tasks = storage.list_all_tasks().unwrap_or_default();
        let project_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|t| {
                t.project_id == project_id
                    && t.status != TaskStatus::Completed
                    && t.lifecycle_state != TaskLifecycleState::Rejected
                    && t.lifecycle_state != TaskLifecycleState::Superseded
                    && t.lifecycle_state != TaskLifecycleState::Fatal
                    && t.lifecycle_state != TaskLifecycleState::Aborted
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
                sandbox_root, project_id, pending_reviews, config, running.clone(), task_semaphore.clone(),
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
                running.store(false, Ordering::SeqCst);
                return;
            }

            // v0.0.31.21: [SSOT_PHASE_STATE] Persist Phase 1 completion to project_state
            let _ = storage.update_project_state(&project_id, "Phase1_Completed", "completed").await;
            let _ = storage.flush().await;
            tracing::info!("🔒 Phase 1 completion locked in project_state for project '{}'", project_id);
        }

        if !phase2.is_empty() {
            tracing::info!("🏗️ Phase 2: Source implementations ({} tasks)", phase2.len());
            Self::execute_phase(
                storage.clone(), event_bus.clone(), juniors.clone(), senior.clone(), &phase2, &architecture_guide,
                sandbox_root, project_id, pending_reviews, config, running.clone(), task_semaphore.clone(),
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
                running.store(false, Ordering::SeqCst);
                return;
            }

            // v0.0.31.21: [SSOT_PHASE_STATE] Persist Phase 2 completion to project_state
            let _ = storage.update_project_state(&project_id, "Phase2_Completed", "completed").await;
            let _ = storage.flush().await;
            tracing::info!("🔒 Phase 2 completion locked in project_state for project '{}'", project_id);
        }

        if !phase3.is_empty() {
            tracing::info!("🏗️ Phase 3: Integrators ({} tasks)", phase3.len());
            Self::execute_phase(
                storage.clone(), event_bus.clone(), juniors.clone(), senior.clone(), &phase3, &architecture_guide,
                sandbox_root, project_id, pending_reviews, config, running.clone(), task_semaphore.clone(),
            )
            .await;
            if Self::is_paused(&running) { return; }

            // v0.0.31.21: [SSOT_PHASE_STATE] Persist Phase 3 completion to project_state
            let _ = storage.update_project_state(&project_id, "Phase3_Completed", "completed").await;
            let _ = storage.flush().await;
            tracing::info!("🔒 Phase 3 completion locked in project_state for project '{}'", project_id);
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

        running.store(false, Ordering::SeqCst);
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
        task_semaphore: Arc<Semaphore>,
    ) {
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
            let permit = task_semaphore.clone();
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

    // v0.0.32: [FAILED_DIAGNOSTIC] Persist structured failure metadata alongside .failed source
    fn save_failed_diagnostic(
        sandbox_root: &PathBuf,
        target: &str,
        diag: &FailedDiagnostic,
    ) {
        let sandbox_path = Self::sandbox_path(sandbox_root, target);
        let failed_path = Self::failed_path(&sandbox_path);
        let json_path = PathBuf::from(format!("{}.json", failed_path.to_string_lossy()));
        if let Ok(json) = serde_json::to_string_pretty(diag) {
            if let Err(e) = std::fs::write(&json_path, json) {
                tracing::warn!("⚠️ [FAILED_DIAGNOSTIC] Failed to write {}: {}", json_path.display(), e);
            } else {
                tracing::info!("📋 [FAILED_DIAGNOSTIC] Saved {} for {} (stage={}, gatekeeper={})",
                    json_path.display(), target, diag.stage, diag.gatekeeper);
            }
        }
    }

    // v0.0.32: [PATCH_RADIUS] Adapter: FailedDiagnostic.error_line → allowed_regions
    // Regions are 1-based inclusive line numbers (e.g. line 42 → (39, 45))
    fn diagnostic_to_regions(diags: &[FailedDiagnostic]) -> Vec<(usize, usize)> {
        diags.iter()
            .filter_map(|d| d.error_line.map(|line| (line.saturating_sub(3), line + 3)))
            .collect()
    }

    // v0.0.32: [PATCH_RADIUS] Validate that Junior's patch output stays within allowed regions
    fn validate_patch_radius(
        original: &str,
        proposed: &str,
        regions: &[(usize, usize)],
    ) -> Result<(), String> {
        let orig_lines: Vec<&str> = original.lines().collect();
        let new_lines: Vec<&str> = proposed.lines().collect();

        let in_allowed = |line_idx: usize| -> bool {
            regions.iter().any(|&(start, end)| {
                let s = start.saturating_sub(1); // 1-based to 0-based
                line_idx >= s && line_idx < end
            })
        };

        let max_len = orig_lines.len().max(new_lines.len());
        for i in 0..max_len {
            let old = orig_lines.get(i).copied().unwrap_or("");
            let new = new_lines.get(i).copied().unwrap_or("");
            if old != new && !in_allowed(i) {
                return Err(format!(
                    "🚨 [PATCH_RADIUS] Line {} modified outside allowed regions {:?}:\n  old: {}\n  new: {}",
                    i + 1, regions, old, new
                ));
            }
        }
        Ok(())
    }

    // v0.0.32: [PATCH_RADIUS] Load failed diagnostic regions from .failed.json
    fn load_failed_regions(sandbox_root: &PathBuf, target: &str) -> Vec<(usize, usize)> {
        let sandbox_path = Self::sandbox_path(sandbox_root, target);
        let failed_path = Self::failed_path(&sandbox_path);
        let json_path = PathBuf::from(format!("{}.json", failed_path.to_string_lossy()));
        match std::fs::read_to_string(&json_path) {
            Ok(json) => match serde_json::from_str::<Vec<FailedDiagnostic>>(&json) {
                Ok(diags) => Self::diagnostic_to_regions(&diags),
                Err(e) => {
                    tracing::debug!("[PATCH_RADIUS] Failed to parse {}: {}", json_path.display(), e);
                    Vec::new()
                }
            },
            Err(e) => {
                tracing::debug!("[PATCH_RADIUS] Failed to load {}: {}", json_path.display(), e);
                Vec::new()
            }
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

    // v0.0.32: [PROMOTION_DECISION] Explicit promotion execution — extracted from inline pipeline logic
    // Handles atomic rename, copy fallback, sandbox cleanup, and CMakeLists.txt restoration.
    fn unlock_promotion(
        task: &Task,
        proposal: &Post,
        sandbox_root: &PathBuf,
        project_id: &str,
        locale: &str,
    ) -> Result<(), String> {
        if let Some(ref target) = task.target_file {
            let sandbox_path = Self::sandbox_path(sandbox_root, target);
            let real_path = sandbox_root.join(target);

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
                        Err(e) => return Err(format!("Failed to write {}: {}", real_path.display(), e)),
                    }
                }
                Ok(())
            };

            if promote_result.is_ok() {
                let _ = std::fs::remove_file(&sandbox_path);

                // [ATOMIC_RESTORE] CMake Pruning 자동 복구 로직
                if target.ends_with(".c") || target.ends_with(".cpp") {
                    tracing::info!("🔧 [ATOMIC_RESTORE] C/C++ Source created. Restoring CMakeLists.txt targets...");
                    if let Ok(arch_text) = std::fs::read_to_string(sandbox_root.join("architecture.md")) {
                        if let Some(ir) = ProjectIR::from_md(&arch_text) {
                            let mut graph = crate::dep_graph::DepGraph::new();
                            graph.build_from_ir(&serde_json::to_value(&ir).unwrap_or_default());
                            let cmake_spec = crate::dep_graph::parse_cmake_spec(&arch_text);
                            let cmake_content = graph.generate_cmake(project_id, locale, sandbox_root, cmake_spec.as_ref());
                            if let Err(e) = std::fs::write(sandbox_root.join("CMakeLists.txt"), cmake_content) {
                                tracing::error!("❌ [ATOMIC_RESTORE] Failed to write CMakeLists.txt: {}", e);
                            } else {
                                tracing::info!("✅ [ATOMIC_RESTORE] CMakeLists.txt targets successfully restored.");
                            }
                        }
                    }
                }
            }

            promote_result.map_err(|_| format!("Promotion failed for {}", target))
        } else {
            Ok(())
        }
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
        let mut rejection_source: &str = "senior";

        loop {
            rejection_source = "senior";
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
                    let raw = std::fs::read_to_string(&sandbox_path)
                        .or_else(|_| std::fs::read_to_string(&failed_path))
                        .unwrap_or_default();

                    // v0.0.31.38: [FIX_EMPTY_FILE_LOOP] Filter out meaningless .failed content
                    // that would cause Junior to hallucinate "empty file" regeneration.
                    // Patterns: "// Empty file", "// This is a new file.", whitespace-only
                    let trimmed = raw.trim();
                    let meaningless = trimmed.is_empty()
                        || trimmed == "// Empty file"
                        || trimmed == "// This is a new file."
                        || trimmed == "// empty file"
                        || trimmed.starts_with("// This is a new file");
                    if meaningless {
                        tracing::info!("🔪 [FIX_EMPTY_FILE_LOOP] Filtered out meaningless .failed content for '{}' — treating as new file", target);
                        String::new()
                    } else {
                        raw
                    }
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
            let mut modified_proposal = proposal.clone();
            if let Some(ref code) = proposal.full_code {
                if let Some(ref target) = task.target_file {
                    let sandbox_path = Self::sandbox_path(sandbox_root, target);
                    if let Some(parent) = sandbox_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&sandbox_path, code);
                    Self::log_sandbox_transition(SandboxState::Clean, SandboxState::Proposed, target, "Junior proposal written");

                    // v0.0.31.38: [AUTONOMOUS_SLICER] Pre-Senior Review Header Contamination Fix
                    // If Junior submitted function bodies in a header file, auto-extract them to .cpp
                    if let Some((cleaned_header, _cpp_content, _cpp_path)) =
                        autonomous_header_slicer(target, code, sandbox_root)
                    {
                        // Overwrite sandbox with cleaned header (declarations only)
                        let _ = std::fs::write(&sandbox_path, &cleaned_header);
                        // Update proposal's full_code so Senior reviews the cleaned version
                        modified_proposal.full_code = Some(cleaned_header);
                        tracing::info!("🔪 [AUTONOMOUS_SLICER] Header cleaned before Senior review: {}", target);
                    }
                }
            }

            // Immediately save Junior's proposal and broadcast the event
            if let Err(e) = storage.save_post_sync(modified_proposal.clone()) {
                tracing::error!("❌ Failed to save Junior proposal: {}", e);
            }
            event_bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                thread_id: Some(task.id.clone()),
                agent_id: Some(modified_proposal.author_id.clone()),
                event_type: EventType::PostAdded,
                level: EventLevel::Info,
                source: "pipeline".to_string(),
                content: format!("Junior agent proposed changes for {}", resolved_target.as_deref().unwrap_or("unknown")),
                payload: Some(serde_json::to_value(&modified_proposal).unwrap_or_default()),
                timestamp: chrono::Local::now(),
            });

            // 1부 & 2부: 전역 헌법 및 프로젝트 계약 가드레일 집행 (0바이트 사멸, forbidden_symbols, allowed_includes 물리 필터링)
            let mut project_ir = std::fs::read_to_string(sandbox_root.join("architecture.md")).ok()
                .and_then(|text| ProjectIR::from_md(&text));

            if project_ir.is_none() {
                tracing::error!("🚨 [IR_PARSE_FAIL] execute_one_task: ProjectIR 파싱 실패. 진행 차단 및 Boss 개입 대기.");
                
                event_bus.publish(Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id.to_string(),
                    thread_id: Some(task.id.clone()),
                    agent_id: Some("pipeline".to_string()),
                    event_type: EventType::SystemLog,
                    level: EventLevel::Error,
                    source: "pipeline".to_string(),
                    content: "🚨 [IR_PARSE_FAIL] execute_one_task ProjectIR 파싱 실패. architecture.md 구조 검토 필요.".to_string(),
                    payload: None,
                    timestamp: chrono::Local::now(),
                });

                let approval_file = sandbox_root.join(format!(".axon_approval_pending_{}", task.id));
                let pending_json = serde_json::json!({
                    "status": "SUSPENDED_BY_IR_FAIL",
                    "reason": "ProjectIR::from_md returned None during task execution. Fix architecture.md and approve.",
                    "project_id": project_id,
                    "task_id": task.id,
                    "approved": false
                });
                let _ = std::fs::write(&approval_file, serde_json::to_string_pretty(&pending_json).unwrap());

                match Self::wait_for_boss_approval(&approval_file).await {
                    Ok(true) => {
                        tracing::info!("✅ [IR_PARSE_FAIL] Boss approved. Resuming task execution.");
                        project_ir = std::fs::read_to_string(sandbox_root.join("architecture.md")).ok()
                            .and_then(|text| ProjectIR::from_md(&text));
                        if project_ir.is_none() {
                            tracing::error!("❌ [IR_PARSE_FAIL] ProjectIR parsing still failing. Aborting task execution.");
                            break;
                        }
                    },
                    Ok(false) | Err(_) => {
                        tracing::error!("❌ [IR_PARSE_FAIL] Boss rejected or timed out. Aborting task execution.");
                        break;
                    }
                }
            }

            let mut auto_reject_reason = None;
            if let Some(ref code) = modified_proposal.full_code {
                // v0.0.31.37: [FIX_HEADER_FALSE_REJECT] Extract file extension for language-aware comment detection
                let file_ext = resolved_target.as_deref()
                    .and_then(|p| std::path::Path::new(p).extension())
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if Self::is_empty_or_comments_only(code, file_ext) {
                    rejection_source = "validator";
                    auto_reject_reason = Some("🚨 [GLOBAL_HARNESS] Auto-rejected: Proposed code is empty or contains only comments. Actual implementation is required.".to_string());
                    if let Some(ref target) = resolved_target {
                        Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                            stage: 19.3,
                            gatekeeper: "validator (global_harness)".to_string(),
                            error_line: None,
                            error_message: "Proposed code is empty or contains only comments.".to_string(),
                            reason_classification: "empty_or_comments".to_string(),
                        });
                    }
                } else {
                    let custom_forbidden: Vec<String> = if let Some(ref ir) = project_ir {
                        if let Some(comp) = ir.get_component(resolved_target.as_deref().unwrap_or("")) {
                            comp.forbidden_symbols.iter().cloned().collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    if let Some(sym) = Self::check_forbidden_symbols(code, &custom_forbidden, sandbox_root) {
                        rejection_source = "validator";
                        auto_reject_reason = Some(format!("🚨 [GLOBAL_HARNESS] Auto-rejected: Code contains forbidden symbol/function '{}'.", sym));
                        if let Some(ref target) = resolved_target {
                            Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                                stage: 19.3,
                                gatekeeper: "validator (global_harness)".to_string(),
                                error_line: None,
                                error_message: format!("Forbidden symbol/function '{}' detected.", sym),
                                reason_classification: "forbidden_symbol".to_string(),
                            });
                        }
                    } else {
                        let mut allowed_list: Vec<String> = if let Some(ref ir) = project_ir {
                            if let Some(comp) = ir.get_component(resolved_target.as_deref().unwrap_or("")) {
                                comp.allowed_includes.iter().cloned().collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        // 프로젝트 내부의 정당한 모든 C/C++ 헤더 파일은 자동으로 allowed_list에 합산하여
                        // 불필요한 보스 승인 대기(Manual Interlock Friction) 발생을 원천 차단함.
                        if let Some(ref ir) = project_ir {
                            for comp_path in ir.components.keys() {
                                if comp_path.ends_with(".h") || comp_path.ends_with(".hpp") {
                                    if let Some(filename) = std::path::Path::new(comp_path).file_name().and_then(|f| f.to_str()) {
                                        allowed_list.push(filename.to_string());
                                    }
                                }
                            }
                        }

                        if let Some(header) = Self::check_allowed_includes(code, &allowed_list) {
                            rejection_source = "validator";
                            auto_reject_reason = Some(format!("🚨 [GLOBAL_HARNESS] Auto-rejected: Non-conforming header '{}' detected. Only allowed includes are permitted.", header));
                            if let Some(ref target) = resolved_target {
                                Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                                    stage: 19.3,
                                    gatekeeper: "validator (global_harness)".to_string(),
                                    error_line: None,
                                    error_message: format!("Non-conforming header '{}' detected.", header),
                                    reason_classification: "non_conforming_include".to_string(),
                                });
                            }
                        }
                    }
                }
            } else {
                rejection_source = "validator";
                auto_reject_reason = Some("🚨 [GLOBAL_HARNESS] Auto-rejected: Proposed code is empty.".to_string());
                if let Some(ref target) = resolved_target {
                    Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                        stage: 19.3,
                        gatekeeper: "validator (global_harness)".to_string(),
                        error_line: None,
                        error_message: "Proposed code is empty (no full_code in proposal).".to_string(),
                        reason_classification: "empty_code".to_string(),
                    });
                }
            }

            // [LINEAGE_GUARD] Junior 제출 코드에서 알려진 위험 계통 패턴을 감지하면 WARN
            // REJECT는 하지 않지만 Senior에게 컨텍스트를 제공하고 로그에 기록
            if auto_reject_reason.is_none() {
                if let Some(ref code) = modified_proposal.full_code {
                    let manifest = TaxonomyMigrationManifest::build_v2();
                    // 코드에서 알려진 증상 키워드 탐지
                    let detected_symptoms: Vec<String> = manifest.migration_map.keys()
                        .filter(|sym| code.contains(sym.as_str()))
                        .cloned()
                        .collect();

                    if !detected_symptoms.is_empty() {
                        let primary_root = manifest.map_legacy_symptom(&detected_symptoms[0]);
                        if PredictiveImmuneLayer::check_against_archive(&primary_root) {
                            tracing::warn!(
                                "⚠️ [LINEAGE_GUARD] Task '{}': Detected known-dangerous causal pattern '{:?}' (symptoms: {:?}). Senior will review with elevated scrutiny.",
                                task.title, primary_root, detected_symptoms
                            );
                        }
                    }
                }
            }

            // v0.0.31.40: [LSP_GATEKEEPER] Static analysis firewall between Junior and Senior
            // LSP intercepts Junior's code BEFORE Senior sees it — blocks syntax errors at the gate
            let mut lsp_diagnostics: Option<Vec<LspDiagnostic>> = None;
            if auto_reject_reason.is_none() {
                if let Some(ref target) = task.target_file {
                    let ext = std::path::Path::new(target.as_str())
                        .extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    let language = match ext.as_str() {
                        "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" => Some(Language::C),
                        "rs" => Some(Language::Rust),
                        "py" => Some(Language::Python),
                        "lua" => Some(Language::Lua),
                        _ => None,
                    };

                    if let Some(lang) = language {
                        let target_path = sandbox_root.join(target);
                        if target_path.exists() {
                            tracing::info!("🛡️ [LSP_GATEKEEPER] Running semantic gate on '{}' (lang: {:?})", target, lang);
                            match LspSupervisor::semantic_gate(lang, sandbox_root, &[target_path]).await {
                                LspVerdict::Reject(errors) => {
                                    // Senior 완전 차단 — 주니어에게 즉시 반려
                                    rejection_source = "lsp";
                                    let error_summary: Vec<String> = errors.iter()
                                        .map(|e| format!("[Line {}] {}", e.line, e.message))
                                        .collect();
                                    auto_reject_reason = Some(format!(
                                        "🚨 [LSP_GATEKEEPER] Auto-rejected: Static analysis found {} error(s):\n{}",
                                        errors.len(),
                                        error_summary.join("\n")
                                    ));
                                    tracing::warn!("❌ [LSP_GATEKEEPER] Senior blocked — {} syntax errors detected", errors.len());
                                    if let Some(first) = errors.first() {
                                        Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                                            stage: 19.4,
                                            gatekeeper: format!("lsp ({})", first.source),
                                            error_line: Some(first.line),
                                            error_message: first.message.clone(),
                                            reason_classification: "syntax_error".to_string(),
                                        });
                                    }
                                }
                                LspVerdict::Warning(warns) => {
                                    // 경고는 Senior에게 진단서로 전달
                                    let warn_count = warns.len();
                                    lsp_diagnostics = Some(warns);
                                    tracing::info!("️ [LSP_GATEKEEPER] {} warnings — passing diagnostic report to Senior", warn_count);
                                }
                                LspVerdict::Clean => {
                                    tracing::info!("✅ [LSP_GATEKEEPER] Code passed static analysis — forwarding to Senior");
                                }
                            }
                        }
                    }
                }
            }

            // v0.0.31.41: [COMPILATION_GATE] Physical compiler harness between LSP and Senior
            // LSP가 문법 규격을 검속한 뒤, 실제 컴파일러로 물리적 무결성 검증
            if auto_reject_reason.is_none() {
                if let Some(ref target) = task.target_file {
                    let ext = std::path::Path::new(target.as_str())
                        .extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    let is_compilable = matches!(ext.as_str(), "c" | "cpp" | "cc" | "cxx" | "rs" | "py" | "lua");
                    if is_compilable {
                        let sandbox_path = Self::sandbox_path(sandbox_root, target);
                        let real_path = sandbox_root.join(target);

                        if sandbox_path.exists() {
                            if let Some(parent) = real_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            let _ = std::fs::remove_file(&real_path);
                            let link_ok = std::fs::hard_link(&sandbox_path, &real_path).is_ok();

                            if link_ok {
                                let root_str = sandbox_root.to_string_lossy().to_string();
                                let target_str = target.clone();
                                let ir_clone = project_ir.clone();
                                let compile_result = tokio::task::spawn_blocking(move || {
                                    execution_validator::validate(&root_str, &target_str, ValidationMode::Incremental, ir_clone.as_ref())
                                }).await;

                                let _ = std::fs::remove_file(&real_path);

                                if let Ok(Err(e)) = compile_result {
                                    rejection_source = "compiler";
                                    let diag = e.to_string();
                                    auto_reject_reason = Some(format!(" [COMPILATION_GATE] Build failed:\n{}", diag));
                                    tracing::warn!("❌ [COMPILATION_GATE] 물리 컴파일 실패 — {}", diag);
                                    let parsed = extract_error_locations(&diag);
                                    let error_line = parsed.first().map(|l| l.line);
                                    let error_message = parsed.first().map(|l| l.message.clone()).unwrap_or_else(|| diag.lines().take(3).collect::<Vec<_>>().join("\n"));
                                    let gatekeeper = match std::path::Path::new(target.as_str()).extension().and_then(|e| e.to_str()) {
                                        Some("rs") => "compiler (cargo)",
                                        Some("py") => "compiler (python3)",
                                        Some("lua") => "compiler (luac)",
                                        _ => "compiler (gcc)",
                                    };
                                    Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                                        stage: 19.5,
                                        gatekeeper: gatekeeper.to_string(),
                                        error_line,
                                        error_message,
                                        reason_classification: "compilation_error".to_string(),
                                    });
                                } else if let Ok(Ok(_)) = compile_result {
                                    tracing::info!("✅ [COMPILATION_GATE] 물리 컴파일 통과 — Senior로 전달");
                                }
                            }
                        }
                    }
                }
            }

            // v0.0.32: [PATCH_RADIUS] Failure 지역 내 변경 감시 (WARN-only)
            if auto_reject_reason.is_none() && retries > 0 {
                if let Some(ref target) = task.target_file {
                    let regions = Self::load_failed_regions(sandbox_root, target);
                    if !regions.is_empty() {
                        if let Some(ref proposed) = modified_proposal.full_code {
                            let sandbox_path = Self::sandbox_path(sandbox_root, target);
                            let failed_path = Self::failed_path(&sandbox_path);
                            let orig_code = std::fs::read_to_string(&failed_path)
                                .or_else(|_| std::fs::read_to_string(&sandbox_path))
                                .unwrap_or_default();
                            if let Err(warn) = Self::validate_patch_radius(&orig_code, proposed, &regions) {
                                tracing::warn!("{}", warn);
                                // WARN-only — auto_reject_reason 미설정, 데이터 수집 후 격상 검토
                            }
                        }
                    }
                }
            }

            let review = if let Some(ref reason) = auto_reject_reason {
                tracing::warn!("{}", reason);
                Post {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: task.id.clone(),
                    author_id: "senior-agent-001".to_string(),
                    content: format!("[REJECT]\n{}", reason),
                    thought: Some("Auto-reject due to global harness verification.".to_string()),
                    full_code: None,
                    post_type: PostType::Review,
                    metrics: None,
                    created_at: chrono::Local::now(),
                }
            } else {
                // Format LSP diagnostics into a report string for Senior
                let lsp_report_str = lsp_diagnostics.as_ref().map(|diags| {
                    diags.iter()
                        .map(|d| format!("[Line {}] {} ({:?})", d.line, d.message, d.severity))
                        .collect::<Vec<_>>()
                        .join("\n")
                });

                // v0.0.32: [PINPOINT_DEFECT_ANALYSIS] Load failed source + diagnostics for Senior
                let failed_code = if retries > 0 {
                    if let Some(ref target) = task.target_file {
                        let sandbox_path = Self::sandbox_path(sandbox_root, target);
                        let failed_path = Self::failed_path(&sandbox_path);
                        std::fs::read_to_string(&failed_path).ok()
                    } else { None }
                } else { None };

                let failed_diagnostics: Option<Vec<FailedDiagnostic>> = if retries > 0 {
                    if let Some(ref target) = task.target_file {
                        let sandbox_path = Self::sandbox_path(sandbox_root, target);
                        let failed_path = Self::failed_path(&sandbox_path);
                        let json_path = PathBuf::from(format!("{}.json", failed_path.to_string_lossy()));
                        std::fs::read_to_string(&json_path).ok()
                            .and_then(|s| serde_json::from_str(&s).ok())
                    } else { None }
                } else { None };

                match senior
                    .review_proposal(
                        task, &modified_proposal, None,
                        Some(event_bus.clone()),
                        lsp_report_str.as_deref(),
                        failed_diagnostics.as_deref(),
                        failed_code.as_deref(),
                        error_feedback.as_deref(),
                    )
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
                }
            };

            // Immediately save Senior's review and broadcast the event
            if let Err(e) = storage.save_post_sync(review.clone()) {
                tracing::error!("❌ Failed to save Senior review: {}", e);
            }
            event_bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                thread_id: Some(task.id.clone()),
                agent_id: Some(review.author_id.clone()),
                event_type: EventType::PostAdded,
                level: EventLevel::Info,
                source: "pipeline".to_string(),
                content: format!("Senior agent completed review for {}", resolved_target.as_deref().unwrap_or("unknown")),
                payload: Some(serde_json::to_value(&review).unwrap_or_default()),
                timestamp: chrono::Local::now(),
            });

            // Phase 7-B: Output Normalization Layer — single convergence point for all Senior output formats
            let normalized = normalize_output(&review.content, true);
            let mut is_approve = normalized.is_approve();
            let mut feedback = normalized.feedback.unwrap_or_default();

            // 2부 규격 1 & 2: Boss Board 최종 인장 인터락 & [HALLUCINATION_GUARD] 독소 조항 거세 및 역전
            // detect_senior_header_hallucination 오버라이드 로직을 폐기하고, 
            // 시니어의 결과(APPROVE/REJECT)와 무관하게 무조건 Boss Board 인간 매니저 최종 승인 인터락을 거치도록 설계함.
            let approval_file = sandbox_root.join(format!(".axon_approval_pending_{}", task.id));
            let approval_info = serde_json::json!({
                "status": "PENDING_BOSS_APPROVAL",
                "message": format!(
                    "Task '{}' requires Boss review. Senior review result: {}. Target file: {:?}",
                    task.title,
                    if is_approve { "APPROVED" } else { "REJECTED" },
                    task.target_file
                ),
                "task_id": task.id,
                "target_file": task.target_file.clone().unwrap_or_default(),
                "senior_approved": is_approve,
                "senior_feedback": feedback,
                "approved": false
            });

            if let Err(e) = std::fs::write(&approval_file, serde_json::to_string_pretty(&approval_info).unwrap_or_default()) {
                tracing::error!("❌ Failed to write task approval file: {}", e);
            }

            // 스레드 상태를 BossApproval로 갱신하여 UI 및 대시보드 경보 작동
            Self::update_thread_status(storage, &task.id, ThreadStatus::BossApproval).await;
            Self::log_active_workers(storage, &format!("task '{}' → Boss Board (Manual Interlock)", task.title));

            event_bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                thread_id: Some(task.id.clone()),
                agent_id: Some("pipeline".to_string()),
                event_type: EventType::ApprovalRequested,
                level: EventLevel::Warning,
                source: "pipeline".to_string(),
                content: format!(
                    "Task '{}' requires Boss approval. Senior status: {}",
                    task.title,
                    if is_approve { "APPROVED" } else { "REJECTED" }
                ),
                payload: None,
                timestamp: chrono::Local::now(),
            });

            tracing::info!("⏳ Pipeline suspended. Waiting for Boss approval for task '{}' via file: {:?}", task.title, approval_file);

            // 보스 승인 대기
            let boss_approved = match Self::wait_for_boss_approval(&approval_file).await {
                Ok(approved) => approved,
                Err(e) => {
                    tracing::error!("❌ Boss approval wait failed: {}", e);
                    false
                }
            };

            // 보스 승인 결과에 따라 최종 승인 상태 분기 매핑
            if boss_approved {
                tracing::info!("✅ Boss approved task '{}'. Proceeding to promotion.", task.title);

                // [CORPUS_GATE] Boss 승인 후 promotion 직전 shadow campaign으로 원본 파일 불변성 검증
                if let Some(ref target) = task.target_file {
                    let sandbox_path = Self::sandbox_path(sandbox_root, target);
                    if sandbox_path.exists() {
                        let shadow_sandbox = std::path::PathBuf::from("/tmp/axon_corpus_gate");
                        let noop_mutation = |s: &str| s.to_string(); // identity: 변환 없음
                        match CorpusExecutor::execute_shadow_campaign(
                            &sandbox_path,
                            &shadow_sandbox,
                            &noop_mutation,
                        ) {
                            Ok(result) if result.pre_hash == result.post_hash => {
                                tracing::info!(
                                    "✅ [CORPUS_GATE] Integrity verified for '{}': pre_hash==post_hash",
                                    target
                                );
                            }
                            Ok(_) => {
                                tracing::error!(
                                    "🚨 [CORPUS_GATE] INTEGRITY VIOLATION for '{}': file mutated during gate!",
                                    target
                                );
                            }
                            Err(e) => {
                                tracing::warn!("⚠️ [CORPUS_GATE] Shadow check skipped for '{}': {}", target, e);
                            }
                        }
                    }
                }

                is_approve = true;
            } else {
                tracing::warn!("❌ Boss rejected task '{}'. Proceeding to retry/rework.", task.title);
                is_approve = false;
                feedback = format!("[BOSS_REJECTED] Boss overridden rejection for task: {}", task.title);
            }

            // v0.0.32: [PROMOTION_DECISION] Aggregate all gate results into explicit decision object
            let validator_status = if auto_reject_reason.is_some() && rejection_source == "validator" {
                GateStatus::Failed(auto_reject_reason.as_deref().unwrap_or("").to_string())
            } else {
                GateStatus::Passed
            };
            let lsp_status = if auto_reject_reason.is_some() && rejection_source == "lsp" {
                GateStatus::Failed(auto_reject_reason.as_deref().unwrap_or("").to_string())
            } else if auto_reject_reason.is_none() || rejection_source != "validator" {
                GateStatus::Passed
            } else {
                GateStatus::Skipped
            };
            let compilation_status = if auto_reject_reason.is_some() && rejection_source == "compiler" {
                GateStatus::Failed(auto_reject_reason.as_deref().unwrap_or("").to_string())
            } else if auto_reject_reason.is_none() || (rejection_source != "validator" && rejection_source != "lsp") {
                GateStatus::Passed
            } else {
                GateStatus::Skipped
            };
            let senior_status = if auto_reject_reason.is_some() {
                GateStatus::Skipped
            } else if is_approve || boss_approved {
                GateStatus::Passed
            } else {
                GateStatus::Failed(feedback.clone())
            };
            let boss_status = if boss_approved {
                GateStatus::Passed
            } else if is_approve {
                GateStatus::Passed
            } else {
                GateStatus::Failed("Boss rejected".to_string())
            };

            let decision = PromotionDecision {
                validator: validator_status,
                lsp: lsp_status,
                compilation: compilation_status,
                senior: senior_status,
                boss: boss_status,
            };

            if decision.eligible() {
                tracing::info!("✅ [PROMOTION] All gates passed — {}", decision.summary());
                if let Err(e) = Self::unlock_promotion(task, &proposal, sandbox_root, project_id, &_config.locale) {
                    tracing::error!("🚨 [PROMOTION] unlock_promotion failed: {}", e);
                }

                let mut updated = task.clone();
                updated.status = TaskStatus::Completed;
                updated.lifecycle_state = TaskLifecycleState::Completed;
                updated.rework_count = retries;
                // Preserve rejection counters from DB (already incremented during rejection path)
                if let Ok(Some(current)) = storage.get_task(&task.id) {
                    updated.validator_rejections = current.validator_rejections;
                    updated.senior_rejections = current.senior_rejections;
                    updated.architecture_rejections = current.architecture_rejections;
                    updated.cargo_rejections = current.cargo_rejections;
                    updated.lsp_rejections = current.lsp_rejections;
                    updated.boss_interventions = current.boss_interventions;
                }
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
                            // v0.0.32: [FAILED_DIAGNOSTIC] Save Senior rejection diagnostic
                            let error_line = regex::Regex::new(r"(?:line|Line)\s*(\d+)")
                                .ok()
                                .and_then(|re| re.captures(&feedback))
                                .and_then(|cap| cap.get(1))
                                .and_then(|m| m.as_str().parse::<usize>().ok());
                            let gatekeeper = match rejection_source {
                                "validator" => "validator (global_harness)",
                                "lsp" => "lsp (semantic_gate)",
                                "compiler" => "compiler (build_gate)",
                                _ => "senior",
                            };
                            Self::save_failed_diagnostic(sandbox_root, target, &FailedDiagnostic {
                                stage: 20.0,
                                gatekeeper: gatekeeper.to_string(),
                                error_line,
                                error_message: feedback.chars().take(500).collect(),
                                reason_classification: "spec_violation".to_string(),
                            });
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
                match rejection_source {
                    "validator" => updated.validator_rejections += 1,
                    "lsp" => updated.lsp_rejections += 1,
                    "compiler" => updated.cargo_rejections += 1,
                    "senior" => updated.senior_rejections += 1,
                     _ => updated.senior_rejections += 1,
                }
                let _ = storage.save_task(updated.clone()).await;

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

                // Sync rejection counters to Thread after each rejection
                Self::update_thread_status(storage, &task.id, ThreadStatus::Working).await;

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    async fn update_thread_status(storage: &Storage, thread_id: &str, status: ThreadStatus) {
        if let Ok(Some(mut thread)) = storage.get_thread(thread_id) {
            thread.status = status;
            thread.updated_at = chrono::Local::now();

            if let Ok(Some(task)) = storage.get_task(thread_id) {
                thread.validator_rejections = task.validator_rejections;
                thread.senior_rejections = task.senior_rejections;
                thread.architecture_rejections = task.architecture_rejections;
                thread.cargo_rejections = task.cargo_rejections;
                thread.lsp_rejections = task.lsp_rejections;
                thread.boss_interventions = task.boss_interventions;
                thread.rejection_count = task.rework_count;
                thread.error_feedback = task.error_feedback.clone();
            }

            let _ = storage.save_thread(thread).await;
            let _ = storage.flush().await;
        }
    }

    async fn wait_for_boss_approval(approval_file: &std::path::Path) -> Result<bool, String> {
        let mut poll_interval = tokio::time::interval(std::time::Duration::from_millis(500));
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(3600); // 1 hour timeout

        loop {
            poll_interval.tick().await;
            if start.elapsed() > timeout {
                return Err("Boss approval timed out after 1 hour.".to_string());
            }

            // Check file directly
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

    // v0.0.31.37: [FIX_HEADER_FALSE_REJECT] Language-aware comment detection
    // C/C++ '#' is a preprocessor directive, NOT a comment. Python '#' IS a comment.
    // v0.0.31.38: [FIX_PTR_DEREF_FALSE_REJECT] '*' and '-' are only stripped inside block comments.
    // Pointer deref (*ptr = 10;) and negative literals (-1) at line start must be preserved.
    fn is_empty_or_comments_only(code: &str, file_ext: &str) -> bool {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return true;
        }

        let has_alphanumeric = trimmed.chars().any(|c| c.is_alphanumeric());
        if !has_alphanumeric {
            return true;
        }

        // Language-specific line comment detection
        let is_python = file_ext == "py";
        let is_lua = file_ext == "lua";

        let mut cleaned = String::new();
        let mut in_block_comment = false;

        for line in trimmed.lines() {
            let l = line.trim();

            // Block comment start detection (line-level)
            if !in_block_comment && l.starts_with("/*") {
                in_block_comment = true;
                // Single-line block comment: /* ... */
                if l.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            // Block comment end detection
            if in_block_comment {
                if l.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            // Language-specific line comment detection (only outside block comments)
            let is_line_comment = if is_python {
                l.starts_with("#")
            } else if is_lua {
                l.starts_with("--") && !l.starts_with("---")
            } else {
                // C, C++, Rust, and others: only // is a line comment
                // '#' is a preprocessor directive (C/C++) or attribute (Rust) — NOT a comment
                l.starts_with("//")
            };

            if is_line_comment {
                continue;
            }

            // '*' and '-' at line start are NO LONGER stripped here.
            // They are valid code: *ptr = 10;  -1  -x + y
            // Block comment inner lines are already handled above.

            cleaned.push_str(l);
        }

        // Final pass: remove inline block comments /* ... */ from cleaned text
        let mut final_cleaned = String::new();
        let mut chars = cleaned.chars().peekable();
        let mut in_block = false;
        while let Some(c) = chars.next() {
            if in_block {
                if c == '*' {
                    if let Some(&'/') = chars.peek() {
                        chars.next();
                        in_block = false;
                    }
                }
            } else {
                if c == '/' {
                    if let Some(&'*') = chars.peek() {
                        chars.next();
                        in_block = true;
                        continue;
                    }
                }
                final_cleaned.push(c);
            }
        }

        !final_cleaned.chars().any(|c| c.is_alphanumeric())
    }

    fn check_forbidden_symbols(code: &str, custom_forbidden: &[String], sandbox_root: &std::path::Path) -> Option<String> {
        let mut global_forbidden = Vec::new();
        
        let constraints_path = sandbox_root.join("immutable_constraints.json");
        if let Ok(content) = std::fs::read_to_string(&constraints_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(forbidden) = json.get("forbidden_patterns").and_then(|v| v.as_array()) {
                    for pat in forbidden {
                        if let Some(s) = pat.as_str() {
                            global_forbidden.push(s.to_string());
                        }
                    }
                }
            }
        }

        for sym in &global_forbidden {
            if code.contains(sym) {
                return Some(sym.to_string());
            }
        }
        for sym in custom_forbidden {
            if code.contains(sym) {
                return Some(sym.to_string());
            }
        }
        None
    }

    fn check_allowed_includes(code: &str, allowed: &[String]) -> Option<String> {
        if allowed.is_empty() {
            return None;
        }
        let std_libs = [
            "stdio.h", "stdlib.h", "string.h", "string", "vector", "iostream",
            "memory", "algorithm", "map", "set", "thread", "mutex", "filesystem",
            "chrono", "dlfcn.h", "pthread.h", "math.h", "cstddef", "cstring", "cassert"
        ];
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let header = parts[1].trim_matches(|c| c == '<' || c == '>' || c == '"');
                    let is_allowed = allowed.iter().any(|a| {
                        let clean_a = a.trim_matches(|c| c == '<' || c == '>' || c == '"');
                        clean_a == header || header.contains(clean_a)
                    }) || std_libs.contains(&header);
                    if !is_allowed {
                        return Some(parts[1].to_string());
                    }
                }
            }
        }
        None
    }
}
