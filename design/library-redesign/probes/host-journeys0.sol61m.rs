// Independent comparison tests appended to kubeless's own host test fixture.
// Fixture helpers are read from the explicitly supplied checkout by the runner,
// never vendored into the kitty production library.

#[test]
fn kitty_three_independent_wait_read_short_candidates() {
    let shared = shared();
    let (long_tx, long_rx) = oneshot::channel();
    let (read_tx, read_rx) = oneshot::channel();
    install(&shared, "launch", "long", Some(long_rx), None);
    install(&shared, "text", "observation", Some(read_rx), None);
    install(&shared, "focus", "short", None, None);
    let work_shared = shared.clone();
    let mut driver = attach(shared.clone(), Options {
        work: Capacity::Limited(std::num::NonZeroUsize::new(3).unwrap()),
        ..Options::default()
    }, key, move |w| execute(work_shared.clone(), w), disposition);
    let wakes = Arc::new(CountWake::default());
    let mut returned = Vec::new();
    drain(&mut driver, &wakes, |event| {
        if let Event::Work { result, .. } = event {
            returned.push(result.ok().unwrap().ok().unwrap().native.label);
        }
    });
    assert_eq!(returned, vec!["focus"]);
    shared.turn(|s| assert_eq!(s.starts.get(), 3));
    long_tx.send(()).unwrap(); read_tx.send(()).unwrap();
    drain(&mut driver, &wakes, |event| {
        if let Event::Work { result, .. } = event {
            returned.push(result.ok().unwrap().ok().unwrap().native.label);
        }
    });
    assert_eq!(returned.len(), 3);
}

#[test]
fn kitty_lost_launch_then_explicit_check_without_replay() {
    let shared = shared();
    let (_tx, rx) = oneshot::channel();
    install(&shared, "launch", "mutation", Some(rx), None);
    let work_shared = shared.clone();
    let mut driver = attach(shared.clone(), Options::default(), key,
        move |w| execute(work_shared.clone(), w), disposition);
    let wakes = Arc::new(CountWake::default());
    drain(&mut driver, &wakes, |_| {});
    drop(driver);
    let old = shared.turn(|s| {
        assert_eq!(s.starts.get(), 1);
        let run = s.owner.runs().next().unwrap();
        assert!(run.returned().is_none());
        run.id()
    });
    install(&shared, "ls", "check-only", None, None);
    let work_shared = shared.clone();
    let mut replacement = attach(shared.clone(), Options::default(), key,
        move |w| execute(work_shared.clone(), w), disposition);
    let mut labels = Vec::new();
    drain(&mut replacement, &wakes, |event| {
        if let Event::Work { result, .. } = event {
            labels.push(result.ok().unwrap().ok().unwrap().native.label);
        }
    });
    assert_eq!(labels, vec!["ls"]);
    shared.turn(|s| {
        assert_eq!(s.starts.get(), 2);
        let custody::Lookup::Present(run) = s.owner.run(old) else { panic!() };
        assert!(run.returned().is_none());
        assert!(!run.notes().is_empty());
    });
}
