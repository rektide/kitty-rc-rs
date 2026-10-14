---
type: ImplementationReport
title: Finite check-only continuation with separate native uncertainty
description: Native listing composition, bounded matching, actual accepted setup evidence and lost-reply/late-effect journeys without remutation.
resource: /design/library-redesign/observation/check0.sol61xh.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec5f1ef25ffexa7YLs09vVLxPL"
    title: Check-only implementation, native effect and late-effect fixtures, strict basis/freshness assessment and scoped gate verification
  - resource: /design/library-redesign/draft1.gpt6a.md#10-observation-and-optional-reconciliation
    title: Read-only continuation and independent native outcome/satisfaction contract
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md#strongest-concrete-precedent-unknown-result--check-only-continuation
    title: Previously reported generic native continuation precedent, inspected again at the relevant API boundary
  - resource: /observation/listing/README.md
    title: Finite listing receipt, complete selected-response coverage, ordering and explicit take/release foundation
  - resource: /observation/listing/src/lib.rs
    title: Actual acquisition and snapshot APIs composed without a second engine
  - resource: /design/library-redesign/observation/implementation0.sol61m.md
    title: Completed snapshot scope and verified limitations
  - resource: /control/registry/src/lib.rs
    title: Actual original summary/native take, finite quotas and detachable custody contracts
  - resource: /control/registry/src/setup.rs
    title: Actual accepted setup and independent local generation APIs
  - resource: /design/library-redesign/implementation/access0.sol61xh.md
    title: Completed access evidence and setup ownership cut, not a server incarnation guarantee
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Available finite Launch/Ls contracts and honest buffered-run acceptance gap
  - resource: file:///home/rektide/src/kubeless/control/checks/src/lib.rs
    title: Inspected generic shared-basis assessment, outcome retention and bounded versus unbounded wait entrances
  - resource: file:///home/rektide/src/kubeless/control/checks/README.md
    title: Native adapter responsibilities and absence of scheduling/custody ownership
  - resource: file:///home/rektide/src/kubeless/control/checks/Cargo.toml
    title: Inspected optional checks dependency composition; publication not verified
  - resource: file:///home/rektide/src/kubeless/control/pass/src/lib.rs
    title: Inspected generic pass ownership/cancellation boundary
  - resource: file:///home/rektide/src/kubeless/probes/control/retained-actions/src/continuation.rs
    title: Actual generic native Call::Read with no write body and independent native unknown result
extensions:
  ticket: kitty-rc-proto-rs-redesign-observation-check
---

# Finite check-only continuation

## Outcome and public boundary

Implemented independent workspace leaf `observation/check` (`kitty-rc-check`),
over the existing `kitty-rc-listing` and registry. The [maintained leaf
README](../../../observation/check/README.md) owns public semantics;
[journeys](../../../observation/check/tests/journeys.rs) exercise native transport,
effectful fake-server state and the actual registry, not a second custody mock.
No source change was needed in listing, access, registry, protocol, facade, root
README or the shared design index. Only the new workspace member and lock package
were added to shared manifests; sibling hunks were isolated.

| API | Responsibility |
|---|---|
| `RetainedOriginal::capture` | Reference a completed retained unresolved native occurrence; capture original `Summary.setup` and a local evidence cutoff, not its write body/bytes |
| `RetainedOriginal::check_input` | Construct bounded read/check-only scope, criteria, condition and freshness metadata |
| `CheckInput::start` | Admit exactly one normal listing operation; no task/controller/loop/replay starts |
| `CheckInput::assess` | Assess an existing snapshot's declared basis/scope/coverage/local freshness, with explicit `NativeSetup::NotChecked` |
| `CheckReceipt::finish`, `Check::finish` | Sole ownership transfer of fresh listing evidence; managed accepted/current setup revalidation; release only the taken read |
| `RetainedOriginal::take` | Transfer original native `Outcome` and unchanged summary/boundary separately, then release its record |

The original native outcome is never replaced by the check result. `Matching`
has absent, unique, ambiguous and evidence-insufficient branches, with a bounded
candidate prefix, exact counts and truncation flag. `Satisfaction` is an
independent projection of caller-selected present/unique/absent conditions.
`Attribution::NotEstablished` stays explicit even for observed satisfaction.
Missing required fields, unsupported focus, partial duplicate-ID membership,
expired/stale samples and basis changes never establish absence or uniqueness.

Read receipt/input clones retain bounded metadata, not copies of native bytes or
mutation input. A report retains its exact check input and moved read snapshot.
Original custody still consumes registry quota; drop is detach, not cleanup.
Recovery after waiter loss uses saved receipt/token and the existing registry.
Explicit read cancel does not alter the original. No hidden retry input, timer,
permanent controller, automatic reconciliation/prune or synthetic cursor exists.

## Basis and freshness decisions

Original summary captures actual accepted `SetupVersion`, not an arbitrary new
caller generation. Managed start and finish compare original accepted setup,
the listing's accepted setup and current registry setup. The public access APIs
completed during this slice: `setup_version` and `Summary.setup` supplied exactly
the required evidence without registry edits or duplicate owner state. The
[access report](../implementation/access0.sol61xh.md) explains queued rejection,
old leased work and separate credential/binding coordinates. Those coordinates
are endpoint-qualified local history, **not server incarnation or authorization**.

Caller subject/binding generation and interpretation evidence are also required
to match exactly, as is the full listing selection scope. Freshness requires a
strictly later acquisition start than the local unresolved-result capture and a
caller-selected conservative maximum age measured from start. Delayed arrival
does not refresh an old acquisition. Local revalidation is not a fence against
subsequent changes, a server-atomic sample interval or proof of remote settlement.

A dedicated late-effect fixture lets the server accept launch, close before reply,
then delay its actual effect until **after** a fresh empty listing. The initial
absence can satisfy an explicitly scoped absent condition; the old launch still
appears later, on the same single launch occurrence. Both assessments leave causal
attribution and original uncertainty unchanged. This guards the important gap
between a local fresh read and a remote effect-completion barrier.

## Generic control reuse, narrowly assessed

The [reported kubeless continuation](../kubeless-adaptation1.gpt6a.md#strongest-concrete-precedent-unknown-result--check-only-continuation)
is generic control/reconciliation over a native mapping backend, not Kubernetes-
or HTTP-specific. Direct inspection found `Call::Read { reader, slot }` separated
from `Call::Write`, retained unknown output, and native checked cleanup with
stronger guarantees that must not be credited to kitty.

The actual `control/checks` API offers `AskSet`, `assess_acquired`, `finish_once`
and retained shared-basis assessment. It deliberately delegates matching,
scope, freshness, closure and judgment to native adapters; it is not a query,
timer or surviving custody engine. `finish_once` retains the native outcome on
ordinary errors but cancellation still drops in-memory owned inputs unless
surviving custody is selected separately. `wait` retains every acquired basis,
which is not a bounded-history policy for an unbounded stream.

For this **one selected finite read**, existing registry/listing custody already
supplies surviving acceptance and bounded native retention. Adding generic full-
ask traversal/pass machinery would not replace the domain matching or basis
checks and would add composition around an already finite report. Chosen result:
small native composition, no kubeless dependency, no generic-custody duplicate,
no giant facade, and no path-only unpublished dependency. This is a justified
journey-specific choice, not rejection of generic control for future multi-ask
or explicitly selected observation-until policies. Publication of the inspected
local kubeless leaves was not verified and is not assumed.

## Verification and checkpoint standing

Initial implementation checkpoints: `f1a2734c` (owned leaf/source and ten native
journeys), `91534ca5` (only root member and lock package, full-ID `jj-hunk` dry-run
then commit). The follow-up correction retains the complete bounded check input
in reports, enforces check bounds on independently supplied larger snapshots,
revalidates native setup after decode, adds explicit cancel/filtered-fact cases
and compile-fail API guards. It is preserved separately, not squashed into the
predecessor. All commands use the user's existing mise Rust policy; no new
`rust-version`, pinned toolchain or rust-policy edit was made.

Final scoped gates are recorded after the last owned-source changes:

| Gate | Result |
|---|---|
| `mise exec -- cargo nextest run -p kitty-rc-check --locked` | 12 passed, no skips |
| `mise exec -- cargo check -p kitty-rc-check --all-targets --locked` | Passed |
| `mise exec -- cargo clippy -p kitty-rc-check --all-targets --no-deps --locked -- -D warnings` | Passed; owned all targets, not dependency-wide lint |
| `mise exec -- cargo test -p kitty-rc-check --doc --locked` | 1 positive and 2 compile-fail examples passed |
| `mise exec -- rustfmt --check --edition 2024 observation/check/src/lib.rs observation/check/tests/journeys.rs` | Passed |

Toolchain readout: `rustc 1.101.0-nightly (822182db4 2026-10-13)`, through the
existing mise entrance. Owned internal dependencies include explicit native
`0.1.0` version requirements alongside development paths; no publication of
unrelated local generic-control crates is assumed.
The doc-pass verified ten local links and their heading anchors. Shared
index/root backlinks remain with their explicitly assigned parent owner.

Coverage:

1. A real fixture launch effect followed by lost terminal reply; failed listing,
   unique candidate, then ambiguous matches; exactly **one launch** on wire;
   retained original remains unresolved and transfers unchanged EOF/progress.
2. Fresh absence before a remote late effect, then later presence, no relaunch and
   no causal/acknowledgment conversion.
3. Waiter/handle loss, detached read recovery, sole take, original drop/inventory
   recovery and refusal when two retained records fill the selected quota.
4. Pre-effect acquired read arriving after the real effect; invalid freshness even
   though its returned candidate matches; an explicitly new check succeeds.
5. Exact caller binding/scope/interpretation and conservative acquisition-age
   requirements; delayed or mismatched bases remain unknown.
6. Native setup replacement after read completion rejects the assessment and old
   input's next admission, preserving original accepted uncertainty.
7. Partial membership, missing facts, known mismatch overriding unrelated unknown
   facts, unsupported focus and bounds on independently acquired snapshots.
8. Missing required typed data, projection overflow and driver stop are read
   failures, not empty state or retries; original native evidence survives.
9. Premature finish returns a recoverable receipt; explicit registry read cancel
   leaves original custody untouched.
10. Metadata byte/cardinality/freshness limits and refusal to capture known outcomes.
11. Actual filtered listing wire fields, conjunctive PID/cwd/user-variable/false
    active/focus facts, retained report requirements and scope-local absence.
12. Explicit check-handle cancellation interrupts only the fresh read.

The compile-fail doctests forbid submitting a `CheckInput` as an arbitrary
`Command` and supplying a mutation body to its admission entrance. A positive
example compiles the intended finite public journey. Full live-kitty/version
conformance and integration gates remain the parent/verification ticket's scope.

## Native dependency and next ownership

Snapshot dependency is closed. Actual `Ls` and finite `process::Launch` commands
are available and used in meaningful wire journeys, and access setup APIs are
complete. The broad `kitty-rc-proto-rs-redesign-capabilities-typed` dependency
nevertheless remains open for buffered-run execution/upload acceptance, per
the [catalog implementation report](../capabilities/implementation0.sol61m.md#upload-boundary-and-precise-next-seam).
This is a **ticket acceptance blocker**, not a missing check API. Therefore the
check ticket remains in progress until that native dependency is completed and
the parent integrates the final shared gates; no premature closure is claimed.

The parent owns shared design-index/root README links, tracked beads export and
integration gates. No push, subagent, `jj edit`, squash, tracker initialization or
migration was performed. This report links the accepted design, generic precedent,
completed access setup and listing foundation without rewriting those sources or
promoting this unstable API beyond its implementation standing.
