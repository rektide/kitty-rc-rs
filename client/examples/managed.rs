//! Run against an explicitly selected endpoint: cargo run -p kitty-rc --example
//! managed -- /path/to/socket [published-key password]
use kitty_rc::{commands::Ls, *};
use std::time::Duration;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let path = args.first().ok_or("usage: managed SOCKET [KEY PASSWORD]")?;
    let protection = match args.len() {
        1 => Protection::Plain,
        3 => Protection::Encrypted {
            key: kitty_rc::parse_server_public_key(&args[1])?,
            password: args[2].clone(),
            key_origin: "explicit command-line fixture".into(),
        },
        _ => return Err("usage: managed SOCKET [KEY PASSWORD]".into()),
    };
    let registry = Registry::new(Limits::default())?;
    let endpoint = registry.register(Endpoint {
        address: Address::Unix(path.into()),
        protection,
    })?;
    let driver = registry.start()?;
    let reservation = registry.reserve(); // identity known before commit
    let operation = registry
        .submit(
            &reservation,
            endpoint,
            Ls::default(),
            Options {
                deadline: Instant::now() + Duration::from_secs(5),
                max_response: 1024 * 1024,
            },
        )
        .map_err(|e| e.reason)?;
    operation.wait().await?; // dropping only this waiter would detach
    if let Completion::Reply(windows) = operation.take()? {
        println!("{} OS windows", windows.len());
    }
    operation.release()?; // deliberate byte/record release
    registry.shutdown(Shutdown::Drain).await?;
    driver.stop().await;
    Ok(())
}
