/// Task Queue System

use crate::Priority;
use dashmap::DashMap;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub priority: Priority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl Task {
    pub fn new(priority: Priority, max_retries: u32) -> Self {
        Task {
            id: Uuid::new_v4().to_string(),
            priority,
            created_at: chrono::Utc::now(),
            deadline: None,
            retry_count: 0,
            max_retries,
        }
    }

    pub fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            chrono::Utc::now() > deadline
        } else {
            false
        }
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
    }
}

pub struct TaskQueue {
    queue: Arc<parking_lot::Mutex<BinaryHeap<Reverse<Task>>>>,
    task_metadata: Arc<DashMap<String, TaskMetadata>>,
    max_size: usize,
}

#[derive(Clone, Debug)]
struct TaskMetadata {
    status: TaskStatus,
    assigned_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskQueue {
    pub fn new(max_size: usize) -> Self {
        TaskQueue {
            queue: Arc::new(parking_lot::Mutex::new(BinaryHeap::new())),
            task_metadata: Arc::new(DashMap::new()),
            max_size,
        }
    }

    pub fn enqueue(&self, task: Task) -> anyhow::Result<()> {
        let mut queue = self.queue.lock();

        if queue.len() >= self.max_size {
            return Err(anyhow::anyhow!("Queue full"));
        }

        self.task_metadata.insert(
            task.id.clone(),
            TaskMetadata {
                status: TaskStatus::Pending,
                assigned_at: None,
            },
        );

        queue.push(Reverse(task));
        Ok(())
    }

    pub fn dequeue(&self) -> Option<Task> {
        let mut queue = self.queue.lock();

        if let Some(Reverse(task)) = queue.pop() {
            if let Some(mut metadata) = self.task_metadata.get_mut(&task.id) {
                metadata.status = TaskStatus::Running;
                metadata.assigned_at = Some(chrono::Utc::now());
            }
            Some(task)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }

    pub fn mark_completed(&self, task_id: &str) {
        if let Some(mut metadata) = self.task_metadata.get_mut(task_id) {
            metadata.status = TaskStatus::Completed;
        }
    }

    pub fn mark_failed(&self, task_id: &str) {
        if let Some(mut metadata) = self.task_metadata.get_mut(task_id) {
            metadata.status = TaskStatus::Failed;
        }
    }

    pub fn get_task_status(&self, task_id: &str) -> Option<String> {
        self.task_metadata.get(task_id).map(|m| format!("{:?}", m.status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(Priority::Normal, 3);
        assert_eq!(task.priority, Priority::Normal);
        assert_eq!(task.max_retries, 3);
    }

    #[test]
    fn test_task_queue() {
        let queue = TaskQueue::new(100);
        let task = Task::new(Priority::High, 3);
        assert!(queue.enqueue(task).is_ok());
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_queue_overflow() {
        let queue = TaskQueue::new(1);
        let task1 = Task::new(Priority::High, 3);
        let task2 = Task::new(Priority::Normal, 3);

        assert!(queue.enqueue(task1).is_ok());
        assert!(queue.enqueue(task2).is_err());
    }
}
