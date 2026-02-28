use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AxonConfig {
    pub project_name: String,
    pub agents: AgentConfig,
    pub humor_level: f32, // 0.0 to 1.0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub juniors: u32,
    pub use_default_persona: bool,
}

impl Default for AxonConfig {
    fn default() -> Self {
        Self {
            project_name: "New Axon Project".to_string(),
            agents: AgentConfig {
                juniors: 2,
                use_default_persona: true,
            },
            humor_level: 0.5,
        }
    }
}

impl AxonConfig {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("axon_config.json") {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("axon_config.json", content)
    }
}
