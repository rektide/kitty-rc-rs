//! Explicit, finite listing checks after an uncertain native effect.
//!
//! A [`CheckInput`] owns only fresh read/check input. The original result is
//! separately retained in registry custody until explicitly transferred through
//! [`RetainedOriginal::take`]. Neither matching nor observed satisfaction changes
//! that result, establishes causation, or permits replay. No task is started here.
use kitty_rc_exchange::Outcome;
use kitty_rc_listing::{
    Acquisition, AcquisitionFailure, Binding, Fact, Interpretation, MembershipBasis, Observer,
    Receipt, Scope, Snapshot,
};
use kitty_rc_registry::{
    CustodyError, Options, RefusalReason, Registry, SetupError, SetupVersion, Status, Summary,
    Token,
};
use std::time::Duration;
use tokio::time::Instant;

/// Bounded, caller-selected metadata/projection retention. Registry response and
/// admission quotas independently bound each native read. No history is retained.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_metadata_bytes: usize,
    pub max_criteria: usize,
    pub max_candidates: usize,
    pub max_windows: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_metadata_bytes: 16_384,
            max_criteria: 16,
            max_candidates: 8,
            max_windows: 16_384,
        }
    }
}

/// Local evidence boundary, not the remote effect's completion time. Binding is
/// a caller assertion; setup identifies local registry generations only.
#[derive(Debug, Clone)]
pub struct Boundary {
    token: Token,
    binding: Binding,
    setup: SetupVersion,
    captured: Instant,
}
impl Boundary {
    pub fn token(&self) -> Token {
        self.token
    }
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn setup(&self) -> SetupVersion {
        self.setup
    }
    pub fn captured(&self) -> Instant {
        self.captured
    }
}

/// Reference to one completed unresolved native occurrence. Drop detaches and
/// never releases; save its token or keep the registry for inventory recovery.
/// The wrapper retains no copy of native bytes or original mutation input.
#[must_use = "dropping does not release original evidence; save the token or explicitly take"]
pub struct RetainedOriginal {
    registry: Registry,
    summary: Summary,
    boundary: Boundary,
}
impl RetainedOriginal {
    /// Captures only a completed, still-untaken unresolved record. The caller's
    /// subject assertion must refer to the same endpoint and fit the given limit.
    pub fn capture(
        registry: &Registry,
        token: Token,
        binding: Binding,
        max_subject_bytes: usize,
    ) -> Result<Self, CaptureError> {
        if binding.subject.len() > max_subject_bytes {
            return Err(CaptureError::SubjectLimit);
        }
        let summary = registry.summary(token).ok_or(CaptureError::Missing)?;
        if summary.endpoint != binding.endpoint {
            return Err(CaptureError::EndpointMismatch);
        }
        if summary.status != Status::Completed || !summary.unresolved {
            return Err(CaptureError::NotRetainedUncertain);
        }
        let boundary = Boundary {
            token,
            binding,
            setup: summary.setup,
            captured: Instant::now(),
        };
        Ok(Self {
            registry: registry.clone(),
            summary,
            boundary,
        })
    }
    pub fn summary(&self) -> &Summary {
        &self.summary
    }
    pub fn boundary(&self) -> &Boundary {
        &self.boundary
    }
    pub fn check_input(&self, request: Request, limits: Limits) -> Result<CheckInput, InputError> {
        CheckInput::new(self.boundary.clone(), request, limits)
    }
    /// Sole native ownership transfer, then explicit release of its registry
    /// slot. On failed take the reference is returned intact. No decoding or
    /// satisfaction conversion is performed; the original evidence is unchanged.
    pub fn take(self) -> Result<OriginalEvidence, Box<TakeFailure>> {
        match self.registry.take_native(self.boundary.token) {
            Ok(native) => {
                let _ = self.registry.release(self.boundary.token);
                Ok(OriginalEvidence {
                    summary: self.summary,
                    boundary: self.boundary,
                    native,
                })
            }
            Err(error) => Err(Box::new(TakeFailure {
                original: self,
                error,
            })),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("original occurrence is missing or released")]
    Missing,
    #[error("original occurrence is not completed, unresolved and still retained")]
    NotRetainedUncertain,
    #[error("original endpoint differs from caller binding assertion")]
    EndpointMismatch,
    #[error("caller subject assertion exceeds selected byte limit")]
    SubjectLimit,
}
#[derive(Debug)]
pub struct OriginalEvidence {
    pub summary: Summary,
    pub boundary: Boundary,
    pub native: Outcome,
}
#[derive(thiserror::Error)]
#[error("original native evidence transfer failed: {error}")]
pub struct TakeFailure {
    pub original: RetainedOriginal,
    #[source]
    pub error: CustodyError,
}
impl std::fmt::Debug for TakeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TakeFailure")
            .field("token", &self.original.boundary.token)
            .field("error", &self.error)
            .finish()
    }
}

/// All criteria are conjoined. Missing required facts are indeterminate, not a
/// mismatch or known absence. A known mismatch overrides unrelated missing facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Criterion {
    WindowId(u64),
    Title(String),
    Pid(u64),
    Cwd(String),
    UserVar { key: String, value: String },
    Focused(bool),
    Active(bool),
}
#[derive(Debug, Clone)]
pub struct Request {
    /// Absence/cardinality is relative to this exact selection, never the desktop.
    pub scope: Scope,
    pub interpretation: Interpretation,
    pub criteria: Vec<Criterion>,
    pub condition: Condition,
    pub freshness: Freshness,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Present,
    Unique,
    Absent,
}
#[derive(Debug, Clone, Copy)]
pub struct Freshness {
    /// Conservative local age measured from acquisition start, including queue
    /// and waiter delay. This is not a remote snapshot clock or effect barrier.
    pub max_age: Duration,
}

/// The only continuation input: one `ls` selection plus native check criteria
/// and bounded evidence requirements. There is no generic Command, callback,
/// write body, owned native outcome, registry, retry timer or mutation entrance.
/// One finite explicitly selected journey (the original remains separately held):
/// ```
/// use kitty_rc_check::{CheckReport, Condition, Criterion, Freshness, Limits, Request, RetainedOriginal};
/// use kitty_rc_listing::{Interpretation, Scope, Support};
/// use kitty_rc_registry::{Options, Registry};
/// use std::time::Duration;
/// use tokio::time::Instant;
/// async fn once(registry: &Registry, original: &RetainedOriginal)
///     -> Result<CheckReport, Box<dyn std::error::Error>> {
///     let input = original.check_input(Request {
///         scope: Scope::default(),
///         interpretation: Interpretation { basis: "caller capability evidence".into(), focus: Support::Unknown },
///         criteria: vec![Criterion::Title("editor".into())],
///         condition: Condition::Unique,
///         freshness: Freshness { max_age: Duration::from_secs(5) },
///     }, Limits::default())?;
///     let check = input.start(registry, Options {
///         deadline: Instant::now() + Duration::from_secs(5), max_response: 16_384,
///     })?;
///     // Save check.receipt().clone() before detaching a waiter.
///     check.wait().await?;
///     Ok(check.finish()?)
/// }
/// ```
/// The continuation cannot be submitted as an arbitrary command:
/// ```compile_fail
/// use kitty_rc_check::CheckInput;
/// use kitty_rc_commands::Command;
/// fn submit<C: Command>(_: C) {}
/// fn cannot_submit(input: CheckInput) { submit(input); }
/// ```
/// Its admission entrance accepts read options, not a mutation body:
/// ```compile_fail
/// use kitty_rc_check::CheckInput;
/// use kitty_rc_commands::process::Launch;
/// use kitty_rc_registry::{Registry, Options};
/// fn cannot_replay(input: CheckInput, registry: &Registry, options: Options) {
///     input.start(registry, Launch::default(), options);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CheckInput {
    boundary: Boundary,
    request: Request,
    limits: Limits,
}
impl CheckInput {
    fn new(boundary: Boundary, request: Request, limits: Limits) -> Result<Self, InputError> {
        if limits.max_candidates == 0 || limits.max_windows == 0 {
            return Err(InputError::ZeroLimit);
        }
        if request.criteria.is_empty() || request.criteria.len() > limits.max_criteria {
            return Err(InputError::CriteriaLimit);
        }
        if request.freshness.max_age.is_zero() {
            return Err(InputError::ZeroFreshness);
        }
        let mut bytes = boundary.binding.subject.len();
        for value in [
            request
                .scope
                .match_window
                .as_ref()
                .map(|value| value.as_str()),
            request.scope.match_tab.as_ref().map(|value| value.as_str()),
            Some(request.interpretation.basis.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(value.len());
        }
        for criterion in &request.criteria {
            bytes = bytes.saturating_add(match criterion {
                Criterion::Title(value) | Criterion::Cwd(value) => value.len(),
                Criterion::UserVar { key, value } => key.len().saturating_add(value.len()),
                _ => 0,
            });
        }
        if bytes > limits.max_metadata_bytes {
            return Err(InputError::MetadataLimit);
        }
        Ok(Self {
            boundary,
            request,
            limits,
        })
    }
    pub fn boundary(&self) -> &Boundary {
        &self.boundary
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn limits(&self) -> Limits {
        self.limits
    }
    /// One synchronous read admission. No task is started and no original record
    /// is changed or released. Refusal returns this fresh input for caller choice.
    pub fn start(self, registry: &Registry, options: Options) -> Result<Check, Box<StartFailure>> {
        let current = match registry.setup_version(self.boundary.binding.endpoint) {
            Ok(current) => current,
            Err(error) => {
                return Err(Box::new(StartFailure {
                    input: self,
                    error: StartError::Setup(error),
                }));
            }
        };
        if current != self.boundary.setup {
            return Err(Box::new(StartFailure {
                input: self,
                error: StartError::SetupChanged,
            }));
        }
        let mut observer = Observer::new(
            registry.clone(),
            self.boundary.binding.clone(),
            self.request.scope.clone(),
            self.request.interpretation.clone(),
            kitty_rc_listing::Limits {
                max_windows: self.limits.max_windows,
            },
        );
        let acquisition = match observer.acquire(options) {
            Ok(acquisition) => acquisition,
            Err(refused) => {
                return Err(Box::new(StartFailure {
                    input: self,
                    error: StartError::Refused(refused.reason),
                }));
            }
        };
        let receipt = CheckReceipt {
            accepted_setup: registry
                .summary(acquisition.receipt().token())
                .map(|summary| summary.setup),
            listing: acquisition.receipt().clone(),
            input: self,
        };
        Ok(Check {
            acquisition,
            receipt,
            registry: registry.clone(),
        })
    }
    /// Inspect an existing listing without acquiring or accepting a baseline.
    /// Enforces declared binding, exact scope/interpretation, coverage and local
    /// freshness. Native registry setup is NOT verified by this entrance; use
    /// `start`/`CheckReceipt::finish` for that additional check.
    pub fn assess(&self, snapshot: &Snapshot) -> Assessment {
        let assessed = Instant::now();
        let receipt = snapshot.receipt();
        let insufficient = if receipt.binding() != &self.boundary.binding {
            Some(Insufficient::BindingChanged)
        } else if receipt.scope() != &self.request.scope {
            Some(Insufficient::ScopeChanged)
        } else if receipt.interpretation() != &self.request.interpretation {
            Some(Insufficient::InterpretationChanged)
        } else if receipt.started() <= self.boundary.captured {
            Some(Insufficient::AcquiredBeforeBoundary)
        } else if snapshot.ended() < receipt.started() || snapshot.ended() > assessed {
            Some(Insufficient::InvalidInterval)
        } else if assessed.duration_since(receipt.started()) > self.request.freshness.max_age {
            Some(Insufficient::Expired)
        } else if snapshot.coverage().membership != MembershipBasis::CompleteSelectedResponse {
            Some(Insufficient::PartialCoverage)
        } else {
            None
        };
        if let Some(reason) = insufficient {
            return Assessment::insufficient(reason, assessed, NativeSetup::NotChecked);
        }
        let mut candidates = Vec::new();
        let mut matched = 0usize;
        let mut indeterminate = 0usize;
        let mut evaluated = 0usize;
        for os in snapshot.listing() {
            for tab in &os.tabs {
                for window in &tab.windows {
                    // An independently acquired snapshot may have used a larger
                    // projection limit than this check's selected bound.
                    if evaluated == self.limits.max_windows {
                        return Assessment::insufficient(
                            Insufficient::WindowLimit,
                            assessed,
                            NativeSetup::NotChecked,
                        );
                    }
                    evaluated += 1;
                    let mut unknown = false;
                    let mut mismatch = false;
                    for criterion in &self.request.criteria {
                        let value = match criterion {
                            Criterion::WindowId(id) => window.id.map(|value| value == *id),
                            Criterion::Title(title) => {
                                window.title.as_ref().map(|value| value == title)
                            }
                            Criterion::Pid(pid) => window.pid.map(|value| value == *pid),
                            Criterion::Cwd(cwd) => window.cwd.as_ref().map(|value| value == cwd),
                            Criterion::UserVar { key, value } => {
                                window.user_vars.get(key).map(|actual| actual == value)
                            }
                            Criterion::Focused(wanted) => window
                                .id
                                .and_then(|id| snapshot.windows().get(&id))
                                .and_then(|sample| match sample.focused {
                                    Fact::Known(value) => Some(value == *wanted),
                                    _ => None,
                                }),
                            Criterion::Active(wanted) => {
                                window.is_active.map(|value| value == *wanted)
                            }
                        };
                        mismatch |= value == Some(false);
                        unknown |= value.is_none();
                    }
                    if mismatch {
                        continue;
                    }
                    if unknown {
                        indeterminate += 1;
                        continue;
                    }
                    let (Some(os_window), Some(tab), Some(window)) = (os.id, tab.id, window.id)
                    else {
                        indeterminate += 1;
                        continue;
                    };
                    matched += 1;
                    if candidates.len() < self.limits.max_candidates {
                        candidates.push(Candidate {
                            os_window,
                            tab,
                            window,
                        });
                    }
                }
            }
        }
        let truncated = matched > candidates.len();
        let matching = if indeterminate != 0 {
            Matching::EvidenceInsufficient {
                reason: Insufficient::MissingRequiredFacts,
                matched,
                indeterminate,
                candidates,
                truncated,
            }
        } else {
            match matched {
                0 => Matching::Absent,
                1 => Matching::Unique(candidates[0]),
                count => Matching::Ambiguous {
                    count,
                    candidates,
                    truncated,
                },
            }
        };
        let satisfaction = match (&matching, self.request.condition) {
            (Matching::Absent, Condition::Absent)
            | (Matching::Unique(_), Condition::Present | Condition::Unique)
            | (Matching::Ambiguous { .. }, Condition::Present) => Satisfaction::Satisfied,
            (Matching::Absent, Condition::Present | Condition::Unique)
            | (Matching::Unique(_) | Matching::Ambiguous { .. }, Condition::Absent) => {
                Satisfaction::Unsatisfied
            }
            _ => Satisfaction::Unknown,
        };
        Assessment {
            matching,
            satisfaction,
            attribution: Attribution::NotEstablished,
            native_setup: NativeSetup::NotChecked,
            assessed,
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("candidate and window limits must be nonzero")]
    ZeroLimit,
    #[error("check requires a nonempty, bounded criteria set")]
    CriteriaLimit,
    #[error("freshness age must be nonzero")]
    ZeroFreshness,
    #[error("check metadata exceeds selected byte limit")]
    MetadataLimit,
}
#[derive(Debug, thiserror::Error)]
pub enum StartError {
    #[error("local setup differs from original evidence basis")]
    SetupChanged,
    #[error(transparent)]
    Setup(#[from] SetupError),
    #[error("listing read admission refused: {0}")]
    Refused(#[source] RefusalReason),
}
#[derive(Debug, thiserror::Error)]
#[error("check admission failed: {error}")]
pub struct StartFailure {
    pub input: CheckInput,
    #[source]
    pub error: StartError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub os_window: u64,
    pub tab: u64,
    pub window: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Matching {
    Absent,
    Unique(Candidate),
    Ambiguous {
        count: usize,
        candidates: Vec<Candidate>,
        truncated: bool,
    },
    EvidenceInsufficient {
        reason: Insufficient,
        matched: usize,
        indeterminate: usize,
        candidates: Vec<Candidate>,
        truncated: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insufficient {
    BindingChanged,
    ScopeChanged,
    InterpretationChanged,
    /// Acquisition did not start strictly after the captured local boundary;
    /// equal clock readings cannot establish temporal freshness either.
    AcquiredBeforeBoundary,
    InvalidInterval,
    Expired,
    PartialCoverage,
    WindowLimit,
    MissingRequiredFacts,
    NativeSetupChanged,
    NativeSetupUnavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Satisfaction {
    Satisfied,
    Unsatisfied,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribution {
    /// A fresh listing is not a remote completion barrier. Even observed absence
    /// cannot rule out a pending late effect or a subsequent competing writer.
    NotEstablished,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSetup {
    NotChecked,
    Consistent(SetupVersion),
    Changed,
    Unavailable,
}
#[derive(Debug)]
pub struct Assessment {
    pub matching: Matching,
    pub satisfaction: Satisfaction,
    pub attribution: Attribution,
    pub native_setup: NativeSetup,
    pub assessed: Instant,
}
impl Assessment {
    fn insufficient(reason: Insufficient, assessed: Instant, native_setup: NativeSetup) -> Self {
        Self {
            matching: Matching::EvidenceInsufficient {
                reason,
                matched: 0,
                indeterminate: 0,
                candidates: Vec::new(),
                truncated: false,
            },
            satisfaction: Satisfaction::Unknown,
            attribution: Attribution::NotEstablished,
            native_setup,
            assessed,
        }
    }
}
#[derive(Debug)]
pub struct CheckReport {
    /// Bounded requirement/correlation metadata, never original mutation input
    /// or bytes. The separate original native result remains caller-owned.
    pub input: CheckInput,
    /// Intact read evidence; large listing trees are moved, not cloned.
    pub snapshot: Snapshot,
    pub assessment: Assessment,
}

/// Saveable recovery metadata for the fresh read only. Cloning authorizes no
/// write and no repeated take. Drop never cancels or releases accepted custody.
#[derive(Debug, Clone)]
pub struct CheckReceipt {
    input: CheckInput,
    listing: Receipt,
    accepted_setup: Option<SetupVersion>,
}
impl CheckReceipt {
    pub fn input(&self) -> &CheckInput {
        &self.input
    }
    pub fn listing(&self) -> &Receipt {
        &self.listing
    }
    pub fn token(&self) -> Token {
        self.listing.token()
    }
    /// Nonblocking sole read take after registry.wait. Transfers read evidence
    /// before release. Failure preserves the receipt and native acquisition error;
    /// a premature take leaves the unfinished record available for later recovery.
    pub fn finish(self, registry: &Registry) -> Result<CheckReport, Box<CheckFailure>> {
        let snapshot = match self.listing.clone().finish(registry) {
            Ok(snapshot) => snapshot,
            Err(failure) => {
                return Err(Box::new(CheckFailure {
                    receipt: self,
                    failure,
                }));
            }
        };
        let mut assessment = self.input.assess(&snapshot);
        // Revalidate after native transfer and decoding. This local check is
        // neither a lock/fence against subsequent rebinding nor remote CAS.
        let current_setup = registry.setup_version(self.input.boundary.binding.endpoint);
        match (self.accepted_setup, current_setup) {
            (Some(accepted), Ok(current))
                if accepted == self.input.boundary.setup && current == accepted =>
            {
                assessment.native_setup = NativeSetup::Consistent(accepted);
            }
            (None, _) | (_, Err(_)) => {
                assessment = Assessment::insufficient(
                    Insufficient::NativeSetupUnavailable,
                    assessment.assessed,
                    NativeSetup::Unavailable,
                );
            }
            _ => {
                assessment = Assessment::insufficient(
                    Insufficient::NativeSetupChanged,
                    assessment.assessed,
                    NativeSetup::Changed,
                );
            }
        }
        Ok(CheckReport {
            input: self.input,
            snapshot,
            assessment,
        })
    }
}
#[derive(Debug, thiserror::Error)]
#[error("fresh listing check failed: {failure}")]
pub struct CheckFailure {
    pub receipt: CheckReceipt,
    #[source]
    pub failure: Box<AcquisitionFailure>,
}
#[must_use = "accepted read custody persists; finish or save its receipt for recovery"]
pub struct Check {
    acquisition: Acquisition,
    receipt: CheckReceipt,
    registry: Registry,
}
impl Check {
    pub fn receipt(&self) -> &CheckReceipt {
        &self.receipt
    }
    pub async fn wait(&self) -> Result<(), CustodyError> {
        self.acquisition.wait().await
    }
    /// Explicit local read interruption, not original-operation cancellation,
    /// release, remote rollback or evidence of the effect's eventual completion.
    pub fn cancel(&self) -> bool {
        self.acquisition.cancel()
    }
    pub fn finish(self) -> Result<CheckReport, Box<CheckFailure>> {
        self.receipt.finish(&self.registry)
    }
}
