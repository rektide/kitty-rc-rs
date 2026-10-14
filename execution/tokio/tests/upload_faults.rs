use kitty_rc_commands::upload::Run;
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{ExecutionError, Failure, Progress};
use kitty_rc_tokio::{Session, SessionOptions, UploadOptions, UploadSource};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
    time::{Duration, Instant},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
#[tokio::test]
async fn malformed_handshake_and_late_total_limit_never_finalize() {
    for malformed in [true, false] {
        let path = std::path::PathBuf::from(format!(
            "/tmp/opencode/kitty-upload-fault-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let peer = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let mut d = kitty_rc_frame::FrameDecoder::new(16384);
            loop {
                let b = s.read_u8().await.unwrap();
                d.extend(&[b]).unwrap();
                if d.next_frame().unwrap().is_some() {
                    break;
                }
            }
            s.write_all(&kitty_rc_frame::frame(if malformed {
                b"{\"ok\":true}"
            } else {
                b"{\"ok\":true,\"stream\":true}"
            }))
            .await
            .unwrap();
            assert_eq!(
                s.read_u8().await.unwrap_err().kind(),
                std::io::ErrorKind::UnexpectedEof
            );
        });
        let mut session = Session::new(Endpoint {
            address: Address::Unix(path.clone()),
            protection: Protection::Plain,
        });
        let options = UploadOptions {
            session: SessionOptions {
                deadline: Instant::now() + Duration::from_secs(3),
                max_request: 16384,
                max_response: 16384,
            },
            max_upload: 4096,
        };
        let result = session
            .upload(
                Run::builder().cmdline(vec!["cat".into()]).build(),
                UploadSource::bytes(vec![0; 8192]),
                options,
            )
            .await;
        if malformed {
            assert!(matches!(
                result,
                Err(Failure::Interrupted {
                    progress: Progress::WriteStarted,
                    reason: ExecutionError::UploadResponse { .. }
                })
            ));
        } else {
            assert!(matches!(
                result,
                Err(Failure::Interrupted {
                    progress: Progress::WriteStarted,
                    reason: ExecutionError::UploadLimit
                })
            ));
        }
        assert!(!session.last_evidence().unwrap().input_complete());
        peer.await.unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
