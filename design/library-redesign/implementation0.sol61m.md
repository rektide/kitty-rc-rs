---
type: ImplementationReport
title: First concurrent typed kitty execution slice
description: Actual concern-crate implementation, native custody decision backed by kubeless probes, lifecycle and resource contracts, scripted tests and isolated real-kitty evidence.
resource: /design/library-redesign/implementation0.sol61m.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec7004672ffeAbyzuSmd4Cov2i"
    title: Commissioned post-G implementation, foundation corrections, concrete custody comparison, scripted and isolated live verification, and handoff
  - resource: /design/library-redesign/teardown0.glm53m.md
    title: Inspected G foundation, recovery coordinate and owned teardown evidence
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Accepted registry-first architecture and three discriminating reuse journeys
  - resource: /design/library-redesign/tickets0.sol61m.md
    title: Inspected native work map and acceptance boundaries
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/launch.py
    title: Directly inspected deferred exit-code/signal response and async behavior
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/get_text.py
    title: Directly inspected finite text payload and result
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/focus_window.py
    title: Directly inspected match payload and acknowledgment
  - resource: file:///home/rektide/src/kubeless/work/custody/src/owner.rs
    title: Reviewed custody take/return/history ownership at local revision 3526b490b7875717a10f8c6f8eae371e68789fbb
  - resource: file:///home/rektide/src/kubeless/control/retained-actions/src/host.rs
    title: Reviewed attached host eligibility, executor construction and native-basis revalidation at the same revision
verification:
  scope: First finite direct/managed slice, not full command catalog, upload conformance, reconciliation or supported-version matrix
  evidence: Workspace gates, executable native/direct-custody/host comparison, and private headless kitty 0.48.2 encrypted conformance recorded below
  limits: In-process custody only; no socket reuse; no real-kitty minimum or latest-version claim; generic framework probes are not a production kitty adapter
---

# First concurrent typed execution slice

## Outcome and ownership

The first slice is implemented and verified **with the managed registry**, not
as a single-socket client awaiting a later pooling retrofit. G's teardown was
received and inspected, not repeated or credited to this session. Its
[handoff](teardown0.glm53m.md) and predecessor `e78489cb` remain recovery evidence.

New real concerns are `access/endpoint`, `execution/exchange`, `execution/tokio`,
`control/registry`, and the `client` facade. Their dependency graph is acyclic;
the lower entrances do not require the facade. `protocol/commands::Command`
associates output, scheduling class, reply/deferred/submission-only mode, request
construction and success-data decoding. The initial catalog is `Ls`, `GetText`,
`FocusWindow`, `LaunchWait`, and explicit `RawFinite`; builders supplement ordinary
Rust construction. Launch wait returns the server's exit-code or signal string,
not a window ID or a fabricated subscription.

The protection path is typed request → plain or encrypted envelope → frame →
prepared transport bytes. `encrypt_request(&PlainRequest, ...)` cannot receive a
protected envelope; `encrypt_raw_request(Value, ...)` is deliberately named as
an untyped escape hatch. Transport never sees a command it can encrypt again.
Endpoint bindings contain immutable explicit setup. Unix paths and numeric TCP
addresses are supported; discovery, DNS and abstract Unix socket convenience are
not inferred. Password diagnostics are redacted; secure-memory erasure is not
claimed.

## Admission, scheduling and retention contract

`Registry::reserve()` creates a caller-owned occurrence with a known `Token`.
`submit(&reservation, endpoint, command, options)` is synchronous: installation
of owned prepared input and record happens under one short state turn before
eligibility. There is **no await between acceptance and handle delivery**. A
dropped returned handle remains discoverable by token or inventory. An unused
reservation creates no operation. Ordinary refusals return the original command,
including non-Clone commands. The reservation's committed bit prevents replay
after record release without an unbounded registry tombstone set. These IDs
provide no remote deduplication or exactly-once execution guarantee.

Accepted records include queued, running, completed and taken records until
explicit release. Limits separately bound global/per-endpoint accepted count,
owned framed input bytes, retained output reservations, and active/connecting
connection slots. Endpoint count and setup bytes are also bounded. Output space
is reserved before acceptance, not after a large response has already arrived.
Failure diagnostics are capped at 256 UTF-8 bytes. The frame decoder accounts
payload allocation capacity, not merely payload length; outer queue cardinality
is separately bounded. Task/socket counts and record counts bound fixed overhead.
These are native retained-resource bounds, not an allocator-wide or process RSS
hard ceiling.

Eligible selection is endpoint round-robin with **strict class partitions** for
short, observation and long-wait work. Each endpoint has independent partitions;
global partitions must fit within the global connection quota. Within a class,
accepted-token order is FIFO. No cross-class capacity borrowing is implemented.
This deliberately favors simple reserved service opportunity over maximum
utilization or a claimed weighted/aging scheduler. A connection slot is claimed
before connection establishment and exclusively belongs to one exchange.
No registry mutex spans connect, socket I/O, waiting or caller typed decoding.

Native reply bytes survive waiter loss. A small structural validation pass
determines whether a terminal reply is malformed without allocating success
data. The sole `Operation::take()` takes bytes/reservation out of custody and
then performs typed interpretation. Success-data ownership is moved rather than
cloned by `ResponseEnvelope::into_kind`. Arbitrary decoded output allocation is
caller-owned, not hidden unbounded registry retention. Repeated takes fail;
`take_native(token)` recovers evidence without a typed handle. `release(token)`
explicitly removes a completed/taken record; nothing automatically prunes it.

`Failure` distinguishes remote error, typed success-data decode failure,
malformed native reply, local/preparation errors and interrupted progress.
Remote/typed/malformed reply failures preserve native bytes after take.
Submission-only completion is not remote acknowledgment. A remote negative
answer is not universal proof of zero side effects.

## Cancellation, driving, deadlines and shutdown

The direct `Session::execute(&mut self, ...)` future owns its socket. Dropping it
drops the transport. Managed waiters/handles only detach. Explicit cancellation
withdraws queued input or interrupts running I/O. Uncertainty is armed before
the first cancelable write; after write starts, local cancellation does not prove
remote rollback, and no replay occurs.

**Every socket is retired**, including successful and submission-only exchanges.
This is an intentional first-slice delta from prospective pooling: immutable
setup is reused, but transport reuse is not claimed before a proven reuse
boundary and live sequential-reuse conformance. Cancellation cannot feed an old
reply into a new operation. Unexpected coalesced or partial surplus is rejected.

The explicit `Driver` task drives owned `FuturesUnordered` exchange futures, not
detached child tasks. Therefore driver destruction synchronously drops its
sockets before the recovery guard releases connection accounting. An earlier
prototype used a `JoinSet`; that was corrected because aborting child tasks only
schedules their destruction and could transiently exceed replacement quotas.
`stop().await` is the replacement handoff. Running unfinished work becomes a
retained unknown/pre-write interruption; queued input remains eligible for a new
driver. Taken/sent input is never reconstructed. Token plus driver generation
guards the exact completion occurrence; stale server replies cannot alter a new
record or consume its lease.

`Options::deadline` is absolute and includes the queue, connect, write and reads.
Reads do not reset it. Already-expired work is not written. Encoding, structural
validation and typed decoding are synchronous and cannot be hard-preempted by a
Tokio timer; admission rechecks the deadline after preparation, and consuming
typed decode is a caller ownership boundary, not an async cancellation point.
Encryption timestamps are fixed at acceptance; a very long queue can produce a
server timestamp refusal rather than an implicit refresh/re-encryption/replay.

`shutdown(Drain|Cancel)` first closes intake, then deliberately waits/cancels and
returns unresolved summaries. Completed malformed replies are unresolved too.
Without a driver, unfinished work returns an error while inventory stays
available; it is not silently discarded. Volatile registry/driver ownership is
not process-crash durability, nor proof that a remote child stopped.

## Kubeless decision: genuine candidate, native finite fallback

The inspected local kubeless tree was clean at
`3526b490b7875717a10f8c6f8eae371e68789fbb`. `git remote -v` listed **no remotes**.
A jj source coordinate and local checkout are not proof of a fetchable published
revision. No shipping path dependency or giant kube/HTTP facade was introduced.

The reproducible [comparison runner](probes/compare0.sol61m.py) declares its
checkout argument explicitly and builds a **scratch-only integration package**.
It uses the actual host's existing test scaffolding plus two independently
authored [host journeys](probes/host-journeys0.sol61m.rs), an independent
[direct-custody probe](probes/direct-journeys0.sol61m.rs), native Unix journeys,
and the custody kernel's actual exact-target tests. These are comparison probes,
not an undeclared portable dependency or a production kitty/framework adapter.

| Journey | Native slice | Direct custody | Retained host |
| --- | --- | --- | --- |
| Long wait/read/short independence | Actual Unix A/B/C and second-endpoint saturation; bounded native selector | Three independently taken claims and gated native futures retain short return before long/read | Three distinct keys at capacity 3 return focus first; equality-key exclusion is not endpoint/class quota policy |
| Uncertain launch then check, no replay | Actual driver loss, unknown retained, new `ls` request, old uncertainty unchanged | Spent launch input cannot be reconstructed; distinct check result leaves old run unfinished | Dropped launch driver then new read returns only `ls`; old unfinished run is retained/noted without another launch |
| New generation versus stale accounting | Fresh operation tokens/driver epochs tested; no latest-goal controller implemented | Existing `old_latest_decision_cannot_overwrite_newer_successor...` and target-qualified kernel tests pass | Existing `asynchronous_native_assessment_revalidates_basis_after_new_state_without_repeating_effect` passes |

**Select native coordination for this finite first slice.** Direct custody would
provide meaningful attached-run identity, input take and return custody. It is
not dismissed as Kubernetes-specific. However `Owner` explicitly leaves memory
unbounded until checked whole-history transfer, and returns native output via
`Arc`. Per-operation consuming take/release plus strict byte reservations still
need native policy and transfer/accounting glue. The host additionally brings
checked ask/work/preparation metadata and asynchronous native assessment that
would be artificial for these independent one-shot commands. Its current lane
supports native eligibility, not the selected endpoint/class round-robin quota
transaction out of the box; native reservations/selection remain necessary.

The fallback keeps one small record per finite operation, not a private clone of
latest obligations, offers, multi-target tickets or recovery/settlement engine.
The framework is **stronger than this slice** for actual latest goals and stale
assessment. Revisit direct custody/retained host when the separately tracked
observation/goals workflow is selected, with a fetchable reviewed revision or
an explicitly accepted workspace integration. No broad cross-repo seam change
was smuggled into this commission.

## Actual verification

- `cargo +1.85.0 check --workspace --locked` failed: locked bon/darling require
  1.88. The root floor was changed to **1.88**, which actually compiles all targets.
- `cargo +1.88.0 check --workspace --all-targets --locked` passed.
- `cargo +1.88.0 nextest run --workspace` passed: **67 tests, 67 passed**, no
  skipped tests. This includes the final explicit-address parser, builders and
  stopped-driver drain inventory case.
- `cargo +1.88.0 clippy --workspace --all-targets -- -D warnings` passed.
- `cargo fmt --all --check` passed; workspace doc-test build passed (no doctests).
- Native fixture covers three connections on one endpoint, 128 KiB fragmented
  text held after its first fragment, short completion before long/text release,
  second-endpoint listing despite a saturated first-endpoint observation lane,
  queue/slow/trickle deadlines, detached outcomes, non-Clone input/output,
  one typed decode, retained-byte/count quotas, explicit release/no replay,
  queued/running cancel, replacement, close/shutdown, malformed/coalesced replies.
- A real non-reading Unix peer forced a 5 MiB request into **partial write**:
  progress stayed `WriteStarted`, cancellation closed the socket, peer reached
  EOF having received nonzero but fewer than request bytes, and outcome remained
  unknown. Connecting-capacity/class/round-robin reservations are tested before
  any connect future is polled.
- `python3 design/library-redesign/probes/compare0.sol61m.py /home/rektide/src/kubeless`
  passed **2 direct + 13 host** probe
  tests, **18** kernel attached tests, and native journeys. The kernel workspace
  emits existing manifest-lint warnings; no upstream source was changed.
- `python3 scripts/verify-isolated-kitty.py` passed twice against installed
  **kitty 0.48.2**. It launches a private headless Weston compositor and a new
  kitty/config/child, captures only that child's published key, and tears down
  only its own processes. Actual encrypted direct listing and managed concurrent
  launch-wait (`exit 7`), text and focus completed. This provides independent
  real-server protection evidence, not merely same-module encryption/decryption.

The final gate exposed and fixed G's probabilistic ciphertext test: searching
random base85 for substring `pw` sometimes finds it by chance. The corrected
test checks absence of plaintext credential/command **fields**, not random
ciphertext coincidences.

## Checkpoints, tickets and next boundary

Owned checkpoints: `cbb82700` bounded linear framing; `c782d53a` typed execution
and initial native registry; `b99e5350` integrated journeys, synchronous driver
resource retirement, conformance and probes; `1adb8f5a` crypto test correction;
`e3c32364` explicit parser/builders; `a764e669` explicit fragmented-read cancel;
`65d60354` stopped-driver drain inventory regression.
Later documentation/tracker checkpoints attach final gate evidence separately.
No push, squash, `jj edit`, tracker migration, new issue or delegated agent.

Native `kitty-rc-proto-rs-redesign-core-reset` is closed for **G's** verified teardown.
The nine first-slice core-wire/core-typed/access-explicit/core-session and
registry-admission/kubeless/leases/custody/slice tickets are closed with native
completion evidence pointing here and at code/tests/commits. Full live matrix,
support policy, store I/O composition, examples/CLI breadth, discovery, uploads,
catalog expansion and observation/goals remain their existing tickets. One live
installed version does not close `verification-live` or select the kitty floor.

[Draft 1](draft1.gpt6a.md) supplies intent and future possibilities; this report
records the implemented smaller landing, including strict partitions, native
finite custody and deliberately non-reused sockets. [The teardown](teardown0.glm53m.md)
is predecessor evidence, not the current public API. [The ticket roadmap](tickets0.sol61m.md)
explains later scope without making it implementation authorization. The parent
owns the shared [design index](README.md); this session does not rewrite it.
