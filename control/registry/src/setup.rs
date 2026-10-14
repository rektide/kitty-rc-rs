//! Explicit setup ownership. Acquisition futures are caller-owned, outside the
//! registry lock. An install ticket names one exact pending binding generation.
use crate::{EndpointId, Limits, Registry, RegistryError, Status, complete};
use kitty_rc_endpoint::{Address, Endpoint, Protection};
use kitty_rc_exchange::{ExecutionError, Outcome, Phase, Progress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingGeneration(u64);
impl BindingGeneration {
    pub fn value(self) -> u64 {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CredentialGeneration(u64);
impl CredentialGeneration {
    pub fn value(self) -> u64 {
        self.0
    }
}
/// Local registry basis, not server incarnation, authorization or observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupVersion {
    pub binding: BindingGeneration,
    pub credentials: CredentialGeneration,
}
impl Default for SetupVersion {
    fn default() -> Self {
        Self {
            binding: BindingGeneration(1),
            credentials: CredentialGeneration(1),
        }
    }
}
#[derive(Debug, Clone)]
pub struct SetupTicket {
    endpoint: EndpointId,
    address: Address,
    version: SetupVersion,
}
impl SetupTicket {
    pub fn endpoint(&self) -> EndpointId {
        self.endpoint
    }
    pub fn address(&self) -> &Address {
        &self.address
    }
    pub fn version(&self) -> SetupVersion {
        self.version
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupRejection {
    pub accepted: SetupVersion,
    pub invalidated_by: SetupVersion,
}
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("unknown or foreign endpoint")]
    UnknownEndpoint,
    #[error("stale setup acquisition: captured {captured:?}, current {current:?}")]
    Stale {
        captured: SetupVersion,
        current: SetupVersion,
    },
    #[error("setup acquisition for this generation already installed")]
    AlreadyInstalled,
    #[error("local setup generation exhausted")]
    GenerationExhausted,
    #[error(transparent)]
    Registry(#[from] RegistryError),
}
#[derive(Debug, thiserror::Error)]
pub enum AcquisitionError<E: std::error::Error + 'static> {
    #[error("credential acquisition failed: {0}")]
    Source(#[source] E),
    #[error(transparent)]
    Install(#[from] SetupError),
}

pub(super) struct Entry {
    pub address: Address,
    pub version: SetupVersion,
    pub setup: Option<Endpoint>,
}
impl Entry {
    pub fn ready(endpoint: Endpoint) -> Self {
        Self {
            address: endpoint.address.clone(),
            version: SetupVersion::default(),
            setup: Some(endpoint),
        }
    }
    fn ticket(&self, endpoint: EndpointId) -> SetupTicket {
        SetupTicket {
            endpoint,
            address: self.address.clone(),
            version: self.version,
        }
    }
}

pub(super) fn normalize(
    mut endpoint: Endpoint,
    limits: &Limits,
) -> Result<Endpoint, RegistryError> {
    let address_bytes = match &endpoint.address {
        Address::Unix(path) => path.as_os_str().as_encoded_bytes().len(),
        Address::Tcp(_) => 32,
    };
    let access_bytes = match &endpoint.protection {
        Protection::Plain => 0,
        Protection::Encrypted {
            password,
            key_origin,
            ..
        } => password
            .len()
            .saturating_add(key_origin.len())
            .saturating_add(32),
    };
    if address_bytes.saturating_add(access_bytes) > limits.max_request {
        return Err(RegistryError::EndpointSetupLimit);
    }
    if let Address::Unix(path) = &mut endpoint.address {
        path.shrink_to_fit();
    }
    if let Protection::Encrypted {
        password,
        key_origin,
        ..
    } = &mut endpoint.protection
    {
        password.shrink_to_fit();
        key_origin.shrink_to_fit();
    }
    Ok(endpoint)
}

impl Registry {
    pub fn setup_version(&self, endpoint: EndpointId) -> Result<SetupVersion, SetupError> {
        self.shared
            .state
            .lock()
            .unwrap()
            .endpoints
            .get(&endpoint)
            .map(|entry| entry.version)
            .ok_or(SetupError::UnknownEndpoint)
    }
    /// Capture a route and exact setup basis. A ready generation cannot be
    /// overwritten by installation; invalidate deliberately to acquire anew.
    pub fn capture_setup(&self, endpoint: EndpointId) -> Result<SetupTicket, SetupError> {
        self.shared
            .state
            .lock()
            .unwrap()
            .endpoints
            .get(&endpoint)
            .map(|entry| entry.ticket(endpoint))
            .ok_or(SetupError::UnknownEndpoint)
    }
    /// Register a selected route without credentials. Submission refuses while
    /// pending. This consumes the same bounded endpoint slot as ready setup.
    pub fn register_pending(&self, address: Address) -> Result<SetupTicket, RegistryError> {
        let normalized = normalize(
            Endpoint {
                address,
                protection: Protection::Plain,
            },
            &self.shared.limits,
        )?;
        let mut s = self.shared.state.lock().unwrap();
        if s.closed {
            return Err(RegistryError::Closed);
        }
        if s.endpoints.len() >= self.shared.limits.endpoints {
            return Err(RegistryError::EndpointCapacity);
        }
        let token = self.mint();
        let id = EndpointId {
            owner: token.owner,
            sequence: token.sequence,
        };
        let entry = Entry {
            address: normalized.address,
            version: SetupVersion::default(),
            setup: None,
        };
        let ticket = entry.ticket(id);
        s.endpoints.insert(id, entry);
        Ok(ticket)
    }
    /// Retire idle old sockets and reject queued old preparation. Already
    /// leased operations keep their exact setup; their completion cannot enter
    /// the new pool. Historical outcomes and remote uncertainty stay unchanged.
    pub fn invalidate_credentials(&self, endpoint: EndpointId) -> Result<SetupTicket, SetupError> {
        self.change_setup(endpoint, None, None)
    }
    /// Explicit association rebinding (even at the same route), not alias
    /// detection or automatic reconciliation.
    /// Binding and credential generations remain separate local coordinates.
    pub fn rebind(
        &self,
        endpoint: EndpointId,
        address: Address,
    ) -> Result<SetupTicket, SetupError> {
        let address = normalize(
            Endpoint {
                address,
                protection: Protection::Plain,
            },
            &self.shared.limits,
        )?
        .address;
        self.change_setup(endpoint, Some(address), None)
    }
    /// Atomic replacement with caller-acquired setup. Address changes advance
    /// binding; every explicit replacement advances credentials, even when equal.
    pub fn replace_endpoint(
        &self,
        endpoint: EndpointId,
        setup: Endpoint,
    ) -> Result<SetupVersion, SetupError> {
        let setup = normalize(setup, &self.shared.limits)?;
        Ok(self.change_setup(endpoint, None, Some(setup))?.version)
    }
    fn change_setup(
        &self,
        endpoint: EndpointId,
        route: Option<Address>,
        ready: Option<Endpoint>,
    ) -> Result<SetupTicket, SetupError> {
        let mut s = self.shared.state.lock().unwrap();
        if s.closed {
            return Err(RegistryError::Closed.into());
        }
        let entry = s
            .endpoints
            .get_mut(&endpoint)
            .ok_or(SetupError::UnknownEndpoint)?;
        let rebinding = route.is_some();
        let address = route.unwrap_or_else(|| {
            ready
                .as_ref()
                .map(|setup| setup.address.clone())
                .unwrap_or_else(|| entry.address.clone())
        });
        let binding = if rebinding || address != entry.address {
            BindingGeneration(
                entry
                    .version
                    .binding
                    .0
                    .checked_add(1)
                    .ok_or(SetupError::GenerationExhausted)?,
            )
        } else {
            entry.version.binding
        };
        let credentials = CredentialGeneration(
            entry
                .version
                .credentials
                .0
                .checked_add(1)
                .ok_or(SetupError::GenerationExhausted)?,
        );
        entry.version = SetupVersion {
            binding,
            credentials,
        };
        entry.address = address;
        entry.setup = ready;
        let ticket = entry.ticket(endpoint);
        for record in s
            .records
            .values_mut()
            .filter(|r| r.endpoint == endpoint && r.status == Status::Queued)
        {
            let frozen = record.input.take();
            let bytes = record.input_bytes;
            record.setup_rejection = Some(SetupRejection {
                accepted: record.setup_version,
                invalidated_by: ticket.version,
            });
            // This explicit update policy cancels the queued occurrence. It does
            // not fabricate a remote error or discard the original plaintext.
            complete(
                record,
                Outcome::Interrupted {
                    progress: Progress::NotWritten,
                    reason: ExecutionError::Cancelled {
                        phase: Phase::Queue,
                    },
                },
            );
            record.rejected_input = frozen;
            record.input_bytes = bytes;
        }
        let retired = s.idle.remove(&endpoint);
        let count = retired.as_ref().map_or(0, Vec::len);
        if count != 0 {
            *s.retiring.entry(endpoint).or_default() += count;
        }
        drop(s);
        drop(retired); // Socket destruction never holds the owner lock.
        crate::finish_retirement(&self.shared, endpoint, count);
        self.shared.wake.notify_one();
        self.shared.lifecycle.notify_waiters();
        Ok(ticket)
    }
    /// One-shot generation-checked installation. No acquisition runs under the
    /// owner lock and a second completion for the same ticket cannot overwrite.
    pub fn install_setup(
        &self,
        ticket: SetupTicket,
        protection: Protection,
    ) -> Result<SetupVersion, SetupError> {
        let setup = normalize(
            Endpoint {
                address: ticket.address.clone(),
                protection,
            },
            &self.shared.limits,
        )?;
        let mut s = self.shared.state.lock().unwrap();
        if s.closed {
            return Err(RegistryError::Closed.into());
        }
        let entry = s
            .endpoints
            .get_mut(&ticket.endpoint)
            .ok_or(SetupError::UnknownEndpoint)?;
        if entry.version != ticket.version {
            return Err(SetupError::Stale {
                captured: ticket.version,
                current: entry.version,
            });
        }
        if entry.setup.is_some() {
            return Err(SetupError::AlreadyInstalled);
        }
        entry.setup = Some(setup);
        drop(s);
        self.shared.wake.notify_one();
        Ok(ticket.version)
    }
    /// Convenience for injected/caller-owned source acquisition. Dropping this
    /// future leaves the pending route unchanged; there is no resident retry.
    pub async fn acquire_setup<F, E>(
        &self,
        ticket: SetupTicket,
        acquire: F,
    ) -> Result<SetupVersion, AcquisitionError<E>>
    where
        F: std::future::Future<Output = Result<Protection, E>>,
        E: std::error::Error + 'static,
    {
        let protection = acquire.await.map_err(AcquisitionError::Source)?;
        Ok(self.install_setup(ticket, protection)?)
    }
    /// Take the immutable plaintext of a setup-rejected queued operation. The
    /// exact occurrence remains spent; resubmission needs a new reservation.
    /// Retained input bytes count against quotas until this take or release.
    pub fn take_rejected_input(
        &self,
        token: crate::Token,
    ) -> Result<kitty_rc_exchange::Prepared, crate::CustodyError> {
        let mut s = self.shared.state.lock().unwrap();
        let record = s
            .records
            .get_mut(&token)
            .ok_or(crate::CustodyError::Missing)?;
        let prepared = record
            .rejected_input
            .take()
            .ok_or(crate::CustodyError::Unavailable)?;
        record.input_bytes = 0;
        Ok(prepared)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, RefusalReason};
    use kitty_rc_commands::{Class, Command, Ls};
    use kitty_rc_schema::PlainRequest;
    use tokio::time::{Duration, Instant};
    fn endpoint() -> Endpoint {
        Endpoint {
            address: Address::Unix("/explicit/no-connection".into()),
            protection: Protection::Plain,
        }
    }
    fn options() -> Options {
        Options {
            deadline: Instant::now() + Duration::from_secs(30),
            max_response: 256,
        }
    }

    #[tokio::test]
    async fn slow_source_does_not_hold_lock_and_stale_install_cannot_replace() {
        for change_binding in [false, true] {
            let registry = Registry::new(Limits::default()).unwrap();
            let ticket = registry.register_pending(endpoint().address).unwrap();
            let id = ticket.endpoint();
            let (started, begin) = tokio::sync::oneshot::channel();
            let (release, wait) = tokio::sync::oneshot::channel();
            let acquiring = registry.clone();
            let task = tokio::spawn(async move {
                acquiring
                    .acquire_setup(ticket, async {
                        started.send(()).unwrap();
                        wait.await.unwrap();
                        Ok::<_, std::io::Error>(Protection::Plain)
                    })
                    .await
            });
            begin.await.unwrap();
            // These synchronous owner turns would deadlock if source acquisition
            // held the owner lock. No connection slot is consumed by the source.
            let other = registry.register(endpoint()).unwrap();
            registry
                .submit(&registry.reserve(), other, Ls::default(), options())
                .unwrap();
            assert_eq!(registry.pool_stats().active_or_connecting, 0);
            let newer = if change_binding {
                registry
                    .rebind(id, Address::Unix("/new/route".into()))
                    .unwrap()
            } else {
                registry.invalidate_credentials(id).unwrap()
            };
            registry
                .install_setup(newer.clone(), Protection::Plain)
                .unwrap();
            release.send(()).unwrap();
            assert!(matches!(
                task.await.unwrap(),
                Err(AcquisitionError::Install(SetupError::Stale { .. }))
            ));
            assert_eq!(
                registry.capture_setup(id).unwrap().address(),
                newer.address()
            );
            assert_eq!(registry.setup_version(id).unwrap(), newer.version());
        }
    }

    #[test]
    fn racing_installations_for_one_ticket_have_exactly_one_winner() {
        let registry = Registry::new(Limits::default()).unwrap();
        let ticket = registry.register_pending(endpoint().address).unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let results = std::thread::scope(|scope| {
            let tasks: Vec<_> = (0..2)
                .map(|_| {
                    let (registry, ticket, barrier) =
                        (registry.clone(), ticket.clone(), barrier.clone());
                    scope.spawn(move || {
                        barrier.wait();
                        registry.install_setup(ticket, Protection::Plain)
                    })
                })
                .collect();
            tasks
                .into_iter()
                .map(|task| task.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|r| matches!(r, Err(SetupError::AlreadyInstalled)))
                .count(),
            1
        );
    }

    #[test]
    fn installation_is_one_shot_foreign_safe_and_generations_are_local() {
        let registry = Registry::new(Limits::default()).unwrap();
        let initial = registry.register_pending(endpoint().address).unwrap();
        let id = initial.endpoint();
        assert!(matches!(
            registry
                .submit(&registry.reserve(), id, Ls::default(), options())
                .err()
                .unwrap()
                .reason,
            RefusalReason::SetupUnavailable
        ));
        let foreign = Registry::new(Limits::default()).unwrap();
        assert!(matches!(
            foreign.install_setup(initial.clone(), Protection::Plain),
            Err(SetupError::UnknownEndpoint)
        ));
        registry
            .install_setup(initial.clone(), Protection::Plain)
            .unwrap();
        assert!(matches!(
            registry.install_setup(initial.clone(), Protection::Plain),
            Err(SetupError::AlreadyInstalled)
        ));
        let credentials = registry.invalidate_credentials(id).unwrap();
        assert_eq!(credentials.version().binding, initial.version().binding);
        assert!(credentials.version().credentials > initial.version().credentials);
        let rebound = registry.rebind(id, initial.address().clone()).unwrap();
        assert!(rebound.version().binding > initial.version().binding);
        assert!(matches!(
            registry.install_setup(credentials, Protection::Plain),
            Err(SetupError::Stale { .. })
        ));
        registry.install_setup(rebound, Protection::Plain).unwrap();
    }

    #[test]
    fn queued_rejection_preserves_frozen_input_and_capacity_until_taken() {
        let command = Ls::default();
        let reservation_bytes = kitty_rc_exchange::prepare(&command, &Protection::Plain, 4096)
            .unwrap()
            .reservation_bytes();
        let registry = Registry::new(Limits {
            input_bytes: reservation_bytes,
            input_bytes_per_endpoint: reservation_bytes,
            ..Limits::default()
        })
        .unwrap();
        let id = registry.register(endpoint()).unwrap();
        let reservation = registry.reserve();
        let old = registry
            .submit(&reservation, id, command, options())
            .unwrap();
        let basis = registry.setup_version(id).unwrap();
        let ticket = registry.invalidate_credentials(id).unwrap();
        registry
            .install_setup(ticket.clone(), Protection::Plain)
            .unwrap();
        let summary = registry.summary(old.token()).unwrap();
        assert_eq!(summary.status, Status::Completed);
        assert_eq!(summary.setup, basis);
        assert_eq!(
            summary.setup_rejection,
            Some(SetupRejection {
                accepted: basis,
                invalidated_by: ticket.version()
            })
        );
        assert!(!summary.unresolved);
        let failed = registry
            .submit(&registry.reserve(), id, Ls::default(), options())
            .err()
            .unwrap();
        assert!(matches!(failed.reason, RefusalReason::InputCapacity));
        assert!(matches!(
            registry.take_native(old.token()).unwrap(),
            Outcome::Interrupted {
                progress: Progress::NotWritten,
                reason: ExecutionError::Cancelled {
                    phase: Phase::Queue
                }
            }
        ));
        let input = registry.take_rejected_input(old.token()).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(input.plaintext()).unwrap()["cmd"],
            "ls"
        );
        assert!(matches!(
            registry.take_rejected_input(old.token()),
            Err(crate::CustodyError::Unavailable)
        ));
        let replacement = registry
            .submit(&registry.reserve(), id, failed.command, options())
            .unwrap();
        registry.release(old.token()).unwrap();
        replacement.cancel();
        registry.release(replacement.token()).unwrap();
        assert!(matches!(
            registry
                .submit(&reservation, id, Ls::default(), options())
                .err()
                .unwrap()
                .reason,
            RefusalReason::Committed
        ));
    }

    #[test]
    fn preparation_revalidates_basis_before_acceptance_and_preserves_command() {
        struct ChangesSetup {
            registry: Registry,
            id: EndpointId,
            value: Box<u8>,
        }
        impl Command for ChangesSetup {
            type Output = ();
            fn request(&self) -> PlainRequest {
                self.registry.replace_endpoint(self.id, endpoint()).unwrap();
                PlainRequest::new("custom")
            }
            fn class(&self) -> Class {
                Class::Short
            }
            fn decode(_: Option<serde_json::Value>) -> Result<(), kitty_rc_commands::DecodeError> {
                Ok(())
            }
        }
        let registry = Registry::new(Limits::default()).unwrap();
        let id = registry.register(endpoint()).unwrap();
        let reservation = registry.reserve();
        let refused = registry
            .submit(
                &reservation,
                id,
                ChangesSetup {
                    registry: registry.clone(),
                    id,
                    value: Box::new(7),
                },
                options(),
            )
            .err()
            .unwrap();
        assert!(matches!(refused.reason, RefusalReason::SetupChanged));
        assert_eq!(*refused.command.value, 7);
        assert!(registry.inventory().is_empty());
        registry
            .submit(&reservation, id, Ls::default(), options())
            .unwrap();
    }

    #[test]
    fn retirement_keeps_connection_capacity_reserved_until_socket_destruction() {
        let registry = Registry::new(Limits {
            connections: 1,
            connections_per_endpoint: 1,
            class_connections: [0, 1, 0],
            class_connections_per_endpoint: [0, 1, 0],
            ..Limits::default()
        })
        .unwrap();
        let id = registry.register(endpoint()).unwrap();
        registry
            .submit(&registry.reserve(), id, Ls::default(), options())
            .unwrap();
        // Model the short turn after idle removal and before outside-lock close.
        registry.shared.state.lock().unwrap().retiring.insert(id, 1);
        assert_eq!(registry.pool_stats().retiring, 1);
        assert!(crate::select(&registry.shared, 1).is_none());
        crate::finish_retirement(&registry.shared, id, 1);
        assert!(crate::select(&registry.shared, 1).is_some());
    }
}
