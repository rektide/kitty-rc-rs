---
type: TicketRoadmap
title: Registry-first redesign epics and ticket landing map
description: Prepared stable-ID work plan with concrete acceptance, dependency edges, preserved CLI backlog and a blocked legacy-store recovery decision; no live tickets created.
resource: /design/library-redesign/tickets0.sol61m.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec71b1985ffepI33XRmG1tf0Z9"
    title: Commissioned draft-to-ticket planning, read-only legacy tracker diagnosis, preserved backlog mapping and prepared unapplied dependency plan
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Current human-corrected architecture and selected direction
  - resource: /design/library-redesign/README.md
    title: Current design standing and preserved research map
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md
    title: Concurrent custody integration candidates and native scheduling constraints
  - resource: /design/library-redesign/kubeless-design0.gpt6a.md
    title: Read targeted framing, lifecycle, crypto and upload contracts
  - resource: /design/upstream-kitty/evolution0.gpt6a.md
    title: Read support baseline and stable/HEAD wire distinctions
  - resource: /.beads/issues.jsonl
    title: Existing 29-record interoperability export inspected and compared read-only with legacy SQLite IDs/statuses
  - resource: file:///home/rektide/src/rekon/praxis/beads/README.md
    title: Stable IDs, native parent edges and non-destructive storage boundaries
  - resource: file:///home/rektide/src/rekon/praxis/ticket-pointers/ticket-pointers0.glm53fx.md
    title: Current document pointers and append-only work history
verification:
  scope: Prepared plan only; JSON uniqueness, dependencies, parent namespace, source pointers and legacy correspondence checked locally
  limits: Live bd issue creation and dependency checks blocked by absent configured Dolt database; no library implementation, migration or live kitty test
---

# Registry-first ticket roadmap

**Prepared, not applied.** Ticketing was explicitly commissioned, but the current
`bd` cannot open the project's legacy store. The [machine-readable plan](tickets0.sol61m.json)
contains exact proposed IDs, descriptions, acceptance criteria, current pointers,
existing-ticket updates and dependency edges. It is a custom planning schema,
**not** an input to `bd import` or `bd create --graph`.

## Tracker blocker and recovery decision

Read-only inspection established:

- `bd version`: **1.0.3**, build `772a656888d3`.
- `bd where`: this project's `.beads`; no redirect or workspace mismatch.
- `bd context --readonly --json`: backend **Dolt**, embedded mode, database
  `beads`. No `.beads/dolt` exists; neither project config nor environment points
  to an alternate live store. No user `.config/beads` or `.beads` directory was found.
- `.beads/metadata.json` still names `beads.db`, a **356,352-byte SQLite** database.
  SQLite opened using `mode=ro`: **29 issues, 20 closed, 9 open**, plus labels,
  comments, events and dependency tables. The export has the same IDs/statuses.
- Required `bd config get issue_prefix` fails with **no beads database found**.
  Read-only legacy `config` confirms `issue_prefix = kitty-rc-proto-rs`; this is the
  preserved prefix used here, not a claim that current `bd config` succeeded.
- `bd bootstrap --dry-run --json` proposes **jsonl-import**, reports
  `has_existing: false`, and would import `.beads/issues.jsonl`. That flag describes
  the absent Dolt store, **not** absence of legacy SQLite history.

**Decision needed:** identify an already-canonical Dolt store, or approve an
explicit backed-up recovery/import procedure that accounts for SQLite events,
comments, labels, dependencies and closed work as well as issue rows. The dry-run
JSONL bootstrap is a candidate, not an approved full-history migration. Do not
initialize over SQLite, blindly run bootstrap/migrate, edit the SQLite store with
SQL, or overwrite its export before that decision. No tracker data/config was
changed, no issues were created, and no other agents' ownership was claimed.

Once recovered, re-read live work first: G and the implementation agent may have
advanced the work while this plan was blocked. Reconcile rather than creating
duplicate IDs or overwriting their reports. The plan is ready for explicit-ID
CLI application, with parent-child edges separate from blocking edges.

## Work anchors and standing

Umbrella proposed ID: **`kitty-rc-proto-rs-redesign`** — independently usable concern
crates, correct typed exchanges and registry-owned concurrent operations. Local
APIs are unstable; **zooming-kittens imposes no compatibility obligation**. Kitty
support and existing stored records remain distinct obligations.

In the tables below, `R` means **`kitty-rc-proto-rs-redesign`**; expand it literally
when using a native command. All detailed records and edges use full IDs in JSON.

| Epic ID | Architectural outcome | Completion boundary |
| --- | --- | --- |
| `R-core` | Exact pure protocol, typed profiles and cancellation-safe session | Source-backed framing/output and phase-fault tests; honest outcomes |
| `R-registry` | Concurrent leases and retained accepted operations | First-slice isolation, discoverable admission, bounded custody and shutdown inventory |
| `R-access` | Explicit setup, separate discovery/credentials/store | No hidden crypto I/O; generation-safe reuse and deliberate record preservation |
| `R-capabilities` | Audited stable command/nested/upload contracts | Typed catalog and exact negotiated-upload conversations |
| `R-observation` | Scoped finite observations and explicit follow-up | Sampled changes and check-only uncertainty follow-up, no implicit controller |
| `R-verification` | Supported-version evidence and adoption entrances | Support decision, reproducible conformance and runnable independent entrances |
| Existing `kitty-rc-proto-rs-slb` | CLI adoption | Existing subcommands/config backlog integrated, not duplicated |

The six new concern epics and existing CLI epic are children of the umbrella.
Ordinary build tickets are selected scope. `registry-kubeless` and
`verification-support` are explicit decisions/experiments: their results select
implementation details, not an excuse to silently grow initial scope.

## First vertical slice

**First build acceptance includes a managed concurrent registry**, not just a
correct exchange engine. The core, access and registry work are one cross-concern
journey; crate-by-crate progress alone does not close its checkpoint.

| Ticket ID | Concrete deliverable / acceptance emphasis | Prerequisites |
| --- | --- | --- |
| `R-core-reset` | Record G's already-started teardown/workspace boundary; preserve docs/tracker/history and hand off foundations | Already underway externally; completion requires G's actual report |
| `R-core-wire` | Pure schema/version and bounded incremental DCS decoder; fragmented/coalesced/surplus fixtures | reset |
| `R-core-typed` | Associated outputs/profiles for listing, text, short focus/action and deferred long wait; finite raw entrance | wire |
| `R-access-explicit` | Explicit endpoint/key/password inputs, pure crypto and exactly one protection envelope | wire |
| `R-core-session` | Single-owner transitions/Tokio driving, absolute budget, progress evidence and pre-I/O invalidation | typed, access-explicit |
| `R-registry-admission` | Select token or reserve-commit; atomically install input/ID before eligibility; refusal preserves owned input | reset |
| `R-registry-kubeless` | Compare native/direct custody/retained host with three scripted journeys; choose justified integration | Can start now without teardown completion |
| `R-registry-leases` | Fair endpoint/classes, connecting quota, independent exclusive leases and setup reuse | admission, session |
| `R-registry-custody` | Store before notify; non-Clone output ownership/bounds/release; explicit cancel/driver replacement/shutdown | admission, kubeless decision |
| `R-registry-slice` | Integrated direct/managed entrances and three-connection isolation fixture | leases, custody |

G's supplied session is
`opencode://ses_ec71e81e3ffercjuzWSBbKFI5w`; the reset ticket records that work,
does **not** assign or claim G, and does not authorize a second teardown. The
implementation agent can use this dependency map after G's handoff even while
tracker recovery is pending.

Required first-slice fixture: one endpoint has A waiting for process/selection,
B reading a large finite response, and C completing a short command **before**
A/B release. Readers remain exchange-local. Dropped waiters leave accepted IDs
and outcomes retrievable; global/endpoint/connecting and memory budgets hold;
interrupted transports do not re-enter the pool. This demonstrates local
independent progress, not parallel server CPU or exactly-once effects.

Admission tests must drop submission **after acceptance but before handle
delivery** as well as before acceptance. Custody tests cover output ownership,
full retention budget, queued versus started cancellation, stale completion,
driver loss without blind replay, and shutdown's unresolved inventory. These
contracts are required first-slice scope, not optional nice-to-haves.

## Subsequent coherent landings

| Ticket ID | Deliverable / acceptance emphasis | Prerequisites |
| --- | --- | --- |
| `R-access-resolve` | Injectable discovery/credential acquisition, pure selection and generation-safe preparation/reuse | slice |
| `R-access-store` | Shared library/utility boundary; legacy TSV preservation/import fixtures; explicit cleanup only | access-explicit |
| `R-capabilities-audit` | Pinned stable outer RC **and nested** action/kitten/config inventory; unresolved/HEAD entries visible | Can start now |
| `R-capabilities-typed` | Extend associated-output families from audited fixtures, not old serializer round trips | audit, slice |
| Existing `kitty-rc-proto-rs-vid` | Correct Action payload, typed known nested profiles and honest raw escape hatch | audit, slice |
| `R-capabilities-upload` | Exact profile handshake/empty/EOF/no-response behavior and alias-aware accumulator lane | audit, slice |
| `R-observation-snapshots` | Scoped route/time/coverage listing and sampled changes; failed/partial/stale acquisition | slice |
| `R-observation-check` | Fresh explicit read/check after uncertainty; native outcome retained, no mutation replay | snapshots, capabilities-typed |
| `R-verification-support` | Separate tested Rust/kitty floor and provisional capability/version policy | Can start now |
| `R-verification-live` | Isolated chosen-version real kitty conformance, including concurrent leases/reuse/uploads/crypto | support, typed catalog, uploads, access-resolve |
| `R-verification-examples` | Compile/run pure, direct and managed entrances; cancellation/retention/ordering contracts | slice, support |

Dependencies are readiness constraints, not permission to defer all verification
to `verification-live`. Every feature's acceptance includes its own scripted or
pure fixtures; the first slice includes an integration harness. The live matrix
can be built incrementally while its final ticket awaits complete coverage.

### Capability and support boundaries

[Draft §6](draft1.gpt6a.md#6-commands-include-the-larger-surface-inside-action)
requires the larger nested vocabulary behind `action`, built-in/installed
`kitten` return/installation contracts and broad `load-config` schema/invalidation.
The capability audit covers these, not merely a top-level RC list. Existing
`vid` owns Action implementation; no duplicate Action ticket is proposed.

[Upload research](kubeless-design0.gpt6a.md#2-what-kittys-streaming-protocol-actually-requires)
is the command-specific oracle: first nonempty handshake, appropriate empty-input
path, exact chunks/terminator, required handshake even where `no_response`
suppresses terminal result, unsupported async suppression and producer failures.
Known aliases share the server-command accumulator exclusion; unrelated work
must remain eligible. Interrupted-upload logical uncertainty is not cleared
merely because its retired socket freed a physical slot.

[Upstream evolution](../upstream-kitty/evolution0.gpt6a.md) distinguishes the old
0.43.1 declared target, latest stable 0.49.2 and inspected unreleased 0.50 HEAD.
Auditing stable schemas does **not** select a support floor or require every
request version to be raised. `verification-support` owns that decision.

## Existing backlog

There are **nine open issues total**, not nine CLI children: `slb`, seven open CLI
children, and separate Action `vid`. The existing closed CLI foundation `htb`
and all other closed history remain untouched. IDs are retained as deliberate
legacy exceptions to new child naming; no bulk rename or history replacement.

| Existing ID | Keep as owner of | New blocker(s) / overlap resolution |
| --- | --- | --- |
| `kitty-rc-proto-rs-slb` | CLI epic | Add umbrella parent; keep original child edges |
| `kitty-rc-proto-rs-goy` | list-windows subcommand | slice |
| `kitty-rc-proto-rs-b9t` | old standalone list binary/output adoption | goy; share implementation, not second list engine |
| `kitty-rc-proto-rs-e6p` | active-window / active / a | snapshots |
| `kitty-rc-proto-rs-g4x` | goto / g | slice; explicit selected target and ordered effects |
| `kitty-rc-proto-rs-wbs` | watch / w | snapshots; sampled finite polling, not reconciliation |
| `kitty-rc-proto-rs-aqh` | endpoint/global socket config | access-resolve; precedence and provenance |
| `kitty-rc-proto-rs-74i` | encryption config | access-resolve, access-store; explicit prepared policy |
| `kitty-rc-proto-rs-vid` | outer/nested Action | audit, slice; add capabilities parent |

Prepared `bd update --append-notes` content preserves historical descriptions and
notes, adds current pointers and records this session. Acceptance criteria are
updated to the new facade; no old closed API work is falsely reopened or credited
as verification of the replacement architecture.

## Experiments and separately selectable work

[Kubeless comparison](draft1.gpt6a.md#9-kubeless-a-constructive-integration-decision)
is a serious integration decision, not a pre-decided no-dependency verdict. Compare
three journeys: independent long/read/short work; lost launch reply/waiter/driver
then explicit listing; new desired generation while old result/observation arrives.
Keep ownership guarantees and duplicated bookkeeping in the comparison.

- **Potential upstream kubeless fair work-lane seam:** if selected host cannot
  supply native eligible selection, request a kubeless-owned change and link a
  fetchable reviewed upstream revision before local adoption. It is not a local
  ticket pretending to own the other repository.
- **Potential generic quota/exclusive claims:** extract only with a demonstrated
  second consumer. No mandatory speculative claims crate.
- **Later continuous goals/reconciliation:** separately select an actual workflow,
  target/channel and writer policy. Apply-once, observe-until, maintain-while and
  yield-on-divergence differ; a check-only read does not reissue old mutation input.
- **Terminal/custom kitten observation routes:** independently commission target
  binding, exclusive terminal I/O or verified deployed extension contract. Base
  RC does not silently read a caller's terminal or claim stock arbitrary queries.

These options are inventoried in JSON but **no build tickets are created for
them by this plan**. Registry concurrency and observation primitives do not depend
on implementing all higher-level possibilities.

## Application and verification handoff

After approved recovery, verify prefix via live `bd config get issue_prefix`,
compare canonical IDs/history, and inspect in-flight work. Apply creates with
explicit `--id`; apply existing updates append-only; then add exact edges. For
example, the dependent comes first:

```sh
bd dep add kitty-rc-proto-rs-redesign-core-wire kitty-rc-proto-rs-redesign-core-reset --type blocks
bd dep add kitty-rc-proto-rs-redesign-core-wire kitty-rc-proto-rs-redesign-core --type parent-child
```

The prepared graph contains no epic-as-blocker shortcuts: parent-child edges
describe ownership, feature blockers describe readiness. Validate issue IDs,
`bd children`, `bd show` cross-epic prerequisites and `bd dep cycles` against the
**live** recovered store. Offline validation is not a claim that native edges
already exist. Only then produce a legitimate tracked export if needed; never
overwrite recovery evidence first. No push is authorized.

Offline checks passed for **27 creates (7 epics, 20 concrete tickets), 9 existing
updates, 28 added parent edges and 37 added blocking edges**. Proposed IDs are
unique and do not collide with legacy records; new child namespaces and one-parent
structure hold. The combined legacy/proposed ownership and blocking graph is
acyclic. Current source-file/heading pointers and roadmap/update links resolve;
legacy SQLite/export IDs and statuses match. Native live checks remain blocked.

The parent owns [the design index](README.md); this session adds only this roadmap
and its machine plan. The index can link here as an **unapplied ticket plan blocked
on tracker recovery**, rather than advertise live created epics.
