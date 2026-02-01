//! Task executor - spawns async tasks

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct TaskExecutor {
    handles: Arc<Mutex<HashMap<Uuid, JoinHandle<()>>>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a task execution
    pub fn spawn<F>(&self, task_id: Uuid, future: F) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        let mut handles = self.handles.lock().unwrap();
        handles.insert(task_id, handle);
        Ok(())
    }

    /// Cancel a running task
    pub fn cancel(&self, task_id: &Uuid) -> bool {
        let mut handles = self.handles.lock().unwrap();
        if let Some(handle) = handles.remove(task_id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    /// Check if task is still running
    pub fn is_running(&self, task_id: &Uuid) -> bool {
        let handles = self.handles.lock().unwrap();
        handles.contains_key(task_id)
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}
