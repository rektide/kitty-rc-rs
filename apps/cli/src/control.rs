//! One process-owned registry/driver; finite observations and deliberate shutdown.
use crate::{
    Error, configuration,
    interface::{Active, Cli, Command, Scope},
    output,
};
use kitty_rc_commands::{
    FocusWindow,
    targets::{TabMatch, WindowId, WindowMatch},
};
use kitty_rc_discovery::ProcessEnvironment;
use kitty_rc_exchange::Completion;
use kitty_rc_listing::{Binding, Change, Interpretation, Observer, Poller, Snapshot, Support};
use kitty_rc_registry::{EndpointId, Limits, Options, Registry, Shutdown};
use serde_json::json;
use std::time::Duration;
use tokio::time::Instant;

pub async fn run(cli: Cli) -> Result<(), Error> {
    let env = ProcessEnvironment;
    let file = configuration::FileConfig::read(cli.config.as_ref()).await?;
    let candidate = configuration::endpoint(&cli, &file, &env).await?;
    let (candidate, resolved) = configuration::access(&cli, &file, &env, candidate).await?;
    if cli.verbose {
        eprintln!(
            "selected {:?} from {:?}; credential sources {:?}",
            candidate.binding, candidate.origin, resolved.provenance
        );
    }
    let registry = Registry::new(registry_limits(cli.max_response as usize))?;
    let endpoint = registry.register(resolved.into_endpoint())?;
    let driver = registry.start()?;
    let result = tokio::select! {
        result = commands(&cli, &registry, endpoint) => result,
        signal = tokio::signal::ctrl_c() => {
            match signal {
                Ok(()) => Err(crate::Interrupted.into()),
                Err(error) => Err(error.into()),
            }
        }
    };
    // Keep driver alive through cancellation completion; native records survive
    // dropped command futures and are reported before explicit release.
    let shutdown = registry.shutdown(Shutdown::Cancel).await;
    for summary in registry.inventory() {
        eprintln!(
            "shutdown operation {:?}: {:?}, unresolved={}; no replay/rollback inferred",
            summary.token, summary.progress, summary.unresolved
        );
        if let Ok(native) = registry.take_native(summary.token) {
            match native {
                kitty_rc_exchange::Outcome::Interrupted { progress, reason } => {
                    eprintln!("  native interruption: {progress:?}; {reason}");
                }
                kitty_rc_exchange::Outcome::Reply(bytes) => {
                    eprintln!(
                        "  completed native reply retained {} bytes before shutdown release",
                        bytes.len()
                    );
                }
                kitty_rc_exchange::Outcome::Submitted => {
                    eprintln!("  local submission only, no remote acknowledgment");
                }
            }
        }
        let _ = registry.release(summary.token);
    }
    driver.stop().await;
    shutdown?;
    result
}
/// The CLI admits one operation at a time and takes/releases before the next.
/// Reserve the response budget plus bounded route-bearing failure context at
/// both quota tiers. Registry endpoint setup already cannot exceed max_request.
fn registry_limits(max_response: usize) -> Limits {
    let mut limits = Limits::default();
    let retention = max_response.saturating_add(limits.max_request);
    limits.output_bytes = retention;
    limits.output_bytes_per_endpoint = retention;
    limits
}
fn options(cli: &Cli) -> Options {
    Options {
        deadline: Instant::now() + Duration::from_millis(cli.timeout_ms),
        max_response: cli.max_response as usize,
    }
}
fn observer(registry: &Registry, endpoint: EndpointId, scope: &Scope) -> Observer {
    Observer::new(
        registry.clone(),
        Binding {
            subject: "CLI-selected endpoint (not verified server identity)".into(),
            endpoint,
            generation: 1,
        },
        kitty_rc_listing::Scope {
            match_window: scope.match_window.clone().map(WindowMatch::raw),
            match_tab: scope.match_tab.clone().map(TabMatch::raw),
            ..Default::default()
        },
        Interpretation {
            basis: "native reply fields; no auto-negotiated focus capability".into(),
            focus: Support::Unknown,
        },
        kitty_rc_listing::Limits::default(),
    )
}
async fn snapshot(observer: &mut Observer, cli: &Cli) -> Result<Snapshot, Error> {
    let acquisition = observer.acquire(options(cli)).map_err(|e| e.reason)?;
    acquisition.wait().await?;
    Ok(acquisition.finish()?)
}
async fn commands(cli: &Cli, registry: &Registry, endpoint: EndpointId) -> Result<(), Error> {
    match &cli.command {
        Command::ListWindows { scope } => {
            let sample = snapshot(&mut observer(registry, endpoint, scope), cli).await?;
            output::list(sample.listing(), cli.json)
        }
        Command::ActiveWindow { target } => {
            let sample = snapshot(&mut observer(registry, endpoint, &target.scope), cli).await?;
            output::selected(&output::active(sample.listing(), target)?, cli.json)
        }
        Command::Goto { window_id } => {
            let reservation = registry.reserve();
            let operation = registry
                .submit(
                    &reservation,
                    endpoint,
                    FocusWindow::id(WindowId(*window_id)),
                    options(cli),
                )
                .map_err(|e| e.reason)?;
            operation.wait().await?;
            let result = operation.take();
            if result.is_err()
                && let Some(summary) = registry
                    .inventory()
                    .into_iter()
                    .find(|s| s.token == reservation.token())
            {
                eprintln!(
                    "focus progress {:?}, unresolved={}; not retried",
                    summary.progress, summary.unresolved
                );
            }
            operation.release()?;
            match result? {
                Completion::Reply(()) => {
                    if cli.json {
                        output::json_line(
                            &json!({"focused_window_id":window_id,"completion":"remote_reply"}),
                        )
                    } else {
                        Ok(())
                    }
                }
                Completion::Submitted => Err("focus unexpectedly lacked a remote reply".into()),
            }
        }
        Command::Watch {
            target,
            interval_ms,
            count,
        } => watch(cli, registry, endpoint, target, *interval_ms, *count).await,
    }
}
async fn watch(
    cli: &Cli,
    registry: &Registry,
    endpoint: EndpointId,
    target: &Active,
    interval_ms: u64,
    count: Option<u64>,
) -> Result<(), Error> {
    let mut poller = Poller::new(
        observer(registry, endpoint, &target.scope),
        Duration::from_millis(interval_ms),
    )?;
    let mut emitted = 0u64;
    loop {
        let acquisition = poller
            .next(
                Duration::from_millis(cli.timeout_ms),
                cli.max_response as usize,
            )
            .await
            .map_err(|e| e.reason)?;
        acquisition.wait().await?;
        let sample = acquisition.finish()?;
        let active = output::sampled_active(sample.listing(), target)?;
        if cli.json {
            let changes = poller
                .observer()
                .changes(&sample)
                .map_err(|e| format!("ambiguous sampled changes: {e:?}"))?
                .map(|change| match change {
                    Change::Returned { id, .. } => json!({"kind":"Returned", "id":id}),
                    Change::Changed { id, .. } => json!({"kind":"Changed", "id":id}),
                    Change::NoLongerReturned { id } => json!({"kind":"NoLongerReturned", "id":id}),
                })
                .collect::<Vec<_>>();
            let timing = sample.receipt().poll().expect("poll receipt");
            output::json_line(
                &json!({"kind":"sample", "sequence":sample.receipt().order(),
                "active":active.as_ref().map(output::value), "changes":changes,
                "active_status":if active.is_none() { output::absence_status(target) } else if target.tab.is_some() || target.os_window.is_some() { "active" } else { "focused" },
                "interval_ms":interval_ms, "delay_ms":sample.receipt().started().saturating_duration_since(timing.scheduled_for).as_millis(),
                "acquisition_ms":sample.ended().saturating_duration_since(sample.receipt().started()).as_millis(),
                "scope":{"match":target.scope.match_window,"match_tab":target.scope.match_tab},
                "absence_meaning":"not returned in this sample, not destroyed"}),
            )?;
        } else if let Some(active) = &active {
            output::selected(active, false)?;
        } else {
            output::absent(target)?;
        }
        drop(
            poller
                .observer_mut()
                .accept(sample)
                .map_err(|(error, _)| format!("sample rejected: {error:?}"))?,
        );
        emitted += 1;
        if count.is_some_and(|count| emitted >= count) {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_reservations_cover_response_and_bounded_failure_context() {
        for budget in [256, 1_048_576, 8_388_609, 16_777_216] {
            let limits = registry_limits(budget);
            assert_eq!(limits.max_request, 256 * 1024);
            assert_eq!(limits.output_bytes, budget + limits.max_request);
            assert_eq!(
                limits.output_bytes_per_endpoint,
                budget + limits.max_request
            );
        }
    }
}
