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
use rusqlite::{params, Connection, Result, OptionalExtension};
use chrono::{DateTime, Local};

pub struct Storage {
    pub conn: Arc<Mutex<Connection>>,
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
                result TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: Add result and dependencies to tasks if they don't exist
        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN result TEXT", []);
        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN dependencies TEXT", []);
        // v0.0.23: Hardened Migration - Ensure columns exist in 'tasks' table
        let table_info: Vec<String> = conn
            .prepare("PRAGMA table_info(tasks)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<Vec<String>, _>>()?;

        // v0.0.25: HARDENED - Enforce target_file NOT NULL via table migration
        let is_nullable = if let Ok(mut stmt) = conn.prepare("PRAGMA table_info(tasks)") {
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, i32>(3)?))
            })?;
            let mut nullable = true;
            for row in rows {
                if let Ok((name, not_null)) = row {
                    if name == "target_file" && not_null == 1 {
                        nullable = false;
                    }
                }
            }
            nullable
        } else { true };

        if !table_info.contains(&"target_file".to_string()) || is_nullable {
            tracing::info!("🛠️ [DB_MIGRATION] Enforcing NOT NULL on tasks.target_file...");
            
            let select_target = if table_info.contains(&"target_file".to_string()) {
                "COALESCE(target_file, 'STUB_RECOVERED')"
            } else {
                "'STUB_RECOVERED'"
            };

            let select_error = if table_info.contains(&"error_feedback".to_string()) {
                "error_feedback"
            } else {
                "NULL"
            };

            let select_deps = if table_info.contains(&"dependencies".to_string()) {
                "dependencies"
            } else {
                "NULL"
            };

            let migration_sql = format!("
                BEGIN TRANSACTION;
                CREATE TABLE tasks_new (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    description TEXT NOT NULL,
                    target_file TEXT NOT NULL,
                    status TEXT NOT NULL,
                    result TEXT,
                    error_feedback TEXT,
                    rework_count INTEGER NOT NULL DEFAULT 0,
                    base_hash TEXT,
                    lock_files TEXT,
                    parent_task TEXT,
                    reason TEXT,
                    kind TEXT NOT NULL DEFAULT 'unknown',
                    retries INTEGER NOT NULL DEFAULT 0,
                    assigned_worker TEXT,
                    created_at TEXT NOT NULL,
                    dependencies TEXT,
                    senior_comment TEXT
                );
                INSERT OR IGNORE INTO tasks_new (id, project_id, title, description, target_file, lock_files, status, result, error_feedback, rework_count, base_hash, parent_task, reason, kind, retries, assigned_worker, created_at, dependencies, senior_comment)
                SELECT id, project_id, title, description, {}, '[]', status, result, {}, 0, NULL, NULL, NULL, 'unknown', 0, NULL, created_at, {}, NULL FROM tasks;
                DROP TABLE tasks;
                ALTER TABLE tasks_new RENAME TO tasks;
                COMMIT;
            ", select_target, select_error, select_deps);

            conn.execute_batch(&migration_sql)?;
        }

        // Refresh table_info after potential migration
        let table_info: Vec<String> = conn
            .prepare("PRAGMA table_info(tasks)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<Vec<String>, _>>()?;
        if !table_info.contains(&"error_feedback".to_string()) {
            conn.execute("ALTER TABLE tasks ADD COLUMN error_feedback TEXT", [])?;
            tracing::info!("🛡️ [STORAGE_MIGRATION] Added 'error_feedback' column to 'tasks' table.");
        }
        
        if !table_info.contains(&"senior_comment".to_string()) {
            conn.execute("ALTER TABLE tasks ADD COLUMN senior_comment TEXT", [])?;
            tracing::info!("🛡️ [STORAGE_MIGRATION] Added 'senior_comment' column to 'tasks' table.");
        }

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
                metrics TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: Add full_code, metrics, and thought if they don't exist
        let _ = conn.execute("ALTER TABLE posts ADD COLUMN full_code TEXT", []);
        let _ = conn.execute("ALTER TABLE posts ADD COLUMN metrics TEXT", []);
        let _ = conn.execute("ALTER TABLE posts ADD COLUMN thought TEXT", []);

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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_stats_store (
                agent_id TEXT PRIMARY KEY,
                success_count INTEGER NOT NULL,
                fail_count INTEGER NOT NULL,
                latencies_json TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_locks (
                file_path TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                worker_id TEXT NOT NULL,
                lease_expiry INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS conflict_events (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                conflict_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                lock_files_json TEXT,
                base_hash_json TEXT,
                current_hash_json TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS worker_stats (
                worker_id TEXT PRIMARY KEY,
                success_rate REAL DEFAULT 0.0,
                avg_retries REAL DEFAULT 0.0,
                total_tasks INTEGER DEFAULT 0,
                specialization_json TEXT, -- HashMap<TaskKind, f32>
                last_updated TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rule_candidates (
                id TEXT PRIMARY KEY,
                pattern TEXT NOT NULL,
                fix_strategy TEXT NOT NULL,
                confidence REAL DEFAULT 0.0,
                occurrences INTEGER DEFAULT 0,
                state TEXT NOT NULL, -- Candidate, Validated, Promoted
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_locks_expiry ON file_locks(lease_expiry)",
            [],
        )?;
        Ok(())
    }

    pub fn save_task(&self, task: &axon_core::Task) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tasks (id, project_id, title, description, status, dependencies, result, target_file, lock_files, error_feedback, rework_count, base_hash, parent_task, reason, kind, retries, assigned_worker, created_at, senior_comment)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                task.id,
                task.project_id,
                task.title,
                task.description,
                format!("{:?}", task.status),
                serde_json::to_string(&task.dependencies).unwrap_or_else(|_| "[]".to_string()),
                task.result,
                task.target_file,
                serde_json::to_string(&task.lock_files).unwrap_or_else(|_| "[]".to_string()),
                task.error_feedback,
                task.rework_count,
                task.base_hash,
                task.parent_task,
                task.reason,
                task.kind,
                task.retries,
                task.assigned_worker,
                task.created_at.to_rfc3339(),
                task.senior_comment,
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
            "INSERT OR REPLACE INTO posts (id, thread_id, author_id, content, thought, full_code, post_type, metrics, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                post.id,
                post.thread_id,
                post.author_id,
                post.content,
                post.thought,
                post.full_code,
                format!("{:?}", post.post_type),
                post.metrics.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()),
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
        let mut stmt = conn.prepare("SELECT id, thread_id, author_id, content, thought, full_code, post_type, created_at, metrics FROM posts WHERE thread_id = ?1 ORDER BY created_at ASC")?;
        let post_iter = stmt.query_map(params![thread_id], |row| {
            Ok(axon_core::Post {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                author_id: row.get(2)?,
                content: row.get(3)?,
                thought: row.get(4)?,
                full_code: row.get(5)?,
                post_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(6)?)).unwrap_or(axon_core::PostType::System),
                metrics: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).unwrap().with_timezone(&Local),
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
        let mut stmt = conn.prepare("SELECT id, project_id, title, description, status, dependencies, result, target_file, lock_files, error_feedback, rework_count, base_hash, created_at, parent_task, reason, assigned_worker, kind, retries, senior_comment FROM tasks ORDER BY created_at DESC")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(axon_core::Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::TaskStatus::Pending),
                dependencies: serde_json::from_str(&row.get::<_, String>(5).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                result: row.get(6)?,
                target_file: row.get(7)?,
                lock_files: serde_json::from_str(&row.get::<_, String>(8).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                error_feedback: row.get(9)?,
                rework_count: row.get(10)?,
                base_hash: row.get(11)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(12)?).unwrap().with_timezone(&Local),
                parent_task: row.get(13)?,
                reason: row.get(14)?,
                assigned_worker: row.get(15)?,
                kind: row.get(16)?,
                retries: row.get(17)?,
                senior_comment: row.get(18)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn get_task(&self, id: &str) -> Result<Option<axon_core::Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, description, status, dependencies, result, target_file, lock_files, error_feedback, rework_count, base_hash, created_at, parent_task, reason, assigned_worker, kind, retries, senior_comment FROM tasks WHERE id = ?1")?;
        let mut task_iter = stmt.query_map(params![id], |row| {
            Ok(axon_core::Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::TaskStatus::Pending),
                dependencies: serde_json::from_str(&row.get::<_, String>(5).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                result: row.get(6)?,
                target_file: row.get(7)?,
                lock_files: serde_json::from_str(&row.get::<_, String>(8).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                error_feedback: row.get(9)?,
                rework_count: row.get(10)?,
                base_hash: row.get(11)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(12)?).unwrap().with_timezone(&Local),
                parent_task: row.get(13)?,
                reason: row.get(14)?,
                assigned_worker: row.get(15)?,
                kind: row.get(16)?,
                retries: row.get(17)?,
                senior_comment: row.get(18)?,
            })
        })?;

        if let Some(task) = task_iter.next() {
            Ok(Some(task?))
        } else {
            Ok(None)
        }
    }

    pub fn save_agent_stats(&self, agent_id: &str, success: usize, fail: usize, latencies_json: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO agent_stats_store (agent_id, success_count, fail_count, latencies_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![agent_id, success, fail, latencies_json],
        )?;
        Ok(())
    }

    pub fn load_all_agent_stats(&self) -> Result<Vec<(String, usize, usize, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT agent_id, success_count, fail_count, latencies_json FROM agent_stats_store")?;
        let stats_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, usize>(1)?,
                row.get::<_, usize>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut stats = Vec::new();
        for s in stats_iter {
            stats.push(s?);
        }
        Ok(stats)
    }

    pub fn get_thread(&self, id: &str) -> Result<Option<axon_core::Thread>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, status, author, milestone_id, created_at, updated_at FROM threads WHERE id = ?1")?;
        let mut thread_iter = stmt.query_map(params![id], |row| {
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

        if let Some(thread) = thread_iter.next() {
            Ok(Some(thread?))
        } else {
            Ok(None)
        }
    }

    pub fn acquire_lock_set(&self, files: &Vec<String>, task_id: &str, worker_id: &str, lease_duration_secs: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = Local::now().timestamp();
        let expiry = now + lease_duration_secs;

        // v0.0.25: Deterministic sorting to prevent deadlocks (Critical Requirement)
        let mut sorted_files = files.clone();
        sorted_files.sort();

        for f in &sorted_files {
            // Clean up expired locks for these files
            let _ = conn.execute("DELETE FROM file_locks WHERE file_path = ?1 AND lease_expiry < ?2", params![f, now]);
            
            // Check collision
            let existing: Option<String> = conn.query_row(
                "SELECT task_id FROM file_locks WHERE file_path = ?1",
                params![f],
                |r| r.get(0)
            ).optional()?;

            if let Some(owner) = existing {
                if owner != task_id {
                    return Ok(false);
                }
            }
        }

        // Acquire all files in the set (Atomic within Mutex protected connection)
        for f in &sorted_files {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO file_locks (file_path, task_id, worker_id, lease_expiry) VALUES (?1, ?2, ?3, ?4)",
                params![f, task_id, worker_id, expiry]
            );
        }
        Ok(true)
    }

    pub fn release_lock_set(&self, files: &Vec<String>, task_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for f in files {
            let _ = conn.execute("DELETE FROM file_locks WHERE file_path = ?1 AND task_id = ?2", params![f, task_id]);
        }
        Ok(())
    }

    pub fn verify_lock_set_owner(&self, files: &Vec<String>, task_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        for f in files {
            let owner: Option<String> = conn.query_row(
                "SELECT task_id FROM file_locks WHERE file_path = ?1",
                params![f],
                |r| r.get(0)
            ).optional()?;
            if owner.as_deref() != Some(task_id) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn is_locked(&self, file_path: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = Local::now().timestamp();
        let exists: Option<String> = conn.query_row(
            "SELECT task_id FROM file_locks WHERE file_path = ?1 AND lease_expiry > ?2",
            params![file_path, now],
            |r| r.get(0)
        ).optional()?;
        Ok(exists.is_some())
    }

    pub fn release_lock(&self, file_path: &str, task_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM file_locks WHERE file_path = ?1 AND task_id = ?2", params![file_path, task_id])?;
        Ok(())
    }

    pub fn release_all_locks_for_task(&self, task_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM file_locks WHERE task_id = ?1", params![task_id])?;
        Ok(())
    }
    pub fn get_all_active_locks(&self) -> Result<std::collections::HashSet<String>> {
        let conn = self.conn.lock().unwrap();
        let now = Local::now().timestamp();
        let mut stmt = conn.prepare("SELECT file_path FROM file_locks WHERE lease_expiry > ?1")?;
        let rows = stmt.query_map(params![now], |row| row.get(0))?;
        let mut locks = std::collections::HashSet::new();
        for row in rows {
            if let Ok(path) = row {
                locks.insert(path);
            }
        }
        Ok(locks)
    }
    pub fn create_conflict_event(&self, task_id: &str, conflict_type: &str, file_path: &str, lock_files: &Vec<String>, base_hash: Option<String>, current_hash: Option<String>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO conflict_events (id, task_id, conflict_type, file_path, lock_files_json, base_hash_json, current_hash_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                task_id,
                conflict_type,
                file_path,
                serde_json::to_string(lock_files).ok(),
                base_hash,
                current_hash,
                now
            ],
        )?;
        Ok(())
    }

    pub fn update_worker_stats(&self, worker_id: &str, success: bool, retries: u32, kind: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Local::now().to_rfc3339();
        
        let success_val = if success { 1 } else { 0 };
        
        // 1. Fetch current specialization
        let current_spec_json: String = conn.query_row(
            "SELECT specialization_json FROM worker_stats WHERE worker_id = ?1",
            params![worker_id],
            |r| r.get(0)
        ).unwrap_or_else(|_| "{}".to_string());
        
        let mut spec: std::collections::HashMap<String, f32> = serde_json::from_str(&current_spec_json).unwrap_or_default();
        
        // 2. Update specialization score for this kind
        let entry = spec.entry(kind.to_string()).or_insert(0.5);
        if success {
            *entry += 0.05;
        } else {
            *entry -= 0.05;
        }
        *entry = entry.clamp(0.0, 1.0);
        
        let new_spec_json = serde_json::to_string(&spec).unwrap_or_else(|_| "{}".to_string());

        // 3. Upsert stats
        conn.execute(
            "INSERT INTO worker_stats (worker_id, success_rate, avg_retries, total_tasks, specialization_json, last_updated)
             VALUES (?1, ?2, ?3, 1, ?4, ?5)
             ON CONFLICT(worker_id) DO UPDATE SET
                success_rate = (success_rate * total_tasks + ?2) / (total_tasks + 1),
                avg_retries = (avg_retries * total_tasks + ?3) / (total_tasks + 1),
                total_tasks = total_tasks + 1,
                specialization_json = ?4,
                last_updated = ?5",
            params![worker_id, success_val as f64, retries as f64, new_spec_json, now],
        )?;
        Ok(())
    }

    pub fn get_worker_stats(&self) -> Result<std::collections::HashMap<String, (f64, f64, i64, std::collections::HashMap<String, f32>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT worker_id, success_rate, avg_retries, total_tasks, specialization_json FROM worker_stats")?;
        let rows = stmt.query_map([], |row| {
            let spec: std::collections::HashMap<String, f32> = serde_json::from_str(&row.get::<_, String>(4).unwrap_or_else(|_| "{}".to_string())).unwrap_or_default();
            Ok((row.get::<_, String>(0)?, (row.get::<_, f64>(1)?, row.get::<_, f64>(2)?, row.get::<_, i64>(3)?, spec)))
        })?;
        
        let mut stats = std::collections::HashMap::new();
        for row in rows {
            if let Ok((id, data)) = row {
                stats.insert(id, data);
            }
        }
        Ok(stats)
    }
}
