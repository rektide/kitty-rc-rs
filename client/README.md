---
type: UsageGuide
title: Independent concern entrances and explicit client lifecycles
description: Typed command constructors and optional bon builders, runnable leaf-crate examples, selected setup and explicit direct/managed lifecycles.
resource: /client/README.md
status: draft
generated:
  by: "model:openai/gpt-6-astra#xhigh"
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Reviewed Sol xhigh finite data, broadcast and nested literal actions; integrated usage, boundaries, migration and API-version guidance
  - resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
    title: Literal text/raw-byte contracts, preparation tests and pinned handler evidence
  - resource: /design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md
    title: Required broadcast identity, optional ID-bearing data, explicit lifecycle messages and source evidence
  - resource: /design/library-redesign/capabilities/nested-text0.sol61xh.md
    title: Literal nested paste/send_text, typed modes and native full-parser verification
  - resource: "opencode://ses_e80eba069ffeSSaD033jmggO0o"
    title: Integrated typed selectors across clients and observation; documented optional bon builders and direct command construction
  - resource: /protocol/commands/src/targets.rs
    title: Typed IDs, native match expressions and command-specific target profiles
  - resource: "opencode://ses_ec5dcd3b6ffeofqbiGNfwr1Cf4"
    title: Support-policy selection and runnable independently imported adoption profiles
  - resource: /doc/usage/support.md
    title: Selected provisional wire, capability and server evidence policy
  - resource: /client/src/lib.rs
    title: Current facade exports; convenience rather than compulsory owner
  - resource: /execution/tokio/src/lib.rs
    title: Session cancellation, local connection ownership and terminal reuse
  - resource: /control/registry/src/lib.rs
    title: Native reserve/admit/wait/take/release, bounded pool and shutdown
  - resource: /control/registry/src/setup.rs
    title: Pending setup tickets and explicit generation invalidation/install
  - resource: /access/discovery/src/lib.rs
    title: Explicit candidates and source acquisition/selection boundaries
  - resource: /access/credentials/src/lib.rs
    title: Selected protection policy, source provenance and no error fallback
  - resource: /observation/listing/src/lib.rs
    title: Finite receipt/snapshot sampling and explicit baseline acceptance
  - resource: /observation/check/src/lib.rs
    title: Separate original-result custody and read-only continuation
---

# Start where the concern lives

`kitty-rc` is an optional convenience facade. Each concern crate is usable
without depending on it. The examples below are hosted here for convenient
execution but **import leaf crates directly**, demonstrating the actual public
entrances. No example needs real credentials, user config or a running kitty to
compile or show `--help`.

Use the user's selected Rust (`mise exec -- cargo …` here), not `cargo +version`
or a repository toolchain pin. Read the [support policy](../doc/usage/support.md)
before selecting a version-sensitive command: the 0.43.1 envelope coordinate is
not a guarantee that the entire source-audited 0.49.2 catalog works on 0.43.1.
Recorded isolated live evidence is on 0.48.2, for specified journeys only.

## Constructing commands

`bon` builders provide optional named-argument construction. This example uses a
typed ID and extent:

```rust
use kitty_rc_commands::{GetText, TextExtent};
use kitty_rc_commands::targets::WindowId;

let text = GetText::builder()
    .match_spec(WindowId(42))
    .extent(TextExtent::Screen)
    .ansi(true)
    .build();
```

Prefer direct constructors when their defaults fit. Public fields support
additional options without going through a builder:

```rust
use kitty_rc_commands::{FocusWindow, GetText, Ls, TextExtent};
use kitty_rc_commands::process::{Launch, LaunchCwd, LaunchType};
use kitty_rc_commands::targets::{TabId, WindowId, WindowMatch};
use kitty_rc_commands::window::WindowCommand;

let listing = Ls::tab(TabId(3));
let focus = FocusWindow::id(WindowId(42));
let screen = GetText::window(WindowId(42), TextExtent::Screen);
let editors = Ls::window(WindowMatch::title("^editor$"));
let title = WindowCommand::title(WindowId(42), "Build output");
let terminal = Launch {
    kind: LaunchType::Tab,
    cwd: Some(LaunchCwd::Current),
    ..Launch::new(["bash"])
};
```

Target types follow each native command's fields:

- `WindowMatch` / `TabMatch`: typed ID and field expressions for listing, focus,
  text, screenshot and other match-only fields. Title/cwd/cmdline constructors
  accept **regular expressions**, preserving their query boundaries.
- `WindowTarget` / `TabTarget`: active, calling window/tab, typed ID or match
  selection for commands with native `self` support.
- `WindowSet`: `window(WindowId(..))`, `tab(TabId(..))` or `all()` for multi-window
  appearance/input commands. Defaults follow the individual native handler.

The observation `Scope` uses the same typed match fields. Command construction
does no I/O; the execution entrance supplies connection, protection and bounds.
The [root examples](../README.md#typed-command-construction) also cover typed key
chords, layout changes, colors and configuration overrides. `kitty_rc::commands`
re-exports these same leaf-crate APIs.

### Literal nested text actions

Use `KnownAction::paste` or `KnownAction::send_text` to construct native nested
actions from literal Unicode text:

```rust
use kitty_rc_commands::actions::{Action, KnownAction, SendTextModes};
use kitty_rc_commands::targets::WindowId;

let paste = Action::new(KnownAction::paste(r" literal \n ").unwrap());
let send = Action::builder()
    .action(KnownAction::send_text(SendTextModes::All, "hello\n").unwrap())
    .target(WindowId(42))
    .build();
```

Literal `r"\n"` stays backslash plus `n`; `"\n"` supplies an actual newline.
The library encodes text for kitty to decode once, preserving edge whitespace
and Unicode. Empty text returns `EmptyActionText`; whitespace-only and NUL
input are accepted. `ActionText` preserves this invariant when constructing
public `KnownAction` variants directly.

`SendTextModes` selects `All`, `Normal`, `Application`, `Kitty`, or a named
combination such as `NormalApplication`. A mode mismatch can surface as a
native `Unknown action` error. Nested actions use `Action.target`'s single
dispatch window. Paste follows configured filtering/confirmation and paste
behavior; acknowledgment establishes dispatch rather than child delivery.
Both use the existing retiring action execution path. The
[implementation report](../design/library-redesign/capabilities/nested-text0.sol61xh.md)
records the full-parser evidence and effect boundaries.

### Named clipboard buffers and window helpers

`KnownAction::copy_to_buffer` / `KnownAction::paste_from_buffer` name a kitty
buffer; `NewProcess` gained `TabWithCwd` / `OsWindowWithCwd` kinds, and
`SimpleAction` gained the zero-argument window/tab and clipboard helpers:

```rust
use kitty_rc_commands::actions::{Action, KnownAction, NewProcess, SimpleAction};

let copy = Action::new(KnownAction::copy_to_buffer("my buffer").unwrap());
let spawn = Action::new(KnownAction::NewProcess {
    kind: NewProcess::OsWindowWithCwd,
    args: vec!["ssh host".into()],
});
let tidy = Action::new(SimpleAction::CloseOtherWindowsInTab);
```

Buffer names travel as the raw remainder — internal spaces, tabs, newlines
and backslashes stay one literal name, unlike the shell-quoted `new_*` argv
or the escape-encoded nested text actions. `BufferName::new` rejects empty
names and names starting/ending with whitespace the native parser would
alter (`InvalidBufferName`); interior whitespace is accepted verbatim. The
names `clipboard` and `primary` reach the system clipboard/selection instead
of kitty's in-memory buffers — a documented string choice, not a typed split.

The `_with_cwd` kinds inherit the working directory from the dispatch
window's shell-integration (OSC 7) reporting. Helper effects depend on
dispatch scope and integration: `close_other_tabs_in_os_window` targets the
dispatch window's OS window (the active one without a target), the
command-output helpers require shell integration, and
`copy_last_command_output` is registered by kitty 0.49.2 but not 0.43.1.
Acknowledgment still establishes dispatch, not selection or child effects.
The [implementation report](../design/library-redesign/capabilities/window-tab-actions0.glm53fx.md)
records the grammar distinctions and native verification.

### Finite text and byte input

`SendText::new` accepts literal Unicode; `SendText::bytes` accepts arbitrary
bytes. Both construct one command without I/O or escape interpretation:

```rust
use kitty_rc_commands::targets::{WindowId, WindowSet};
use kitty_rc_commands::window::{BracketedPaste, SendText, SendTextData};

let literal = SendText::new(r"literal \n"); // two characters: backslash and n
let binary = SendText {
    windows: WindowSet::window(WindowId(42)),
    ..SendText::bytes([0, 0xff])
};
let paste = SendText::builder()
    .data("hello\n") // an actual newline, supplied by Rust
    .windows(WindowSet::window(WindowId(42)))
    .bracketed_paste(BracketedPaste::Auto)
    .build();
let explicit = SendText::builder()
    .data(SendTextData::Bytes(vec![0, 0xff]))
    .build();
```

Text uses native `text:` encoding; bytes use standard `base64:`. A Unicode
`ÿ` therefore supplies UTF-8 bytes `c3 bf`, whereas the byte constructor can
supply the single byte `ff`. Bracketed-paste wrapping/sanitization remains
server behavior when that option is selected.

Pass these commands to the ordinary direct or managed execution entrance.
They suppress replies and retire the transport; `Completion::Submitted`
means local submission, not child delivery. Existing request-size bounds
apply to the complete request. A 1024-byte reference-client chunk size does
not impose a limit on these finite constructors. File/stdin chunking belongs
to a separate execution conversation.

**API migration:** `kitty-rc-commands` 0.3 and facade `kitty-rc` 0.8 replace
the public `text: String` field with `data: SendTextData`; use
`data: old_text.into()` in struct literals and `.data(old_text)` in builders.
Full struct literals also supply `broadcast: None` for ordinary data, or use
`..SendText::default()`.
Direct `SendText::new(old_text)` calls retain their behavior. The source
revision advances the command-dependent execution/registry/observation/CLI
packages alongside these markers; use that coherent dependency set.

The [finite-core report](../design/library-redesign/capabilities/sendtext-core0.sol61xh.md)
records source-handler and preparation evidence. The
[native boundary review](../design/library-redesign/capabilities/sendtext-boundaries0.gpt6a.md)
explains why an escaped-text convenience and nested paste need separate
grammar decisions.

### Explicit broadcast messages

`BroadcastId` names native broadcast bookkeeping. Choose a nonempty ID for the
intended broadcast; the constructor preserves it exactly. Data may carry that
ID, and `BroadcastControl` requires it for explicit start/end messages:

```rust
use kitty_rc_commands::targets::{TabId, WindowSet};
use kitty_rc_commands::window::{BroadcastControl, BroadcastId, SendText};

let id = BroadcastId::new("caller-chosen-unique-id").unwrap();
let recipients = WindowSet::tab(TabId(3));
let start = BroadcastControl {
    windows: recipients.clone(),
    ..BroadcastControl::start(id.clone())
};
let data = SendText::builder()
    .data("hello\n")
    .broadcast(id.clone())
    .windows(recipients.clone())
    .build();
let end = BroadcastControl {
    windows: recipients,
    ..BroadcastControl::end(id)
};
// These are command values; construction has submitted nothing.
```

Each is an independent finite, submission-only command using ordinary execution.
The caller owns explicit end attempts. Awaiting submissions on separate sockets
does not establish remote effect order or cleanup; dropping a value sends
nothing. A running ordered broadcast conversation remains a separate capability.

ID-bearing data can enroll recipients even without start. End affects the ID's
accumulated membership beyond its current match/exclusion scope, and selection
failure can prevent cleanup. Origin-window callbacks are distinct from recipient
selection and socket lifetime; end leaves those callbacks installed, so reusing
an old ID can reconnect them to new membership. Distinct IDs also do not isolate
the shared per-window cursor-render flag. See the
[broadcast report](../design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md)
for exact native behavior and evidence.

## Concern map

| Need | Public crate / entrance | Ownership boundary |
|---|---|---|
| Envelope and response classification | `kitty-rc-schema`: `PlainRequest`, `ResponseEnvelope::classify` | No framing, I/O or command semantics |
| Incremental DCS frames | `kitty-rc-frame`: `frame`, `FrameDecoder` | Caller owns stream/limits; ambiguous bytes poison decoder |
| Explicit protection | `kitty-rc-crypto`: `ServerPublicKey`, `encrypt_raw_request` | Caller supplies key/password/timestamp/RNG; exactly one protection boundary |
| Native command/profile | `kitty-rc-commands`: `Command`, domain modules, `catalog` | Typed output and completion mode; no connection/setup |
| Route and protection | `kitty-rc-endpoint`: `Address`, `AccessBinding`, `Endpoint` | Address/provenance are not verified server identity |
| Candidate acquisition/selection | `kitty-rc-discovery`: `discover`, `select` | Caller selects environment/config/socket sources and ambiguity policy |
| Credential acquisition/validation | `kitty-rc-credentials`: `resolve`, `Policy`, selected file/store/environment sources | No implicit helper or plaintext fallback |
| Credential persistence | `kitty-rc-key-store`: parsing/selection and `storage` | Explicit path and bounded I/O; persisted compatibility separate from Rust selection |
| Pure preparation/evidence | `kitty-rc-exchange`: `prepare`, `Evidence`, `Outcome` | No runtime, no waiter or automatic retry |
| Caller-owned serial socket | `kitty-rc-tokio`: `Session` | One in-flight exchange per mutable session |
| Concurrent native custody | `kitty-rc-registry`: `Registry`, `Reservation`, `Operation` | Bounded operations, socket pool and volatile result ownership |
| Finite scoped observation | `kitty-rc-listing`: `Observer`, `Receipt`, `Snapshot` | Explicit sample/publish; no subscription or resident task |
| Uncertain-outcome follow-up | `kitty-rc-check`: `RetainedOriginal`, `CheckInput` | Fresh `ls` only; original result stays separate, never replayed |

Runtime-independent schema/frame/crypto/commands/exchange form lower layers;
runtime access/execution, registry and observation build on those, never on the
facade. `client` depends upward on those entrances, not the reverse. CLI adoption
lives in `apps/cli`; its command-line help is a separate entrance, not a required
library dependency. These APIs are still being redesigned below 1.0: no local API
stability or compatibility with removed legacy serializers is promised.

## Runnable profiles

All endpoint examples require `--endpoint unix:/absolute/socket` or
`--endpoint tcp:IP:port` **and** a deliberate protection choice. Nothing reads
`KITTY_LISTEN_ON`, guesses a PID socket suffix, finds a default credential store,
spawns a helper, or writes to an ambient active kitty.

```sh
mise exec -- cargo run -p kitty-rc --example pure
mise exec -- cargo run -p kitty-rc --example direct -- --help
mise exec -- cargo run -p kitty-rc --example managed -- --help
mise exec -- cargo run -p kitty-rc --example setup -- --help
mise exec -- cargo run -p kitty-rc --example snapshots -- --help
```

To use an explicitly selected server permitting plaintext:

```sh
mise exec -- cargo run -p kitty-rc --example direct -- --endpoint unix:/chosen/kitty.sock --plain
mise exec -- cargo run -p kitty-rc --example managed -- --endpoint unix:/chosen/kitty.sock --plain
mise exec -- cargo run -p kitty-rc --example snapshots -- --endpoint unix:/chosen/kitty.sock --plain --subject chosen-server
```

For encrypted requests, replace `--plain` with
`--key-file /chosen/published-key --password-file /chosen/password`. Each file
must contain exactly one UTF-8 line (terminal LF/CRLF allowed; other whitespace
is preserved). The key is kitty's published `1:<RFC1924-base85>` X25519 key.
Files are explicitly selected, bounded at 4096 bytes, and the key is scoped to
the chosen route. Choose restrictive file permissions; examples do not create
these files or print passwords. **Only requests are protected** by kitty's
application-level protocol: this is not TLS or encrypted reply transport.
Wrong/malformed/missing credentials fail; they never retry in plaintext.

### Pure schema, framing and crypto

[pure.rs](examples/pure.rs) constructs `Ls::all()`, obtains its typed command's
schema envelope, serializes it,
frames it and incrementally decodes seven-byte fragments without any runtime or
server. The optional offline specimen accepts `--key`, `--password` and
`--timestamp-ns` together, using explicit caller timestamp plus OS randomness.
It never reads the clock or a credential source. Do **not** supply production
passwords on command lines: shell history/process listings expose them. The
offline encrypted specimen is encoding evidence, not authenticated live-server
conformance and not something to send with a stale timestamp.

### Direct session

[direct.rs](examples/direct.rs) performs two serial listing reads through a
caller-owned `Session`, with request/response bounds and absolute deadlines.
`connection_token()` makes actual compatible socket reuse visible; it is a
local transport-lifetime token, not server identity, health or replay authority.
Audited profiles may reuse only at a clean terminal boundary; arbitrary raw,
nested or submission-only profiles retire by policy.

Dropping an in-flight direct `execute` future cancels **that local execution**
and retires its owned transport. Already written work may still happen remotely;
inspect retained execution evidence and do not infer rollback. The direct
session has no managed inventory or detached completion for that cancelled
future. Calling the same command again is a new occurrence, not recovery.
`--cancel-read` demonstrates a short waiter deadline on a separate direct read
and prints its local evidence; a fast completed reply can win the cancel race.

### Managed concurrency and result custody

[managed.rs](examples/managed.rs) reserves caller-known identities before two
accepted concurrent `Ls` submissions, demonstrates a detachable waiter timeout,
waits both, performs each sole typed take, then explicitly releases its record.
`--cancel-second` requests **operation** cancellation separately. A raced
completion can win cancellation; `cancel()` is a request, not rollback proof.

- Reserve is local identity creation. `submit` is the synchronous acceptance
  cut; refusal returns the command/reason. An accepted occurrence is spent even
  after release: a saved reservation is not replay authority.
- Dropping/timing out `operation.wait()` or dropping an operation handle only
  detaches a waiter. Save its token or inspect `registry.inventory()`; the driver
  continues accepted work. `registry.wait(token)` and `take_native(token)` are
  recovery entrances, not resubmission.
- `take()` transfers native bytes out of custody and decodes exactly once;
  the decoded allocation belongs to the caller, not the registry quota. Required
  data does not become an empty/default success. Errors preserve native evidence.
  `release()` is still explicit after take, including a failed typed decode.
- The actual bounded pool holds idle sockets as well as active/connecting/retiring
  slots. Connection quotas have command-class partitions; registry operation,
  input and retained-output quotas are separate. Waiting does not hold a socket
  lease. Retained records consume admission capacity until deliberately released.
- Concurrent submissions have **no per-endpoint total remote order** across
  sockets. Await one terminal boundary before submitting a dependent effect.
  Acknowledgment, UI effect and child exit are distinct; choose `LaunchWait` only
  when its deferred exit contract is wanted.
- Custody is in-memory and volatile. Process loss or driver termination does not
  leave a durable job journal. No automatic retries, replay, pruning or cleanup
  occur. Upload lanes have additional unresolved-lane ownership/release rules;
  these finite examples do not bypass them.

The success path uses `shutdown(Shutdown::Drain)` before stopping the driver:
close intake, finish accepted work, close transports. `Shutdown::Cancel` requests
local cancellation instead; retained inventory/results still need explicit
inspection/take/release. Unexpected driver stop returns unfinished inventory
rather than a fabricated clean drain. These short executable examples exit on
setup/admission errors; long-lived applications should keep the registry/token
inventory and use an explicit error/shutdown boundary instead of losing volatile
custody when their process exits.

### Explicit setup generations

[setup.rs](examples/setup.rs) selects a candidate and credentials independently,
registers a pending route, installs its exact ticket, deliberately invalidates
credentials and reacquires/reinstalls a new generation. It opens **no socket**.
`register(endpoint)` is the ready shortcut; `register_pending`/`install_setup`
make slower setup ownership explicit. Intake refuses while setup is pending.

Discovery and credential acquisition happen outside the owner lock. Explicit
candidate selection bypasses all environment/configured acquisition. Alternative
callers can opt into `CapturedEnvironment`, `SocketPaths`, `CapturedKittyKey`,
`KeyFile`, `StoreKey` and `CapturedPassword`; source ordering/missing-material
policy is caller-selected, and errors do not silently fall through. A PID hint
or store lookup is not proof against PID reuse or proof of liveness.

Setup tickets bind local route/credential generations. Stale installs cannot
overwrite newer setup. Invalidation can reject queued old-basis work; inspect
its retained setup rejection/input, and use a **new reservation** for any
deliberately selected new attempt. Running work retains its captured basis.
Credential refresh is not an implicit replay, server-incarnation discovery or
permission to change another active endpoint.

### Snapshots and check-only follow-up

[snapshots.rs](examples/snapshots.rs) makes two finite reads, reports sampled
differences and explicitly accepts each as the next baseline. Scope, subject,
local generation, interpretation, issuance order and timing travel with the
receipt. A filtered “no longer returned” is not proof a window was destroyed;
sampling is not an event feed, causal barrier, server-atomic history or resume
cursor. Unknown/missing/unsupported facts remain distinct. No polling task starts
unless the caller drives one; there is no automatic reconciliation.

The optional `--focus-window-id ID` is a **mutation opt-in** against the explicit
endpoint. It uses `FocusWindow::id(WindowId(id))` for stable selection. If that native
operation completes unresolved, the example keeps its original result, selects
a fresh read-only `CheckInput` for that exact ID and focused fact, reports the
assessment, then transfers/releases the original separately. The ordinary
successful-focus path needs no uncertain-result check. Do not force a network
fault on a human server merely to exercise the branch.

The check continuation owns one fresh `ls` plus bounded criteria/freshness,
**never the original mutation input**. Matching, observed satisfaction and
causation are separate: satisfaction does not rewrite an unknown native result,
prove attribution, permit replay or imply remote cleanup. A check failure also
leaves original evidence separate. Explicit stable ID matches are still scoped
to a selected server basis; an active scope means “whatever is active when the
server interprets it,” not a stable target identity.

## Verification entrances

```sh
mise exec -- cargo check -p kitty-rc --examples --locked
mise exec -- cargo test -p kitty-rc -p kitty-rc-commands --doc --locked
mise exec -- cargo clippy -p kitty-rc --examples --locked -- -D warnings
```

[conformance.rs](examples/conformance.rs) is preserved for the isolated live
runner (`mise exec -- cargo run -p kitty-rc-verify -- finite`); unlike the
offline/help checks, it performs deliberate mutations in a newly created
private kitty instance. Do not point conformance at an ambient user
endpoint. Current evidence and remaining multiversion limitations belong in
the
[verification report](../design/library-redesign/verification/entrances0.sol61m.md).

## Cross-references

- [Support policy](../doc/usage/support.md) ⇄ this guide: capability selection versus executable adoption.
- [Capability inventory](../design/library-redesign/capabilities/inventory0.sol61m.md): exact stable/HEAD, nested dispatch, configuration and native completion boundaries.
- [Observation report](../design/library-redesign/observation/implementation0.sol61m.md): scoped sample/difference semantics and check-only implementation evidence.
- [Core implementation](../design/library-redesign/implementation0.sol61m.md): corrected cancellation, native custody and actual isolated live journeys.
