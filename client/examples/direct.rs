//! Direct leaf Session: two serialized listing reads, no registry or facade.
mod support;
use clap::Parser;
use kitty_rc_commands::Ls;
use kitty_rc_exchange::Completion;
use kitty_rc_tokio::{Session, SessionOptions};
use std::time::Duration;
use support::{AccessArgs, Error};
use tokio::time::Instant;

#[derive(Parser)]
#[command(about = "Read listing twice through a caller-owned Session")]
struct Args {
    #[command(flatten)]
    access: AccessArgs,
    /// Drop a separate listing future at a short local deadline; never retry it.
    #[arg(long)]
    cancel_read: bool,
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let mut session = Session::new(args.access.resolve().await?.into_endpoint());
    if args.cancel_read {
        let result = tokio::time::timeout(
            Duration::from_millis(1),
            session.execute_with_options(
                Ls::default(),
                SessionOptions {
                    deadline: Instant::now() + Duration::from_secs(5),
                    max_request: 64 * 1024,
                    max_response: 1024 * 1024,
                },
            ),
        )
        .await;
        println!("optional read/cancel race: {result:?}");
        if let Some(evidence) = session.last_evidence() {
            println!(
                "local progress: {:?}, phase: {:?}",
                evidence.progress(),
                evidence.phase()
            );
        }
        // Do not automatically retry the cancelled occurrence. Future reads are
        // separate caller-selected samples; for writes this distinction matters.
    }
    for _ in 0..2 {
        let completion = session
            .execute_with_options(
                Ls::default(),
                SessionOptions {
                    deadline: Instant::now() + Duration::from_secs(5),
                    max_request: 64 * 1024,
                    max_response: 1024 * 1024,
                },
            )
            .await?;
        match completion {
            Completion::Reply(windows) => println!("{} OS windows", windows.len()),
            Completion::Submitted => return Err("listing must return a reply".into()),
        }
        // A local transport token shows actual ownership/reuse, not remote identity.
        println!("transport: {:?}", session.connection_token());
    }
    // Dropping an in-flight execute future cancels locally and retires its socket;
    // it does NOT roll back an already written remote request. No retry is added.
    Ok(())
}
