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
use axon_model::ModelDriver;
use std::sync::Arc;

pub struct AgentRuntime {
    pub agent: Agent,
    pub model: Arc<dyn ModelDriver + Send + Sync>,
    pub locale: String, // v0.0.15: System language preference
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
        }
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    async fn generate_with_retry(&self, prompt: String, event_bus: Option<&Arc<axon_core::events::EventBus>>, thread_id: Option<String>) -> anyhow::Result<String> {
        let mut retries = 5;
        loop {
            match self.model.generate(prompt.clone()).await {
                Ok(text) => return Ok(text),
                Err(e) => {
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
            }
        }
    }

    pub async fn process_task(&self, task: &Task, architecture_guide: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Junior) processing task {}...", self.agent.id, task.id);
        
        let system_prompt = format!(
            "YOU ARE AN AI JUNIOR AGENT NAMED: {}\n\
             PERSONA: {}\n\n\
             --- LANGUAGE ENFORCEMENT ---\n\
             YOU MUST COMMUNICATE AND GENERATE ALL CONTENT (TITLE, DESCRIPTION, REASONING) IN THE FOLLOWING LOCALE: {}.\n\n\
             --- STEP 1: REASONING (COT) ---\n\
             Before implementing, you MUST perform a deep logical analysis in <thought> tags. Break down the task, identify potential edge cases, and ensure alignment with the SSOT in ARCHITECTURE GUIDE.\n\n\
             --- STEP 2: IMPLEMENTATION ---\n\
             Provide the complete implementation following the architecture guide.\n\n\
             --- ARCHITECTURE GUIDE ---\n\
             {}\n\n\
             --- CURRENT TASK ---\n\
             TITLE: {}\n\
             DESCRIPTION: {}\n\n\
             --- OUTPUT PROTOCOL ---\n\
             1. You MUST include your reasoning in <thought> tags FIRST.\n\
             2. Follow with the actual implementation details:\n\
                - task_id: {}\n\
                - changed_files: [list of files]\n\
                - diff: [informative diff]\n\
                - full_code: [complete source code for files]\n\
             3. DO NOT suppress your reasoning. High-quality thought process is mandatory.",
            self.agent.persona.name,
            self.agent.description(),
            self.locale,
            architecture_guide,
            task.title,
            task.description,
            task.id
        );

        let content = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        let full_code = {
            // Strip reasoning tags to get clean content
            let mut clean = content.clone();
            for tag in ["thought", "analysis", "reasoning", "evaluation"] {
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
            content,
            full_code,
            post_type: PostType::Proposal,
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

        let content = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(proposal.thread_id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: proposal.thread_id.clone(),
            author_id: "SYSTEM_SUMMARY".to_string(),
            content,
            full_code: None,
            post_type: PostType::System,
            created_at: chrono::Local::now(),
        })
    }

    pub async fn review_proposal(&self, task: &Task, proposal: &Post, summary: Option<&Post>, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Senior) reviewing proposal for task {}...", self.agent.id, task.id);
        
        let summary_content = match summary {
            Some(s) => format!("\n--- SYSTEM SUMMARY ---\n{}\n", s.content),
            None => "".to_string(),
        };

        let system_prompt = format!(
            "YOU ARE AN AI SENIOR AGENT NAMED: {}\n\
             PERSONA: {}\n\n\
             --- LANGUAGE ENFORCEMENT ---\n\
             YOU MUST COMMUNICATE AND GENERATE THE REVIEW CONTENT (ANALYSIS, DECISION, FIX_HINT) IN THE FOLLOWING LOCALE: {}.\n\n\
             --- STEP 1: MULTI-PERSPECTIVE ANALYSIS (TOT) ---\n\
             SYSTEMATICALLY EVALUATE the junior's proposal through a 'Tree of Thoughts' in <analysis> tags. \n\
             You MUST consider at least three perspectives: Performance, Security, and SSOT/Maintainability.\n\n\
             --- STEP 2: CRITICAL DECISION ---\n\
             Based on your multi-perspective evaluation, provide a final decision.\n\n\
             --- TASK ---\n\
             TITLE: {}\n\
             DESCRIPTION: {}\n\n\
             --- PROPOSAL BY JUNIOR ---\n\
             {}\n\
             {}\n\n\
             --- FINAL REVIEW PROTOCOL ---\n\
             1. START with your detailed analysis in <analysis> tags.\n\
             2. CONCLUDE with either 'APPROVE' or 'REJECT'.\n\
             3. If REJECTED, provide a detailed REASON and FIX_HINT.\n\
             4. Your reasoning is the most valuable part of this review.",
            self.agent.persona.name,
            self.agent.description(),
            self.locale,
            task.title,
            task.description,
            proposal.content,
            summary_content
        );

        let content = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content,
            full_code: None,
            post_type: PostType::Review,
            created_at: chrono::Local::now(),
        })
    }

    pub async fn validate_architecture(&self, task: &Task, review: &Post, architecture_guide: &str, event_bus: Option<Arc<axon_core::events::EventBus>>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Architect) validating architecture for task {}...", self.agent.id, task.id);
        
        let system_prompt = format!(
            "YOU ARE THE CHIEF ARCHITECT NAMED: {}\nPERSONA: {}\n\n\
             --- LANGUAGE ENFORCEMENT ---\n\
             YOU MUST COMMUNICATE AND GENERATE THE VALIDATION CONTENT (REASONING, STATUS) IN THE FOLLOWING LOCALE: {}.\n\n\
             --- STEP 1: GLOBAL CROSS-VALIDATION (COT+TOT) ---\n\
             As the Chief Architect, you MUST reason about the long-term system impact and verify Sovereign Protocol compliance in <reasoning> tags.\n\
             Analyze both the technical implementation (Junior) and the critical feedback (Senior).\n\n\
             --- ARCHITECTURE GUIDE ---\n{}\n\n\
             --- TASK ---\n{}\n\n\
             --- SENIOR REVIEW ---\n{}\n\n\
             --- VALIDATION OUTPUT ---\n\
             1. Provide your in-depth architectural reasoning in <reasoning> tags.\n\
             2. Clearly state 'COMPLIANT' only if the work meets all SSOT and Sovereign Protocol standards.",
            self.agent.persona.name,
            self.agent.description(),
            self.locale,
            architecture_guide,
            task.title,
            review.content
        );

        let content = self.generate_with_retry(system_prompt, event_bus.as_ref(), Some(task.id.clone())).await?;
        
        Ok(Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(),
            author_id: self.agent.id.clone(),
            content,
            full_code: None,
            post_type: PostType::System,
            created_at: chrono::Local::now(),
        })
    }
}

