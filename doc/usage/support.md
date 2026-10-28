---
type: SupportPolicy
title: Rust selection and kitty capability support
description: Explicit wire baseline, source and live evidence tiers, capability gates and provisional upstream policy.
resource: /doc/usage/support.md
status: provisional
generated:
  by: "model:openai/gpt-6-astra#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_e7b57d91bffeto158h8GVlMvHn"
    title: Distinguished finite literal text/raw bytes and explicit broadcast messages from managed conversations, without expanding live or oldest-release claims
  - resource: "opencode://ses_ec5dcd3b6ffeofqbiGNfwr1Cf4"
    title: Commissioned support boundary separating user Rust, wire baseline, audited features and exercised servers
  - resource: /protocol/schema/src/version.rs
    title: Existing envelope baseline and major/minor comparison
  - resource: /protocol/commands/src/catalog.rs
    title: Revision-pinned capability evidence, not automatic runtime negotiation
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Stable handler inventory and release-specific capability evidence
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Source-backed typed profiles and finite versus upload boundaries
  - resource: /design/library-redesign/implementation0.sol61m.md#actual-verification
    title: Reported isolated installed kitty 0.48.2 evidence and remaining live-matrix boundary
  - resource: "opencode://ses_ec7cd6e45ffeqAP7k1wV3STT72"
    title: Parent review and integration of Rust-runner version/mode evidence without expanding compatibility claims
  - resource: /design/library-redesign/verification/rust-tooling0.glm53m.md
    title: Post-conversion real three-release matrix, encrypted uploads and both-mode finite/CLI evidence
---

# Support boundary

Work anchor: `kitty-rc-proto-rs-redesign-verification-support`. This is the selected
**provisional evidence-based policy**, not a claim of a tested minimum kitty
release. The selected live matrix and runner entrance corrections are verified;
`kitty-rc-proto-rs-redesign-verification-live` is closed for that recorded scope.

## Rust belongs to the user

Use the user's selected Rust, here through `mise exec -- cargo …`. There is no
repository toolchain pin and no maintained fixed Rust minimum. On 2026-10-14 the
selection was `nightly-2026-10-14`, reporting rustc `1.101.0-nightly`
(`822182db4`, 2026-10-13). This is evidence for this work, not a required version.
Earlier Rust 1.88 reports are historical; do not copy their `cargo +version`
commands into current entrances. Dependency portability and persisted credential
formats are separate obligations, not established by compiling once.

## Three different kitty coordinates

| Coordinate | Current evidence | What it establishes |
|---|---|---|
| Request envelope baseline | `SUPPORTED = [0,43,1]` in `kitty-rc-schema` | Existing declared wire coordinate, retained without a global bump. The name is **not** a tested support matrix. |
| Stable source audit | v0.49.2 commit `b2cb871254fbdac19a085188813c8dfb258b95cd` | Inspected handler/payload contracts and source-executed fixtures. Not live conformance or proof of earliest release. |
| Actually exercised servers | Private kitty **0.43.1**, installed **0.48.2**, and **0.49.2** | Rust-runner finite/nested and CLI profiles passed encrypted and plain; encrypted image/logo/run uploads passed all three. Exact recorded profiles/options and version gates define the evidence. |

The dispatcher compares request major/minor against its own version. Accepting
the envelope does not establish that the named command, field, option or result
shape exists. Keeping `[0,43,1]` does **not** promise every catalog entry works on
0.43.1. Nor does auditing 0.49.2 justify changing every request to `[0,49,2]`.
The oldest server in the selected matrix is 0.43.1; that is not a guaranteed
minimum or a whole-catalog live compatibility range.

### Recorded Rust-runner matrix

| Server | Finite/nested profiles | Upload profiles | CLI smoke |
| --- | --- | --- | --- |
| 0.43.1 | Encrypted and plain; 0.48/0.49 features skipped | Encrypted background/logo, suppression handshake, binary empty/small/exact/large run, managed aliases | Both modes |
| 0.48.2 | Encrypted and plain; 0.49 features skipped | Same encrypted upload journeys | Both modes |
| 0.49.2 | Encrypted and plain, including screenshots and OS-window title | Same encrypted upload journeys | Both modes |

Plain uploads remain unclaimed. The [Rust verification report](../../design/library-redesign/verification/rust-tooling0.glm53m.md)
records exact release/source and executable provenance, selected toolchain and
witness path; [the maintained tool](../../tool/verification/README.md) reproduces
the runs. Earlier Python-runner witnesses remain historical evidence, distinct
from the new Rust-runner witness.

## Gate the exact profile

`kitty_rc_commands::catalog::capability(name)` exposes source revision, coverage,
completion and `earliest_documented`. It is evidence metadata, **not a runtime
version probe, capability negotiation or execution guard**. A missing earliest
release means unknown, never “supported on every release.” The caller must
select features using supplied server/version/capability evidence; unsupported
remote errors remain errors, not silent downgraded success.

- Screenshot and OS-window title have earliest documented release **0.49**.
   Do not select those profiles on 0.43.1 or 0.48.2 just because they
  compile or the envelope is accepted.
- Alternate text extents have release evidence from **0.48**. Gate the extent,
  not merely the `get-text` command name. Some text extents also require shell
  integration; selection clearing is an effect, not a universally pure read.
- Other stable catalog entries are observed at the pinned stable revision unless
  their metadata says more. Oldest availability remains unasserted. Coverage
  applies to the exact typed profile: e.g. finite literal text/raw-byte submission is not a
  file/stdin/broadcast conversation; PNG uploads and buffered run have their own
  conversation/exclusion requirements. Read the current catalog and upload
  contracts instead of treating one familiar command name as complete support.
- Finite `SendText` literal text and raw bytes have handler-source evidence at
  stable 0.49.2 and the named inspected HEAD, not a new live-matrix claim.
  Backslashes stay literal; bracketed paste can wrap/sanitize on the server.
  Submission completion is not child delivery, even for binary data. The
  [finite-core report](../../design/library-redesign/capabilities/sendtext-core0.sol61xh.md)
  records the source-oracle scope and the public `text`/builder-to-`data` migration.
- Optional broadcast IDs on data and explicit start/end messages have additional
  source-executed membership/callback evidence at those two revisions, not live
  delivery or cleanup evidence. They retire like ordinary finite submissions.
  Caller-owned IDs are server-global; end can clear accumulated members outside
  its current match/exclusion scope. Awaiting local submissions on separate
  sockets does not prove remote effect ordering, and socket closure is not
  origin-window removal. See the [broadcast report](../../design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md).
- Server configuration, permissions, installed kittens, visible targets, shell
  integration and window-manager cooperation can constrain a version-present
  feature. A successful RC acknowledgment need not prove the later UI/child
  effect or delivered keys. Transport reuse never grants replay safety.

## Unreleased and local APIs

Inspected HEAD `5bf3c2dc676efd5c4b4526eb135c922701565707` is **0.50dev**, not the
default stable contract. HEAD-only `resize-window-edge` and extensions to resize,
detach or launch remain provisional/raw unless a separately selected profile
explicitly says otherwise. Do not infer stable availability from HEAD source.

The concern crates and facade are a redesign in progress below 1.0. This policy
does not promise local Rust API stability, legacy serializer compatibility, or
remote rollback. Results can be unknown after outbound progress; do not replay
automatically. Encrypted requests use kitty protection protocol 1; replies are
not a general encrypted transport. Plain mode requires an explicitly selected
endpoint and server permissions and is never an error fallback from encryption.

## Extending the evidence

For each additional server, record exact installed version/source, isolation,
protection mode, exact profile/options, native output, cancellation/reuse and
failure coverage. A failed capability gate is useful evidence, not permission
to falsify the envelope. Extend the matrix by named profiles and versions rather
than implying universal coverage from a passing subset.

## Cross-references

- [Independent entrances and lifecycle](../../client/README.md): applying these gates without hidden setup or replay.
- [Capability inventory](../../design/library-redesign/capabilities/inventory0.sol61m.md): full source audit and nested completion boundaries.
- [Capability implementation](../../design/library-redesign/capabilities/implementation0.sol61m.md): exact implemented profiles and verification scope at that checkpoint.
- [First implementation evidence](../../design/library-redesign/implementation0.sol61m.md): historical and corrected live evidence; later upload work owns additional journeys.
