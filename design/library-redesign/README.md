---
type: DesignIndex
title: Kitty remote-control library redesign
description: Clean-slate API and crate design with a post-discussion registry-first draft and preserved independent research.
resource: /design/library-redesign/README.md
status: draft
generated:
  by: "model:openai/gpt-6-astra#medium"
  at: 2026-10-14
sources:
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
---

# Library redesign

**Find the best API and architecture. All current interfaces are unstable, and
no consumer imposes a compatibility obligation.** The human favors a substantial
restructure with independently composable concern crates, explicit lifecycle,
and separated discovery/authentication steps.

## Current design

- [Registry-first draft 1](draft1.gpt6a.md): post-discussion synthesis. The human
  agreed to registry-owned accepted work, concurrent connection leases, and thin
  managed coordination with retained outcomes first. Exact kubeless integration
  remains subject to a native workflow comparison. Reconciliation is separately
  usable; no current consumer constrains the new API.

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

The first slice is checkpointed; parent review commissioned a correction pass
for actual reusable connection leases (the initial implementation retired every
socket), cross-process-safe remote async IDs, typed error causes, send-time
encryption timestamps, and shutdown concurrent with driver loss. The checkpoint's
successful tests do not establish those additional properties. The report and
native tickets carry subsequent completion evidence.

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
