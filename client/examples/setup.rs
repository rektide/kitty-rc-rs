//! Separate pending route and credential acquisition/install generations.
mod support;
use clap::Parser;
use kitty_rc_registry::{Limits, Registry, Shutdown};
use support::{AccessArgs, Error};

#[derive(Parser)]
#[command(about = "Select access, install a pending setup, then explicitly invalidate/reacquire")]
struct Args {
    #[command(flatten)]
    access: AccessArgs,
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let registry = Registry::new(Limits::default())?;
    let pending = registry.register_pending(args.access.endpoint.clone())?;
    let endpoint = pending.endpoint();
    // Selected discovery/credential adapters are caller-owned, outside owner lock.
    // Dropping acquisition leaves setup pending; nothing silently retries.
    let resolved = args.access.resolve().await?;
    println!("credential provenance: {:?}", resolved.provenance);
    let version = registry.install_setup(pending, resolved.protection)?;
    println!("installed: {version:?}");
    let next = registry.invalidate_credentials(endpoint)?;
    let resolved = args.access.resolve().await?;
    println!(
        "reinstalled: {:?}",
        registry.install_setup(next, resolved.protection)?
    );
    // A ticket captured before invalidation is stale; generations are local setup
    // bases, not server-incarnation identities, liveness probes or permissions.
    registry.shutdown(Shutdown::Drain).await?;
    Ok(())
}
