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
use std::sync::Arc;

pub struct AgentRuntime {
    pub agent: Agent,
    pub model: Arc<dyn ModelDriver + Send + Sync>,
    pub locale: String, // v0.0.15: System language preference
    pub timeout: std::time::Duration,
    pub throttler: Option<Arc<tokio::sync::Semaphore>>,
}

impl AgentRuntime {
    pub fn new(id: String, role: AgentRole, model_driver: Arc<dyn ModelDriver + Send + Sync>) -> Self {
        let persona = persona::AffixSystem::generate_random(role.clone());
        let agent = Agent {
            id: id.clone(),
            name: persona.name.clone(),
            role,
            persona,
            model: "gemini-1.5-pro".to_string(), // Default for now
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
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = std::time::Duration::from_secs(seconds);
        self
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    async fn generate_with_retry(&self, prompt: String, event_bus: Option<&Arc<axon_core::events::EventBus>>, thread_id: Option<String>) -> anyhow::Result<ModelResponse> {
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
                Ok(Ok(resp)) => return Ok(resp),
                Ok(Err(e)) => {
                    let err_str = e.to_string();
                    if err_str.starts_with("QUOTA_WAIT:") {
                        if retries > 0 {
                            let wait_secs: f64 = err_str.strip_prefix("QUOTA_WAIT:").unwrap_or("60.0").parse().unwrap_or(60.0);
                            tracing::warn!("Agent {} waiting for {:.1}s due to quota...", self.agent.id, wait_secs);
                            
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

    pub async fn process_task(&self, task: &Task, architecture_guide: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
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
        
        let system_prompt = if self.locale.as_str() == "ko_KR" {
            format!(
                "### SYSTEM: AI JUNIOR AGENT: {} ###\n\
                 --- 중요: 반드시 아래 지정된 언어로만 답변하십시오 (FORCE LANGUAGE) ---\n\
                 언어: {}\n\n\
                 아키텍처 가이드를 기반으로 다음 태스크를 구현하십시오.\n\n\
                 --- 아키텍처 가이드 ---\n\
                 {}\n\n\
                 --- 현재 태스크 ---\n\
                 제목: {}\n\
                 설명: {}\n\n\
                 --- 출력 규격 ---\n\
                 1. 서론이나 생각 과정을 적지 마십시오. 즉시 코드를 출력하십시오.\n\
                 2. 반드시 다음 항목을 포함하여 작성하십시오:\n\
                    - task_id: {}\n\
                    - changed_files: [수정된 파일 목록]\n\
                    - diff: [변경 사항 요약]\n\
                    - full_code: [전체 코드 블록]",
                self.agent.persona.name, lang_name, architecture_guide, task.title, task.description, task.id
            )
        } else {
            format!(
                "### SYSTEM: AI JUNIOR AGENT: {} ###\n\
                 --- IMPORTANT: FORCE LANGUAGE ---\n\
                 LANGUAGE: {}\n\n\
                 Implement the following task based on the architecture guide.\n\n\
                 --- ARCHITECTURE GUIDE ---\n\
                 {}\n\n\
                 --- CURRENT TASK ---\n\
                 TITLE: {}\n\
                 DESCRIPTION: {}\n\n\
                 --- OUTPUT SPEC ---\n\
                 1. STRICT RULE: NO PREAMBLE. NO EXPLANATION TEXT. ONLY CODE.\n\
                 2. EXACT FORMAT REQUIRED:\n\n\
                 [OUTPUT]\n\n\
                 FILE: filename.ext\n\
                 ```language\n\
                 # full executable code here\n\
                 ```\n\
                 END_FILE",
                self.agent.persona.name, lang_name, architecture_guide, task.title, task.description
            )
        };

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        // v0.0.22: CRITICAL RESOURCE BOTTLENECK PROTECTION
        // If Ollama returns empty content due to memory/GPU timeout, DO NOT treat it as success.
        if resp.text.trim().is_empty() || !resp.text.contains("```") {
            tracing::error!("❌ [RESOURCE ERROR]: Junior produced no code blocks. System may be overloaded.");
            return Err(anyhow::anyhow!("Ollama produced empty response or missing code blocks. Check system resources."));
        }

        let full_code = {
            // Strip reasoning tags to get clean content
            let mut clean = resp.text.clone();
            for tag in ["thought", "analysis", "reasoning", "evaluation", "thought"] {
                let start_tag = format!("<{}>", tag);
                let end_tag = format!("</{}>", tag);
                while let (Some(s), Some(e)) = (clean.find(&start_tag), clean.find(&end_tag)) {
                    clean.replace_range(s..e + end_tag.len(), "");
                }
            }
            Some(clean.trim().to_string())
        };

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

    pub async fn process_bootstrap_step1(&self, task: &Task, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let log_msg = match self.locale.as_str() {
            "ko_KR" => format!("요원 {} (아키텍트) 1단계: 아키텍처 설계 중...", self.agent.id),
            "ja_JP" => format!("エージェント {} (アー키텍트) ステージ1: アー키텍처 설계 중...", self.agent.id),
            _ => format!("Agent {} (Architect) Stage 1: Designing Architecture...", self.agent.id),
        };
        tracing::info!("{}", log_msg);
        
        let system_prompt = format!(
            "### TASK ###\n\
             CREATE AN EXECUTABLE IMPLEMENTATION SPECIFICATION FOR PROJECT: {}.\n\n\
             ### ARCHITECTURE PROTOCOL (v0.0.19 Executable) ###\n\
             DO NOT write a conceptual essay. NO abstract 'Hub/Cluster/Node' jargon.\n\
             YOU MUST provide a concrete file-level design.\n\
             1. PROJECT STRUCTURE: List exact filenames (e.g. main.py, database.py).\n\
             2. MODULE RESPONSIBILITIES: Define exact functions, interfaces, and roles for each file.\n\
             3. EXECUTION FLOW: Map the concrete execution path.\n\
             4. DATA MODEL: Define concrete data schemas.\n\
             5. DEPENDENCIES: List explicitly required libraries.\n\n\
             ### RULES ###\n\
             - LANGUAGE: USE {}.\n\
             - OUTPUT: ONLY markdown (architecture.md content). NO CHAT.\n\n\
             ### SPECIFICATION ###\n\
             {}",
            self.agent.persona.name,
            lang_name,
            task.description
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content: resp.text,
            full_code: None,
            post_type: PostType::Instruction,
            metrics: Some(axon_core::RuntimeMetrics {
                total_duration: resp.total_duration,
                eval_count: resp.eval_count,
                eval_duration: resp.eval_duration,
            }),
            created_at: chrono::Local::now(),
        })
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
        let lang_name = match self.locale.as_str() {
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
             1. 출력 규약 검증 (CRITICAL): 주니어의 제안에 반드시 `[OUTPUT]` 마크업과 `FILE: 파일명` 및 `END_FILE` 구조가 포함되어 있어야 합니다. 이 구조가 없거나 설명 텍스트가 섞여 있다면 **무조건 REJECT** 하십시오.\n\
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

