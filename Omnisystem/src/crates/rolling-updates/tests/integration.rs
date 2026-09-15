use rolling_updates::*;

#[test]
fn test_full_rollout_uneven_batches() {
    let m = Manager::new(11, 4).unwrap();
    let mut total = 0;
    loop {
        let size = match m.start_batch() {
            Ok(s) => s,
            Err(_) => break,
        };
        total += size;
        if m.complete_batch().unwrap() == BatchResult::Done {
            break;
        }
    }
    assert_eq!(total, 11);
    assert!(m.is_complete());
}

#[test]
fn test_pause_then_resume_mid_rollout() {
    let m = Manager::new(6, 2).unwrap();
    m.start_batch().unwrap();
    m.complete_batch().unwrap();
    m.pause();
    assert!(m.start_batch().is_err());
    m.resume();
    assert_eq!(m.start_batch().unwrap(), 2);
}

#[test]
fn test_failed_batch_is_retried_without_progress() {
    let m = Manager::new(4, 2).unwrap();
    m.start_batch().unwrap();
    m.rollback_batch().unwrap();
    assert_eq!(m.updated_count(), 0);
    m.start_batch().unwrap();
    assert_eq!(m.complete_batch().unwrap(), BatchResult::Remaining(2));
}
