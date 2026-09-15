use crate::error::{Error, Result};
use crate::types::BatchResult;
use std::sync::RwLock;

/// Rolls a fleet of `total_instances` forward in batches of at most
/// `max_unavailable` instances at a time, never taking more than that many
/// instances out of service simultaneously.
#[derive(Debug)]
pub struct Manager {
    total_instances: usize,
    max_unavailable: usize,
    updated: RwLock<usize>,
    in_progress: RwLock<usize>,
    paused: RwLock<bool>,
}

impl Manager {
    pub fn new(total_instances: usize, max_unavailable: usize) -> Result<Self> {
        if max_unavailable == 0 {
            return Err(Error::InvalidConfig("max_unavailable must be at least 1".into()));
        }
        if total_instances == 0 {
            return Err(Error::InvalidConfig("total_instances must be at least 1".into()));
        }
        Ok(Self {
            total_instances,
            max_unavailable,
            updated: RwLock::new(0),
            in_progress: RwLock::new(0),
            paused: RwLock::new(false),
        })
    }

    pub fn updated_count(&self) -> usize {
        *self.updated.read().unwrap()
    }

    pub fn remaining_count(&self) -> usize {
        self.total_instances - self.updated_count()
    }

    /// Size of the next batch, respecting `max_unavailable` and the number
    /// of instances left.
    pub fn next_batch_size(&self) -> usize {
        self.max_unavailable.min(self.remaining_count())
    }

    /// Take the next batch out of service. Fails if paused, if a batch is
    /// already in flight, or if the rollout is already complete.
    pub fn start_batch(&self) -> Result<usize> {
        if *self.paused.read().unwrap() {
            return Err(Error::BatchNotStartable("controller is paused".into()));
        }
        let mut in_progress = self.in_progress.write().unwrap();
        if *in_progress > 0 {
            return Err(Error::BatchNotStartable("a batch is already in progress".into()));
        }
        let size = self.next_batch_size();
        if size == 0 {
            return Err(Error::BatchNotStartable("rollout already complete".into()));
        }
        *in_progress = size;
        Ok(size)
    }

    /// Mark the in-flight batch as successfully updated.
    pub fn complete_batch(&self) -> Result<BatchResult> {
        let mut in_progress = self.in_progress.write().unwrap();
        if *in_progress == 0 {
            return Err(Error::NoBatchInProgress);
        }
        let mut updated = self.updated.write().unwrap();
        *updated += *in_progress;
        *in_progress = 0;
        if *updated >= self.total_instances {
            Ok(BatchResult::Done)
        } else {
            Ok(BatchResult::Remaining(self.total_instances - *updated))
        }
    }

    /// A batch failed its health check: bring it back into service without
    /// counting it as updated.
    pub fn rollback_batch(&self) -> Result<()> {
        let mut in_progress = self.in_progress.write().unwrap();
        if *in_progress == 0 {
            return Err(Error::NoBatchInProgress);
        }
        *in_progress = 0;
        Ok(())
    }

    pub fn pause(&self) {
        *self.paused.write().unwrap() = true;
    }

    pub fn resume(&self) {
        *self.paused.write().unwrap() = false;
    }

    pub fn is_paused(&self) -> bool {
        *self.paused.read().unwrap()
    }

    pub fn is_complete(&self) -> bool {
        self.updated_count() >= self.total_instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_max_unavailable() {
        assert!(matches!(Manager::new(10, 0).unwrap_err(), Error::InvalidConfig(_)));
    }

    #[test]
    fn batches_are_sized_by_max_unavailable_and_remainder() {
        let m = Manager::new(10, 3).unwrap();
        assert_eq!(m.next_batch_size(), 3);
        assert_eq!(m.start_batch().unwrap(), 3);
        assert_eq!(m.complete_batch().unwrap(), BatchResult::Remaining(7));

        assert_eq!(m.start_batch().unwrap(), 3);
        assert_eq!(m.complete_batch().unwrap(), BatchResult::Remaining(4));

        assert_eq!(m.start_batch().unwrap(), 3);
        assert_eq!(m.complete_batch().unwrap(), BatchResult::Remaining(1));

        // final, short batch
        assert_eq!(m.start_batch().unwrap(), 1);
        assert_eq!(m.complete_batch().unwrap(), BatchResult::Done);
        assert!(m.is_complete());
    }

    #[test]
    fn cannot_start_two_batches_concurrently() {
        let m = Manager::new(10, 3).unwrap();
        m.start_batch().unwrap();
        assert!(matches!(m.start_batch().unwrap_err(), Error::BatchNotStartable(_)));
    }

    #[test]
    fn pause_blocks_new_batches() {
        let m = Manager::new(10, 3).unwrap();
        m.pause();
        assert!(m.is_paused());
        assert!(matches!(m.start_batch().unwrap_err(), Error::BatchNotStartable(_)));
        m.resume();
        assert_eq!(m.start_batch().unwrap(), 3);
    }

    #[test]
    fn rollback_keeps_updated_count_unchanged() {
        let m = Manager::new(10, 3).unwrap();
        m.start_batch().unwrap();
        m.rollback_batch().unwrap();
        assert_eq!(m.updated_count(), 0);
        assert_eq!(m.next_batch_size(), 3);
    }

    #[test]
    fn completing_with_no_batch_in_progress_errors() {
        let m = Manager::new(10, 3).unwrap();
        assert!(matches!(m.complete_batch().unwrap_err(), Error::NoBatchInProgress));
    }

    #[test]
    fn starting_after_completion_errors() {
        let m = Manager::new(2, 2).unwrap();
        m.start_batch().unwrap();
        m.complete_batch().unwrap();
        assert!(m.is_complete());
        assert!(matches!(m.start_batch().unwrap_err(), Error::BatchNotStartable(_)));
    }
}
