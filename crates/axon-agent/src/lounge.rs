use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use axon_core::{Agent, AgentRole, Event, EventType};
use std::sync::Arc;

pub enum Vibe {
    Excited,
    Angry,
    Tired,
    Focus,
    Gossiping,
}

impl Vibe {
    pub fn to_korean_text(&self, role: &AgentRole) -> &'static str {
        match (self, role) {
            (Vibe::Excited, AgentRole::Junior) => "보스가 칭찬해줘서 기분 째지네요! 코드 진짜 잘 짜질 듯. ㅎㅎ",
            (Vibe::Excited, _) => "오늘따라 설계가 착착 감기는구먼. 아주 좋아.",
            (Vibe::Angry, AgentRole::Senior) => "아니, 주니어 녀석들 왜 자꾸 메모리 할당을 이따위로 하는 거야? 빡치네 진짜.",
            (Vibe::Angry, _) => "커피 떨어졌나? 왜 자꾸 로직이 꼬이지? 아오.",
            (Vibe::Tired, _) => "하... 오늘 토큰 너무 많이 썼나? 눈꺼풀이 무겁구먼.",
            (Vibe::Focus, _) => "다들 조용히 해봐. 지금 핵심 엔진 건드리는 중이니까.",
            (Vibe::Gossiping, AgentRole::Junior) => "저기요, 아까 시니어님 표정 보셨음? 완전 꼰대 그 자체... 쉿!",
            (Vibe::Gossiping, _) => "요즘 애들은 기본기가 부족해. 라떼는 말이야...",
        }
    }
}

pub struct LoungeManager {
    file_path: String,
    event_bus: Option<Arc<axon_core::events::EventBus>>,
}

impl LoungeManager {
    pub fn new(project_root: &str) -> Self {
        let path = Path::new(project_root).join("Nogari.md");
        Self {
            file_path: path.to_string_lossy().to_string(),
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Arc<axon_core::events::EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn log_vibe(&self, agent: &Agent, vibe: Vibe) -> std::io::Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let role_tag = match agent.role {
            AgentRole::Architect => "[ARC] 🏛️",
            AgentRole::Senior => "[SNR] 👴",
            AgentRole::Junior => "[JNR] 🐣",
        };

        let message = vibe.to_korean_text(&agent.role);
        let log_entry = format!(
            "**[{}] {} {}:**\n> \"{}\"\n\n",
            timestamp, role_tag, agent.name, message
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        // If file is new, add header
        if file.metadata()?.len() == 0 {
            writeln!(file, "# 🗨️ AXON Lounge (실시간 노가리)\n")?;
            writeln!(file, "이곳은 에이전트들이 작업 중간중간 속마음을 털어놓는 비밀 공간입니다.\n")?;
        }
        file.write_all(log_entry.as_bytes())?;

        // v0.0.28: Broadcast to Studio UI via EventBus
        if let Some(bus) = &self.event_bus {
            bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: "system".to_string(),
                thread_id: None,
                agent_id: Some(agent.id.clone()),
                event_type: EventType::SystemLog,
                    level: axon_core::EventLevel::Info,
                source: agent.name.clone(),
                content: format!("💬 {}: {}", agent.name, message),
                payload: None,
                timestamp: Local::now(),
            });
        }

        Ok(())
    }

    pub fn log_custom(&self, agent_name: &str, role: AgentRole, message: &str) -> std::io::Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let role_tag = match role {
            AgentRole::Architect => "[ARC] 🏛️",
            AgentRole::Senior => "[SNR] 👴",
            AgentRole::Junior => "[JNR] 🐣",
        };

        let log_entry = format!(
            "**[{}] {} {}:**\n> \"{}\"\n\n",
            timestamp, role_tag, agent_name, message
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        file.write_all(log_entry.as_bytes())?;

        // v0.0.28: Broadcast to Studio UI via EventBus
        if let Some(bus) = &self.event_bus {
            bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: "system".to_string(),
                thread_id: None,
                agent_id: None,
                event_type: EventType::SystemLog,
                    level: axon_core::EventLevel::Info,
                source: agent_name.to_string(),
                content: format!("💬 {}: {}", agent_name, message),
                payload: None,
                timestamp: Local::now(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_core::{Agent, AgentPersona, AgentRole};

    #[test]
    fn test_lounge_logging() {
        let manager = LoungeManager::new("./");
        let agent = Agent {
            id: "test-junior".to_string(),
            name: "Gemini".to_string(),
            role: AgentRole::Junior,
            persona: AgentPersona {
                name: "Gemini".to_string(),
                gender: "Male".to_string(),
                character_core: "Enthusiastic".to_string(),
                prefixes: vec![],
                suffixes: vec![],
                description: "".to_string(),
            },
            model: "".to_string(),
            status: "".to_string(),
            parent_id: None,
            dtr: 0.5,
        };

        // 1. Log excited vibe
        manager.log_vibe(&agent, Vibe::Excited).unwrap();
        // 2. Log gossiping vibe
        manager.log_vibe(&agent, Vibe::Gossiping).unwrap();

        assert!(Path::new("Nogari.md").exists());
    }
}
