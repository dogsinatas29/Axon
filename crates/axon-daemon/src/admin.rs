use std::sync::Arc;
use axon_core::{Post, PostType, AgentRole};
use axon_storage::Storage;
use chrono::Local;
use tracing::info;

pub enum InterventionType {
    Formal,      // "BOSS" name shown
    Anonymous,   // Name hidden or spoofed
    Instigate,   // Inciting conflict between agents
}

pub struct AdminSystem {
    storage: Arc<Storage>,
}

impl AdminSystem {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Boss intervenes in a thread
    pub async fn intervene(
        &self, 
        thread_id: &str, 
        content: &str, 
        intervention: InterventionType
    ) -> anyhow::Result<()> {
        let author_id = match intervention {
            InterventionType::Formal => "BOSS".to_string(),
            InterventionType::Anonymous => "???".to_string(),
            InterventionType::Instigate => "Internal_System_Bot".to_string(),
        };

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread_id.to_string(),
            author_id,
            content: content.to_string(),
            full_code: None,
            post_type: PostType::Instruction,
            created_at: Local::now(),
        };

        self.storage.save_post(&post)?;
        
        match intervention {
            InterventionType::Formal => info!("👑 BOSS has formally intervened in thread: {}", thread_id),
            InterventionType::Anonymous => info!("👤 Anonymous intervention recorded in thread: {}", thread_id),
            InterventionType::Instigate => info!("🔥 System instigation triggered in thread: {}", thread_id),
        }

        Ok(())
    }

    /// Boss manually locks a thread/architecture section
    pub fn force_lock(&self, thread_id: &str) -> anyhow::Result<()> {
        // Logic to finalize thread and mark as [✅ Locked] in architecture.md
        // (Delegates to Daemon's lock_in_architecture in the real implementation)
        info!("🔒 BOSS forced LOCK for thread: {}", thread_id);
        Ok(())
    }
}
