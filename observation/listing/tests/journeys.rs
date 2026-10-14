use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{Failure, Progress};
use kitty_rc_listing::*;
use kitty_rc_registry::{Options, Registry, Shutdown};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, oneshot},
    task::{JoinHandle, JoinSet},
    time::{Instant, timeout},
};

struct Fixture {
    endpoint: Endpoint,
    seen: mpsc::Receiver<(Value, oneshot::Sender<Value>)>,
    task: JoinHandle<()>,
}
impl Fixture {
    async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = Endpoint {
            address: Address::Tcp(listener.local_addr().unwrap()),
            protection: Protection::Plain,
        };
        let (tx, seen) = mpsc::channel(8);
        let task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            loop {
                tokio::select! {
                    connection = listener.accept() => {
                        let (mut socket, _) = connection.unwrap();
                        let tx = tx.clone();
                        children.spawn(async move {
                            loop {
                            let mut decoder = kitty_rc_frame::FrameDecoder::new(16_384);
                            let mut bytes = [0; 1024];
                            let request = loop {
                                let n = socket.read(&mut bytes).await.unwrap();
                                if n == 0 { return }
                                decoder.extend(&bytes[..n]).unwrap();
                                if let Some(frame) = decoder.next_frame().unwrap() { break serde_json::from_slice(&frame).unwrap() }
                            };
                            let (reply, response) = oneshot::channel();
                            tx.send((request, reply)).await.unwrap();
                            if let Ok(value) = response.await {
                                if socket.write_all(&kitty_rc_frame::frame(&serde_json::to_vec(&value).unwrap())).await.is_err() { return; }
                            } else { return; }
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
            task,
        }
    }
    async fn request(&mut self) -> (Value, oneshot::Sender<Value>) {
        timeout(Duration::from_secs(3), self.seen.recv())
            .await
            .unwrap()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn registry(f: &Fixture) -> (Registry, kitty_rc_registry::EndpointId) {
    let limits = kitty_rc_registry::Limits {
        connections: 2,
        connections_per_endpoint: 2,
        class_connections: [0, 2, 0],
        class_connections_per_endpoint: [0, 2, 0],
        ..Default::default()
    };
    let r = Registry::new(limits).unwrap();
    let endpoint = r.register(f.endpoint.clone()).unwrap();
    (r, endpoint)
}
fn observer(r: &Registry, endpoint: kitty_rc_registry::EndpointId, scope: Scope) -> Observer {
    Observer::new(
        r.clone(),
        Binding {
            subject: "explicit kitty instance".into(),
            endpoint,
            generation: 7,
        },
        scope,
        Interpretation {
            basis: "fixture declares focus".into(),
            focus: Support::Supported,
        },
        Limits { max_windows: 4 },
    )
}
fn options() -> Options {
    Options {
        deadline: Instant::now() + Duration::from_secs(3),
        max_response: 16_384,
    }
}
fn listing(id: u64, focus: Option<bool>) -> Value {
    let mut window = json!({"id": id, "is_active": true});
    if let Some(focus) = focus {
        window["is_focused"] = json!(focus);
    }
    json!({"ok":true,"data":json!([{"id":1,"tabs":[{"id":2,"windows":[window]}]}]).to_string()})
}

#[tokio::test]
async fn filtered_coverage_sampled_changes_and_failed_acquisition_preserve_baseline() {
    let mut f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let driver = r.start().unwrap();
    let scope = Scope {
        match_window: Some("title:editor".into()),
        match_tab: Some("id:2".into()),
        only_self: true,
        all_env_vars: false,
    };
    let mut o = observer(&r, ep, scope.clone());
    let first = o.acquire(options()).unwrap();
    let (request, reply) = f.request().await;
    assert_eq!(request["payload"]["match"], "title:editor");
    assert_eq!(request["payload"]["match_tab"], "id:2");
    assert_eq!(request["payload"]["self"], true);
    reply.send(listing(3, None)).unwrap();
    first.wait().await.unwrap();
    let first = first.finish().unwrap();
    assert_eq!(first.receipt().scope(), &scope);
    assert_eq!(first.receipt().binding().generation, 7);
    assert!(first.ended() >= first.receipt().started());
    assert!(first.coverage().filtered);
    assert_eq!(
        first.coverage().membership,
        MembershipBasis::CompleteSelectedResponse
    );
    assert_eq!(first.windows()[&3].focused, Fact::Missing);
    assert!(o.baseline().is_none());
    o.accept(first).unwrap();
    assert!(r.inventory().is_empty());

    let failed = o.acquire(options()).unwrap();
    f.request().await.1.send(json!({"ok":true})).unwrap();
    failed.wait().await.unwrap();
    assert!(matches!(
        failed.finish().unwrap_err().error,
        AcquisitionError::Native(Failure::TypedDecode { .. })
    ));
    assert_eq!(o.baseline().unwrap().windows().len(), 1);
    assert!(r.inventory().is_empty());
    let next = o.acquire(options()).unwrap();
    f.request().await.1.send(listing(4, Some(true))).unwrap();
    next.wait().await.unwrap();
    let next = next.finish().unwrap();
    let changes: Vec<_> = o.changes(&next).unwrap().collect();
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, Change::NoLongerReturned { id: 3 }))
    );
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, Change::Returned { id: 4, .. }))
    );
    assert!(o.baseline().unwrap().windows().contains_key(&3));
    o.accept(next).unwrap();
    let focus = o.acquire(options()).unwrap();
    f.request().await.1.send(listing(4, Some(false))).unwrap();
    focus.wait().await.unwrap();
    let focus = focus.finish().unwrap();
    let changes: Vec<_> = o.changes(&focus).unwrap().collect();
    assert!(
        matches!(changes.as_slice(), [Change::Changed { id: 4, before, after }] if before.focused == Fact::Known(true) && after.focused == Fact::Known(false))
    );
    o.accept(focus).unwrap();
    r.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn old_response_arrival_and_new_binding_cannot_publish() {
    let mut f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let driver = r.start().unwrap();
    let mut o = observer(&r, ep, Scope::default());
    let old = o.acquire(options()).unwrap();
    let old_reply = f.request().await.1;
    let new = o.acquire(options()).unwrap();
    f.request().await.1.send(listing(9, Some(true))).unwrap();
    new.wait().await.unwrap();
    o.accept(new.finish().unwrap()).unwrap();
    old_reply.send(listing(8, Some(false))).unwrap();
    old.wait().await.unwrap();
    assert_eq!(
        o.accept(old.finish().unwrap()).unwrap_err().0,
        Reject::Stale
    );
    assert!(o.baseline().unwrap().windows().contains_key(&9));
    let foreign = o.acquire(options()).unwrap();
    f.request().await.1.send(listing(10, Some(true))).unwrap();
    foreign.wait().await.unwrap();
    let mut replacement = Observer::new(
        r.clone(),
        Binding {
            subject: "replacement binding".into(),
            endpoint: ep,
            generation: 8,
        },
        Scope::default(),
        Interpretation {
            basis: "new access binding".into(),
            focus: Support::Supported,
        },
        Limits::default(),
    );
    assert_eq!(
        replacement.accept(foreign.finish().unwrap()).unwrap_err().0,
        Reject::ForeignOwner
    );
    r.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn dropped_acquisition_can_be_recovered_and_driver_stop_retains_uncertainty() {
    let mut f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let driver = r.start().unwrap();
    let mut o = observer(&r, ep, Scope::default());
    let a = o.acquire(options()).unwrap();
    let receipt = a.receipt().clone();
    let reply = f.request().await.1;
    // Cancel only the waiter future. The native operation still succeeds.
    assert!(timeout(Duration::from_millis(5), a.wait()).await.is_err());
    drop(a);
    reply.send(listing(1, Some(true))).unwrap();
    r.wait(receipt.token()).await.unwrap();
    o.accept(receipt.finish(&r).unwrap()).unwrap();
    assert!(r.inventory().is_empty());
    let a = o.acquire(options()).unwrap();
    let _pending_reply = f.request().await.1;
    driver.stop().await;
    a.wait().await.unwrap();
    let failure = a.finish().unwrap_err();
    assert!(matches!(
        failure.error,
        AcquisitionError::Native(Failure::Interrupted {
            progress: Progress::WriteStarted | Progress::OutboundComplete,
            ..
        })
    ));
    assert!(r.inventory().is_empty());
    assert_eq!(o.baseline().unwrap().windows().len(), 1);
}

#[tokio::test]
async fn polling_is_caller_driven_and_drop_creates_no_hidden_work() {
    let f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let mut poll =
        Poller::new(observer(&r, ep, Scope::default()), Duration::from_secs(60)).unwrap();
    assert!(r.inventory().is_empty());
    let a = poll.next(Duration::from_secs(3), 4096).await.unwrap();
    let receipt = a.receipt().clone();
    drop(a);
    assert!(receipt.poll().unwrap().scheduled_for <= receipt.started());
    assert!(
        timeout(
            Duration::from_millis(5),
            poll.next(Duration::from_secs(3), 4096)
        )
        .await
        .is_err()
    );
    assert_eq!(r.inventory().len(), 1);
    drop(poll);
    assert_eq!(r.inventory().len(), 1);
    assert!(r.cancel(receipt.token()));
    r.wait(receipt.token()).await.unwrap();
    assert!(receipt.finish(&r).is_err());
    assert!(r.inventory().is_empty());
}

#[tokio::test]
async fn partial_unsupported_and_projection_bounds_are_not_empty_authoritative_state() {
    let mut f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let driver = r.start().unwrap();
    let mut o = Observer::new(
        r.clone(),
        Binding {
            subject: "subject".into(),
            endpoint: ep,
            generation: 0,
        },
        Scope::default(),
        Interpretation {
            basis: "focus unavailable on selected route".into(),
            focus: Support::Unsupported,
        },
        Limits { max_windows: 4 },
    );
    let a = o.acquire(options()).unwrap();
    f.request().await.1.send(listing(1, None)).unwrap();
    a.wait().await.unwrap();
    let sample = a.finish().unwrap();
    assert_eq!(sample.windows()[&1].focused, Fact::Unsupported);
    o.accept(sample).unwrap();
    let old = o.acquire(options()).unwrap();
    let old_reply = f.request().await.1;
    let newest = o.acquire(options()).unwrap();
    f.request()
        .await
        .1
        .send(json!({"ok":true,"data":json!([{"id":1,"tabs":[{"id":2,"windows":[{"id":2},{"id":2}]}]}]).to_string()}))
        .unwrap();
    newest.wait().await.unwrap();
    let newest = newest.finish().unwrap();
    assert_eq!(
        newest.coverage().membership,
        MembershipBasis::PartialSelectedResponse
    );
    assert_eq!(newest.coverage().unidentified_windows, 0);
    assert!(newest.coverage().duplicate_window_ids);
    assert_eq!(newest.windows()[&2].os_window, Fact::Known(1));
    assert!(matches!(
        o.changes(&newest),
        Err(Reject::AmbiguousProjection)
    ));
    // Incomplete snapshots are still inspectable, not silently substituted by [];
    // the caller decides whether to accept their limited coverage.
    o.accept(newest).unwrap();
    old_reply.send(listing(9, Some(true))).unwrap();
    old.wait().await.unwrap();
    assert_eq!(
        o.accept(old.finish().unwrap()).unwrap_err().0,
        Reject::Stale
    );
    let excessive = o.acquire(options()).unwrap();
    f.request().await.1.send(json!({"ok":true,"data":json!([{"id":1,"tabs":[{"id":2,"windows":[{"id":1},{"id":2},{"id":3},{"id":4},{"id":5}]}]}]).to_string()})).unwrap();
    excessive.wait().await.unwrap();
    assert!(matches!(
        excessive.finish().unwrap_err().error,
        AcquisitionError::WindowLimit
    ));
    assert!(o.baseline().unwrap().coverage().duplicate_window_ids);
    for data in [
        "[{}]",
        "[{\"id\":1,\"tabs\":[{\"id\":2,\"windows\":[{}]}]}]",
    ] {
        let malformed = o.acquire(options()).unwrap();
        f.request()
            .await
            .1
            .send(json!({"ok":true,"data":data}))
            .unwrap();
        malformed.wait().await.unwrap();
        assert!(matches!(
            malformed.finish().unwrap_err().error,
            AcquisitionError::Native(Failure::TypedDecode { .. })
        ));
        assert!(o.baseline().unwrap().coverage().duplicate_window_ids);
    }
    assert!(r.inventory().is_empty());
    r.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn failed_new_generation_invalidates_old_inflight_even_without_new_baseline() {
    let mut f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let driver = r.start().unwrap();
    let mut o = observer(&r, ep, Scope::default());
    let old = o.acquire(options()).unwrap();
    let reply = f.request().await.1;
    let newest = o.acquire(options()).unwrap();
    f.request()
        .await
        .1
        .send(json!({"ok":false,"error":"denied"}))
        .unwrap();
    newest.wait().await.unwrap();
    let failure = newest.finish().unwrap_err();
    assert!(
        matches!(failure.error, AcquisitionError::Native(Failure::Remote { ref native, .. }) if !native.is_empty())
    );
    reply.send(listing(1, Some(true))).unwrap();
    old.wait().await.unwrap();
    assert_eq!(
        o.accept(old.finish().unwrap()).unwrap_err().0,
        Reject::Stale
    );
    assert!(o.baseline().is_none());
    assert!(r.inventory().is_empty());
    r.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn premature_take_returns_recoverable_receipt_not_release_or_empty_listing() {
    let f = Fixture::new().await;
    let (r, ep) = registry(&f);
    let mut o = observer(&r, ep, Scope::default());
    let a = o.acquire(options()).unwrap();
    let failure = a.finish().unwrap_err();
    assert!(matches!(
        failure.error,
        AcquisitionError::Native(Failure::Unavailable(_))
    ));
    assert_eq!(r.inventory().len(), 1);
    assert!(o.baseline().is_none());
    let receipt = failure.receipt;
    assert!(r.cancel(receipt.token()));
    r.wait(receipt.token()).await.unwrap();
    let failure = receipt.finish(&r).unwrap_err();
    assert!(matches!(
        failure.error,
        AcquisitionError::Native(Failure::Interrupted {
            progress: Progress::NotWritten,
            ..
        })
    ));
    assert!(r.inventory().is_empty());
}
