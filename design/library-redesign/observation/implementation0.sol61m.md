---
type: ImplementationReport
title: Finite scoped listing snapshots over managed custody
description: Native listing adapter with caller-owned acceptance, detached recovery and sampled polling, without implicit reconciliation.
resource: /design/library-redesign/observation/implementation0.sol61m.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec60376ecffey5Px0wsjSP7MhR"
    title: Implementation, custody and partial coverage decisions for listing snapshots
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
    title: Inspected tolerant typed listing model and omitted-container limitation
    revision: "jj-commit:956d0596"
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
- The existing decoder defaults omitted nested containers. Rather than claim
  completeness it cannot verify, coverage explicitly stays returned-rows-only.
  Duplicate/missing IDs are reported, missing facts differ from unsupported
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
temporary sibling-work blockers, not observation gate results. Tests and lint
results will be recorded here after dependencies become buildable.

## Remaining limits and connections

No server identity verification, authoritative exhaustive collection coverage,
global desktop focus winner, terminal channel, active-window CLI, watch CLI, or
check-only uncertain mutation follow-up is implemented. Those require their own
native policies/entrances. Polling and sampled focus/window facts equip the
existing follow-up tickets without owning desired state.

The [architecture's observation boundary](../draft1.gpt6a.md#10-observation-and-optional-reconciliation)
is the scope authority. The [registry implementation's correction standing](../implementation0.sol61m.md#current-correction-standing)
supplies reusable pool/custody context. The [kubeless ownership discussion](../kubeless-adaptation1.gpt6a.md#8-explicit-policies-ownership-and-guarantees)
is the conceptual precedent: observation can establish current satisfaction but
cannot rewrite a previous unknown effect into acknowledgment.
