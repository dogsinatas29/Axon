use axon_core::{Task, TaskStatus};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct WorkerPool {
    // In a real implementation, this would hold handles to agent processes/threads
}

pub struct Dispatcher {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    worker_tx: mpsc::Sender<Assignment>,
}

pub struct Assignment {
    pub task: Task,
    pub agent_id: String,
}

impl Dispatcher {
    pub fn new(worker_tx: mpsc::Sender<Assignment>) -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_tx,
        }
    }

    pub fn enqueue_task(&self, task: Task) {
        let mut queue = self.task_queue.lock().unwrap();
        queue.push_back(task);
        // Trigger scheduling
    }

    pub async fn schedule(&self, available_agents: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.task_queue.lock().unwrap();
        
        for agent_id in available_agents {
            if let Some(mut task) = queue.pop_front() {
                task.status = TaskStatus::InProgress;
                self.worker_tx.send(Assignment {
                    task,
                    agent_id,
                }).await?;
            } else {
                break;
            }
        }
        Ok(())
    }
}
