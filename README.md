---
type: ProjectOverview
title: Kitty remote-control typed execution and managed registry
description: Independently usable protocol, explicit access, finite execution and bounded concurrent operation custody.
resource: /README.md
status: active
generated:
  by: "model:openai/gpt-6-astra#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec71e81e3ffercjuzWSBbKFI5w"
    title: Preserved teardown foundation and preceding overview
  - resource: "opencode://ses_ec7004672ffeAbyzuSmd4Cov2i"
    title: Typed execution and bounded registry, then parent-review corrections for pooling, fresh first-send protection, remote IDs, error causes and lifecycle races
  - resource: /design/library-redesign/implementation0.sol61m.md
    title: Implementation decisions, actual verification and remaining limits
  - resource: "opencode://ses_ec7cd6e45ffeqAP7k1wV3STT72"
    title: Human direction to use the user's mise-selected Rust without repository version pinning or a maintained minimum-version commitment
---

# kitty-rc (unstable first slice)

> Rust crates for the kitty terminal emulator's remote-control (RC) protocol,
> rebuilt around typed exchanges and a concurrent registry. Under active
> redesign; the first direct and managed execution slice is usable, not a full catalog.

## Current standing

This repository is in the middle of the redesign described in
[`design/library-redesign/draft1.gpt6a.md`](design/library-redesign/draft1.gpt6a.md)
(registry-first concurrent managed execution, one exchange per connection,
explicit operation ownership). The **legacy 0.5.0 client was removed**
rather than kept as a compatibility facade: its execution path had
double-encrypted sends, implicit credential/subprocess lookup, generic
long-string chunking that corrupted payloads, drifted command wire shapes,
and self-confirming tests. The removal record, disposition map, and
restoration inventory live in
[`design/library-redesign/teardown0.glm53m.md`](design/library-redesign/teardown0.glm53m.md);
the previous implementation is recoverable from jj commit `e78489cb`.

The first slice includes typed `ls`, `get-text`, `focus-window`, deferred
launch-and-wait, and an explicit finite raw entrance; direct sessions and a
concurrent managed registry share one protection boundary. No hidden endpoint,
password, environment, subprocess lookup, automatic retry or pruning occurs.

## Workspace layout

| Crate | Owns |
| --- | --- |
| [`protocol/schema`](protocol/schema) | Wire envelope types: `PlainRequest` (with the upstream `async` wire name), `ProtectedEnvelope`, `ResponseEnvelope` classification (terminal success/failure vs streaming acknowledgment), and the declared `SUPPORTED` kitty version `[0, 43, 1]` |
| [`protocol/frame`](protocol/frame) | Incremental DCS framing (`ESC P @kitty-cmd … ESC \`): split-frame reassembly, in-order coalesced-frame delivery with surplus retention, bounded payloads, poisoning on out-of-frame bytes, clean-EOF truncation reporting |
| [`protocol/commands`](protocol/commands) | Associated-output commands and tolerant listing models (`WindowInfo`, `TabInfo`, `OsWindow`); layout-specific options remain JSON maps |
| [`access/crypto`](access/crypto) | The one outbound protection boundary: exact `1:<base85>` X25519 key parsing, X25519 → SHA-256 → AES-256-GCM sealing with RFC 1924 base85, explicit inputs only (no env/file/subprocess/clock), plus a decrypt oracle for conformance fixtures |
| [`access/key-store`](access/key-store) | The persisted pubkey record format (TSV): parsing with malformed-line reporting, rendering, newest-per-PID selection |
| [`apps/cli`](apps/cli) | The `kitty-pubkey-db` binary (`init`/`add`/`get`/`cleanup`) over the shared store library |
| [`access/endpoint`](access/endpoint) | Explicit Unix/TCP addresses and plain or key/password protection; secret-redacted setup diagnostics |
| [`execution/exchange`](execution/exchange) | Immutable bounded plaintext, reserved protection expansion, fresh first-send encryption and structured causes/progress |
| [`execution/tokio`](execution/tokio) | Reusable single-owner sessions, persistent framing state and absolute deadlines; interrupted/ambiguous/submission-only transports retire |
| [`control/registry`](control/registry) | Caller-known reserve/commit IDs, bounded fair endpoint/class selection, native result custody, cancel, driver replacement and shutdown |
| [`client`](client) | Direct/managed re-exports and public examples |

Discovery, credential-source resolution, uploads, continuing observations/goals,
the full command catalog, and CLI command adoption remain later work. The
registry reuses immutable setup **and clean audited reply-bearing sessions**.
Short, observation and long-wait active/connecting capacity is strictly
partitioned; compatible idle sessions can serve any class. Idle, active and
connecting sockets all count against global/per-endpoint connection limits.
Unused incompatible idle routes can be retired on demand to serve eligible
work at another endpoint; there is no TTL pruning. Input/protection reservations,
retained native output bytes and record counts also have explicit limits.

## Explicit managed usage

```sh
cargo run -p kitty-rc --example managed -- /explicit/kitty.sock
```

The [managed example](client/examples/managed.rs) shows `register`, `start`,
`reserve`, synchronous `submit`, `wait`, consuming typed `take`, explicit
`release`, and `shutdown`. `Registry` supports concurrent `&self` callers;
dropping its waiter/handle detaches, while `cancel(token)` acts on the operation.
Use the reservation's known token to recover native evidence after handle loss.
A committed reservation cannot replay even after explicit record release.

`Session::execute(&mut self, ...)` has the opposite drop contract: dropping its
future drops its transport. After writing begins, interruption means unknown
remote completion, not rollback. Dependent actions must be explicitly awaited;
cross-connection submission order is not effect ordering. Typed results need no
`Clone`: registry custody stores bounded native bytes, and the sole taker owns
decoded output and its memory. `release` removes a completed record deliberately.
Custody is in-process only. Keep the explicit `Driver` alive; use `stop().await`
before replacement. No driver replacement replays a sent input.

Prefer `Session::execute_with_options(command, SessionOptions { ... })` for the
direct entrance. Only audited listing/text/focus/deferred-launch profiles opt into
reuse; raw/custom profiles default to retirement. An observed stale idle peer
returns a pre-write failure, not an automatic reconnect/replay. Each accepted
input is protected once at first send, so queue time does not age an already
encrypted password envelope. `RefusalReason`, `PrepareError`, `ExecutionError`
and `DecodeError` preserve matchable stages and original causes; native reply
bytes remain available. Drain interrupted by driver loss returns a typed
`ShutdownError` containing retained inventory instead of hanging.

Remote async IDs use independent CSPRNG randomness, never local operation or
connection tokens. Kitty's server-global table retains only **32 active async
registrations**, evicting older entries beyond that limit. Other clients and
unresolved remote work can consume it; local quotas cannot prevent their eviction.

## Building and testing

```sh
cargo build            # build all crates and the kitty-pubkey-db binary
cargo nextest run      # test suite (or: cargo test)
cargo clippy --workspace --all-targets
cargo fmt --check
python3 scripts/verify-isolated-kitty.py  # optional: installed kitty + headless Weston
```

Use the user's current Rust toolchain, selected through mise in this workspace.
The repository does not pin a toolchain or maintain a fixed Rust minimum-version
commitment. If your shell has not activated mise, prefix commands with
`mise exec --`, including `mise exec -- python3 scripts/verify-isolated-kitty.py`
so its Cargo subprocess uses the same environment. `mise current rust` and
`mise exec -- rustc --version` show the actual selection. Earlier reports retain
their historical Rust 1.88 verification results.

Scripted tests verify concurrency and lifecycle;
an isolated real **kitty 0.48.2** passed repeated encrypted direct/managed commands
on unchanged connection tokens and concurrent launch-wait/text/focus. The wire version remains `[0,43,1]`; this is not a tested
kitty-minimum or full support matrix. See the [implementation report](design/library-redesign/implementation0.sol61m.md).

The `kitty-pubkey-db` store stays at `$XDG_STATE_HOME/kitty/pubkey.tsv`
with its existing line format (`pid\twindow_id\tpubkey\ttimestamp`);
existing files remain readable. The legacy automatic cleanup-on-add
behavior was removed — run `kitty-pubkey-db cleanup` explicitly. `scripts/enable-rc.sh`
remains for deliberately configuring a local kitty for future live
interoperability runs.

## Design and research

- [`design/library-redesign/`](design/library-redesign/README.md) — redesign
  home: accepted registry-first draft, preserved research, and the teardown
  handoff for the next implementation stage.
- [`design/upstream-kitty/evolution0.gpt6a.md`](design/upstream-kitty/evolution0.gpt6a.md) —
  revision-pinned upstream evolution, defect inventory, and support-policy
  context.
- [`doc/`](doc/) — historical plans and bug evidence kept as research
  material; not current API documentation.

## License

[MIT](LICENSE)
