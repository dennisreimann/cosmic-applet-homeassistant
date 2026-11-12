use cosmic_applet_homeassistant::refresh::RefreshFlow;

#[test]
fn new_fetch_invalidates_old_result_and_timer() {
    let mut flow = RefreshFlow::default();
    let stale = flow.start();
    let current = flow.start();
    assert!(!flow.accept_finished(stale));
    assert!(flow.accept_finished(current));
    flow.finished(current);
    assert!(flow.pending_timer());
    assert!(!flow.tick(stale));
    assert!(flow.tick(current));
}

#[test]
fn accepted_result_installs_only_one_timer() {
    let mut flow = RefreshFlow::default();
    let a = flow.start();
    assert!(flow.accept_finished(a));
    flow.finished(a);
    let b = flow.start();
    assert!(flow.accept_finished(b));
    flow.finished(b);
    assert!(!flow.tick(a));
    assert!(flow.tick(b));
}
