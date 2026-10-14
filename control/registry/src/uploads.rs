//! Specialty same-command lanes. Explicit aliases, exact occurrence claims and
//! retained interruption policy; no endpoint-wide lock or remote cleanup claim.
use super::*;
use kitty_rc_commands::upload::{UploadCommand, UploadKey};
use kitty_rc_exchange::{PreparedUpload, prepare_upload};
use kitty_rc_tokio::{SessionOptions, UploadSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoordinationGroupId {
    owner: u64,
    sequence: u64,
}
impl CoordinationGroupId {
    pub(super) fn for_endpoint(id: EndpointId) -> Self {
        Self {
            owner: id.owner,
            sequence: id.sequence,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneClaim {
    pub group: CoordinationGroupId,
    pub command: UploadKey,
    pub token: Token,
    generation: u64,
}
impl LaneClaim {
    pub(super) fn new(
        lane: (CoordinationGroupId, UploadKey),
        token: Token,
        generation: u64,
    ) -> Self {
        Self {
            group: lane.0,
            command: lane.1,
            token,
            generation,
        }
    }
}
/// Deliberate permission to let a NEW stream replace a possibly retained native
/// accumulator. Not cancellation, replay, cleanup or proof of remote completion.
#[derive(Debug, Clone, Copy)]
pub enum LanePolicy {
    AllowNewStreamToSupersede,
}
#[derive(Debug, thiserror::Error)]
pub enum LaneError {
    #[error("unknown endpoint or foreign group")]
    Unknown,
    #[error("endpoint already owns accepted work; aliases must be assigned before submission")]
    Busy,
    #[error("claim is stale or no longer owned")]
    Stale,
    #[error("claimed operation is still running")]
    Running,
    #[error("release retained upload claim explicitly before releasing its operation")]
    Retained,
}
#[derive(Debug, Clone, Copy)]
pub struct UploadOptions {
    pub operation: Options,
    pub max_upload: u64,
}
pub struct RefusedUpload<C> {
    pub command: C,
    pub source: UploadSource,
    pub reason: RefusalReason,
}
pub struct RejectedUpload {
    pub prepared: PreparedUpload,
    pub source: UploadSource,
    pub max_upload: u64,
}
pub(super) enum Input {
    Finite(Prepared),
    Upload(RejectedUpload),
}
impl Input {
    pub(super) fn reservation_bytes(&self) -> usize {
        match self {
            Self::Finite(p) => p.reservation_bytes(),
            Self::Upload(p) => p
                .prepared
                .reservation_bytes()
                .saturating_add(p.source.retained_bytes()),
        }
    }
    pub(super) fn key(&self) -> Option<UploadKey> {
        match self {
            Self::Finite(_) => None,
            Self::Upload(p) => Some(p.prepared.key()),
        }
    }
}
impl Registry {
    pub fn coordination_group(
        &self,
        endpoint: EndpointId,
    ) -> Result<CoordinationGroupId, LaneError> {
        self.shared
            .state
            .lock()
            .unwrap()
            .groups
            .get(&endpoint)
            .copied()
            .ok_or(LaneError::Unknown)
    }
    /// Assign a newly registered endpoint as a known alias. Immutable once that
    /// endpoint has accepted work. Setup/credential changes never alter its group.
    pub fn assign_alias(
        &self,
        endpoint: EndpointId,
        group: CoordinationGroupId,
    ) -> Result<(), LaneError> {
        let mut s = self.shared.state.lock().unwrap();
        if !s.groups.contains_key(&endpoint)
            || group.owner != self.shared.owner
            || !s.groups.values().any(|g| *g == group)
        {
            return Err(LaneError::Unknown);
        }
        if s.records.values().any(|r| r.endpoint == endpoint)
            || s.lanes
                .keys()
                .any(|(g, _)| Some(g) == s.groups.get(&endpoint))
        {
            return Err(LaneError::Busy);
        }
        s.groups.insert(endpoint, group);
        Ok(())
    }
    pub fn lane_claim(&self, group: CoordinationGroupId, command: UploadKey) -> Option<LaneClaim> {
        self.shared
            .state
            .lock()
            .unwrap()
            .lanes
            .get(&(group, command))
            .copied()
    }
    /// Bounded by retained operation slots. Inspect separately from command
    /// outcome: a known remote refusal can leave accumulator ownership unknown.
    pub fn lane_inventory(&self) -> Vec<LaneClaim> {
        self.shared
            .state
            .lock()
            .unwrap()
            .lanes
            .values()
            .copied()
            .collect()
    }
    /// Exact stale-safe release after local driving has ended. Closing the socket
    /// does not establish accumulator cleanup; caller explicitly chooses overwrite.
    pub fn resolve_lane(&self, claim: LaneClaim, _policy: LanePolicy) -> Result<(), LaneError> {
        let mut s = self.shared.state.lock().unwrap();
        let key = (claim.group, claim.command);
        if s.lanes.get(&key) != Some(&claim) {
            return Err(LaneError::Stale);
        }
        if s.records
            .get(&claim.token)
            .is_some_and(|r| r.status == Status::Running)
        {
            return Err(LaneError::Running);
        }
        s.lanes.remove(&key);
        drop(s);
        self.shared.wake.notify_one();
        Ok(())
    }
    pub fn submit_upload<C: UploadCommand>(
        &self,
        reservation: &Reservation,
        endpoint: EndpointId,
        command: C,
        source: UploadSource,
        options: UploadOptions,
    ) -> Result<Operation<C>, RefusedUpload<C>> {
        let captured = self
            .shared
            .state
            .lock()
            .unwrap()
            .endpoints
            .get(&endpoint)
            .map(|e| (e.version, e.setup.clone()));
        let prepared = (|| {
            if reservation.token.owner != self.shared.owner {
                return Err(RefusalReason::ForeignToken);
            }
            if options.operation.max_response < 256 {
                return Err(RefusalReason::ResponseReservation);
            }
            let (_, setup) = captured.as_ref().ok_or(RefusalReason::UnknownEndpoint)?;
            let setup = setup.as_ref().ok_or(RefusalReason::SetupUnavailable)?;
            prepare_upload(&command, &setup.protection, self.shared.limits.max_request)
                .map_err(RefusalReason::Prepare)
        })();
        let prepared = match prepared {
            Ok(p) => p,
            Err(reason) => {
                return Err(RefusedUpload {
                    command,
                    source,
                    reason,
                });
            }
        };
        let input = Input::Upload(RejectedUpload {
            prepared,
            source,
            max_upload: options.max_upload,
        });
        self.admit::<C>(
            reservation,
            endpoint,
            captured.unwrap().0,
            class_index(command.class()),
            input,
            options.operation,
        )
        .map_err(|(input, reason)| {
            let Input::Upload(input) = input else {
                unreachable!()
            };
            RefusedUpload {
                command,
                source: input.source,
                reason,
            }
        })
    }
    /// Recovers a setup-rejected, unpolled source with its frozen metadata. The
    /// original occurrence stays spent; resubmission is a new owner decision.
    pub fn take_rejected_upload(&self, token: Token) -> Result<RejectedUpload, CustodyError> {
        let mut s = self.shared.state.lock().unwrap();
        let r = s.records.get_mut(&token).ok_or(CustodyError::Missing)?;
        let input = r.rejected_upload.take().ok_or(CustodyError::Unavailable)?;
        r.input_bytes = 0;
        Ok(input)
    }
}
pub(super) async fn execute_input(
    session: &mut Session,
    input: Input,
    deadline: Instant,
    max_response: usize,
    evidence: Evidence,
) -> Outcome {
    match input {
        Input::Finite(p) => {
            session
                .execute_prepared(p, deadline, max_response, evidence)
                .await
        }
        Input::Upload(p) => {
            session
                .upload_prepared(
                    p.prepared,
                    p.source,
                    kitty_rc_tokio::UploadOptions {
                        session: SessionOptions {
                            deadline,
                            max_request: 0,
                            max_response,
                        },
                        max_upload: p.max_upload,
                    },
                    evidence,
                )
                .await
        }
    }
}
pub(super) fn successful_terminal(outcome: &Outcome) -> bool {
    #[derive(serde::Deserialize)]
    struct Header {
        ok: bool,
        stream: Option<bool>,
    }
    matches!(outcome,Outcome::Reply(bytes) if serde_json::from_slice::<Header>(bytes).is_ok_and(|h|h.ok && h.stream!=Some(true)))
}
pub(super) fn release_exact(
    s: &mut State,
    lane: (CoordinationGroupId, UploadKey),
    token: Token,
    generation: u64,
) {
    if s.lanes
        .get(&lane)
        .is_some_and(|claim| claim.token == token && claim.generation == generation)
    {
        s.lanes.remove(&lane);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_release_cannot_unblock_a_successor_and_claim_waiters_do_not_take_slots() {
        let r = Registry::new(Limits {
            class_connections: [2, 3, 3],
            class_connections_per_endpoint: [1, 1, 1],
            ..Limits::default()
        })
        .unwrap();
        let endpoint = || Endpoint {
            address: kitty_rc_endpoint::Address::Unix("/none".into()),
            protection: kitty_rc_endpoint::Protection::Plain,
        };
        let a = r.register(endpoint()).unwrap();
        let alias = r.register(endpoint()).unwrap();
        let g = r.coordination_group(a).unwrap();
        r.assign_alias(alias, g).unwrap();
        let command = || {
            kitty_rc_commands::upload::Run::builder()
                .cmdline(vec!["cat".into()])
                .build()
        };
        let opts = UploadOptions {
            operation: Options {
                deadline: Instant::now() + std::time::Duration::from_secs(5),
                max_response: 256,
            },
            max_upload: 10,
        };
        let first = r
            .submit_upload(
                &r.reserve(),
                a,
                command(),
                UploadSource::bytes(vec![]),
                opts,
            )
            .map_err(|e| e.reason)
            .unwrap();
        let second = r
            .submit_upload(
                &r.reserve(),
                alias,
                command(),
                UploadSource::bytes(vec![]),
                opts,
            )
            .map_err(|e| e.reason)
            .unwrap();
        let old = select(&r.shared, 1).unwrap();
        assert_eq!(old.token, first.token());
        assert!(select(&r.shared, 1).is_none());
        assert_eq!(r.pool_stats().active_or_connecting, 1);
        let claim = r.lane_claim(g, UploadKey::Run).unwrap();
        {
            let mut s = r.shared.state.lock().unwrap();
            complete(
                s.records.get_mut(&first.token()).unwrap(),
                Outcome::Interrupted {
                    progress: Progress::WriteStarted,
                    reason: ExecutionError::Eof,
                },
            );
        }
        r.resolve_lane(claim, LanePolicy::AllowNewStreamToSupersede)
            .unwrap();
        let next = select(&r.shared, 2).unwrap();
        assert_eq!(next.token, second.token());
        {
            let mut s = r.shared.state.lock().unwrap();
            release_exact(&mut s, (g, UploadKey::Run), first.token(), 1);
        }
        assert_eq!(
            r.lane_claim(g, UploadKey::Run).unwrap().token,
            second.token()
        );
        assert!(matches!(
            r.resolve_lane(claim, LanePolicy::AllowNewStreamToSupersede),
            Err(LaneError::Stale)
        ));
    }
}
