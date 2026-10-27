//! Concurrent bounded native custody using leaf crates, no facade requirement.
mod support;
use clap::Parser;
use kitty_rc_commands::Ls;
use kitty_rc_exchange::Completion;
use kitty_rc_registry::{Limits, Options, Registry, Shutdown};
use std::time::Duration;
use support::{AccessArgs, Error};
use tokio::time::Instant;

#[derive(Parser)]
#[command(about = "Submit two concurrent reads, take and explicitly release results")]
struct Args {
    #[command(flatten)]
    access: AccessArgs,
    /// Request cancellation of the second occurrence, not just its waiter.
    #[arg(long)]
    cancel_second: bool,
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let registry = Registry::new(Limits::default())?;
    let endpoint = registry.register(args.access.resolve().await?.into_endpoint())?;
    let driver = registry.start()?;
    let mut operations = Vec::new();
    for _ in 0..2 {
        let reservation = registry.reserve(); // identity known before acceptance
        println!("reserved: {:?}", reservation.token());
        operations.push(
            registry
                .submit(
                    &reservation,
                    endpoint,
                    Ls::all(),
                    Options {
                        deadline: Instant::now() + Duration::from_secs(5),
                        max_response: 1024 * 1024,
                    },
                )
                .map_err(|e| e.reason)?,
        );
    }
    if args.cancel_second {
        println!("cancel requested: {}", operations[1].cancel());
    }
    // A local waiter timeout detaches: the accepted operation keeps running.
    let _detached = tokio::time::timeout(Duration::ZERO, operations[0].wait()).await;
    tokio::try_join!(operations[0].wait(), operations[1].wait())?;
    for operation in operations {
        let native_result = operation.take(); // sole typed take; output is now ours
        operation.release()?; // release even when native_result is an error
        match native_result {
            Ok(Completion::Reply(windows)) => println!("{} OS windows", windows.len()),
            Ok(Completion::Submitted) => return Err("listing must return a reply".into()),
            Err(failure) => eprintln!("native read failure (not replayed): {failure:?}"),
        }
    }
    println!("retained idle pool: {:?}", registry.pool_stats());
    // A subsequent read can use an actual compatible idle socket. No per-endpoint
    // total ordering or replay is inferred from sharing that pool.
    let next = registry.reserve();
    let operation = registry
        .submit(
            &next,
            endpoint,
            Ls::all(),
            Options {
                deadline: Instant::now() + Duration::from_secs(5),
                max_response: 1024 * 1024,
            },
        )
        .map_err(|e| e.reason)?;
    operation.wait().await?;
    let result = operation.take();
    operation.release()?;
    println!("independent follow-up read: {result:?}");
    println!("pool after follow-up: {:?}", registry.pool_stats());
    registry.shutdown(Shutdown::Drain).await?;
    driver.stop().await;
    Ok(())
}
