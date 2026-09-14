/// Task Scheduler

use crate::queue::Task;
use crate::Priority;
use std::collections::VecDeque;

pub struct TaskScheduler {
    critical_queue: VecDeque<Task>,
    high_queue: VecDeque<Task>,
    normal_queue: VecDeque<Task>,
    low_queue: VecDeque<Task>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            critical_queue: VecDeque::new(),
            high_queue: VecDeque::new(),
            normal_queue: VecDeque::new(),
            low_queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, task: Task) {
        match task.priority {
            Priority::Critical => self.critical_queue.push_back(task),
            Priority::High => self.high_queue.push_back(task),
            Priority::Normal => self.normal_queue.push_back(task),
            Priority::Low => self.low_queue.push_back(task),
            Priority::Background => self.low_queue.push_back(task),
        }
    }

    pub fn dequeue(&mut self) -> Option<Task> {
        // Priority: Critical > High > Normal > Low > Background
        if let Some(task) = self.critical_queue.pop_front() {
            return Some(task);
        }
        if let Some(task) = self.high_queue.pop_front() {
            return Some(task);
        }
        if let Some(task) = self.normal_queue.pop_front() {
            return Some(task);
        }
        if let Some(task) = self.low_queue.pop_front() {
            return Some(task);
        }
        None
    }

    pub fn queue_size(&self) -> usize {
        self.critical_queue.len()
            + self.high_queue.len()
            + self.normal_queue.len()
            + self.low_queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.critical_queue.is_empty()
            && self.high_queue.is_empty()
            && self.normal_queue.is_empty()
            && self.low_queue.is_empty()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_priority() {
        let mut scheduler = TaskScheduler::new();

        scheduler.enqueue(Task::new(Priority::Low, 3));
        scheduler.enqueue(Task::new(Priority::Critical, 3));
        scheduler.enqueue(Task::new(Priority::Normal, 3));

        // Critical should be first
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::Critical);
        // Normal should be second
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::Normal);
        // Low should be third
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::Low);
    }
}
