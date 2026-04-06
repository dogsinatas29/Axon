use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SystemState {
    Running,
    Paused,
    Terminated,
}

pub struct ControlSystem {
    tx: Arc<watch::Sender<SystemState>>,
    rx: watch::Receiver<SystemState>,
}

impl ControlSystem {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(SystemState::Running);
        Self {
            tx: Arc::new(tx),
            rx,
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<SystemState> {
        self.rx.clone()
    }

    pub fn pause(&self) {
        let _ = self.tx.send(SystemState::Paused);
        info!("🔴 System PAUSED by ControlSystem");
    }

    pub fn resume(&self) {
        let _ = self.tx.send(SystemState::Running);
        info!("🟢 System RESUMED by ControlSystem");
    }

    pub fn terminate(&self) {
        let _ = self.tx.send(SystemState::Terminated);
        info!("⏹️ System TERMINATED by ControlSystem");
    }

    pub fn current_state(&self) -> SystemState {
        *self.rx.borrow()
    }
}
