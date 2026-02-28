use crate::protocol::PacketType;
use notify::{Config, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::watch;

#[derive(Debug, Serialize, Clone)]
pub enum ThreadStatus {
    Idle,
    Running,
    Pending,
    Done,
    Hold,
}

#[derive(Debug, Serialize, Clone)]
pub struct AgentThread {
    pub id: String,
    pub title: String,
    pub status: ThreadStatus,
    pub progress: u8,
}

pub struct FactoryDaemon {
    pub status_tx: watch::Sender<PacketType>,
    pub status_rx: watch::Receiver<PacketType>,
    pub threads: Mutex<HashMap<String, AgentThread>>,
}

impl FactoryDaemon {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(PacketType::Status);

        // Mock initial threads
        let mut threads = HashMap::new();
        threads.insert(
            "thread-1".to_string(),
            AgentThread {
                id: "thread-1".to_string(),
                title: "AXP Protocol Implementation".to_string(),
                status: ThreadStatus::Running,
                progress: 45,
            },
        );

        Self {
            status_tx: tx,
            status_rx: rx,
            threads: Mutex::new(threads),
        }
    }

    pub fn pause(&self) -> Result<(), watch::error::SendError<PacketType>> {
        self.status_tx.send(PacketType::Hold)
    }

    pub fn resume(&self) -> Result<(), watch::error::SendError<PacketType>> {
        self.status_tx.send(PacketType::Resume)
    }

    pub fn get_threads(&self) -> Vec<AgentThread> {
        let threads = self.threads.lock().unwrap();
        threads.values().cloned().collect()
    }

    pub fn start_watcher(&self, path: &str) -> notify::Result<()> {
        let path = path.to_string();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        let mut watcher = notify::RecommendedWatcher::new(
            move |res| {
                let _ = tx.blocking_send(res);
            },
            Config::default(),
        )?;

        watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

        tokio::spawn(async move {
            // Keep the watcher alive by moving it into the task
            let _watcher = watcher;
            while let Some(res) = rx.recv().await {
                match res {
                    Ok(event) => {
                        if event.kind.is_modify() {
                            tracing::info!("📝 Architecture.md changed, notifying agents...");
                        }
                    }
                    Err(e) => tracing::error!("Watcher error: {:?}", e),
                }
            }
        });

        Ok(())
    }
}

pub type SharedDaemon = Arc<FactoryDaemon>;
