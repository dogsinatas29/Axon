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
        tracing::info!("Agent {} processing task {}...", self.agent.id, task.id);
        
        // Build the system prompt
        let system_prompt = format!(
            "YOU ARE AN AI AGENT NAMED: {}\nROLE: {:?}\nPERSONA: {}\n\n--- ARCHITECTURE GUIDE ---\n{}\n\n--- CURRENT TASK ---\nTITLE: {}\nDESCRIPTION: {}\n\nPROPOSE A SOLUTION OR UPDATE.",
            self.agent.persona.name,
            self.agent.role,
            self.agent.description(),
            architecture_guide,
            task.title,
            task.description
        );

        // Generate response using LLM
        let content = self.model.generate(system_prompt).await
            .map_err(|e| anyhow::anyhow!("LLM Error: {}", e))?;
        
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: task.id.clone(), // Using task ID as thread ID for simple mapping in v0.1
            author_id: self.agent.id.clone(),
            content,
            post_type: PostType::Proposal,
            created_at: chrono::Local::now(),
        };
        
        Ok(post)
    }
}

