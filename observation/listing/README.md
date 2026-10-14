---
type: Reference
title: Scoped managed listing observations
description: Finite ls acquisitions, explicit publication, sampled differences, and caller-driven polling over registry custody.
resource: /observation/listing/README.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec60376ecffey5Px0wsjSP7MhR"
    title: Listing snapshot implementation and ownership decisions
  - resource: /design/library-redesign/draft1.gpt6a.md#10-observation-and-optional-reconciliation
    title: Scoped sampled observation requirements
    revision: "jj-commit:956d0596"
  - resource: /control/registry/src/lib.rs
    title: Inspected reserve, submit, wait, take and release custody API
    revision: "jj-commit:956d0596"
  - resource: /protocol/commands/src/outputs.rs
    title: Concurrent catalog correction requires listing IDs and nested containers
    revision: "sha256:70e80f9e1af8255ca0b99a0a39e2bd73fbdeae64cc02f0fd40ad794a2319a7eb"
---

# Scoped managed listing observations

`kitty-rc-listing` is an independent workspace leaf. It depends on commands,
exchange and registry, not the client facade. It implements observation only:
no desired state, mutation, implicit reconciliation, prune, or RC event stream.

## Finite entrance

Create an `Observer` with an existing `Registry`, immutable `Binding`, `Scope`,
`Interpretation` and projection `Limits`. The binding's subject and generation
are **caller assertions**, not verified kitty process identity. Recreate the
observer when its relevant binding, scope or capability evidence changes.
`Receipt::route()` identifies `kitty-rc/ls`; receipt metadata records all filters,
environment request policy, local issuance order, start, token and basis.

```rust,ignore
let acquisition = observer.acquire(options)?; // synchronous registry acceptance
let receipt = acquisition.receipt().clone();   // save for detachable recovery
acquisition.wait().await?;
let snapshot = acquisition.finish()?;         // sole take; explicitly releases
for change in observer.changes(&snapshot)? {
    // Consume the lazy sampled difference without retaining an output queue.
}
let previous = observer.accept(snapshot)?;    // explicit baseline advancement
drop(previous);                              // caller-selected retention
```

`Snapshot::listing()` borrows the owned typed listing without cloning it.
The local end timestamp is the time of take/interpretation, an upper bound
including any waiter delay; neither timestamp is a server-atomic sample clock.
The observer retains at most one accepted snapshot. Failed acquisition leaves
that baseline unchanged. A newer accepted submission invalidates older pending
samples **even if the newer acquisition fails**. A later arrival cannot win by
arrival order. Snapshots from a different observer owner are rejected, including
ones for the same endpoint. Rejection returns the snapshot intact.

## Knowledge and coverage

The typed `Ls` decoder tolerates additive unknown fields but requires structural
IDs and nested containers. A successfully decoded unambiguous listing has
`CompleteSelectedResponse` membership; duplicate IDs make it
`PartialSelectedResponse`. Completeness is relative to the response and exact
filters, never a claim about an entire desktop or a server-atomic interval.
Filtered snapshots are not absence evidence for unselected windows/tabs.
Environment completeness is recorded as **requested**, not invented for the
default differing-environment response.

Known IDs produce compact samples of parent OS-window/tab IDs, focus and active
status. Coverage defensively accounts for unidentified/duplicate IDs; the current
decoder rejects missing structural IDs before snapshot construction. Missing
focus values are `Fact::Missing`, not false. Focus support is chosen
from caller-supplied capability evidence: `Unsupported` differs from `Missing`;
unknown support with no value is `UnknownSupport`. An explicit supported false
value remains `Known(false)`. No global-focus winner is guessed from missing data
or filtered results. Required IDs/containers missing from a native reply produce
a typed acquisition failure rather than a partial authoritative listing.

`changes` compares compact samples lazily: `Returned`, `Changed`, and
`NoLongerReturned`. The last means only absence from the new returned projection,
**not window destruction**. Duplicate IDs make differences ambiguous and are
rejected, while the raw partial snapshot remains inspectable. The caller decides
whether to accept partial coverage. Missing success data, invalid typed output,
remote refusal, interruption and projection overflow are acquisition failures,
not fabricated empty snapshots. A valid `[]` remains an inspectable sampled empty
response, complete only for its selected scope.

## Custody and shutdown

Dropping `wait`, `Acquisition`, `Observer`, or `Poller` never cancels an already
accepted operation. Registry custody is deliberately retained until explicitly
taken/released. Save the receipt before detaching:

```rust,ignore
registry.wait(receipt.token()).await?;
let recovered = receipt.finish(&registry)?;
```

Receipt clones permit recovery, not replay or repeated takes. `finish` transfers
native outcome ownership before release, then decodes outside registry locks.
Native failures retain their bytes/progress/source in `AcquisitionFailure`.
Taking too early returns an unavailable failure **with the receipt** and leaves
the unfinished registry record intact for later recovery. `cancel()` is explicit
interruption only, not release or proof of rollback. Even after registry driver
stop, take the uncertain outcome before release. Unrecovered detached tokens can
fill the registry's finite quota; this is explicit retention, not automatic
cleanup. Registry driver shutdown policy remains its owner's responsibility.

## Polling and bounds

`Poller::new` starts no task. Its `next(timeout, max_response)` sleeps until the
selected nonzero cadence is due and then synchronously admits one finite
acquisition. The caller owns the loop/task, wait, output handling, accept and
shutdown. Cancellation during the sleep creates no accepted operation. Fixed
delay is measured from actual submission; delayed/missed polls are skipped,
not burst-replayed. Receipts record scheduled time and cadence alongside actual
start so delay and gaps are visible. There is no remote subscription/resume
cursor, retained history or hidden queue.

The registry bounds admitted records and native bytes (`Options::max_response`).
`Limits::max_windows` additionally bounds compact projection cardinality, including
unidentified windows. Typed listing allocations belong to the caller; this is not
a process-RSS hard limit. Listing decoding is inherited from the existing typed
command and is not made allocation-free here. Large listings are moved, not
cloned by the adapter. Comparisons are lazy and local; no observation lock spans
I/O, and the ordinary registry observation class participates in its pool policy.

## Cross-references

- [Implementation evidence](../../design/library-redesign/observation/implementation0.sol61m.md): scoped choices, gates and remaining limits.
- [Registry correction standing](../../design/library-redesign/implementation0.sol61m.md#current-correction-standing): reusable sessions, native outcome custody and driver lifetime.
- [Observation architecture](../../design/library-redesign/draft1.gpt6a.md#10-observation-and-optional-reconciliation): samples are not event subscriptions or retrospective mutation acknowledgment.
- [Kubeless adaptation](../../design/library-redesign/kubeless-adaptation1.gpt6a.md#8-explicit-policies-ownership-and-guarantees): separate observed satisfaction, native outcome and continuing authority.
