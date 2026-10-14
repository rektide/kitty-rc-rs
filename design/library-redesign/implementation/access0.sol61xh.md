---
type: ImplementationReport
title: Explicit access acquisition and generation-safe registry setup
description: Implemented native-path discovery, scoped credential sources, preserved TSV storage, explicit setup mutation and exact old-lease accounting, with race and encrypted journey evidence.
resource: /design/library-redesign/implementation/access0.sol61xh.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec6045a97ffe0ixiueO2D98CSI"
    title: Parent-led access implementation, explicit source and setup contracts, generation races, shared-store assessment and verification
  - resource: /design/library-redesign/draft1.gpt6a.md#7-discovery-credentials-and-setup-remain-separate
    title: Accepted separation of discovery, credentials, setup acquisition and managed execution
  - resource: /design/library-redesign/implementation0.sol61m.md#review-driven-corrections
    title: Inspected reusable session/pool, first-send protection, custody and retirement foundation
  - resource: /design/library-redesign/teardown0.glm53m.md#4-persisted-user-data-and-the-key-store
    title: Preserved legacy TSV shape and removed helper subprocess and automatic cleanup
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md#identity-and-generations-must-stay-independent
    title: Local binding/credential coordinates, exact leased basis and truthful identity boundaries
  - resource: /doc/PLAN-pubkey-db.md
    title: Historical publication problem and existing persisted-record format, not an authorization for its old automatic cleanup
  - resource: /apps/cli/src/main.rs
    title: Existing CLI shares parser/rendering but still duplicates filesystem I/O and drops malformed lines on explicit cleanup
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/child.py
    title: Directly inspected per-child KITTY_PID and KITTY_PUBLIC_KEY publication with route or inherited fd binding
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/utils.py
    title: Directly inspected Unix/abstract and numeric TCP address conventions
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/remote_control.py
    title: Directly inspected published public-key protocol/version parsing; no server key-query assumption
verification:
  scope: Access acquisition, shared-store library boundary and registry setup generations over the existing finite execution engine
  evidence: Workspace nextest 127/127, strict workspace clippy, locked all-target check, formatting and doc-test checks; specific journeys described below
  limits: Scripted Unix peers, not a new live-kitty supported-version matrix; CLI filesystem migration remains outside this slice; no automatic credential watcher or server-incarnation proof
---

# Explicit access and generation-safe setup

## Implemented outcome

`access/discovery` and `access/credentials` are useful independent entrances,
re-exported by the facade. They compose into actual encrypted managed commands,
not merely abstract source interfaces. Registry setup is explicitly mutable and
generation-checked. Existing typed submission, custody and default concurrent
class partitions remain intact.

Native work: `kitty-rc-proto-rs-redesign-access-resolve` and
`kitty-rc-proto-rs-redesign-access-store`. This report records implementation and
verification, not human API acceptance or completion of the later CLI pass.

## Discovery and source selection

- `CapturedEnvironment::capture(&source)` reads only selected `KITTY_LISTEN_ON`
  and `KITTY_PID` through an injectable source. Interpretation is a separate pure
  `candidates()` call. Process environment is never mutated by tests or adapters.
- `Candidate` carries `AccessBinding { address, pid_hint }` and structured
  `Origin`. Configured routes can be supplied directly; `SocketPaths` checks only
  explicitly supplied paths. Missing socket paths are absent candidates;
  non-sockets and operational errors are not silently skipped.
- Pure `select` supports `Unique`, explicit acquisition-order `First`, or
  ordered source-tier `Prefer`. A tier with multiple candidates is ambiguous.
  Equal route addresses with different provenance are not silently merged.
- `discover(Some(explicit), ...)` bypasses source acquisition entirely. It does
  not first read malformed environment or inspect unnecessary files.
- `Address::parse_native` preserves native Unix path bytes. No expansion,
  canonicalization, DNS, directory sweep, guessed PID suffix, terminal fallback,
  abstract-socket substitution or inherited-fd interpretation occurs. `fd:` and
  `unix:@...` produce explicit unsupported-route errors. Numeric TCP remains
  supported, including bracketed IPv6 and `tcp6:` input.

PID suffix patterns can be supplied as caller-declared `Convention` candidates;
they never manufacture verified identity. Alias/group coordination remains a
separate future upload concern: this slice does not infer groups from PID,
address equality, key equality or a local connection token.

The registry does not yet expose a coordination-group key. Upcoming upload
admission needs an explicitly caller-assigned, stable `CoordinationGroupId` shared
by known aliases, with a lane key `(group, command)`. Credential/setup changes
must not mint a fresh lane while an old upload lease may still be active. Any
group merge/reassignment needs an explicit freeze/claim handoff, not an implicit
equality-based merge. This is the seam for the parent/upload slice, not an
implemented exclusion guarantee in this report.

## Credentials: explicit scope, no downgrade

`Policy::Plain` is an explicit choice, not recovery from failed encryption.
`Policy::Encrypted` supplies ordered key/password sources and a `MissingPolicy`.
`TryNext` permits fallback only for missing material. Malformed key, encoding,
route/PID mismatch, store diagnostics and operational errors stop resolution.
An empty explicitly supplied password is still encrypted material; it does not
claim that encryption alone grants authorization.

Built-in sources are inline material, captured kitty publication, named captured
password variables, bounded key/password files and the legacy TSV store.
`validate_key` is pure. File/store key sources check their declared route/PID
scope **before I/O**, and the pure core checks returned material again before
cryptographic use. Injectable sources receive the selected binding.

`KITTY_PUBLIC_KEY` is associated only with its captured `KITTY_LISTEN_ON` route
and optional PID hint. A publication without a usable route is unbound; a key
for a different chosen endpoint is a binding error. A caller can deliberately
declare another association using explicit material, but the library does not
guess it or claim a PID proves process identity. Stored record provenance retains
path, PID, window ID and publication timestamp.

Filesystem adapters live in `credentials::sources`, separate from policy and
validation. Credential files are one UTF-8 line with one optional LF/CRLF;
internal newlines fail rather than silently truncate. Normal files are bounded;
directories/devices/pipes are refused. These checks are not a hostile-filesystem
sandbox or a promise to preempt every OS filesystem operation. The store adapter
uses an explicit blocking executor for the shared blocking storage boundary.
Neither crypto, transport nor built-in acquisition starts a helper process.

`Resolved` returns structured provenance and the chosen binding/protection.
`into_endpoint()` keeps the resolved route. No source is secretly reread during
`submit`: installing a snapshot deliberately selects reuse until an explicit
owner update. Per-call reacquisition is caller policy, not a hidden watcher.

## Registry setup ownership and exact generations

Public additive API:

```text
setup_version(EndpointId) -> Result<SetupVersion, SetupError>
capture_setup(EndpointId) -> Result<SetupTicket, SetupError>
register_pending(Address) -> Result<SetupTicket, RegistryError>
invalidate_credentials(EndpointId) -> Result<SetupTicket, SetupError>
rebind(EndpointId, Address) -> Result<SetupTicket, SetupError>
replace_endpoint(EndpointId, Endpoint) -> Result<SetupVersion, SetupError>
install_setup(SetupTicket, Protection) -> Result<SetupVersion, SetupError>
acquire_setup(SetupTicket, Future<Result<Protection, E>>) -> Result<SetupVersion, AcquisitionError<E>>
take_rejected_input(Token) -> Result<Prepared, CustodyError>
```

`BindingGeneration::value()` and `CredentialGeneration::value()` expose local
coordinates. Interpret them together with their `EndpointId`, never as server
incarnation, observed sample order or blanket authorization. Credential
invalidation leaves binding unchanged. Explicit `rebind` advances the binding
even at the same route; replacement advances credentials and also binding when
the address changes. Ready setup cannot be overwritten by a second completion
for the same ticket. Stale or foreign tickets never install.

Source futures are caller-owned and run outside locks, consume no connection
slot and are not retained/retried by the driver. Pending entries use the normal
bounded endpoint slots; pending submission refuses with `SetupUnavailable`.
Source failure leaves setup pending rather than restoring old credentials or
downgrading. Dropping acquisition cannot later install its result.

The concrete execution ownership cut is **lease selection**:

1. Submission captures setup, freezes plaintext outside the lock, then rechecks
   the setup version before accepting. A racing change returns the original
   command with `SetupChanged`; its reservation remains uncommitted.
2. Explicit update cancels still-queued old preparation as
   `NotWritten / Cancelled(Queue)`. `Summary::setup_rejection` records accepted
   and invalidating versions. The immutable plaintext remains recoverable with
   `take_rejected_input`, counted against input quotas until taken or released.
   No same-token replay or automatic re-preparation occurs.
3. Already leased work—including a connection being established—owns its exact
   old endpoint/setup snapshot and may finish. Updating setup is not a claim that
   every unsent leased exchange or remote permission has been revoked. Explicit
   token cancellation remains available, with its ordinary progress/uncertainty
   contract. Historical receipts and unknown outcomes retain their old basis.
4. Idle stale sessions retire on update. Old leased completions keep their native
   outcome, but cannot put a connection into a newer setup pool. Token, driver
   epoch and setup version all qualify completion.

Retirement reserves its connection slots until socket destruction completes
outside the owner lock. Thus another thread cannot see prematurely freed capacity
and connect before the stale transport closes. `PoolStats::retiring` exposes
these temporary reservations. Class isolation defaults are unchanged.

## Existing TSV store: preserved, not rewritten

The parser/rendering boundary already existed and is already used by
`kitty-pubkey-db`. This slice extends it with `storage::{read, append}` rather
than creating a competing format. Existing three-field records still read with
timestamp zero, four-field records retain their metadata, and malformed lines
retain line-numbered diagnostics. Credential lookup chooses the newest selected
PID record and refuses malformed stores; it never drops bad lines to make a key
lookup appear clean.

Append preserves all prior bytes, including malformed records and a last record
without a trailing newline. It rejects embedded TSV separators before writing,
never removes earlier publications, never creates a parent directory implicitly
and never tests liveness or prunes. Multi-process writers need deliberate caller
coordination; this is not a transactional replacement database.

**Remaining store-ticket gap:** CLI `init/add/get/cleanup` filesystem code is
still local to `apps/cli`, outside this session's source ownership. In particular,
existing explicit cleanup reports malformed lines and then drops them during its
rewrite. The parent CLI pass should adopt the shared normal read/append boundary
and refuse or deliberately preserve malformed cleanup input. The store ticket
must remain open until that exact acceptance is verified.

## Verification and checkpoints

With user-selected mise `nightly-2026-10-14`, `rustc 1.101.0-nightly
(822182db4 2026-10-13)`:

- Targeted predecessor access/registry nextest: **43 passed**.
- Final shared-tree `cargo nextest run --workspace --locked`: **127 passed**,
  no skipped tests. This total includes concurrent catalog/observation work;
  those implementations are not attributed to this session.
- Workspace strict all-target clippy, locked all-target check, formatting check
  and doc-test run passed. The latter included one concurrent catalog doctest.
- Tests cover source order and explicit bypass, native non-Unicode paths,
  scoped environment/store/key-file mismatches before reads, missing versus
  malformed versus operational errors, regular-file bounds, untouched legacy
  records, delayed rebinding and credential-only changes, one-shot simultaneous
  installations, admission revalidation and retained rejected-input quotas.
- Actual encrypted Unix journeys resolve a captured route plus legacy-store
  fallback and named captured password, preserve store bytes, run a listing,
  then retain a different command's native denial. Another holds an old encrypted
  launch while new credentials serve independent listing/focus; the old completion
  never replaces the newer idle token. Rebinding retires old idle sockets without
  changing history; driver loss after setup change preserves original uncertainty
  and does not replay a queued rejected launch.

Owned checkpoints: `1b7486af` shared TSV I/O; `cf377115` initial generation/source
stage; `d4c9ce58` final modular source/race/journey landing on the current line.
The initial generation commit raced a concurrent observation commit and remains
as a divergent historical checkpoint. The final landing selects only owned
access/registry/facade and workspace/lock hunks, leaving independent source work
untouched. No push, `jj edit`, squash, tracker migration or subagent spawn.

## Cross-references and next ownership

- [Draft 1](../draft1.gpt6a.md#7-discovery-credentials-and-setup-remain-separate)
  owns architectural intent; this report supplies actual smaller API/evidence.
- [Foundation corrections](../implementation0.sol61m.md#review-driven-corrections)
  own reusable transport, send-time protection and bounded custody. This slice
  qualifies their immutable-setup description without rewriting that history.
- [Generation distinctions](../kubeless-adaptation1.gpt6a.md#identity-and-generations-must-stay-independent)
  remain the precedent for local coordinates and old-result accounting.
- [Credential adapters](../../../access/credentials/src/sources.rs) and
  [setup transitions](../../../control/registry/src/setup.rs) are the executable
  ownership boundaries. [Journeys](../../../control/registry/tests/journeys.rs)
  connect them to useful encrypted operations.
- The parent owns the shared design index, root README, toolchain policy, CLI
  adoption and any stronger future first-write/revocation policy. This slice
  chooses neither server identity reconciliation nor automatic cleanup.
