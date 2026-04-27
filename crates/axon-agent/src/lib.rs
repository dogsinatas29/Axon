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
        // ... (Junior logic remains same)
        tracing::info!("Agent {} (Junior) processing task {}...", self.agent.id, task.id);
        
        let lang_name = match self.locale.as_str() {
            "ko_KR" => "한국어 (Korean)",
            "ja_JP" => "日本語 (Japanese)",
            _ => "English",
        };

        let system_prompt = format!(
            "### SYSTEM: AI JUNIOR AGENT: {} ###\n\
             --- 중요: 반드시 아래 지정된 언어로만 답변하십시오 (FORCE LANGUAGE) ---\n\
             언어: {}\n\n\
             주어진 아키텍처 가이드를 준수하여 아래 태스크를 구현하십시오.\n\n\
             --- 아키텍처 가이드 ---\n\
             {}\n\n\
             --- 현재 태스크 ---\n\
             제목: {}\n\
             설명: {}\n\n\
             --- 출력 규격 ---\n\
             1. 서론이나 생각(<thought>)은 생략하고 즉시 구현 내용을 출력하십시오.\n\
             2. 다음 형식을 반드시 포함하십시오:\n\
                - task_id: {}\n\
                - changed_files: [수정된 파일 목록]\n\
                - diff: [주요 변경 사항 요약]\n\
                - full_code: [파일의 전체 소스 코드]",
            self.agent.persona.name,
            lang_name,
            architecture_guide,
            task.title,
            task.description,
            task.id
        );

        let resp = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
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
        tracing::info!("Agent {} (Architect) Stage 1: Designing Architecture...", self.agent.id);
        
        let system_prompt = format!(
            "### TASK ###\n\
             DESIGN MASTER ARCHITECTURE (Hub->Cluster->Node) FOR PROJECT: {}.\n\n\
             ### PROTOCOL (Sovereign v0.2.21+) ###\n\
             1. CORE: Define a central 'Hub' for SSOT.\n\
             2. MODULES: Group logic into 'Clusters'.\n\
             3. ATOMS: Define 'Nodes' for specific functions.\n\n\
             ### RULES ###\n\
             - LANGUAGE: {}.\n\
             - OUTPUT: ONLY markdown (architecture.md content). NO CHAT.\n\n\
             ### SPECIFICATION ###\n\
             {}",
            self.agent.persona.name,
            self.locale,
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

    pub async fn process_bootstrap_step2(&self, architecture: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Architect) Stage 2: Extracting Tasks...", self.agent.id);
        
        let system_prompt = format!(
            "### INSTRUCTION ###\n\
             YOU ARE THE CHIEF TECHNOLOGY OFFICER (CTO).\n\
             ANALYZE THE MASTER ARCHITECTURE AND DECOMPOSE IT INTO A COMPREHENSIVE SET OF ATOMIC IMPLEMENTATION TASKS.\n\n\
             ### STRATEGY ###\n\
             1. GRANULARITY: Each task must be small enough for a single developer to implement in one sprint. DO NOT group multiple systems into one task.\n\
             2. PARALLELISM: Identify independent modules that can be built simultaneously by different workers.\n\
             3. COVERAGE: Ensure 100% of the architecture nodes are covered by the generated tasks.\n\n\
             ### RULES ###\n\
             1. LOCALE: USE {}.\n\
             2. FORMAT: OUTPUT ONLY RAW JSON ARRAY. NO MARKDOWN BLOCKS. NO PREAMBLE.\n\
             3. SCHEMA: [{{ \"title\": \"...\", \"description\": \"...\" }}, ...]\n\n\
             ### ARCHITECTURE CONTENT ###\n\
             {}",
            self.locale,
            architecture
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
        tracing::info!("System generating summary for proposal {}...", proposal.id);
        
        let system_prompt = format!(
            "YOU ARE THE AXON SYSTEM SUMMARY LAYER.\n\n\
             --- LANGUAGE ENFORCEMENT ---\n\
             YOU MUST GENERATE THE SUMMARY IN THE FOLLOWING LOCALE: {}.\n\n\
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
        );

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
             1. 생각(<analysis>) 과정은 생략하십시오.\n\
             2. 마지막에 반드시 'APPROVE' 또는 'REJECT'를 명시하십시오.\n\
             3. 반려(REJECT) 시에는 짧고 명확한 사유와 수정 힌트(FIX_HINT)를 한국어로 적으십시오.",
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

