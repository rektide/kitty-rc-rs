use kitty_rc_check::*;
use kitty_rc_commands::{Ls, process::Launch};
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{ExecutionError, Failure, Outcome, Progress};
use kitty_rc_listing::{Binding, Interpretation, Observer, Scope, Snapshot, Support};
use kitty_rc_registry::{EndpointId, Options, Registry, Shutdown, Status};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, oneshot},
    task::{JoinHandle, JoinSet},
    time::{Instant, timeout},
};

enum Reply {
    Current,
    Value(Value),
}
#[derive(Default)]
struct Remote {
    windows: Vec<Value>,
    pending: Vec<Value>,
}
// Native finite memory backend: launch changes actual state independently of
// reply delivery. It is a control/custody fixture, not real-kitty conformance.
struct Fixture {
    endpoint: Endpoint,
    seen: mpsc::Receiver<(Value, oneshot::Sender<Reply>)>,
    remote: Arc<Mutex<Remote>>,
    launches: Arc<AtomicUsize>,
    reads: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}
impl Fixture {
    async fn new(late_effect: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = Endpoint {
            address: Address::Tcp(listener.local_addr().unwrap()),
            protection: Protection::Plain,
        };
        let (tx, seen) = mpsc::channel(16);
        let remote = Arc::new(Mutex::new(Remote::default()));
        let launches = Arc::new(AtomicUsize::new(0));
        let reads = Arc::new(AtomicUsize::new(0));
        let state = remote.clone();
        let writes = launches.clone();
        let queries = reads.clone();
        let task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            loop {
                tokio::select! {
                    connection = listener.accept() => {
                        let (mut socket, _) = connection.unwrap();
                        let tx = tx.clone();
                        let state = state.clone();
                        let writes = writes.clone();
                        let queries = queries.clone();
                        children.spawn(async move {
                            loop {
                                let mut decoder = kitty_rc_frame::FrameDecoder::new(65_536);
                                let mut bytes = [0; 1024];
                                let request: Value = loop {
                                    let n = socket.read(&mut bytes).await.unwrap();
                                    if n == 0 { return; }
                                    decoder.extend(&bytes[..n]).unwrap();
                                    if let Some(frame) = decoder.next_frame().unwrap() {
                                        break serde_json::from_slice(&frame).unwrap();
                                    }
                                };
                                match request["cmd"].as_str().unwrap() {
                                    "launch" => {
                                        let id = writes.fetch_add(1, Ordering::SeqCst) + 1;
                                        let window = json!({
                                            "id": id, "title": request["payload"]["window_title"],
                                            "pid": 100 + id, "cwd": "/fixture",
                                            "cmdline": request["payload"]["args"],
                                            "is_active": true, "is_focused": true,
                                        });
                                        let mut state = state.lock().unwrap();
                                        if late_effect { state.pending.push(window); }
                                        else { state.windows.push(window); }
                                    }
                                    "ls" => { queries.fetch_add(1, Ordering::SeqCst); }
                                    command => panic!("unexpected fixture mutation {command}"),
                                }
                                let (reply, response) = oneshot::channel();
                                if tx.send((request, reply)).await.is_err() { return; }
                                let value = match response.await {
                                    Ok(Reply::Current) => listing(state.lock().unwrap().windows.clone()),
                                    Ok(Reply::Value(value)) => value,
                                    Err(_) => return, // real effect, lost terminal reply
                                };
                                if socket.write_all(&kitty_rc_frame::frame(&serde_json::to_vec(&value).unwrap())).await.is_err() { return; }
                            }
                        });
                    },
                    _ = children.join_next(), if !children.is_empty() => {},
                }
            }
        });
        Self {
            endpoint,
            seen,
            remote,
            launches,
            reads,
            task,
        }
    }
    async fn request(&mut self, expected: &str) -> (Value, oneshot::Sender<Reply>) {
        let request = timeout(Duration::from_secs(3), self.seen.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(request.0["cmd"], expected);
        request
    }
    fn insert(&self, value: Value) {
        self.remote.lock().unwrap().windows.push(value);
    }
    fn complete_late_effect(&self) {
        let mut state = self.remote.lock().unwrap();
        let pending = std::mem::take(&mut state.pending);
        state.windows.extend(pending);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn listing(windows: Vec<Value>) -> Value {
    json!({"ok":true,"data":json!([{"id":11,"tabs":[{"id":22,"windows": windows}]}]).to_string()})
}
fn registry(f: &Fixture, accepted: usize) -> (Registry, EndpointId) {
    let registry = Registry::new(kitty_rc_registry::Limits {
        accepted,
        accepted_per_endpoint: accepted,
        connections: 2,
        connections_per_endpoint: 2,
        class_connections: [1, 1, 0],
        class_connections_per_endpoint: [1, 1, 0],
        ..Default::default()
    })
    .unwrap();
    let endpoint = registry.register(f.endpoint.clone()).unwrap();
    (registry, endpoint)
}
fn binding(endpoint: EndpointId) -> Binding {
    Binding {
        subject: "caller-selected kitty".into(),
        endpoint,
        generation: 7,
    }
}
fn interpretation() -> Interpretation {
    Interpretation {
        basis: "fixture focus capability".into(),
        focus: Support::Supported,
    }
}
fn request() -> Request {
    Request {
        scope: Scope::default(),
        interpretation: interpretation(),
        criteria: vec![Criterion::Title("trial".into())],
        condition: Condition::Unique,
        freshness: Freshness {
            max_age: Duration::from_secs(3),
        },
    }
}
fn options() -> Options {
    Options {
        deadline: Instant::now() + Duration::from_secs(3),
        max_response: 16_384,
    }
}
async fn unknown(f: &mut Fixture, registry: &Registry, endpoint: EndpointId) -> RetainedOriginal {
    let operation = registry
        .submit(
            &registry.reserve(),
            endpoint,
            Launch {
                args: vec!["fixture-program".into(), "write-body-only-secret".into()],
                window_title: Some("trial".into()),
                ..Default::default()
            },
            options(),
        )
        .unwrap();
    let token = operation.token();
    // Lose both an advisory handle and its waiter, but not accepted native work.
    drop(operation);
    let (wire, reply) = f.request("launch").await;
    assert_eq!(wire["payload"]["args"][1], "write-body-only-secret");
    drop(reply);
    registry.wait(token).await.unwrap();
    RetainedOriginal::capture(registry, token, binding(endpoint), 4096).unwrap()
}
async fn run_current(f: &mut Fixture, registry: &Registry, input: CheckInput) -> CheckReport {
    let check = input.start(registry, options()).unwrap();
    let (wire, reply) = f.request("ls").await;
    // Fresh native read contains neither launch fields nor hidden mutation data.
    assert!(wire["payload"].get("args").is_none());
    assert!(wire["payload"].get("window_title").is_none());
    assert!(!wire.to_string().contains("write-body-only-secret"));
    reply.send(Reply::Current).ok().unwrap();
    check.wait().await.unwrap();
    check.finish().unwrap()
}
fn reason(assessment: &Assessment) -> Option<Insufficient> {
    match assessment.matching {
        Matching::EvidenceInsufficient { reason, .. } => Some(reason),
        _ => None,
    }
}

#[tokio::test]
async fn lost_launch_reply_failed_check_unique_and_ambiguous_never_relaunch() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original
        .check_input(
            request(),
            Limits {
                max_candidates: 1,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!format!("{input:?}").contains("write-body-only-secret"));
    let failed = input.clone().start(&registry, options()).unwrap();
    f.request("ls")
        .await
        .1
        .send(Reply::Value(json!({"ok":false,"error":"read denied"})))
        .ok()
        .unwrap();
    failed.wait().await.unwrap();
    let failure = failed.finish().unwrap_err();
    assert!(
        matches!(&failure.failure.error, kitty_rc_listing::AcquisitionError::Native(Failure::Remote { native, .. }) if !native.is_empty())
    );
    assert_eq!(registry.inventory().len(), 1);
    assert!(
        registry
            .summary(original.boundary().token())
            .unwrap()
            .unresolved
    );
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    assert_eq!(f.reads.load(Ordering::SeqCst), 1);

    // Another check requires explicit fresh input admission, not implicit retry.
    let report = run_current(&mut f, &registry, failure.receipt.input().clone()).await;
    assert_eq!(
        report.assessment.matching,
        Matching::Unique(Candidate {
            os_window: 11,
            tab: 22,
            window: 1
        })
    );
    assert_eq!(report.assessment.satisfaction, Satisfaction::Satisfied);
    assert_eq!(report.assessment.attribution, Attribution::NotEstablished);
    assert!(matches!(
        report.assessment.native_setup,
        NativeSetup::Consistent(_)
    ));

    // A competing matching window prevents unambiguous target selection.
    f.insert(json!({"id":9,"title":"trial"}));
    let report = run_current(&mut f, &registry, input).await;
    assert!(
        matches!(report.assessment.matching, Matching::Ambiguous { count: 2, ref candidates, truncated: true } if candidates.len() == 1)
    );
    assert_eq!(report.assessment.satisfaction, Satisfaction::Unknown);
    assert_eq!(report.assessment.attribution, Attribution::NotEstablished);
    let summary = registry.summary(original.boundary().token()).unwrap();
    assert_eq!(summary.status, Status::Completed);
    assert!(summary.unresolved);
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    assert_eq!(f.reads.load(Ordering::SeqCst), 3);
    let evidence = original.take().unwrap();
    assert!(matches!(
        evidence.native,
        Outcome::Interrupted {
            progress: Progress::OutboundComplete,
            reason: ExecutionError::Eof
        }
    ));
    assert!(evidence.summary.unresolved);
    assert!(registry.inventory().is_empty());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn fresh_absence_does_not_prove_a_late_effect_is_finished() {
    let mut f = Fixture::new(true).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let mut asked = request();
    asked.condition = Condition::Absent;
    let input = original.check_input(asked, Limits::default()).unwrap();
    let before = run_current(&mut f, &registry, input.clone()).await;
    assert_eq!(before.assessment.matching, Matching::Absent);
    assert_eq!(before.assessment.satisfaction, Satisfaction::Satisfied);
    assert_eq!(before.assessment.attribution, Attribution::NotEstablished);
    // Server accepted the old launch and can execute it *after* the fresh read.
    f.complete_late_effect();
    let after = run_current(&mut f, &registry, input).await;
    assert!(matches!(after.assessment.matching, Matching::Unique(_)));
    assert_eq!(after.assessment.satisfaction, Satisfaction::Unsatisfied);
    assert_eq!(after.assessment.attribution, Attribution::NotEstablished);
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn waiter_loss_and_read_drop_preserve_both_custodies_under_quota() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 2);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original.check_input(request(), Limits::default()).unwrap();
    let check = input.clone().start(&registry, options()).unwrap();
    let receipt = check.receipt().clone();
    let reply = f.request("ls").await.1;
    assert!(
        timeout(Duration::from_millis(5), check.wait())
            .await
            .is_err()
    );
    drop(check);
    let refused = input.clone().start(&registry, options()).err().unwrap();
    assert!(matches!(
        refused.error,
        StartError::Refused(kitty_rc_registry::RefusalReason::AcceptedCapacity)
    ));
    assert_eq!(registry.inventory().len(), 2);
    reply.send(Reply::Current).ok().unwrap();
    registry.wait(receipt.token()).await.unwrap();
    assert_eq!(registry.inventory().len(), 2);
    let report = receipt.clone().finish(&registry).unwrap();
    assert_eq!(report.assessment.satisfaction, Satisfaction::Satisfied);
    assert_eq!(registry.inventory().len(), 1);
    assert!(receipt.finish(&registry).is_err()); // no repeated take/replay
    let token = original.boundary().token();
    drop(original);
    assert!(registry.summary(token).unwrap().unresolved);
    let native = registry.take_native(token).unwrap();
    registry.release(token).unwrap();
    assert!(native.unresolved());
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

fn observer(
    registry: &Registry,
    endpoint: EndpointId,
    scope: Scope,
    basis: Interpretation,
    generation: u64,
) -> Observer {
    let mut bound = binding(endpoint);
    bound.generation = generation;
    Observer::new(
        registry.clone(),
        bound,
        scope,
        basis,
        kitty_rc_listing::Limits::default(),
    )
}
async fn sample(f: &mut Fixture, observer: &mut Observer, value: Value) -> Snapshot {
    let acquisition = observer.acquire(options()).unwrap();
    f.request("ls")
        .await
        .1
        .send(Reply::Value(value))
        .ok()
        .unwrap();
    acquisition.wait().await.unwrap();
    acquisition.finish().unwrap()
}

#[tokio::test]
async fn pre_effect_acquisition_is_stale_even_if_reply_arrives_after_real_effect() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let mut observation = observer(&registry, endpoint, Scope::default(), interpretation(), 7);
    let old = observation.acquire(options()).unwrap();
    let old_reply = f.request("ls").await.1;
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original.check_input(request(), Limits::default()).unwrap();
    old_reply.send(Reply::Current).ok().unwrap();
    old.wait().await.unwrap();
    let snapshot = old.finish().unwrap();
    assert_eq!(
        snapshot.listing()[0].tabs[0].windows[0].title.as_deref(),
        Some("trial")
    );
    let assessment = input.assess(&snapshot);
    assert_eq!(
        reason(&assessment),
        Some(Insufficient::AcquiredBeforeBoundary)
    );
    assert_eq!(assessment.satisfaction, Satisfaction::Unknown);
    assert_eq!(assessment.native_setup, NativeSetup::NotChecked);
    assert_eq!(
        run_current(&mut f, &registry, input)
            .await
            .assessment
            .satisfaction,
        Satisfaction::Satisfied
    );
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn exact_binding_scope_interpretation_and_conservative_age_are_required() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original.check_input(request(), Limits::default()).unwrap();
    for (generation, scope, basis, expected) in [
        (
            8,
            Scope::default(),
            interpretation(),
            Insufficient::BindingChanged,
        ),
        (
            7,
            Scope {
                match_window: Some("id:1".into()),
                ..Default::default()
            },
            interpretation(),
            Insufficient::ScopeChanged,
        ),
        (
            7,
            Scope::default(),
            Interpretation {
                basis: "changed capability".into(),
                focus: Support::Supported,
            },
            Insufficient::InterpretationChanged,
        ),
    ] {
        let mut observation = observer(&registry, endpoint, scope, basis, generation);
        let snapshot = sample(
            &mut f,
            &mut observation,
            listing(vec![json!({"id":1,"title":"trial"})]),
        )
        .await;
        let assessment = input.assess(&snapshot);
        assert_eq!(reason(&assessment), Some(expected));
        assert_eq!(assessment.satisfaction, Satisfaction::Unknown);
    }
    let mut asked = request();
    asked.freshness.max_age = Duration::from_nanos(1);
    let expired = run_current(
        &mut f,
        &registry,
        original.check_input(asked, Limits::default()).unwrap(),
    )
    .await;
    assert_eq!(reason(&expired.assessment), Some(Insufficient::Expired));
    assert_eq!(expired.assessment.satisfaction, Satisfaction::Unknown);
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn native_setup_change_after_acquisition_is_insufficient_and_refuses_old_input() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original.check_input(request(), Limits::default()).unwrap();
    let check = input.clone().start(&registry, options()).unwrap();
    f.request("ls").await.1.send(Reply::Current).ok().unwrap();
    check.wait().await.unwrap();
    registry
        .replace_endpoint(endpoint, f.endpoint.clone())
        .unwrap();
    let report = check.finish().unwrap();
    assert_eq!(
        reason(&report.assessment),
        Some(Insufficient::NativeSetupChanged)
    );
    assert_eq!(report.assessment.satisfaction, Satisfaction::Unknown);
    assert_eq!(report.assessment.native_setup, NativeSetup::Changed);
    assert!(matches!(
        input.start(&registry, options()).err().unwrap().error,
        StartError::SetupChanged
    ));
    assert_eq!(f.reads.load(Ordering::SeqCst), 1);
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn partial_membership_and_missing_required_facts_do_not_establish_uniqueness() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original.check_input(request(), Limits::default()).unwrap();
    let mut observation = observer(&registry, endpoint, Scope::default(), interpretation(), 7);
    let partial = sample(
        &mut f,
        &mut observation,
        listing(vec![
            json!({"id":1,"title":"trial"}),
            json!({"id":1,"title":"trial"}),
        ]),
    )
    .await;
    assert_eq!(
        reason(&input.assess(&partial)),
        Some(Insufficient::PartialCoverage)
    );
    let missing = sample(
        &mut f,
        &mut observation,
        listing(vec![json!({"id":1,"title":"trial"}), json!({"id":2})]),
    )
    .await;
    let assessment = input.assess(&missing);
    assert!(matches!(
        assessment.matching,
        Matching::EvidenceInsufficient {
            reason: Insufficient::MissingRequiredFacts,
            matched: 1,
            indeterminate: 1,
            ..
        }
    ));
    assert_eq!(assessment.satisfaction, Satisfaction::Unknown);
    let mut asked = request();
    asked.criteria.insert(0, Criterion::WindowId(1));
    let precise = original.check_input(asked, Limits::default()).unwrap();
    assert!(matches!(
        precise.assess(&missing).matching,
        Matching::Unique(_)
    ));
    let bounded = original
        .check_input(
            request(),
            Limits {
                max_windows: 1,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        reason(&bounded.assess(&missing)),
        Some(Insufficient::WindowLimit)
    );
    let mut asked = request();
    asked.criteria.push(Criterion::Focused(true));
    asked.interpretation.focus = Support::Unsupported;
    let focus = original
        .check_input(asked.clone(), Limits::default())
        .unwrap();
    let mut observation = observer(
        &registry,
        endpoint,
        Scope::default(),
        asked.interpretation,
        7,
    );
    let unsupported = sample(
        &mut f,
        &mut observation,
        listing(vec![json!({"id":1,"title":"trial","is_focused":true})]),
    )
    .await;
    assert_eq!(
        reason(&focus.assess(&unsupported)),
        Some(Insufficient::MissingRequiredFacts)
    );
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn exact_filtered_read_and_native_fact_criteria_remain_scope_local() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let mut asked = request();
    asked.scope = Scope {
        match_window: Some("title:trial".into()),
        match_tab: Some("id:22".into()),
        only_self: true,
        all_env_vars: true,
    };
    asked.criteria = vec![
        Criterion::WindowId(1),
        Criterion::Pid(101),
        Criterion::Cwd("/fixture".into()),
        Criterion::UserVar {
            key: "check".into(),
            value: "marker".into(),
        },
        Criterion::Active(false),
        Criterion::Focused(false),
    ];
    let input = original.check_input(asked, Limits::default()).unwrap();
    let check = input.clone().start(&registry, options()).unwrap();
    let (wire, reply) = f.request("ls").await;
    assert_eq!(wire["payload"]["match"], "title:trial");
    assert_eq!(wire["payload"]["match_tab"], "id:22");
    assert_eq!(wire["payload"]["self"], true);
    assert_eq!(wire["payload"]["all_env_vars"], true);
    reply
        .send(Reply::Value(listing(vec![json!({
            "id":1,"pid":101,"cwd":"/fixture","user_vars":{"check":"marker"},
            "is_active":false,"is_focused":false
        })])))
        .ok()
        .unwrap();
    check.wait().await.unwrap();
    let report = check.finish().unwrap();
    assert!(report.snapshot.coverage().filtered);
    assert_eq!(report.input.request().scope, input.request().scope);
    assert!(matches!(report.assessment.matching, Matching::Unique(_)));
    assert_eq!(report.assessment.satisfaction, Satisfaction::Satisfied);
    assert_eq!(report.input.boundary().token(), original.boundary().token());
    let empty = input.start(&registry, options()).unwrap();
    f.request("ls")
        .await
        .1
        .send(Reply::Value(listing(vec![])))
        .ok()
        .unwrap();
    empty.wait().await.unwrap();
    let empty = empty.finish().unwrap();
    assert_eq!(empty.assessment.matching, Matching::Absent);
    assert_eq!(empty.assessment.satisfaction, Satisfaction::Unsatisfied);
    // Absence refers only to the exact filtered selection, not global deletion.
    assert!(empty.snapshot.coverage().filtered);
    assert_eq!(empty.assessment.attribution, Attribution::NotEstablished);
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn explicit_check_handle_cancel_interrupts_only_the_new_read() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let check = original
        .check_input(request(), Limits::default())
        .unwrap()
        .start(&registry, options())
        .unwrap();
    let _reply = f.request("ls").await.1;
    assert!(check.cancel());
    check.wait().await.unwrap();
    assert!(matches!(
        check.finish().unwrap_err().failure.error,
        kitty_rc_listing::AcquisitionError::Native(Failure::Interrupted {
            reason: ExecutionError::Cancelled { .. },
            ..
        })
    ));
    assert_eq!(registry.inventory().len(), 1);
    assert!(
        registry
            .summary(original.boundary().token())
            .unwrap()
            .unresolved
    );
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn failed_decode_projection_and_driver_stop_keep_original_evidence_separate() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    let input = original
        .check_input(
            request(),
            Limits {
                max_windows: 1,
                ..Default::default()
            },
        )
        .unwrap();
    for value in [
        json!({"ok":true}),
        listing(vec![json!({"id":1}), json!({"id":2})]),
    ] {
        let check = input.clone().start(&registry, options()).unwrap();
        f.request("ls")
            .await
            .1
            .send(Reply::Value(value))
            .ok()
            .unwrap();
        check.wait().await.unwrap();
        assert!(check.finish().is_err());
        assert_eq!(registry.inventory().len(), 1);
        assert!(
            registry
                .summary(original.boundary().token())
                .unwrap()
                .unresolved
        );
    }
    let check = input.start(&registry, options()).unwrap();
    let _reply = f.request("ls").await.1;
    driver.stop().await;
    check.wait().await.unwrap();
    let failure = check.finish().unwrap_err();
    assert!(matches!(
        failure.failure.error,
        kitty_rc_listing::AcquisitionError::Native(Failure::Interrupted {
            reason: ExecutionError::DriverStopped { .. },
            ..
        })
    ));
    assert_eq!(f.launches.load(Ordering::SeqCst), 1);
    assert!(original.take().unwrap().native.unresolved());
    assert!(registry.inventory().is_empty());
}

#[tokio::test]
async fn premature_finish_and_explicit_read_cancel_never_cancel_original() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    driver.stop().await;
    let input = original.check_input(request(), Limits::default()).unwrap();
    let check = input.start(&registry, options()).unwrap();
    let failure = check.finish().unwrap_err();
    assert!(matches!(
        failure.failure.error,
        kitty_rc_listing::AcquisitionError::Native(Failure::Unavailable(_))
    ));
    assert_eq!(registry.inventory().len(), 2);
    assert!(registry.cancel(failure.receipt.token()));
    registry.wait(failure.receipt.token()).await.unwrap();
    assert!(failure.receipt.finish(&registry).is_err());
    assert_eq!(registry.inventory().len(), 1);
    assert!(
        registry
            .summary(original.boundary().token())
            .unwrap()
            .unresolved
    );
    assert!(original.take().unwrap().native.unresolved());
}

#[tokio::test]
async fn input_metadata_and_criteria_are_bounded_and_capture_requires_uncertainty() {
    let mut f = Fixture::new(false).await;
    let (registry, endpoint) = registry(&f, 8);
    let driver = registry.start().unwrap();
    let known = registry
        .submit(&registry.reserve(), endpoint, Ls::default(), options())
        .unwrap();
    f.request("ls").await.1.send(Reply::Current).ok().unwrap();
    known.wait().await.unwrap();
    assert!(matches!(
        RetainedOriginal::capture(&registry, known.token(), binding(endpoint), 4096),
        Err(CaptureError::NotRetainedUncertain)
    ));
    known.take().unwrap();
    known.release().unwrap();
    let original = unknown(&mut f, &registry, endpoint).await;
    assert!(matches!(
        RetainedOriginal::capture(&registry, original.boundary().token(), binding(endpoint), 1),
        Err(CaptureError::SubjectLimit)
    ));
    let mut asked = request();
    asked.criteria.clear();
    assert!(matches!(
        original.check_input(asked, Limits::default()),
        Err(InputError::CriteriaLimit)
    ));
    assert!(matches!(
        original.check_input(
            request(),
            Limits {
                max_criteria: 0,
                ..Default::default()
            }
        ),
        Err(InputError::CriteriaLimit)
    ));
    assert!(matches!(
        original.check_input(
            request(),
            Limits {
                max_metadata_bytes: 1,
                ..Default::default()
            }
        ),
        Err(InputError::MetadataLimit)
    ));
    assert!(matches!(
        original.check_input(
            request(),
            Limits {
                max_candidates: 0,
                ..Default::default()
            }
        ),
        Err(InputError::ZeroLimit)
    ));
    let mut asked = request();
    asked.freshness.max_age = Duration::ZERO;
    assert!(matches!(
        original.check_input(asked, Limits::default()),
        Err(InputError::ZeroFreshness)
    ));
    assert!(original.take().unwrap().native.unresolved());
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}
