use kitty_rc_commands::{Class, Command, FocusWindow, GetText, LaunchWait, Ls, RawFinite};
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{Completion, Failure, Outcome, Progress};
use kitty_rc_registry::{Limits, Options, Registry, Shutdown, Status};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::{Semaphore, mpsc},
    task::{JoinHandle, JoinSet},
    time::{Duration, Instant, timeout},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    path: std::path::PathBuf,
    task: JoinHandle<()>,
    seen: mpsc::Receiver<(String, Value)>,
    long: Arc<Semaphore>,
    text: Arc<Semaphore>,
    accepts: Arc<AtomicUsize>,
}
impl Fixture {
    fn new(secret: Option<x25519_dalek::StaticSecret>) -> Self {
        let path = std::path::PathBuf::from(format!(
            "/tmp/opencode/kitty-rc-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let (tx, seen) = mpsc::channel(32);
        let long = Arc::new(Semaphore::new(0));
        let text = Arc::new(Semaphore::new(0));
        let (a, b) = (long.clone(), text.clone());
        let secret = secret.map(Arc::new);
        let accepts = Arc::new(AtomicUsize::new(0));
        let counted = accepts.clone();
        let active = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            loop {
                tokio::select! {
                    connection = listener.accept() => {
                        let (mut socket, _) = connection.unwrap();
                        counted.fetch_add(1, Ordering::SeqCst);
                        let (tx, a, b, secret, active) = (tx.clone(), a.clone(), b.clone(), secret.clone(), active.clone());
                        children.spawn(async move {
                            loop {
                            let Some(mut request) = read_request(&mut socket).await else { return; };
                            if let Some(secret) = &secret {
                                let envelope = serde_json::from_value(request).unwrap();
                                request = kitty_rc_crypto::decrypt_envelope(&envelope, secret).unwrap();
                            }
                            let cmd = request["cmd"].as_str().unwrap().to_owned();
                            if cmd == "get-text" {
                                let bytes = kitty_rc_frame::frame(&serde_json::to_vec(&json!({"ok": true, "data": "x".repeat(128 * 1024)})).unwrap());
                                socket.write_all(&bytes[..4096]).await.unwrap();
                                tx.send((cmd, request)).await.unwrap();
                                b.acquire().await.unwrap().forget();
                                for chunk in bytes[4096..].chunks(511) { if socket.write_all(chunk).await.is_err() { return; } tokio::task::yield_now().await; }
                            } else {
                                tx.send((cmd.clone(), request.clone())).await.unwrap();
                                let response = match cmd.as_str() {
                                    "launch" => { let id = request["async"].as_str().unwrap().to_owned(); active.lock().unwrap().insert(id.clone()); a.acquire().await.unwrap().forget(); if !active.lock().unwrap().remove(&id) { return; } json!({"ok": true, "data": "0"}) },
                                    "ls" => json!({"ok": true, "data": "[]"}),
                                    "remote-error" => json!({"ok": false, "error": "denied"}),
                                    "bad-text" => json!({"ok": true, "data": 42}),
                                    "missing" => json!({"ok": true}),
                                    "malformed" => { let _ = socket.write_all(&kitty_rc_frame::frame(b"{" )).await; return; },
                                    "surplus" => { let mut bytes = kitty_rc_frame::frame(b"{\"ok\":true}"); bytes.extend(kitty_rc_frame::frame(b"{\"ok\":true}")); let _ = socket.write_all(&bytes).await; return; },
                                    "trickle" => { for byte in kitty_rc_frame::frame(b"{\"ok\":true}") { if socket.write_all(&[byte]).await.is_err() { return; } tokio::time::sleep(Duration::from_millis(15)).await; } return; },
                                    _ => json!({"ok": true}),
                                };
                                if socket.write_all(&kitty_rc_frame::frame(&serde_json::to_vec(&response).unwrap())).await.is_err() { return; }
                            }
                            }
                        });
                    },
                    _ = children.join_next(), if !children.is_empty() => {},
                }
            }
        });
        Self {
            path,
            task,
            seen,
            long,
            text,
            accepts,
        }
    }
    fn endpoint(&self) -> Endpoint {
        Endpoint {
            address: Address::Unix(self.path.clone()),
            protection: Protection::Plain,
        }
    }
    async fn next(&mut self) -> (String, Value) {
        timeout(Duration::from_secs(3), self.seen.recv())
            .await
            .unwrap()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn read_request(socket: &mut UnixStream) -> Option<Value> {
    let mut decoder = kitty_rc_frame::FrameDecoder::new(10 * 1024 * 1024);
    let mut b = [0; 8192];
    loop {
        let n = socket.read(&mut b).await.unwrap();
        if n == 0 {
            return None;
        }
        decoder.extend(&b[..n]).unwrap();
        if let Some(payload) = decoder.next_frame().unwrap() {
            return Some(serde_json::from_slice(&payload).unwrap());
        }
    }
}
fn options() -> Options {
    Options {
        deadline: Instant::now() + Duration::from_secs(5),
        max_response: 256 * 1024,
    }
}
fn focus() -> FocusWindow {
    FocusWindow {
        match_spec: "id:1".into(),
    }
}
fn launch() -> LaunchWait {
    LaunchWait {
        args: vec!["sh".into(), "-c".into(), "exit 0".into()],
    }
}
fn raw(name: &str) -> RawFinite {
    RawFinite {
        name: name.into(),
        payload: json!({}),
        class: Class::Short,
        submission_only: false,
    }
}

#[tokio::test]
async fn concurrent_three_connection_isolation_and_second_endpoint() {
    let mut first = Fixture::new(None);
    let mut second = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(first.endpoint()).unwrap();
    let ep2 = registry.register(second.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    let a = registry
        .submit(&registry.reserve(), ep, launch(), options())
        .unwrap();
    assert_eq!(first.next().await.0, "launch");
    let b = registry
        .submit(&registry.reserve(), ep, GetText::default(), options())
        .unwrap();
    assert_eq!(first.next().await.0, "get-text");
    let c = registry
        .submit(&registry.reserve(), ep, focus(), options())
        .unwrap();
    assert_eq!(first.next().await.0, "focus-window");
    timeout(Duration::from_secs(1), c.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(c.take().unwrap(), Completion::Reply(())));
    assert_eq!(registry.summary(a.token()).unwrap().status, Status::Running);
    assert_eq!(registry.summary(b.token()).unwrap().status, Status::Running);
    // First endpoint's observation lane is saturated; a second endpoint's
    // listing must progress while another first-endpoint observation queues.
    let queued = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    let other = registry
        .submit(&registry.reserve(), ep2, Ls::default(), options())
        .unwrap();
    assert_eq!(second.next().await.0, "ls");
    other.wait().await.unwrap();
    assert_eq!(
        registry.summary(queued.token()).unwrap().status,
        Status::Queued
    );
    assert!(matches!(other.take().unwrap(), Completion::Reply(v) if v.is_empty()));
    first.long.add_permits(1);
    first.text.add_permits(1);
    a.wait().await.unwrap();
    b.wait().await.unwrap();
    queued.wait().await.unwrap();
    assert!(matches!(a.take().unwrap(), Completion::Reply(v) if v == "0"));
    assert!(matches!(b.take().unwrap(), Completion::Reply(v) if v.len() == 128 * 1024));
    assert!(registry.shutdown(Shutdown::Drain).await.unwrap().is_empty());
    driver.stop().await;
}

#[tokio::test]
async fn waiter_loss_running_cancel_and_driver_replacement_never_replay() {
    let mut fixture = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    let reservation = registry.reserve();
    let token = reservation.token();
    drop(
        registry
            .submit(&reservation, ep, launch(), options())
            .unwrap(),
    );
    fixture.next().await;
    assert!(registry.cancel(token));
    registry.wait(token).await.unwrap();
    assert!(registry.summary(token).unwrap().unresolved);
    fixture.long.add_permits(1); // late reply belongs to retired socket
    let reading = registry
        .submit(&registry.reserve(), ep, GetText::default(), options())
        .unwrap();
    assert_eq!(fixture.next().await.0, "get-text");
    assert!(reading.cancel());
    reading.wait().await.unwrap();
    assert!(registry.summary(reading.token()).unwrap().unresolved);
    fixture.text.add_permits(1); // partial response's tail cannot be reused
    let new = registry
        .submit(&registry.reserve(), ep, focus(), options())
        .unwrap();
    fixture.next().await;
    new.wait().await.unwrap();
    assert!(matches!(new.take().unwrap(), Completion::Reply(())));
    let running = registry
        .submit(&registry.reserve(), ep, launch(), options())
        .unwrap();
    fixture.next().await;
    let queued = registry
        .submit(&registry.reserve(), ep, launch(), options())
        .unwrap();
    driver.stop().await;
    assert!(registry.summary(running.token()).unwrap().unresolved);
    assert_eq!(
        registry.summary(queued.token()).unwrap().status,
        Status::Queued
    );
    let replacement = registry.start().unwrap();
    assert_eq!(fixture.next().await.0, "launch");
    // Both old and new server handlers may finish, but old completion cannot
    // mutate the new occurrence or erase the original unknown evidence.
    fixture.long.add_permits(2);
    queued.wait().await.unwrap();
    assert!(matches!(queued.take().unwrap(), Completion::Reply(v) if v == "0"));
    assert!(registry.summary(running.token()).unwrap().unresolved);
    assert!(matches!(
        registry.take_native(token).unwrap(),
        Outcome::Interrupted { .. }
    ));
    // Explicit check after uncertain launch is a new read, not a relaunch.
    let check = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    assert_eq!(fixture.next().await.0, "ls");
    check.wait().await.unwrap();
    assert!(registry.summary(running.token()).unwrap().unresolved);
    assert_eq!(registry.shutdown(Shutdown::Drain).await.unwrap().len(), 3);
    replacement.stop().await;
}

#[tokio::test]
async fn admission_custody_byte_budgets_close_and_queued_cancel() {
    let fixture = Fixture::new(None);
    let limits = Limits {
        accepted: 2,
        accepted_per_endpoint: 2,
        output_bytes: 1024,
        output_bytes_per_endpoint: 1024,
        ..Limits::default()
    };
    let registry = Registry::new(limits).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let unused = {
        let before = registry.reserve();
        before.token()
    };
    assert!(registry.summary(unused).is_none());
    let opts = Options {
        max_response: 256,
        ..options()
    };
    let reserved = registry.reserve();
    let id = reserved.token();
    // Simulate handle-delivery loss immediately after non-awaiting commit.
    drop(registry.submit(&reserved, ep, focus(), opts).unwrap());
    assert_eq!(registry.summary(id).unwrap().status, Status::Queued);
    assert!(registry.submit(&reserved, ep, focus(), opts).is_err());
    let second = registry
        .submit(&registry.reserve(), ep, focus(), opts)
        .unwrap();
    let refused = registry
        .submit(&registry.reserve(), ep, focus(), opts)
        .err()
        .unwrap();
    assert_eq!(refused.command.match_spec, "id:1");
    assert!(registry.cancel(id));
    assert!(second.cancel());
    registry.wait(id).await.unwrap();
    assert!(!registry.summary(id).unwrap().unresolved);
    registry.take_native(id).unwrap();
    // Taking output does not silently prune the accepted record.
    assert!(
        registry
            .submit(&registry.reserve(), ep, focus(), opts)
            .is_err()
    );
    registry.release(id).unwrap();
    // The caller-owned committed occurrence cannot be replayed after release.
    assert!(registry.submit(&reserved, ep, focus(), opts).is_err());
    let third = registry
        .submit(&registry.reserve(), ep, focus(), opts)
        .unwrap();
    third.cancel();
    registry.close();
    assert!(
        registry
            .submit(&registry.reserve(), ep, focus(), opts)
            .is_err()
    );
    assert!(
        registry
            .shutdown(Shutdown::Cancel)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(registry.register(fixture.endpoint()).is_err());
}

#[tokio::test]
async fn queue_deadline_and_absolute_slow_read_deadline() {
    let mut fixture = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    let blocked = registry
        .submit(&registry.reserve(), ep, launch(), options())
        .unwrap();
    fixture.next().await;
    let expired = registry
        .submit(
            &registry.reserve(),
            ep,
            launch(),
            Options {
                deadline: Instant::now() + Duration::from_millis(30),
                ..options()
            },
        )
        .unwrap();
    timeout(Duration::from_secs(1), expired.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        expired.take(),
        Err(Failure::Interrupted {
            progress: Progress::NotWritten,
            ..
        })
    ));
    let slow = registry
        .submit(
            &registry.reserve(),
            ep,
            GetText::default(),
            Options {
                deadline: Instant::now() + Duration::from_millis(80),
                ..options()
            },
        )
        .unwrap();
    fixture.next().await;
    slow.wait().await.unwrap();
    assert!(registry.summary(slow.token()).unwrap().unresolved);
    let trickle = registry
        .submit(
            &registry.reserve(),
            ep,
            raw("trickle"),
            Options {
                deadline: Instant::now() + Duration::from_millis(80),
                ..options()
            },
        )
        .unwrap();
    trickle.wait().await.unwrap();
    assert!(matches!(
        trickle.take(),
        Err(Failure::Interrupted {
            progress: Progress::OutboundComplete,
            ..
        })
    ));
    blocked.cancel();
    registry.shutdown(Shutdown::Cancel).await.unwrap();
    driver.stop().await;
}

#[tokio::test]
async fn native_remote_error_decode_failure_submission_only_and_surplus() {
    let fixture = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    let error = registry
        .submit(&registry.reserve(), ep, raw("remote-error"), options())
        .unwrap();
    error.wait().await.unwrap();
    assert!(
        matches!(error.take(), Err(Failure::Remote { error, native }) if error == "denied" && !native.is_empty())
    );
    struct Text;
    impl Command for Text {
        type Output = String;
        fn request(&self) -> kitty_rc_schema::PlainRequest {
            kitty_rc_schema::PlainRequest::new("bad-text")
        }
        fn class(&self) -> Class {
            Class::Short
        }
        fn decode(data: Option<Value>) -> Result<String, kitty_rc_commands::DecodeError> {
            <GetText as Command>::decode(data)
        }
    }
    let bad = registry
        .submit(&registry.reserve(), ep, Text, options())
        .ok()
        .unwrap();
    bad.wait().await.unwrap();
    assert!(matches!(bad.take(), Err(Failure::TypedDecode { native, .. }) if !native.is_empty()));
    let malformed = registry
        .submit(&registry.reserve(), ep, raw("malformed"), options())
        .unwrap();
    malformed.wait().await.unwrap();
    assert!(matches!(
        malformed.take(),
        Err(Failure::MalformedReply { .. })
    ));
    assert!(registry.summary(malformed.token()).unwrap().unresolved);
    let surplus = registry
        .submit(&registry.reserve(), ep, raw("surplus"), options())
        .unwrap();
    surplus.wait().await.unwrap();
    assert!(matches!(surplus.take(), Err(Failure::Interrupted { .. })));
    let mut command = raw("submission");
    command.submission_only = true;
    let submitted = registry
        .submit(&registry.reserve(), ep, command, options())
        .unwrap();
    submitted.wait().await.unwrap();
    assert!(matches!(submitted.take().unwrap(), Completion::Submitted));
    driver.stop().await;
}

static DECODES: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug)]
struct NonCloneCommand(Box<u8>);
struct NonCloneOutput(Box<u8>);
impl Command for NonCloneCommand {
    type Output = NonCloneOutput;
    fn request(&self) -> kitty_rc_schema::PlainRequest {
        assert_eq!(*self.0, 7);
        kitty_rc_schema::PlainRequest::new("focus-window")
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn decode(_: Option<Value>) -> Result<Self::Output, kitty_rc_commands::DecodeError> {
        DECODES.fetch_add(1, Ordering::SeqCst);
        Ok(NonCloneOutput(Box::new(9)))
    }
}
#[tokio::test]
async fn nonclone_refusal_and_result_decodes_once_after_detached_wait() {
    let fixture = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    let op = registry
        .submit(
            &registry.reserve(),
            ep,
            NonCloneCommand(Box::new(7)),
            options(),
        )
        .unwrap();
    let mut wait = Box::pin(op.wait());
    tokio::select! { _ = &mut wait => {}, _ = tokio::task::yield_now() => {} }
    drop(wait);
    op.wait().await.unwrap();
    match op.take().unwrap() {
        Completion::Reply(v) => assert_eq!(*v.0, 9),
        _ => panic!(),
    }
    assert!(op.take().is_err());
    assert_eq!(DECODES.load(Ordering::SeqCst), 1);
    op.release().unwrap();
    registry.close();
    let refusal = registry
        .submit(
            &registry.reserve(),
            ep,
            NonCloneCommand(Box::new(7)),
            options(),
        )
        .err()
        .unwrap();
    assert_eq!(*refusal.command.0, 7);
    driver.stop().await;
}

#[tokio::test]
async fn encrypted_once_and_explicit_secret_safe_setup() {
    let secret = x25519_dalek::StaticSecret::from([42; 32]);
    let public = x25519_dalek::PublicKey::from(&secret);
    let key = kitty_rc_crypto::ServerPublicKey::parse(&format!(
        "1:{}",
        base85::encode(public.as_bytes())
    ))
    .unwrap();
    let mut fixture = Fixture::new(Some(secret));
    let mut endpoint = fixture.endpoint();
    endpoint.protection = Protection::Encrypted {
        key,
        password: "test-secret".into(),
        key_origin: "fixture".into(),
    };
    assert!(!format!("{endpoint:?}").contains("test-secret"));
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(endpoint).unwrap();
    let driver = registry.start().unwrap();
    let op = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    let (_, plain) = fixture.next().await;
    assert_eq!(plain["password"], "test-secret");
    assert_eq!(plain["cmd"], "ls");
    assert!(plain.get("encrypted").is_none());
    op.wait().await.unwrap();
    assert!(matches!(op.take().unwrap(), Completion::Reply(v) if v.is_empty()));
    driver.stop().await;
}

#[tokio::test]
async fn direct_future_drop_retires_transport() {
    let mut fixture = Fixture::new(None);
    let mut session = kitty_rc_tokio::Session::new(fixture.endpoint());
    let mut future = Box::pin(session.execute(launch(), options().deadline, 4096, 4096));
    tokio::select! { _ = &mut future => panic!("long wait finished"), event = fixture.next() => assert_eq!(event.0, "launch") }
    drop(future);
    fixture.long.add_permits(1);
    assert!(matches!(
        session
            .execute(focus(), options().deadline, 4096, 4096)
            .await
            .unwrap(),
        Completion::Reply(())
    ));
    assert_eq!(fixture.accepts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn cancellation_through_real_partial_unix_write_retires_socket() {
    let path = std::path::PathBuf::from(format!(
        "/tmp/opencode/kitty-rc-write-{}-{}.sock",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let listener = UnixListener::bind(&path).unwrap();
    let registry = Registry::new(Limits {
        max_request: 6 * 1024 * 1024,
        input_bytes: 32 * 1024 * 1024,
        input_bytes_per_endpoint: 32 * 1024 * 1024,
        ..Limits::default()
    })
    .unwrap();
    let ep = registry
        .register(Endpoint {
            address: Address::Unix(path.clone()),
            protection: Protection::Plain,
        })
        .unwrap();
    let driver = registry.start().unwrap();
    let command = RawFinite {
        payload: json!({"data": "x".repeat(5 * 1024 * 1024)}),
        ..raw("large-write")
    };
    let op = registry
        .submit(&registry.reserve(), ep, command, options())
        .unwrap();
    let (mut peer, _) = listener.accept().await.unwrap();
    // Not reading makes the kernel socket buffer smaller than the request.
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(
        registry.summary(op.token()).unwrap().progress,
        Progress::WriteStarted
    );
    op.cancel();
    op.wait().await.unwrap();
    let mut received = Vec::new();
    timeout(Duration::from_secs(1), peer.read_to_end(&mut received))
        .await
        .unwrap()
        .unwrap();
    assert!(!received.is_empty());
    assert!(received.len() < 5 * 1024 * 1024);
    assert!(registry.summary(op.token()).unwrap().unresolved);
    driver.stop().await;
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn concurrent_shared_submit_and_independent_input_retention_quotas() {
    let fixture = Fixture::new(None);
    let registry = Registry::new(Limits {
        output_bytes: 512,
        output_bytes_per_endpoint: 512,
        ..Limits::default()
    })
    .unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let opts = Options {
        max_response: 256,
        ..options()
    };
    let r1 = registry.clone();
    let r2 = registry.clone();
    let (a, b) = tokio::join!(
        async move { r1.submit(&r1.reserve(), ep, focus(), opts) },
        async move { r2.submit(&r2.reserve(), ep, focus(), opts) }
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    let op = a.ok().or_else(|| b.ok()).unwrap();
    op.cancel();
    op.wait().await.unwrap();
    assert!(
        registry
            .submit(&registry.reserve(), ep, focus(), opts)
            .is_err()
    );
    registry.take_native(op.token()).unwrap();
    let new = registry
        .submit(&registry.reserve(), ep, focus(), opts)
        .unwrap();
    new.cancel();
    // Input-byte admission is independent of count/output slots.
    let tight = Registry::new(Limits {
        input_bytes: 1,
        input_bytes_per_endpoint: 1,
        ..Limits::default()
    })
    .unwrap();
    let ep = tight.register(fixture.endpoint()).unwrap();
    assert!(matches!(
        tight
            .submit(&tight.reserve(), ep, focus(), opts)
            .err()
            .unwrap()
            .reason,
        kitty_rc_registry::RefusalReason::InputCapacity
    ));
}

#[tokio::test]
async fn stopped_drain_returns_unfinished_inventory_without_discarding_input() {
    let fixture = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let op = registry
        .submit(&registry.reserve(), ep, focus(), options())
        .unwrap();
    assert!(registry.shutdown(Shutdown::Drain).await.is_err());
    assert_eq!(registry.summary(op.token()).unwrap().status, Status::Queued);
    let driver = registry.start().unwrap();
    op.wait().await.unwrap();
    assert!(matches!(op.take().unwrap(), Completion::Reply(())));
    assert!(registry.shutdown(Shutdown::Drain).await.unwrap().is_empty());
    driver.stop().await;
}

#[tokio::test]
async fn driver_stop_after_drain_started_returns_instead_of_hanging() {
    let mut fixture = Fixture::new(None);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    let running = registry
        .submit(&registry.reserve(), ep, launch(), options())
        .unwrap();
    fixture.next().await;
    let queued = registry
        .submit(&registry.reserve(), ep, launch(), options())
        .unwrap();
    let mut drain = Box::pin(registry.shutdown(Shutdown::Drain));
    tokio::select! { result = &mut drain => panic!("premature {result:?}"), _ = tokio::task::yield_now() => {} }
    driver.stop().await;
    assert!(
        timeout(Duration::from_secs(1), drain)
            .await
            .unwrap()
            .is_err()
    );
    assert!(registry.summary(running.token()).unwrap().unresolved);
    assert_eq!(
        registry.summary(queued.token()).unwrap().status,
        Status::Queued
    );
}

#[tokio::test]
async fn direct_and_managed_sequential_audited_commands_reuse_one_connection() {
    let fixture = Fixture::new(None);
    let mut direct = kitty_rc_tokio::Session::new(fixture.endpoint());
    for _ in 0..4 {
        assert!(
            matches!(direct.execute(Ls::default(), options().deadline, 4096, 4096).await.unwrap(), Completion::Reply(v) if v.is_empty())
        );
    }
    assert_eq!(fixture.accepts.load(Ordering::SeqCst), 1);
    drop(direct);
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(fixture.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    for _ in 0..4 {
        let listing = registry
            .submit(&registry.reserve(), ep, Ls::default(), options())
            .unwrap();
        listing.wait().await.unwrap();
        listing.take().unwrap();
        listing.release().unwrap();
        let action = registry
            .submit(&registry.reserve(), ep, focus(), options())
            .unwrap();
        action.wait().await.unwrap();
        action.take().unwrap();
        action.release().unwrap();
        assert_eq!(registry.pool_stats().idle, 1);
        assert_eq!(registry.pool_stats().active_or_connecting, 0);
    }
    assert_eq!(fixture.accepts.load(Ordering::SeqCst), 2); // one per entrance
    driver.stop().await;
    assert_eq!(registry.pool_stats().idle, 1); // registry, not driver, owns idle
    let replacement = registry.start().unwrap();
    let action = registry
        .submit(&registry.reserve(), ep, focus(), options())
        .unwrap();
    action.wait().await.unwrap();
    action.take().unwrap();
    assert_eq!(fixture.accepts.load(Ordering::SeqCst), 2);
    registry.shutdown(Shutdown::Drain).await.unwrap();
    assert_eq!(registry.pool_stats().idle, 0);
    replacement.stop().await;
}

#[tokio::test]
async fn independent_registries_have_distinct_remote_ids_and_async_replies() {
    let mut fixture = Fixture::new(None);
    let a = Registry::new(Limits::default()).unwrap();
    let b = Registry::new(Limits::default()).unwrap();
    let ea = a.register(fixture.endpoint()).unwrap();
    let eb = b.register(fixture.endpoint()).unwrap();
    let da = a.start().unwrap();
    let db = b.start().unwrap();
    let aa = a.submit(&a.reserve(), ea, launch(), options()).unwrap();
    let ba = b.submit(&b.reserve(), eb, launch(), options()).unwrap();
    let (_, first) = fixture.next().await;
    let (_, second) = fixture.next().await;
    assert_ne!(first["async"], second["async"]);
    assert_eq!(first["async"].as_str().unwrap().len(), 64);
    fixture.long.add_permits(2);
    aa.wait().await.unwrap();
    ba.wait().await.unwrap();
    assert!(matches!(aa.take().unwrap(), Completion::Reply(v) if v == "0"));
    assert!(matches!(ba.take().unwrap(), Completion::Reply(v) if v == "0"));
    da.stop().await;
    db.stop().await;
}

#[tokio::test]
async fn queued_encrypted_input_uses_first_send_timestamp_not_admission_timestamp() {
    let secret = x25519_dalek::StaticSecret::from([43; 32]);
    let public = x25519_dalek::PublicKey::from(&secret);
    let key = kitty_rc_crypto::ServerPublicKey::parse(&format!(
        "1:{}",
        base85::encode(public.as_bytes())
    ))
    .unwrap();
    let mut fixture = Fixture::new(Some(secret));
    let mut endpoint = fixture.endpoint();
    endpoint.protection = Protection::Encrypted {
        key,
        password: "fresh-first-send".into(),
        key_origin: "fixture".into(),
    };
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(endpoint).unwrap();
    let driver = registry.start().unwrap();
    let block = registry
        .submit(&registry.reserve(), ep, GetText::default(), options())
        .unwrap();
    fixture.next().await;
    let queued = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    assert_eq!(
        registry.summary(queued.token()).unwrap().status,
        Status::Queued
    );
    tokio::time::sleep(Duration::from_millis(30)).await;
    let first_send_floor = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    fixture.text.add_permits(1);
    let (_, actual) = fixture.next().await;
    assert!(actual["timestamp"].as_u64().unwrap() >= first_send_floor);
    assert_eq!(actual["password"], "fresh-first-send");
    block.wait().await.unwrap();
    queued.wait().await.unwrap();
    queued.take().unwrap();
    assert_eq!(fixture.accepts.load(Ordering::SeqCst), 1);
    driver.stop().await;
}

#[tokio::test]
async fn structured_connect_and_preparation_errors_keep_sources_and_stage() {
    let endpoint = Endpoint {
        address: Address::Unix("/tmp/opencode/no-such-kitty-socket-for-cause-test".into()),
        protection: Protection::Plain,
    };
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry.register(endpoint).unwrap();
    let driver = registry.start().unwrap();
    let op = registry
        .submit(&registry.reserve(), ep, focus(), options())
        .unwrap();
    op.wait().await.unwrap();
    let error = op.take().unwrap_err();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .source()
            .is_some()
    );
    match error {
        Failure::Interrupted {
            progress: Progress::NotWritten,
            reason:
                kitty_rc_exchange::ExecutionError::Io {
                    phase: kitty_rc_exchange::Phase::Connect,
                    source,
                    ..
                },
        } => assert_eq!(source.kind(), std::io::ErrorKind::NotFound),
        other => panic!("{other:?}"),
    }
    let too_large = RawFinite {
        payload: json!({"data": "x".repeat(300 * 1024)}),
        ..raw("oversize")
    };
    assert!(matches!(
        registry
            .submit(&registry.reserve(), ep, too_large, options())
            .err()
            .unwrap()
            .reason,
        kitty_rc_registry::RefusalReason::Prepare(kitty_rc_exchange::PrepareError::Limit)
    ));
    driver.stop().await;
}

#[tokio::test]
async fn closed_idle_peer_is_not_replayed_and_reconnect_is_a_new_operation() {
    let path = std::path::PathBuf::from(format!(
        "/tmp/opencode/kitty-rc-idle-{}-{}.sock",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let listener = UnixListener::bind(&path).unwrap();
    let (closed_tx, closed_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut first, _) = listener.accept().await.unwrap();
        read_request(&mut first).await.unwrap();
        first
            .write_all(&kitty_rc_frame::frame(b"{\"ok\":true,\"data\":\"[]\"}"))
            .await
            .unwrap();
        first.shutdown().await.unwrap();
        drop(first);
        closed_tx.send(()).unwrap();
        let (mut second, _) = listener.accept().await.unwrap();
        read_request(&mut second).await.unwrap();
        second
            .write_all(&kitty_rc_frame::frame(b"{\"ok\":true,\"data\":\"[]\"}"))
            .await
            .unwrap();
    });
    let mut session = kitty_rc_tokio::Session::new(Endpoint {
        address: Address::Unix(path.clone()),
        protection: Protection::Plain,
    });
    session
        .execute(Ls::default(), options().deadline, 4096, 4096)
        .await
        .unwrap();
    closed_rx.await.unwrap();
    let error = session
        .execute(Ls::default(), options().deadline, 4096, 4096)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Failure::Interrupted {
            progress: Progress::NotWritten,
            reason: kitty_rc_exchange::ExecutionError::IdleClosed
        }
    ));
    assert!(!session.is_connected());
    // Caller deliberately submits a new command; no hidden replay occurred.
    session
        .execute(Ls::default(), options().deadline, 4096, 4096)
        .await
        .unwrap();
    server.await.unwrap();
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn idle_sockets_count_in_quotas_and_compatible_classes_share_them() {
    let mut first = Fixture::new(None);
    let mut second = Fixture::new(None);
    let third = Fixture::new(None);
    let fourth = Fixture::new(None);
    let registry = Registry::new(Limits {
        connections: 3,
        class_connections: [1, 1, 1],
        ..Limits::default()
    })
    .unwrap();
    let ea = registry.register(first.endpoint()).unwrap();
    let eb = registry.register(second.endpoint()).unwrap();
    let ec = registry.register(third.endpoint()).unwrap();
    let ed = registry.register(fourth.endpoint()).unwrap();
    let driver = registry.start().unwrap();
    for ep in [ea, eb, ec] {
        let op = registry
            .submit(&registry.reserve(), ep, Ls::default(), options())
            .unwrap();
        op.wait().await.unwrap();
        op.take().unwrap();
        op.release().unwrap();
    }
    assert_eq!(registry.pool_stats().idle, 3);
    // Observation-created idle sessions are compatible with long/short work.
    let long = registry
        .submit(&registry.reserve(), ea, launch(), options())
        .unwrap();
    assert_eq!(first.next().await.0, "ls");
    assert_eq!(first.next().await.0, "launch");
    let read = registry
        .submit(&registry.reserve(), eb, GetText::default(), options())
        .unwrap();
    assert_eq!(second.next().await.0, "ls");
    assert_eq!(second.next().await.0, "get-text");
    let short = registry
        .submit(&registry.reserve(), ec, focus(), options())
        .unwrap();
    short.wait().await.unwrap();
    short.take().unwrap();
    assert_eq!(first.accepts.load(Ordering::SeqCst), 1);
    assert_eq!(second.accepts.load(Ordering::SeqCst), 1);
    assert_eq!(third.accepts.load(Ordering::SeqCst), 1);
    let stats = registry.pool_stats();
    assert_eq!(stats.idle + stats.active_or_connecting, 3);
    // Full idle-inclusive budget transfers an unused incompatible route slot
    // on demand; it does not block short work behind unrelated idle resources.
    let other = registry
        .submit(&registry.reserve(), ed, focus(), options())
        .unwrap();
    other.wait().await.unwrap();
    other.take().unwrap();
    let stats = registry.pool_stats();
    assert_eq!(stats.idle + stats.active_or_connecting, 3);
    assert_eq!(stats.endpoints[&ec].0, 0);
    assert_eq!(fourth.accepts.load(Ordering::SeqCst), 1);
    assert_eq!(
        registry.summary(long.token()).unwrap().status,
        Status::Running
    );
    assert_eq!(
        registry.summary(read.token()).unwrap().status,
        Status::Running
    );
    assert_eq!(registry.shutdown(Shutdown::Cancel).await.unwrap().len(), 2);
    assert_eq!(registry.pool_stats().idle, 0);
    driver.stop().await;
}

#[tokio::test]
async fn managed_closed_idle_peer_retires_without_replaying_accepted_input() {
    let path = std::path::PathBuf::from(format!(
        "/tmp/opencode/kitty-rc-managed-idle-{}-{}.sock",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let listener = UnixListener::bind(&path).unwrap();
    let accepts = Arc::new(AtomicUsize::new(0));
    let counted = accepts.clone();
    let (closed_tx, closed_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut first, _) = listener.accept().await.unwrap();
        counted.fetch_add(1, Ordering::SeqCst);
        read_request(&mut first).await.unwrap();
        first
            .write_all(&kitty_rc_frame::frame(b"{\"ok\":true,\"data\":\"[]\"}"))
            .await
            .unwrap();
        first.shutdown().await.unwrap();
        drop(first);
        closed_tx.send(()).unwrap();
        let (mut second, _) = listener.accept().await.unwrap();
        counted.fetch_add(1, Ordering::SeqCst);
        read_request(&mut second).await.unwrap();
        second
            .write_all(&kitty_rc_frame::frame(b"{\"ok\":true,\"data\":\"[]\"}"))
            .await
            .unwrap();
    });
    let registry = Registry::new(Limits::default()).unwrap();
    let ep = registry
        .register(Endpoint {
            address: Address::Unix(path.clone()),
            protection: Protection::Plain,
        })
        .unwrap();
    let driver = registry.start().unwrap();
    let first = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    first.wait().await.unwrap();
    first.take().unwrap();
    closed_rx.await.unwrap();
    let stale = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    stale.wait().await.unwrap();
    assert!(matches!(
        stale.take(),
        Err(Failure::Interrupted {
            progress: Progress::NotWritten,
            reason: kitty_rc_exchange::ExecutionError::IdleClosed
        })
    ));
    assert_eq!(accepts.load(Ordering::SeqCst), 1);
    assert_eq!(registry.pool_stats().idle, 0);
    let deliberate_new = registry
        .submit(&registry.reserve(), ep, Ls::default(), options())
        .unwrap();
    deliberate_new.wait().await.unwrap();
    deliberate_new.take().unwrap();
    assert_eq!(accepts.load(Ordering::SeqCst), 2);
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
    server.await.unwrap();
    std::fs::remove_file(path).unwrap();
}
