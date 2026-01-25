//! Task queue with priority sorting

use crate::Task;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

/// Wrapper for priority queue ordering
#[derive(Clone)]
struct PriorityTask(Task);

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        self.0.priority.cmp(&other.0.priority)
    }
}

/// Priority-based task queue
pub struct TaskQueue {
    queue: Arc<Mutex<BinaryHeap<PriorityTask>>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    pub fn enqueue(&self, task: Task) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(PriorityTask(task));
    }

    pub fn dequeue(&self) -> Option<Task> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop().map(|pt| pt.0)
    }

    pub fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let queue = TaskQueue::new();
        
        let low = Task::new("low").with_priority(50);
        let high = Task::new("high").with_priority(200);
        let medium = Task::new("medium").with_priority(100);
        
        queue.enqueue(low);
        queue.enqueue(high);
        queue.enqueue(medium);
        
        assert_eq!(queue.dequeue().unwrap().name, "high");
        assert_eq!(queue.dequeue().unwrap().name, "medium");
        assert_eq!(queue.dequeue().unwrap().name, "low");
    }
}
