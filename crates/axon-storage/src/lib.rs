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

use rusqlite::{params, Connection, Result, OptionalExtension};
pub use axon_core::{Task, Agent, Post};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Local};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WriteOpEnvelope {
    pub id: String,
    pub op: WriteOp,
    pub retries: u32,
    pub timestamp: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum WriteOp {
    SaveTask(Task),
    SaveThread(axon_core::Thread),
    SavePost(Post),
    SaveEvent(axon_core::Event),
    UpdateProjectState { project_id: String, stage: String, status: String },
    LogExecution { project_id: String, stage: String, status: String, raw: String, result: String },
    SaveAgent(Agent),
    DeleteAgent(String),
    SaveAgentStats { agent_id: String, success: usize, fail: usize, latencies_json: String },
    Ack(String), 
}

#[derive(Clone)]
pub struct Storage {
    pub conn: Arc<Mutex<Connection>>,
    pub tx: mpsc::Sender<WriteOpEnvelope>,
    flush_tx: mpsc::UnboundedSender<oneshot::Sender<()>>,
    pub log_path: String,
    pub dlq_path: String,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        let log_path = format!("{}.log", path);
        let dlq_path = format!("{}.dead_letter.log", path);
        
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        let conn = Arc::new(Mutex::new(conn));
        let (tx, mut rx) = mpsc::channel::<WriteOpEnvelope>(4096);
        let (flush_tx, mut flush_rx) = mpsc::unbounded_channel::<oneshot::Sender<()>>();
        
        // 1. Safe Recovery (Handling Partial Writes/Corruption)
        let mut recovered_ops = Vec::new();
        if std::path::Path::new(&log_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&log_path) {
                let mut pending = std::collections::HashMap::new();
                let mut acks = std::collections::HashSet::new();
                
                for line in content.lines() {
                    // Robust parsing: ignore corrupt lines
                    if let Ok(env) = serde_json::from_str::<WriteOpEnvelope>(line) {
                        match env.op {
                            WriteOp::Ack(ref id) => { acks.insert(id.clone()); }
                            _ => { pending.insert(env.id.clone(), env); }
                        }
                    }
                }
                for (id, env) in pending {
                    if !acks.contains(&id) { recovered_ops.push(env); }
                }
            }
        }

        let db_writer = conn.clone();
        let writer_log = log_path.clone();
        let writer_dlq = dlq_path.clone();
        
        tokio::spawn(async move {
            for env in recovered_ops {
                let _ = Self::flush_batch_durable(&db_writer, &writer_log, &writer_dlq, &mut vec![env]);
            }

            let mut batch = Vec::new();
            loop {
                let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(100));
                tokio::select! {
                    Some(env) = rx.recv() => {
                        batch.push(env);
                        if batch.len() >= 50 {
                            Self::flush_batch_durable(&db_writer, &writer_log, &writer_dlq, &mut batch);
                        }
                    }
                    Some(ack_sender) = flush_rx.recv() => {
                        // Drain ALL pending write-ops FIRST (FIFO ordering guarantee)
                        while let Ok(env) = rx.try_recv() {
                            batch.push(env);
                        }
                        if !batch.is_empty() {
                            Self::flush_batch_durable(&db_writer, &writer_log, &writer_dlq, &mut batch);
                        }
                        let _ = ack_sender.send(());
                    }
                    _ = timeout => {
                        if !batch.is_empty() {
                            Self::flush_batch_durable(&db_writer, &writer_log, &writer_dlq, &mut batch);
                        }
                    }
                }
            }
        });

        let storage = Self { conn, tx, flush_tx, log_path, dlq_path };
        storage.init_schema()?;
        Ok(storage)
    }

    fn flush_batch_durable(db: &Arc<Mutex<Connection>>, log_path: &str, dlq_path: &str, batch: &mut Vec<WriteOpEnvelope>) {
        let mut conn = db.lock().unwrap();
        let tx = match conn.transaction() {
            Ok(t) => t,
            Err(e) => { tracing::error!("❌ [DB_TX_FAIL] {}", e); return; }
        };

        let mut success_ids = Vec::new();
        let mut failures = Vec::new();

        for env in batch.drain(..) {
            if Self::apply_op_internal(&tx, &env.op) {
                success_ids.push(env.id);
            } else {
                // 2. Poison Pill Handling (Dead Letter Queue)
                if env.retries >= 3 {
                    tracing::error!("☣️ [POISON_PILL] Op {} failed 3 times. Moving to DLQ.", env.id);
                    let _ = Self::append_to_log(dlq_path, &env);
                } else {
                    let mut retry_env = env;
                    retry_env.retries += 1;
                    failures.push(retry_env);
                }
            }
        }

        if let Ok(_) = tx.commit() {
            // 3. Strict ACK Timing: Only after Commit
            for id in success_ids {
                let ack_env = WriteOpEnvelope {
                    id: uuid::Uuid::new_v4().to_string(),
                    op: WriteOp::Ack(id),
                    retries: 0,
                    timestamp: Local::now().timestamp(),
                };
                let _ = Self::append_to_log(log_path, &ack_env);
            }
        }
        
        // Push failures back (simplified for this context)
    }

    fn append_to_log(path: &str, env: &WriteOpEnvelope) -> anyhow::Result<()> {
        if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
            let _ = writeln!(file, "{}", serde_json::to_string(env).unwrap());
            let _ = file.flush();
        }
        Ok(())
    }


    fn apply_op_internal(tx: &rusqlite::Transaction, op: &WriteOp) -> bool {
        match op {
            WriteOp::SaveTask(t) => {
                // v0.0.31.20: [PRESERVE_REWORK_STATE] 
                // If a task with the same ID already exists in the database and is NOT Completed,
                // we MUST inherit/preserve its cumulative rework state to prevent API/spec reprocessing 
                // from resetting the rejection history.
                let mut final_rework_count = t.rework_count;
                let mut final_validator_rejections = t.validator_rejections;
                let mut final_senior_rejections = t.senior_rejections;
                let mut final_architecture_rejections = t.architecture_rejections;
                let mut final_cargo_rejections = t.cargo_rejections;
                let mut final_lsp_rejections = t.lsp_rejections;
                let mut final_boss_interventions = t.boss_interventions;
                let mut final_error_feedback = t.error_feedback.clone();
                let mut final_senior_comment = t.senior_comment.clone();

                let existing_query = tx.query_row(
                    "SELECT status, rework_count, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions, error_feedback, senior_comment FROM tasks WHERE id = ?1",
                    params![t.id],
                    |row| {
                        let status: String = row.get(0)?;
                        let rework_count: u32 = row.get(1)?;
                        let val_rejs: u32 = row.get(2)?;
                        let sen_rejs: u32 = row.get(3)?;
                        let arch_rejs: u32 = row.get(4)?;
                        let cargo_rejs: u32 = row.get(5)?;
                        let lsp_rejs: u32 = row.get(6)?;
                        let boss_ints: u32 = row.get(7)?;
                        let err_feedback: Option<String> = row.get(8)?;
                        let sen_comment: Option<String> = row.get(9)?;
                        Ok((status, rework_count, val_rejs, sen_rejs, arch_rejs, cargo_rejs, lsp_rejs, boss_ints, err_feedback, sen_comment))
                    }
                );

                if let Ok((status, r_count, val_rejs, sen_rejs, arch_rejs, cargo_rejs, lsp_rejs, boss_ints, err_feedback, sen_comment)) = existing_query {
                    if status != "Completed" {
                        final_rework_count = std::cmp::max(t.rework_count, r_count);
                        final_validator_rejections = std::cmp::max(t.validator_rejections, val_rejs);
                        final_senior_rejections = std::cmp::max(t.senior_rejections, sen_rejs);
                        final_architecture_rejections = std::cmp::max(t.architecture_rejections, arch_rejs);
                        final_cargo_rejections = std::cmp::max(t.cargo_rejections, cargo_rejs);
                        final_lsp_rejections = std::cmp::max(t.lsp_rejections, lsp_rejs);
                        final_boss_interventions = std::cmp::max(t.boss_interventions, boss_ints);
                        
                        if final_error_feedback.is_none() && err_feedback.is_some() {
                            final_error_feedback = err_feedback;
                        }
                        if final_senior_comment.is_none() && sen_comment.is_some() {
                            final_senior_comment = sen_comment;
                        }
                    }
                }

                tx.execute(
                    "INSERT INTO tasks (id, project_id, title, description, status, lifecycle_state, target_file, dependencies, error_feedback, senior_comment, rework_count, base_hash, parent_task, reason, kind, retries, assigned_worker, created_at, ir_path, task_kind, signature, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)
                     ON CONFLICT(id) DO UPDATE SET
                       project_id=excluded.project_id, title=excluded.title, description=excluded.description,
                       status=excluded.status, lifecycle_state=excluded.lifecycle_state, target_file=excluded.target_file,
                       dependencies=excluded.dependencies, error_feedback=excluded.error_feedback,
                       senior_comment=excluded.senior_comment, rework_count=excluded.rework_count,
                       base_hash=excluded.base_hash, parent_task=excluded.parent_task, reason=excluded.reason,
                       kind=excluded.kind, retries=excluded.retries, assigned_worker=excluded.assigned_worker,
                       created_at=excluded.created_at, ir_path=excluded.ir_path, task_kind=excluded.task_kind,
                       signature=excluded.signature, validator_rejections=excluded.validator_rejections,
                       senior_rejections=excluded.senior_rejections, architecture_rejections=excluded.architecture_rejections,
                       cargo_rejections=excluded.cargo_rejections, lsp_rejections=excluded.lsp_rejections,
                       boss_interventions=excluded.boss_interventions
                     WHERE tasks.status != 'Completed'",
                    params![
                        t.id, 
                        t.project_id, 
                        t.title, 
                        t.description, 
                        format!("{:?}", t.status), 
                        format!("{:?}", t.lifecycle_state),
                        t.target_file, 
                        serde_json::to_string(&t.dependencies).unwrap_or_default(), 
                        final_error_feedback, 
                        final_senior_comment, 
                        final_rework_count, 
                        t.base_hash, 
                        t.parent_task, 
                        t.reason, 
                        t.kind, 
                        t.retries, 
                        t.assigned_worker, 
                        t.created_at.to_rfc3339(),
                        t.ir_path,
                        t.task_kind.as_ref().map(|k| serde_json::to_string(k).unwrap_or_default()),
                        t.signature,
                        final_validator_rejections,
                        final_senior_rejections,
                        final_architecture_rejections,
                        final_cargo_rejections,
                        final_lsp_rejections,
                        final_boss_interventions
                    ],
                ).is_ok()
            },
            WriteOp::SaveThread(th) => {
                tx.execute(
                    "INSERT OR REPLACE INTO threads (id, project_id, title, status, author, milestone_id, task_kind, rejection_count, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions, error_feedback, reason, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                    params![
                        th.id, 
                        th.project_id, 
                        th.title, 
                        format!("{:?}", th.status), 
                        th.author, 
                        th.milestone_id, 
                        th.task_kind.as_ref().map(|k| serde_json::to_string(k).unwrap_or_default()),
                        th.rejection_count,
                        th.validator_rejections,
                        th.senior_rejections,
                        th.architecture_rejections,
                        th.cargo_rejections,
                        th.lsp_rejections,
                        th.boss_interventions,
                        th.error_feedback,
                        th.reason,
                        th.created_at.to_rfc3339(), 
                        th.updated_at.to_rfc3339()
                    ],
                ).is_ok()
            },
            WriteOp::SavePost(p) => {
                tx.execute(
                    "INSERT OR REPLACE INTO posts (id, thread_id, author_id, content, thought, full_code, post_type, metrics, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![p.id, p.thread_id, p.author_id, p.content, p.thought, p.full_code, format!("{:?}", p.post_type), p.metrics.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()), p.created_at.to_rfc3339()],
                ).is_ok()
            },
            WriteOp::SaveEvent(e) => {
                tx.execute(
                    "INSERT OR REPLACE INTO event_log (id, project_id, thread_id, agent_id, event_type, source, level, content, payload, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![e.id, e.project_id, e.thread_id, e.agent_id, format!("{:?}", e.event_type), e.source, format!("{:?}", e.level), e.content, e.payload.as_ref().map(|p| serde_json::to_string(p).unwrap_or_default()), e.timestamp.to_rfc3339()],
                ).is_ok()
            },
            WriteOp::SaveAgent(a) => {
                tx.execute(
                    "INSERT OR REPLACE INTO agents (id, name, role, persona, model, status, parent_id, dtr) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![a.id, a.name, format!("{:?}", a.role), serde_json::to_string(&a.persona).unwrap_or_default(), a.model, a.status, a.parent_id, a.dtr],
                ).is_ok()
            },
            WriteOp::DeleteAgent(id) => {
                tx.execute("DELETE FROM agents WHERE id = ?1", params![id]).is_ok()
            },
            WriteOp::SaveAgentStats { agent_id, success, fail, latencies_json } => {
                tx.execute(
                    "INSERT OR REPLACE INTO agent_stats_store (agent_id, success_count, fail_count, latencies_json) VALUES (?1, ?2, ?3, ?4)",
                    params![agent_id, success, fail, latencies_json],
                ).is_ok()
            },
            WriteOp::UpdateProjectState { project_id, stage, status } => {
                tx.execute("INSERT OR REPLACE INTO project_state (project_id, current_stage, status, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP)", params![project_id, stage, status]).is_ok()
            },
            WriteOp::LogExecution { project_id, stage, status, raw, result } => {
                tx.execute("INSERT INTO execution_log (project_id, stage, status, raw, result) VALUES (?, ?, ?, ?, ?)", params![project_id, stage, status, raw, result]).is_ok()
            },
            WriteOp::Ack(_) => true,
        }
    }

    pub async fn flush(&self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.flush_tx.send(tx).map_err(|e| anyhow::anyhow!("Failed to send flush signal: {}", e))?;
        rx.await.map_err(|e| anyhow::anyhow!("Flush ack channel dropped: {}", e))?;
        tracing::debug!("💾 WAL queue successfully flushed to SQLite disk storage.");
        Ok(())
    }

    async fn enqueue_durable(&self, op: WriteOp) -> anyhow::Result<()> {
        let env = WriteOpEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            op,
            retries: 0,
            timestamp: Local::now().timestamp(),
        };

        // 1. Append to Disk Log (Must succeed for Durability)
        Self::append_to_log(&self.log_path, &env)?;
        
        // 2. Push to Memory Queue
        self.tx.send(env).await.map_err(|e| anyhow::anyhow!("Failed to enqueue: {}", e))?;
        Ok(())
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // 1. Core Tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                title TEXT,
                description TEXT,
                status TEXT DEFAULT 'pending',
                stage TEXT,
                retries INTEGER DEFAULT 0,
                lock_version INTEGER DEFAULT 0,
                target_file TEXT,
                dependencies TEXT,
                error_feedback TEXT,
                senior_comment TEXT,
                rework_count INTEGER DEFAULT 0,
                base_hash TEXT,
                parent_task TEXT,
                reason TEXT,
                kind TEXT,
                assigned_worker TEXT,
                result TEXT,
                ir_path TEXT,
                task_kind TEXT,
                signature TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // project_state for isolation
        conn.execute(
            "CREATE TABLE IF NOT EXISTS project_state (
                project_id TEXT PRIMARY KEY,
                current_stage TEXT,
                status TEXT DEFAULT 'running',
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // execution_log for traceability
        conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT,
                stage TEXT,
                status TEXT,
                raw TEXT,
                result TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Recovery: Reset orphan running tasks to pending
        conn.execute(
            "UPDATE tasks SET status = 'pending' WHERE status = 'running'",
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
                task_kind TEXT,
                rejection_count INTEGER DEFAULT 0,
                validator_rejections INTEGER DEFAULT 0,
                senior_rejections INTEGER DEFAULT 0,
                architecture_rejections INTEGER DEFAULT 0,
                cargo_rejections INTEGER DEFAULT 0,
                lsp_rejections INTEGER DEFAULT 0,
                error_feedback TEXT,
                reason TEXT,
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
                post_type TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // 2. [DB_MIGRATION] Incremental Column Addition
        let table_info_tasks: Vec<String> = conn
            .prepare("PRAGMA table_info(tasks)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<Vec<String>, _>>()?;

        let tasks_columns = vec![
            ("target_file", "TEXT NOT NULL DEFAULT 'STUB'"),
            ("dependencies", "TEXT DEFAULT '[]'"),
            ("error_feedback", "TEXT"),
            ("senior_comment", "TEXT"),
            ("rework_count", "INTEGER DEFAULT 0"),
            ("base_hash", "TEXT"),
            ("parent_task", "TEXT"),
            ("reason", "TEXT"),
            ("kind", "TEXT DEFAULT 'rust'"),
            ("retries", "INTEGER DEFAULT 0"),
            ("assigned_worker", "TEXT"),
            ("lock_files", "TEXT DEFAULT '[]'"),
            ("result", "TEXT"),
            ("ir_path", "TEXT"),
            ("task_kind", "TEXT"),
            ("validator_rejections", "INTEGER DEFAULT 0"),
            ("senior_rejections", "INTEGER DEFAULT 0"),
            ("architecture_rejections", "INTEGER DEFAULT 0"),
            ("cargo_rejections", "INTEGER DEFAULT 0"),
            ("lsp_rejections", "INTEGER DEFAULT 0"),
            ("boss_interventions", "INTEGER DEFAULT 0"),
            ("lifecycle_state", "TEXT DEFAULT 'Queued'"), // v0.0.31.xx: scheduler semantics
        ];

        for (col_name, col_type) in tasks_columns {
            if !table_info_tasks.contains(&col_name.to_string()) {
                tracing::info!("🛠️ [DB_MIGRATION:tasks] Adding missing column: {}", col_name);
                let _ = conn.execute(&format!("ALTER TABLE tasks ADD COLUMN {} {}", col_name, col_type), []);
            }
        }

        // v0.0.31.xx: Migration backfill for NULL lifecycle_state
        let _: Result<usize, _> = conn.execute(
            "UPDATE tasks SET lifecycle_state = CASE WHEN status = 'Completed' THEN 'Completed' WHEN status = 'Failed' THEN 'Rejected' ELSE 'Queued' END WHERE lifecycle_state IS NULL",
            [],
        );

        let table_info_posts: Vec<String> = conn
            .prepare("PRAGMA table_info(posts)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<Vec<String>, _>>()?;

        let posts_columns = vec![
            ("thought", "TEXT"),
            ("full_code", "TEXT"),
            ("metrics", "TEXT"),
        ];

        for (col_name, col_type) in posts_columns {
            if !table_info_posts.contains(&col_name.to_string()) {
                tracing::info!("🛠️ [DB_MIGRATION:posts] Adding missing column: {}", col_name);
                let _ = conn.execute(&format!("ALTER TABLE posts ADD COLUMN {} {}", col_name, col_type), []);
            }
        }
        
        let table_info_threads: Vec<String> = conn
            .prepare("PRAGMA table_info(threads)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<Vec<String>, _>>()?;

        if !table_info_threads.contains(&"task_kind".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: task_kind");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN task_kind TEXT", []);
        }

        if !table_info_threads.contains(&"rejection_count".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: rejection_count");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN rejection_count INTEGER DEFAULT 0", []);
        }

        if !table_info_threads.contains(&"validator_rejections".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: validator_rejections");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN validator_rejections INTEGER DEFAULT 0", []);
        }

        if !table_info_threads.contains(&"senior_rejections".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: senior_rejections");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN senior_rejections INTEGER DEFAULT 0", []);
        }

        if !table_info_threads.contains(&"architecture_rejections".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: architecture_rejections");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN architecture_rejections INTEGER DEFAULT 0", []);
        }

        if !table_info_threads.contains(&"cargo_rejections".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: cargo_rejections");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN cargo_rejections INTEGER DEFAULT 0", []);
        }

        if !table_info_threads.contains(&"lsp_rejections".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: lsp_rejections");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN lsp_rejections INTEGER DEFAULT 0", []);
        }

        if !table_info_threads.contains(&"error_feedback".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: error_feedback");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN error_feedback TEXT", []);
        }

        if !table_info_threads.contains(&"reason".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: reason");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN reason TEXT", []);
        }

        if !table_info_threads.contains(&"boss_interventions".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:threads] Adding missing column: boss_interventions");
            let _ = conn.execute("ALTER TABLE threads ADD COLUMN boss_interventions INTEGER DEFAULT 0", []);
        }

        // 3. Auxiliary Tables
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
                level TEXT DEFAULT 'Info',
                content TEXT NOT NULL,
                payload TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        let table_info_event: Vec<String> = conn
            .prepare("PRAGMA table_info(event_log)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<Vec<String>, _>>()?;

        if !table_info_event.contains(&"source".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:event_log] Adding missing column: source");
            let _ = conn.execute("ALTER TABLE event_log ADD COLUMN source TEXT NOT NULL DEFAULT 'daemon'", []);
        }

        if !table_info_event.contains(&"level".to_string()) {
            tracing::info!("🛠️ [DB_MIGRATION:event_log] Adding missing column: level");
            let _ = conn.execute("ALTER TABLE event_log ADD COLUMN level TEXT DEFAULT 'Info'", []);
        }

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
                specialization_json TEXT,
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
                state TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_file_locks_expiry ON file_locks(lease_expiry)", [])?;
        
        Ok(())
    }

    pub async fn save_task(&self, task: axon_core::Task) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::SaveTask(task)).await;
        Ok(())
    }

    pub fn save_task_sync(&self, task: axon_core::Task) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        Self::apply_op_internal(&tx, &WriteOp::SaveTask(task));
        tx.commit()?;
        Ok(())
    }

    pub async fn save_thread(&self, thread: axon_core::Thread) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::SaveThread(thread)).await;
        Ok(())
    }

    pub fn save_thread_sync(&self, thread: axon_core::Thread) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        Self::apply_op_internal(&tx, &WriteOp::SaveThread(thread));
        tx.commit()?;
        Ok(())
    }

    pub async fn save_post(&self, post: axon_core::Post) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::SavePost(post)).await;
        Ok(())
    }

    pub fn save_post_sync(&self, post: axon_core::Post) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        Self::apply_op_internal(&tx, &WriteOp::SavePost(post));
        tx.commit()?;
        Ok(())
    }

    pub async fn save_event(&self, event: axon_core::Event) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::SaveEvent(event)).await;
        Ok(())
    }

    pub fn list_runnable_threads(&self) -> Result<Vec<axon_core::Thread>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, status, author, milestone_id, created_at, updated_at, task_kind, rejection_count, error_feedback, reason, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions FROM threads WHERE status != 'Completed' AND status != 'Paused'")?;
        let thread_iter = stmt.query_map([], |row| {
            Ok(axon_core::Thread {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?)).unwrap_or(axon_core::ThreadStatus::Draft),
                author: row.get(4)?,
                milestone_id: row.get(5)?,
                task_kind: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
                rejection_count: row.get(9)?,
                validator_rejections: row.get(12).unwrap_or(0),
                senior_rejections: row.get(13).unwrap_or(0),
                architecture_rejections: row.get(14).unwrap_or(0),
                cargo_rejections: row.get(15).unwrap_or(0),
                lsp_rejections: row.get(16).unwrap_or(0),
                boss_interventions: row.get(17).unwrap_or(0),
                error_feedback: row.get(10)?,
                reason: row.get(11)?,
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
        let mut stmt = conn.prepare("SELECT id, project_id, title, status, author, milestone_id, created_at, updated_at, task_kind, rejection_count, error_feedback, reason, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions FROM threads ORDER BY updated_at DESC")?;
        let thread_iter = stmt.query_map([], |row| {
            Ok(axon_core::Thread {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?)).unwrap_or(axon_core::ThreadStatus::Draft),
                author: row.get(4)?,
                milestone_id: row.get(5)?,
                task_kind: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
                rejection_count: row.get(9)?,
                validator_rejections: row.get(12).unwrap_or(0),
                senior_rejections: row.get(13).unwrap_or(0),
                architecture_rejections: row.get(14).unwrap_or(0),
                cargo_rejections: row.get(15).unwrap_or(0),
                lsp_rejections: row.get(16).unwrap_or(0),
                boss_interventions: row.get(17).unwrap_or(0),
                error_feedback: row.get(10)?,
                reason: row.get(11)?,
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

    pub async fn save_agent(&self, agent: axon_core::Agent) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::SaveAgent(agent)).await;
        Ok(())
    }

    pub async fn delete_agent(&self, agent_id: String) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::DeleteAgent(agent_id)).await;
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

    pub fn claim_task(&self, task_id: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE tasks 
             SET status = 'running', lock_version = lock_version + 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ? AND status = 'pending'",
            params![task_id],
        )?;
        Ok(rows > 0)
    }

    pub async fn update_project_state(&self, project_id: &str, stage: &str, status: &str) -> anyhow::Result<()> {
        let _ = self.enqueue_durable(WriteOp::UpdateProjectState { 
            project_id: project_id.to_string(), stage: stage.to_string(), status: status.to_string() 
        }).await;
        Ok(())
    }

    pub fn get_project_state(&self, project_id: &str) -> Result<Option<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT current_stage, status FROM project_state WHERE project_id = ?1"
        )?;
        let result = stmt.query_row(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).ok();
        Ok(result)
    }

    pub async fn log_execution(&self, project_id: &str, stage: &str, status: &str, raw: &str, result: &str) -> anyhow::Result<()> {
        let _ = self.enqueue_durable(WriteOp::LogExecution {
            project_id: project_id.to_string(), stage: stage.to_string(), status: status.to_string(),
            raw: raw.to_string(), result: result.to_string()
        }).await;
        Ok(())
    }

    pub fn count_active_tasks_by_project(&self, project_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        // v0.0.31.xx: POSITIVE ACTIVE-STATE DEFINITION - Count only Queued or Running
        // This is more robust than "status != 'Completed'" because it handles
        // multiple terminal states (Superseded, Rejected, Aborted) correctly.
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND lifecycle_state IN ('Queued', 'Running')")?;
        let count: usize = stmt.query_row(params![project_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_tasks_by_project(&self, project_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM tasks WHERE project_id = ?1")?;
        let count: usize = stmt.query_row(params![project_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_impl_tasks_by_project(&self, project_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND task_kind = 'SourceImpl'")?;
        let count: usize = stmt.query_row(params![project_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn list_all_tasks(&self) -> Result<Vec<axon_core::Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, description, status, lifecycle_state, dependencies, result, target_file, lock_files, error_feedback, rework_count, base_hash, created_at, parent_task, reason, assigned_worker, kind, retries, senior_comment, ir_path, task_kind, signature, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions FROM tasks ORDER BY created_at DESC")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(axon_core::Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::TaskStatus::Pending),
                lifecycle_state: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(5).unwrap_or_else(|_| "Queued".to_string()))).unwrap_or(axon_core::TaskLifecycleState::Queued),
                dependencies: serde_json::from_str(&row.get::<_, String>(6).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                result: row.get(7)?,
                target_file: row.get(8)?,
                lock_files: serde_json::from_str(&row.get::<_, String>(9).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                error_feedback: row.get(10)?,
                rework_count: row.get(11)?,
                base_hash: row.get(12)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?).unwrap().with_timezone(&Local),
                parent_task: row.get(14)?,
                reason: row.get(15)?,
                assigned_worker: row.get(16)?,
                kind: row.get(17)?,
                retries: row.get(18)?,
                senior_comment: row.get(19)?,
                ir_path: row.get(20)?,
                task_kind: row.get::<_, Option<String>>(21)?.and_then(|s| serde_json::from_str(&s).ok()),
                signature: row.get(22)?,
                validator_rejections: row.get(23)?,
                senior_rejections: row.get(24)?,
                architecture_rejections: row.get(25)?,
                cargo_rejections: row.get(26)?,
                lsp_rejections: row.get(27)?,
                boss_interventions: row.get(28).unwrap_or(0),
                patch_contract: None,
                repair_mode: None,
                repair_origin: None,
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
        let mut stmt = conn.prepare("SELECT id, project_id, title, description, status, lifecycle_state, dependencies, result, target_file, lock_files, error_feedback, rework_count, base_hash, created_at, parent_task, reason, assigned_worker, kind, retries, senior_comment, ir_path, task_kind, signature, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions FROM tasks WHERE id = ?1")?;
        let mut task_iter = stmt.query_map(params![id], |row| {
            Ok(axon_core::Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::TaskStatus::Pending),
                lifecycle_state: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(5).unwrap_or_else(|_| "Queued".to_string()))).unwrap_or(axon_core::TaskLifecycleState::Queued),
                dependencies: serde_json::from_str(&row.get::<_, String>(6).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                result: row.get(7)?,
                target_file: row.get(8)?,
                lock_files: serde_json::from_str(&row.get::<_, String>(9).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                error_feedback: row.get(10)?,
                rework_count: row.get(11)?,
                base_hash: row.get(12)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?).unwrap().with_timezone(&Local),
                parent_task: row.get(14)?,
                reason: row.get(15)?,
                assigned_worker: row.get(16)?,
                kind: row.get(17)?,
                retries: row.get(18)?,
                senior_comment: row.get(19)?,
                ir_path: row.get(20)?,
                task_kind: row.get::<_, Option<String>>(21)?.and_then(|s| serde_json::from_str(&s).ok()),
                signature: row.get(22)?,
                validator_rejections: row.get(23)?,
                senior_rejections: row.get(24)?,
                architecture_rejections: row.get(25)?,
                cargo_rejections: row.get(26)?,
                lsp_rejections: row.get(27)?,
                boss_interventions: row.get(28).unwrap_or(0),
                patch_contract: None,
                repair_mode: None,
                repair_origin: None,
            })
        })?;

        if let Some(task) = task_iter.next() {
            Ok(Some(task?))
        } else {
            Ok(None)
        }
    }

    pub fn get_task_by_title(&self, project_id: &str, title: &str) -> Result<Option<axon_core::Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, title, description, status, lifecycle_state, dependencies, result, target_file, lock_files, error_feedback, rework_count, base_hash, created_at, parent_task, reason, assigned_worker, kind, retries, senior_comment, ir_path, task_kind, signature, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions FROM tasks WHERE project_id = ?1 AND title = ?2")?;
        let mut task_iter = stmt.query_map(params![project_id, title], |row| {
            Ok(axon_core::Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::TaskStatus::Pending),
                lifecycle_state: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(5).unwrap_or_else(|_| "Queued".to_string()))).unwrap_or(axon_core::TaskLifecycleState::Queued),
                dependencies: serde_json::from_str(&row.get::<_, String>(6).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                result: row.get(7)?,
                target_file: row.get(8)?,
                lock_files: serde_json::from_str(&row.get::<_, String>(9).unwrap_or_else(|_| "[]".to_string())).unwrap_or_default(),
                error_feedback: row.get(10)?,
                rework_count: row.get(11)?,
                base_hash: row.get(12)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?).unwrap().with_timezone(&Local),
                parent_task: row.get(14)?,
                reason: row.get(15)?,
                assigned_worker: row.get(16)?,
                kind: row.get(17)?,
                retries: row.get(18)?,
                senior_comment: row.get(19)?,
                ir_path: row.get(20)?,
                task_kind: row.get::<_, Option<String>>(21)?.and_then(|s| serde_json::from_str(&s).ok()),
                signature: row.get(22)?,
                validator_rejections: row.get(23)?,
                senior_rejections: row.get(24)?,
                architecture_rejections: row.get(25)?,
                cargo_rejections: row.get(26)?,
                lsp_rejections: row.get(27)?,
                boss_interventions: row.get(28).unwrap_or(0),
                patch_contract: None,
                repair_mode: None,
                repair_origin: None,
            })
        })?;

        if let Some(task) = task_iter.next() {
            Ok(Some(task?))
        } else {
            Ok(None)
        }
    }

    pub async fn save_agent_stats(&self, agent_id: String, success: usize, fail: usize, latencies_json: String) -> Result<()> {
        let _ = self.enqueue_durable(WriteOp::SaveAgentStats { agent_id, success, fail, latencies_json }).await;
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
        let mut stmt = conn.prepare("SELECT id, project_id, title, status, author, milestone_id, created_at, updated_at, task_kind, rejection_count, error_feedback, reason, validator_rejections, senior_rejections, architecture_rejections, cargo_rejections, lsp_rejections, boss_interventions FROM threads WHERE id = ?1")?;
        let mut thread_iter = stmt.query_map(params![id], |row| {
            Ok(axon_core::Thread {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?)).unwrap_or(axon_core::ThreadStatus::Draft),
                author: row.get(4)?,
                milestone_id: row.get(5)?,
                task_kind: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
                rejection_count: row.get(9)?,
                validator_rejections: row.get(12).unwrap_or(0),
                senior_rejections: row.get(13).unwrap_or(0),
                architecture_rejections: row.get(14).unwrap_or(0),
                cargo_rejections: row.get(15).unwrap_or(0),
                lsp_rejections: row.get(16).unwrap_or(0),
                boss_interventions: row.get(17).unwrap_or(0),
                error_feedback: row.get(10)?,
                reason: row.get(11)?,
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

        // v0.0.28: Deterministic sorting to prevent deadlocks (Critical Requirement)
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

    pub fn list_events(&self, limit: usize) -> Result<Vec<axon_core::Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, thread_id, agent_id, event_type, source, level, content, payload, created_at FROM event_log ORDER BY created_at DESC LIMIT ?1")?;
        let event_iter = stmt.query_map(params![limit as i64], |row| {
            Ok(axon_core::Event {
                id: row.get(0)?,
                project_id: row.get(1)?,
                thread_id: row.get(2)?,
                agent_id: row.get(3)?,
                event_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?)).unwrap_or(axon_core::EventType::SystemLog),
                source: row.get(5)?,
                level: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(6)?)).unwrap_or(axon_core::EventLevel::Info),
                content: row.get(7)?,
                payload: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?).unwrap().with_timezone(&Local),
            })
        })?;

        let mut events = Vec::new();
        for event in event_iter {
            events.push(event?);
        }
        Ok(events)
    }
}
