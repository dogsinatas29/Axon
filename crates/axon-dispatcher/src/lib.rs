use axon_core::{Task, TaskStatus};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct Dispatcher {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    worker_tx: mpsc::Sender<axon_core::BatchAssignment>,
    queue_limit: usize,
}

impl Dispatcher {
    pub fn new(worker_tx: mpsc::Sender<axon_core::BatchAssignment>) -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_tx,
            queue_limit: 100,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.queue_limit = limit;
        self
    }

    pub fn enqueue_task(&self, task: Task) -> Result<usize, String> {
        let mut queue = self.task_queue.lock().unwrap();
        if queue.len() >= self.queue_limit {
            return Err("QUEUE_FULL".to_string());
        }
        queue.push_back(task);
        Ok(queue.len())
    }

    pub fn len(&self) -> usize {
        self.task_queue.lock().unwrap().len()
    }

    pub fn limit(&self) -> usize {
        self.queue_limit
    }

    pub fn pop_task(&self) -> Option<Task> {
        let mut queue = self.task_queue.lock().unwrap();
        queue.pop_front()
    }

    pub fn pop_ready_task<F>(&self, check_ready: F) -> Option<Task>
    where F: Fn(&Task) -> bool {
        let mut queue = self.task_queue.lock().unwrap();
        let mut target_idx = None;
        
        for (idx, task) in queue.iter().enumerate() {
            if check_ready(task) {
                target_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = target_idx {
            queue.remove(idx)
        } else {
            None
        }
    }

    pub async fn schedule(&self, available_agents: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.task_queue.lock().unwrap();
        
        for _agent_id in available_agents {
            if let Some(mut task) = queue.pop_front() {
                task.status = TaskStatus::InProgress;
                let batch = axon_core::Batch {
                    id: uuid::Uuid::new_v4().to_string(),
                    tasks: vec![task],
                    dependency_closure: std::collections::HashSet::new(),
                    priority: 0,
                };
                self.worker_tx.send(axon_core::BatchAssignment {
                    batch,
                }).await.map_err(|e| e.to_string())?;
            } else {
                break;
            }
        }
        Ok(())
    }
}
