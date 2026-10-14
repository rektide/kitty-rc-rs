---
type: ImplementationReport
title: Finite scoped listing snapshots over managed custody
description: Native listing adapter with caller-owned acceptance, detached recovery and sampled polling, without implicit reconciliation.
resource: /design/library-redesign/observation/implementation0.sol61m.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec60376ecffey5Px0wsjSP7MhR"
    title: Listing implementation, strict structural coverage adaptation, and seven verified custody/publication/polling journeys
  - resource: /design/library-redesign/draft1.gpt6a.md#10-observation-and-optional-reconciliation
    title: Accepted observation boundary and sampled freshness requirements
    revision: "jj-commit:956d0596"
  - resource: /design/library-redesign/implementation0.sol61m.md#current-correction-standing
    title: Reusable registry correction and explicit native custody
    revision: "jj-commit:956d0596"
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md
    title: Fact knowledge, observation ownership and capability basis concepts
    revision: "jj-commit:956d0596"
  - resource: /protocol/commands/src/outputs.rs
    title: Inspected baseline tolerant listing model
    revision: "jj-commit:956d0596"
  - resource: /protocol/commands/src/outputs.rs
    title: Concurrent catalog correction now requires structural listing IDs and containers
    revision: "sha256:70e80f9e1af8255ca0b99a0a39e2bd73fbdeae64cc02f0fd40ad794a2319a7eb"
extensions:
  ticket: kitty-rc-proto-rs-redesign-observation-snapshots
---

# Finite scoped listing snapshots over managed custody

## Owned slice

Implemented `observation/listing`, an independently usable `kitty-rc-listing`
leaf over the public managed typed `Ls` entrance. It does not depend on the
facade or modify registry, access, exchange or commands. [Its maintained
README](../../../observation/listing/README.md) documents public ownership and
coverage; [journey tests](../../../observation/listing/tests/journeys.rs) exercise
native fake-server traffic, not a second mock observation engine.

The small native adapter uses the useful fact/knowledge and explicit baseline
concepts from kubeless research without adding unpublished local path
dependencies, a Kubernetes/HTTP facade, or a duplicate custody engine. Registry
custody remains the sole accepted operation owner. No new generation of remote
endpoint identity is invented: binding generation is explicitly caller-supplied.

## Decisions

- Synchronous `Observer::acquire` returns an accepted acquisition and saveable
  receipt. Waiting can detach; receipt recovery takes native custody exactly once.
- Local order belongs to one observer owner. Explicit `accept` advances one
  baseline, returns the predecessor, and rejects older issuance and foreign owners.
  New failed issuance does not install empty state or revive older pending work.
- Failures move native evidence to caller-owned errors and release taken records.
  Unfinished records remain retained. No destructor cancels, prunes, or spawns a
  cleanup task that might erase uncertain outcomes.
- Exact route, subject assertion, endpoint ID, binding generation, scope,
  interpretation evidence, local issuance and acquisition interval are retained.
- The catalog correction requires structural IDs/containers, so successful
  unambiguous decoding establishes complete selected-response membership, never
  unfiltered desktop membership or atomicity. Duplicate IDs produce explicit
  partial projection coverage. Missing structural data is a typed failure;
  missing optional facts differ from unsupported
  capability and known false, and sampled disappearance is `NoLongerReturned`.
- Polling is caller-driven fixed delay, with scheduled time/cadence recorded in
  receipts. Delayed polls are skipped, with no implicit catch-up queue, permanent
  constructor task, server watch stream or cursor.
- Registry admission/output budgets remain effective; a caller-selected window
  projection limit bounds samples. Borrowed listings and lazy differences avoid
  cloning full decoded trees or retaining an output history.

## Verification standing

Verified the requested human toolchain entrance with `mise exec -- rustc
--version`: `rustc 1.101.0-nightly (822182db4 2026-10-13)`. No Rust version pin or
MSRV promise was introduced.

The first targeted test attempt was blocked by the concurrent access workspace
member being declared before its manifest existed. A later attempt advanced to
commands compilation, where concurrent catalog module declarations preceded their
files (`actions`, `appearance`, `catalog`, `configuration`, `process`). These are
temporary sibling-work blockers, not observation gate results. An ignored scratch
archive at `.test-agent/observation-baseline/` first verified six journeys against
the committed managed API at `0ace4ac3`; it exposed a missing public-module import
and a fixture that incorrectly closed reusable connections. Both were corrected.
That scratch result is not credited as compatibility with later sibling changes.

The final shared working-copy gates passed after catalog files became buildable:

| Gate | Result |
|---|---|
| `mise exec -- cargo nextest run -p kitty-rc-listing --locked` | 7 passed |
| `mise exec -- cargo check -p kitty-rc-listing --all-targets --locked` | Passed |
| `mise exec -- cargo clippy -p kitty-rc-listing --all-targets --no-deps --locked -- -D warnings` | Passed; owned crate/all targets linted |
| `mise exec -- rustfmt --check --edition 2024 observation/listing/src/lib.rs observation/listing/tests/journeys.rs` | Passed |

The seven journeys check exact filtered wire scope and receipt binding, complete
selected-response coverage, missing optional facts versus unsupported focus,
partial duplicate-ID coverage, missing structural-data failure, projection
overflow, sampled focus changes, failed acquisition retaining the baseline,
stale arrival, failed-new-generation invalidation, replacement-binding ownership,
waiter/acquisition drop recovery, driver-stop uncertainty, polling sleep/drop
ownership, and premature take followed by explicit recovery/release.

An initial scratch Clippy invocation including dependencies found the existing
registry `collapsible_if` lint. Dependency-wide lint is not claimed here; the
owned all-target `--no-deps` gate passed. Parent integration owns full-workspace
gates and shared index/root README connections.

Saved coherent implementation checkpoint `3e31c409`; workspace member and lock
package were isolated with full-ID `jj-hunk` dry-run/commit in `0708df24`, leaving
other agents' access/catalog hunks untouched. The follow-up correction seals
receipt ordering behind accessors, bounds error result sizes, and adapts coverage
to strict structural listing decoding. These checkpoints are implementation
evidence, not human promotion of the unstable API.

## Remaining limits and connections

No server identity verification, exhaustive unfiltered desktop coverage,
global desktop focus winner, terminal channel, active-window CLI, watch CLI, or
check-only uncertain mutation follow-up is implemented. Those require their own
native policies/entrances. Polling and sampled focus/window facts equip the
existing follow-up tickets without owning desired state.

The [architecture's observation boundary](../draft1.gpt6a.md#10-observation-and-optional-reconciliation)
is the scope authority. The [registry implementation's correction standing](../implementation0.sol61m.md#current-correction-standing)
supplies reusable pool/custody context. The [kubeless ownership discussion](../kubeless-adaptation1.gpt6a.md#8-explicit-policies-ownership-and-guarantees)
is the conceptual precedent: observation can establish current satisfaction but
cannot rewrite a previous unknown effect into acknowledgment.
