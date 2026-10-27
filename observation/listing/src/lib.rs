//! Finite, scoped `ls` acquisition over managed registry custody.
//! No task is started here. Dropping an acquisition detaches, not cancels: save
//! its receipt and recover its token through the registry. Publication and
//! baseline advancement require an explicit call on the caller-owned observer.
use kitty_rc_commands::{
    Ls,
    outputs::OsWindow,
    targets::{TabMatch, WindowMatch},
};
use kitty_rc_exchange::{Completion, Failure};
use kitty_rc_registry::{CustodyError, EndpointId, Operation, Options, Refused, Registry, Token};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::time::Instant;

/// Caller assertion, not proof of server process identity. Replace the observer
/// when subject, access binding, generation, scope, or interpretation changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub subject: String,
    pub endpoint: EndpointId,
    pub generation: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scope {
    pub match_window: Option<WindowMatch>,
    pub match_tab: Option<TabMatch>,
    pub only_self: bool,
    pub all_env_vars: bool,
}
impl Scope {
    fn command(&self) -> Ls {
        Ls {
            match_spec: self.match_window.clone(),
            match_tab: self.match_tab.clone(),
            only_self: self.only_self,
            all_env_vars: self.all_env_vars,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    Supported,
    Unsupported,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fact<T> {
    Known(T),
    Missing,
    Unsupported,
    UnknownSupport,
}
fn fact<T>(value: Option<T>, support: Support) -> Fact<T> {
    match support {
        Support::Supported => value.map_or(Fact::Missing, Fact::Known),
        Support::Unsupported => Fact::Unsupported,
        Support::Unknown => value.map_or(Fact::UnknownSupport, Fact::Known),
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpretation {
    /// Caller-supplied version/capability evidence, not auto-detected.
    pub basis: String,
    pub focus: Support,
}
/// The registry bounds native response bytes; this additionally bounds the
/// compact sampled projection. Typed listing allocations belong to the caller.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_windows: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_windows: 16_384,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Receipt {
    binding: Binding,
    scope: Scope,
    interpretation: Interpretation,
    token: Token,
    /// Local acquisition issuance order, never a remote event/resume cursor.
    order: u64,
    started: Instant,
    poll: Option<PollTiming>,
    owner: Arc<()>,
    limits: Limits,
}
/// Scheduling evidence only; delayed calls skip catch-up acquisitions.
#[derive(Debug, Clone, Copy)]
pub struct PollTiming {
    pub scheduled_for: Instant,
    pub cadence: Duration,
}
impl Receipt {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    pub fn interpretation(&self) -> &Interpretation {
        &self.interpretation
    }
    pub fn token(&self) -> Token {
        self.token
    }
    pub fn order(&self) -> u64 {
        self.order
    }
    pub fn started(&self) -> Instant {
        self.started
    }
    pub fn poll(&self) -> Option<PollTiming> {
        self.poll
    }
    pub fn route(&self) -> &'static str {
        "kitty-rc/ls"
    }
    /// Recover a detached acquisition after `registry.wait(token)`. Receipt
    /// clones are not replay authority: the registry permits only one take.
    pub fn finish(self, registry: &Registry) -> Result<Snapshot, Box<AcquisitionFailure>> {
        let ended = Instant::now();
        let native = match registry.take_native(self.token) {
            Ok(native) => native,
            Err(error) => {
                return Err(Box::new(AcquisitionFailure {
                    receipt: self,
                    ended,
                    error: AcquisitionError::Native(Failure::Unavailable(Box::new(error))),
                }));
            }
        };
        // A concurrent external release may already have removed this taken
        // record. Neither case retains output; no unfinished record is pruned.
        let _ = registry.release(self.token);
        let result = match native.decode::<Ls>() {
            Ok(Completion::Reply(listing)) => Snapshot::build(self.clone(), listing, ended),
            Ok(Completion::Submitted) => Err(AcquisitionError::UnexpectedSubmission),
            Err(error) => Err(AcquisitionError::Native(error)),
        };
        result.map_err(|error| {
            Box::new(AcquisitionFailure {
                receipt: self,
                ended,
                error,
            })
        })
    }
}
/// Coverage is about the selected `ls` response, never absence outside filters
/// or a server-atomic interval. The typed decoder requires IDs and containers;
/// duplicate IDs still make the membership projection partial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coverage {
    /// Completeness is relative to this response and its exact selection scope,
    /// not the desktop or unknown facts omitted from individual rows.
    pub membership: MembershipBasis,
    pub filtered: bool,
    pub all_environment_requested: bool,
    pub unidentified_windows: usize,
    pub duplicate_window_ids: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipBasis {
    CompleteSelectedResponse,
    PartialSelectedResponse,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSample {
    pub os_window: Fact<u64>,
    pub tab: Fact<u64>,
    pub focused: Fact<bool>,
    pub active: Fact<bool>,
}
#[derive(Debug)]
pub struct Snapshot {
    receipt: Receipt,
    ended: Instant,
    listing: Vec<OsWindow>,
    windows: BTreeMap<u64, WindowSample>,
    coverage: Coverage,
}
impl Snapshot {
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
    pub fn ended(&self) -> Instant {
        self.ended
    }
    pub fn listing(&self) -> &[OsWindow] {
        &self.listing
    }
    pub fn windows(&self) -> &BTreeMap<u64, WindowSample> {
        &self.windows
    }
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    fn build(
        receipt: Receipt,
        listing: Vec<OsWindow>,
        ended: Instant,
    ) -> Result<Self, AcquisitionError> {
        let mut windows = BTreeMap::new();
        let mut coverage = Coverage {
            membership: MembershipBasis::CompleteSelectedResponse,
            filtered: receipt.scope.match_window.is_some()
                || receipt.scope.match_tab.is_some()
                || receipt.scope.only_self,
            all_environment_requested: receipt.scope.all_env_vars,
            unidentified_windows: 0,
            duplicate_window_ids: false,
        };
        let mut count = 0usize;
        for os in &listing {
            for tab in &os.tabs {
                for window in &tab.windows {
                    count += 1;
                    if count > receipt.limits.max_windows {
                        return Err(AcquisitionError::WindowLimit);
                    }
                    let Some(id) = window.id else {
                        coverage.unidentified_windows += 1;
                        continue;
                    };
                    let sample = WindowSample {
                        os_window: fact(os.id, Support::Supported),
                        tab: fact(tab.id, Support::Supported),
                        focused: fact(window.is_focused, receipt.interpretation.focus),
                        active: fact(window.is_active, Support::Supported),
                    };
                    if windows.insert(id, sample).is_some() {
                        coverage.duplicate_window_ids = true;
                    }
                }
            }
        }
        if coverage.unidentified_windows != 0 || coverage.duplicate_window_ids {
            coverage.membership = MembershipBasis::PartialSelectedResponse;
        }
        Ok(Self {
            receipt,
            ended,
            listing,
            windows,
            coverage,
        })
    }
}
#[derive(Debug, thiserror::Error)]
pub enum AcquisitionError {
    #[error("native listing acquisition failed: {0}")]
    Native(#[source] Failure),
    #[error("listing command returned submission-only completion")]
    UnexpectedSubmission,
    #[error("listing exceeds caller-selected window projection limit")]
    WindowLimit,
}
#[derive(Debug, thiserror::Error)]
#[error("listing acquisition failed: {error}")]
pub struct AcquisitionFailure {
    pub receipt: Receipt,
    pub ended: Instant,
    #[source]
    pub error: AcquisitionError,
}
#[must_use = "accepted custody persists: finish or save receipt/token for explicit recovery"]
pub struct Acquisition {
    receipt: Receipt,
    operation: Operation<Ls>,
    registry: Registry,
}
impl Acquisition {
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
    pub async fn wait(&self) -> Result<(), CustodyError> {
        self.operation.wait().await
    }
    /// Explicit interruption; it does not release custody or prove rollback.
    pub fn cancel(&self) -> bool {
        self.operation.cancel()
    }
    /// Nonblocking sole take. Call after wait. An unavailable take retains the
    /// native record; every actually taken outcome is transferred to this return
    /// value before its registry record is explicitly released.
    pub fn finish(self) -> Result<Snapshot, Box<AcquisitionFailure>> {
        self.receipt.finish(&self.registry)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reject {
    ForeignOwner,
    Stale,
    AmbiguousProjection,
}
/// Caller-owned acquisition order and explicitly accepted baseline. At most one
/// listing is retained; acquire does not implicitly accept or change it.
pub struct Observer {
    registry: Registry,
    binding: Binding,
    scope: Scope,
    interpretation: Interpretation,
    limits: Limits,
    owner: Arc<()>,
    issued: u64,
    baseline: Option<Snapshot>,
}
impl Observer {
    pub fn new(
        registry: Registry,
        binding: Binding,
        scope: Scope,
        interpretation: Interpretation,
        limits: Limits,
    ) -> Self {
        Self {
            registry,
            binding,
            scope,
            interpretation,
            limits,
            owner: Arc::new(()),
            issued: 0,
            baseline: None,
        }
    }
    /// Only accepted submissions advance local ordering. Newer failed samples
    /// preserve the accepted baseline but still invalidate older in-flight work.
    pub fn acquire(&mut self, options: Options) -> Result<Acquisition, Refused<Ls>> {
        let reservation = self.registry.reserve();
        let started = Instant::now();
        let operation = self.registry.submit(
            &reservation,
            self.binding.endpoint,
            self.scope.command(),
            options,
        )?;
        self.issued = self
            .issued
            .checked_add(1)
            .expect("observation order exhausted");
        let receipt = Receipt {
            binding: self.binding.clone(),
            scope: self.scope.clone(),
            interpretation: self.interpretation.clone(),
            token: reservation.token(),
            order: self.issued,
            started,
            poll: None,
            owner: self.owner.clone(),
            limits: self.limits,
        };
        Ok(Acquisition {
            receipt,
            operation,
            registry: self.registry.clone(),
        })
    }
    pub fn baseline(&self) -> Option<&Snapshot> {
        self.baseline.as_ref()
    }
    pub fn validate(&self, snapshot: &Snapshot) -> Result<(), Reject> {
        if !Arc::ptr_eq(&self.owner, &snapshot.receipt.owner) {
            return Err(Reject::ForeignOwner);
        }
        if snapshot.receipt.order != self.issued
            || self
                .baseline
                .as_ref()
                .is_some_and(|old| old.receipt.order >= snapshot.receipt.order)
        {
            return Err(Reject::Stale);
        }
        Ok(())
    }
    /// Compare without advancing the baseline. Lazy, bounded by the two sample
    /// projections; no output queue or clone of either full listing.
    pub fn changes<'a>(
        &'a self,
        snapshot: &'a Snapshot,
    ) -> Result<impl Iterator<Item = Change> + 'a, Reject> {
        self.validate(snapshot)?;
        if snapshot.coverage.duplicate_window_ids
            || self
                .baseline
                .as_ref()
                .is_some_and(|s| s.coverage.duplicate_window_ids)
        {
            return Err(Reject::AmbiguousProjection);
        }
        let old = self.baseline.as_ref().map(|s| &s.windows);
        Ok(snapshot
            .windows
            .iter()
            .filter_map(
                move |(&id, &sample)| match old.and_then(|m| m.get(&id)).copied() {
                    None => Some(Change::Returned { id, sample }),
                    Some(before) if before != sample => Some(Change::Changed {
                        id,
                        before,
                        after: sample,
                    }),
                    _ => None,
                },
            )
            .chain(old.into_iter().flat_map(move |m| {
                m.keys().filter_map(move |&id| {
                    (!snapshot.windows.contains_key(&id)).then_some(Change::NoLongerReturned { id })
                })
            })))
    }
    /// Explicit publish/accept; returns the displaced baseline for caller-owned
    /// retention or immediate drop. Rejected snapshots are returned intact.
    pub fn accept(
        &mut self,
        snapshot: Snapshot,
    ) -> Result<Option<Snapshot>, (Reject, Box<Snapshot>)> {
        if let Err(error) = self.validate(&snapshot) {
            return Err((error, Box::new(snapshot)));
        }
        Ok(self.baseline.replace(snapshot))
    }
    pub fn clear_baseline(&mut self) -> Option<Snapshot> {
        self.baseline.take()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Returned {
        id: u64,
        sample: WindowSample,
    },
    Changed {
        id: u64,
        before: WindowSample,
        after: WindowSample,
    },
    NoLongerReturned {
        id: u64,
    },
}
/// Caller-driven fixed-delay polling entrance. Missed/delayed polls are not
/// replayed: next due is based on actual submission. No task or queue exists.
pub struct Poller {
    observer: Observer,
    cadence: Duration,
    due: Instant,
}
impl Poller {
    pub fn new(observer: Observer, cadence: Duration) -> Result<Self, &'static str> {
        if cadence.is_zero() {
            return Err("poll cadence must be nonzero");
        }
        Ok(Self {
            observer,
            cadence,
            due: Instant::now(),
        })
    }
    pub fn due(&self) -> Instant {
        self.due
    }
    pub fn observer(&self) -> &Observer {
        &self.observer
    }
    pub fn observer_mut(&mut self) -> &mut Observer {
        &mut self.observer
    }
    /// Sleeping before synchronous admission is cancellation-safe: no accepted
    /// operation exists if this future is dropped while waiting for cadence.
    pub async fn next(
        &mut self,
        timeout: Duration,
        max_response: usize,
    ) -> Result<Acquisition, Refused<Ls>> {
        tokio::time::sleep_until(self.due).await;
        let scheduled_for = self.due;
        let now = Instant::now();
        self.due = now + self.cadence;
        let mut acquisition = self.observer.acquire(Options {
            deadline: now + timeout,
            max_response,
        })?;
        acquisition.receipt.poll = Some(PollTiming {
            scheduled_for,
            cadence: self.cadence,
        });
        Ok(acquisition)
    }
    pub fn into_observer(self) -> Observer {
        self.observer
    }
}
