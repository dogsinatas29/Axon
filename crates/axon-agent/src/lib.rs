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

use axon_core::{Agent, Post, PostType, AgentRole, Task};
use axon_model::ModelDriver;
use std::sync::Arc;

pub struct AgentRuntime {
    pub agent: Agent,
    pub model: Arc<dyn ModelDriver + Send + Sync>,
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
        Self { agent, model: model_driver }
    }

    pub async fn process_task(&self, task: &Task, architecture_guide: &str) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Junior) processing task {}...", self.agent.id, task.id);
        
        let system_prompt = format!(
            "YOU ARE AN AI JUNIOR AGENT NAMED: {}\n\
             PERSONA: {}\n\n\
             --- ARCHITECTURE GUIDE ---\n\
             {}\n\n\
             --- CURRENT TASK ---\n\
             TITLE: {}\n\
             DESCRIPTION: {}\n\n\
             --- STRICT OUTPUT CONSTRAINTS ---\n\
             1. DO NOT SUMMARIZE. DO NOT EXPLAIN. NO RISK ANALYSIS.\n\
             2. PROVIDE ONLY THE FOLLOWING FOUR FIELDS IN THE EXACT ORDER:\n\
                - task_id: {}\n\
                - changed_files: [file_a, file_b]\n\
                - diff: [standard diff format]\n\
                - full_code: [entire content of modified files]\n\n\
             FAILURE TO ADHERE TO THESE CONSTRAINTS WILL RESULT IN REJECTION.",
            self.agent.persona.name,
            self.agent.description(),
            architecture_guide,
            task.title,
            task.description,
            task.id
        );

        let content = self.model.generate(system_prompt).await
            .map_err(|e| anyhow::anyhow!("LLM Error: {}", e))?;
        
        let full_code = if self.agent.role == AgentRole::Architect {
            Some(content.clone())
        } else {
            // Very simple extraction for Junior (look for a code block if possible, or assume entire content)
            Some(content.clone())
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

    pub async fn generate_system_summary(&self, proposal: &Post) -> anyhow::Result<Post> {
        tracing::info!("System generating summary for proposal {}...", proposal.id);
        
        let system_prompt = format!(
            "YOU ARE THE AXON SYSTEM SUMMARY LAYER.\n\n\
             --- JUNIOR PROPOSAL CONTENT ---\n\
             {}\n\n\
             --- INSTRUCTION ---\n\
             ANALYZE THE PROPOSAL ABOVE. PROVIDE A NEUTRAL TECHNICAL SUMMARY.\n\
             1. LIST CHANGED FILES.\n\
             2. SUMMARIZE CORE LOGIC CHANGES IN 2-3 BULLET POINTS.\n\
             3. DO NOT PROVIDE OPINIONS, FEEDBACK, OR RISK ANALYSIS.\n\
             4. BE CONCISE.",
            proposal.content
        );

        let content = self.model.generate(system_prompt).await
            .map_err(|e| anyhow::anyhow!("LLM Error: {}", e))?;
        
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

    pub async fn review_proposal(&self, task: &Task, proposal: &Post, summary: Option<&Post>) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Senior) reviewing proposal for task {}...", self.agent.id, task.id);
        
        let summary_content = match summary {
            Some(s) => format!("\n--- SYSTEM SUMMARY ---\n{}\n", s.content),
            None => "".to_string(),
        };

        let system_prompt = format!(
            "YOU ARE AN AI SENIOR AGENT NAMED: {}\n\
             PERSONA: {}\n\n\
             --- TASK ---\n\
             TITLE: {}\n\
             DESCRIPTION: {}\n\n\
             --- PROPOSAL BY JUNIOR ---\n\
             {}\n\
             {}\n\n\
             --- DECISION PROTOCOL ---\n\
             1. IF THE PROPOSAL IS CORRECT AND MEETS REQUIREMENTS: START WITH 'APPROVE'.\n\
             2. IF NOT: START WITH 'REJECT'.\n\
                - REASON: [one line only explaining why]\n\
                - FIX_HINT: [one line only providing the solution direction]\n\
             3. NEVER EXPLAIN YOUR REASONING OR PROVIDE EXTRA FEEDBACK.\n\
             4. BE RIGOROUS AND CRITICAL.",
            self.agent.persona.name,
            self.agent.description(),
            task.title,
            task.description,
            proposal.content,
            summary_content
        );

        let content = self.model.generate(system_prompt).await
            .map_err(|e| anyhow::anyhow!("LLM Error: {}", e))?;
        
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

    pub async fn validate_architecture(&self, task: &Task, review: &Post, arch_guide: &str) -> anyhow::Result<Post> {
        tracing::info!("Agent {} (Architect) validating architecture for task {}...", self.agent.id, task.id);
        
        let system_prompt = format!(
            "YOU ARE THE CHIEF ARCHITECT NAMED: {}\nPERSONA: {}\n\n--- ARCHITECTURE GUIDE ---\n{}\n\n--- TASK ---\n{}\n\n--- SENIOR REVIEW ---\n{}\n\nVALIDATE IF THIS COMPLIES WITH SYSTEM PRINCIPLES. IF YES, SAY 'COMPLIANT'.",
            self.agent.persona.name,
            self.agent.description(),
            arch_guide,
            task.title,
            review.content
        );

        let content = self.model.generate(system_prompt).await
            .map_err(|e| anyhow::anyhow!("LLM Error: {}", e))?;
        
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

