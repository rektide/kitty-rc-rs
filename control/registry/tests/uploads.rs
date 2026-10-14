use kitty_rc_commands::{
    FocusWindow, GetText,
    upload::{Run, UploadKey},
};
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{Completion, Failure, Progress};
use kitty_rc_registry::{
    CustodyError, LaneError, LanePolicy, Limits, Options, Registry, Shutdown, Status, UploadOptions,
};
use kitty_rc_tokio::UploadSource;
use serde_json::{Value, json};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    time::{Instant, timeout},
};
static NEXT: AtomicUsize = AtomicUsize::new(1);
struct Fixture {
    path: std::path::PathBuf,
    task: tokio::task::JoinHandle<()>,
    seen: tokio::sync::mpsc::Receiver<Value>,
}
impl Fixture {
    fn new() -> Self {
        let path = format!(
            "/tmp/opencode/kitty-managed-upload-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )
        .into();
        let listener = UnixListener::bind(&path).unwrap();
        let (tx, seen) = tokio::sync::mpsc::channel(32);
        let task = tokio::spawn(async move {
            let mut tasks = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted=listener.accept()=>{let(mut socket,_)=accepted.unwrap();let tx=tx.clone();tasks.spawn(async move {
                        loop {
                            let Some(request)=receive(&mut socket).await else{return};
                            tx.send(request.clone()).await.unwrap();
                            let reply=match request["cmd"].as_str().unwrap(){
                                "run"=>if request["payload"]["data"]!=""{json!({"ok":true,"stream":true})}else{json!({"ok":true,"data":{"stdout":"","stderr":"","exit_code":0,"exit_status":0}})},
                                "get-text"=>json!({"ok":true,"data":"independent text"}),
                                _=>json!({"ok":true}),
                            };
                            if socket.write_all(&kitty_rc_frame::frame(&serde_json::to_vec(&reply).unwrap())).await.is_err(){return;}
                        }
                    });},
                    _=tasks.join_next(),if !tasks.is_empty()=>{},
                }
            }
        });
        Self { path, task, seen }
    }
    fn endpoint(&self) -> Endpoint {
        Endpoint {
            address: Address::Unix(self.path.clone()),
            protection: Protection::Plain,
        }
    }
    async fn next(&mut self) -> Value {
        timeout(Duration::from_secs(3), self.seen.recv())
            .await
            .unwrap()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
        std::fs::remove_file(&self.path).unwrap();
    }
}
async fn receive(s: &mut UnixStream) -> Option<Value> {
    let mut d = kitty_rc_frame::FrameDecoder::new(16384);
    loop {
        let b = match s.read_u8().await {
            Ok(b) => b,
            Err(_) => return None,
        };
        d.extend(&[b]).unwrap();
        if let Some(p) = d.next_frame().unwrap() {
            return Some(serde_json::from_slice(&p).unwrap());
        }
    }
}
fn run() -> Run {
    Run::builder().cmdline(vec!["cat".into()]).build()
}
fn opts() -> Options {
    Options {
        deadline: Instant::now() + Duration::from_secs(5),
        max_response: 16384,
    }
}
fn up() -> UploadOptions {
    UploadOptions {
        operation: opts(),
        max_upload: 1024 * 1024,
    }
}
fn submit(
    registry: &Registry,
    ep: kitty_rc_registry::EndpointId,
    source: UploadSource,
) -> kitty_rc_registry::Operation<Run> {
    registry
        .submit_upload(&registry.reserve(), ep, run(), source, up())
        .map_err(|e| e.reason)
        .unwrap()
}
#[tokio::test]
async fn aliases_empty_run_exclusion_unrelated_progress_cancel_policy_and_stale_claim() {
    let mut server = Fixture::new();
    let mut other = Fixture::new();
    let registry = Registry::new(Limits::default()).unwrap();
    let a = registry.register(server.endpoint()).unwrap();
    let alias = registry.register(server.endpoint()).unwrap();
    let b = registry.register(other.endpoint()).unwrap();
    let group = registry.coordination_group(a).unwrap();
    registry.assign_alias(alias, group).unwrap();
    let driver = registry.start().unwrap();
    // A bounded AsyncRead remains pending until its writer is deliberately closed.
    let (read, mut writer) = tokio::io::duplex(4096);
    writer.write_all(&[42; 64]).await.unwrap();
    let old = submit(&registry, a, UploadSource::reader(read, 4096));
    server.next().await;
    let claim = registry.lane_claim(group, UploadKey::Run).unwrap();
    assert_eq!(claim.token, old.token());
    let queued = submit(&registry, alias, UploadSource::bytes(vec![]));
    let independent = submit(&registry, b, UploadSource::bytes(vec![]));
    other.next().await;
    independent.wait().await.unwrap();
    assert!(matches!(independent.take().unwrap(), Completion::Reply(_)));
    let text = registry
        .submit(&registry.reserve(), a, GetText::default(), opts())
        .unwrap();
    let focus = registry
        .submit(
            &registry.reserve(),
            alias,
            FocusWindow {
                match_spec: "id:1".into(),
            },
            opts(),
        )
        .unwrap();
    for _ in 0..2 {
        assert_ne!(server.next().await["cmd"], "run");
    }
    text.wait().await.unwrap();
    focus.wait().await.unwrap();
    text.take().unwrap();
    focus.take().unwrap();
    assert_eq!(
        registry.summary(queued.token()).unwrap().status,
        Status::Queued
    );
    // Credential/setup replacement cannot split the lane from the old leased input.
    registry.replace_endpoint(a, server.endpoint()).unwrap();
    assert_eq!(registry.coordination_group(a).unwrap(), group);
    old.cancel();
    old.wait().await.unwrap();
    assert!(matches!(
        old.take(),
        Err(Failure::Interrupted {
            progress: Progress::WriteStarted,
            ..
        })
    ));
    assert_eq!(
        registry.summary(queued.token()).unwrap().status,
        Status::Queued
    );
    assert_eq!(registry.lane_claim(group, UploadKey::Run), Some(claim));
    assert!(matches!(
        registry.release(old.token()),
        Err(CustodyError::UploadLaneRetained)
    ));
    registry
        .resolve_lane(claim, LanePolicy::AllowNewStreamToSupersede)
        .unwrap();
    assert_eq!(server.next().await["payload"]["data"], "");
    queued.wait().await.unwrap();
    queued.take().unwrap();
    assert!(matches!(
        registry.resolve_lane(claim, LanePolicy::AllowNewStreamToSupersede),
        Err(LaneError::Stale)
    ));
    for token in [
        old.token(),
        queued.token(),
        independent.token(),
        text.token(),
        focus.token(),
    ] {
        registry.release(token).unwrap();
    }
    drop(writer);
    registry.shutdown(Shutdown::Drain).await.unwrap();
    driver.stop().await;
}
#[tokio::test]
async fn upload_source_and_metadata_quota_and_setup_rejection_are_retained() {
    let registry = Registry::new(Limits::default()).unwrap();
    let endpoint = Endpoint {
        address: Address::Unix("/not/connected".into()),
        protection: Protection::Plain,
    };
    let ep = registry.register(endpoint).unwrap();
    let mut bytes = vec![0; 16];
    bytes.reserve_exact(5 * 1024 * 1024);
    let refused = registry
        .submit_upload(
            &registry.reserve(),
            ep,
            run(),
            UploadSource::bytes(bytes),
            up(),
        )
        .err()
        .unwrap();
    assert!(matches!(
        refused.reason,
        kitty_rc_registry::RefusalReason::InputCapacity
    ));
    assert!(refused.source.retained_bytes() > 5 * 1024 * 1024);
    let accepted = submit(&registry, ep, UploadSource::bytes(vec![7; 4096]));
    let ticket = registry.invalidate_credentials(ep).unwrap();
    registry.install_setup(ticket, Protection::Plain).unwrap();
    assert_eq!(
        registry.summary(accepted.token()).unwrap().progress,
        Progress::NotWritten
    );
    assert!(
        registry
            .summary(accepted.token())
            .unwrap()
            .setup_rejection
            .is_some()
    );
    assert!(matches!(
        registry.take_rejected_input(accepted.token()),
        Err(CustodyError::Unavailable)
    ));
    let rejected = registry.take_rejected_upload(accepted.token()).unwrap();
    assert_eq!(rejected.prepared.key(), UploadKey::Run);
    assert!(rejected.source.retained_bytes() >= 4096);
    registry.release(accepted.token()).unwrap();
}
