use zero_downtime_deployment::*;

#[test]
fn test_full_drain_and_terminate_cycle() {
    let m = Manager::new();
    m.register_instance("web-1");
    m.mark_ready("web-1").unwrap();

    for _ in 0..3 {
        m.record_request_start("web-1").unwrap();
    }
    m.begin_drain("web-1").unwrap();
    assert!(!m.can_terminate("web-1").unwrap());

    for _ in 0..3 {
        m.record_request_end("web-1").unwrap();
    }
    assert!(m.can_terminate("web-1").unwrap());
    m.terminate("web-1").unwrap();
    assert!(m.state("web-1").is_err());
}

#[test]
fn test_multiple_instances_are_independent() {
    let m = Manager::new();
    m.register_instance("a");
    m.register_instance("b");
    m.mark_ready("a").unwrap();
    m.mark_ready("b").unwrap();
    m.begin_drain("a").unwrap();

    assert!(m.can_terminate("a").unwrap());
    assert!(!m.can_terminate("b").unwrap()); // b never started draining
}

#[test]
fn test_cannot_double_terminate() {
    let m = Manager::new();
    m.register_instance("a");
    m.mark_ready("a").unwrap();
    m.begin_drain("a").unwrap();
    m.terminate("a").unwrap();
    assert!(m.terminate("a").is_err());
}
