//! Only for a deliberately isolated real kitty fixture. Launches a child.
use kitty_rc::{
    commands::{FocusWindow, GetText, LaunchWait, Ls},
    *,
};
use std::time::Duration;
use tokio::time::Instant;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("isolated fixture only: SOCKET KEY PASSWORD".into());
    }
    let endpoint = Endpoint {
        address: Address::Unix(args[0].clone().into()),
        protection: Protection::Encrypted {
            key: parse_server_public_key(&args[1])?,
            password: args[2].clone(),
            key_origin: "isolated child's explicit environment export".into(),
        },
    };
    let mut session = Session::new(endpoint.clone());
    let options = || Options {
        deadline: Instant::now() + Duration::from_secs(5),
        max_response: 1024 * 1024,
    };
    let Completion::Reply(windows) = session
        .execute(
            Ls::default(),
            options().deadline,
            65536,
            options().max_response,
        )
        .await?
    else {
        return Err("listing suppressed".into());
    };
    let id = windows
        .iter()
        .flat_map(|os| &os.tabs)
        .flat_map(|tab| &tab.windows)
        .find_map(|w| w.id)
        .ok_or("no isolated window")?;
    let registry = Registry::new(Limits::default())?;
    let ep = registry.register(endpoint)?;
    let driver = registry.start()?;
    let long = registry
        .submit(
            &registry.reserve(),
            ep,
            LaunchWait {
                args: vec!["sh".into(), "-c".into(), "sleep 0.5; exit 7".into()],
            },
            options(),
        )
        .map_err(|e| e.reason)?;
    let text = registry
        .submit(
            &registry.reserve(),
            ep,
            GetText {
                match_spec: Some(format!("id:{id}")),
                ..GetText::default()
            },
            options(),
        )
        .map_err(|e| e.reason)?;
    let focus = registry
        .submit(
            &registry.reserve(),
            ep,
            FocusWindow {
                match_spec: format!("id:{id}"),
            },
            options(),
        )
        .map_err(|e| e.reason)?;
    focus.wait().await?;
    focus.take()?;
    text.wait().await?;
    text.take()?;
    long.wait().await?;
    match long.take()? {
        Completion::Reply(code) if code == "7" => {}
        other => return Err(format!("unexpected launch exit {other:?}").into()),
    }
    long.release()?;
    text.release()?;
    focus.release()?;
    registry.shutdown(Shutdown::Drain).await?;
    driver.stop().await;
    println!("isolated encrypted direct listing and concurrent managed launch/text/focus passed");
    Ok(())
}
