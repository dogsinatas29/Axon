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

use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Local};

pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn: Arc::new(Mutex::new(conn)) };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS threads (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                author TEXT NOT NULL,
                milestone_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS posts (
                id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                content TEXT NOT NULL,
                full_code TEXT,
                post_type TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: Add full_code if it doesn't exist
        let _ = conn.execute("ALTER TABLE posts ADD COLUMN full_code TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                persona TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL,
                parent_id TEXT,
                dtr REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS patches (
                id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                workspace_path TEXT NOT NULL,
                diff TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_log (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                thread_id TEXT,
                agent_id TEXT,
                event_type TEXT NOT NULL,
                source TEXT NOT NULL,
                content TEXT NOT NULL,
                payload TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save_task(&self, task: &axon_core::Task) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tasks (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                task.id,
                task.project_id,
                task.title,
                task.description,
                format!("{:?}", task.status),
                task.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn save_thread(&self, thread: &axon_core::Thread) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO threads (id, project_id, title, status, author, milestone_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                thread.id,
                thread.project_id,
                thread.title,
                format!("{:?}", thread.status),
                thread.author,
                thread.milestone_id,
                thread.created_at.to_rfc3339(),
                thread.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn save_post(&self, post: &axon_core::Post) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO posts (id, thread_id, author_id, content, full_code, post_type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                post.id,
                post.thread_id,
                post.author_id,
                post.content,
                post.full_code,
                format!("{:?}", post.post_type),
                post.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn save_event(&self, event: &axon_core::Event) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO event_log (id, project_id, thread_id, agent_id, event_type, source, content, payload, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.id,
                event.project_id,
                event.thread_id,
                event.agent_id,
                format!("{:?}", event.event_type),
                event.source,
                event.content,
                event.payload.as_ref().map(|p| p.to_string()),
                event.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_runnable_threads(&self) -> Result<Vec<axon_core::Thread>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, status, author, milestone_id, created_at, updated_at FROM threads WHERE status != 'Completed' AND status != 'Paused'")?;
        let thread_iter = stmt.query_map([], |row| {
            Ok(axon_core::Thread {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?)).unwrap_or(axon_core::ThreadStatus::Draft),
                author: row.get(4)?,
                milestone_id: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap().with_timezone(&Local),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).unwrap().with_timezone(&Local),
            })
        })?;

        let mut threads = Vec::new();
        for thread in thread_iter {
            threads.push(thread?);
        }
        Ok(threads)
    }

    pub fn list_all_threads(&self) -> Result<Vec<axon_core::Thread>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, status, author, milestone_id, created_at, updated_at FROM threads ORDER BY updated_at DESC")?;
        let thread_iter = stmt.query_map([], |row| {
            Ok(axon_core::Thread {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?)).unwrap_or(axon_core::ThreadStatus::Draft),
                author: row.get(4)?,
                milestone_id: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap().with_timezone(&Local),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).unwrap().with_timezone(&Local),
            })
        })?;

        let mut threads = Vec::new();
        for thread in thread_iter {
            threads.push(thread?);
        }
        Ok(threads)
    }

    pub fn list_posts_by_thread(&self, thread_id: &str) -> Result<Vec<axon_core::Post>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, thread_id, author_id, content, full_code, post_type, created_at FROM posts WHERE thread_id = ?1 ORDER BY created_at ASC")?;
        let post_iter = stmt.query_map(params![thread_id], |row| {
            Ok(axon_core::Post {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                author_id: row.get(2)?,
                content: row.get(3)?,
                full_code: row.get(4)?,
                post_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(5)?)).unwrap_or(axon_core::PostType::System),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap().with_timezone(&Local),
            })
        })?;

        let mut posts = Vec::new();
        for post in post_iter {
            posts.push(post?);
        }
        Ok(posts)
    }

    pub fn list_agents(&self) -> Result<Vec<axon_core::Agent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, role, persona, model, status, parent_id, dtr FROM agents")?;
        let agent_iter = stmt.query_map([], |row| {
            Ok(axon_core::Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                role: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(2)?)).unwrap_or(axon_core::AgentRole::Junior),
                persona: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                model: row.get(4)?,
                status: row.get(5)?,
                parent_id: row.get(6)?,
                dtr: row.get(7)?,
            })
        })?;

        let mut agents = Vec::new();
        for agent in agent_iter {
            agents.push(agent?);
        }
        Ok(agents)
    }

    pub fn save_agent(&self, agent: &axon_core::Agent) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO agents (id, name, role, persona, model, status, parent_id, dtr)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                agent.id,
                agent.name,
                format!("{:?}", agent.role),
                serde_json::to_string(&agent.persona).unwrap(),
                agent.model,
                agent.status,
                agent.parent_id,
                agent.dtr,
            ],
        )?;
        Ok(())
    }
    pub fn delete_agent(&self, agent_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM agents WHERE id = ?1", params![agent_id])?;
        Ok(())
    }

    pub fn reassign_agents_by_parent(&self, old_parent_id: &str, new_parent_id: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE agents SET parent_id = ?1 WHERE parent_id = ?2",
            params![new_parent_id, old_parent_id],
        )?;
        Ok(())
    }

    pub fn list_all_tasks(&self) -> Result<Vec<axon_core::Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, description, status, created_at FROM tasks ORDER BY created_at DESC")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(axon_core::Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::TaskStatus::Pending),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?).unwrap().with_timezone(&Local),
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }
}
