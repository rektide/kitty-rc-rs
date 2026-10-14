//! ONLY invoked with the private fixture created by kitty-rc-verify uploads.
use kitty_rc_commands::{
    GetText, Ls,
    targets::{WindowId, WindowTarget},
    upload::{BackgroundPng, LogoPng, Run, UploadSubmission},
};
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::Completion;
use kitty_rc_registry::{Limits, Options, Registry, Shutdown, UploadOptions};
use kitty_rc_tokio::{Session, SessionOptions, UploadSource};
use tokio::time::{Duration, Instant};

/// The fixture child binary (kitty-rc-verify-child) launched by kitty for
/// the raw run journeys: exact binary stdin/stdout round trip, fixed stderr
/// bytes and a chosen exit code — no embedded interpreter strings.
fn child_program() -> Result<String, Box<dyn std::error::Error>> {
    std::env::var("KITTY_RC_MATRIX_CHILD").map_err(|_| {
        "private fixture contract: KITTY_RC_MATRIX_CHILD must name the fixture child".into()
    })
}

fn run() -> Result<Run, Box<dyn std::error::Error>> {
    Ok(Run::builder()
        .cmdline(vec![
            child_program()?,
            "stream".into(),
            "--stderr-hex".into(),
            "00ff80".into(),
            "--exit".into(),
            "7".into(),
        ])
        .build())
}
fn opts() -> Options {
    Options {
        deadline: Instant::now() + Duration::from_secs(10),
        max_response: 1024 * 1024,
    }
}
fn direct() -> kitty_rc_tokio::UploadOptions {
    kitty_rc_tokio::UploadOptions {
        session: SessionOptions {
            deadline: opts().deadline,
            max_request: 65536,
            max_response: opts().max_response,
        },
        max_upload: 1024 * 1024,
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("private fixture only: SOCKET KEY PASSWORD PNG".into());
    }
    let endpoint = Endpoint {
        address: Address::Unix(args[0].clone().into()),
        protection: Protection::Encrypted {
            key: kitty_rc_crypto::ServerPublicKey::parse(&args[1])?,
            password: args[2].clone(),
            key_origin: "private isolated child's explicit publication".into(),
        },
    };
    let mut session = Session::new(endpoint.clone());
    let Completion::Reply(listing) = session
        .execute(Ls::default(), opts().deadline, 65536, opts().max_response)
        .await?
    else {
        panic!()
    };
    let id = listing
        .iter()
        .flat_map(|os| &os.tabs)
        .flat_map(|tab| &tab.windows)
        .find_map(|w| w.id)
        .ok_or("no private window")?;
    let png = std::fs::read(&args[3])?;
    session
        .upload(
            BackgroundPng::builder()
                .match_spec(format!("id:{id}"))
                .build(),
            UploadSource::bytes(png.clone()),
            direct(),
        )
        .await?;
    session
        .upload(
            LogoPng::builder()
                .target(WindowTarget::Id(WindowId(id)))
                .position("bottom-right")
                .build(),
            UploadSource::bytes(png.clone()),
            direct(),
        )
        .await?;
    // A suppressed terminal still needs a native stream handshake; its local
    // completion is not advertised as image application success.
    assert!(matches!(
        session
            .upload(
                UploadSubmission(BackgroundPng::default()),
                UploadSource::bytes(png),
                direct()
            )
            .await?,
        Completion::Submitted
    ));
    for size in [0, 1, 4096, 16385] {
        let bytes: Vec<_> = (0..size).map(|n| (n % 256) as u8).collect();
        let expected = bytes.clone();
        let Completion::Reply(output) = session
            .upload(run()?, UploadSource::bytes(bytes), direct())
            .await?
        else {
            panic!()
        };
        assert_eq!(output.stdout, expected);
        assert_eq!(output.stderr, [0, 255, 128]);
        assert_eq!(output.exit_code, 7);
        assert_eq!(output.exit_status, 1792);
    }
    let registry = Registry::new(Limits::default())?;
    let a = registry.register(endpoint.clone())?;
    let alias = registry.register(endpoint)?;
    registry.assign_alias(alias, registry.coordination_group(a)?)?;
    let driver = registry.start()?;
    let long = Run::builder()
        .cmdline(vec![
            child_program()?,
            "sleep".into(),
            "--ms".into(),
            "200".into(),
        ])
        .build();
    let first = registry
        .submit_upload(
            &registry.reserve(),
            a,
            long,
            UploadSource::bytes(vec![]),
            UploadOptions {
                operation: opts(),
                max_upload: 1024,
            },
        )
        .map_err(|e| e.reason)?;
    let second = registry
        .submit_upload(
            &registry.reserve(),
            alias,
            run()?,
            UploadSource::bytes(vec![0, 255, 128]),
            UploadOptions {
                operation: opts(),
                max_upload: 1024,
            },
        )
        .map_err(|e| e.reason)?;
    let text = registry
        .submit(&registry.reserve(), a, GetText::default(), opts())
        .map_err(|e| e.reason)?;
    text.wait().await?;
    text.take()?;
    first.wait().await?;
    first.take()?;
    second.wait().await?;
    let Completion::Reply(output) = second.take()? else {
        panic!()
    };
    assert_eq!(output.stdout, [0, 255, 128]);
    first.release()?;
    second.release()?;
    text.release()?;
    assert!(registry.lane_inventory().is_empty());
    registry.shutdown(Shutdown::Drain).await?;
    driver.stop().await;
    println!(
        "private encrypted kitty PNG background/logo, suppressed-terminal handshake, binary empty/small/exact/large run and managed aliases passed"
    );
    Ok(())
}
