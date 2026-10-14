use kitty_rc_commands::{
    Command,
    upload::{BackgroundPng, LogoPng, Run, UploadSubmission},
};
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{
    Completion, Evidence, ExecutionError, Failure, Outcome, Progress, prepare_upload,
};
use kitty_rc_tokio::{Session, SessionOptions, UploadOptions, UploadSource};
use serde_json::{Value, json};
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf},
    net::{UnixListener, UnixStream},
    time::Instant,
};
static NEXT: AtomicUsize = AtomicUsize::new(1);
struct Peer {
    path: std::path::PathBuf,
    listener: UnixListener,
}
impl Peer {
    fn new() -> Self {
        let path = format!(
            "/tmp/opencode/kitty-upload-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )
        .into();
        Self {
            listener: UnixListener::bind(&path).unwrap(),
            path,
        }
    }
    fn session(&self) -> Session {
        Session::new(Endpoint {
            address: Address::Unix(self.path.clone()),
            protection: Protection::Plain,
        })
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).unwrap();
    }
}
fn options() -> UploadOptions {
    UploadOptions {
        session: SessionOptions {
            deadline: Instant::now() + Duration::from_secs(3),
            max_request: 16384,
            max_response: 16384,
        },
        max_upload: 1024 * 1024,
    }
}
fn run() -> Run {
    Run::builder()
        .cmdline(vec!["literal 🦀".into(), "$HOME".into()])
        .build()
}
async fn receive(socket: &mut UnixStream) -> Value {
    let mut d = kitty_rc_frame::FrameDecoder::new(16384);
    loop {
        let byte = socket.read_u8().await.unwrap();
        d.extend(&[byte]).unwrap();
        if let Some(frame) = d.next_frame().unwrap() {
            return serde_json::from_slice(&frame).unwrap();
        }
    }
}
async fn send(socket: &mut UnixStream, reply: Value) {
    socket
        .write_all(&kitty_rc_frame::frame(&serde_json::to_vec(&reply).unwrap()))
        .await
        .unwrap();
}
struct Probe {
    bytes: Vec<u8>,
    pos: usize,
    calls: Arc<AtomicUsize>,
    max: Arc<AtomicUsize>,
    fail: bool,
    pending: bool,
}
impl AsyncRead for Probe {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.max.fetch_max(buf.remaining(), Ordering::Relaxed);
        if self.pos == self.bytes.len() {
            if self.fail {
                return Poll::Ready(Err(std::io::Error::other("producer exact cause")));
            }
            if self.pending {
                return Poll::Pending;
            }
            return Poll::Ready(Ok(()));
        }
        let n = buf.remaining().min(self.bytes.len() - self.pos);
        buf.put_slice(&self.bytes[self.pos..self.pos + n]);
        self.pos += n;
        Poll::Ready(Ok(()))
    }
}
fn probe(
    bytes: Vec<u8>,
    fail: bool,
    pending: bool,
) -> (UploadSource, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let retained = bytes.capacity();
    let calls = Arc::new(AtomicUsize::new(0));
    let max = Arc::new(AtomicUsize::new(0));
    (
        UploadSource::reader(
            Probe {
                bytes,
                pos: 0,
                calls: calls.clone(),
                max: max.clone(),
                fail,
                pending,
            },
            retained,
        ),
        calls,
        max,
    )
}

#[tokio::test]
async fn run_empty_small_exact_large_preserves_bytes_metadata_ids_and_one_eof() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    for size in [0usize, 1, 4096, 4097, 16384] {
        let peer = Peer::new();
        let mut session = peer.session();
        let bytes: Vec<_> = (0..size).map(|i| (i % 256) as u8).collect();
        let expected = bytes.clone();
        let (source, calls, max) = probe(bytes, false, false);
        let demanded = calls.clone();
        let task = tokio::spawn(async move {
            let (mut socket, _) = peer.listener.accept().await.unwrap();
            let first = receive(&mut socket).await;
            let sid = first["stream_id"].clone();
            let aid = first["async"].clone();
            assert!(
                sid.as_str().unwrap().len() == 64
                    && aid.as_str().unwrap().len() == 64
                    && sid != aid
            );
            let mut frame = first;
            let mut restored = Vec::new();
            let mut count = 0;
            loop {
                assert_eq!(frame["payload"]["cmdline"], json!(["literal 🦀", "$HOME"]));
                assert_eq!(frame["stream_id"], sid);
                assert_eq!(frame["async"], aid);
                let data = frame["payload"]["data"].as_str().unwrap();
                if data.is_empty() {
                    assert!(frame.get("stream").is_none());
                    break;
                }
                let raw = STANDARD.decode(data).unwrap();
                assert!(raw.len() <= 4096);
                restored.extend(raw);
                if count == 0 {
                    assert_eq!(frame["stream"], true);
                    assert_eq!(demanded.load(Ordering::Relaxed), 1);
                    send(&mut socket, json!({"ok":true,"stream":true})).await;
                } else {
                    assert!(frame.get("stream").is_none());
                }
                count += 1;
                frame = receive(&mut socket).await;
            }
            assert_eq!(restored, expected);
            assert_eq!(count, size.div_ceil(4096));
            send(&mut socket,json!({"ok":true,"data":{"stdout":"AP+YgA==","stderr":"AP8=","exit_code":7,"exit_status":1792}})).await;
            // A reusable connection sends nothing further without a new command.
        });
        let result = session.upload(run(), source, options()).await.unwrap();
        let Completion::Reply(output) = result else {
            panic!()
        };
        assert_eq!(output.stdout, [0, 255, 152, 128]);
        assert_eq!(output.stderr, [0, 255]);
        assert_eq!(output.exit_code, 7);
        assert!(max.load(Ordering::Relaxed) <= 4096);
        task.await.unwrap();
    }
}
#[tokio::test]
async fn png_profiles_use_own_bound_and_submission_still_waits_for_handshake() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    for (logo, size) in [false, true].into_iter().flat_map(|logo| {
        let bound = if logo { 2048 } else { 512 };
        [8, bound, bound + 1, bound * 2]
            .into_iter()
            .map(move |size| (logo, size))
    }) {
        let peer = Peer::new();
        let mut session = peer.session();
        let bound = if logo { 2048 } else { 512 };
        let mut bytes = vec![42; size];
        bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        let expected = bytes.clone();
        let (source, calls, max) = probe(bytes, false, false);
        let task = tokio::spawn(async move {
            let (mut s, _) = peer.listener.accept().await.unwrap();
            let first = receive(&mut s).await;
            assert_eq!(calls.load(Ordering::Relaxed), 1);
            assert_eq!(first["stream"], true);
            assert_eq!(first["no_response"], true);
            tokio::time::sleep(Duration::from_millis(20)).await;
            assert_eq!(calls.load(Ordering::Relaxed), 1);
            send(&mut s, json!({"ok":true,"stream":true})).await;
            let metadata = first["payload"].clone();
            let mut frame = first;
            let mut raw = vec![];
            let mut terms = 0;
            loop {
                let data = frame["payload"]["data"].as_str().unwrap();
                if data.is_empty() {
                    terms += 1;
                    break;
                }
                let part = STANDARD.decode(data).unwrap();
                assert!(part.len() <= bound);
                raw.extend(part);
                frame = receive(&mut s).await;
                assert!(frame.get("stream").is_none());
                for key in if logo {
                    vec!["match", "self", "alpha", "position"]
                } else {
                    vec!["match", "all", "configured", "layout"]
                } {
                    assert_eq!(frame["payload"][key], metadata[key]);
                }
            }
            assert_eq!(raw, expected);
            assert_eq!(terms, 1);
            assert_eq!(
                s.read_u8().await.unwrap_err().kind(),
                std::io::ErrorKind::UnexpectedEof
            );
        });
        let result = if logo {
            session
                .upload(UploadSubmission(LogoPng::default()), source, options())
                .await
                .unwrap()
        } else {
            session
                .upload(
                    UploadSubmission(BackgroundPng::default()),
                    source,
                    options(),
                )
                .await
                .unwrap()
        };
        assert!(matches!(result, Completion::Submitted));
        assert!(!session.is_connected());
        assert_eq!(max.load(Ordering::Relaxed), bound);
        task.await.unwrap();
    }
}
#[tokio::test]
async fn refusal_stops_production_and_late_producer_error_never_terminates() {
    for reject in [true, false] {
        let peer = Peer::new();
        let mut session = peer.session();
        let (source, calls, _) = probe(vec![1; 4096], !reject, false);
        let task = tokio::spawn(async move {
            let (mut s, _) = peer.listener.accept().await.unwrap();
            receive(&mut s).await;
            send(
                &mut s,
                if reject {
                    json!({"ok":false,"error":"refused"})
                } else {
                    json!({"ok":true,"stream":true})
                },
            )
            .await;
            assert_eq!(
                s.read_u8().await.unwrap_err().kind(),
                std::io::ErrorKind::UnexpectedEof
            );
            calls.load(Ordering::Relaxed)
        });
        let result = session.upload(run(), source, options()).await;
        if reject {
            assert!(matches!(result, Err(Failure::Remote { .. })));
            assert_eq!(task.await.unwrap(), 1);
        } else {
            assert!(matches!(
                result,
                Err(Failure::Interrupted {
                    progress: Progress::WriteStarted,
                    reason: ExecutionError::Source(_)
                })
            ));
            assert_eq!(task.await.unwrap(), 2);
        }
        assert!(!session.is_connected());
    }
}
#[tokio::test]
async fn early_error_interrupts_pending_source_and_cancellation_retains_evidence() {
    for cancel in [false, true] {
        let peer = Peer::new();
        let mut session = peer.session();
        let (source, _, _) = probe(vec![1; 4096], false, true);
        let (ready, started) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let (mut s, _) = peer.listener.accept().await.unwrap();
            receive(&mut s).await;
            send(&mut s, json!({"ok":true,"stream":true})).await;
            ready.send(()).unwrap();
            if !cancel {
                tokio::time::sleep(Duration::from_millis(20)).await;
                send(&mut s, json!({"ok":false,"error":"early"})).await;
            }
            assert_eq!(
                s.read_u8().await.unwrap_err().kind(),
                std::io::ErrorKind::UnexpectedEof
            );
        });
        let evidence = Evidence::default();
        let p = prepare_upload(&run(), &Protection::Plain, 16384).unwrap();
        if cancel {
            let mut future = Box::pin(session.upload_prepared(
                p,
                source,
                options().session.deadline,
                options().session.max_response,
                options().max_upload,
                evidence.clone(),
            ));
            tokio::select! { _= &mut future => panic!("pending producer unexpectedly complete"),_=started=>{} }
            tokio::time::sleep(Duration::from_millis(20)).await;
            tokio::select! { _=&mut future=>panic!(),_=tokio::time::sleep(Duration::from_millis(20))=>{} }
            drop(future);
            assert_eq!(evidence.progress(), Progress::WriteStarted);
            assert!(evidence.stream_acknowledged());
            assert!(!evidence.input_complete());
        } else {
            let out = session
                .upload_prepared(
                    p,
                    source,
                    options().session.deadline,
                    options().session.max_response,
                    options().max_upload,
                    evidence,
                )
                .await;
            assert!(matches!(out, Outcome::Reply(_)));
            started.await.unwrap();
        }
        assert!(!session.is_connected());
        task.await.unwrap();
    }
}
#[tokio::test]
async fn source_and_trickled_handshake_share_absolute_deadline() {
    let peer = Peer::new();
    let mut session = peer.session();
    let (source, _, _) = probe(vec![1; 4096], false, false);
    let task = tokio::spawn(async move {
        let (mut s, _) = peer.listener.accept().await.unwrap();
        receive(&mut s).await;
        for byte in kitty_rc_frame::frame(b"{\"ok\":true,\"stream\":true}") {
            if s.write_all(&[byte]).await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(15)).await;
        }
    });
    let mut opts = options();
    opts.session.deadline = Instant::now() + Duration::from_millis(100);
    assert!(matches!(
        session.upload(run(), source, opts).await,
        Err(Failure::Interrupted {
            reason: ExecutionError::Deadline { .. },
            progress: Progress::WriteStarted
        })
    ));
    task.await.unwrap();
    let (source, _, _) = probe(vec![], false, true);
    let mut opts = options();
    opts.session.deadline = Instant::now() + Duration::from_millis(30);
    assert!(matches!(
        session.upload(run(), source, opts).await,
        Err(Failure::Interrupted {
            reason: ExecutionError::Deadline { .. },
            progress: Progress::NotWritten
        })
    ));
}
#[tokio::test]
async fn empty_invalid_png_limit_and_finite_misuse_fail_without_transport() {
    let peer = Peer::new();
    let mut session = peer.session();
    assert!(matches!(
        session
            .upload(
                BackgroundPng::default(),
                UploadSource::bytes(vec![]),
                options()
            )
            .await,
        Err(Failure::Interrupted {
            progress: Progress::NotWritten,
            reason: ExecutionError::InvalidPng
        })
    ));
    let mut opts = options();
    opts.max_upload = 1;
    assert!(matches!(
        session
            .upload(run(), UploadSource::bytes(vec![0; 4096]), opts)
            .await,
        Err(Failure::Interrupted {
            progress: Progress::NotWritten,
            reason: ExecutionError::UploadLimit
        })
    ));
    assert!(matches!(
        session
            .execute(run(), opts.session.deadline, 16384, 16384)
            .await,
        Err(Failure::Prepare(_))
    ));
    assert!(run().is_upload());
}

#[tokio::test]
async fn coalesced_ack_and_early_refusal_are_consumed_before_more_source_demand() {
    let peer = Peer::new();
    let mut session = peer.session();
    let (source, calls, _) = probe(vec![0; 8192], false, false);
    let task = tokio::spawn(async move {
        let (mut s, _) = peer.listener.accept().await.unwrap();
        receive(&mut s).await;
        let mut replies = kitty_rc_frame::frame(b"{\"ok\":true,\"stream\":true}");
        replies.extend(kitty_rc_frame::frame(
            b"{\"ok\":false,\"error\":\"early coalesced\"}",
        ));
        s.write_all(&replies).await.unwrap();
        assert_eq!(
            s.read_u8().await.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    });
    assert!(matches!(
        session.upload(run(), source, options()).await,
        Err(Failure::Remote { .. })
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    task.await.unwrap();
}

#[tokio::test]
async fn upload_future_fixed_storage_fits_prepared_reservation_without_polling_source() {
    let peer = Peer::new();
    let mut session = peer.session();
    let prepared = prepare_upload(&run(), &Protection::Plain, 16384).unwrap();
    let reserved = prepared.reservation_bytes();
    let (source, calls, _) = probe(vec![], false, true);
    let future = session.upload_prepared(
        prepared,
        source,
        options().session.deadline,
        options().session.max_response,
        options().max_upload,
        Evidence::default(),
    );
    assert!(std::mem::size_of_val(&future) <= reserved);
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    drop(future);
}
