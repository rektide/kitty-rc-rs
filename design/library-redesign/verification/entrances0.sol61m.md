---
type: VerificationReport
title: Support-policy decision and independent adoption entrances
description: Evidence-tier kitty policy, user-selected Rust, runnable leaf-import profiles, scoped checks and current live-matrix limitations.
resource: /design/library-redesign/verification/entrances0.sol61m.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5dcd3b6ffeofqbiGNfwr1Cf4"
    title: Commissioned support selection, independent examples, native lifecycle guide and scoped offline/fixture verification
  - resource: /doc/usage/support.md
    title: Selected provisional evidence-tier support boundary
  - resource: /client/README.md
    title: Public concern entrances and runnable lifecycle usage
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Stable pinned handler audit and feature-specific release evidence
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Source-backed finite catalog, actual profile modes and source verification
  - resource: /design/library-redesign/implementation0.sol61m.md
    title: Prior real isolated kitty 0.48.2 evidence, corrected native custody/reuse boundaries
  - resource: /observation/listing/src/lib.rs
    title: Actual sampled receipts, publication and baseline ownership
  - resource: /observation/check/src/lib.rs
    title: Original-result custody, fresh read-only continuation and assessment separation
verification:
  scope: Representative example compilation, existing doctests, argument boundaries, pure encoding, private scripted native control flow and dependency direction
  evidence: Selected-toolchain locked example check, strict example clippy, two existing command doctests, five help profiles, four missing-protection refusals, pure plaintext/encrypted specimens and four private loopback journeys
  limits: No new real kitty/version conformance, no encrypted-server fixture claim, no full live multiversion matrix or API stability promise
---

# Outcome

Work anchors: `kitty-rc-proto-rs-redesign-verification-support` and
`kitty-rc-proto-rs-redesign-verification-examples`.

The [support policy](../../../doc/usage/support.md) selects honest **provisional
evidence tiers**, closing the policy decision without inventing a tested oldest
kitty version. The 0.43.1 wire coordinate is retained; the v0.49.2 peeled source
commit `b2cb871254fbdac19a085188813c8dfb258b95cd` remains the stable catalog audit;
the previous implementation report's isolated installed **0.48.2** journeys are
actual live evidence only for their recorded profiles. Earliest-known 0.48 text
extent and 0.49 screenshot/OS-title gates remain profile-specific caller policy,
not automatic runtime negotiation. Inspected 0.50dev HEAD is provisional, not a
default contract. No all-catalog 0.43.1 guarantee or local API-stability claim.

Rust follows the user's current mise selection. No repository toolchain pin,
maintained fixed minimum, `rust-version` change or `cargo +version` override was
introduced. Historical 1.88 reports stay history, not a current requirement.
`mise exec -- rustc --version` actually reported `rustc 1.101.0-nightly`
(`822182db4`, 2026-10-13) in this session.

Owned checkpoints: `f63938bf` support policy, `26e52630` leaf-import examples and
narrow client dev-dependencies. Shared engine/CLI/upload/tracker/lock changes
were not committed by this agent.

## Runnable entrances

The [client guide](../../../client/README.md) is the maintained adoption entrance.
These examples are hosted in `client` but import **leaf crates directly**:

| Example | Actual behavior |
|---|---|
| `pure.rs` | Offline schema serialization, seven-byte incremental DCS framing/decode; optional key/password/timestamp with explicit randomness through crypto |
| `direct.rs` | Explicit access selection, two serial Session listing reads, actual local transport tokens; optional direct future-drop deadline and retained local evidence |
| `managed.rs` | Two accepted concurrent reads, pre-known reservations, detachable waiter timeout, optional separate operation cancel, sole typed take/release, idle-pool inventory and independent follow-up read, drain/driver stop |
| `setup.rs` | Explicit discovery/credential selection, pending route ticket, install, deliberate invalidation/reacquisition/new-generation install; no socket connection |
| `snapshots.rs` | Two scoped listing samples, differences and explicit baseline acceptance; optional explicit-ID focus mutation with original-custody/fresh read-only check branch on uncertain result |

`examples/support/mod.rs` shares clap explicit-route/protection arguments and
selected credential adapters. Plaintext is deliberate, never fallback. Real
encrypted profiles use selected bounded one-line files, not command-line
passwords or ambient stores. The offline pure specimen warns against production
secrets in command arguments. All five profiles compile/help without files,
config, secrets or a live server. The pre-existing `conformance.rs` and
`scripts/verify-isolated-kitty.py` are untouched and remain upload/live-owner work.

The guide explains actual pooling and partitions, caller-known occurrence versus
waiter identity, volatile custody, no total remote ordering across sockets, no
replay after unknown outcomes, explicit native result release, setup rejection,
local generation versus server identity, stable targets versus active scopes,
sampled differences versus events, and satisfaction versus attribution.
No facade export changes or engine/registry/CLI rewrites were needed.

## Checks actually performed

Using the user's `mise exec --` selection:

- `cargo check -p kitty-rc --examples --locked`: passed.
- `cargo clippy -p kitty-rc --examples --locked -- -D warnings`: passed.
- `cargo test -p kitty-rc -p kitty-rc-commands -p kitty-rc-schema -p kitty-rc-frame -p kitty-rc-crypto --doc --locked`: passed. **Two existing command doctests** (builder and upload compile-fail) ran; facade/schema/frame/crypto had zero doctests. These are not new README doctests or a full workspace test claim.
- Scoped `rustfmt --edition 2024` on only the five owned examples and shared argument module: applied; scoped check passed.
- Executed all five `--help` entrances successfully; four endpoint profiles rejected missing protection with clap exit 2 **before any I/O**.
- Ran pure plain specimen and checked exact `{"cmd":"ls","version":[0,43,1]}` output. Ran encrypted offline specimen with explicit X25519 basepoint key, fixture password, epoch timestamp 1 and OS RNG; checked the protected envelope field set and absence of plaintext top-level command/password. This is **not** a live-server encryption assertion.
- Executed direct cancellation/read, managed cancellation/concurrency/follow-up pool, setup install/invalidate and snapshot/uncertain-focus/check journeys against one private loopback scripted native-envelope fixture. The fixture deliberately performs one focus effect and loses its reply; the example retains `Interrupted` evidence and obtains `Satisfied` with `NotEstablished` attribution from a fresh read, with **exactly one** focus request. Scratch recipe: `.test-agent/entrances/verify.py` with its explanatory README; deliberately ignored scratch, not a shipped protocol test or a real kitty.
- Inspected `cargo metadata --no-deps --locked` workspace normal-dependency graph: acyclic; no leaf depends on `kitty-rc`. Verified all five representative profiles import leaf crates rather than `kitty_rc::`.
- Verified all relative Markdown link file targets in this report, the client guide and support policy exist.

No ambient-target writes, new live kitty suite, full workspace rerun, subagents,
new tickets, JSONL export, push, squash or `jj edit`. Root manifest did not need
edits; narrow example-only dependencies live in `client/Cargo.toml`. Cargo.lock
contains those dev-dependency edges mixed with other agents' work and is left
for the parent's shared-file integration.

## Remaining boundaries and parent integration

The policy ticket is closed; the examples ticket can close for these verified
profiles and guide. `verification-live` remains open for the full supported-version
matrix after upload completion; one installed live version plus this synthetic
fixture cannot satisfy it. Unknown earliest capability releases stay unknown.
Upload-specific completion/lane custody is linked as a separate obligation;
these finite examples neither implement it nor imply remote cleanup.

Parent-owned root README should link [client usage](../../../client/README.md)
as the adoption entrance and [support policy](../../../doc/usage/support.md) for
wire/capability/tested-server distinctions. Parent-owned redesign index should
link this report as scoped verification evidence, not a live-matrix completion.

## Cross-references

- [Support policy](../../../doc/usage/support.md): decision and feature selection boundary.
- [Client guide](../../../client/README.md) ⇄ this report: runnable entrances and actual check scope.
- [Capability inventory](../capabilities/inventory0.sol61m.md): source facts, earliest-known releases and nested contracts.
- [Observation implementation](../observation/implementation0.sol61m.md): prior listing implementation evidence and concern ownership.
- [Core implementation](../implementation0.sol61m.md): reported actual isolated live journeys, not rerun here.
