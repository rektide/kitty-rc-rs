---
type: ImplementationReport
title: CLI adoption over explicit access and scoped observations
description: Implemented list, focused/active target selection, typed goto, bounded sampled watch, configuration source precedence and shared-store maintenance with actual executable journeys.
resource: /design/library-redesign/cli/implementation0.sol61m.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec5e53929ffezfWrk0e0AOF9Uc"
    title: Owned CLI/store implementation, native backlog adoption, isolated verification, parent-reviewed normal-absence correction and handoff
  - resource: /design/library-redesign/tickets0.sol61m.md#existing-backlog
    title: Native IDs and preserved list/standalone/config/observation mapping
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Accepted registry-first access and observation ownership
  - resource: /README.md
    title: Inspected pre-CLI first-slice standing and removed legacy compatibility surface
  - resource: /design/library-redesign/implementation/access0.sol61xh.md
    title: Actual discovery/source/generation API and the exact shared-store gap
  - resource: /observation/listing/README.md
    title: Scoped finite acquisition, explicit baseline, receipt custody and skipped-poll semantics
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Typed catalog/strict output and focus completion contract
  - resource: /doc/PLAN-pubkey-db.md
    title: Historical shell-publication need; automatic cleanup is superseded
  - resource: /src/bin/list_windows.rs
    title: Inspected legacy standalone source; useful human fields adopted without its guessed endpoint or false-success behavior
    revision: "jj-commit:e78489cb"
verification:
  scope: Actual kitty-rc and kitty-pubkey-db executables against private scripted Unix peers and isolated subprocess HOME/XDG/environment
  evidence: Initial 23 and post-review 26 targeted nextest tests passed; locked all-target check, strict no-deps all-target clippy and targeted formatting check passed; compiled --help inspected
  limits: No new live-kitty run or supported-version claim; no compositor focus, server subscription, automatic retries/pruning or transactional noncooperating-writer database
---

# Implemented CLI frontier

## Interfaces and ownership

`kitty-rc` implements:

- `list-windows` (`ls`, `list`), readable output and flat JSON window rows;
- `active-window` (`active`, `a`), unique global focused terminal or explicit
  `--tab` / `--os-window` active-target selection;
- `goto` (`g`) with stable window ID and typed reply-bearing `FocusWindow`;
- `watch` (`w`), finite `ls` sampling, cadence/count, Ctrl-C and JSONL changes;
- explicit native `--to` (`--socket`), declared bounded JSON `--config`, auth,
  key-file/shared-store/PID and named-password/file selection, deadlines,
  bounded responses and secret-free verbose source provenance.

`kitty-pubkey-db` keeps init/add/get and explicitly quiescent cleanup, now with
`--store`. Both binaries are thin wrappers over domain-grouped CLI library
modules: `interface`, `configuration`, `control`, `output`, `store`.
The app depends directly on the already implemented concern leaves. No library
API or parent-owned registry/discovery/credential/transport/catalog source was
modified. No second execution engine or old stub facade was introduced.

The [CLI README](/apps/cli/README.md) owns detailed flags, examples and safety
contracts. Leaf limits/receipts, source policy and registry setup remain native
library responsibilities, not a copied CLI framework.

## Consequential policy choices

**Selection:** explicit CLI route bypasses discovery acquisition; otherwise
captured selected kitty environment wins over declared configured routes. Within
the winning configured tier, multiple routes fail as ambiguous. Native path bytes
are preserved. Unsupported inherited/abstract addresses fail; no guessed socket,
ambient config sweep, PID suffix or fallback to the terminal.

**Auth:** explicit CLI > selected environment > declared JSON > plain default.
Key/password selectors replace the complete lower-tier source choice. Only the
selected source is acquired; `credentials::Policy` receives scoped sources with
`MissingPolicy::Stop`. No source error/missing material becomes plaintext. Default
encrypted key is captured route-bound publication; standalone key-file/store
explicitly declares endpoint association, with a PID required for shared-store
selection. Defaults do not invent verified server identity. Password defaults
to a named selected variable and is never part of args Debug or diagnostics.

**Focus:** global focus requires unfiltered response and all relevant focus facts;
per-tab active state is not a global winner. Explicit OS-window active selection
requires the unique active tab and terminal. Missing/duplicate/ambiguous targets
fail separately from failed acquisition. Goto only awaits kitty focus; no desktop
compositor operation or post-error mutation retry.

**Observation:** the app uses `Observer`/`Poller`, takes/releases every finished
record, accepts one baseline explicitly and drops displaced snapshots. Sequential
polling prevents out-of-order publication. Failed reads stop rather than emit a
stale success or fabricate empty state. Skipped slots aren't burst-replayed.
After parent review, valid zero-focus and empty replies continue watch with
`active:null` and `active_status:no-focused`; text emits `-\t(no-focused)`.
Scoped zero-active selection uses `no-active`. One-shot active remains strict;
missing facts/explicit targets, duplicate IDs and ambiguity remain errors.
JSON records include local order, selected scope, acquisition duration/scheduling
delay, active row and `Returned`/`Changed`/`NoLongerReturned` labels. Title-only
changes aren't compact projection changes; each successful sample still prints
the active title. No lossless subscription/destruction/desired-state claim.

**Lifecycle:** every process owns a concurrent registry and explicitly held
driver. Normal command/failure/Ctrl-C paths close intake, complete local cancel,
report residual progress/uncertainty, transfer/release owned native records and
stop the driver. Ctrl-C is exit 130. Dropped command futures do not implicitly
cancel accepted work; shutdown is the explicit owner action. Retained evidence
is in-process and reported, not a crash-durable transcript. Response limits,
queue/execution deadline and listing projection limits are explicit.

**Store maintenance:** [store report](store0.sol61m.md) records shared I/O,
byte preservation, create-new init and deliberate CLI lock/atomic cleanup.
The quiescent external-writer boundary is explicit rather than falsely claiming
the changed-byte check eliminates arbitrary concurrent-writer races.

## Existing standalone adoption (`b9t`)

Inspected old `src/bin/list_windows.rs` at `e78489cb`. Useful fields were window
ID/title, shell PID/CWD/argv, foreground process PID/argv/CWD and OS/tab grouping.
These are preserved in readable output and the actual executable JSON fixture.
The hardcoded `./kitty.socket`, stdout connection chatter, permissive malformed
listing skipping, and `Response ok:false` printed as a successful run are
deliberately not preserved. The standalone binary is not restored: `kitty-rc ls`
is the shared replacement. This is the ticket's authorized migration decision,
not a duplicate list engine or unstable Rust API compatibility obligation.

## Actual checks

User-selected mise Rust `nightly-2026-10-14`, without any toolchain/rust-version pin:

- `cargo nextest run -p kitty-rc-cli`: initial **23**, normal-absence correction **26 passed**, no skipped tests.
- `cargo check -p kitty-rc-cli --all-targets --locked`: passed.
- `cargo clippy -p kitty-rc-cli --all-targets --no-deps -- -D warnings`: passed.
- `cargo fmt -p kitty-rc-cli --check`: passed.
- `cargo run -p kitty-rc-cli --bin kitty-rc -- --help`: compiled/help inspected.

Journeys execute real subprocesses, not just parser/serializer round trips:

- list aliases and legacy useful human/JSON fields, fragmented native frames;
- focused versus per-tab/OS active target, missing/ambiguous/filter restrictions;
- stable goto payload and remote denial without retry;
- missing data, invalid structures, remote error, bounded response and deadline;
- 71 watch samples past the default per-endpoint record quota, sampled absence, skipped delayed
  polls, failed second poll with no third stale sample, accepted-read Ctrl-C with
  unresolved evidence and peer EOF;
- parent-review correction: 72 actual samples cycle focused → valid no-focus →
  empty → focused, retaining membership changes and explicit null/status without
  hidden mutation or record leaks; plain output tests exact no-focus sentinel;
  missing facts, duplicates and ambiguous winners still stop watch;
- explicit/environment/declared-config precedence, actionable missing/ambiguous
  endpoint and unsupported fd/abstract routes, non-Unicode explicit socket path;
- actual encrypted captured environment, shared legacy TSV and declared key/
  password file journeys with a server-secret decrypt oracle; exactly one
  protection layer, selected password, unchanged store bytes and no secret logs;
- malformed/missing/password multiline/binding failures without downgrade;
- byte-preserving store append/init/refused cleanup, competing-writer lock,
  surviving legacy CRLF cleanup bytes, explicit store and malformed numeric env.

No ambient human files/endpoint mutation. A strict dependency-inclusive clippy
attempt encountered a concurrent parent-owned registry `needless_return`; the
owned CLI no-deps strict gate passed instead. Parent owns final shared-tree gates.
No independent live kitty or global support-floor claim is made.

## Native tickets and handoff

Owned native work: `kitty-rc-proto-rs-redesign-access-store` and original CLI
children `goy`, `b9t`, `e6p`, `g4x`, `wbs`, `aqh`, `74i` beneath `slb`.
The store ticket was closed after its isolated preservation checks; all existing
IDs/history are preserved, without new substitute tickets. Dependencies are
honored (store before encrypted CLI; list before standalone adoption).
All seven selected CLI children and the existing `slb` epic are now closed
with native acceptance reasons/provenance; its older closed `htb` remains intact.

Checkpoints already retained: `ddb9d505` store acceptance;
`0a72ad12` isolated owned Cargo.lock source/signal hunks;
`cb9c4112` first executable CLI stage. Later modular-store, stricter lifecycle,
additional journeys are in `318a33d7`; full CLI reference/evidence is in
`95b25c40`. Final closure standing is recorded separately.
Cargo.lock's unrelated upload-engine dependency hunks remain with their owner.
No root workspace manifest changes were necessary.

Parent integration: update root README/workspace table and shared design index;
export/commit tracker snapshot; run converged workspace gates. No remaining
selected CLI feature scope is deferred. Live supported-version verification
remains with its existing verification owner, not a new CLI ticket.

Cross-links: [access implementation](../implementation/access0.sol61xh.md#existing-tsv-store-preserved-not-rewritten)
is predecessor evidence (its CLI-gap statement is historical);
[listing reference](/observation/listing/README.md#polling-and-bounds) owns sampled
semantics; [capability report](../capabilities/implementation0.sol61m.md#honest-outcome-and-reuse-boundaries)
owns typed decoding/native acknowledgment; [historical key plan](/doc/PLAN-pubkey-db.md)
is publication motivation with superseded auto-cleanup, not current CLI instructions.
