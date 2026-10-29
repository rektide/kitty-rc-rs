---
type: DesignIndex
title: Kitty remote-control library redesign
description: Clean-slate API and crate design with a post-discussion registry-first draft and preserved independent research.
resource: /design/library-redesign/README.md
status: draft
generated:
  by: "model:openai/gpt-6-astra#xhigh"
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Human commissions send-text implementation, then redirects toward ordinary unfinished conversions; Astra reviews built slices and restores current coverage navigation
  - resource: /design/library-redesign/capabilities/sendtext-boundaries0.gpt6a.md
    title: Native escape/paste corrections and core-first implementation direction
  - resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
    title: Built finite text/raw-byte core, migration and independent handler/preparation checks
  - resource: /design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md
    title: Built finite broadcast messages, native lifecycle evidence and 248-test workspace convergence
  - resource: /design/library-redesign/capabilities/sendtext-followthrough0.gpt6a.md
    title: Post-build epic and ticket scope proposed for human discussion before creation
  - resource: /design/library-redesign/capabilities/conversion-coverage0.gpt6a.md
    title: Native action inventory, representative conversion gaps and listing/embedded-option catch-up after human reprioritization
  - resource: "opencode://ses_ec7cd6e45ffeqAP7k1wV3STT72"
    title: Human redesign direction, research review and corrections, accepted managed-operation direction, and post-discussion draft
  - resource: /design/library-redesign/draft0.gpt6a.md
    title: Independent parent proposal before research integration
  - resource: /design/upstream-kitty/evolution0.gpt6a.md
    title: Completed upstream study, baseline and summary inspected for navigation
  - resource: /design/library-redesign/kubeless-assessment0.glm53fx.md
    title: Initial kubeless opportunity inventory, qualified by the subsequent Astra study
  - resource: /design/library-redesign/kubeless-design0.gpt6a.md
    title: Verified kubeless mechanisms and kitty-native exchange design, read for discussion
  - resource: /design/library-redesign/tmax-lifecycle0.glm53m.md
    title: Lifecycle comparison and reviewed corrections to correlation and cancellation recommendations
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md
    title: Human-corrected registry-first concurrency design and constructive generic kubeless adaptations
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Post-discussion registry-first synthesis and remaining design choices
  - resource: /design/library-redesign/implementation0.sol61m.md
    title: First implemented concurrent slice, verified evidence and parent-reviewed lifecycle gaps
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Source-audited finite catalog and nested Action implementation
  - resource: /design/library-redesign/implementation/access0.sol61xh.md
    title: Explicit acquisition and generation-safe setup implementation, shared-store CLI gap
  - resource: /design/library-redesign/observation/implementation0.sol61m.md
    title: Scoped snapshot, sampled change and detached receipt implementation
  - resource: /design/library-redesign/observation/check0.sol61xh.md
    title: Finite uncertain-effect follow-up implementation and native setup evidence
  - resource: /design/library-redesign/verification/entrances0.sol61m.md
    title: Support-policy and runnable independent entrance verification
  - resource: /design/library-redesign/cli/implementation0.sol61m.md
    title: Completed CLI/store adoption and verified no-focus watch and maximum-response corrections
  - resource: /design/library-redesign/uploads/implementation0.sol61xh.md
    title: Completed image/logo/run exchanges and exact alias-aware managed lanes
  - resource: /design/library-redesign/verification/rust-tooling0.glm53m.md
    title: Rust tooling conversion and selected three-release live matrix
---

# Library redesign

**Find the best API and architecture. All current interfaces are unstable, and
no consumer imposes a compatibility obligation.** The human favors a substantial
restructure with independently composable concern crates, explicit lifecycle,
and separated discovery/authentication steps.

## Current design

The selected redesign and linked CLI work are implemented; all native work items
in this roadmap are closed. Final integrated gates passed **224 workspace tests**,
locked all-target build, strict Clippy and formatting, with documentation tests
also passing during the converged verification. Live evidence remains scoped to
the exact version/profile matrix below. This is implementation completion, not
API stability or acceptance of every future possibility in the research.

- [Registry-first draft 1](draft1.gpt6a.md): post-discussion synthesis. The human
  agreed to registry-owned accepted work, concurrent connection leases, and thin
  managed coordination with retained outcomes first. Exact kubeless integration
  remains subject to a native workflow comparison. Reconciliation is separately
  usable; no current consumer constrains the new API.

## Remaining conversion work

The human selected literal nested `paste`/`send_text` next and redirected
attention toward unfinished ordinary conversions. CLI expansion and bounded
streaming are not current priorities. The
[coverage review](capabilities/conversion-coverage0.gpt6a.md) distinguishes the
outer RC catalog from the much larger nested-action surface, and identifies
clipboard, key, window/tab, output-model and embedded-option candidates.

## Send-text follow-on

After the first public release, the human selected the deferred send-text
surface for further work. These reports preserve the two completed finite
slices and the native-contract corrections that informed them.

- [Slice-A sketch](capabilities/sendtext0.glm53m.md): the initial finite-data,
  escape, broadcast and paste proposal, with B/C conversation and CLI directions.
- [Native boundary review](capabilities/sendtext-boundaries0.gpt6a.md): corrects
  the sketch's double-expanded paste, ordinary-octal and API-compatibility claims
  using native kitty 0.43.1/0.49.2 evidence; prioritizes literal text/bytes and
  explicit broadcast messages before choosing escape and nested-action contracts.
- [Finite text/raw-byte implementation](capabilities/sendtext-core0.sol61xh.md):
  `SendTextData`, literal/byte constructors and builders, honest submission-only
  outcomes, public migration, and 11 native handler cases at each named revision.
  [Usage](../../client/README.md#finite-text-and-byte-input) shows the public API.
- [Explicit broadcast implementation](capabilities/sendtext-broadcast0.sol61xh.md):
  validated identity, optional ID-bearing data and required-ID start/end controls;
  three fresh native lifecycle scenarios at each revision and **248 workspace
  tests**, eight doctests, strict Clippy and formatting passed. The parent reviewed
  the types, preparation checks and native oracle boundary. The
  [broadcast guide](../../client/README.md#explicit-broadcast-messages) explains
  caller-owned lifecycle and the absence of remote receipt/order evidence.
- [Post-build ticket discussion](capabilities/sendtext-followthrough0.gpt6a.md):
  preserved earlier proposal. The human subsequently dropped tracking discussion
  and deprioritized its B/C sequence; current direction is the conversion work
  above. No native work items were created by this implementation session.

## Preserved initial position

- [Initial architecture draft](draft0.gpt6a.md): typed commands/responses, a
  concern-crate workspace, explicit setup and exchange lifecycles, cancellation
  and outcome semantics, command-specific streaming, and verification strategy.
- [Earlier registry plan](../../doc/PLAN-registry.md): historical motivation;
  pooling and retries need reconsideration under the new lifecycle model.
- [Earlier public-key store plan](../../doc/PLAN-pubkey-db.md): historical
  standalone authentication context and a candidate integration to redesign.

## Implementation and work anchors

- [Applied epics and ticket roadmap](tickets0.sol61m.md): native work IDs,
  dependencies, acceptance criteria, and tracker migration evidence.
- [G's teardown and foundation handoff](teardown0.glm53m.md): removed legacy paths,
  preserved capability inventory, pure concern crates and recovery coordinate.
- [Sol's first concurrent slice](implementation0.sol61m.md): typed direct/managed
  execution, detached waiters, bounded native outcome custody, scripted concurrency
  tests and isolated real-kitty 0.48.2 encrypted conformance.
- [Finite catalog and nested actions](capabilities/implementation0.sol61m.md):
  inventory of 41 stable command names, 39 fully or partially typed finite
  entrances, grouped action grammar, and independently executed upstream fixtures.
  Run and image/logo input still require the upload conversation.
- [Explicit access and setup generations](implementation/access0.sol61xh.md):
  discovery/credential source policies, preserved shared-store I/O, recoverable
  rejected queued preparation, and exact old/new pool setup accounting.
- [Scoped listing observations](observation/implementation0.sol61m.md): finite
  acquisition receipts, caller-owned baseline acceptance, sampled differences,
  bounded projection and caller-driven polling. [Library guide](../../observation/listing/README.md).
- [Check-only follow-up](observation/check0.sol61xh.md): fresh listing assessment
  after an unresolved native effect, with matching and satisfaction separate from
  causal attribution. Its ticket closed after the typed/upload dependency completed.
- [Public entrances and support evidence](verification/entrances0.sol61m.md):
  [usage guide](../../client/README.md), compiled leaf-import examples and
  [provisional support policy](../../doc/usage/support.md).
- [CLI and shared-store adoption](cli/implementation0.sol61m.md): list/active/goto/
  sampled watch, explicit configuration/authentication, preserved TSV maintenance,
  and 28 targeted executable journeys after parent-review corrections.
  [CLI guide](../../apps/cli/README.md).
- [Command-specific uploads and run](uploads/implementation0.sol61xh.md):
  bounded source consumption, handshake/EOF/terminal modes, binary run results,
  explicit alias groups and exact interruption/supersession lane accounting.
  Core gates passed 185 workspace tests plus private encrypted kitty 0.48.2
  upload conformance. Its original Python runners are preserved in revision history;
  maintained orchestration is now Rust.
- [Rust verification tooling and live matrix](verification/rust-tooling0.glm53m.md):
  [maintained runner guide](../../tool/verification/README.md), Rust fixture children,
  and selected 0.43.1/0.48.2/0.49.2 finite/CLI profiles in both modes, with encrypted
  uploads on all three. Two small fixtures remain Python at native kitten and
  upstream semantic-oracle boundaries. Cold-start fixture-child building, Cargo's
  default binary and no-build entrance corrections are verified in `09975234`.

The first slice and its review-driven corrections are committed. Audited
reply-bearing sessions now reuse sockets; idle/active/connecting resources count
against registry quotas. Remote async IDs use CSPRNG randomness, execution errors
retain typed causes, encryption timestamps are created at first send, and drain
handles concurrent driver loss. Interrupted, ambiguous and submission-only
exchanges retire their transports; raw/custom profiles default to retirement.

The [correction report](implementation0.sol61m.md#review-driven-corrections)
records 82 passing workspace tests, Rust 1.88 all-target/Clippy/format gates,
scripted connection-count/reuse checks, and repeated encrypted direct/managed
reuse against isolated kitty 0.48.2. Parent review inspected the corrected session,
pool selection and error/evidence code. A full supported-version/live matrix,
full upload conversations and CLI adoption remain tracked work. Discovery and
finite catalog breadth are now implemented by the subsequent slices above.

The Rust-runner correction and all implementation handoffs are complete.
User-directed Rust policy follows the user's mise selection without
repository pinning; historical toolchain results remain historical evidence.

## Research contributions

All commissioned contributions are available. The human selected the direction
recorded in draft 1; individual research recommendations remain proposals unless
that draft explicitly incorporates them.

| Inquiry | Agent | Role |
| --- | --- | --- |
| [Upstream kitty evolution](../upstream-kitty/evolution0.gpt6a.md) | Astra medium | Build baseline, stable/HEAD behavior, schemas and server fixes |
| [Kubeless reconnaissance](kubeless-assessment0.glm53fx.md) | FX | Initial lifecycle/streaming opportunity map; read with Astra's corrections |
| [Kubeless research and design](kubeless-design0.gpt6a.md) | Astra medium, after FX | Verified evidence/ownership patterns, reuse decisions and concrete exchange/crate proposal |
| [Kubeless adaptation and registry design](kubeless-adaptation1.gpt6a.md) | Astra medium, after human correction | Constructive control-framework reuse, concurrent pooled execution, retained outcomes and optional reconciliation |
| [Tmax lifecycle comparison](tmax-lifecycle0.glm53m.md) | G | Typed-operation and deadline precedents, protocol-specific machinery, corrected correlation/cancellation guidance |

### Available upstream evidence

- [Kitty evolution from the build baseline](../upstream-kitty/evolution0.gpt6a.md):
  February 2026 library release, declared kitty 0.43.1 target, latest stable 0.49.2,
  and pinned development HEAD. Its opening assessment distinguishes a stable
  core protocol from command growth, server fixes, and local pre-existing bugs.
  This informs draft 1's support policy and verification requirements; specific
  minimum versions remain to be selected.

## Reading the findings together

- **Common direction:** typed request/output binding, explicit outcome evidence,
  absolute deadlines, one exchange owner, and concern-specific components.
- **Kubeless reuse, revised:** the first Astra study evaluated the low-level
  exchange too narrowly. The adaptation follow-on studies kubeless's generic
  ready-work/executor, custody, retained host, checks, observations and capability
  descriptions as serious managed-registry building blocks. Its strongest proposed
  framework extension is a fair native work-lane entrance into the retained host.
  HTTP retry and Kubernetes framing remain separate from kitty's native exchange.
- **Tmax boundary:** borrow its execution discipline and typed preparation
  pattern; its tmux-specific correlation engine and sync-first task structure
  are not a starting template for this Tokio client.
- **New protocol constraints:** Astra's source-backed upload study describes
  handshake versus terminal completion, empty-upload handling, and a shared
  per-command upload accumulator that can cause interference across connections.
  The parent checked the accumulator and Go handshake source; live kitty
  verification is still outstanding.
- **Registry motivation, clarified by the human:** multiple independent commands
  should progress across multiple bounded connection leases. Long waits and large
  reads must not block quick operations globally. One exchange per connection
  is not one exchange per registry; upload exclusion is a specialty constraint.
- **Selected direction and remaining decisions:** registry ownership outlives
  individual waiters, with explicit operation cancellation and thin coordination
  first. Direct custody versus the fuller retained host needs a workflow comparison;
  admission commit details, capacity defaults, result retention, extension
  boundaries and upstream/runtime support remain open. Reconciliation is a
  selectable higher-level capability, not a prerequisite for concurrent commands.

The initial draft remains preserved. Human discussion and agreement preceded
draft 1; it is a design checkpoint, not an implemented or fully validated API.
