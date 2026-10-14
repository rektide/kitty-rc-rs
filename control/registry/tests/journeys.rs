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
        let task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            loop {
                tokio::select! {
                    connection = listener.accept() => {
                        let (mut socket, _) = connection.unwrap();
                        let (tx, a, b, secret) = (tx.clone(), a.clone(), b.clone(), secret.clone());
                        children.spawn(async move {
                            let mut request = read_request(&mut socket).await;
                            if let Some(secret) = secret {
                                let envelope = serde_json::from_value(request).unwrap();
                                request = kitty_rc_crypto::decrypt_envelope(&envelope, &secret).unwrap();
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
                                    "launch" => { a.acquire().await.unwrap().forget(); json!({"ok": true, "data": "0"}) },
                                    "ls" => json!({"ok": true, "data": "[]"}),
                                    "remote-error" => json!({"ok": false, "error": "denied"}),
                                    "bad-text" => json!({"ok": true, "data": 42}),
                                    "missing" => json!({"ok": true}),
                                    "malformed" => { let _ = socket.write_all(&kitty_rc_frame::frame(b"{" )).await; return; },
                                    "surplus" => { let mut bytes = kitty_rc_frame::frame(b"{\"ok\":true}"); bytes.extend(kitty_rc_frame::frame(b"{\"ok\":true}")); let _ = socket.write_all(&bytes).await; return; },
                                    "trickle" => { for byte in kitty_rc_frame::frame(b"{\"ok\":true}") { if socket.write_all(&[byte]).await.is_err() { return; } tokio::time::sleep(Duration::from_millis(15)).await; } return; },
                                    _ => json!({"ok": true}),
                                };
                                let _ = socket.write_all(&kitty_rc_frame::frame(&serde_json::to_vec(&response).unwrap())).await;
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
async fn read_request(socket: &mut UnixStream) -> Value {
    let mut decoder = kitty_rc_frame::FrameDecoder::new(10 * 1024 * 1024);
    let mut b = [0; 8192];
    loop {
        let n = socket.read(&mut b).await.unwrap();
        assert!(n != 0);
        decoder.extend(&b[..n]).unwrap();
        if let Some(payload) = decoder.next_frame().unwrap() {
            return serde_json::from_slice(&payload).unwrap();
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
    assert_eq!(registry.shutdown(Shutdown::Drain).await.unwrap().len(), 2);
    replacement.stop().await;
}

#[tokio::test]
async fn admission_custody_byte_budgets_close_and_queued_cancel() {
    let fixture = Fixture::new(None);
    let limits = Limits {
        accepted: 2,
        accepted_per_endpoint: 2,
        output_bytes: 512,
        output_bytes_per_endpoint: 512,
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
        fn decode(data: Option<Value>) -> Result<String, String> {
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
    fn decode(_: Option<Value>) -> Result<Self::Output, String> {
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
        input_bytes: 6 * 1024 * 1024,
        input_bytes_per_endpoint: 6 * 1024 * 1024,
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
        output_bytes: 256,
        output_bytes_per_endpoint: 256,
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
    assert!(
        tight
            .submit(&tight.reserve(), ep, focus(), opts)
            .err()
            .unwrap()
            .reason
            .contains("input-byte")
    );
}
