//! Bounded managed operation custody. Submission is a synchronous ownership
//! cut; waiting is detachable. A caller-known token makes every accepted
//! occurrence discoverable even if its returned handle is never retained.
//!
//! Native wire outcomes are retained, not arbitrary custom decoded allocations.
//! `Operation::take` is the sole consuming typed decode, outside the lock. Typed
//! output memory then belongs to the caller. Release is explicit, never pruning.
use kitty_rc_commands::{Class, Command};
use kitty_rc_endpoint::Endpoint;
use kitty_rc_exchange::{Completion, Evidence, Failure, Outcome, Prepared, Progress, prepare};
use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tokio::{
    sync::Notify,
    task::{JoinHandle, JoinSet},
    time::Instant,
};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    owner: u64,
    sequence: u64,
}

/// Caller-owned submission occurrence. Its committed bit survives release of
/// the registry record, preventing a saved token from authorizing replay.
pub struct Reservation { token: Token, committed: AtomicBool }
impl Reservation { pub fn token(&self) -> Token { self.token } }
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
    /// Active sockets plus connecting attempts, which each own one slot.
    pub connections: usize,
    pub connections_per_endpoint: usize,
    /// Strict partitions [short, observation, long-wait]; no borrowing.
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
pub struct Refused<C> {
    pub command: C,
    pub reason: String,
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
}
struct Shared {
    owner: u64,
    sequence: AtomicU64,
    limits: Limits,
    state: Mutex<State>,
    wake: Notify,
}
#[derive(Clone)]
pub struct Registry {
    shared: Arc<Shared>,
}
impl Registry {
    pub fn new(limits: Limits) -> Result<Self, String> {
        if limits.class_connections.iter().sum::<usize>() > limits.connections
            || limits.class_connections_per_endpoint.iter().sum::<usize>()
                > limits.connections_per_endpoint
            || limits.connections == 0
            || limits.connections_per_endpoint == 0
            || limits.max_request == 0
        {
            return Err("invalid class/connection limits".into());
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
                }),
                wake: Notify::new(),
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

    pub fn reserve(&self) -> Reservation { Reservation { token: self.mint(), committed: AtomicBool::new(false) } }
    pub fn register(&self, endpoint: Endpoint) -> Result<EndpointId, String> {
        let mut state = self.shared.state.lock().unwrap();
        if state.closed || state.endpoints.len() == self.shared.limits.endpoints {
            return Err("endpoint intake closed or full".into());
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
                return Err("foreign submission token".into());
            }
            if options.max_response < 256 {
                return Err("response reservation must be at least 256 bytes".into());
            }
            let setup = setup.ok_or("unknown endpoint")?;
            prepare(
                &command,
                &setup.protection,
                format!("{}-{}", token.owner, token.sequence),
                kitty_rc_tokio::timestamp_ns()?,
                self.shared.limits.max_request,
            )
        })();
        let input = match prepared {
            Ok(p) => p,
            Err(reason) => return Err(Refused { command, reason }),
        };
        let class = class_index(command.class());
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
            Some("intake closed")
        } else if state.records.contains_key(&token) {
            Some("token already accepted")
        } else if options.deadline <= Instant::now() {
            Some("deadline elapsed before acceptance")
        } else if state.records.len() >= l.accepted || ep_count >= l.accepted_per_endpoint {
            Some("accepted-record quota full")
        } else if input.bytes.len() > l.input_bytes.saturating_sub(input_total)
            || input.bytes.len() > l.input_bytes_per_endpoint.saturating_sub(ep_input)
        {
            Some("input-byte quota full")
        } else if options.max_response > l.output_bytes.saturating_sub(output_total)
            || options.max_response > l.output_bytes_per_endpoint.saturating_sub(ep_output)
        {
            Some("output-retention quota full")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(Refused {
                command,
                reason: reason.into(),
            });
        }
        if reservation.committed.swap(true, Ordering::SeqCst) { return Err(Refused { command, reason: "submission occurrence already committed".into() }); }
        let done = Arc::new(Notify::new());
        let input_bytes = input.bytes.len();
        state.records.insert(
            token,
            Record {
                endpoint,
                class,
                deadline: options.deadline,
                max_response: options.max_response,
                input: Some(input),
                input_bytes,
                reservation: options.max_response,
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
                    reason: "queued cancellation".into(),
                },
            ),
            Status::Running => r.cancel.notify_one(),
            _ => return false,
        }
        self.shared.wake.notify_one();
        true
    }
    /// Explicitly release completed native output and its admission slot.
    pub fn release(&self, token: Token) -> Result<(), String> {
        let mut s = self.shared.state.lock().unwrap();
        match s.records.get(&token) {
            Some(r) if matches!(r.status, Status::Completed | Status::Taken) => {
                s.records.remove(&token);
                Ok(())
            }
            Some(_) => Err("operation unfinished".into()),
            None => Err("unknown operation".into()),
        }
    }
    /// Recover native evidence using the caller-known token, without typed decode.
    pub fn take_native(&self, token: Token) -> Result<Outcome, String> {
        let mut state = self.shared.state.lock().unwrap();
        let r = state.records.get_mut(&token).ok_or("unknown operation")?;
        let outcome = r.outcome.take().ok_or("unfinished or already taken")?;
        r.status = Status::Taken;
        r.reservation = 0;
        Ok(outcome)
    }
    pub async fn wait(&self, token: Token) -> Result<(), String> {
        loop {
            let done = self
                .shared
                .state
                .lock()
                .unwrap()
                .records
                .get(&token)
                .map(|r| r.done.clone())
                .ok_or("unknown operation")?;
            let notified = done.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let status = self.summary(token).ok_or("released operation")?.status;
            if matches!(status, Status::Completed | Status::Taken) {
                return Ok(());
            }
            notified.await;
        }
    }
    /// Driver is an explicitly owned task; dropping Driver aborts it. The owner
    /// retains unfinished queued work and uncertain running outcomes.
    pub fn start(&self) -> Result<Driver, String> {
        let mut s = self.shared.state.lock().unwrap();
        if s.driver {
            return Err("driver already attached".into());
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
    pub async fn shutdown(&self, policy: Shutdown) -> Result<Vec<Summary>, String> {
        self.close();
        if policy == Shutdown::Cancel {
            for s in self.inventory() {
                self.cancel(s.token);
            }
        }
        if !self.shared.state.lock().unwrap().driver
            && self
                .inventory()
                .iter()
                .any(|s| matches!(s.status, Status::Queued | Status::Running))
        {
            return Err(
                "unfinished work without attached driver; inventory remains available".into(),
            );
        }
        for s in self.inventory() {
            self.wait(s.token).await?;
        }
        Ok(self
            .inventory()
            .into_iter()
            .filter(|s| s.unresolved)
            .collect())
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
    pub async fn wait(&self) -> Result<(), String> {
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
            .map_err(Failure::InvalidReply)?
            .decode::<C>()
    }
    pub fn release(self) -> Result<(), String> {
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
                    reason: "driver stopped; never replayed".into(),
                },
            );
        }
        s.driver = false;
        self.shared.wake.notify_one();
    }
}
fn class_index(c: Class) -> usize {
    match c {
        Class::Short => 0,
        Class::Observation => 1,
        Class::LongWait => 2,
    }
}
fn complete(r: &mut Record, mut outcome: Outcome) {
    if let Outcome::Interrupted { reason, .. } = &mut outcome {
        let mut end = reason.len().min(256);
        while !reason.is_char_boundary(end) { end -= 1; }
        reason.truncate(end);
        reason.shrink_to_fit();
    }
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
    endpoint: Endpoint,
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
                reason: "queue deadline elapsed".into(),
            },
        );
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
                s.cursor = Some(endpoint);
                let r = s.records.get_mut(&token).unwrap();
                r.status = Status::Running;
                r.generation = generation;
                let prepared = r.input.take().expect("queued input owned");
                // Input byte quota stays reserved until the driving task returns.
                return Some(Work {
                    token,
                    endpoint: setup,
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
    let mut tasks = JoinSet::new();
    loop {
        let wake = shared.wake.notified();
        tokio::pin!(wake);
        wake.as_mut().enable();
        while let Some(work) = select(&shared, generation) {
            tasks.spawn(async move {
                let evidence = work.evidence.clone();
                let outcome = tokio::select! {
                    biased;
                    _ = work.cancel.notified() => Outcome::Interrupted { progress: evidence.progress(), reason: "explicit cancellation".into() },
                    outcome = kitty_rc_tokio::drive(&work.endpoint.address, work.prepared, work.deadline, work.max_response, work.evidence) => outcome,
                };
                (work.token, outcome)
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
            returned = tasks.join_next(), if !tasks.is_empty() => {
                match returned {
                    Some(Ok((token, outcome))) => {
                        let mut s = shared.state.lock().unwrap();
                        if let Some(r) = s.records.get_mut(&token) { if r.status == Status::Running && r.generation == generation { complete(r, outcome); } }
                    },
                    // Panic/abort is driver loss, not an invented native result.
                    Some(Err(_)) => return,
                    None => {},
                }
            }
        }
    }
}
