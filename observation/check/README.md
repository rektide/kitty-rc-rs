---
type: Reference
title: Explicit check-only follow-up after uncertain effects
description: Bounded listing checks with separate native outcome custody, explicit matching knowledge and no write replay.
resource: /observation/check/README.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec5f1ef25ffexa7YLs09vVLxPL"
    title: Finite native check implementation, actual setup-basis integration and lost-reply/late-effect verification
  - resource: /design/library-redesign/draft1.gpt6a.md#10-observation-and-optional-reconciliation
    title: Check-only input and independent observed satisfaction boundary
  - resource: /observation/listing/README.md
    title: Scoped snapshot, receipt, coverage and detachable custody foundation
  - resource: /control/registry/src/setup.rs
    title: Inspected independent local setup generations and exact accepted basis API
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md#strongest-concrete-precedent-unknown-result--check-only-continuation
    title: Generic native unknown-to-read continuation precedent
---

# Explicit check-only follow-up

`kitty-rc-check` is an independently usable observation/control leaf above
`kitty-rc-listing` and the existing registry. It starts no task, controller, poll
loop or timer. Every acquisition is one explicitly admitted `ls`; no entrance
accepts a mutation command, performs cleanup, or implicitly retries a write.

## The finite journey

1. Submit a native command normally and save its token. Waiter/handle loss does
   not remove registry custody.
2. Once its retained native result is completed and unresolved, call
   `RetainedOriginal::capture(&registry, token, binding, max_subject_bytes)`.
   This stores bounded summary/reference metadata, **not copied native bytes or
   the original command**. The subject and binding generation remain caller
   assertions, not proof of kitty process incarnation.
3. Select `Request` scope, interpretation, conjunctive `Criterion`s, `Condition`,
   freshness and `Limits`, then construct `original.check_input(request, limits)`.
   `CheckInput` contains only fresh read/check input and correlation metadata.
4. `input.start(&registry, options)` synchronously admits one ordinary observation
   operation. `check.wait().await` detaches safely; `check.finish()` transfers the
   fresh listing and assessment, then releases **only that read's** native record.
5. Inspect `CheckReport.input`, `.snapshot` and `.assessment`. Decide explicitly
   whether another independent check is wanted. No result/error schedules it.
6. Keep the original separately. `original.take()` transfers the unchanged
   `Outcome` and original summary/boundary by ownership, then releases its registry
   slot. The caller now owns that evidence. There is no inferred acknowledgment.

The compiled Rust example lives on [`CheckInput`](src/lib.rs). A new read may be
admitted after transferring the original outcome: its bounded boundary/reference
remains useful, but then the caller must retain the transferred native evidence.

## Knowledge and matching

All criteria are ANDed: exact window ID, title, PID, cwd, user-variable value,
focused or active state. There is no arbitrary captured predicate/command callback
inside the continuation. A missing required fact is indeterminate; missing user
variables are not certified absence. Unsupported/unknown focus does not become
false. A known mismatch excludes a row despite unrelated missing facts.

`Matching` returns **Absent**, **Unique**, **Ambiguous** or
**EvidenceInsufficient**. Ambiguity exposes the exact match count plus a bounded
candidate-ID prefix and truncation flag. Missing required facts expose known
match and indeterminate-row counts. A fresh decoded empty listing can establish
absence **within the exact selected scope**; failure, duplicate IDs and missing
required structure cannot. No desktop-wide absence or window destruction is
inferred from a filtered result.

| Required condition | Absent | Unique | Ambiguous | Insufficient |
|---|---|---|---|---|
| Present | Unsatisfied | Satisfied | Satisfied | Unknown |
| Unique | Unsatisfied | Satisfied | Unknown | Unknown |
| Absent | Satisfied | Unsatisfied | Unsatisfied | Unknown |

Matching, condition satisfaction and causal attribution remain separate.
`Attribution::NotEstablished` is always explicit. Another actor may create the
same candidate, IDs/titles may be reused, and an old launch may still take effect
**after** a fresh listing. Observed absence is not a remote completion barrier;
observed satisfaction never proves that the lost-reply command caused it.

## Scope, freshness and basis

Assessment requires exact caller `Binding`, `Scope` and `Interpretation` equality,
complete selected-response membership, and the selected projection bound.
Acquisition must start **strictly after** the local unresolved-result capture;
equal timestamps are insufficient. Conservative age is measured from acquisition
start, including queue/waiter delays, not from late arrival or decoding time.
Snapshots acquired before the boundary are rejected even if their delayed reply
contains the eventual effect. Timestamps are local, not server-atomic clocks.

Managed start/finish additionally compare the original `Summary.setup`, the
check's accepted setup and current `Registry::setup_version`. Native binding and
credential generations are independent local coordinates qualified by endpoint;
neither is server incarnation or authorization. A changed basis rejects admission
or produces `EvidenceInsufficient`, preserving the read snapshot and original
outcome. Revalidation is local evidence, not a fence against later external change.

`CheckInput::assess(&snapshot)` is also available for independently acquired
listing snapshots. It checks their **declared** metadata/coverage/freshness and
returns `NativeSetup::NotChecked`; it must not be credited with the additional
managed registry setup validation. Neither entrance advances an observer baseline
or chooses a global latest sample across independent checks.

## Custody, drop and finite bounds

Save `check.receipt().clone()` before dropping its waiter/handle. Recovery is:

```rust,ignore
registry.wait(receipt.token()).await?;
let report = receipt.finish(&registry)?;
```

Receipt clones enable one take, not repeated takes or replay. Premature finish
returns `CheckFailure` with the receipt and acquisition error; the unfinished
record remains recoverable. Native read errors transfer their existing bytes,
progress and causes without cloning them. Projection overflow inherits listing's
diagnostic-only failure, not a fabricated empty snapshot. `check.cancel()`
interrupts only the fresh read; it never cancels/reverts/releases the original.

Dropping a check, receipt or `RetainedOriginal` never cancels or releases accepted
custody. Native results survive waiter loss, not destruction of all in-memory
owners. Save original tokens/use registry inventory for recovery. Completed
originals continue counting against finite accepted/output quotas until explicitly
taken/released; a new check can be refused when retention fills those quotas.
External consumers must coordinate sole takes. An explicit external release can
discard evidence; this wrapper neither conceals that choice nor restores it.

`Limits` bounds metadata byte lengths, criteria count, projected windows and
retained candidate IDs. Registry admission/output limits independently bound native
reads. Listings move by ownership; reports retain one snapshot and bounded check
metadata, not a history. This is not a process-RSS hard limit on caller-created
allocations or unlimited caller clones. Driver, shutdown and retention policies
remain with the existing registry owner.

## Cross-references

- [Scoped listing](../listing/README.md): the only acquisition/projection/custody
  engine composed here; polling remains a separately selected caller-owned policy.
- [Implementation evidence](../../design/library-redesign/observation/check0.sol61xh.md):
  concrete gates, fixture coverage, generic reuse assessment and dependency standing.
- [Access implementation](../../design/library-redesign/implementation/access0.sol61xh.md#registry-setup-ownership-and-exact-generations):
  local generation/lease semantics; this leaf validates them without owning setup.
- [Architecture](../../design/library-redesign/draft1.gpt6a.md#10-observation-and-optional-reconciliation):
  optional checks do not force desired-state maintenance onto managed execution.
