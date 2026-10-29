---
type: ProjectOverview
title: Kitty remote-control typed execution and managed registry
description: Typed command constructors and optional bon builders, independently usable protocol, explicit access and bounded concurrent execution.
resource: /README.md
status: active
generated:
  by: "model:openai/gpt-6-astra#xhigh"
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Directed and reviewed Sol xhigh finite input and nested literal action conversions; integrated public usage, coverage navigation and API-version markers
  - resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
    title: Finite literal text and raw bytes, public field/builder migration and pinned-source evidence
  - resource: /design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md
    title: Explicit finite broadcast start/data/end messages with caller-owned lifecycle
  - resource: /design/library-redesign/capabilities/nested-text0.sol61xh.md
    title: Literal nested paste/send_text constructors, typed keyboard modes and native parser checks
  - resource: "opencode://ses_e80eba069ffeSSaD033jmggO0o"
    title: Commissioned two f agents for typed command options and bon builders; integrated callers, verified contracts and rewrote examples around direct constructors
  - resource: /protocol/commands/src/targets.rs
    title: Typed match selectors, IDs, keys, modifiers and signals
  - resource: /protocol/commands/src/lib.rs
    title: Typed command constructors and optional bon builders
  - resource: "opencode://ses_ec71e81e3ffercjuzWSBbKFI5w"
    title: Preserved teardown foundation and preceding overview
  - resource: "opencode://ses_ec7004672ffeAbyzuSmd4Cov2i"
    title: Typed execution and bounded registry, then parent-review corrections for pooling, fresh first-send protection, remote IDs, error causes and lifecycle races
  - resource: /design/library-redesign/implementation0.sol61m.md
    title: Implementation decisions, actual verification and remaining limits
  - resource: "opencode://ses_ec7cd6e45ffeqAP7k1wV3STT72"
    title: Parent-led integration of catalog, access, observation and public usage; human direction to use mise-selected Rust without repository pinning
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Audited finite and nested command catalog
  - resource: /design/library-redesign/implementation/access0.sol61xh.md
    title: Explicit discovery, credentials and generation-safe setup
  - resource: /client/README.md
    title: Runnable independent public entrances and lifecycle guide
  - resource: /doc/usage/support.md
    title: Provisional source and live evidence policy
  - resource: /design/library-redesign/cli/implementation0.sol61m.md
    title: Completed CLI and shared-store adoption, including watch absence and response-budget corrections
  - resource: /design/library-redesign/uploads/implementation0.sol61xh.md
    title: Bounded image/logo/run conversations, exact managed lanes and verified upload evidence
  - resource: /design/library-redesign/verification/rust-tooling0.glm53m.md
    title: Rust verification replacement, converged workspace checks and version/mode-specific live evidence
---

# kitty-rc (unstable redesign)

> Rust crates for the kitty terminal emulator's remote-control (RC) protocol,
> rebuilt around typed exchanges and a concurrent registry. Under active
> redesign, with independently usable protocol, access, execution and observation crates.

## Typed command construction

Commands expose typed options, ordinary public structs and associated constructors.
`bon` builders are an optional convenience for named arguments; they use the same
types. For example:

```rust
use kitty_rc::commands::{GetText, TextExtent};
use kitty_rc::commands::process::{Launch, LaunchCwd, LaunchType};
use kitty_rc::commands::targets::WindowId;

let screen = GetText::builder()
    .match_spec(WindowId(42))
    .extent(TextExtent::Screen)
    .ansi(true)
    .build();

let terminal = Launch::builder()
    .kind(LaunchType::Tab)
    .cwd(LaunchCwd::Current)
    .args(vec!["bash".into()])
    .build();
```

For everyday commands, use the constructors directly:

```rust
use kitty_rc::commands::{FocusWindow, GetText, Ls, TextExtent};
use kitty_rc::commands::actions::{Action, SimpleAction};
use kitty_rc::commands::tab::TabCommand;
use kitty_rc::commands::targets::{Key, KeyChord, Layout, TabId, WindowId, WindowSet};
use kitty_rc::commands::window::{ScrollAmount, SendKey, WindowCommand};

let window = WindowId(42);
let listing = Ls::all();
let tab_listing = Ls::tab(TabId(3));
let focus = FocusWindow::id(window);
let text = GetText::window(window, TextExtent::Selection);
let title = WindowCommand::title(window, "Editor");
let scroll = WindowCommand::scroll(window, ScrollAmount::End);
let layout = TabCommand::layout(TabId(3), Layout::Stack);
let next_tab = Action::new(SimpleAction::NextTab);

// Set additional options directly with ordinary struct-update syntax.
let keys = SendKey {
    windows: WindowSet::window(window),
    ..SendKey::new([KeyChord::ctrl('a'), KeyChord::new(Key::Enter)])
};
```

Appearance, configuration and process commands follow the same approach:

```rust
use kitty_rc::commands::appearance::{Appearance, ColorName, ColorValue, FontChange, SetFontSize};
use kitty_rc::commands::configuration::{ConfigOverride, LoadConfig};
use kitty_rc::commands::process::{Launch, LaunchCwd, LaunchType};
use kitty_rc::commands::targets::{Rgb, WindowId, WindowSet};

let font = SetFontSize::new(FontChange::increase(1.0).unwrap());
let foreground = Rgb::new(0x88c0d0).unwrap();
let colors = Appearance::colors(
    WindowSet::window(WindowId(42)),
    [(ColorName::Foreground, ColorValue::Rgb(foreground))],
);
let config = LoadConfig {
    overrides: vec![ConfigOverride::foreground(foreground)],
    ..LoadConfig::default()
};
let terminal = Launch {
    kind: LaunchType::Tab,
    cwd: Some(LaunchCwd::Current),
    ..Launch::new(["bash"])
};
```

Constructing a command performs no I/O. Pass it to `Session::execute_with_options`
or `Registry::submit`; upload commands use the dedicated upload entrances.
The same types are available directly from the `kitty-rc-commands` crate.

Finite text input distinguishes literal Unicode from arbitrary bytes:

```rust
use kitty_rc::commands::window::SendText;

let literal = SendText::new(r"literal \n"); // backslash and n stay literal
let binary = SendText::bytes([0, 0xff]);   // no UTF-8 requirement
let named = SendText::builder().data("hello\n").build();
```

Each is one bounded submission. Completion reports local outbound progress;
bracketed-paste options can wrap/sanitize input on the server. The public
`text` field and builder `.text(...)` have become `data` and `.data(...)`.
See [text and byte input](client/README.md#finite-text-and-byte-input) for scope,
migration and evidence.

`BroadcastId`, `BroadcastControl::start/end` and `SendText.broadcast` expose
explicit finite broadcast messages with caller-owned lifecycle. See the
[broadcast guide](client/README.md#explicit-broadcast-messages) for construction,
membership and submission-order boundaries.

Nested text actions now accept literal strings too:

```rust
use kitty_rc::commands::actions::{Action, KnownAction, SendTextModes};

let paste = Action::new(KnownAction::paste(r" literal \n ").unwrap());
let send = Action::new(KnownAction::send_text(SendTextModes::All, "hello\n").unwrap());
```

These constructors reject empty text and encode valid input for one native
decoding pass. See [nested text actions](client/README.md#literal-nested-text-actions)
for keyboard modes, targeting and paste/dispatch semantics.

`WindowId` and `TabId` keep identities distinct. `WindowMatch` and `TabMatch`
provide typed field constructors; their title/cwd/cmdline patterns are native
regular expressions with query boundaries preserved. Complete custom expressions
use explicit `raw` constructors. Titles, text, paths and argv remain string data;
`ActionSpec::Raw`, `KeyChord::raw` and `RawFinite` expose deliberate grammar
extensions. See the [client guide](client/README.md#constructing-commands) for
target selection and [command modules](protocol/commands/src) for the full surface.

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

The source-audited finite catalog includes window/tab/layout/style/process/config
commands, deferred results, nested mappable actions and deliberate raw entrances.
Direct sessions and a concurrent managed registry share one protection boundary.
Discovery and credential sources are explicitly selected; generation-safe setup
changes preserve old operation evidence. Scoped snapshots and check-only
follow-up compose above the registry without replaying uncertain mutations.

Start with the [usage guide and runnable examples](client/README.md), and read the
[support policy](doc/usage/support.md) to distinguish audited command contracts
from the exact server versions and profiles exercised live.

## Workspace layout

| Crate | Owns |
| --- | --- |
| [`protocol/schema`](protocol/schema) | Wire envelope types: `PlainRequest` (with the upstream `async` wire name), `ProtectedEnvelope`, `ResponseEnvelope` classification (terminal success/failure vs streaming acknowledgment), and the declared `SUPPORTED` kitty version `[0, 43, 1]` |
| [`protocol/frame`](protocol/frame) | Incremental DCS framing (`ESC P @kitty-cmd … ESC \`): split-frame reassembly, in-order coalesced-frame delivery with surplus retention, bounded payloads, poisoning on out-of-frame bytes, clean-EOF truncation reporting |
| [`protocol/commands`](protocol/commands) | Grouped associated-output command families and nested actions; required structural listing data and tolerant additive fields |
| [`access/crypto`](access/crypto) | The one outbound protection boundary: exact `1:<base85>` X25519 key parsing, X25519 → SHA-256 → AES-256-GCM sealing with RFC 1924 base85, explicit inputs only (no env/file/subprocess/clock), plus a decrypt oracle for conformance fixtures |
| [`access/key-store`](access/key-store) | The persisted pubkey record format (TSV): parsing with malformed-line reporting, rendering, newest-per-PID selection |
| [`apps/cli`](apps/cli) | `kitty-rc` listing, active-window, goto and sampled watch with explicit setup; `kitty-pubkey-db` over the shared store library |
| [`access/endpoint`](access/endpoint) | Explicit Unix/TCP addresses and plain or key/password protection; secret-redacted setup diagnostics |
| [`access/discovery`](access/discovery) | Injectable candidate acquisition, native paths, provenance and explicit ambiguity policy |
| [`access/credentials`](access/credentials) | Endpoint-scoped key/password sources, pure validation and explicit resolution policy |
| [`execution/exchange`](execution/exchange) | Immutable bounded plaintext, reserved protection expansion, fresh first-send encryption and structured causes/progress |
| [`execution/tokio`](execution/tokio) | Reusable single-owner sessions, persistent framing state and absolute deadlines; interrupted/ambiguous/submission-only transports retire |
| [`control/registry`](control/registry) | Caller-known reserve/commit IDs, bounded fair endpoint/class selection, native result custody, cancel, driver replacement and shutdown |
| [`observation/listing`](observation/listing) | Scoped finite snapshots, detachable receipts, sampled changes and caller-driven polling |
| [`observation/check`](observation/check) | Explicit fresh listing checks after uncertainty, with matching/satisfaction separate from original native outcomes |
| [`client`](client) | Direct/managed re-exports and public examples |
| [`tool/verification`](tool/verification) | Rust-owned private server fixtures, live matrix, release acquisition, CLI smoke and source/probe checks |

Bounded PNG background/logo uploads and buffered asynchronous `run` are implemented.
The facade exposes `UploadSource`, `SessionUploadOptions` and
`RegistryUploadOptions`; command profiles live in `commands::upload`.
Managed uploads use explicit alias groups and same-command lanes. Interrupted or
suppressed exchanges retain their lane until an explicit caller supersession
decision; ordinary unrelated operations remain eligible. See the
[upload contract and evidence](design/library-redesign/uploads/implementation0.sol61xh.md).

Continuous desired-state goals remain separately selectable. The registry reuses
resolved setup **and clean audited reply-bearing sessions**. Setup invalidation
rejects queued stale preparation with recoverable input; already leased work
keeps its accepted basis and cannot populate a newer setup pool.
Short, observation and long-wait active/connecting capacity is strictly
partitioned; compatible idle sessions can serve any class. Idle, active and
connecting sockets all count against global/per-endpoint connection limits.
Unused incompatible idle routes can be retired on demand to serve eligible
work at another endpoint; there is no TTL pruning. Input/protection reservations,
retained native output bytes and record counts also have explicit limits.

## Command-line usage

```sh
mise exec -- cargo run -p kitty-rc-cli --bin kitty-rc -- --help
kitty-rc --to unix:/chosen/kitty.sock ls --json
kitty-rc --to unix:/chosen/kitty.sock active
kitty-rc --to unix:/chosen/kitty.sock goto 42
kitty-rc --to unix:/chosen/kitty.sock watch --count 10 --json
```

`ls` can use the selected `KITTY_LISTEN_ON` when no explicit route is supplied.
Encrypted mode, declared configuration, source precedence and credential binding
are documented in the [CLI guide](apps/cli/README.md). Watch uses finite polling,
reports normal no-focus samples without terminating, and does not claim a server
event subscription. The [key-store utility](apps/cli/README.md#shared-public-key-utility)
preserves existing records and keeps cleanup explicit.

## Explicit managed usage

```sh
mise exec -- cargo run -p kitty-rc --example managed -- --endpoint unix:/explicit/kitty.sock --plain
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
direct entrance. Audited reply-bearing profiles opt into reuse; raw/custom
profiles default to retirement. An observed stale idle peer
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
cargo build --workspace
cargo nextest run      # test suite (or: cargo test)
cargo clippy --workspace --all-targets
cargo fmt --all -- --config max_width=100 --check
mise exec -- cargo run -p kitty-rc-verify -- finite
mise exec -- cargo run -p kitty-rc-verify -- uploads
mise exec -- cargo run -p kitty-rc-verify -- matrix --cli-smoke
```

Use the user's current Rust toolchain, selected through mise in this workspace.
The repository does not pin a toolchain or maintain a fixed Rust minimum-version
commitment. If your shell has not activated mise, prefix commands with
`mise exec --` so verification subprocesses use the same environment. `mise current rust` and
`mise exec -- rustc --version` show the actual selection. Earlier reports retain
their historical Rust 1.88 verification results.

Scripted tests verify concurrency and lifecycle. The Rust-runner matrix exercised
private kitty **0.43.1, 0.48.2 and 0.49.2**: selected finite/nested commands and CLI
smoke in encrypted and plain modes, plus **encrypted uploads** on all three.
Version-sensitive cases are explicitly gated. This is not a whole-catalog,
oldest-supported-version or plain-upload claim. The wire coordinate remains
`[0,43,1]`; see [support evidence](doc/usage/support.md) and the
[verification guide](tool/verification/README.md).

Verification orchestration and fixture children are Rust. Small Python fixtures
serve the native kitty kitten and upstream semantic-oracle boundaries, including
an opt-in full-parser check for Rust-generated nested text actions. Its command
and evidence are in the [nested-text report](design/library-redesign/capabilities/nested-text0.sol61xh.md).

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
