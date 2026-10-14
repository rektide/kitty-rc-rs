---
type: TicketRoadmap
title: Registry-first redesign epics and ticket landing map
description: Applied stable-ID work plan with concrete acceptance, dependency edges, preserved CLI backlog and verified history-preserving SQLite to embedded Dolt recovery.
resource: /design/library-redesign/tickets0.sol61m.md
status: active
generated:
  by: "model:openai/gpt-6.1-sol#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec71b1985ffepI33XRmG1tf0Z9"
    title: Commissioned draft-to-ticket planning, operator-approved backed-up history-preserving tracker migration, native issue application and verification, and preserved backlog mapping
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
  - resource: https://github.com/steveyegge/beads/tree/772a656888d35aedab5a8c070fc042205db17a97
    title: Exact installed source inspected for bootstrap/import, schema, SQL, config, export and legacy migration script limitations
verification:
  scope: All 27 creates and 9 existing updates applied natively; complete legacy archive, 46 native historical events, compatible issue fields, dependencies, child edges, cycle checks, backup hashes and pointers verified
  limits: Native dates have whole-second precision; exact legacy timestamps retained in archival provenance and original SQLite. No library implementation or live kitty test; no remote backup/push.
---

# Registry-first ticket roadmap

**Applied after approved, history-preserving tracker recovery.** There are now
**56 live issues: 36 open and 20 closed**. This session created 7 epics and 20
concrete tickets, updated the 9 existing open issues append-only, and installed
28 new parent edges and 37 blocking edges. No implementation was claimed or closed.
The [machine-readable plan](tickets0.sol61m.json)
contains exact native IDs, descriptions, acceptance criteria, current pointers,
existing-ticket updates, dependency edges and application provenance. It is a custom planning schema,
**not** an input to `bd import` or `bd create --graph`.

## Tracker recovery and preservation

Initial read-only inspection established the version/backend mismatch:

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

The operator subsequently approved **backed-up, history-preserving migration,
verification of existing records, then application**. The local beads archive
exactly matches installed build `772a656888d35aedab5a8c070fc042205db17a97`.
Its migration script omits historical events and replays dependencies without
preserving types; it was inspected but **not executed**.

Actual method:

1. Before any tracker change, copy the entire original `.beads` tree and make a
   consistent SQLite backup. Check `PRAGMA integrity_check`, all table rows and
   SHA-256 file manifest. Baseline: **29 issues, 10 dependencies, 46 events,
   11 config entries, 6 metadata entries, 0 comments and 0 labels**.
2. Run supported non-destructive `bd bootstrap --yes --json` against this project.
   It creates embedded Dolt and imports the existing 29-record JSONL snapshot;
   it neither replaces nor removes original SQLite.
3. Use the supported local Dolt SQL entrance in one transaction to archive every
   legacy table schema and all **135 source rows**, including metadata/config and
   old caches/sequence values, with per-row SHA-256. Restore every compatible
   native issue field, dependency type/actor/provenance and legacy config value.
   Correct bootstrap's closed/dependency date timezone shift using explicit UTC.
4. Transfer all **46 historical events** into native `events`, preserving actor,
   old/new payload and comment content. Deterministic UUIDs replace SQLite integer
   IDs; `legacy_sqlite_event_map` retains the exact mapping. Keep bootstrap's 29
   migration-created events rather than rewriting history.
5. Verify all legacy rows/native fields and history before any plan mutation.
   Reinspect live issues for concurrent changes; none were present. Then create
   exact IDs, update legacy acceptance and append notes, and add native edges.

Canonical source of truth is now **`.beads/embeddeddolt/beads`**, selected explicitly
by tracked `.beads/metadata.json` (`backend: dolt`, `dolt_mode: embedded`, database
`beads`). Local databases/backups are ignored; `.beads/issues.jsonl` is only a
Git-tracked interoperability export. `export.auto` is supported and **true**;
an explicit final export ensures the snapshot is current. No cleanup/pruning,
remote backup, push, other tracker instance or issue assignment was performed.

### Recoverable evidence and history limits

Verified backup directory:
**`.test-agent/tracker-migration/backup-20261014-ses_ec71b1985ffepI33XRmG1tf0Z9/`**.
It contains original `.beads/`, `sqlite-consistent.db`, `baseline-tables.json`
and `SHA256.json`; directory is private and excluded through local Git excludes.
Original `.beads/beads.db` remains byte-identical in place. The scratch README,
transfer SQL/scripts and verification reports explain the procedure. A second
`verified-dolt-before-plan/` copy captures recovered canonical state before work
creation; `verified-dolt-after-plan/` and `post-plan-dolt-SHA256.json` preserve the
final canonical database and its native history. Original SQLite SHA-256 is
`95b2732e6a351a5b75be8353254a8fe9d67ce156d3c39d3686ca900b0d409bff`.
Migration checkpoint is Dolt commit
**`tthmp7mhp15lk5bnvj2pojnu969iihrr`**; the transfer itself was retained by native
schema auto-migration commit `lr80d3965jtrmvc0f0eeihgedk6ph7ne`.

Native schema uses `DATETIME(0)`: dates represent correct UTC instants to whole
seconds, not original nanosecond strings/offsets. Exact originals, deprecated
fields and SQLite event IDs remain lossless in `legacy_sqlite_archive`,
`legacy_sqlite_schema`, `legacy_sqlite_event_map`, `legacy_sqlite_migration`,
the unchanged SQLite file and verified backup. Historical events are queryable
from native Dolt; they are **not** fabricated as pre-migration Dolt commits.
`bd history` reports native version history starting at bootstrap; the older
actor/payload timeline is in the retained events/archive.

To inspect historical evidence without another store:

```sh
dolt --data-dir .beads/embeddeddolt/beads sql -q 'SELECT e.* FROM events e JOIN legacy_sqlite_event_map m ON e.id=m.native_id ORDER BY m.sqlite_id'
```

Deliberate restoration: stop all tracker writers; first preserve any newer
canonical Dolt state and export. Verify backup hashes and SQLite integrity, then
restore the complete backed-up `.beads` tree to a separately named recovery
directory or explicitly approved original location (including old config and
metadata). Keep current Dolt alongside, never overwrite it blindly. A compatible
SQLite-era tool or read-only SQLite can inspect the old state; `bd` 1.0.3 cannot.
For a new canonical recovery use the preserved records/history and verified
transfer procedure, not JSONL alone. No rollback or loss of later work is implied.

## Work anchors and standing

Umbrella native ID: **`kitty-rc-proto-rs-redesign`** — independently usable concern
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

The six new concern epics and existing CLI epic are native children of the umbrella.
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
implementation agent can use this dependency map after G's handoff. All newly
created issues remain open; the reset description records already-started work
without assigning another agent or presuming its completion.

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

Applied `bd update --append-notes` content preserves historical descriptions and
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

## Applied verification and implementation handoff

Live `bd config get issue_prefix` now returns `kitty-rc-proto-rs`. Creates used
explicit `--id`; existing updates were append-only and exact edges installed.
For any later work, inspect live state first rather than reapplying the historical
plan over advanced tickets. The dependent comes first:

```sh
bd dep add kitty-rc-proto-rs-redesign-core-wire kitty-rc-proto-rs-redesign-core-reset --type blocks
bd dep add kitty-rc-proto-rs-redesign-core-wire kitty-rc-proto-rs-redesign-core --type parent-child
```

The graph contains no epic-as-blocker shortcuts: parent-child edges describe
ownership, feature blockers describe readiness. Native checks passed:

- **56 issues: 36 open, 20 closed**; all 29 legacy IDs/statuses preserved.
- **27 creates (7 epics, 20 concrete tickets), 9 existing updates**; descriptions,
  acceptance and current pointer notes match the plan. Legacy descriptions and
  prior notes remain; closed issues were not reopened or updated by the plan.
- **75 native relationships: 36 parent-child, 39 blocks**, including original 10
  relationships. Exact directions/types match; `bd children` checked all eight
  epics, including all eight original CLI children (one closed).
- `bd show` confirms cross-epic prerequisite direction; `bd dep cycles --json`
  returns **`[]`**. Local combined ownership/blocking graph also has no cycles.
- **112 native events**, including all 46 exact transferred historical events,
  with lossless **135-row archive** still verified after plan application.
- Original SQLite byte identity and backup manifest hashes rechecked; 0 comments
  and 0 labels preserved; native `bd history` works. Current document pointers
  resolve. Final export is an issue snapshot, not the authoritative history store.

No remaining tracker blocker. Implementation tickets remain open and unassigned;
G's reset checkpoint and the concurrent first slice are next build milestones.
This recovery/application adds no source-code or live-kitty verification claim.
No push is authorized.

The parent owns [the design index](README.md); this session adds only this roadmap
and its machine plan, plus legitimate tracker configuration/export changes. The
index can now link here as an **applied ticket roadmap with verified recovered
history and native work IDs**.
