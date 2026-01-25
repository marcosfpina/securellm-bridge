//! Task Management System for SecureLLM Bridge
//!
//! Provides asynchronous task orchestration with priority queuing,
//! progress tracking, and SQLite persistence.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

mod executor;
mod persistence;
mod queue;

pub use executor::TaskExecutor;
pub use persistence::TaskStore;
pub use queue::TaskQueue;

/// Task execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskState {
    /// Task is queued but not started
    Pending,
    
    /// Task is currently executing
    Running {
        started_at: DateTime<Utc>,
    },
    
    /// Task completed successfully
    Completed {
        duration: Duration,
        result: serde_json::Value,
    },
    
    /// Task failed with error
    Failed {
        error: String,
        duration: Duration,
    },
    
    /// Task was cancelled
    Cancelled {
        reason: Option<String>,
    },
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskState::Completed { .. } | TaskState::Failed { .. } | TaskState::Cancelled { .. })
    }

    pub fn is_running(&self) -> bool {
        matches!(self, TaskState::Running { .. })
    }
}

/// Represents a managed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: Uuid,
    
    /// Human-readable task name
    pub name: String,
    
    /// Task description
    pub description: Option<String>,
    
    /// Priority (0-255, higher = more important)
    pub priority: u8,
    
    /// Current execution state
    pub state: TaskState,
    
    /// Progress percentage (0.0 - 1.0)
    pub progress: f32,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
}

impl Task {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            priority: 128, // Default medium priority
            state: TaskState::Pending,
            progress: 0.0,
            created_at: now,
            updated_at: now,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
    }

    pub fn transition_to(&mut self, state: TaskState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
}

/// Central task management system
pub struct TaskManager {
    tasks: Arc<DashMap<Uuid, Task>>,
    queue: TaskQueue,
    executor: TaskExecutor,
    store: Option<TaskStore>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            queue: TaskQueue::new(),
            executor: TaskExecutor::new(),
            store: None,
        }
    }

    pub async fn with_persistence(db_path: &str) -> Result<Self> {
        let store = TaskStore::new(db_path).await?;
        Ok(Self {
            tasks: Arc::new(DashMap::new()),
            queue: TaskQueue::new(),
            executor: TaskExecutor::new(),
            store: Some(store),
        })
    }

    /// Submit a new task to the queue
    pub async fn submit(&self, task: Task) -> Result<Uuid> {
        let task_id = task.id;
        
        // Insert into in-memory map
        self.tasks.insert(task_id, task.clone());
        
        // Persist if enabled
        if let Some(store) = &self.store {
            store.save_task(&task).await?;
        }
        
        // Add to execution queue
        self.queue.enqueue(task);
        
        tracing::info!(task_id = %task_id, "Task submitted");
        Ok(task_id)
    }

    /// Get task by ID
    pub fn get(&self, task_id: &Uuid) -> Option<Task> {
        self.tasks.get(task_id).map(|entry| entry.clone())
    }

    /// Cancel a task
    pub async fn cancel(&self, task_id: &Uuid, reason: Option<String>) -> Result<()> {
        if let Some(mut task) = self.tasks.get_mut(task_id) {
            if !task.state.is_terminal() {
                task.transition_to(TaskState::Cancelled { reason: reason.clone() });
                
                if let Some(store) = &self.store {
                    store.save_task(&task).await?;
                }
                
                tracing::info!(task_id = %task_id, "Task cancelled");
            }
        }
        Ok(())
    }

    /// List all tasks matching filter
    pub fn list(&self, filter: Option<TaskFilter>) -> Vec<Task> {
        self.tasks
            .iter()
            .filter(|entry| {
                if let Some(ref f) = filter {
                    f.matches(entry.value())
                } else {
                    true
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Update task progress
    pub async fn update_progress(&self, task_id: &Uuid, progress: f32) -> Result<()> {
        if let Some(mut task) = self.tasks.get_mut(task_id) {
            task.update_progress(progress);
            
            if let Some(store) = &self.store {
                store.save_task(&task).await?;
            }
        }
        Ok(())
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter for querying tasks
#[derive(Debug, Clone)]
pub struct TaskFilter {
    pub states: Option<Vec<TaskState>>,
    pub min_priority: Option<u8>,
    pub name_contains: Option<String>,
}

impl TaskFilter {
    pub fn matches(&self, task: &Task) -> bool {
        if let Some(ref states) = self.states {
            if !states.iter().any(|s| std::mem::discriminant(s) == std::mem::discriminant(&task.state)) {
                return false;
            }
        }

        if let Some(min_prio) = self.min_priority {
            if task.priority < min_prio {
                return false;
            }
        }

        if let Some(ref name_pat) = self.name_contains {
            if !task.name.contains(name_pat) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_creation() {
        let task = Task::new("test_task")
            .with_priority(200)
            .with_description("A test task");

        assert_eq!(task.name, "test_task");
        assert_eq!(task.priority, 200);
        assert!(matches!(task.state, TaskState::Pending));
    }

    #[tokio::test]
    async fn test_task_submission() {
        let manager = TaskManager::new();
        let task = Task::new("test_submit");
        
        let task_id = manager.submit(task).await.unwrap();
        let retrieved = manager.get(&task_id).unwrap();
        
        assert_eq!(retrieved.name, "test_submit");
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let manager = TaskManager::new();
        let task = Task::new("test_cancel");
        
        let task_id = manager.submit(task).await.unwrap();
        manager.cancel(&task_id, Some("Test cancellation".to_string())).await.unwrap();
        
        let retrieved = manager.get(&task_id).unwrap();
        assert!(matches!(retrieved.state, TaskState::Cancelled { .. }));
    }
}
