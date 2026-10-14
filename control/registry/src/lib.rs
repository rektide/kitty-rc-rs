//! Bounded managed operation custody. Submission is a synchronous ownership
//! cut; waiting is detachable. A caller-known token makes every accepted
//! occurrence discoverable even if its returned handle is never retained.
//!
//! Native wire outcomes are retained, not arbitrary custom decoded allocations.
//! `Operation::take` is the sole consuming typed decode, outside the lock. Typed
//! output memory then belongs to the caller. Release is explicit, never pruning.
use futures_util::{StreamExt, stream::FuturesUnordered};
use kitty_rc_commands::{Class, Command};
use kitty_rc_endpoint::Endpoint;
use kitty_rc_exchange::{
    Completion, Evidence, ExecutionError, Failure, Outcome, Phase, PrepareError, Prepared,
    Progress, prepare,
};
use kitty_rc_tokio::Session;
use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tokio::{sync::Notify, task::JoinHandle, time::Instant};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    owner: u64,
    sequence: u64,
}

/// Caller-owned submission occurrence. Its committed bit survives release of
/// the registry record, preventing a saved token from authorizing replay.
pub struct Reservation {
    token: Token,
    committed: AtomicBool,
}
impl Reservation {
    pub fn token(&self) -> Token {
        self.token
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointId {
    owner: u64,
    sequence: u64,
}

#[derive(Debug, Clone)]
pub struct Limits {
    pub endpoints: usize,
    /// Includes queued, running, and retained completed records.
    pub accepted: usize,
    pub accepted_per_endpoint: usize,
    /// Idle + active sockets + connecting attempts; each owns one slot.
    pub connections: usize,
    pub connections_per_endpoint: usize,
    /// Strict active/connecting partitions [short, observation, long-wait].
    /// Compatible idle sessions can serve any class without borrowing its quota.
    pub class_connections: [usize; 3],
    pub class_connections_per_endpoint: [usize; 3],
    pub input_bytes: usize,
    pub input_bytes_per_endpoint: usize,
    /// Reserved at acceptance, so slow consumers cannot overrun retention.
    pub output_bytes: usize,
    pub output_bytes_per_endpoint: usize,
    pub max_request: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            endpoints: 32,
            accepted: 128,
            accepted_per_endpoint: 32,
            connections: 8,
            connections_per_endpoint: 3,
            class_connections: [2, 3, 2],
            class_connections_per_endpoint: [1, 1, 1],
            input_bytes: 4 * 1024 * 1024,
            input_bytes_per_endpoint: 1024 * 1024,
            output_bytes: 16 * 1024 * 1024,
            output_bytes_per_endpoint: 8 * 1024 * 1024,
            max_request: 256 * 1024,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub deadline: Instant,
    pub max_response: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Queued,
    Running,
    Completed,
    Taken,
}
#[derive(Debug, Clone)]
pub struct Summary {
    pub token: Token,
    pub endpoint: EndpointId,
    pub status: Status,
    pub progress: Progress,
    pub unresolved: bool,
}
#[derive(Debug)]
pub struct PoolStats {
    pub idle: usize,
    pub active_or_connecting: usize,
    pub endpoints: BTreeMap<EndpointId, (usize, usize)>,
    pub idle_tokens: BTreeMap<EndpointId, Vec<kitty_rc_tokio::ConnectionToken>>,
}
#[derive(Debug)]
pub struct Refused<C> {
    pub command: C,
    pub reason: RefusalReason,
}
#[derive(Debug, thiserror::Error)]
pub enum RefusalReason {
    #[error("intake closed")]
    Closed,
    #[error("foreign submission token")]
    ForeignToken,
    #[error("response reservation must be at least 256 bytes")]
    ResponseReservation,
    #[error("unknown endpoint")]
    UnknownEndpoint,
    #[error("token already accepted")]
    AlreadyAccepted,
    #[error("deadline elapsed before acceptance")]
    Deadline,
    #[error("accepted-record quota full")]
    AcceptedCapacity,
    #[error("input-byte quota full")]
    InputCapacity,
    #[error("output-retention quota full")]
    OutputCapacity,
    #[error("submission occurrence already committed")]
    Committed,
    #[error("preparation failed: {0}")]
    Prepare(#[from] PrepareError),
}
#[derive(Debug, thiserror::Error)]
pub enum CustodyError {
    #[error("unknown or released operation")]
    Missing,
    #[error("operation unfinished or result already taken")]
    Unavailable,
    #[error("operation unfinished")]
    Unfinished,
}
#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("driver stopped with unfinished work; retained inventory has {} operations", inventory.len())]
    DriverStopped { inventory: Vec<Summary> },
}
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("invalid class/connection limits")]
    InvalidLimits,
    #[error("endpoint setup byte limit exceeded")]
    EndpointSetupLimit,
    #[error("intake closed")]
    Closed,
    #[error("endpoint capacity full")]
    EndpointCapacity,
    #[error("driver already attached")]
    DriverAttached,
}

struct Record {
    endpoint: EndpointId,
    class: usize,
    deadline: Instant,
    max_response: usize,
    input: Option<Prepared>,
    input_bytes: usize,
    reservation: usize,
    status: Status,
    evidence: Evidence,
    outcome: Option<Outcome>,
    unresolved: bool,
    generation: u64,
    cancel: Arc<Notify>,
    done: Arc<Notify>,
}
struct State {
    endpoints: BTreeMap<EndpointId, Endpoint>,
    records: BTreeMap<Token, Record>,
    closed: bool,
    driver: bool,
    generation: u64,
    cursor: Option<EndpointId>,
    idle: BTreeMap<EndpointId, Vec<Session>>,
}
struct Shared {
    owner: u64,
    sequence: AtomicU64,
    limits: Limits,
    state: Mutex<State>,
    wake: Notify,
    lifecycle: Notify,
}
#[derive(Clone)]
pub struct Registry {
    shared: Arc<Shared>,
}
impl Registry {
    pub fn pool_stats(&self) -> PoolStats {
        let s = self.shared.state.lock().unwrap();
        let endpoints: BTreeMap<_, _> = s
            .endpoints
            .keys()
            .map(|&id| {
                (
                    id,
                    (
                        s.idle.get(&id).map_or(0, Vec::len),
                        s.records
                            .values()
                            .filter(|r| r.endpoint == id && r.status == Status::Running)
                            .count(),
                    ),
                )
            })
            .collect();
        let idle_tokens = s
            .idle
            .iter()
            .map(|(&id, sessions)| {
                (
                    id,
                    sessions
                        .iter()
                        .filter_map(Session::connection_token)
                        .collect(),
                )
            })
            .collect();
        PoolStats {
            idle: endpoints.values().map(|(idle, _)| idle).sum(),
            active_or_connecting: endpoints.values().map(|(_, active)| active).sum(),
            endpoints,
            idle_tokens,
        }
    }
    pub fn new(limits: Limits) -> Result<Self, RegistryError> {
        let total = |classes: &[usize; 3]| {
            classes
                .iter()
                .try_fold(0usize, |sum, value| sum.checked_add(*value))
        };
        if total(&limits.class_connections).is_none_or(|sum| sum > limits.connections)
            || total(&limits.class_connections_per_endpoint)
                .is_none_or(|sum| sum > limits.connections_per_endpoint)
            || limits.connections == 0
            || limits.connections_per_endpoint == 0
            || limits.max_request == 0
        {
            return Err(RegistryError::InvalidLimits);
        }
        let owner = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
        assert!(owner != u64::MAX, "owner identity exhausted");
        Ok(Self {
            shared: Arc::new(Shared {
                owner,
                sequence: AtomicU64::new(1),
                limits,
                state: Mutex::new(State {
                    endpoints: BTreeMap::new(),
                    records: BTreeMap::new(),
                    closed: false,
                    driver: false,
                    generation: 0,
                    cursor: None,
                    idle: BTreeMap::new(),
                }),
                wake: Notify::new(),
                lifecycle: Notify::new(),
            }),
        })
    }
    fn mint(&self) -> Token {
        let sequence = self.shared.sequence.fetch_add(1, Ordering::Relaxed);
        assert!(sequence != u64::MAX, "token exhausted");
        Token {
            owner: self.shared.owner,
            sequence,
        }
    }

    pub fn reserve(&self) -> Reservation {
        Reservation {
            token: self.mint(),
            committed: AtomicBool::new(false),
        }
    }
    pub fn register(&self, mut endpoint: Endpoint) -> Result<EndpointId, RegistryError> {
        let address_bytes = match &endpoint.address {
            kitty_rc_endpoint::Address::Unix(path) => path.as_os_str().as_encoded_bytes().len(),
            kitty_rc_endpoint::Address::Tcp(_) => 32,
        };
        let access_bytes = match &endpoint.protection {
            kitty_rc_endpoint::Protection::Plain => 0,
            kitty_rc_endpoint::Protection::Encrypted {
                password,
                key_origin,
                ..
            } => password
                .len()
                .saturating_add(key_origin.len())
                .saturating_add(32),
        };
        if address_bytes.saturating_add(access_bytes) > self.shared.limits.max_request {
            return Err(RegistryError::EndpointSetupLimit);
        }
        if let kitty_rc_endpoint::Address::Unix(path) = &mut endpoint.address {
            path.shrink_to_fit();
        }
        if let kitty_rc_endpoint::Protection::Encrypted {
            password,
            key_origin,
            ..
        } = &mut endpoint.protection
        {
            password.shrink_to_fit();
            key_origin.shrink_to_fit();
        }
        let mut state = self.shared.state.lock().unwrap();
        if state.closed {
            return Err(RegistryError::Closed);
        }
        if state.endpoints.len() == self.shared.limits.endpoints {
            return Err(RegistryError::EndpointCapacity);
        }
        let t = self.mint();
        let id = EndpointId {
            owner: t.owner,
            sequence: t.sequence,
        };
        state.endpoints.insert(id, endpoint);
        Ok(id)
    }
    /// No await occurs between input installation and handle delivery. Refusal
    /// preserves the original command. Token reuse never replays an operation.
    pub fn submit<C: Command>(
        &self,
        reservation: &Reservation,
        endpoint: EndpointId,
        command: C,
        options: Options,
    ) -> Result<Operation<C>, Refused<C>> {
        let token = reservation.token;
        let setup = self
            .shared
            .state
            .lock()
            .unwrap()
            .endpoints
            .get(&endpoint)
            .cloned();
        let prepared = (|| {
            if token.owner != self.shared.owner {
                return Err(RefusalReason::ForeignToken);
            }
            if options.max_response < 256 {
                return Err(RefusalReason::ResponseReservation);
            }
            let setup = setup.as_ref().ok_or(RefusalReason::UnknownEndpoint)?;
            prepare(&command, &setup.protection, self.shared.limits.max_request)
                .map_err(RefusalReason::Prepare)
        })();
        let input = match prepared {
            Ok(p) => p,
            Err(reason) => return Err(Refused { command, reason }),
        };
        let class = class_index(command.class());
        let cause_bytes = match &setup.as_ref().unwrap().address {
            kitty_rc_endpoint::Address::Unix(path) => path.as_os_str().as_encoded_bytes().len(),
            kitty_rc_endpoint::Address::Tcp(_) => 32,
        };
        let output_reservation = options.max_response.saturating_add(cause_bytes);
        let mut state = self.shared.state.lock().unwrap();
        let l = &self.shared.limits;
        let records = state.records.values();
        let ep_count = records.clone().filter(|r| r.endpoint == endpoint).count();
        let input_total = records.clone().map(|r| r.input_bytes).sum::<usize>();
        let output_total = records.clone().map(|r| r.reservation).sum::<usize>();
        let ep_input = records
            .clone()
            .filter(|r| r.endpoint == endpoint)
            .map(|r| r.input_bytes)
            .sum::<usize>();
        let ep_output = records
            .filter(|r| r.endpoint == endpoint)
            .map(|r| r.reservation)
            .sum::<usize>();
        let reason = if state.closed {
            Some(RefusalReason::Closed)
        } else if state.records.contains_key(&token) {
            Some(RefusalReason::AlreadyAccepted)
        } else if options.deadline <= Instant::now() {
            Some(RefusalReason::Deadline)
        } else if state.records.len() >= l.accepted || ep_count >= l.accepted_per_endpoint {
            Some(RefusalReason::AcceptedCapacity)
        } else if input.reservation_bytes() > l.input_bytes.saturating_sub(input_total)
            || input.reservation_bytes() > l.input_bytes_per_endpoint.saturating_sub(ep_input)
        {
            Some(RefusalReason::InputCapacity)
        } else if output_reservation > l.output_bytes.saturating_sub(output_total)
            || output_reservation > l.output_bytes_per_endpoint.saturating_sub(ep_output)
        {
            Some(RefusalReason::OutputCapacity)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(Refused { command, reason });
        }
        if reservation.committed.swap(true, Ordering::SeqCst) {
            return Err(Refused {
                command,
                reason: RefusalReason::Committed,
            });
        }
        let done = Arc::new(Notify::new());
        let input_bytes = input.reservation_bytes();
        state.records.insert(
            token,
            Record {
                endpoint,
                class,
                deadline: options.deadline,
                max_response: options.max_response,
                input: Some(input),
                input_bytes,
                reservation: output_reservation,
                status: Status::Queued,
                evidence: Evidence::default(),
                outcome: None,
                unresolved: false,
                generation: 0,
                cancel: Arc::new(Notify::new()),
                done,
            },
        );
        drop(state);
        self.shared.wake.notify_one();
        Ok(Operation {
            registry: self.clone(),
            token,
            marker: PhantomData,
        })
    }
    pub fn summary(&self, token: Token) -> Option<Summary> {
        self.shared
            .state
            .lock()
            .unwrap()
            .records
            .get(&token)
            .map(|r| Summary {
                token,
                endpoint: r.endpoint,
                status: r.status,
                progress: r.evidence.progress(),
                unresolved: r.unresolved,
            })
    }
    pub fn inventory(&self) -> Vec<Summary> {
        self.shared
            .state
            .lock()
            .unwrap()
            .records
            .iter()
            .map(|(&token, r)| Summary {
                token,
                endpoint: r.endpoint,
                status: r.status,
                progress: r.evidence.progress(),
                unresolved: r.unresolved,
            })
            .collect()
    }
    pub fn cancel(&self, token: Token) -> bool {
        let mut state = self.shared.state.lock().unwrap();
        let Some(r) = state.records.get_mut(&token) else {
            return false;
        };
        match r.status {
            Status::Queued => complete(
                r,
                Outcome::Interrupted {
                    progress: Progress::NotWritten,
                    reason: ExecutionError::Cancelled {
                        phase: Phase::Queue,
                    },
                },
            ),
            Status::Running => r.cancel.notify_one(),
            _ => return false,
        }
        self.shared.wake.notify_one();
        self.shared.lifecycle.notify_waiters();
        true
    }
    /// Explicitly release completed native output and its admission slot.
    pub fn release(&self, token: Token) -> Result<(), CustodyError> {
        let mut s = self.shared.state.lock().unwrap();
        match s.records.get(&token) {
            Some(r) if matches!(r.status, Status::Completed | Status::Taken) => {
                s.records.remove(&token);
                Ok(())
            }
            Some(_) => Err(CustodyError::Unfinished),
            None => Err(CustodyError::Missing),
        }
    }
    /// Recover native evidence using the caller-known token, without typed decode.
    pub fn take_native(&self, token: Token) -> Result<Outcome, CustodyError> {
        let mut state = self.shared.state.lock().unwrap();
        let r = state.records.get_mut(&token).ok_or(CustodyError::Missing)?;
        let outcome = r.outcome.take().ok_or(CustodyError::Unavailable)?;
        r.status = Status::Taken;
        r.reservation = 0;
        Ok(outcome)
    }
    pub async fn wait(&self, token: Token) -> Result<(), CustodyError> {
        loop {
            let done = self
                .shared
                .state
                .lock()
                .unwrap()
                .records
                .get(&token)
                .map(|r| r.done.clone())
                .ok_or(CustodyError::Missing)?;
            let notified = done.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let status = self.summary(token).ok_or(CustodyError::Missing)?.status;
            if matches!(status, Status::Completed | Status::Taken) {
                return Ok(());
            }
            notified.await;
        }
    }
    /// Driver is an explicitly owned task; dropping Driver aborts it. The owner
    /// retains unfinished queued work and uncertain running outcomes.
    pub fn start(&self) -> Result<Driver, RegistryError> {
        let mut s = self.shared.state.lock().unwrap();
        if s.driver {
            return Err(RegistryError::DriverAttached);
        }
        s.driver = true;
        s.generation += 1;
        let generation = s.generation;
        drop(s);
        let shared = self.shared.clone();
        // Guard exists before spawn so even abort-before-first-poll recovers.
        let guard = DriverGuard {
            shared: shared.clone(),
            generation,
        };
        let task = tokio::spawn(async move {
            let _guard = guard;
            driver_loop(shared, generation).await;
        });
        Ok(Driver { task: Some(task) })
    }
    pub fn close(&self) {
        self.shared.state.lock().unwrap().closed = true;
        self.shared.wake.notify_one();
    }
    /// Close intake, then deliberately drain or cancel. Drain needs an attached
    /// driver; return all unknown outcomes, not a false remote rollback claim.
    pub async fn shutdown(&self, policy: Shutdown) -> Result<Vec<Summary>, ShutdownError> {
        self.close();
        if policy == Shutdown::Cancel {
            for s in self.inventory() {
                self.cancel(s.token);
            }
        }
        loop {
            let changed = self.shared.lifecycle.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let (attached, unfinished) = {
                let s = self.shared.state.lock().unwrap();
                (
                    s.driver,
                    s.records
                        .values()
                        .any(|r| matches!(r.status, Status::Queued | Status::Running)),
                )
            };
            if !unfinished {
                let idle = std::mem::take(&mut self.shared.state.lock().unwrap().idle);
                drop(idle); // Close transports outside the owner lock.
                return Ok(self
                    .inventory()
                    .into_iter()
                    .filter(|s| s.unresolved)
                    .collect());
            }
            if !attached {
                return Err(ShutdownError::DriverStopped {
                    inventory: self.inventory(),
                });
            }
            changed.await;
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shutdown {
    Drain,
    Cancel,
}
pub struct Operation<C: Command> {
    registry: Registry,
    token: Token,
    marker: PhantomData<fn() -> C>,
}
impl<C: Command> Operation<C> {
    pub fn token(&self) -> Token {
        self.token
    }
    pub async fn wait(&self) -> Result<(), CustodyError> {
        self.registry.wait(self.token).await
    }
    pub fn cancel(&self) -> bool {
        self.registry.cancel(self.token)
    }
    /// Sole typed take; no Clone bound. The native bytes and output reservation
    /// leave custody before synchronous decoding. Error retains native evidence.
    pub fn take(&self) -> Result<Completion<C::Output>, Failure> {
        self.registry
            .take_native(self.token)
            .map_err(|e| Failure::Unavailable(Box::new(e)))?
            .decode::<C>()
    }
    pub fn release(self) -> Result<(), CustodyError> {
        self.registry.release(self.token)
    }
}
pub struct Driver {
    task: Option<JoinHandle<()>>,
}
impl Driver {
    pub async fn stop(mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
            let _ = task.await;
        }
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
struct DriverGuard {
    shared: Arc<Shared>,
    generation: u64,
}
impl Drop for DriverGuard {
    fn drop(&mut self) {
        let mut s = self.shared.state.lock().unwrap();
        if s.generation != self.generation {
            return;
        }
        for r in s
            .records
            .values_mut()
            .filter(|r| r.status == Status::Running && r.generation == self.generation)
        {
            complete(
                r,
                Outcome::Interrupted {
                    progress: r.evidence.progress(),
                    reason: ExecutionError::DriverStopped {
                        phase: r.evidence.phase(),
                    },
                },
            );
        }
        s.driver = false;
        self.shared.wake.notify_one();
        self.shared.lifecycle.notify_waiters();
    }
}
fn class_index(c: Class) -> usize {
    match c {
        Class::Short => 0,
        Class::Observation => 1,
        Class::LongWait => 2,
    }
}
fn complete(r: &mut Record, outcome: Outcome) {
    r.unresolved = outcome.unresolved();
    r.input = None;
    r.input_bytes = 0;
    r.outcome = Some(outcome);
    r.status = Status::Completed;
    // Storage precedes notification, under the same ownership turn.
    r.done.notify_waiters();
}
struct Work {
    token: Token,
    endpoint_id: EndpointId,
    session: Session,
    retired: Option<Session>,
    prepared: Prepared,
    deadline: Instant,
    max_response: usize,
    evidence: Evidence,
    cancel: Arc<Notify>,
}
fn select(shared: &Shared, generation: u64) -> Option<Work> {
    let mut s = shared.state.lock().unwrap();
    for r in s
        .records
        .values_mut()
        .filter(|r| r.status == Status::Queued && r.deadline <= Instant::now())
    {
        complete(
            r,
            Outcome::Interrupted {
                progress: Progress::NotWritten,
                reason: ExecutionError::Deadline {
                    phase: Phase::Queue,
                },
            },
        );
        shared.lifecycle.notify_waiters();
    }
    let l = &shared.limits;
    let running: Vec<_> = s
        .records
        .values()
        .filter(|r| r.status == Status::Running)
        .map(|r| (r.endpoint, r.class))
        .collect();
    if running.len() >= l.connections {
        return None;
    }
    let mut endpoints: Vec<_> = s.endpoints.keys().copied().collect();
    if let Some(cursor) = s.cursor {
        let split = endpoints.partition_point(|id| *id <= cursor);
        endpoints.rotate_left(split);
    }
    for endpoint in endpoints {
        if running.iter().filter(|(ep, _)| *ep == endpoint).count() >= l.connections_per_endpoint {
            continue;
        }
        for class in 0..3 {
            if running.iter().filter(|(_, c)| *c == class).count() >= l.class_connections[class]
                || running
                    .iter()
                    .filter(|(ep, c)| *ep == endpoint && *c == class)
                    .count()
                    >= l.class_connections_per_endpoint[class]
            {
                continue;
            }
            let token = s
                .records
                .iter()
                .find(|(_, r)| {
                    r.endpoint == endpoint && r.class == class && r.status == Status::Queued
                })
                .map(|(token, _)| *token);
            if let Some(token) = token {
                let setup = s.endpoints[&endpoint].clone();
                let mut retired = None;
                let session =
                    if let Some(session) = s.idle.get_mut(&endpoint).and_then(|idle| idle.pop()) {
                        session
                    } else {
                        let idle_count = s.idle.values().map(Vec::len).sum::<usize>();
                        if running.len() + idle_count >= l.connections {
                            // Demand-driven capacity transfer, not TTL pruning.
                            let victim = s
                                .idle
                                .iter()
                                .find(|(_, sockets)| !sockets.is_empty())
                                .map(|(id, _)| *id);
                            if let Some(victim) = victim {
                                retired = s.idle.get_mut(&victim).unwrap().pop();
                            } else {
                                continue;
                            }
                        }
                        Session::new(setup)
                    };
                s.cursor = Some(endpoint);
                let r = s.records.get_mut(&token).unwrap();
                r.status = Status::Running;
                r.generation = generation;
                let prepared = r.input.take().expect("queued input owned");
                // Input byte quota stays reserved until the driving task returns.
                return Some(Work {
                    token,
                    endpoint_id: endpoint,
                    session,
                    retired,
                    prepared,
                    deadline: r.deadline,
                    max_response: r.max_response,
                    evidence: r.evidence.clone(),
                    cancel: r.cancel.clone(),
                });
            }
        }
    }
    None
}
async fn driver_loop(shared: Arc<Shared>, generation: u64) {
    // Futures are owned directly, not detached child tasks: driver destruction
    // synchronously drops every socket before DriverGuard releases slots.
    let mut tasks = FuturesUnordered::new();
    loop {
        let wake = shared.wake.notified();
        tokio::pin!(wake);
        wake.as_mut().enable();
        while let Some(mut work) = select(&shared, generation) {
            // The transferred lease still owns its old idle transport until
            // this close; no new connection starts before it is retired.
            drop(work.retired.take());
            tasks.push(async move {
                let evidence = work.evidence.clone();
                let outcome = tokio::select! {
                    biased;
                    _ = work.cancel.notified() => Outcome::Interrupted { progress: evidence.progress(), reason: ExecutionError::Cancelled { phase: evidence.phase() } },
                    outcome = work.session.execute_prepared(work.prepared, work.deadline, work.max_response, work.evidence) => outcome,
                };
                (work.token, work.endpoint_id, work.session, outcome)
            });
        }
        let next_deadline = shared
            .state
            .lock()
            .unwrap()
            .records
            .values()
            .filter(|r| r.status == Status::Queued)
            .map(|r| r.deadline)
            .min();
        tokio::select! {
            _ = &mut wake => {},
            _ = async { match next_deadline { Some(d) => tokio::time::sleep_until(d).await, None => std::future::pending().await } } => {},
            returned = tasks.next(), if !tasks.is_empty() => {
                if let Some((token, endpoint, session, outcome)) = returned {
                        let idle = if session.is_connected() && matches!(&outcome, Outcome::Reply(_)) && !outcome.unresolved() { Some(session) } else { drop(session); None };
                        let mut s = shared.state.lock().unwrap();
                        if let Some(r) = s.records.get_mut(&token) { if r.status == Status::Running && r.generation == generation { complete(r, outcome); if let Some(session) = idle { s.idle.entry(endpoint).or_default().push(session); } } }
                        shared.lifecycle.notify_waiters();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kitty_rc_commands::{FocusWindow, GetText, LaunchWait};
    use kitty_rc_endpoint::{Address, Protection};
    #[test]
    fn connecting_reservations_are_exclusive_fair_and_class_partitioned() {
        let registry = Registry::new(Limits::default()).unwrap();
        let endpoint = || Endpoint {
            address: Address::Unix("/explicit/nonexistent".into()),
            protection: Protection::Plain,
        };
        let a = registry.register(endpoint()).unwrap();
        let b = registry.register(endpoint()).unwrap();
        let opts = Options {
            deadline: Instant::now() + std::time::Duration::from_secs(5),
            max_response: 256,
        };
        for ep in [a, b] {
            for _ in 0..2 {
                registry
                    .submit(
                        &registry.reserve(),
                        ep,
                        FocusWindow {
                            match_spec: "id:1".into(),
                        },
                        opts,
                    )
                    .unwrap();
                registry
                    .submit(&registry.reserve(), ep, GetText::default(), opts)
                    .unwrap();
                registry
                    .submit(&registry.reserve(), ep, LaunchWait { args: vec![] }, opts)
                    .unwrap();
            }
        }
        // Select without ever polling connect: connecting resources already
        // own their slots. No executor slot waits inside a lease semaphore.
        let mut selected = Vec::new();
        while let Some(work) = select(&registry.shared, 1) {
            selected.push(work);
        }
        assert_eq!(selected.len(), 6);
        assert_eq!(registry.summary(selected[0].token).unwrap().endpoint, a);
        assert_eq!(registry.summary(selected[1].token).unwrap().endpoint, b);
        let state = registry.shared.state.lock().unwrap();
        for ep in [a, b] {
            for class in 0..3 {
                assert_eq!(
                    state
                        .records
                        .values()
                        .filter(|r| r.endpoint == ep
                            && r.class == class
                            && r.status == Status::Running)
                        .count(),
                    1
                );
            }
        }
    }
}
