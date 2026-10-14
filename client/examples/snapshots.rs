//! Finite listing samples and an optional uncertain-focus read-only follow-up.
mod support;
use clap::Parser;
use kitty_rc_check::{Condition, Criterion, Freshness, Request, RetainedOriginal};
use kitty_rc_commands::FocusWindow;
use kitty_rc_listing::{Binding, Interpretation, Observer, Scope, Support};
use kitty_rc_registry::{Limits, Options, Registry, Shutdown};
use std::time::Duration;
use support::{AccessArgs, Error};
use tokio::time::Instant;

#[derive(Parser)]
#[command(
    about = "Read two scoped snapshots; optionally focus an explicit ID and check an uncertain result"
)]
struct Args {
    #[command(flatten)]
    access: AccessArgs,
    /// Caller-selected subject label, not a verified server identity.
    #[arg(long)]
    subject: String,
    /// Explicit listing scope; absence is only relative to this selection.
    #[arg(long)]
    match_window: Option<String>,
    /// MUTATION opt-in: focus exactly this window ID on the selected endpoint.
    #[arg(long)]
    focus_window_id: Option<u64>,
}
fn options() -> Options {
    Options {
        deadline: Instant::now() + Duration::from_secs(5),
        max_response: 1024 * 1024,
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let registry = Registry::new(Limits::default())?;
    let endpoint = registry.register(args.access.resolve().await?.into_endpoint())?;
    let driver = registry.start()?;
    let binding = Binding {
        subject: args.subject,
        endpoint,
        generation: registry.setup_version(endpoint)?.binding.value(),
    };
    let interpretation = Interpretation {
        basis: "caller-selected server; focus availability not pre-negotiated".into(),
        focus: Support::Unknown,
    };
    let mut observer = Observer::new(
        registry.clone(),
        binding.clone(),
        Scope {
            match_window: args.match_window,
            ..Default::default()
        },
        interpretation.clone(),
        kitty_rc_listing::Limits::default(),
    );
    for _ in 0..2 {
        let acquisition = observer.acquire(options()).map_err(|e| e.reason)?;
        // Save receipt before awaiting if cancellation/recovery matters.
        println!("sample receipt: {:?}", acquisition.receipt());
        acquisition.wait().await?;
        let snapshot = acquisition.finish()?; // transfers listing and releases read
        for change in observer.changes(&snapshot).map_err(|reason| {
            std::io::Error::other(format!("sample comparison rejected: {reason:?}"))
        })? {
            println!("sample difference: {change:?}");
        }
        let displaced = observer.accept(snapshot).map_err(|(reason, _)| {
            std::io::Error::other(format!("baseline acceptance rejected: {reason:?}"))
        })?;
        drop(displaced); // explicit baseline advancement, no retained history
    }
    if let Some(id) = args.focus_window_id {
        let reservation = registry.reserve();
        let operation = registry
            .submit(
                &reservation,
                endpoint,
                FocusWindow {
                    match_spec: format!("id:{id}"),
                },
                options(),
            )
            .map_err(|e| e.reason)?;
        operation.wait().await?;
        let summary = registry
            .summary(operation.token())
            .ok_or("missing operation")?;
        if summary.unresolved {
            let original = RetainedOriginal::capture(&registry, operation.token(), binding, 4096)?;
            // No Command or mutation input enters CheckInput: exactly one fresh ls.
            let input = original.check_input(
                Request {
                    scope: Scope {
                        match_window: Some(format!("id:{id}")),
                        ..Default::default()
                    },
                    interpretation,
                    criteria: vec![Criterion::WindowId(id), Criterion::Focused(true)],
                    condition: Condition::Unique,
                    freshness: Freshness {
                        max_age: Duration::from_secs(5),
                    },
                },
                kitty_rc_check::Limits::default(),
            )?;
            let check = input.start(&registry, options())?;
            check.wait().await?;
            // A check failure still leaves the original available for its own take.
            let report = check.finish();
            let evidence = original.take()?; // transfer/release ORIGINAL separately
            println!("original unchanged: {:?}", evidence.native);
            println!("check-only evidence: {:?}", report?.assessment);
            // Satisfaction never establishes causation or authorizes replay.
        } else {
            let result = operation.take();
            operation.release()?;
            println!("focus native result (not replayed): {result:?}");
        }
    }
    registry.shutdown(Shutdown::Drain).await?;
    driver.stop().await;
    Ok(())
}
