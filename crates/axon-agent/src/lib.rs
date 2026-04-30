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

use axon_core::{Agent, Post, PostType, AgentRole, Task};
use axon_model::{ModelDriver, ModelResponse};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
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

pub struct AgentRuntime {
    pub agent: Agent,
    pub model: Arc<dyn ModelDriver + Send + Sync>,
    pub locale: String, // v0.0.15: System language preference
    pub timeout: std::time::Duration,
    pub throttler: Option<Arc<tokio::sync::Semaphore>>,
    pub hot_cache: Arc<Mutex<HotRuleCache>>,
    pub project_id: String,
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
        }
    }

    pub fn with_project(mut self, project_id: String) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = std::time::Duration::from_secs(seconds);
        self
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    async fn generate_with_retry(&self, prompt: String, event_bus: Option<&Arc<axon_core::events::EventBus>>, thread_id: Option<String>) -> anyhow::Result<ModelResponse> {
        if let Some(bus) = event_bus {
            bus.publish(axon_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: self.project_id.clone(),
                thread_id: thread_id.clone(),
                agent_id: Some(self.agent.id.clone()),
                event_type: axon_core::EventType::AgentAction,
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

            let gen_future = self.model.generate(prompt.clone());
            match tokio::time::timeout(self.timeout, gen_future).await {
                Ok(Ok(resp)) => {
                    if let Some(bus) = event_bus {
                        bus.publish(axon_core::Event {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: self.project_id.clone(),
                            thread_id: thread_id.clone(),
                            agent_id: Some(self.agent.id.clone()),
                            event_type: axon_core::EventType::AgentResponse,
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

    pub async fn process_task(&self, task: &Task, architecture_guide: &str, error_feedback: Option<String>, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let log_msg = match self.locale.as_str() {
            "ko_KR" => format!("요원 {} (주니어)가 태스크 {}를 처리 중입니다...", self.agent.id, task.id),
            "ja_JP" => format!("エージェント {} (ジュニア) がタスク {} を処理しています...", self.agent.id, task.id),
            _ => format!("Agent {} (Junior) processing task {}...", self.agent.id, task.id),
        };
        tracing::info!("{}", log_msg);
        
        let feedback_block = match &error_feedback {
            Some(err) => format!("\n--- [CRITICAL] PREVIOUS ATTEMPT FAILED ---\nERROR: {}\nFIX THE CODE BASED ON THIS ERROR.\n", err),
            None => "".to_string(),
        };

        let guide_limit = 2000;
        let short_guide = if architecture_guide.len() > guide_limit {
            format!("{}... [TRUNCATED]", &architecture_guide[..guide_limit])
        } else {
            architecture_guide.to_string()
        };

        let system_prompt = format!(
            "### AI JUNIOR AGENT: {} ###\n\
             ROLE: Implement the task below using the provided guide.\n\
             LANG: {}\n\n\
             {}\n\n\
             ### ARCHITECTURE GUIDE ###\n\
             {}\n\n\
             ### TASK ###\n\
             TITLE: {}\n\
             DESC: {}\n\n\
             ### OUTPUT RULE (STRICT) ###\n\
             1. YOU MUST OUTPUT A VALID JSON ARRAY ONLY.\n\
             2. NO EXPLANATORY TEXT. NO MARKDOWN OUTSIDE THE JSON.\n\
             3. FORMAT: [{{ \"target\": \"filename\", \"type\": \"rewrite\", \"code\": \"...\" }}]\n\
             4. Ensure the 'code' field contains the full source code for the file.",
            self.agent.persona.name, lang_name, feedback_block, short_guide, task.title, task.description
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        // v0.0.22: CRITICAL RESOURCE BOTTLENECK PROTECTION
        // If Ollama returns empty content due to memory/GPU timeout, DO NOT treat it as success.
        // v0.0.22: Flexible validation for small models
        if resp.text.trim().is_empty() {
            tracing::error!("❌ [RESOURCE ERROR]: Junior produced an empty response.");
            return Err(anyhow::anyhow!("Ollama produced empty response. Check context limits."));
        }

        let clean_json = extract_json(&resp.text)
            .ok_or_else(|| anyhow::anyhow!("Failed to find valid JSON in Junior response. Raw: {}", resp.text))?;

        let full_code = Some(clean_json.clone());

        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: resp.text,
            full_code,
            post_type: PostType::Proposal,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
    }

    pub async fn generate_ir(&self, spec: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::ir::IR> {
        // v0.0.22: Token Overflow Protection (Simple Truncate for 1.8B models)
        let model_name = self.agent.model.to_lowercase();
        let is_small = model_name.contains("qwen") || model_name.contains("1.8b") || model_name.contains("2b");
        
        let processed_spec = if is_small && spec.len() > 4000 {
            tracing::warn!("⚠️ Spec is too large ({} bytes). Truncating for {} to 4000 bytes...", spec.len(), self.agent.model);
            format!("{}... [TRUNCATED]", &spec[..4000])
        } else {
            spec.to_string()
        };

        let system_prompt = format!(
            "### SOURCE SPECIFICATION ###\n\
             {}\n\n\
             ### TASK ###\n\
             Based on the specification above, define the system architecture in the following JSON IR format.\n\
             STRICT RULES:\n\
             1. ONLY output the JSON object. NO talk.\n\
             2. Use lowercase 'true'/'false'.\n\
             3. Do NOT use Python comments (#).\n\
             4. The root key MUST be \"files\".\n\n\
             ### EXPECTED OUTPUT FORMAT ###\n\
             {{\n\
               \"files\": [\n\
                 {{\n\
                   \"name\": \"example.rs\",\n\
                   \"responsibility\": \"Example role\",\n\
                   \"functions\": [\n\
                     {{ \"name\": \"example_func\", \"args\": [], \"returns\": \"()\" }}\n\
                   ]\n\
                 }}\n\
               ]\n\
             }}\n\n\
             Generate the JSON IR starting with '{{ \"files\": ' now:",
            processed_spec
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), None).await?;
        let raw_text = resp.text.trim();
        
        if raw_text.is_empty() {
            tracing::error!("❌ [LLM EMPTY RESPONSE] Full Model Output was empty. Target Model: {}", self.agent.model);
            return Err(anyhow::anyhow!("LLM returned an empty response. Check if model '{}' is loaded and context limit is not exceeded.", self.agent.model));
        }

        let clean_json = match extract_json(raw_text) {
            Some(j) => {
                // v0.0.22: Auto-repair common small-model syntax errors
                let repaired = j.replace(",,", ",")
                                .replace("}}", "}")
                                .replace("True", "true")
                                .replace("False", "false");
                repaired
            },
            None => {
                tracing::error!("❌ [JSON EXTRACTION FAILED] Raw Text:\n---\n{}\n---", raw_text);
                return Err(anyhow::anyhow!("Failed to find JSON object in LLM response. See logs for raw output."));
            }
        };

        let ir: axon_core::ir::IR = serde_json::from_str(&clean_json)
            .map_err(|e| anyhow::anyhow!("JSON Parse Error: {} | Raw: {}", e, clean_json))?;
        Ok(ir)
    }

    pub async fn repair_ir(&self, ir: &axon_core::ir::IR, errors: &[axon_core::ir::ValidationError], event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<axon_core::ir::IR> {
        let system_prompt = format!(
            "### TASK: REPAIR JSON IR ###\n\
             STRICT RULE: RETURN ONLY THE FIXED JSON OBJECT. NO EXPLANATIONS.\n\n\
             Rules:\n\
             - Fix ONLY fields in error list\n\
             - DO NOT modify valid fields\n\
             - DO NOT add keys\n\n\
             Input IR:\n\
             {}\n\n\
             Errors Found:\n\
             {}\n\n\
             FINAL REMINDER: RETURN ONLY VALID JSON.",
            serde_json::to_string_pretty(ir).unwrap(),
            serde_json::to_string_pretty(errors).unwrap()
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), None).await?;
        let raw_text = resp.text.trim();

        if raw_text.is_empty() {
            return Err(anyhow::anyhow!("LLM returned an empty response during IR repair."));
        }

        let clean_json = extract_json(raw_text)
            .ok_or_else(|| anyhow::anyhow!("Failed to find JSON object in LLM response during repair: {}", raw_text))?;

        let ir: axon_core::ir::IR = serde_json::from_str(&clean_json)
            .map_err(|e| anyhow::anyhow!("JSON Parse Error during repair: {} | Raw: {}", e, clean_json))?;
        Ok(ir)
    }

    pub async fn generate_architecture_from_ir(&self, ir: &axon_core::ir::IR, _event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<String> {
        tracing::info!("🛠️ Generating deterministic architecture from IR...");
        
        let mut md = String::new();
        md.push_str("# Project Architecture (Deterministic IR-based)\n\n");
        md.push_str("## Overview\nThis architecture is automatically generated from the converged IR.\n\n");
        
        md.push_str("## Components\n");
        let mut components_json = serde_json::json!({ "components": [] });
        
        for file in &ir.files {
            md.push_str(&format!("### {}\n", file.name));
            md.push_str(&format!("- **Responsibility**: {}\n", file.responsibility));
            md.push_str("- **Functions**:\n");
            for func in &file.functions {
                md.push_str(&format!("  - `{}({})` -> `{}`\n", func.name, func.args.join(", "), func.returns));
            }
            md.push_str("\n");
            
            // Build the mandatory marker data
            components_json["components"].as_array_mut().unwrap().push(serde_json::json!({
                "name": file.name.split('.').next().unwrap_or("Module"),
                "file": file.name,
                "symbols": file.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                "type": if file.name.contains("main") { "entry" } else { "module" }
            }));
        }
        
        md.push_str("\n### AXON:SPEC:COMPONENTS\n");
        md.push_str("<!-- AXON:SPEC:COMPONENTS\n");
        md.push_str(&serde_json::to_string_pretty(&components_json).unwrap());
        md.push_str("\n-->\n");
        
        Ok(md)
    }

    pub async fn process_bootstrap_step1(&self, task: &Task, error_feedback: Option<String>, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
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
            if let Some(conditional) = forbidden["conditional"].as_array() {
                for rule in conditional {
                    contract.push(format!("- [FORBIDDEN]: '{}' is forbidden unless {}.", 
                        rule["token"].as_str().unwrap_or(""), 
                        rule["allow_if"].as_str().unwrap_or("")
                    ));
                }
            }
        }

        let contract_text = contract.join("\n");
        let hot_hints = {
            let mut cache = self.hot_cache.lock().unwrap();
            cache.decay(); // Apply decay per call
            cache.get_hints(5)
        };

        let system_prompt = if is_small_model {
            format!(
                "### TASK: Generate architecture.md for project: {}.\n\n\
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
                 {{ \"components\": [ {{ \"name\": \"Main\", \"file\": \"main.py\", \"symbols\": [\"main\"], \"type\": \"entry\" }} ] }}\n\
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
                "### OBJECTIVE ###\n\
                 Generate a COMPREHENSIVE and EXECUTABLE architecture.md for project: {}.\n\n\
                 {}\n\
                 {}\n\n\
                 ### 🏛️ ARCHITECTURE PROTOCOL (v0.0.21) ###\n\
                 YOU MUST follow this structure EXACTLY:\n\
                 ## Components\n\
                 - Detailed list of every file and its specific responsibility.\n\n\
                 ## Data Flow\n\
                 - Exhaustive step-by-step logic and data movement path.\n\n\
                 ## File Map\n\
                 - Direct mapping of modules to file paths.\n\n\
                 ## Interfaces\n\
                 - Precise function signatures, arguments, and return types.\n\n\
                 ### 🔒 HARD CONSTRAINTS (NON-NEGOTIABLE) ###\n\
                 1. REQUIRED: You MUST include 'main.py' and a '```mermaid' block.\n\
                 2. FORBIDDEN: NEVER use 'controller' (Use 'orchestrator'), 'manager', or 'hub'.\n\
                 3. LANGUAGE: Use {}.\n\
                 4. OUTPUT: ONLY markdown content. NO conversational preamble.\n\n\
                 ### 🗺️ REQUIRED MAPPING BLOCK (MANDATORY AT THE END) ###\n\
                 <!-- AXON:SPEC:COMPONENTS\n\
                 {{ \"components\": [ {{ \"name\": \"Name\", \"file\": \"main.py\", \"symbols\": [\"main\"], \"type\": \"entry\" }} ] }}\n\
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

            let resp = self.generate_with_retry(current_prompt, event_bus.as_ref(), Some(task.id.clone())).await;
            
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
            "### TASK ###\n\
             ROLE: CTO & CHIEF ARCHITECT.\n\
             DECOMPOSE THE FOLLOWING ARCHITECTURE INTO ATOMIC TASKS.\n\n\
             ### OUTPUT RULES ###\n\
             1. LANGUAGE: USE {}.\n\
             2. FORMAT: VALID JSON ARRAY OF OBJECTS ONLY.\n\
             3. OBJECT SCHEMA: {{ \"id\": \"unique_id\", \"title\": \"Descriptive Title\", \"description\": \"Detailed task description for a Junior agent\" }}\n\
             4. SCOPE: EXACTLY ONE TASK PER CONCRETE FILE (e.g., main.py, database.py).\n\
             5. ORCHESTRATION: You MUST ensure one task is explicitly created for the main execution orchestrator (main.py).\n\
             6. NO CONCEPTUAL TASKS: Do NOT create tasks for abstract concepts, folders, or non-executable code.\n\n\
             ### ARCHITECTURE GUIDE ###\n\
             {}",
            lang_name,
            architecture_content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), None).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: "bootstrap-extraction".to_string(),
            author_id: self.agent.id.clone(),
            content: resp.text,
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

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(proposal.thread_id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: proposal.thread_id.clone(),
            author_id: "SYSTEM_SUMMARY".to_string(),
            content: resp.text,
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
             --- 중요: 반드시 아래 지정된 언어로만 답변하십시오 (FORCE LANGUAGE) ---\n\
             언어: {}\n\n\
             주니어의 제안을 검토하고 승인 여부를 결정하십시오.\n\n\
             --- 태스크 ---\n\
             제목: {}\n\
             설명: {}\n\n\
             --- 주니어 제안 ---\n\
             {}\n\
             {}\n\n\
             --- 검토 규격 ---\n\
             1. 출력 규약 검증 (CRITICAL): 주니어의 제안이 유효한 JSON 배열 형식을 따르고 있는지 확인하십시오. 이 형식이 아니면 **무조건 REJECT** 하십시오.\n\
             2. 코드 및 의존성 검증: 코드가 완성된 상태인지, 실행 가능한지, 환각 라이브러리(SovereignProtocol 등)가 없는지 확인하십시오.\n\
             3. 생각(<analysis>) 과정은 생략하십시오.\n\
             4. 마지막에 반드시 'APPROVE' 또는 'REJECT'를 명시하십시오.\n\
             5. 반려(REJECT) 시에는 짧고 명확한 사유와 수정 힌트(FIX_HINT)를 한국어로 적으십시오.",
            self.agent.persona.name,
            lang_name,
            task.title,
            task.description,
            proposal.content,
            summary_content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: resp.text,
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
             2. 준수되었을 경우에만 'COMPLIANT'라고 답변하십시오.",
            self.agent.persona.name,
            lang_name,
            architecture_guide,
            task.title,
            review.content
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: resp.text,
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

/// v0.0.22: Robust JSON Extraction Helper
fn extract_json(raw: &str) -> Option<String> {
    let start = raw.find('{')?;
    let mut count = 0;
    let mut end = None;
    let bytes = raw.as_bytes();

    for i in start..bytes.len() {
        if bytes[i] == b'{' {
            count += 1;
        } else if bytes[i] == b'}' {
            count -= 1;
            if count == 0 {
                end = Some(i);
                break;
            }
        }
    }

    let end_idx = end?;
    Some(raw[start..=end_idx].to_string())
}

