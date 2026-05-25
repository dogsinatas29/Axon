use chrono::{Local, TimeZone};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use axon_core::{Agent, AgentRole, Event, EventType};
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NogariPersona {
    pub agent_id: String,
    pub nickname: String,
    pub conversation_style: String,
    pub custom_emojis: Vec<String>,
}


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
    lounge_dir: PathBuf,
    event_bus: Option<Arc<axon_core::events::EventBus>>,
}

impl LoungeManager {
    pub fn new(project_root: &str) -> Self {
        let lounge_dir = Path::new(project_root).join("lounge");
        let _ = fs::create_dir_all(&lounge_dir);
        let _ = fs::create_dir_all(lounge_dir.join("reflections"));
        Self {
            lounge_dir,
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
            "[{}] {} {}: {}\n",
            timestamp, role_tag, agent.name, message
        );

        let file_name = format!("agent_{}.log", agent.id);
        let file_path = self.lounge_dir.join(file_name);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

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
            "[{}] {} {}: {}\n",
            timestamp, role_tag, agent_name, message
        );

        let file_name = format!("agent_{}.log", agent_name);
        let file_path = self.lounge_dir.join(file_name);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

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

    pub fn log_task_thought(&self, task_id: &str, agent_name: &str, role: AgentRole, thought: &str) -> std::io::Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let role_tag = match role {
            AgentRole::Architect => "[ARC] 🏛️",
            AgentRole::Senior => "[SNR] 👴",
            AgentRole::Junior => "[JNR] 🐣",
        };

        let log_entry = format!(
            "### [{}] {} {}'s Reflection:\n{}\n\n",
            timestamp, role_tag, agent_name, thought
        );

        let file_name = format!("task_{}.md", task_id);
        let file_path = self.lounge_dir.join(file_name);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        if file.metadata()?.len() == 0 {
            writeln!(file, "# 📝 Task {} - Agent Nogari & Reflections\n", task_id)?;
        }
        file.write_all(log_entry.as_bytes())?;

        // Broadcast to Studio UI via EventBus
        if let Some(bus) = &self.event_bus {
            bus.publish(Event {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: "system".to_string(),
                thread_id: Some(task_id.to_string()),
                agent_id: None,
                event_type: EventType::SystemLog,
                level: axon_core::EventLevel::Info,
                source: agent_name.to_string(),
                content: format!("💬 [Thought] {}: {}", agent_name, thought),
                payload: None,
                timestamp: Local::now(),
            });
        }

        Ok(())
    }

    pub fn log_reflection(&self, agent_name: &str, role: AgentRole, reflection: &str) -> std::io::Result<()> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let file_name = format!("{}_{}.md", timestamp, agent_name);
        let file_path = self.lounge_dir.join("reflections").join(file_name);

        let role_tag = match role {
            AgentRole::Architect => "[ARC] 🏛️",
            AgentRole::Senior => "[SNR] 👴",
            AgentRole::Junior => "[JNR] 🐣",
        };

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_path)?;

        let content = format!(
            "# Agent Reflection - {}\nRole: {}\nTime: {}\n\n## Content\n{}\n",
            agent_name, role_tag, Local::now().to_rfc3339(), reflection
        );
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn read_all_lounge_posts(&self) -> std::io::Result<Vec<axon_core::Post>> {
        let mut posts = Vec::new();

        // 1. Read agent log files (agent_*.log)
        if let Ok(entries) = fs::read_dir(&self.lounge_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name.starts_with("agent_") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            for line in content.lines() {
                                if line.starts_with('[') && line.contains("] ") {
                                    if let Some(close_bracket_idx) = line.find(']') {
                                        let timestamp_str = &line[1..close_bracket_idx];
                                        let rest = &line[close_bracket_idx + 2..];
                                        let parts: Vec<&str> = rest.splitn(2, ':').collect();
                                        if parts.len() == 2 {
                                            let author_info = parts[0].trim();
                                            let message = parts[1].trim();

                                            let author_name = if let Some(last_space_idx) = author_info.rfind(' ') {
                                                author_info[last_space_idx + 1..].to_string()
                                            } else {
                                                author_info.to_string()
                                            };

                                            let created_at = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                                                .map(|dt| chrono::Local.from_local_datetime(&dt).unwrap())
                                                .unwrap_or_else(|_| chrono::Local::now());

                                            posts.push(axon_core::Post {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                thread_id: "lounge".to_string(),
                                                author_id: author_name,
                                                content: message.to_string(),
                                                thought: None,
                                                full_code: None,
                                                post_type: axon_core::PostType::Nogari,
                                                metrics: None,
                                                created_at,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Read task md files (task_*.md)
        if let Ok(entries) = fs::read_dir(&self.lounge_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name.starts_with("task_") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let mut lines = content.lines();
                            let mut current_post: Option<axon_core::Post> = None;
                            let mut current_body = String::new();

                            while let Some(line) = lines.next() {
                                if line.starts_with("### [") {
                                    if let Some(post) = current_post.take() {
                                        let mut finalized_post = post;
                                        finalized_post.content = current_body.trim().to_string();
                                        posts.push(finalized_post);
                                        current_body.clear();
                                    }

                                    if let Some(end_time_idx) = line.find(']') {
                                        let timestamp_str = &line[5..end_time_idx];
                                        let rest = &line[end_time_idx + 2..];
                                        let author_name = if let Some(reflection_idx) = rest.find("'s Reflection:") {
                                            let author_info = &rest[..reflection_idx];
                                            if let Some(last_space_idx) = author_info.rfind(' ') {
                                                author_info[last_space_idx + 1..].to_string()
                                            } else {
                                                author_info.to_string()
                                            }
                                        } else {
                                            "Agent".to_string()
                                        };

                                        let created_at = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                                            .map(|dt| chrono::Local.from_local_datetime(&dt).unwrap())
                                            .unwrap_or_else(|_| chrono::Local::now());

                                        current_post = Some(axon_core::Post {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            thread_id: "lounge".to_string(),
                                            author_id: author_name,
                                            content: String::new(),
                                            thought: None,
                                            full_code: None,
                                            post_type: axon_core::PostType::Nogari,
                                            metrics: None,
                                            created_at,
                                        });
                                    }
                                } else if current_post.is_some() {
                                    current_body.push_str(line);
                                    current_body.push('\n');
                                }
                            }
                            if let Some(post) = current_post.take() {
                                let mut finalized_post = post;
                                finalized_post.content = current_body.trim().to_string();
                                posts.push(finalized_post);
                            }
                        }
                    }
                }
            }
        }

        posts.sort_by_key(|p| p.created_at);
        Ok(posts)
    }

    pub fn compile_to_nogari_md(&self) -> std::io::Result<()> {
        let posts = self.read_all_lounge_posts().unwrap_or_default();
        let nogari_path = self.lounge_dir.parent().unwrap_or(&self.lounge_dir).join("Nogari.md");
        
        let mut content = String::new();
        content.push_str("# 🍻 AXON Lounge - Nogari History (Decoupled System)\n");
        content.push_str("*본 파일은 에이전트들의 잡담 페르소나에 의해 백그라운드로 안전하게 생성되었습니다.*\n\n---\n\n");
        
        for post in posts {
            let role_emoji = if post.author_id.contains("Senior") || post.author_id.contains("Claude") {
                "👴"
            } else if post.author_id.contains("Architect") {
                "🏛️"
            } else {
                "🐣"
            };
            
            content.push_str(&format!(
                "**[{}] {} {}:**\n> {}\n\n",
                post.created_at.format("%Y-%m-%d %H:%M:%S"),
                role_emoji,
                post.author_id,
                post.content
            ));
        }

        // Atomic write via temp file
        let tmp_path = nogari_path.with_extension("tmp");
        {
            let mut tmp_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)?;
            tmp_file.write_all(content.as_bytes())?;
            tmp_file.sync_all()?;
        }
        fs::rename(&tmp_path, &nogari_path)?;

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

        assert!(Path::new("lounge").exists());
        assert!(Path::new("lounge/agent_test-junior.log").exists());

        // 3. Compile to Nogari.md
        manager.compile_to_nogari_md().unwrap();
        assert!(Path::new("Nogari.md").exists());
        let _ = fs::remove_file("Nogari.md");
    }
}
