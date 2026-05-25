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

pub mod persona;
pub mod lounge;
pub mod composer;

use axon_core::{Agent, Post, PostType, AgentRole};
pub use axon_core::{Task, DecomposedTask};
use axon_model::{ModelDriver, ModelResponse};
use axon_ir::{ComponentTier, default_true};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque, BTreeMap, BTreeSet};
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct HotRule {
    pub key: String,
    pub target: String,
    pub action: String,
    pub weight: f32,
    pub last_seen: DateTime<Local>,
}

pub struct HotRuleCache {
    pub rules: HashMap<(String, String), HotRule>,
    pub order: VecDeque<(String, String)>,
    pub max_size: usize,
}

impl HotRuleCache {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            order: VecDeque::new(),
            max_size: 32,
        }
    }

    pub fn upsert(&mut self, key: String, target: String, action: String) {
        let k = (key.clone(), target.clone());
        if let Some(rule) = self.rules.get_mut(&k) {
            rule.weight += 1.0;
            rule.last_seen = Local::now();
        } else {
            if self.rules.len() >= self.max_size {
                if let Some(old_k) = self.order.pop_front() {
                    self.rules.remove(&old_k);
                }
            }
            self.rules.insert(k.clone(), HotRule {
                key, target, action, weight: 1.0, last_seen: Local::now()
            });
            self.order.push_back(k);
        }
    }

    pub fn decay(&mut self) {
        let keys_to_remove: Vec<_> = self.rules.iter()
            .filter_map(|(k, v)| {
                let new_weight = v.weight * 0.9;
                if new_weight < 0.5 { Some(k.clone()) } else { None }
            })
            .collect();
        
        for k in keys_to_remove {
            self.rules.remove(&k);
            self.order.retain(|x| x != &k);
        }
        
        for rule in self.rules.values_mut() {
            rule.weight *= 0.9;
        }
    }

    pub fn get_hints(&self, count: usize) -> String {
        let mut sorted_rules: Vec<_> = self.rules.values().collect();
        sorted_rules.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

        // Conflict resolution: one hint per target
        let mut hints = Vec::new();
        let mut seen_targets = std::collections::HashSet::new();

        for rule in sorted_rules {
            if !seen_targets.contains(&rule.target) {
                hints.push(format!("- MUST: {}", rule.action));
                seen_targets.insert(rule.target.clone());
                if hints.len() >= count { break; }
            }
        }

        if hints.is_empty() { "".to_string() }
        else { format!("\n### 💡 RECENT HINTS (HOT CACHE) ###\n{}\n", hints.join("\n")) }
    }
}

#[derive(Clone)]
pub struct AgentRuntime {
    pub agent: Agent,
    pub model: Arc<dyn ModelDriver + Send + Sync>,
    pub locale: String, // v0.0.15: System language preference
    pub timeout: std::time::Duration,
    pub throttler: Option<Arc<tokio::sync::Semaphore>>,
    pub hot_cache: Arc<Mutex<HotRuleCache>>,
    pub project_id: String,
    pub ir: Option<axon_core::ir::ProjectIR>, // v0.0.28: Architectural Contract
}

impl AgentRuntime {
    pub fn new(id: String, role: AgentRole, model_name: String, model_driver: Arc<dyn ModelDriver + Send + Sync>) -> Self {
        let persona = persona::AffixSystem::generate_random(role.clone());
        let agent = Agent {
            id: id.clone(),
            name: persona.name.clone(),
            role,
            persona,
            model: model_name,
            status: "Idle".to_string(),
            parent_id: None,
            dtr: 0.5,
        };
        Self { 
            agent, 
            model: model_driver,
            locale: "en_US".to_string(), // Default
            timeout: std::time::Duration::from_secs(300),
            throttler: None,
            hot_cache: Arc::new(Mutex::new(HotRuleCache::new())),
            project_id: "default-project".to_string(),
            ir: None,
        }
    }

    pub fn with_project(mut self, project_id: String) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_ir(mut self, ir: axon_core::ir::ProjectIR) -> Self {
        self.ir = Some(ir);
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = std::time::Duration::from_secs(seconds);
        self
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    fn extract_enveloped_json(&self, raw: &str) -> Option<String> {
        let raw = raw.trim();
        if raw.is_empty() { return None; }

        // Phase 1: Hard Envelope Check
        if let Some(start_idx) = raw.find("<JSON_START>") {
            if let Some(end_idx) = raw.rfind("<JSON_END>") {
                let body = &raw[start_idx + "<JSON_START>".len()..end_idx];
                return Some(body.trim().to_string());
            }
        }

        // Phase 2: Fallback to curly braces
        let start = raw.find('{')?;
        let end = raw.rfind('}')?;
        
        if end > start {
            Some(raw[start..=end].to_string())
        } else {
            None
        }
    }

    fn extract_thought(&self, raw: &str) -> Option<String> {
        let patterns = ["THOUGHT:", "Reasoning:", "### Thought", "### Reasoning"];
        for pattern in patterns {
            if let Some(idx) = raw.find(pattern) {
                let start = idx + pattern.len();
                let rest = &raw[start..];
                // Take until the next section or end of text
                let end = rest.find("\n###").or_else(|| rest.find("<JSON_START>")).unwrap_or(rest.len());
                let thought = rest[..end].trim().to_string();
                if !thought.is_empty() { return Some(thought); }
            }
        }
        None
    }

    fn extract_json(&self, raw: &str) -> Option<String> {
        self.extract_enveloped_json(raw)
    }

    async fn generate_with_retry(&self, prompt: String, event_bus: Option<&Arc<axon_core::events::EventBus>>, thread_id: Option<String>, context_size: usize) -> anyhow::Result<ModelResponse> {
        if let Some(bus) = event_bus {
            bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: self.project_id.clone(),
                thread_id: thread_id.clone(),
                agent_id: Some(self.agent.id.clone()),
                event_type: axon_core::EventType::AgentAction,
                    level: axon_core::EventLevel::Info,
                source: self.agent.id.clone(),
                content: format!("Agent {} is thinking/generating response...", self.agent.name),
                payload: None,
                timestamp: chrono::Local::now(),
            });
        }
        let mut retries = 5;
        loop {
            // PHASE_07: Throttling control
            let _permit = if let Some(t) = &self.throttler {
                Some(t.acquire().await?)
            } else {
                None
            };

            let gen_future = self.model.generate_with_context(prompt.clone(), context_size);
            match tokio::time::timeout(self.timeout, gen_future).await {
                Ok(Ok(mut resp)) => {
                    // v0.0.28: Extract internal reasoning (thoughts) from raw text
                    if let Some(thought) = self.extract_thought(&resp.text) {
                        resp.thought = Some(thought);
                    }
                    if let Some(bus) = event_bus {
                        bus.publish(axon_core::Event {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: self.project_id.clone(),
                            thread_id: thread_id.clone(),
                            agent_id: Some(self.agent.id.clone()),
                            event_type: axon_core::EventType::AgentResponse,
                    level: axon_core::EventLevel::Info,
                            source: self.agent.id.clone(),
                            content: format!("Agent {} completed generation.", self.agent.name),
                            payload: None,
                            timestamp: chrono::Local::now(),
                        });
                    }
                    return Ok(resp)
                },
                Ok(Err(e)) => {
                    let err_str = e.to_string();
                    if err_str.starts_with("QUOTA_WAIT:") {
                        if retries > 0 {
                            let mut wait_secs: f64 = err_str.strip_prefix("QUOTA_WAIT:").unwrap_or("60.0").parse().unwrap_or(60.0);
                            
                            // v0.0.20: Add random jitter (1-5s) to avoid thundering herd problem
                            let jitter = 1.0 + (rand::random::<f64>() * 4.0);
                            wait_secs += jitter;

                            tracing::warn!("Agent {} waiting for {:.1}s (including {:.1}s jitter) due to quota limit...", self.agent.id, wait_secs, jitter);
                            
                            if let Some(bus) = event_bus {
                                bus.publish(axon_core::Event {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    project_id: "default-project".to_string(),
                                    thread_id: thread_id.clone(),
                                    agent_id: Some(self.agent.id.clone()),
                                    event_type: axon_core::EventType::SystemWarning,
                    level: axon_core::EventLevel::Info,
                                    source: self.agent.id.clone(),
                                    content: format!("⚠️ API Quota Limit. Agent entering Standby for {:.0} seconds...", wait_secs),
                                    payload: None,
                                    timestamp: chrono::Local::now(),
                                });
                            }
                            
                            tokio::time::sleep(tokio::time::Duration::from_secs_f64(wait_secs)).await;
                            retries -= 1;
                            continue;
                        }
                    }
                    return Err(anyhow::anyhow!("LLM Error: {}", err_str));
                }
                Err(_) => {
                    tracing::error!("🕒 LLM generate attempt timed out after {}s", self.timeout.as_secs());
                    return Err(anyhow::anyhow!("TIMEOUT | LLM response exceeded {}s", self.timeout.as_secs()));
                }
            }
        }
    }

    pub async fn process_task(&self, task: &Task, architecture_guide: &str, error_feedback: Option<String>, event_bus: Option<Arc<axon_core::events::EventBus>>, existing_code: &str) -> anyhow::Result<Post> {
        let (lang_name, lang_instruction) = match self.locale.as_str() {
            "ko_KR" => ("한국어 (Korean)", "생각(Thought), 노가리(Lounge), 주석, 로그 등 모든 텍스트 응답은 반드시 한국어(Korean)로 작성하십시오. 한국어가 최우선 순위이며, 영어(English)는 절대 금지입니다."),
            "ja_JP" => ("日本語 (Japanese)", "すべてのコメントと出力文字列は 반드시 日本語で作成してください。中国語は絶対に使用しないでください。"),
            _ => ("English", "All comments and output strings must be written in English. Do not use any other languages."),
        };

        let log_msg = match self.locale.as_str() {
            "ko_KR" => format!("요원 {} (주니어)가 태스크 {}를 처리 중입니다...", self.agent.id, task.id),
            "ja_JP" => format!("エージェント {} (ジュニア) がタスク {} を処理しています...", self.agent.id, task.id),
            _ => format!("Agent {} (Junior) processing task {}...", self.agent.id, task.id),
        };
        tracing::info!("{}", log_msg);
        
        let feedback_block = if let Some(err) = &task.error_feedback {
            format!("\n--- [CRITICAL] PREVIOUS ATTEMPT FAILED ---\nERROR: {}\nFIX THE CODE BASED ON THIS ERROR.\n", err)
        } else if let Some(err) = &error_feedback {
            format!("\n--- [CRITICAL] PREVIOUS ATTEMPT FAILED ---\nERROR: {}\nFIX THE CODE BASED ON THIS ERROR.\n", err)
        } else {
            "".to_string()
        };

        // v0.0.23: Use explicit target_file if available, fallback to title parsing
        let target_file_owned = if let Some(target) = &task.target_file {
            target.clone()
        } else {
            let raw_target = task.title.split_whitespace().last().unwrap_or("unknown");
            raw_target
                .trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')' || c == '`' || c == '*')
                .split(']')
                .next()
                .unwrap_or(raw_target)
                .split('(')
                .next()
                .unwrap_or(raw_target)
                .to_string()
        };
        let target_file = &target_file_owned;
        let is_rework = task.rework_count > 0 || error_feedback.is_some();
        let is_patch_mode = task.repair_mode == Some(axon_core::RepairMode::PatchOnly);
        let _target_original_code = if is_rework {
            let sandbox_path = std::path::Path::new(&task.project_id)
                .join(".axon/sandbox")
                .join(target_file);
            match std::fs::read_to_string(&sandbox_path) {
                Ok(code) => {
                    tracing::info!("🎯 Found sandbox target code for patch generation: {:?}", sandbox_path);
                    code
                }
                Err(_) => {
                    let fpath = std::path::Path::new(&task.project_id).join(target_file);
                    std::fs::read_to_string(fpath).unwrap_or_default()
                }
            }
        } else {
            String::new()
        };
        let effective_rework = !existing_code.is_empty(); // Phase 7-A: Conditional rework — enable feedback incorporation when existing code exists (retry scenario)
        let patch_id = uuid::Uuid::new_v4().to_string(); // Phase 8: Transaction Envelope — unique patch ID

        // v0.0.23: IR Chunking (Fixed) - Extract only the block that mentions the target_file
        let filtered_guide = {
            let lines: Vec<&str> = architecture_guide.lines().collect();
            let mut result = String::new();
            let mut target_section_start = None;

            // 1. Find the section index that contains the target_file
            for (i, line) in lines.iter().enumerate() {
                let line_upper = line.to_uppercase();
                let target_upper = target_file.to_uppercase();
                
                // v0.0.26: More flexible matching for "- **File**: name" or "FILE: name"
                if line_upper.contains(&target_upper) && (line_upper.contains("FILE") || line_upper.contains("**FILE**")) {
                    // We found the file line. Now look backwards for the nearest header (Component/Section)
                    for j in (0..=i).rev() {
                        let trimmed = lines[j].trim();
                        if trimmed.starts_with("##") || trimmed.starts_with("###") {
                            target_section_start = Some(j);
                            break;
                        }
                    }
                    break;
                }
            }

            // 2. If found, capture until the next header
            if let Some(start) = target_section_start {
                for i in start..lines.len() {
                    if i > start && lines[i].starts_with("##") {
                        break;
                    }
                    result.push_str(lines[i]);
                    result.push('\n');
                }
            }

            if result.is_empty() { 
                tracing::warn!("⚠️ [CHUNK_FAIL] Could not find IR section for {}. Using full guide as safety fallback.", target_file);
                architecture_guide.to_string()
            } else { 
                result 
            }
        };

        let guide_limit = 3000;
        let short_guide = if filtered_guide.len() > guide_limit {
            format!("{}... [TRUNCATED]", &filtered_guide[..guide_limit])
        } else {
            filtered_guide
        };

        // v0.0.28: Executable Architecture Contract (Strict Rules)
        let mut constraint_block = String::new();
        if let Some(ref ir) = self.ir {
            if let Some(comp) = ir.get_component(target_file) {
                constraint_block.push_str("### 🔒 EXECUTABLE CONTRACT CONSTRAINTS ###\n");
                
                if !comp.allowed_includes.is_empty() {
                    constraint_block.push_str("- **ALLOWED INCLUDES**: Only these are permitted: ");
                    constraint_block.push_str(&comp.allowed_includes.iter().cloned().collect::<Vec<_>>().join(", "));
                    constraint_block.push('\n');
                }
                
                if !comp.forbidden_includes.is_empty() {
                    constraint_block.push_str("- **FORBIDDEN INCLUDES**: DO NOT include these: ");
                    constraint_block.push_str(&comp.forbidden_includes.iter().cloned().collect::<Vec<_>>().join(", "));
                    constraint_block.push('\n');
                }
                
                if !comp.forbidden_symbols.is_empty() {
                    constraint_block.push_str("- **FORBIDDEN LOGIC**: This module MUST NOT contain: ");
                    constraint_block.push_str(&comp.forbidden_symbols.iter().cloned().collect::<Vec<_>>().join(", "));
                    constraint_block.push('\n');
                }

                if let Some(owner) = comp.metadata.get("ownership") {
                    constraint_block.push_str(&format!("- **OWNERSHIP**: This module owns the logic for: {}\n", owner));
                }

                if !comp.functions.is_empty() {
                    constraint_block.push_str("- **REQUIRED FUNCTIONS**: You MUST implement these exact signatures:\n");
                    for func in comp.functions.values() {
                        constraint_block.push_str(&format!("  - {}\n", func.signature));
                    }
                }
            }
        }

        // v0.0.28: Topological Integrator Context
        if let Some(kind) = task.task_kind {
            let is_integrator = match kind {
                axon_core::LanguageTaskKind::C(axon_core::CTaskKind::Integrator) => true,
                axon_core::LanguageTaskKind::Rust(axon_core::RustTaskKind::Integrator) => true,
                _ => false,
            };
            if is_integrator {
                if let Some(ref ir) = self.ir {
                    constraint_block.push_str("\n### 🔗 GLOBAL SYMBOL REGISTRY (INTEGRATION TARGETS) ###\n");
                    constraint_block.push_str("You MUST call functions from these modules to ensure project integration:\n");
                    for (path, comp) in &ir.components {
                        if comp.is_entrypoint { continue; }
                        // v0.0.29.25: Pruning Awareness
                        // In an ideal world, we'd check if the file exists here, but the agent is stateless.
                        // We rely on the Daemon to have already pruned the IR or tasks.
                        // However, as a safeguard, we mark optional components as potentially missing.
                        let tier_tag = if !comp.is_blocking { " [OPTIONAL]" } else { "" };
                        
                        for func in comp.functions.values() {
                            constraint_block.push_str(&format!("- {}: {}{}\n", path, func.signature, tier_tag));
                        }
                    }
                    constraint_block.push_str("\n**PRUNING RULE**: If an [OPTIONAL] module is missing from the project, do NOT attempt to call its functions. Comment out the integration code for it.\n");
                    constraint_block.push_str("**DO NOT** generate a simple placeholder. You are the final glue of the project.\n");
                }
            }
        }


        let mut c_rule_block = String::new();
        if target_file.ends_with(".c") {
            c_rule_block.push_str("\n### ⚠️ [CRITICAL_C_CONSTITUTION - MANDATORY] ###\n");
            c_rule_block.push_str("1. **MANDATORY INCLUDES**: You MUST include `<stdio.h>`, `<stdlib.h>`, and `<string.h>` at the top. ALSO include your corresponding .h file (e.g., `#include \"database.h\"`) FIRST before other logic.\n");
            c_rule_block.push_str("2. **ABI INTEGRITY**: You MUST use EXACT function names and signatures from the architecture. NO variations allowed (e.g., `init_db` vs `init_database` is a FAILURE).\n");
            c_rule_block.push_str("3. **SQLITE3 SAFETY**: NEVER use `strncpy` or `memcpy` into `sqlite3_column_text()` results. These are READ-ONLY. Copy to a local buffer instead. `sqlite3_exec` must pass `&zErrMsg` for error reporting.\n");
            c_rule_block.push_str("4. **NO HALLUCINATED LIBRARIES**: ONLY include headers defined in the architecture or standard C headers. Do NOT add `sqlite3.h` to non-database modules.\n");
            c_rule_block.push_str("5. **STRING LITERALS**: Do NOT break string literals with newlines in the source. Use `\\n` within a single line literal.\n");
            c_rule_block.push_str("6. **STRUCT VISIBILITY**: If the IR defines a struct (e.g., `struct user_record`), you MUST define it or include the header that defines it. Do NOT invent new struct names.\n");
            c_rule_block.push_str("7. **SEMANTIC SEALING (v0.0.29)**: If the architecture lacks a strict 'struct' definition or 'ownership' policy for a function you need to implement, DO NOT GUESS. Output 'ERROR: INSUFFICIENT_SEMANTICS - Missing [Struct/Policy Name]' and terminate.\n");
            c_rule_block.push_str("VIOLATION OF THIS CONSTITUTION WILL TRIGGER AN IMMEDIATE REJECT SIGNAL.\n");
        } else if target_file.ends_with(".h") {
            c_rule_block.push_str("\n### ⚠️ [CRITICAL WARNING - MANDATORY C HEADER RULE] ###\n");
            c_rule_block.push_str("1. You are writing a PURE C HEADER FILE (.h).\n");
            c_rule_block.push_str("2. MUST include Header Guards (#ifndef, #define, #endif).\n");
            c_rule_block.push_str("3. DECLARATIONS ONLY. ABSOLUTELY NO FUNCTION BODIES ( { ... } ).\n");
            c_rule_block.push_str("4. ONLY signatures with a semicolon at the end (e.g., int func(int);).\n");
            c_rule_block.push_str("If you include a function body in this header, the Senior will reject it and you will be penalized.\n");
        }

        let output_format = if effective_rework {
            format!(
                "### 🔒 OUTPUT FORMAT (REWORK MODE — TRANSACTION ENVELOPE REQUIRED) ###\n\
                 You are fixing existing code based on Senior feedback.\n\
                 Review the EXISTING CODE and PREVIOUS FEEDBACK sections above.\n\
                 You MUST wrap your output in a Transaction Envelope:\n\n\
                 ===AXON_PATCH_BEGIN===\n\
                 PATCH_ID: {patch_id}\n\
                 TARGET: {target_file}\n\
                 ===AXON_PATCH_BODY===\n\
                 <YOUR COMPLETE CORRECTED SOURCE CODE HERE>\n\
                 ===AXON_PATCH_END===\n\n\
                 YOU MUST ADDRESS EVERY ISSUE MENTIONED IN THE PREVIOUS FEEDBACK.\n\
                 The envelope MUST have all three markers: BEGIN, BODY, and END.\n\
                 If any marker is missing, your patch will be REJECTED as truncated."
            )
        } else {
            format!(
                "### 🔒 OUTPUT FORMAT (SIMPLE — TRANSACTION ENVELOPE REQUIRED) ###\n\
                 You MUST wrap your output in a Transaction Envelope:\n\n\
                 ===AXON_PATCH_BEGIN===\n\
                 PATCH_ID: {patch_id}\n\
                 TARGET: {target_file}\n\
                 ===AXON_PATCH_BODY===\n\
                 <YOUR COMPLETE SOURCE CODE HERE>\n\
                 ===AXON_PATCH_END===\n\n\
                 The envelope MUST have all three markers: BEGIN, BODY, and END.\n\
                 If any marker is missing, your patch will be REJECTED as truncated."
            )
        };

        // v0.0.31.xx: Phase P0 — Patch Mode code truncation defense-in-depth
        let existing_code_patch = if is_patch_mode && existing_code.len() > 4000 {
            format!("{}...\n// [PATCH MODE: truncated to first 4000 chars]\n", &existing_code[..4000])
        } else {
            existing_code.to_string()
        };

        let patch_block = if is_patch_mode {
            let mut block = String::from(
                "### ⚠️ PATCH MODE ACTIVATED ###\n\
                 AUTHORIZED TARGET:\n\
                 File: {target_file}\n"
            );
            if let Some(ref contract) = task.patch_contract {
                if let Some(ref sym) = contract.symbol {
                    block.push_str(&format!("Symbol: {}\n", sym));
                }
                if let Some(line) = contract.error_line {
                    block.push_str(&format!("Error Line: {}\n", line));
                }
                block.push_str(&format!(
                    "\nCOMPILER ERROR:\n{}\n\n",
                    contract.error_message
                ));
                if !contract.hard_constraints.is_empty() {
                    block.push_str("HARD CONSTRAINTS:\n");
                    for c in &contract.hard_constraints {
                        block.push_str(&format!("- {}\n", c));
                    }
                }
                if !contract.forbidden_patterns.is_empty() {
                    block.push_str("\nFORBIDDEN PATTERNS:\n");
                    for p in &contract.forbidden_patterns {
                        block.push_str(&format!("- {}\n", p));
                    }
                }
            }
            block.push_str(
                "\nMANDATORY RULES:\n\
                 1. Modify ONLY the authorized region above\n\
                 2. Preserve ALL unrelated code byte-for-byte\n\
                 3. Do NOT rewrite the entire file\n\
                 4. Do NOT add helper functions or new symbols\n\
                 5. Do NOT rename existing symbols\n\
                 6. Do NOT modify public APIs\n\
                 7. Do NOT change indentation or formatting outside the patch\n\
                 8. Return ONLY the patched section\n\n\
                 IF THE FIX REQUIRES MODIFICATION OUTSIDE THE AUTHORIZED REGION:\n\
                 RETURN EXACTLY:\n\
                 AXON_PATCH_SCOPE_VIOLATION\n\n"
            );
            block
        } else {
            String::new()
        };

        let system_prompt = format!(
            "### 🏛️ SOVEREIGN CONSTITUTION (v0.0.29 GLOBAL MANDATES) ###\n\
             1. **TECH STACK**: MUST use SQLite3 for persistence as defined in spec.md. NO local arrays or files unless specified.\n\
             2. **C STANDARDS**: MUST follow C99. Use 'strncpy', 'strncat', 'snprintf' for security.\n\
             3. **HEADER FREEZE**: Headers (.h) are for DECLARATIONS ONLY. NO function bodies allowed.\n\n\
             ### [ROLE: AI JUNIOR AGENT - {persona_name}] ###\n\
             LANGUAGE: {lang_name}\n\
             {lang_instruction}\n\n\
             {c_rule_block}\
             ### 🏛️ IR CONTRACT ENFORCEMENT (ABSOLUTE SSOT) ###\n\
             1. **EXACT MAPPING**: You MUST use the EXACT function names and signatures defined in the ARCHITECTURE GUIDE below. Renaming is FORBIDDEN.\n\
             2. **CONSISTENCY > QUALITY**: Even if you think a different name or structure is 'better', YOU MUST NOT CHANGE THE IR. You are an EXECUTOR, not an architect.\n\
             3. **DEPENDENCY LOCK**: ONLY #include files listed in the IR or Standard Library. NEVER invent headers (like 'main.h') if they are not in the IR.\n\
             4. **C/C++ INTERFACE RULE**: Implementation files (.c/.cpp) MUST #include their corresponding .h file FIRST.\n\
             5. **C SYNTAX SAFETY**: No multiline string breaks without escapes. Ensure proper null termination.\n\
             6. **NO STUBS**: Implement FULL functional logic. No placeholders or TODOs allowed.\n\n\
             {constraint_block}\n\
             ### 🗺️ ARCHITECTURE GUIDE (YOUR CONTRACT) ###\n\
             {short_guide}\n\n\
             ### 📋 TASK DETAILS ###\n\
             Target File: {target_file}\n\
             Task Title: {t_title}\n\
             Task Description: {t_desc}\n\n\
             ### ⚠️ PREVIOUS FEEDBACK ###\n\
             {feedback_block}\n\n\
              ### 📄 EXISTING CODE ###\n\
              ```\n\
              {existing_code_patch}\n\
              ```\n\n\
              {patch_block}\
              {output_format}",
            persona_name = self.agent.persona.name,
            lang_name = lang_name,
            lang_instruction = lang_instruction,
            short_guide = short_guide,
            target_file = target_file,
            t_title = task.title,
            t_desc = task.description,
            feedback_block = feedback_block,
            existing_code_patch = existing_code_patch,
            constraint_block = constraint_block,
            c_rule_block = c_rule_block,
            patch_block = patch_block,
            output_format = output_format
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await?;
        
        // v0.0.22: CRITICAL RESOURCE BOTTLENECK PROTECTION
        // If Ollama returns empty content due to memory/GPU timeout, DO NOT treat it as success.
        // v0.0.22: Flexible validation for small models
        if resp.text.trim().is_empty() {
            tracing::error!("❌ [RESOURCE ERROR]: Junior produced an empty response.");
            return Err(anyhow::anyhow!("Ollama produced empty response. Check context limits."));
        }

        // PHASE 09: AXON Patch Protocol v2 Pipeline + Phase 8: Transaction Envelope
        let repaired_text = auto_repair_v2(&resp.text);

        // Phase 8: Try Transaction Envelope first
        let envelope = extract_patch_envelope(&repaired_text);
        // Accept envelope if structural markers are present (BEGIN + BODY + END)
        // BYTE_COUNT/CHECKSUM mismatches are tolerated (LLM can't compute them)
        let structural_ok = !envelope.patch_id.is_empty()
            && !envelope.target.is_empty()
            && envelope.is_complete
            && !envelope.body.is_empty();

        let (full_code, thought) = if structural_ok {
            // Valid envelope structure — use body directly
            tracing::info!("✅ [ENVELOPE_VALID] PATCH_ID={} TARGET={} errors={:?}", envelope.patch_id, envelope.target, envelope.integrity_errors);
            (Some(envelope.body.clone()), None)
        } else {
            // Envelope failed — log errors and fall back
            if !envelope.integrity_errors.is_empty() {
                tracing::warn!("⚠️ [ENVELOPE_INVALID] errors={:?} — falling back to legacy extractors", envelope.integrity_errors);
            }

            // Phase 7-C: Preserve sandbox file as .failed — retry needs original_code
            if let Some(ref target) = task.target_file {
                let sandbox_path = std::path::Path::new(&task.project_id)
                    .join(".axon/sandbox")
                    .join(target);
                if sandbox_path.exists() {
                    let ext = sandbox_path.extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let failed_path = if ext.is_empty() {
                        sandbox_path.with_extension("failed")
                    } else {
                        sandbox_path.with_extension(format!("{}.failed", ext))
                    };
                    let _ = std::fs::rename(&sandbox_path, &failed_path);
                    tracing::info!("📦 Sandbox [PROPOSED] → [CONTAMINATED] | {} | parser failed — renamed to .failed for retry preservation", target);
                }
            }

            // Legacy extraction chain: AXON Patch v2 → markdown code block → C/C++ raw → raw text
            let code = extract_axon_patch_v2(&repaired_text)
                .map(|patch| {
                    patch.files.get(0).map(|f| f.code.clone()).unwrap_or_default()
                })
                .or_else(|| extract_code_block(&repaired_text))
                .or_else(|| extract_cpp_c_code(&repaired_text))
                .or_else(|| {
                    tracing::warn!("⚠️ No code extracted. Using raw output as-is.");
                    Some(repaired_text.clone())
                });
            (code, None)
        };

        let final_code = full_code;

        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: final_code.clone().unwrap_or(resp.text), // v0.0.29: Prioritize clean code for materializer
            thought,
            full_code: final_code,
            post_type: PostType::Proposal,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
    }

    pub async fn run_implementation_task(&self, task: &axon_core::Task, event_bus: std::sync::Arc<axon_core::events::EventBus>, _lang_name: &str, _lang_instruction: &str, architecture_guide: &str, existing_code: &str) -> anyhow::Result<axon_core::Post> {
        self.process_task(task, architecture_guide, None, Some(event_bus), existing_code).await
    }

    pub async fn generate_ir(&self, spec: &str, hint: Option<String>, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::ir::ProjectIR> {
        self.generate_ir_with_context(spec, hint, None, 0, event_bus).await
    }

    pub async fn generate_ir_with_context(&self, spec: &str, hint: Option<String>, constraints: Option<&axon_core::spec::ImmutableConstraints>, context_size: usize, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::ir::ProjectIR> {
        // v0.0.22: Token Overflow Protection (Simple Truncate for 1.8B models)
        let model_name = self.agent.model.to_lowercase();
        let is_small = model_name.contains("qwen") || model_name.contains("1.8b") || model_name.contains("2b");
        
        let processed_spec = if is_small && spec.len() > 24000 {
            let ceiling = std::cmp::min(spec.len(), 48000);
            if ceiling >= spec.len() {
                tracing::info!("📐 Spec is {} bytes, ceiling {} — no truncation needed.", spec.len(), ceiling);
                spec.to_string()
            } else {
                tracing::warn!("⚠️ Spec is too large ({} bytes). Truncating for {} to {} bytes...", spec.len(), self.agent.model, ceiling);
                let cutoff = spec.floor_char_boundary(ceiling);
                format!("{}... [TRUNCATED]", &spec[..cutoff])
            }
        } else {
            spec.to_string()
        };

        let (lang_name, lang_instruction) = match self.locale.as_str() {
            "ko_KR" => ("한국어 (Korean)", "생각(Thought), 노가리(Lounge), 주석, 로그 등 모든 텍스트 응답은 반드시 한국어(Korean)로 작성하십시오. 한국어가 최우선 순위이며, 영어(English)는 절대 금지입니다."),
            "ja_JP" => ("日本語 (Japanese)", "すべてのコメントと出力文字列は 반드시 日本語で作成してください。中国語は絶対に使用しないでください。"),
            _ => ("English", "All comments and output strings must be written in English. Do not use any other languages."),
        };

        // v0.0.32: Platform-aware ontology detection - detect Win32/GUI from spec content
        let spec_lower = spec.to_lowercase();
        let mut is_win32 = spec_lower.contains("win32")
            || spec_lower.contains("winnt")
            || spec_lower.contains("windows gui")
            || spec_lower.contains("windows_native")
            || spec_lower.contains("winsdk")
            || spec_lower.contains("subsystem: windows")
            || spec_lower.contains("platform: win32");

        // v0.0.33 Hardening: Override platform classification via explicit immutable constraints
        if let Some(c) = constraints {
            if let Some(ref plat) = c.platform {
                if plat.to_lowercase() == "linux" {
                    is_win32 = false;
                } else if plat.to_lowercase() == "win32" || plat.to_lowercase() == "windows" {
                    is_win32 = true;
                }
            }
        }

        let is_win32_gui = is_win32 && (spec_lower.contains("gui") || spec_lower.contains("window")
            || spec_lower.contains("hwnd") || spec_lower.contains("wndproc")
            || spec_lower.contains("message loop") || spec_lower.contains("wpaaint"));
        let is_rust = spec_lower.contains("language: rust") || spec_lower.contains("language:rust");
        let is_python = spec_lower.contains("language: python") || spec_lower.contains("language:python");

        let feedback_block = if let Some(h) = hint {
            format!("\n### [CRITICAL FEEDBACK FROM PREVIOUS ATTEMPT] ###\n{}\n\n", h)
        } else {
            "".to_string()
        };

        let constraint_block = if let Some(c) = constraints {
            format!("\n### 🔒 IMMUTABLE CONSTRAINTS (MANDATORY) ###\n{}\n\n", serde_json::to_string_pretty(c).unwrap())
        } else {
            "".to_string()
        };

        // v0.0.32: Platform-specific IR prompt branching
        let (platform_block, json_schema_block, post_rules) = if is_win32_gui {
            (
                "### [PLATFORM: Win32 GUI] ###\n\
                 LANGUAGE: C++ (NOT C)\n\
                 SUBSYSTEM: WindowsGui\n\
                 ENTRY_POINT_TYPE: wWinMain\n\
                 - This is a Win32 Native GUI Application.\n\
                 - NEVER use C language or console-style `main(void)`.\n\
                 - You MUST generate C++ source files (.cpp).\n\
                 - Entry point MUST be `int WINAPI wWinMain(HINSTANCE, HINSTANCE, PWSTR, int)` in src/winmain.cpp\n\
                 - CMake MUST use: add_executable(... WIN32) and set(CMAKE_EXE_LINKER_FLAGS \"-mwindows\")\n\
                 - DLL imports: user32, gdi32, kernel32, shell32, comdlg32\n\
                 - FORBIDDEN: main.c, int main(void), src/main.c\n\n".to_string(),
                "### EXPECTED JSON SCHEMA (Win32 GUI - MANDATORY) ###\n\
                 <JSON_START>\n\
                 {\n\
                   \"node_mapping\": { \"SPEC_NODE\": \"file\" },\n\
                   \"language\": \"cpp\",\n\
                   \"platform\": \"win32\",\n\
                   \"subsystem\": \"windowsgui\",\n\
                   \"entrypoint_type\": \"wwinmain\",\n\
                   \"runtime_model\": \"win32gui\",\n\
                   \"components\": [\n\
                     {\n\
                       \"name\": \"src/winmain.cpp\",\n\
                       \"file\": \"src/winmain.cpp\",\n\
                       \"type\": \"win32_message_loop\",\n\
                       \"functions\": [{ \"name\": \"wWinMain\", \"signature\": \"int WINAPI wWinMain(HINSTANCE, HINSTANCE, PWSTR, int)\" }],\n\
                       \"is_entrypoint\": true,\n\
                       \"dll_imports\": [\"user32\", \"gdi32\", \"kernel32\"]\n\
                     },\n\
                     {\n\
                       \"name\": \"src/window.cpp\",\n\
                       \"file\": \"src/window.cpp\",\n\
                       \"type\": \"win32_wndproc\",\n\
                       \"functions\": [{ \"name\": \"WndProc\", \"signature\": \"LRESULT CALLBACK WndProc(HWND, UINT, WPARAM, LPARAM)\" }]\n\
                     },\n\
                     {\n\
                       \"name\": \"src/windowclass.cpp\",\n\
                       \"file\": \"src/windowclass.cpp\",\n\
                       \"type\": \"win32_window_class\",\n\
                       \"functions\": [{ \"name\": \"register_window_class\", \"signature\": \"ATOM register_window_class(HINSTANCE)\" }]\n\
                     }\n\
                   ]\n\
                 }\n\
                 <JSON_END>\n\n".to_string(),
                "### WIN32 GUI CONSTITUTIONAL RULES ###\n\
                 1. Entry point file MUST be `src/winmain.cpp` (NOT `main.c` or `src/main.c`)\n\
                 2. Entry point function MUST be `wWinMain` (NOT `main`)\n\
                 3. Every .cpp file using HWND, MSG, WNDCLASSEX MUST `#include <windows.h>`\n\
                 4. CMakeLists.txt MUST have `add_executable(... WIN32)` and link user32, gdi32, kernel32\n\
                 5. Rendering MUST only happen inside `case WM_PAINT:` block in WndProc\n\
                 6. FORBIDDEN files: src/main.c, user32.c, gdi32.c, kernel32.c, comdlg32.c\n\
                 7. Win32 API declarations like CreateWindow/DispatchMessage must come ONLY from windows.h\n\n"
            )
        } else if is_win32 {
            (
                "### [PLATFORM: Win32 Console] ###\n\
                 LANGUAGE: C++\n\
                 SUBSYSTEM: Console\n\
                 - Win32 API allowed, but GUI subsystem not required.\n\
                 - Use C++ source files.\n\n".to_string(),
                "### EXPECTED JSON SCHEMA (Win32 - MANDATORY) ###\n\
                 <JSON_START>\n\
                 {\n\
                   \"node_mapping\": { \"SPEC_NODE\": \"file\" },\n\
                   \"language\": \"cpp\",\n\
                   \"platform\": \"win32\",\n\
                   \"subsystem\": \"console\",\n\
                   \"entrypoint_type\": \"wmain\",\n\
                   \"runtime_model\": \"console\",\n\
                   \"components\": [\n\
                     {\n\
                       \"name\": \"src/main.cpp\",\n\
                       \"file\": \"src/main.cpp\",\n\
                       \"type\": \"project_module\",\n\
                       \"functions\": [{ \"name\": \"wmain\", \"signature\": \"int wmain(int argc, wchar_t* argv[])\" }],\n\
                       \"is_entrypoint\": true\n\
                     }\n\
                   ]\n\
                 }\n\
                 <JSON_END>\n\n".to_string(),
                "### WIN32 CONSTITUTIONAL RULES ###\n\
                 1. Entry point MUST be `wmain` or `wWinMain` (NOT `main`)\n\
                 2. FORBIDDEN: `int main(void)`, `int main(int argc, char** argv)`\n\
                 3. Win32 API calls must come from windows.h\n\n"
            )
        } else if is_rust {
            (
                "### [PLATFORM: Rust] ###\n\
                 LANGUAGE: Rust\n\
                 SUBSYSTEM: Console\n\
                 - Use .rs files and Cargo.toml. DO NOT use .c, .h, or CMake.\n\
                 - Entry point: `fn main()` in src/main.rs\n\n".to_string(),
                "### EXPECTED JSON SCHEMA (Rust - MANDATORY) ###\n\
                 <JSON_START>\n\
                 {\n\
                   \"language\": \"rust\",\n\
                   \"platform\": \"generic\",\n\
                   \"subsystem\": \"console\",\n\
                   \"entrypoint_type\": \"main\",\n\
                   \"runtime_model\": \"console\",\n\
                   \"components\": [\n\
                     {\n\
                       \"name\": \"src/main.rs\",\n\
                       \"file\": \"src/main.rs\",\n\
                       \"type\": \"project_module\",\n\
                       \"functions\": [{ \"name\": \"main\", \"signature\": \"fn main()\" }],\n\
                       \"is_entrypoint\": true\n\
                     }\n\
                   ]\n\
                 }\n\
                 <JSON_END>\n\n".to_string(),
                "### RUST CONSTITUTIONAL RULES ###\n\
                 1. NEVER write .c, .h, CMakeLists.txt files for Rust projects\n\
                 2. ALWAYS use Cargo.toml and .rs files only\n\
                 3. FORBIDDEN: #include, malloc, printf, stdio.h, sqlite3.h (use crates instead)\n\n"
            )
        } else if is_python {
            (
                "### [PLATFORM: Python] ###\n\
                 LANGUAGE: Python\n\
                 SUBSYSTEM: Console\n\
                 - Use .py files and requirements.txt. DO NOT use .c, .rs, CMake.\n\
                 - Entry point: `main()` function in main.py\n\n".to_string(),
                "### EXPECTED JSON SCHEMA (Python - MANDATORY) ###\n\
                 <JSON_START>\n\
                 {\n\
                   \"language\": \"python\",\n\
                   \"platform\": \"generic\",\n\
                   \"subsystem\": \"console\",\n\
                   \"entrypoint_type\": \"main\",\n\
                   \"runtime_model\": \"console\",\n\
                   \"components\": [\n\
                     {\n\
                       \"name\": \"main.py\",\n\
                       \"file\": \"main.py\",\n\
                       \"type\": \"project_module\",\n\
                       \"functions\": [{ \"name\": \"main\", \"signature\": \"def main()\" }],\n\
                       \"is_entrypoint\": true\n\
                     }\n\
                   ]\n\
                 }\n\
                 <JSON_END>\n\n".to_string(),
                "### PYTHON CONSTITUTIONAL RULES ###\n\
                 1. NEVER write .c, .h, .rs, CMakeLists.txt files for Python projects\n\
                 2. ALWAYS use .py files and requirements.txt only\n\
                 3. FORBIDDEN: #include, extern, Cargo.toml, cmake\n\n"
            )
        } else {
            // Generic C / C++ (default fallback)
            (
                "### [PLATFORM: Generic C] ###\n\
                 LANGUAGE: C\n\
                 SUBSYSTEM: Console\n\
                 - Standard C99 console application.\n\n".to_string(),
                "### EXPECTED JSON SCHEMA (C - MANDATORY) ###\n\
                 <JSON_START>\n\
                 {\n\
                   \"language\": \"c\",\n\
                   \"platform\": \"generic\",\n\
                   \"subsystem\": \"console\",\n\
                   \"entrypoint_type\": \"main\",\n\
                   \"runtime_model\": \"console\",\n\
                   \"components\": [\n\
                     {\n\
                       \"name\": \"src/main.c\",\n\
                       \"file\": \"src/main.c\",\n\
                       \"type\": \"project_module\",\n\
                       \"functions\": [{ \"name\": \"main\", \"signature\": \"int main(void)\" }],\n\
                       \"is_entrypoint\": true\n\
                     },\n\
                     {\n\
                       \"name\": \"include/module.h\",\n\
                       \"file\": \"include/module.h\",\n\
                       \"type\": \"project_module\",\n\
                       \"functions\": [{ \"name\": \"func\", \"signature\": \"int func(int a)\" }]\n\
                     },\n\
                     {\n\
                       \"name\": \"src/module.c\",\n\
                       \"file\": \"src/module.c\",\n\
                       \"type\": \"project_module\",\n\
                       \"functions\": [{ \"name\": \"func\", \"signature\": \"int func(int a)\" }]\n\
                     }\n\
                   ]\n\
                 }\n\
                 <JSON_END>\n\n".to_string(),
                "### C PROJECT RULES ###\n\
                 1. Every module (except main) MUST have a .h header in include/ and a .c source in src/\n\
                 2. Entry point MUST be `int main(void)` or `int main(int argc, char** argv)`\n\n"
            )
        };

        let system_prompt = format!(
            "{}\
             {}\
             {}\
             ### [LANGUAGE: {lang_name}] ###\n\
             - {}\n\n\
             ### ROLE: CTO & CHIEF ARCHITECT (L-DDP Isolation Mode) ###\n\
             Design the system SKELETON. OUTPUT ONLY VALID JSON.\n\n\
             ### OUTPUT CONTRACT (STRICT) ###\n\
             1. **RETURN ONLY VALID JSON**.\n\
             2. **NO EXPLANATIONS**.\n\
             3. **ENVELOPE**: Wrap your JSON between <JSON_START> and <JSON_END> tokens.\n\n\
             ### IMMUTABLE RULES ###\n\
             - You MUST respect the status (Core/Optional) defined in the constraints.\n\
             - DO NOT promote 'Optional' to 'Core'.\n\
             - DO NOT mark 'Optional' components as 'is_blocking: true'.\n\n\
             ### SOURCE SPEC ###\n\
             {}\n\n\
             {}\n\
             {}\n\
             CRITICAL: You MUST analyze the SOURCE SPEC and extract EVERY module, EVERY function, and EVERY header defined there.\n\
             {}\n\
             Generate JSON Specification NOW:",
            feedback_block, constraint_block, platform_block,
            lang_name, lang_instruction,
            processed_spec, json_schema_block, post_rules
        );

        let mut last_err = String::new();
        for attempt in 1..=5 {
            let resp = self.generate_with_retry(system_prompt.clone(), event_bus.as_ref(), None, context_size).await?;
            let raw_text = resp.text.trim();
            
            if raw_text.is_empty() {
                last_err = "LLM returned an empty response.".to_string();
                tracing::warn!("⚠️ [IR_GEN_FAIL] Attempt {}: {}", attempt, last_err);
                continue;
            }

            // Phase 1: Direct Extraction & Parse
            let clean_json = match self.extract_enveloped_json(raw_text) {
                Some(j) => auto_repair_json_fuzzy(&j),
                None => {
                    tracing::warn!("⚠️ [IR_GEN_FAIL] Attempt {}: No envelope found. Trying raw extraction...", attempt);
                    match self.extract_json(raw_text) {
                        Some(j) => auto_repair_json_fuzzy(&j),
                        None => {
                            last_err = format!("Failed to find JSON envelope or object.");
                            tracing::warn!("⚠️ [IR_GEN_FAIL] Attempt {}: {}\nRAW: {}", attempt, last_err, raw_text);
                            continue;
                        }
                    }
                }
            };

            match parse_ir_from_llm_json(&clean_json) {
                Ok(ir) => {
                    if ir.components.is_empty() {
                        last_err = "Empty components in IR".to_string();
                        tracing::warn!("⚠️ [IR_GEN_FAIL] Attempt {}: {}", attempt, last_err);
                        continue;
                    }
                    // v0.0.28: Preserve thought in IR metadata
                    let mut ir_with_thought = ir;
                    ir_with_thought.thought = resp.thought.clone();
                    return Ok(ir_with_thought);
                },
                Err(e) => {
                    // Phase 2: Repair Pass
                    tracing::warn!("🛠️ [REPAIR_PASS] IR parsing failed ({}). Attempting self-repair...", e);
                    match self.repair_ir_pass(&clean_json, e.to_string(), event_bus.clone()).await {
                        Ok(fixed_ir) => {
                            tracing::info!("✅ [REPAIR_SUCCESS] IR recovered via repair pass.");
                            return Ok(fixed_ir);
                        },
                        Err(repair_err) => {
                            last_err = format!("JSON_PARSE_FAIL: {} | REPAIR_FAIL: {}", e, repair_err);
                            tracing::error!("❌ [IR_GEN_FAIL] Attempt {}: {}\nRAW: {}", attempt, last_err, raw_text);
                        }
                    }
                }
            }
        }
        Err(anyhow::anyhow!("IR_GEN_STABILIZATION_FAILED: {}", last_err))
    }

    async fn repair_ir_pass(&self, broken_json: &str, error_msg: String, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::ir::ProjectIR> {
        let prompt = format!(
            "[ROLE]\nYou are a JSON Repair Expert.\n\n\
             [TASK]\nFix the following invalid JSON IR according to the schema.\n\n\
             [ERROR FROM PARSER]\n{}\n\n\
             [BROKEN JSON]\n{}\n\n\
             [CONSTRAINTS]\n\
             - Output ONLY valid JSON between <JSON_START> and <JSON_END>.\n\
             - Do not add explanations.\n\
             - Preserve the original architecture intent.\n\n\
             Fixed JSON:",
            error_msg, broken_json
        );

        let resp = self.generate_with_retry(prompt, event_bus.as_ref(), None, 0).await?;
        let fixed_str = self.extract_enveloped_json(&resp.text).ok_or_else(|| anyhow::anyhow!("Repair failed to produce enveloped JSON"))?;
        
        let ir = parse_ir_from_llm_json(&fixed_str)
            .map_err(|e| anyhow::anyhow!("Repair Result Parse Fail: {} | Raw: {}", e, fixed_str))?;
        Ok(ir)
    }

    pub async fn repair_ir(&self, ir: &axon_core::ir::ProjectIR, errors: &[String], event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::ir::ProjectIR> {
        let (lang_name, lang_instruction) = match self.locale.as_str() {
            "ko_KR" => ("한국어 (Korean)", "생각(Thought), 노가리(Lounge), 주석, 로그 등 모든 텍스트 응답은 반드시 한국어(Korean)로 작성하십시오. 한국어가 최우선 순위이며, 영어(English)는 절대 금지입니다."),
            "ja_JP" => ("日本語 (Japanese)", "すべてのコメントと出力文字列は 반드시 日本語で作成してください。中国語は絶対に使用しないでください。"),
            _ => ("English", "All comments and output strings must be written in English. Do not use any other languages."),
        };

        let system_prompt = format!(
            "### [LANGUAGE_ENFORCEMENT: {lang_name}] ###\n\
             - {lang_instruction}\n\n\
             ### TASK: REPAIR JSON IR ###\n\
             STRICT RULE: RETURN ONLY THE FIXED JSON OBJECT. NO EXPLANATIONS.\n\n\
             Rules:\n\
             - Fix ONLY fields in error list\n\
             - DO NOT modify valid fields\n\
             - If a node or component is missing, ADD it to the components list\n\n\
             Input IR:\n\
             {}\n\n\
             Errors Found:\n\
             {}\n\n\
             FINAL REMINDER: RETURN ONLY VALID JSON.",
            serde_json::to_string_pretty(ir).unwrap(),
            errors.join("\n")
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), None, 0).await?;
        let raw_text = resp.text.trim();

        if raw_text.is_empty() {
            return Err(anyhow::anyhow!("LLM returned an empty response during IR repair."));
        }

        let clean_json = extract_json(raw_text)
            .ok_or_else(|| anyhow::anyhow!("Failed to find JSON object in LLM response during repair: {}", raw_text))?;

        let ir = parse_ir_from_llm_json(&clean_json)
            .map_err(|e| anyhow::anyhow!("JSON Parse Error during repair: {} | Raw: {}", e, clean_json))?;
        Ok(ir)
    }

    pub async fn generate_architecture_from_ir(&self, ir: &axon_core::ir::ProjectIR, _event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<String> {
        tracing::info!("🛠️ Generating deterministic architecture from IR...");
        
        let mut md = String::new();
        md.push_str("# Project Architecture (Deterministic IR-based)\n\n");
        md.push_str("## Overview\nThis architecture is automatically generated from the converged IR.\n\n");
        
        md.push_str("## Components\n");
        let mut components_json = serde_json::json!({ 
            "node_mapping": ir.node_mapping,
            "components": [] 
        });
        
        // Sort components alphabetically for determinism
        let mut comp_names: Vec<_> = ir.components.keys().collect();
        comp_names.sort();

        for name in comp_names {
            let comp = &ir.components[name];
            md.push_str(&format!("### Component: {}\n", comp.name));
            md.push_str(&format!("- **File**: {}\n", comp.file_path));
            md.push_str("- **Functions**:\n");
            
            // Sort functions alphabetically
            let mut func_names: Vec<_> = comp.functions.keys().collect();
            func_names.sort();

            let mut json_functions = Vec::new();
            for f_name in func_names {
                let func = &comp.functions[f_name];
                md.push_str(&format!("  - {}\n", func.signature));
                
                json_functions.push(serde_json::json!({
                    "name": func.name,
                    "signature": func.signature
                }));
            }
            md.push_str("\n");
            
            // Build the mandatory marker data
            components_json["components"].as_array_mut().unwrap().push(serde_json::json!({
                "name": comp.name,
                "file": comp.file_path,
                "functions": json_functions,
                "type": if comp.name.contains("main") { "entry" } else { "module" },
                "tier": comp.tier,
                "is_blocking": comp.is_blocking
            }));
        }
        
        md.push_str("\n### AXON:SPEC:COMPONENTS\n");
        md.push_str("<!-- AXON:SPEC:COMPONENTS\n");
        md.push_str(&serde_json::to_string_pretty(&components_json).unwrap());
        md.push_str("\n-->\n");
        
        Ok(md)
    }

    pub async fn process_spec_analysis(&self, spec_content: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::spec::ImmutableConstraints> {
        let (lang_name, _lang_instruction) = match self.locale.as_str() {
            "ko_KR" => ("한국어 (Korean)", "응답은 한국어로 작성하십시오."),
            "ja_JP" => ("日本語 (Japanese)", "日本語で作成してください。"),
            _ => ("English", "Write in English."),
        };

        let log_msg = match self.locale.as_str() {
            "ko_KR" => format!("요원 {} (아키텍트) 명세 분석 중: 불변 제약 조건 추출...", self.agent.id),
            _ => format!("Agent {} (Architect) Analyzing Spec: Extracting Immutable Constraints...", self.agent.id),
        };
        tracing::info!("{}", log_msg);

let system_prompt = format!(
            "### SPEC CONSTRAINT EXTRACTOR (v0.0.31.36 [SEMANTIC_CRITICALITY + CONSTITUTION_NORM]) ###\n\
             ROLE: CHIEF COMPLIANCE OFFICER. EXTRACT IMMUTABLE CONSTRAINTS FROM THE SPEC.\n\
             YOUR GOAL: Identify which components are 'Optional', 'Core/Required', or 'Experimental' as per the HUMAN specification.\n\n\
             ### EXTRACTION RULES ###\n\
             1. LANGUAGE: {}.\n\
             2. FORMAT: VALID JSON OBJECT ONLY.\n\
             3. MANDATORY SCHEMA - MUST INCLUDE ALL FIELDS:\n\
             {{\n\
               \"project_id\": \"MUST be present - this is the project identifier\",\n\
               \"contract_tier\": \"Core\" | \"Optional\" | \"Experimental\",\n\
               \"platform\": \"linux\" | \"win32\" | \"generic\",\n\
               \"subsystem\": \"gtk4\" | \"windowsgui\" | \"console\" | \"posix\",\n\
               \"runtime_model\": \"gtk4\" | \"win32gui\" | \"console\" | \"eventdriven\",\n\
               \"runtime_core\": [\"component_name1\", \"component_name2\"],\n\
               \"components\": [\n                 {{\n\
                   \"name\": \"component_name\",\n\
                   \"file_path\": \"src/component.cpp\",\n\
                   \"status\": \"Core\" | \"Optional\" | \"Experimental\",\n\
                   \"promotion_forbidden\": true,\n\
                   \"blocking_forbidden\": true,\n\
                   \"criticality\": \"CORE\" | \"OPTIONAL\" | \"EXPERIMENTAL\" | \"AUXILIARY\",\n\
                   \"failure_allowed\": true | false\n                 }}\n               ],\n\
               \"forbidden_patterns\": [\"sqlite3\", \"libpq\", \"mysqlclient\", \"ncurses\", \"WinMain\", \"HWND\"]\n\
             }}\n\
             4. EXPLICITLY FORBIDDEN (DO NOT ALLOW):\n\
             - sqlite3, sqlite, libpq, mysqlclient, mysql, mariadb - No database in v0.0.1\n\
             - ncurses, curses, termcap - GTK4 uses Cairo, not terminal\n\
             - WinMain, wWinMain, HWND, WNDCLASS, CreateWindowExW - Win32-specific\n\
             - ORM layers, persistence frameworks, session caches\n\
             - std::thread if not explicitly required\n\
             5. OPTIONAL VS CORE: If the spec mentions 'Choice', 'Optional', '선택', '가변', or 'If needed', mark as 'Optional' or 'Experimental'.\n\
             6. CRITICALITY & FAILURE ALLOWED: Mark 'failure_allowed': true and 'criticality': 'OPTIONAL'/'EXPERIMENTAL' for graceful degradation components (e.g. vi_mode, lsp, tree-sitter, themes) to distinguish them from core components.\n\
             7. RUNTIME CORE: List essential core system module keywords (e.g., gtk4, gtk_application, cairo, lua_runtime, text_buffer, input_handler) in 'runtime_core' array.\n\
             8. PLATFORM CONSISTENCY: If platform is 'linux' and subsystem is 'gtk4', DO NOT add Win32-specific components.\n\
             9. REQUIRED HEADERS: List all required #include headers in each component.\n\
             10. NO HALLUCINATION: Only extract what is explicitly or implicitly mentioned in the spec.\n\
             11. PROJECT_ID MANDATORY: The 'project_id' field MUST always be present in output.\n\n\
             ### SPECIFICATION SOURCE ###\n\
             {}\n\n\
             <JSON_START>\n\
             Generate Immutable Constraints JSON NOW:",
            lang_name,
            spec_content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), None, 0).await?;

        let json_text = extract_json(&resp.text).unwrap_or_else(|| resp.text.trim().to_string());

        // Phase 1: Robust Fuzzy JSON Parsing with structured fallback
        let mut val: serde_json::Value = match serde_json::from_str(&json_text) {
            Ok(v) => normalize_json_root(v).unwrap_or(serde_json::json!({})),
            Err(e) => {
                tracing::warn!("⚠️ [DETERMINISTIC_NORMALIZER] Constraint JSON parsing failed: {}. Normalizer layer will synthesize directly.", e);
                serde_json::json!({})
            }
        };

        // Ensure val is an object
        if !val.is_object() {
            val = serde_json::json!({});
        }

        // Phase 2: Project Metadata Normalization
        if val.get("project_id").is_none() || val["project_id"].as_str().unwrap_or("").is_empty() {
            val["project_id"] = serde_json::json!("default-project");
        }
        if val.get("contract_tier").is_none() {
            val["contract_tier"] = serde_json::json!("Core");
        }

        // Phase 3: Platform Parameter Normalization (Rust as Truth Authority)
        let spec_lower = spec_content.to_lowercase();
        let is_gtk = spec_lower.contains("gtk") || spec_lower.contains("cairo") || spec_lower.contains("g_spawn");
        let is_win32 = spec_lower.contains("win32") || spec_lower.contains("windows.h") || spec_lower.contains("winmain");

        if is_gtk {
            val["platform"] = serde_json::json!("linux");
            val["subsystem"] = serde_json::json!("gtk4");
            val["runtime_model"] = serde_json::json!("gtk4");
            val["language"] = serde_json::json!("c");
        } else if is_win32 {
            val["platform"] = serde_json::json!("win32");
            val["subsystem"] = serde_json::json!("windowsgui");
            val["runtime_model"] = serde_json::json!("win32gui");
            val["language"] = serde_json::json!("cpp");
        } else {
            if val.get("platform").is_none() { val["platform"] = serde_json::json!("generic"); }
            if val.get("subsystem").is_none() { val["subsystem"] = serde_json::json!("console"); }
            if val.get("runtime_model").is_none() { val["runtime_model"] = serde_json::json!("console"); }
            if val.get("language").is_none() { val["language"] = serde_json::json!("c"); }
        }

        // Phase 4: Component Constraint Merging (Deterministic baseline merged with LLM hints)
        let deterministic_components = deterministic_extract_components(spec_content);
        let mut merged_components = std::collections::HashMap::new();

        // 4-1. Populating from LLM response components (if any)
        if let Some(arr) = val.get("components").and_then(|c| c.as_array()) {
            for comp_val in arr {
                if let Some(name) = comp_val.get("name").and_then(|n| n.as_str()) {
                    let mut c = comp_val.clone();
                    if c.get("file_path").is_none() {
                        c["file_path"] = serde_json::json!(name);
                    }
                    if c.get("status").is_none() {
                        c["status"] = serde_json::json!("Core");
                    }
                    if c.get("promotion_forbidden").is_none() {
                        c["promotion_forbidden"] = serde_json::json!(true);
                    }
                    if c.get("blocking_forbidden").is_none() {
                        let is_optional = c.get("status").and_then(|s| s.as_str()).map(|s| s.to_lowercase() == "optional").unwrap_or(false);
                        c["blocking_forbidden"] = serde_json::json!(is_optional);
                    }
                    if c.get("criticality").is_none() {
                        c["criticality"] = serde_json::json!("CORE");
                    }
                    if c.get("failure_allowed").is_none() {
                        c["failure_allowed"] = serde_json::json!(false);
                    }
                    merged_components.insert(name.to_string(), c);
                }
            }
        }

        // 4-2. Forcing deterministic extracted components to bypass LLM extraction bugs
        for det in &deterministic_components {
            if !merged_components.contains_key(&det.name) {
                let is_optional = matches!(det.status, axon_core::spec::ComponentStatus::Optional);
                let c = serde_json::json!({
                    "name": det.name,
                    "file_path": det.file_path.clone().unwrap_or_else(|| det.name.clone()),
                    "status": match det.status {
                        axon_core::spec::ComponentStatus::Core => "Core",
                        axon_core::spec::ComponentStatus::Optional => "Optional",
                        axon_core::spec::ComponentStatus::Experimental => "Experimental",
                    },
                    "promotion_forbidden": true,
                    "blocking_forbidden": is_optional,
                    "criticality": det.criticality.clone().unwrap_or_else(|| "CORE".to_string()),
                    "failure_allowed": det.failure_allowed.unwrap_or(false),
                });
                merged_components.insert(det.name.clone(), c);
            } else {
                let existing = merged_components.get_mut(&det.name).unwrap();
                if existing.get("file_path").is_none() {
                    existing["file_path"] = serde_json::json!(det.file_path.clone().unwrap_or_else(|| det.name.clone()));
                }
            }
        }
        val["components"] = serde_json::json!(merged_components.values().collect::<Vec<_>>());

        // Phase 4-3: Remove hallucinated system library components (e.g. src/gtk.cpp, src/libc.cpp)
        let forbidden_system_files: &[&str] = &[
            "libc", "gtk", "cairo", "pthread", "ncurses",
        ];
        if let Some(arr) = val["components"].as_array_mut() {
            arr.retain(|comp| {
                let name = comp["name"].as_str().unwrap_or("");
                let file_path = comp["file_path"].as_str().unwrap_or("");
                let is_forbidden = forbidden_system_files.iter().any(|pat| {
                    let file_stem = std::path::Path::new(file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    let name_stem = std::path::Path::new(name)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    file_stem.eq_ignore_ascii_case(pat) || name_stem.eq_ignore_ascii_case(pat)
                });
                if is_forbidden {
                    tracing::warn!("🧹 Removing forbidden system library component: '{}' (file: {})", name, file_path);
                }
                !is_forbidden
            });
        }

        // Phase 5: Forbidden Patterns & Core Runtime Normalization
        let mut forbidden = std::collections::HashSet::new();
        if let Some(arr) = val.get("forbidden_patterns").and_then(|f| f.as_array()) {
            for f_val in arr {
                if let Some(s) = f_val.as_str() {
                    forbidden.insert(s.to_string());
                }
            }
        }
        // Force inject constitutional patterns
        let default_forbidden = &["sqlite3", "libpq", "mysqlclient", "ncurses", "WinMain", "HWND"];
        for pat in default_forbidden {
            forbidden.insert(pat.to_string());
        }
        val["forbidden_patterns"] = serde_json::json!(forbidden.into_iter().collect::<Vec<_>>());

        let mut cores = std::collections::HashSet::new();
        if let Some(arr) = val.get("runtime_core").and_then(|rc| rc.as_array()) {
            for rc_val in arr {
                if let Some(s) = rc_val.as_str() {
                    cores.insert(s.to_string());
                }
            }
        }
        if spec_lower.contains("gtk") { cores.insert("gtk4".to_string()); }
        if spec_lower.contains("cairo") { cores.insert("cairo".to_string()); }
        if spec_lower.contains("lua") { cores.insert("lua_runtime".to_string()); }
        val["runtime_core"] = serde_json::json!(cores.into_iter().collect::<Vec<_>>());

        // Hardening metadata default flags
        if val.get("ambiguity_detected").is_none() {
            val["ambiguity_detected"] = serde_json::json!(false);
        }
        if val.get("ambiguity_details").is_none() {
            val["ambiguity_details"] = serde_json::json!([]);
        }
        if val.get("recommended_action").is_none() {
            val["recommended_action"] = serde_json::json!("PROCEED");
        }

        let (_, _, _) = deterministic_extract_sections(spec_content);

        // Phase 6: Deserialize safe structure
        let mut constraints: axon_core::spec::ImmutableConstraints = match serde_json::from_value(val) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("⚠️ [DETERMINISTIC_NORMALIZER] Deserialize failed: {}. Re-synthesizing baseline constraints.", e);
                let spec_id = std::path::Path::new("spec.md")
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let mut recovered = axon_core::spec::ImmutableConstraints::new(format!("{}_recovery", spec_id));
                recovered.components = deterministic_components.clone();
                recovered.ambiguity_detected = false;
                recovered.ambiguity_details.push(format!("Normalizer Recovery triggered due to: {}", e));
                recovered
            }
        };

        // v0.0.33: P1-C(light) Authority Isolation - extract sections with authority levels
        let authority_sections = extract_sections_with_authority(spec_content);
        let hard_sections: Vec<_> = authority_sections.iter()
            .filter(|s| matches!(s.authority, AuthorityLevel::HARD))
            .collect();
        let legacy_sections: Vec<_> = authority_sections.iter()
            .filter(|s| matches!(s.authority, AuthorityLevel::LEGACY))
            .collect();

        if !hard_sections.is_empty() {
            tracing::info!("🔒 [AUTHORITY_ISOLATION] Found {} HARD authority sections", hard_sections.len());
            for section in &hard_sections {
                tracing::debug!("  - Section #{}: {} (lines {}-{})", section.id, section.title, section.line_start, section.line_end);
            }
        }
        if !legacy_sections.is_empty() {
            tracing::warn!("⚠️ [AUTHORITY_ISOLATION] Found {} LEGACY sections - potential contamination", legacy_sections.len());
        }

        let mut undeclared_optional_count = 0;
        let mut core_violations = Vec::new();

        if let Some(ref cores) = constraints.runtime_core {
            for comp in &constraints.components {
                if cores.iter().any(|c_name| comp.name.to_lowercase().contains(&c_name.to_lowercase())) {
                    if comp.status == axon_core::spec::ComponentStatus::Optional 
                        || comp.status == axon_core::spec::ComponentStatus::Experimental 
                    {
                        core_violations.push(format!("CORE runtime component '{}' is marked as Optional/Experimental, which violates structural criticality.", comp.name));
                    }
                }
            }
        }

        for comp in &constraints.components {
            if comp.status == axon_core::spec::ComponentStatus::Optional 
                || comp.status == axon_core::spec::ComponentStatus::Experimental 
            {
                let is_declared = comp.failure_allowed == Some(true)
                    || comp.criticality.as_deref() == Some("OPTIONAL")
                    || comp.criticality.as_deref() == Some("EXPERIMENTAL");
                if !is_declared {
                    undeclared_optional_count += 1;
                }
            }
        }

        let total_count = constraints.components.len();
        let undeclared_ratio = if total_count > 0 { undeclared_optional_count as f64 / total_count as f64 } else { 0.0 };

        if undeclared_ratio > 0.3 {
            constraints.ambiguity_detected = true;
            constraints.ambiguity_details.push(format!(
                "명세의 {:.0}%가 선언되지 않은 모호한 선택적(Undeclared Optional) 요소입니다. Graceful degradation 계약(criticality / failure_allowed) 선언이 필요합니다.",
                undeclared_ratio * 100.0
            ));
        }

        if !core_violations.is_empty() {
            constraints.ambiguity_detected = true;
            constraints.ambiguity_details.extend(core_violations);
        }

        if total_count < 3 {
            constraints.ambiguity_detected = true;
            constraints.ambiguity_details.push("명세에 명시된 컴포넌트가 3개 미만입니다. 설계 결정이 필요할 수 있습니다.".to_string());
        }

        // v0.0.31.32: P1-B Improved ambiguity scoring with threshold
        let (detected, score_details, _score) = calculate_ambiguity_score(spec_content, &deterministic_components);
        if detected {
            constraints.ambiguity_detected = true;
            constraints.ambiguity_details.extend(score_details);
            constraints.recommended_action = "REVIEW_REQUIRED".to_string();
        }

        if spec_content.contains("CREATE TABLE") {
            let has_table_name = spec_content.contains("user_records");
            let has_id_field = spec_content.contains("id") && spec_content.contains("PRIMARY KEY");
            let has_created_at = spec_content.contains("created_at");
            let has_timestamp = spec_content.contains("TIMESTAMP");

            if !has_table_name {
                constraints.ambiguity_details.push("Spec에 명시된 테이블명이 architecture에서 변경되었을 수 있습니다.".to_string());
                constraints.ambiguity_detected = true;
            }
            if !has_id_field || !has_created_at || !has_timestamp {
                constraints.ambiguity_details.push("Spec에 명시된 SQLite 스키마 필드(id, created_at 등)가 architecture에 누락되었을 수 있습니다.".to_string());
                constraints.ambiguity_detected = true;
            }
        }

        if constraints.ambiguity_detected {
            constraints.recommended_action = "REVIEW_REQUIRED".to_string();
            tracing::warn!("⚠️ [AMBIGUITY_DETECTED] 명세 분석 결과 모호함이 감지되었습니다: {:?}", constraints.ambiguity_details);
        } else {
            constraints.recommended_action = "PROCEED".to_string();
        }

        Ok(constraints)
    }

    pub async fn process_bootstrap_step1(&self, task: &Task, error_feedback: Option<String>, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        let (lang_name, lang_instruction) = match self.locale.as_str() {
            "ko_KR" => ("한국어 (Korean)", "생각(Thought), 노가리(Lounge), 주석, 로그 등 모든 텍스트 응답은 반드시 한국어(Korean)로 작성하십시오. 한국어가 최우선 순위이며, 영어(English)는 절대 금지입니다."),
            "ja_JP" => ("日本語 (Japanese)", "すべてのコメントと出力文字列は 반드시 日本語で作成してください。中国語は絶対に使用しないでください。"),
            _ => ("English", "All comments and output strings must be written in English. Do not use any other languages."),
        };

        let model_name = self.agent.model.to_lowercase();
        let is_small_model = model_name.contains("qwen") || model_name.contains("gemma") || model_name.contains("1.8b") || model_name.contains("2b");
        tracing::info!("🔍 [MODEL DIAGNOSIS] Agent: {}, Model: '{}', SmallModel: {}", self.agent.id, self.agent.model, is_small_model);

        // v0.0.22: Stateless Generator / Stateful Learner Paradigm
        // 1. Dynamic Failure Memory Injection (Read recent logs)
        let mut failure_memory = Vec::new();
        if let Ok(file) = std::fs::File::open("trace.log") {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            // Get last 20 lines to find recent violations
            let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
            let last_lines = if lines.len() > 20 { &lines[lines.len()-20..] } else { &lines };
            
            for line in last_lines {
                if let Ok(trace) = serde_json::from_str::<serde_json::Value>(line) {
                    if trace["type"] == "RULE_VIOLATION" || trace["type"] == "VALIDATION_FAILED" {
                        if let Some(msg) = trace["message"].as_str() {
                            failure_memory.push(format!("- {}", msg));
                        } else if let Some(errs) = trace["errors"].as_array() {
                            for e in errs {
                                if let Some(s) = e.as_str() { failure_memory.push(format!("- {}", s)); }
                            }
                        }
                    }
                }
            }
        }
        // Deduplicate and limit to 5
        failure_memory.sort();
        failure_memory.dedup();
        if failure_memory.len() > 5 { failure_memory.truncate(5); }
        
        let failure_context = if !failure_memory.is_empty() {
            format!("\n### ⚠️ RECENT FAILURES (DO NOT REPEAT) ###\n{}\n", failure_memory.join("\n"))
        } else {
            "".to_string()
        };

        let feedback_block = match &error_feedback {
            Some(err) => format!("\n--- [CRITICAL] PREVIOUS ATTEMPT FAILED ---\nERROR: {}\nFIX THE ARCHITECTURE BASED ON THIS ERROR.\n", err),
            None => "".to_string(),
        };

        // v0.0.23: Layered IR Resolution (Global -> Profile -> Project Override)
        let shadow_rules: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut final_ir = serde_json::json!({
            "ir_version": "1.0.0",
            "files": {}, "syntax": {}, "forbidden": { "global": [], "conditional": [] }, "sections": {}
        });

        // 1. Load Global IR (The Constitution)
        if let Ok(global_raw) = std::fs::read_to_string(".axon_registry/global/ir/current.json") {
            if let Ok(global_ir) = serde_json::from_str::<serde_json::Value>(&global_raw) {
                // Merge Global (Baseline)
                final_ir = global_ir; 
            }
        }

        // 2. Load Profiles (The Statutes)
        // (Simplified: iterate active profiles and merge into final_ir)
        let active_profiles = vec!["python_standard"];
        for _p_name in active_profiles {
             // ... profile merge logic ...
        }

        // 3. Apply Project Overrides (The Local Ordinances - Highest Priority)
        if let Ok(override_raw) = std::fs::read_to_string(".axon_registry/projects/current/overrides.json") {
            if let Ok(overrides) = serde_json::from_str::<serde_json::Value>(&override_raw) {
                if let Some(obj) = overrides.as_object() {
                    for (key, val) in obj {
                        // Priority Override: Deep merge or overwrite
                        final_ir[key] = val.clone();
                    }
                }
            }
        }

        let ir = final_ir;

        // 🎯 IR -> PROMPT GENERATION (Flattened)
        let mut contract = Vec::new();
        contract.push("### 🔒 MANDATORY CONTRACT (IR-GENERATED) ###".to_string());
        
        if let Some(files) = ir["files"].as_object() {
            for (f, cfg) in files {
                if cfg["required"].as_bool().unwrap_or(false) {
                    contract.push(format!("- [FILES]: You MUST define '{}' in your architecture.", f));
                }
            }
        }
        if let Some(syntax) = ir["syntax"].as_object() {
            for (name, cfg) in syntax {
                if cfg["required"].as_bool().unwrap_or(false) {
                    contract.push(format!("- [SYNTAX]: MUST include a '{}' block for {}.", cfg["pattern"].as_str().unwrap_or(""), name));
                }
            }
        }
        if let Some(forbidden) = ir["forbidden"].as_object() {
            if let Some(global) = forbidden["global"].as_array() {
                for word in global {
                    contract.push(format!("- [FORBIDDEN]: Do NOT use the word '{}'.", word.as_str().unwrap_or("")));
                }
            }
        }
        
        // v0.0.24: Sovereign Protocol - Forbidden Files (Hard Blacklist)
        contract.push("- [FORBIDDEN_FILES]: You are NOT authorized to modify 'architecture.md', 'mile_stone/', 'release_note/', or '.gemini/'. Any attempt will be REJECTED.".to_string());
        contract.push("- [NO_MARKDOWN]: Do NOT use markdown code blocks (```) inside the AXON protocol. Use ---CODE START--- instead.".to_string());

        let contract_text = contract.join("\n");
        let hot_hints = {
            let mut cache = self.hot_cache.lock().unwrap();
            cache.decay(); // Apply decay per call
            cache.get_hints(5)
        };

        // v0.0.31.27: Staged IR Synthesis check
        let is_staged = is_small_model || task.description.contains("Win32") || task.description.contains("windows.h") || task.description.contains("lua") || task.description.contains("WM_PAINT");

        let mut platform_ir = String::new();
        let mut window_ir = String::new();
        let mut rendering_ir = String::new();
        let mut lua_ir = String::new();

        if is_staged {
            tracing::info!("🚀 [STAGED IR SYNTHESIS] Initiating stage-by-stage IR compilation...");

            // Stage 1: Platform IR
            let stage1_prompt = format!(
                "### STEP 1: Platform IR Generation ###\n\
                 Based on the following project specification, generate the Platform IR details.\n\
                 You MUST extract and specify: \n\
                 - Entry point requirements (e.g. wWinMain vs main)\n\
                 - Native headers needed (e.g. windows.h)\n\
                 - System libraries to link (e.g. user32, gdi32, kernel32)\n\
                 - Forbidden patterns (e.g. MUST NOT use int main())\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 Provide only the Platform IR specification in markdown format. Do not write a conversational preamble.",
                task.description
            );
            if let Ok(r) = self.generate_with_retry(stage1_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await {
                platform_ir = r.text;
                tracing::info!("✅ Stage 1 (Platform IR) completed.");
            } else {
                platform_ir = "Fallback: Win32 GUI subsystem with wWinMain entry and windows.h header.".to_string();
            }

            // Stage 2: Window Runtime IR
            let stage2_prompt = format!(
                "### STEP 2: Window Runtime IR Generation ###\n\
                 Based on the Platform IR and project spec, define the Window Runtime IR.\n\
                 You MUST specify: \n\
                 - Window component definitions (e.g. MainWindow, ActivityBar, Sidebar, EditorCanvas)\n\
                 - HWND ownership structure and parent-child hierarchy.\n\n\
                 ### PLATFORM IR REFERENCE ###\n\
                 {}\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 Provide only the Window Runtime IR specification in markdown format. Do not write a conversational preamble.",
                platform_ir, task.description
            );
            if let Ok(r) = self.generate_with_retry(stage2_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await {
                window_ir = r.text;
                tracing::info!("✅ Stage 2 (Window Runtime IR) completed.");
            } else {
                window_ir = "Fallback: MainWindow owns Sidebar, ActivityBar, and EditorCanvas via HWND parent-child relation.".to_string();
            }

            // Stage 3: Rendering IR
            let stage3_prompt = format!(
                "### STEP 3: Rendering IR Generation ###\n\
                 Based on the previous IRs and project spec, define the Rendering IR.\n\
                 You MUST specify: \n\
                 - Paint handling scope (MUST occur only inside WM_PAINT via BeginPaint/EndPaint)\n\
                 - GDI APIs to use (e.g. TextOut, DrawText)\n\
                 - Thread safety or main loop rendering rules.\n\n\
                 ### WINDOW IR REFERENCE ###\n\
                 {}\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 Provide only the Rendering IR specification in markdown format. Do not write a conversational preamble.",
                window_ir, task.description
            );
            if let Ok(r) = self.generate_with_retry(stage3_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await {
                rendering_ir = r.text;
                tracing::info!("✅ Stage 3 (Rendering IR) completed.");
            } else {
                rendering_ir = "Fallback: All drawing and text rendering MUST occur only inside WM_PAINT scope via BeginPaint and EndPaint.".to_string();
            }

            // Stage 4: Lua Runtime IR
            let stage4_prompt = format!(
                "### STEP 4: Lua Runtime IR Generation ###\n\
                 Based on the previous IRs and project spec, define the Lua Runtime IR.\n\
                 You MUST specify: \n\
                 - lua_State lifecycle (e.g. one lua_State per process)\n\
                 - Lua script ownership over editor logic\n\
                 - C-to-Lua bindings or API export rules.\n\n\
                 ### RENDERING IR REFERENCE ###\n\
                 {}\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 Provide only the Lua Runtime IR specification in markdown format. Do not write a conversational preamble.",
                rendering_ir, task.description
            );
            if let Ok(r) = self.generate_with_retry(stage4_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await {
                lua_ir = r.text;
                tracing::info!("✅ Stage 4 (Lua Runtime IR) completed.");
            } else {
                lua_ir = "Fallback: Single lua_State per process. Lua script controls the editor business logic.".to_string();
            }
        }

        let system_prompt = if is_staged {
            format!(
                "### [LANGUAGE_ENFORCEMENT: {lang_name}] ###\n\
                 - {lang_instruction}\n\n\
                 ### OBJECTIVE ###\n\
                 You are the Lead Architect. Merge the stage-by-stage IR specifications into a single, comprehensive, and executable `architecture.md`.\n\n\
                 Your output MUST include all 5 required sections in detailed format:\n\
                 - ## Components (list of every file path, e.g. src/main.c)\n\
                 - ## Data Schema (exhaustive custom structs/enums field definitions)\n\
                 - ## Data Flow\n\
                 - ## File Map\n\
                 - ## Interfaces (precise function signatures with Owner and Error policies)\n\
                 - ## Semantic Criticality ('core' or 'optional' classification)\n\n\
                 ### CONSTRAINTS ###\n\
                 {}\n\
                 {}\n\n\
                 ### 🗺️ REQUIRED MAPPING BLOCK (MANDATORY AT THE END) ###\n\
                 <!-- AXON:SPEC:COMPONENTS\n\
                 {{ \"components\": [ \n\
                   {{ \"name\": \"Main\", \"file\": \"src/main.c\", \"symbols\": [\"wWinMain\"], \"type\": \"entry\" }},\n\
                   {{ \"name\": \"Header\", \"file\": \"include/main.h\", \"symbols\": [], \"type\": \"header\" }}\n\
                 ] }}\n\
                 -->\n\n\
                 ### STAGE IR RAW DATA ###\n\
                 - PLATFORM IR:\n\
                 {}\n\n\
                 - WINDOW IR:\n\
                 {}\n\n\
                 - RENDERING IR:\n\
                 {}\n\n\
                 - LUA IR:\n\
                 {}\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 FINAL REMINDER: Be exhaustive. Do NOT summarize. Every detail from the spec must be represented.",
                contract_text,
                feedback_block,
                platform_ir,
                window_ir,
                rendering_ir,
                lua_ir,
                task.description
            )
        } else if is_small_model {
            format!(
                "### [LANGUAGE_ENFORCEMENT: {lang_name}] ###\n\
                 - {lang_instruction}\n\n\
                 ### TASK: Generate architecture.md for project: {}.\n\n\
                 {}\n\
                 {}\n\
                 {}\n\
                 {}\n\n\
                 ## Components\n\
                 - main.py: Entry point for {}.\n\n\
                 ## File Map\n\
                 - main.py\n\n\
                 ## Graph\n\
                 ```mermaid\n\
                 graph TD\n\
                   User --> main.py\n\
                 ```\n\n\
                 ### 🗺️ REQUIRED MAPPING ###\n\
                 --- CRITICAL: YOU MUST INCLUDE THIS EXACT BLOCK AT THE END ---\n\
                 <!-- AXON:SPEC:COMPONENTS\n\
                 {{ \"components\": [ {{ \"name\": \"Main\", \"file\": \"[ENTRY_FILE]\", \"symbols\": [\"main\"], \"type\": \"entry\" }} ] }}\n\
                 -->\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 LANGUAGE: {}. Be EXTREMELY BRIEF and follow the contract EXACTLY. Do NOT repeat characters or loop.",
                self.agent.persona.name,
                contract_text,
                failure_context,
                feedback_block,
                hot_hints,
                self.agent.persona.name,
                task.description,
                lang_name
            )
        } else {
            format!(
                "### [LANGUAGE_ENFORCEMENT: {lang_name}] ###\n\
                 - {lang_instruction}\n\n\
                 ### OBJECTIVE ###\n\
                 Generate a COMPREHENSIVE and EXECUTABLE architecture.md for project: {}.\n\n\
                 {}\n\
                 {}\n\n\
                 ### 🏛️ ARCHITECTURE PROTOCOL (v0.0.29 [SEMANTIC_SEALING]) ###
                 YOU MUST follow this structure EXACTLY:
                 ## Components
                 - Detailed list of every file and its specific responsibility.
                 - **OPTIONAL ISOLATION**: DO NOT include 'optional' or 'choice' features from spec.md unless specifically instructed.
                 
                 ## Data Schema
                 - EXHAUSTIVE list of all structs, enums, and custom types.
                 - **NO INFERENCE**: Do NOT invent structs from SQL schemas. Define them ONLY if explicitly required for C interop.
                 - Every field name and type must be defined here.

                 ## Data Flow
                 - Exhaustive step-by-step logic and data movement path.

                 ## File Map
                 - Direct mapping of modules to file paths.

                 ## Interfaces
                 - Precise function signatures, arguments, and return types.
                 - **SEMANTIC CONTRACT**: Explicitly define 'Ownership' (Caller/Callee) and 'Error Handling' for EVERY function.
                 - EVERY custom struct used here MUST be defined in 'Data Schema'.

                 ## Semantic Criticality
                 - Classify every component as 'core' or 'optional'.
                 - 'core': Essential for the MVP. `is_blocking: true`.
                 - 'optional': UI, Logging, or high-complexity features. `is_blocking: false`.

                 ### 🔒 HARD CONSTRAINTS (NON-NEGOTIABLE) ###
                 1. REQUIRED: You MUST include a 'main' entry file and a '```mermaid' block.
                 2. FORBIDDEN: NEVER use 'controller' (Use 'orchestrator'), 'manager', or 'hub'.
                 3. SEMANTIC REDUCTION: If a feature introduces high complexity (e.g. ncurses, external IO) without a clear contract, EXCLUDE it.
                 4. LANGUAGE: Use {}.
                 5. OUTPUT: ONLY markdown content. NO conversational preamble.
                 ### 🗺️ REQUIRED MAPPING BLOCK (MANDATORY AT THE END) ###\n\
                 <!-- AXON:SPEC:COMPONENTS\n\
                 {{ \"components\": [ \n\
                   {{ \"name\": \"Name\", \"file\": \"[ENTRY_FILE]\", \"symbols\": [\"main\"], \"type\": \"entry\" }},\n\
                   {{ \"name\": \"Header\", \"file\": \"[HEADER_FILE]\", \"symbols\": [], \"type\": \"header\" }}\n\
                 ] }}\n\
                 -->\n\n\
                 ### SPECIFICATION SOURCE ###\n\
                 {}\n\n\
                 FINAL REMINDER: Be exhaustive. Do NOT summarize. Every detail from the spec must be represented.",
                self.agent.persona.name,
                failure_context,
                feedback_block,
                lang_name,
                task.description
            )
        };

        let mut last_err_msg: Option<String> = None;
        let max_attempts = 5;

        for attempt in 0..max_attempts {
            let mut current_prompt = system_prompt.clone();
            if let Some(err) = &last_err_msg {
                current_prompt = format!("⚠️ CRITICAL: PREVIOUS ATTEMPT FAILED\nERRORS FOUND:\n{}\nYOU MUST FIX ALL THE ABOVE ERRORS IN THIS ATTEMPT.\n\n{}", err, current_prompt);
            }

            let resp = self.generate_with_retry(current_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await;
            
            match resp {
                Ok(r) => {
                    // 🎯 IR -> DETERMINISTIC VALIDATION (with Shadow Awareness)
                    let mut is_valid = true;
                    let mut errors = Vec::new();
                    let output_text = r.text.clone();
                    let output_lower = output_text.to_lowercase();

                    // 1. Forbidden Words
                    if let Some(forbidden) = ir["forbidden"].as_object() {
                        if let Some(global) = forbidden["global"].as_array() {
                            for word in global {
                                let w = word.as_str().unwrap_or("");
                                let rule_id = format!("forbidden:global:{}", w);
                                if output_lower.contains(w) {
                                    if shadow_rules.contains(&rule_id) {
                                        tracing::info!("🕶️ [SHADOW_VIOLATION] Found '{}' (Shadow Profile)", w);
                                    } else {
                                        let err_msg = format!("Global Forbidden word '{}' found.", w);
                                        errors.push(format!("- {}", err_msg));
                                        
                                        // Update Hot Cache
                                        let mut cache = self.hot_cache.lock().unwrap();
                                        cache.upsert("forbidden_word".to_string(), w.to_string(), format!("Do NOT use the word '{}'", w));
                                        
                                        is_valid = false;
                                    }
                                } else {
                                    // SUCCESS: Log for FPR calculation
                                    tracing::debug!("✅ [RULE_PASS] Forbidden word check: '{}'", w);
                                }
                            }
                        }
                    }

                    // (Other validation rules updated to respect shadow_rules HashSet...)

                    // 2. Required Files
                    if let Some(files) = ir["files"].as_object() {
                        for (f, cfg) in files {
                            if cfg["required"].as_bool().unwrap_or(false) && !output_lower.contains(f) {
                                errors.push(format!("- Required file '{}' missing.", f));
                                is_valid = false;
                            }
                        }
                    }

                    // 3. Syntax Blocks
                    if let Some(syntax) = ir["syntax"].as_object() {
                        for (name, cfg) in syntax {
                            if cfg["required"].as_bool().unwrap_or(false) {
                                let pattern = cfg["pattern"].as_str().unwrap_or("");
                                if !output_text.contains(pattern) {
                                    errors.push(format!("- Missing mandatory '{}' block for {}.", pattern, name));
                                    is_valid = false;
                                }
                            }
                        }
                    }

                    // 4. Mandatory Sections
                    if let Some(sections) = ir["sections"].as_object() {
                        for (section, cfg) in sections {
                            if cfg["required"].as_bool().unwrap_or(false) && !output_lower.contains(&section.to_lowercase()) {
                                errors.push(format!("- Missing mandatory section '{}'.", section));
                                is_valid = false;
                            }
                        }
                    }

                    // 5. Spec Block Validation (v0.0.21 Hardening)
                    if !output_text.contains("<!-- AXON:SPEC:") && !output_text.contains("```spec:") {
                        errors.push("- CRITICAL: Missing AXON:SPEC or spec block. IR Compilation will fail.".to_string());
                        is_valid = false;
                    }

                    // 6. Repetition Protection (Guard against small model 'exploding')
                    if output_text.contains("sharpsharpsharp") || output_text.contains("####") && output_text.matches("#").count() > 100 {
                        errors.push("- Repetition detected. Please re-generate with a clean structure.".to_string());
                        is_valid = false;
                    }

                    if !is_valid {
                        let err = errors.join("\n");
                        tracing::warn!("❌ [VALIDATION FAILED] Attempt {}:\n{}", attempt + 1, err);
                        last_err_msg = Some(err);
                    }

                    if is_valid {
                        tracing::info!("✅ Architect output passed all structural and contract checks.");
                        return Ok(Post {
                            id: uuid::Uuid::new_v4().to_string(),
                            thread_id: task.id.clone(),
                            author_id: self.agent.id.clone(),
                            content: r.text,
                            thought: None,
                            full_code: None,
                            post_type: PostType::Instruction,
                            metrics: Some(axon_core::RuntimeMetrics {
                                total_duration: r.total_duration,
                                eval_count: r.eval_count,
                                eval_duration: r.eval_duration,
                            }),
                            created_at: chrono::Local::now(),
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ Architect generation API error on attempt {}: {}", attempt + 1, e);
                    last_err_msg = Some(e.to_string());
                }
            }
        }
        
        Err(anyhow::anyhow!("Architect failed to generate valid architecture after {} retries. Last errors:\n{}", max_attempts, last_err_msg.unwrap_or_else(|| "Unknown".to_string())))
    }

    pub async fn process_bootstrap_step2(&self, architecture_content: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        self.process_bootstrap_step2_with_context(architecture_content, 0, event_bus).await
    }

    pub async fn process_bootstrap_step2_with_context(&self, architecture_content: &str, context_size: usize, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let log_msg = match self.locale.as_str() {
            "ko_KR" => format!("요원 {} (아키텍트) 2단계: 태스크 분해 중...", self.agent.id),
            "ja_JP" => format!("エージェント {} (アーキテクト) ステージ2: タスク分解中...", self.agent.id),
            _ => format!("Agent {} (Architect) Stage 2: Extracting Tasks...", self.agent.id),
        };
        tracing::info!("{}", log_msg);
        
        let system_prompt = format!(
            "### TASK DISPATCHER (L-DDP Isolation Phase) ###\n\
             ROLE: CTO & CHIEF ARCHITECT. DECOMPOSE INTO ATOMIC TASKS.\n\
             DECOMPOSE THE FOLLOWING ARCHITECTURE INTO ATOMIC TASKS.\n\n\
             ### DISPATCH RULES ###\n\
             1. LANGUAGE: USE {}.\n\
             2. FORMAT: VALID JSON ARRAY OF OBJECTS ONLY.\n\
             3. SCHEMA: {{ \"id\": \"task_id\", \"title\": \"Title\", \"description\": \"desc\", \"component_id\": \"component_name\" }}\n\
             4. component_id: MUST match the physical file path from architecture (e.g. \"src/main.c\")\n\
             5. title: MUST contain filename (e.g. \"Implement dataprocessor.c\")\n\
             6. NO LEAKAGE: Do NOT include logic in descriptions.\n\
             7. MANDATORY IMPLEMENTATION: YOU MUST create tasks for ALL source files (.c, .h, .rs, .py, etc.) in architecture.\n\
                - For every component (type=\"module\" or type=\"entry\"), create a corresponding task.\n\
                - NEVER skip the entry point (e.g. main.c).\n\
             8. SCOPE: ONE TASK PER FILE. DO NOT create separate tasks for individual functions. One task MUST cover the entire file implementation.\n\
             9. TYPE DEFINITION: Every custom struct/enum defined in the architecture MUST be included in full in its corresponding .h task.\n\
             10. ENVELOPE: Wrap your JSON array between <JSON_START> and <JSON_END> tokens.\n\n\
             ### ARCHITECTURE GUIDE ###\n\
             {}\n\n\
             ### EXPECTED OUTPUT EXAMPLE ###\n\
             <JSON_START>\n\
             [\n\
               {{ \"id\": \"task_001\", \"title\": \"Implement calculator.h\", \"description\": \"Core interface\", \"component_id\": \"include/calculator.h\" }},\n\
               {{ \"id\": \"task_002\", \"title\": \"Implement calculator.c\", \"description\": \"Core logic\", \"component_id\": \"src/calculator.c\" }},\n\
               {{ \"id\": \"task_003\", \"title\": \"Implement database.h\", \"description\": \"Database interface\", \"component_id\": \"include/database.h\" }},\n\
               {{ \"id\": \"task_004\", \"title\": \"Implement database.c\", \"description\": \"Database logic\", \"component_id\": \"src/database.c\" }}\n\
             ]\n\
             <JSON_END>\n\n\
             Generate Decomposed Tasks NOW:",
            lang_name,
            architecture_content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), None, context_size).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: "bootstrap-extraction".to_string(),
            author_id: self.agent.id.clone(),
            content: resp.text,
            thought: None,
            full_code: None,
            post_type: PostType::System,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
    }

    pub async fn generate_system_summary(&self, proposal: &Post, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        let _lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let log_msg = match self.locale.as_str() {
            "ko_KR" => format!("시스템이 제안서 {}에 대한 요약 생성 중...", proposal.id),
            "ja_JP" => format!("システムが提案 {} の概要を生成しています...", proposal.id),
            _ => format!("System generating summary for proposal {}...", proposal.id),
        };
        tracing::info!("{}", log_msg);
        
        let system_prompt = if self.locale.as_str() == "ko_KR" {
            format!(
                "당신은 AXON 시스템의 요약 레이어(System Summary Layer)입니다.\n\n\
                 --- 중요: 반드시 아래 지정된 언어로만 답변하십시오 ---\n\
                 언어: 한국어 (Korean)\n\n\
                 --- 주니어 제안 내용 ---\n\
                 {}\n\n\
                 --- 지시 사항 ---\n\
                 위 제안을 분석하여 중립적인 기술 요약을 제공하십시오.\n\
                 1. 변경된 파일 목록을 명시하십시오.\n\
                 2. 핵심 로직 변경 사항을 2-3개의 글머리 기호로 요약하십시오.\n\
                 3. 개인적인 의견, 피드백 또는 위험 분석을 제공하지 마십시오.\n\
                 4. 최대한 간결하게 작성하십시오.",
                 proposal.content
            )
        } else {
            format!(
                "YOU ARE THE AXON SYSTEM SUMMARY LAYER.\n\n\
                  --- LANGUAGE ENFORCEMENT ---\n\
                  YOU MUST GENERATE THE SUMMARY IN THE FOLLOWING LANGUAGE: {}.\n\n\
                 --- JUNIOR PROPOSAL CONTENT ---\n\
                 {}\n\n\
                 --- INSTRUCTION ---\n\
                 ANALYZE THE PROPOSAL ABOVE. PROVIDE A NEUTRAL TECHNICAL SUMMARY.\n\
                 1. LIST CHANGED FILES.\n\
                 2. SUMMARIZE CORE LOGIC CHANGES IN 2-3 BULLET POINTS.\n\
                 3. DO NOT PROVIDE OPINIONS, FEEDBACK, OR RISK ANALYSIS.\n\
                 4. BE CONCISE.",
                 self.locale,
                 proposal.content
            )
        };

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(proposal.thread_id.clone()), 0).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: proposal.thread_id.clone(),
            author_id: "SYSTEM_SUMMARY".to_string(),
            content: resp.text,
            thought: None,
            full_code: None,
            post_type: PostType::System,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
    }

    pub async fn review_proposal(&self, task: &Task, proposal: &Post, summary: Option<&Post>, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Senior) reviewing proposal for task {}...", self.agent.id, task.id);
        
        let summary_content = match summary {
            Some(s) => format!("\n--- SYSTEM SUMMARY ---\n{}\n", s.content),
            None => "".to_string(),
        };

        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let system_prompt = format!(
            "### SYSTEM: AI SENIOR AGENT: {} ###\n\
             [CRITICAL CONSTRAINT]\n\
             NEVER output any markdown code blocks (```cpp ... ```).\n\
             If you need to reject, ONLY output [REJECT] followed by a short textual description of the error.\n\
             If you approve, ONLY output [APPROVE] with optional brief praise.\n\n\
             --- 중요: 반드시 아래 지정된 언어로만 답변하십시오 (FORCE LANGUAGE) ---\n\
             언어: {}\n\n\
             주니어의 제안을 검토하고 승인 여부를 결정하십시오.\n\n\
             --- 태스크 ---\n\
             제목: {}\n\
             설명: {}\n\n\
             --- 주니어 제안 ---\n\
             {}\n\
             {}\n\n\
             --- 검토 규격 (CRITICAL) ---\n\
              1. **Strict Reject Rules**: 논리적 중복(예: x - x), 하드코딩(예: 2023, 2024), 비효율적 조건문 발견 시 **즉시 REJECT**하고 날카로운 독설을 섞은 피드백을 남기십시오.\n\
              2. **KISS 원칙 강제**: '가장 단순한 코드가 최고의 코드'입니다. 불필요하게 복잡하게 꼬아놓은 로직은 지능의 부족을 가리기 위한 기만으로 간주하고 엄격히 평가하십시오.\n\
              3. **C/C++ STAGE 1 Enforcement**: \n\
                 - 헤더(.h) 파일: 전방 선언(Forward Declaration)으로 해결 가능한 타입을 불필요하게 `#include` 했는지 확인하십시오. 위반 시 **즉시 REJECT** 하십시오.\n\
                 - 헤더 내 구현: 함수 본문(Body)이 헤더에 포함되어 있다면 **무조건 REJECT** 하십시오.\n\
                 - 구현(.c/.cpp) 파일: 대응하는 헤더(.h)를 최상단에서 인클루드 하지 않았거나, 암시적 선언(Implicit Declaration)을 방치했다면 **REJECT** 하십시오.\n\
              4. 코드 및 의존성 검증: 코드가 완성된 상태인지, 실행 가능한지, 환각 라이브러리가 없는지 확인하십시오.\n\
               --- OUTPUT FORMAT (ULTRA LIGHT) ---\n\
               First line: [APPROVE] or [REJECT].\n\
               Next lines: free text feedback in {lang_name} (1-2 lines maximum).\n\
               NO JSON. NO MARKDOWN. JUST THE FIRST LINE DECISION.",
            self.agent.persona.name,
            lang_name,
            task.title,
            task.description,
            proposal.content,
            summary_content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: resp.text,
            thought: None,
            full_code: None,
            post_type: PostType::Review,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
    }

    pub async fn validate_architecture(&self, task: &Task, review: &Post, architecture_guide: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Architect) validating architecture for task {}...", self.agent.id, task.id);
        
        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let system_prompt = format!(
            "### SYSTEM: CHIEF ARCHITECT: {} ###\n\
             --- 중요: 반드시 아래 지정된 언어로만 답변하십시오 (FORCE LANGUAGE) ---\n\
             언어: {}\n\n\
             본 작업이 Sovereign Protocol 및 SSOT를 준수하는지 최종 확인하십시오.\n\n\
             --- 아키텍처 가이드 ---\n{}\n\n\
             --- 태스크 ---\n{}\n\n\
             --- 시니어 리뷰 ---\n{}\n\n\
             --- 출력 규격 ---\n\
             1. 분석 과정(<reasoning>)은 생략하십시오.\n\
             2. 준수되었을 경우에만 '[COMPLIANT]'라고 답변하십시오. (반드시 대괄호를 포함할 것)",
            self.agent.persona.name,
            lang_name,
            architecture_guide,
            task.title,
            review.content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone()), 0).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: resp.text,
            thought: None,
            full_code: None,
            post_type: PostType::System,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
    }
}

/// Parses LLM JSON output into a ProjectIR.
/// Handles both formats LLMs produce:
///   - `"components": [{ "name": "foo", "file": "foo.py", "functions": [...] }]` (array — common)
///   - `"components": { "foo": { ... } }` (hashmap — strict)
fn parse_ir_from_llm_json(json: &str) -> anyhow::Result<axon_core::ir::ProjectIR> {

    use axon_core::ir::{Component, Function};

    #[derive(serde::Deserialize)]
    struct RawFunction {
        name: String,
        #[serde(default)]
        signature: String,
    }

    #[derive(serde::Deserialize)]
    struct RawComponent {
        name: String,
        #[serde(default)]
        file: String,
        #[serde(alias = "functions", default)]
        functions: Vec<RawFunction>,
        #[serde(default)]
        is_entrypoint: bool,
        #[serde(default)]
        tier: ComponentTier,
        #[serde(default = "default_true")]
        is_blocking: bool,
        #[serde(rename = "type", default)]
        _type: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct RawIR {
        #[serde(default)]
        node_mapping: std::collections::HashMap<String, String>,
        components: Vec<RawComponent>,
        #[serde(default)]
        language: axon_ir::schema::Language,
        #[serde(default)]
        platform: axon_ir::schema::Platform,
        #[serde(default)]
        subsystem: axon_ir::schema::Subsystem,
        #[serde(default)]
        entrypoint_type: axon_ir::schema::EntrypointType,
        #[serde(default)]
        runtime_model: axon_ir::schema::RuntimeModel,
    }

    let cleaned = clean_json_robust(json);
    let raw: RawIR = serde_json::from_str(&cleaned)?;

    let mut components = BTreeMap::new();
    for c in raw.components {
        let mut functions = BTreeMap::new();
        for f in c.functions {
            let sig = if f.signature.is_empty() {
                format!("{}()", f.name)
            } else {
                f.signature.clone()
            };
            functions.insert(f.name.clone(), Function {
                name: f.name.clone(),
                signature: sig,
                dependencies: BTreeSet::new(),
                body_hash: None,
                locked: false,
            });
        }
        let file = if c.file.is_empty() {
            format!("{}.py", c.name)
        } else {
            c.file.clone()
        };

        // Determine component type with system library auto-classification fallback
        let mut comp_type = match c._type.as_deref().unwrap_or("").to_lowercase().as_str() {
            "system_library" | "system" => axon_ir::schema::types::ComponentType::SystemLibrary,
            "external_runtime" | "external" => axon_ir::schema::types::ComponentType::ExternalRuntime,
            _ => axon_ir::schema::types::ComponentType::ProjectModule,
        };

        let file_lower = file.to_lowercase();
        let base_name = std::path::Path::new(&file_lower)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let name_lower = c.name.to_lowercase();
        let system_libs = ["user32", "gdi32", "kernel32", "shell32", "comdlg32", "gdi"];
        if system_libs.contains(&base_name) || system_libs.contains(&name_lower.as_str()) {
            comp_type = axon_ir::schema::types::ComponentType::SystemLibrary;
        }

        // v0.0.29: Use physical file_path as the primary key for IR components
        // This ensures deterministic lookup from DecomposedTasks that use paths as IDs.
        components.insert(file.clone(), Component {
            name: c.name,
            file_path: file,
            functions,
            imports: BTreeSet::new(),
            associated_files: Vec::new(),
            is_entrypoint: c.is_entrypoint,
            data_models: Vec::new(),
            metadata: BTreeMap::new(),
            allowed_includes: BTreeSet::new(),
            forbidden_includes: BTreeSet::new(),
            forbidden_symbols: BTreeSet::new(),
            tier: c.tier,
            is_blocking: c.is_blocking,
            locked: false,
            component_type: comp_type,
            subsystem: None,
            dll_imports: BTreeSet::new(),
            ownership: axon_ir::OwnershipMetadata::generator_patchable(),
        });
    }

    Ok(axon_core::ir::ProjectIR {
        node_mapping: raw.node_mapping.into_iter().collect(),
        components,
        constraints: Vec::new(),
        constraint_ids: std::collections::HashSet::new(),
        thought: None,
        language: raw.language,
        platform: raw.platform,
        subsystem: raw.subsystem,
        entrypoint_type: raw.entrypoint_type,
        runtime_model: raw.runtime_model,
    })
}

/// v0.0.29: Deterministic JSON cleaner for LLM outputs.
/// Handles common syntax issues and normalization without LLM retries.
fn clean_json_robust(json: &str) -> String {
    let mut s = json.trim().to_string();
    
    // 1. Strip potential trailing semicolons inside function signatures
    // Many LLMs output "signature": "void f();" which might confuse AST parsers.
    // We'll do a simple regex-like replace for common signature end patterns.
    s = s.replace("\";\"", "\"");
    s = s.replace("\");\"", "\")");
    
    // 2. Remove trailing commas in arrays/objects (very common LLM mistake)
    // Simple heuristic: remove comma before closing brace/bracket
    s = s.replace(",\n]", "\n]");
    s = s.replace(",\n}", "\n}");
    s = s.replace(",]", "]");
    s = s.replace(",}", "}");

    s
}

/// v0.0.22: Universal JSON Extraction Helper (Supports { } and [ ])
fn extract_json(raw: &str) -> Option<String> {
    // v0.0.30: Explicitly strip markdown code blocks first
    let mut sanitized = raw.trim().to_string();
    if sanitized.starts_with("```") {
        if let Some(start) = sanitized.find('{') {
            sanitized = sanitized[start..].to_string();
        }
        if let Some(end) = sanitized.rfind('}') {
            sanitized = sanitized[..=end].to_string();
        } else if let Some(end) = sanitized.rfind("```") {
            sanitized = sanitized[..end].to_string();
        }
    }
    
    let raw = sanitized.as_str();
    let bytes = raw.as_bytes();

    let start_obj = {
        let mut found = None;
        if let Some(idx) = raw.find("{") {
            let sub = &raw[idx..];
            // v0.0.30: Added SpecAnalysis keywords (project_id, forbidden_patterns)
            if sub.contains("\"components\"") || sub.contains("\"node_mapping\"") || 
               sub.contains("\"project_id\"") || sub.contains("\"forbidden_patterns\"") {
                found = Some(idx);
            } else {
                if let Some(idx2) = sub[1..].find("{") {
                    found = Some(idx + 1 + idx2);
                } else {
                    found = Some(idx); // Fallback
                }
            }
        }
        found
    };
    let start_arr = {
        let mut found = None;
        let mut pos = 0;
        while pos < bytes.len() {
            if bytes[pos] == b'[' {
                let next = bytes.get(pos + 1).copied().unwrap_or(0);
                // Valid JSON array start: '[' followed by '{', '"', '[', digit, space, or ']' (empty)
                if matches!(next, b'{' | b'"' | b'[' | b']' | b'\n' | b'\r' | b' ' | b'0'..=b'9') {
                    found = Some(pos);
                    break;
                }
                // Skip markdown alert: [!NOTE], [!TIP], [!WARNING], [!CAUTION], [!IMPORTANT]
            }
            pos += 1;
        }
        found
    };
    
    let (start, open_char, close_char) = match (start_obj, start_arr) {
        (Some(i), Some(j)) if i < j => (i, b'{', b'}'),
        (Some(i), Some(j)) if j < i => (j, b'[', b']'),
        (Some(i), None) => (i, b'{', b'}'),
        (None, Some(j)) => (j, b'[', b']'),
        _ => {
            tracing::warn!("⚠️ No '{{' or '[' found in LLM response.");
            return None;
        }
    };
    
    let mut count = 0;
    let mut end = None;
    let bytes = raw.as_bytes();

    for i in start..bytes.len() {
        if bytes[i] == open_char {
            count += 1;
        } else if bytes[i] == close_char {
            count -= 1;
            if count == 0 {
                end = Some(i);
                break;
            }
        }
    }

    match end {
        Some(end_idx) => Some(raw[start..=end_idx].to_string()),
        None => {
            // v0.0.22 Self-Healing: Try to balance the JSON if it was truncated
            if count > 0 {
                tracing::warn!("⚠️ Unbalanced JSON detected (char='{}', count={}). Attempting auto-repair...", open_char as char, count);
                
                // v0.0.25: Iteratively try to find a valid JSON end point by backtracking
                for j in (start..bytes.len()).rev() {
                    let c = bytes[j];
                    // Potential end characters: }, ], ", digit, or last char of true/false/null
                    if matches!(c, b'}' | b']' | b'"' | b'0'..=b'9' | b'e' | b'l' | b'u') {
                        let mut candidate = raw[start..=j].to_string();
                        // We need to re-calculate count for this specific prefix
                        let mut prefix_count = 0;
                        let prefix_bytes = candidate.as_bytes();
                        for &b in prefix_bytes {
                            if b == open_char { prefix_count += 1; }
                            else if b == close_char { prefix_count -= 1; }
                        }
                        
                        if prefix_count >= 0 {
                            for _ in 0..prefix_count {
                                candidate.push(close_char as char);
                            }
                            
                            if serde_json::from_str::<serde_json::Value>(&candidate).is_ok() {
                                tracing::info!("✅ Auto-balanced JSON successfully at index {} after trimming.", j);
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
            None
        }
    }
}

/// v0.0.31.30: Normalize JSON root - Handle LLM array instability
/// Qwen 7B often returns [{...}] instead of {...}, this normalizes to single object
fn normalize_json_root(val: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    match &val {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                anyhow::bail!("[NORMALIZE_ERROR] Empty array returned by LLM");
            }
            tracing::warn!("⚠️ [NORMALIZE] LLM returned array with {} elements, extracting first element", arr.len());
            if let Some(first) = arr.first() {
                Ok(first.clone())
            } else {
                anyhow::bail!("[NORMALIZE_ERROR] Failed to extract first element from array");
            }
        }
        serde_json::Value::Object(_) => Ok(val),
        _ => {
            let type_str = match &val {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "bool",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };
            anyhow::bail!("[NORMALIZE_ERROR] Root JSON must be object or array, got: {}", type_str)
        }
    }
}

/// v0.0.31.32: P1-B Protocol Downsizing — extract code from markdown code block
fn extract_code_block(raw: &str) -> Option<String> {
    let lines: Vec<&str> = raw.lines().collect();
    let mut in_block = false;
    let mut code = String::new();
    for line in lines {
        let trimmed = line.trim();
        if !in_block {
            if trimmed.starts_with("```") {
                in_block = true;
            }
        } else {
            if trimmed.starts_with("```") {
                return Some(code);
            }
            if !code.is_empty() {
                code.push('\n');
            }
            code.push_str(line);
        }
    }
    if !code.is_empty() { Some(code) } else { None }
}

/// v0.0.31.35: P1-C C/C++ raw pattern extractor
/// Markdown code block이 없을 때 C/C++ 코드 영역을 휴리스틱으로 추출
fn extract_cpp_c_code(raw: &str) -> Option<String> {
    // Phase 7-D: JSON array format extraction — handle {"src/main.cpp": ["#include...", ...]} patterns
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw) {
        // Try to find any string array values in the JSON
        let mut all_lines = Vec::new();
        fn collect_string_arrays(val: &serde_json::Value, out: &mut Vec<String>) {
            match val {
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            out.push(s.to_string());
                        }
                    }
                }
                serde_json::Value::Object(obj) => {
                    for v in obj.values() {
                        collect_string_arrays(v, out);
                    }
                }
                _ => {}
            }
        }
        collect_string_arrays(&json, &mut all_lines);
        if !all_lines.is_empty() {
            let code = all_lines.join("\n");
            if code.len() > 20 {
                return Some(code);
            }
        }
    }

    // Also try: {"response": "```cpp...```"} or {"header_file": "..."} patterns
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(response) = json.get("response").and_then(|v| v.as_str()) {
            if response.len() > 20 {
                // Strip markdown code blocks if present
                if let Some(code) = extract_code_block(response) {
                    return Some(code);
                }
                return Some(response.to_string());
            }
        }
        if let Some(header) = json.get("header_file").and_then(|v| v.as_str()) {
            if header.len() > 20 {
                return Some(header.to_string());
            }
        }
    }

    let lines: Vec<&str> = raw.lines().collect();
    let mut code_start: Option<usize> = None;
    let mut code_end: Option<usize> = None;

    // C/C++ 패턴 감지: #include, struct, class, void, int, extern 등
    let cpp_pattern = regex::Regex::new(
        r"^\s*(#include|struct\s|class\s|void\s|int\s|bool\s|float\s|double\s|char\s|extern\s|typedef\s|enum\s|#define|#ifndef|#pragma)"
    ).unwrap();

    for (i, line) in lines.iter().enumerate() {
        if cpp_pattern.is_match(line) {
            if code_start.is_none() {
                code_start = Some(i);
            }
            code_end = Some(i);
        }
    }

    if let (Some(start), Some(end)) = (code_start, code_end) {
        let trim_start = start.saturating_sub(1);
        let trim_end = (end + 1).min(lines.len());
        let code = lines[trim_start..trim_end].join("\n");
        if code.len() > 10 {
            return Some(code);
        }
    }
    None
}

/// Phase 8: Transaction Envelope Parser
/// Validates: BEGIN marker, END marker, BODY presence
/// Returns PatchEnvelope with integrity_errors if any structural check fails
pub fn extract_patch_envelope(raw: &str) -> axon_core::patch::PatchEnvelope {
    use axon_core::patch::PatchEnvelope;

    let mut envelope = PatchEnvelope::new();
    let mut state = "seeking_begin"; // seeking_begin, reading_header, reading_body, done

    for line in raw.lines() {
        let trimmed = line.trim();

        match state {
            "seeking_begin" => {
                if trimmed.contains("===AXON_PATCH_BEGIN===") {
                    state = "reading_header";
                }
            }
            "reading_header" => {
                if trimmed.contains("===AXON_PATCH_BODY===") {
                    state = "reading_body";
                } else if trimmed.starts_with("PATCH_ID:") {
                    envelope.patch_id = trimmed["PATCH_ID:".len()..].trim().to_string();
                } else if trimmed.starts_with("TARGET:") {
                    envelope.target = trimmed["TARGET:".len()..].trim().to_string();
                } else if trimmed.starts_with("PATCH_VERSION:") {
                    envelope.patch_version = trimmed["PATCH_VERSION:".len()..].trim().parse().unwrap_or(2);
                } else if trimmed.starts_with("HUNK_COUNT:") {
                    envelope.hunk_count = trimmed["HUNK_COUNT:".len()..].trim().parse().unwrap_or(1);
                } else if trimmed.starts_with("BYTE_COUNT:") {
                    envelope.byte_count = trimmed["BYTE_COUNT:".len()..].trim().parse().unwrap_or(0);
                } else if trimmed.starts_with("CHECKSUM:") {
                    envelope.checksum = trimmed["CHECKSUM:".len()..].trim().to_string();
                } else if trimmed.contains("===AXON_PATCH_END===") {
                    // END without BODY — truncated
                    envelope.is_complete = true;
                    envelope.validate();
                    return envelope;
                }
            }
            "reading_body" => {
                if trimmed.contains("===AXON_PATCH_END===") {
                    envelope.is_complete = true;
                    state = "done";
                    break;
                } else {
                    if !envelope.body.is_empty() {
                        envelope.body.push('\n');
                    }
                    envelope.body.push_str(line);
                }
            }
            _ => {}
        }
    }

    if state == "reading_body" {
        // END marker missing — truncated completion
        tracing::warn!("⚠️ [PATCH_TRUNCATED] ===AXON_PATCH_END=== missing — completion was cut off");
    } else if state == "reading_header" {
        // BODY marker missing — malformed envelope
        tracing::warn!("⚠️ [PATCH_TRUNCATED] ===AXON_PATCH_BODY=== missing — malformed envelope");
    } else if state == "seeking_begin" {
        // BEGIN marker missing — no envelope at all
        tracing::debug!("⚠️ [NO_ENVELOPE] ===AXON_PATCH_BEGIN=== not found — falling back to legacy extractors");
    }

    envelope.validate();
    envelope
}

/// v0.0.31.32: P1-A Deterministic Component Extractor
/// Pre-pass: Extract components using regex BEFORE LLM inference
/// This is much more stable than LLM extraction for well-structured specs
fn deterministic_extract_components(spec: &str) -> Vec<axon_core::spec::ComponentConstraint> {
    let mut components = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Pattern 1: src/*.cpp, src/*.h, src/*.c files in lists
    let src_file_regex = regex::Regex::new(r"(?m)^\s*src/([a-zA-Z0-9_]+)\.(cpp|h|c)(\s|$|,)").unwrap();
    for cap in src_file_regex.captures_iter(spec) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let ext = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let key = format!("{}.{}", name, ext);
        let key_str = key.clone();
        if !seen.contains(&key_str) && !name.is_empty() {
            seen.insert(key_str);
            components.push(axon_core::spec::ComponentConstraint {
                name: key.clone(),
                file_path: Some(format!("src/{}", key)),
                interface_signature: None,
                status: axon_core::spec::ComponentStatus::Core,
                promotion_forbidden: false,
                blocking_forbidden: false,
                criticality: Some("CORE".to_string()),
                failure_allowed: Some(false),
            });
        }
    }

    // Pattern 2: include/*.h header files
    let include_regex = regex::Regex::new(r"(?m)^\s*include/([a-zA-Z0-9_]+)\.h(\s|$|,)").unwrap();
    for cap in include_regex.captures_iter(spec) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let key = format!("{}.h", name);
        let key_str = key.clone();
        if !seen.contains(&key_str) && !name.is_empty() {
            seen.insert(key_str);
            components.push(axon_core::spec::ComponentConstraint {
                name: key.clone(),
                file_path: Some(format!("include/{}", key)),
                interface_signature: None,
                status: axon_core::spec::ComponentStatus::Core,
                promotion_forbidden: false,
                blocking_forbidden: false,
                criticality: Some("CORE".to_string()),
                failure_allowed: Some(false),
            });
        }
    }

    // Pattern 3: lua/*.lua script files
    let lua_regex = regex::Regex::new(r"(?m)^\s*lua/([a-zA-Z0-9_/]+)\.lua(\s|$|,)").unwrap();
    for cap in lua_regex.captures_iter(spec) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let key = format!("{}.lua", name);
        let key_str = key.clone();
        if !seen.contains(&key_str) && !name.is_empty() {
            seen.insert(key_str);
            components.push(axon_core::spec::ComponentConstraint {
                name: key.clone(),
                file_path: Some(format!("lua/{}", key)),
                interface_signature: None,
                status: axon_core::spec::ComponentStatus::Core,
                promotion_forbidden: false,
                blocking_forbidden: false,
                criticality: Some("CORE".to_string()),
                failure_allowed: Some(false),
            });
        }
    }

    // Pattern 4: Table format | src/filename | description |
    let table_regex = regex::Regex::new(r"(?m)\|\s*src/([a-zA-Z0-9_]+\.(?:cpp|h|c|lua))\s*\|").unwrap();
    for cap in table_regex.captures_iter(spec) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let name_str = name.to_string();
        if !seen.contains(&name_str) && !name.is_empty() {
            seen.insert(name_str.clone());
            components.push(axon_core::spec::ComponentConstraint {
                name: name.to_string(),
                file_path: Some(format!("src/{}", name)),
                interface_signature: None,
                status: axon_core::spec::ComponentStatus::Core,
                promotion_forbidden: false,
                blocking_forbidden: false,
                criticality: Some("CORE".to_string()),
                failure_allowed: Some(false),
            });
        }
    }

    tracing::info!("🔍 [DETERMINISTIC_EXTRACTOR] Found {} components from spec", components.len());
    components
}

/// v0.0.31.32: P1-C Section-aware Parsing
/// Extract sections to reduce noise in ambiguity detection
fn deterministic_extract_sections(spec: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut required = Vec::new();
    let mut forbidden = Vec::new();
    let mut notes = Vec::new();

    // Extract REQUIRED sections
    let required_regex = regex::Regex::new(r"(?im)^#+\s*\d+\..*REQUIRED|^\s*REQUIRED:|(?im)^\s*MUST\s+").unwrap();
    for cap in required_regex.captures_iter(spec) {
        if let Some(m) = cap.get(0) {
            let line = m.as_str().to_string();
            if !required.iter().any(|s: &String| s.contains(&line)) {
                required.push(line);
            }
        }
    }

    // Extract FORBIDDEN sections
    let forbidden_regex = regex::Regex::new(r"(?im)^#+\s*\d+\..*FORBIDDEN|^\s*FORBIDDEN:|(?im)^\s*STRICTLY FORBIDDEN").unwrap();
    for cap in forbidden_regex.captures_iter(spec) {
        if let Some(m) = cap.get(0) {
            let line = m.as_str().to_string();
            if !forbidden.iter().any(|s: &String| s.contains(&line)) {
                forbidden.push(line);
            }
        }
    }

    // Extract NOTE/Reason sections (these should be excluded from ambiguity)
    let note_regex = regex::Regex::new(r"(?im)^#+\s*\d+\..*NOTE|^\s*NOTE:|^\s*Reason:|^\s*Reason\n").unwrap();
    for cap in note_regex.captures_iter(spec) {
        if let Some(m) = cap.get(0) {
            let line = m.as_str().to_string();
            if !notes.iter().any(|s: &String| s.contains(&line)) {
                notes.push(line);
            }
        }
    }

    (required, forbidden, notes)
}

/// v0.0.31.33: P1-C(light) Authority Isolation - Section Registry with Authority Levels
#[derive(Debug, Clone)]
pub enum AuthorityLevel {
    HARD,     // PRIMARY RULE, FORBIDDEN, STRICTLY, REQUIRED
    REQUIRED, // REQUIRED, MUST
    PATCH,    // 버그 픽스 반영, v0.0.x 예정
    SOFT,     // NOTE, Reason, Example
    FUTURE,   // v0.0.2 예정, later, future
    LEGACY,   // historical, deprecated, legacy
}

#[derive(Debug, Clone)]
pub struct SpecSection {
    pub id: String,
    pub title: String,
    pub authority: AuthorityLevel,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Extract sections with authority levels for contamination isolation
fn extract_sections_with_authority(spec: &str) -> Vec<SpecSection> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = spec.lines().collect();

    // Regex for main section (# XX)
    let main_section_regex = regex::Regex::new(r"(?m)^#\s+(\d+)\.?\s*(.*)$").unwrap();
    // Regex for subsection (## XX)
    let subsection_regex = regex::Regex::new(r"(?m)^##\s+(\d+\.?\d*)\.?\s*(.*)$").unwrap();
    let mut current_section: Option<(String, String, usize)> = None;
    let mut current_content = String::new();

    for (idx, line) in lines.iter().enumerate() {
        // Check for main section
        if let Some(caps) = main_section_regex.captures(line) {
            // Save previous section if exists
            if let Some((id, title, start)) = current_section.take() {
                let title_ref: &str = &title;
                let content_ref: &str = &current_content;
                sections.push(SpecSection {
                    id: id.clone(),
                    title: title.clone(),
                    authority: classify_authority(title_ref, content_ref),
                    content: current_content.clone(),
                    line_start: start,
                    line_end: idx,
                });
            }
            // Start new section
            let id = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let title = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim().to_string();
            current_section = Some((id, title, idx));
            current_content = String::new();
        }
        // Check for subsection
        else if subsection_regex.is_match(line) {
            // For now, append subsection to parent content
            current_content.push_str(line);
            current_content.push('\n');
        }
        // Regular content
        else if current_section.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if let Some((id, title, start)) = current_section.take() {
        let title_ref: &str = &title;
        let content_ref: &str = &current_content;
        let auth = classify_authority(title_ref, content_ref);
        sections.push(SpecSection {
            id,
            title,
            authority: auth,
            content: current_content,
            line_start: start,
            line_end: lines.len(),
        });
    }

    tracing::info!("🔍 [AUTHORITY_ISOLATION] Found {} sections with authority levels", sections.len());
    sections
}

/// Classify authority level based on section title and content
fn classify_authority(title_str: &str, content_str: &str) -> AuthorityLevel {
    let title_lower = title_str.to_lowercase();
    let content_lower = content_str.to_lowercase();

    // HARD: PRIMARY RULE, FORBIDDEN, STRICTLY FORBIDDEN, MUST NOT
    if title_lower.contains("primary rule")
        || title_lower.contains("forbidden")
        || title_lower.contains("strictly")
        || title_lower.contains("must not")
        || content_lower.contains("strictly forbidden")
        || content_lower.contains("must not")
    {
        return AuthorityLevel::HARD;
    }

    // REQUIRED: REQUIRED, MUST
    if title_lower.contains("required")
        || title_lower.contains("must")
        || title_lower.contains("entry point")
    {
        return AuthorityLevel::REQUIRED;
    }

    // PATCH: 버그 픽스, patch notes, v0.0.x
    if title_lower.contains("버그 픽스")
        || title_lower.contains("patch")
        || title_lower.contains("버그 수정")
    {
        return AuthorityLevel::PATCH;
    }

    // FUTURE: later, future, upcoming, v0.0.2
    if title_lower.contains("예정")
        || title_lower.contains("future")
        || title_lower.contains("later")
        || title_lower.contains("v0.0.2")
        || title_lower.contains("v0.0.3")
    {
        return AuthorityLevel::FUTURE;
    }

    // LEGACY: deprecated, legacy, historical
    if title_lower.contains("deprecated")
        || title_lower.contains("legacy")
        || title_lower.contains("historical")
        || content_lower.contains("deprecated")
    {
        return AuthorityLevel::LEGACY;
    }

    // SOFT: NOTE, Reason, Example, IMPLEMENTATION
    if title_lower.contains("note")
        || title_lower.contains("reason")
        || title_lower.contains("example")
        || title_lower.contains("implementation")
    {
        return AuthorityLevel::SOFT;
    }

    // Default to REQUIRED for main content sections
    AuthorityLevel::REQUIRED
}

/// Get authority precedence for contamination resolution
#[allow(dead_code)]
fn get_authority_precedence(level: &AuthorityLevel) -> u8 {
    match level {
        AuthorityLevel::HARD => 10,
        AuthorityLevel::REQUIRED => 7,
        AuthorityLevel::PATCH => 5,
        AuthorityLevel::SOFT => 3,
        AuthorityLevel::FUTURE => 2,
        AuthorityLevel::LEGACY => 1,
    }
}

/// v0.0.31.32: P1-B Improved Ambiguity Scoring
/// Score-based ambiguity detection with threshold
fn calculate_ambiguity_score(spec: &str, hard_constraints: &[axon_core::spec::ComponentConstraint]) -> (bool, Vec<String>, f32) {
    let mut score = 0.0;
    let mut details = Vec::new();

    // High-weight patterns (真正의 ambiguity)
    let high_weight = [
        ("TODO", 3.0),
        ("TBD", 3.0),
        ("not decided", 3.0),
        ("unclear", 2.5),
        ("if needed", 2.0),
        ("when available", 2.0),
        ("depends on", 2.0),
    ];

    // Low-weight patterns (declarative optional, not ambiguity)
    let low_weight = [
        ("optional", 0.5),
        ("선택", 0.5),
        ("추가", 0.3),
        ("could", 0.5),
        ("may", 0.5),
    ];

    // Check high weight patterns
    for (pattern, weight) in high_weight {
        let count = spec.to_lowercase().matches(&pattern.to_lowercase()).count();
        if count > 0 {
            score += weight * count as f32;
            details.push(format!("Found '{}' {} times (weight: {})", pattern, count, weight));
        }
    }

    // Check low weight patterns
    for (pattern, weight) in low_weight {
        let count = spec.to_lowercase().matches(&pattern.to_lowercase()).count();
        if count > 0 {
            score += weight * count as f32;
        }
    }

    // Hard constraint discount: if we have many components, reduce ambiguity
    let hard_count = hard_constraints.len() as f32;
    if hard_count >= 5.0 {
        score = (score - hard_count * 0.3).max(0.0);
        details.push(format!("Hard constraint discount: -{} ({} components)", hard_count * 0.3, hard_count));
    }

    // Threshold: 5.0
    let detected = score >= 5.0;
    if detected {
        details.insert(0, format!("Ambiguity score: {} (threshold: 5.0)", score));
    }

    (detected, details, score)
}

/// v0.0.31.31: Schema Validator - Validate ImmutableConstraints schema before deserialization
/// Prevents "missing field" errors by checking required fields upfront
#[allow(dead_code)]
fn validate_constraint_schema(val: &serde_json::Value) -> anyhow::Result<()> {
    let obj = val.as_object().ok_or_else(|| anyhow::anyhow!("[SCHEMA_VALIDATOR] Value is not a JSON object"))?;

    // Required fields for ImmutableConstraints
    let required_fields = ["project_id", "components", "contract_tier"];
    for field in required_fields {
        if !obj.contains_key(field) {
            tracing::warn!("⚠️ [SCHEMA_VALIDATOR] Missing required field: {}", field);
            // Not fatal - we have default injection logic below
        }
    }

    // Validate components is an array if present
    if let Some(components) = obj.get("components") {
        if !components.is_array() {
            let type_str = match components {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "bool",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };
            anyhow::bail!("[SCHEMA_VALIDATOR] 'components' must be an array, got: {}", type_str);
        }
    }

    Ok(())
}

/// v0.0.26: Fuzzy JSON Repair Engine
/// Handles unquoted keys, trailing commas, and Python-style values.
fn auto_repair_json_fuzzy(json: &str) -> String {
    let mut s = json.to_string();
    
    // 1. Fix unquoted keys: { key: "value" } -> { "key": "value" }
    // More aggressive regex to catch keys even if they start with numbers or follow complex whitespace
    let key_regex = regex::Regex::new(r"(?m)([{,]\s*)([a-zA-Z0-9_\-]+)\s*:").unwrap();
    s = key_regex.replace_all(&s, "$1\"$2\":").to_string();

    // 2. Fix Python-style booleans and nulls
    s = s.replace("True", "true")
         .replace("False", "false")
         .replace("None", "null")
         .replace(",,", ",");

    // 3. Fix trailing commas: [1, 2, ] -> [1, 2]
    let trailing_comma_regex = regex::Regex::new(r",\s*([\]}])").unwrap();
    s = trailing_comma_regex.replace_all(&s, "$1").to_string();

    // 4. Fix double quotes issue (v0.0.26)
    s = s.replace("\": \"\"", "\": \"");

    s
}

/// AXON Patch Protocol v2: Deterministic FSM Parser (Robust)
fn strip_markdown(content: &str) -> String {
    content.lines()
        .filter(|line| !line.trim().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn extract_axon_patch_v2_simple(text: &str) -> Option<axon_core::patch::Patch> {
    use axon_core::patch::{Patch, FilePatch, PatchAction};
    let mut patch = Patch::new();

    let re_thought = regex::Regex::new(r"(?m)^\s*(?://|#|--)?\s*THOUGHT:\s*(.*)$").unwrap();
    let re_file = regex::Regex::new(r"(?m)^\s*(?://|#|--)?\s*FILE:\s*(.*)$").unwrap();
    let re_action = regex::Regex::new(r"(?m)^\s*(?://|#|--)?\s*ACTION:\s*(.*)$").unwrap();
    
    let start_marker = "---CODE START---";
    let end_marker = "---CODE END---";

    if let Some(caps) = re_thought.captures(text) {
        patch.thought = Some(caps.get(1).unwrap().as_str().trim().to_string());
    }

    // Strategy 1: Standard Extraction (With Markers)
    if let Some(file_caps) = re_file.captures(text) {
        let filename = file_caps.get(1).unwrap().as_str().trim().to_string();
        let action_str = if let Some(act_caps) = re_action.captures(text) {
            act_caps.get(1).unwrap().as_str().trim().to_lowercase()
        } else {
            "rewrite".to_string()
        };

        let lines: Vec<&str> = text.lines().collect();
        let mut start_idx = None;
        let mut end_idx = None;

        for (i, line) in lines.iter().enumerate() {
            if line.contains(start_marker) {
                start_idx = Some(i);
            } else if line.contains(end_marker) {
                end_idx = Some(i);
                break;
            }
        }

        if let (Some(s), Some(e)) = (start_idx, end_idx) {
            if s < e {
                let code = lines[s + 1..e].join("\n");
                patch.files.push(FilePatch {
                    path: filename,
                    action: match action_str.as_str() {
                        "append" => PatchAction::Append,
                        "delete" => PatchAction::Delete,
                        _ => PatchAction::Rewrite,
                    },
                    code: strip_markdown(&code),
                });
                return Some(patch);
            }
        }
    }

    // Strategy 2: Fuzzy Extraction (Missing Markers, but has END marker)
    if text.contains("===AXON_PATCH_END===") {
        tracing::warn!("⚠️ [PARSER_RECOVERY] Markers missing. Trying robust markdown extraction...");
        let end_pos = text.find("===AXON_PATCH_END===").unwrap();
        let pre_end = &text[..end_pos];
        
        // Find the FIRST markdown block opening
        if let Some(start_pos) = pre_end.find("```") {
            let extracted = pre_end[start_pos..].to_string();
            let clean_code = strip_markdown(&extracted);
            if !clean_code.is_empty() {
                // Try to guess filename from text or task context (materializer will help)
                let guessed_file = re_file.captures(text)
                    .map(|c| c.get(1).unwrap().as_str().trim().to_string())
                    .unwrap_or_else(|| "unknown_recovered.c".to_string());

                patch.files.push(FilePatch {
                    path: guessed_file,
                    action: PatchAction::Rewrite,
                    code: clean_code,
                });
                return Some(patch);
            }
        }
    }

    if patch.files.is_empty() { None } else { Some(patch) }
}

pub fn extract_axon_patch_v2(input: &str) -> Option<axon_core::patch::Patch> {
    #[derive(PartialEq)]
    enum State { Idle, InPatch, InFile, InCode }
    
    let mut state = State::Idle;
    let mut current_file: Option<axon_core::patch::FilePatch> = None;
    let mut patch = axon_core::patch::Patch::new();

    for line in input.lines() {
        let line_trimmed = line.trim();
        let clean_line = line_trimmed.trim_start_matches(|c| c == '/' || c == '#' || c == ' ' || c == '*').trim();
        
        match state {
            State::Idle => {
                if clean_line.contains("===AXON_PATCH_START===") {
                    state = State::InPatch;
                }
            }
            State::InPatch => {
                if clean_line.starts_with("THOUGHT:") {
                    patch.thought = Some(clean_line[8..].trim().to_string());
                } else if clean_line.contains("===AXON_PATCH_END===") {
                    state = State::Idle;
                } else if clean_line.starts_with("FILE:") {
                    let path = clean_line[5..].trim().trim_matches(|c| c == '`' || c == '"' || c == '\'').to_string();
                    if !path.is_empty() {
                        current_file = Some(axon_core::patch::FilePatch {
                            path,
                            action: axon_core::patch::PatchAction::Rewrite,
                            code: String::new(),
                        });
                        state = State::InFile;
                    }
                } else if !clean_line.is_empty() {
                    // v0.0.28: Support multi-line thoughts
                    if let Some(ref mut t) = patch.thought {
                        t.push('\n');
                        t.push_str(line_trimmed);
                    }
                }
            }
            State::InFile => {
                if clean_line.starts_with("ACTION:") {
                    let action_str = clean_line[7..].trim().to_lowercase();
                    if let Some(ref mut f) = current_file {
                        f.action = match action_str.as_str() {
                            "append" => axon_core::patch::PatchAction::Append,
                            "delete" => axon_core::patch::PatchAction::Delete,
                            _ => axon_core::patch::PatchAction::Rewrite,
                        };
                    }
                } else if clean_line.contains("---CODE START---") {
                    state = State::InCode;
                } else if clean_line.contains("===AXON_PATCH_END===") {
                    if let Some(f) = current_file.take() {
                        patch.files.push(f);
                    }
                    state = State::Idle;
                }
            }
            State::InCode => {
                if clean_line.contains("---CODE END---") {
                    if let Some(mut f) = current_file.take() {
                        f.code = strip_markdown(&f.code.trim_end().to_string());
                        patch.files.push(f);
                    }
                    state = State::InPatch;
                } else {
                    if let Some(ref mut f) = current_file {
                        f.code.push_str(line);
                        f.code.push('\n');
                    }
                }
            }
        }
    }
    
    if state == State::InCode {
        if let Some(mut f) = current_file.take() {
            // v0.0.25: Step 4 - Eliminate markdown pollution before storing
            f.code = strip_markdown(&f.code);
            // v0.0.26: Aggressive marker stripping to prevent leakage
            f.code = f.code.replace("---CODE START---", "").replace("---CODE END---", "").trim().to_string();
            patch.files.push(f);
        }
    } else {
        // v0.0.26: Even if we are not in InCode, check already pushed files
        for f in &mut patch.files {
            f.code = f.code.replace("---CODE START---", "").replace("---CODE END---", "").trim().to_string();
        }
    }
    
    if patch.files.is_empty() { None } else { Some(patch) }
}

fn auto_repair_v2(input: &str) -> String {
    let mut working_text = input.to_string();
    
    // --- Level 0: JSON Unwrapping (for Llama3 style hallucinations) ---
    if working_text.trim().starts_with("[") || working_text.trim().starts_with("{") {
        if let Some(json_str) = extract_json(&working_text) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                // If it is a list, look for "code" field in the first element
                if let Some(first) = val.as_array().and_then(|a| a.get(0)) {
                    if let Some(code) = first["code"].as_str() {
                        working_text = code.to_string();
                    }
                } else if let Some(code) = val["code"].as_str() {
                    working_text = code.to_string();
                }
            }
        }
    }

    let lines: Vec<String> = working_text.lines().map(|l| l.to_string()).collect();
    let mut repaired = Vec::new();

    // --- Level 1: Safe Fixes ---
    let has_start = lines.iter().any(|l| l.contains("===AXON_PATCH_START==="));
    let has_end = lines.iter().any(|l| l.contains("===AXON_PATCH_END==="));
    
    if !has_start {
        repaired.push("===AXON_PATCH_START===".to_string());
    }

    let mut in_file_spec = false;
    let mut in_code_block = false;

    for line in lines {
        let trimmed = line.trim();
        let clean = trimmed.trim_start_matches(|c| c == '/' || c == '#' || c == ' ' || c == '*').trim();
        
        if clean.starts_with("FILE:") {
            if in_code_block {
                repaired.push("---CODE END---".to_string());
                in_code_block = false;
            }
            in_file_spec = true;
            repaired.push(line);
            continue;
        }

        if clean.starts_with("ACTION:") {
            repaired.push(line);
            continue;
        }

        if clean.contains("---CODE START---") {
            in_code_block = true;
            in_file_spec = false;
            repaired.push(line);
            continue;
        }

        if clean.contains("---CODE END---") {
            in_code_block = false;
            repaired.push(line);
            continue;
        }

        if in_file_spec && !trimmed.is_empty() && !clean.starts_with("ACTION:") {
            repaired.push("---CODE START---".to_string());
            in_code_block = true;
            in_file_spec = false;
        }

        repaired.push(line);
    }

    if in_code_block {
        repaired.push("---CODE END---".to_string());
    }
    if !has_end {
        repaired.push("===AXON_PATCH_END===".to_string());
    }

    let mut output = repaired.join("\n");
    output = output.replace("\\n", "\n").replace("\\\"", "\"").replace("\\\\", "\\");

    if !output.contains("FILE:") && (output.contains("def ") || output.contains("fn ") || output.contains("class ")) {
        output = format!(
            "===AXON_PATCH_START===\nFILE: recovery_logic.py\nACTION: rewrite\n---CODE START---\n{}\n---CODE END---\n===AXON_PATCH_END===",
            output
        );
    }

    output
}

/// v0.0.23: Legacy Raw Code Tagging Parser (Kept for fallback)
#[allow(dead_code)]
fn extract_raw_code_as_json(raw: &str) -> Option<String> {
    let mut target = "unknown";
    let mut patch_type = "rewrite";
    
    for line in raw.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.to_uppercase().starts_with("# TARGET:") {
            target = line_trimmed[9..].trim();
        } else if line_trimmed.to_uppercase().starts_with("# TYPE:") {
            patch_type = line_trimmed[7..].trim();
        }
        if line_trimmed.contains("---CODE START---") {
            break;
        }
    }

    let start_tag = "---CODE START---";
    let end_tag = "---CODE END---";
    
    let start_idx = raw.find(start_tag)? + start_tag.len();
    let end_idx = raw.find(end_tag)?;
    
    let code = raw[start_idx..end_idx].trim();
    
    let patch = serde_json::json!([{
        "target": target,
        "type": patch_type,
        "code": code
    }]);
    
    Some(patch.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_unbalanced_with_trailing_text() {
        let raw_text = r#"
**JSON Block**
{
  "foo": [
    { "bar": 1 }
  ]

**Verification Targets**
1. Fix this
"#;
        let result = extract_json(raw_text);
        assert!(result.is_some(), "Should extract JSON even if unbalanced with trailing text");
    }

    #[test]
    fn test_extract_json_user_reported_fail() {
        let raw_text = r#"
Based on the provided Axon IR Spec (Rust) v0.4-Heavy, I will generate a deterministic architecture specification for AXON.

**JSON Block**

{
"components": [
{
"name": "input_handler",
"file": "input.rs",
"functions": [
{ "name": "get_name", "signature": "get_name()" },
{ "name": "get_year", "signature": "get_year()" }
]
}
]

**Verification Targets**

1. Loop detection correctness
2. Bypass edge integrity
"#;
        let result = extract_json(raw_text);
        assert!(result.is_some(), "Should extract user reported case");
        let json = result.unwrap();
        assert!(json.ends_with("]}"), "Should have balanced the JSON at the right spot");
    }
}

