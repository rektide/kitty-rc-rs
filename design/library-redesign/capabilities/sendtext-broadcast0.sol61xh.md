---
type: ImplementationReport
title: Explicit finite send-text broadcast messages
description: Validated caller-chosen broadcast identity, separate finite start/end commands, optional ID-bearing data, native lifecycle source evidence and submission-only limits.
resource: /design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-28
sources:
  - resource: "opencode://ses_e7b57d91bffeto158h8GVlMvHn"
    title: Parent-selected explicit broadcast follow-on, typed implementation and fresh native-class lifecycle scenarios, followed by workspace convergence checks
  - resource: /design/library-redesign/capabilities/sendtext-boundaries0.gpt6a.md
    title: Parent's reviewed order and native escape/action evidence; broadcast selected while local decoder requires a human grammar choice
  - resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
    title: Historical finite literal-text/raw-byte implementation checkpoint extended without changing its content enum
  - resource: /protocol/commands/src/window.rs
    title: Broadcast identity, explicit control command, optional data identity and public completion/lifecycle documentation
  - resource: /protocol/commands/src/catalog.rs
    title: Updated finite coverage note without claiming a conversation or upload lane
  - resource: /protocol/commands/tests/broadcast.rs
    title: Empty-ID and exact-preservation tests, independent payload fixtures and ordinary-data omission
  - resource: /execution/exchange/tests/send_text.rs
    title: Executor-owned no_response and retirement checks for start, data and end
  - resource: /tool/verification/src/source.rs
    title: Independent per-step state goldens and comparisons at both named upstream revisions
  - resource: /tool/verification/fixtures/source_oracle.py
    title: Fresh native classes/global state per scenario with only windows, selection, boss and callback invocations injected
  - resource: /tool/verification/README.md
    title: Maintained source verification entrance and explicit source-versus-live limits
  - resource: /doc/usage/support.md
    title: Exact-profile support policy without expanding live-matrix or oldest-version claims
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/send_text.py
    title: Directly inspected stable v0.49.2 handler, Session, SessionAction, ClearSession and FocusChangedSession; executed in the source oracle
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/send_text.py
    title: Identical inspected HEAD broadcast implementation, confirmed by git diff and separately executed in the source oracle
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/base.py
    title: windows_for_payload selects recipients before lifecycle handling and may raise for unmatched expressions
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kittens/broadcast/rc.go
    title: Reference broadcast uses no_response, caller-generated random identity and explicit TTY control messages
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kittens/broadcast/main.go
    title: Reference loop emits start, data and an explicit end attempt; not evidence of socket-origin callbacks
verification:
  scope: Rust finite message/preparation tests, independently executed native handler/classes at stable and inspected HEAD, and final workspace gates
  evidence: 63 targeted commands/exchange tests and 248 final workspace tests passed; eight workspace doctests, strict all-target clippy, workspace formatting and owned OKF checks passed; three fresh-state broadcast scenarios plus eleven finite data cases passed at each named revision
  limits: No live socket/TTY, origin event dispatcher, matching implementation, remote receipt/order/cleanup observation, minimum-version claim or managed broadcast owner
---

# Explicit finite broadcast messages

## Selection and implementation boundary

The parent accepted the [finite core](sendtext-core0.sol61xh.md), then selected
explicit broadcast messages as the second bounded slice. Its
[native escape assessment](sendtext-boundaries0.gpt6a.md) supersedes the core
report's tentative next-candidate recommendation: **no local escaped helper is
built**. A grammar choice remains human-owned. The literal Rust text/byte
entrances need no decoder.

Implemented identity and finite start/data/end payloads only. No new runtime,
socket, upload lane, automatic cleanup, managed conversation, CLI, random ID,
environment/filesystem lookup or ticket action was introduced. Parent-owned
README/index navigation, manifests, lockfile and API-marker integration were
left alone.

## API shape and rationale

All types live in [window.rs](/protocol/commands/src/window.rs):

| Surface | Contract |
| --- | --- |
| `BroadcastId::new(impl Into<String>) -> Result<BroadcastId, EmptyBroadcastId>` | Reject only an empty string; preserve every other string exactly. Private storage and no unchecked default/deserialization path maintain validation. |
| `BroadcastId::as_str()` | Borrow the caller's unchanged identity. |
| `SendText.broadcast: Option<BroadcastId>` | Optional ID on literal text or raw bytes; serialize `session_id` only when selected. |
| `BroadcastOperation::{Start, End}` | Explicit control operation, separate from content and identity. |
| `BroadcastControl::new(id, operation)`, `::start(id)`, `::end(id)` | Construct one finite `send-text` control message with required validated identity. |
| `BroadcastControl::builder()` | Requires `.broadcast(id)` and `.operation(...)`; native window scope and exclusion have existing defaults. |

`SendTextData` remains **only** `Text(String)` / `Bytes(Vec<u8>)`. A lifecycle
variant could otherwise be constructed without the required identity. The
optional data field and mandatory control field make that distinction structural.
The builder accepts a validated `BroadcastId`, not an unchecked string that could
defer a fallible conversion until infallible request construction.

The identity is not called `Session`, because the existing transport session and
a hypothetical running broadcast owner are different things. `BroadcastControl`
is a message, not an owner or cleanup guard. There is no `Default` control or
default/random broadcast identity. Whitespace, Unicode, colon and NUL remain
valid nonempty IDs; no unsupported grammar, trimming or normalization is imposed.
Nonempty validation is a deliberate library guard against the native empty-name
slot, not a claim that upstream itself rejects empty names.

For example, this constructs values only:

```rust
use kitty_rc_commands::targets::WindowSet;
use kitty_rc_commands::window::{BroadcastControl, BroadcastId, SendText};

let id = BroadcastId::new("caller-chosen").unwrap();
let start = BroadcastControl {
    windows: WindowSet::all(),
    ..BroadcastControl::start(id.clone())
};
let data = SendText::builder()
    .data(vec![0, 0xff])
    .broadcast(id.clone())
    .windows(WindowSet::all())
    .build();
let end = BroadcastControl {
    windows: WindowSet::all(),
    ..BroadcastControl::end(id)
};
```

Both commands use `Class::Short`, `Mode::SubmissionOnly`, default retirement and
the existing executor-injected `no_response: true`. Plain data without an ID
continues emitting no `session_id` and no implicit lifecycle. Controls emit
`session:start` or `session:end`; they expose scope/exclusion but no ignored
bracketed-paste setting. Constructors do no I/O and drop sends nothing.

The optional field is also a Rust struct-literal migration: a full explicit
`SendText { ... }` must supply `broadcast: None` or use struct-update defaults.
Existing `new`, `bytes` and builder calls without an ID retain their wire form.
The catalog keeps its finite-subset classification and explains the expanded
finite capability without claiming upload/conversation support.

## Exact native behavior

Audited stable v0.49.2 `b2cb871254fbdac19a085188813c8dfb258b95cd` and inspected
HEAD `5bf3c2dc676efd5c4b4526eb135c922701565707` again. Their entire
`kitty/rc/send_text.py` files have no diff; each is nevertheless separately
executed in the source verifier.

- ID-bearing **data** creates/updates accumulated membership and marks selected
  recipients even if start never occurred. It does not install origin hooks.
- **Start** accumulates selected recipients. Only a supplied origin window gets
  removal/focus callbacks. Repeated start replaces that ID's callbacks on that
  origin while retaining callbacks for other IDs.
- **Focus callbacks** change cursor-render flags for the ID's recorded members;
  they do not remove those members.
- **End** first clears/discards currently selected, non-excluded recipients,
  then native `ClearSession` removes the map entry and clears all remaining
  recorded members still present in the boss's window map. This includes members
  outside the present match and previously enrolled active windows now excluded.
- Explicit end does **not** remove the origin's callback list entries. Later
  native callbacks for the removed map entry are harmless no-ops unless that ID
  has been reused. The library does not pretend to detach callbacks.
- Native origin-removal callbacks can clear the membership they reference.
  **Ordinary socket closure is not origin-window removal.** The reference TTY
  broadcast loop's origin and callbacks must not be assumed for socket calls.

Match selection runs before lifecycle handling and can raise before end does any
cleanup. End is consequently an **attempt**, not a confirmed remote cleanup.
Even a valid local request can be lost, refused or processed later without a
reply. Awaiting local submission of start/data/end over separate retiring sockets
does **not** establish their remote effect ordering. A late data request can
enroll members after an earlier end; no hidden retry/reconciliation repairs it.

Identities are server-global, caller-chosen bookkeeping keys, not leases or
isolation guarantees. The cursor-render flag itself is per-window, not a
reference-counted resource per broadcast ID. Callers must not infer ownership
isolation for overlapping broadcasts from distinct IDs.

The inspected reference `kittens/broadcast/{rc,main}.go` generates a random ID,
uses `no_response`, writes TTY escape messages, and explicitly attempts end after
its loop. Those are choices of that running client, not protocol requirements to
silently import into these finite commands.

## Independent evidence

The [new payload tests](/protocol/commands/tests/broadcast.rs) cover empty-ID
rejection and exact preservation, distinct start/text/bytes/end encodings with a
shared ID and options, omission for ordinary data, and control scope defaults.
A compile-fail doctest establishes that the control builder cannot build without
an ID. [Preparation tests](/execution/exchange/tests/send_text.rs) establish
short/submission-only profiles, `no_response`, retirement, absent upload/async
metadata and an honest `Completion::Submitted`, without runtime changes.

The source oracle executes upstream's actual `response_from_kitty`, `Session`,
`SessionAction`, `ClearSession` and `FocusChangedSession`, extracted unchanged
from each named revision. Every scenario gets fresh classes, global map and
fixture objects. The fixture does **not** implement membership or cleanup logic.
Rust supplies independent expected snapshots of members, cursor flags, write
bytes and callback type/ID lists after each step:

1. Four steps: data enrolls active + another recipient; excluded-active binary
   data adds a third without removing prior membership; ordinary untagged data
   does not change membership; end clears selected, unselected and excluded
   accumulated members.
2. Two steps: repeated no-origin starts accumulate members with **no callbacks**.
   The scenario deliberately leaves membership present; scenario teardown is not
   a claimed library/native cleanup behavior.
3. Seven steps: origin 99 is distinct from selected recipients; starts for IDs
   a/b/a establish per-ID callback replacement, injected focus loss/gain updates
   flags, explicit end clears a without detaching hooks, and injected removal
   invokes native callbacks to clear b.

The injected boundaries are explicit: an already-selected recipient list, a
synthetic boss/window map and active window, an optional origin, and invocation
of its stored callbacks. The oracle does not test matching, the real origin-event
dispatcher, socket/TTY delivery, socket-close cleanup or network ordering.
These limits are part of the evidence, not simulated away by a fake server.

## Checks and remaining boundaries

All Rust commands used the user's `mise exec --` selection. Final convergence
ran once across the workspace after implementation and oracle work:

| Check | Result |
| --- | --- |
| `cargo nextest run -p kitty-rc-commands -p kitty-rc-exchange --locked` | 63 passed, none skipped, at implementation checkpoint. |
| `cargo run -p kitty-rc-verify --locked -- source /home/rektide/archive/kovidgoyal/kitty` | Existing stable inventory/constructors/parsers/uploads plus eleven finite data cases and three broadcast scenarios passed at **each** named revision. |
| `cargo nextest run --workspace --locked` | 248 passed, none skipped. |
| `cargo test --workspace --doc --locked` | Eight passed: four positive and four compile-fail examples. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed. |
| `cargo fmt --all --check -- --config max_width=100` | Passed; explicit width retains the existing layout rather than inherited 160-column churn. |
| Rekon `check --okf` on the broadcast/core reports and maintained support/tool guides | Passed. |

No new live kitty claim or oldest supported version follows from these gates.
Remaining capabilities include B's bounded multi-message submission conversation
design, C's CLI/source acquisition, `kitty-key`, optional
human-selected escape grammar, literal nested paste/send_text encoding/effects,
and targeted live evidence. None is implemented or ticketed here. B must still
avoid claiming remote receipt/order/cleanup merely from successful local writes.

The [core report](sendtext-core0.sol61xh.md#architectural-checkpoint-and-next-candidate)
preserves the predecessor checkpoint with an explained follow-on link.
[Support policy](/doc/usage/support.md#gate-the-exact-profile) and
[verification guidance](/tool/verification/README.md#the-two-deliberate-python-boundaries)
link back to this report. Return to the parent for architectural review and
human discussion of prospective tracking; building these finite messages does
not authorize creating epics/tickets or continuing into the deferred surfaces.
