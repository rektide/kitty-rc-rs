---
type: ImplementationReport
title: Finite send-text literal text and raw-byte core
description: Revision-verified text/base64 contracts, typed finite API and public migration, submission-only preparation tests, source-handler oracles and the next isolated capability boundary.
resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-28
sources:
  - resource: "opencode://ses_e7b57d91bffeto158h8GVlMvHn"
    title: Commissioned finite text/raw-byte core, independently checked named upstream handlers, implemented typed constructors/builders and source/preparation evidence, and identified next-slice boundaries
  - resource: /design/library-redesign/capabilities/sendtext0.glm53m.md
    title: Unverified broader proposal used as an inquiry prompt, not an accepted contract or implementation mandate
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Prior stable audit, submission-only policy and deferred send-text boundaries
  - resource: /protocol/commands/src/window.rs
    title: Implemented SendTextData and finite SendText constructors, builder and payload
  - resource: /protocol/commands/tests/contracts.rs
    title: Independent finite payload goldens, ordinary builder inputs and retained scope/options
  - resource: /execution/exchange/src/lib.rs
    title: Existing executor-owned suppression, request limits and prepared reuse metadata
  - resource: /execution/exchange/tests/send_text.rs
    title: New preparation evidence for both finite data variants without runtime changes
  - resource: /tool/verification/src/source.rs
    title: Independent source-handler fixtures and two-revision comparisons
  - resource: /tool/verification/fixtures/source_oracle.py
    title: Minimal unchanged-source execution boundary with synthetic selected-window sinks
  - resource: /tool/verification/README.md
    title: Maintained source verification entrance and evidence limits
  - resource: /doc/usage/support.md
    title: Exact-profile source versus live evidence policy
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/send_text.py
    title: Directly inspected stable v0.49.2 generator, protocol specification and server handler
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/send_text.py
    title: Identical inspected HEAD source, confirmed by git diff and executed separately by the oracle
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/utils.py
    title: Native bracketed-paste sanitizer inspected and executed in the oracle
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/utils.py
    title: parse_send_text_bytes, nested send_text and paste parsers read to distinguish future grammar ownership
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/data-types.c
    title: Native escape state machine inspected for next-slice risks, not translated or claimed executed here
verification:
  scope: Finite Rust payload and preparation checks, targeted crate gates, workspace compilation and source-executed upstream handlers
  evidence: 59 commands/exchange tests and 23 verification-library tests passed; commands doctests, targeted strict clippy, owned-file formatting and document checks passed; 11 finite send-text handler cases passed at each named revision
  limits: Not a live kitty/child/TTY test, a new minimum-version claim, or acceptance of the broader proposal; source fixture bypasses matching and dispatcher suppression and excludes sessions/key events
---

# Finite send-text core

## Scope and outcome

Built first under the parent's explicit commission: one finite `SendText`
submission containing literal Unicode text or arbitrary raw bytes. No tickets
were created or changed. The broader [GLM proposal](sendtext0.glm53m.md) remains
an unverified independent proposal; this report neither edits nor adopts it.

The existing source profile stays `Mode::SubmissionOnly`, short-class,
non-reusable. Constructors perform no I/O, expansion, chunking, endpoint
selection or execution. A submitted result is local outbound completion only:
it cannot establish target matching, child delivery or remote success. There
are no execution/runtime implementation changes.

## Established upstream contract

Inspected the local upstream archive at stable v0.49.2 commit
`b2cb871254fbdac19a085188813c8dfb258b95cd` and inspected 0.50dev HEAD
`5bf3c2dc676efd5c4b4526eb135c922701565707`. `git diff` between these exact
revisions shows **no changes in `kitty/rc/send_text.py`**; the new source oracle
executes each revision separately rather than assuming equivalence.

| Payload | Native handler behavior |
| --- | --- |
| `text:<remainder>` | Split at the first colon, then UTF-8-encode the entire remainder; no backslash expansion. |
| `base64:<remainder>` | Standard-base64-decode the remainder into bytes; no UTF-8 conversion or escape interpretation. |

Thus text U+00FF produces UTF-8 `c3 bf`; a raw `ff` byte needs the byte path.
NUL is valid in both representations. Literal `\n` remains backslash + `n`.
Empty text/bytes each produce one finite request, even though the reference
client's generator emits no chunk for an empty source.

The Python reference client sets `limit = 1024` in `message_to_kitty`, expands
positional input through `parse_send_text_bytes`, and slices the resulting bytes
into base64 requests. File and non-TTY stdin sources also use that generator
limit. **1024 is a client chunk choice, not a declared protocol frame limit.**
The handler oracle accepts a 1025-byte payload in one invocation. That does not
establish an unlimited dispatcher/transport ceiling; the Rust preparation
boundary still enforces the caller's request-size budget.

`exclude_active` filters selected windows before writes. The retained
`bracketed_paste` option can wrap and sanitize the data server-side: raw-byte
encoding is byte-preserving before that deliberate option, not a promise that
all paste modes write identical bytes. No new bracketed-paste behavior is added
or normalized locally.

Upstream's `disallow_responses` describes reference-client policy; the audited
[inventory](inventory0.sol61m.md#revision-and-outcome-law) already distinguishes
that policy from a socket server's ability to acknowledge unsuppressed requests.
This library continues suppressing replies through existing execution
preparation and retiring the transport; it does not expose a reply-based
"delivered" profile.

## Typed API and deliberate migration

Implemented in [window.rs](/protocol/commands/src/window.rs):

```rust
pub enum SendTextData {
    Text(String),
    Bytes(Vec<u8>),
}
```

- `SendText::new(text: impl Into<String>)` remains **literal text**.
- `SendText::bytes(bytes: impl Into<Vec<u8>>)` accepts arbitrary bytes in one
  request; there is no encoding validation or UTF-8 requirement.
- `SendText::builder().data(value)` accepts `&str`, `String`, `Vec<u8>`,
  `&[u8]`, or explicit `SendTextData` via `Into`.
- `SendTextData::default()` is empty literal text, retaining the old default
  command's `text:` representation. The builder still requires explicit data.
- `windows`, `exclude_active` and `bracketed_paste` retain their existing fields,
  builder setters, defaults and wire names.

**This changes the public Rust API.** Struct literals/field access using
`text: String` must migrate to `data: SendTextData`; builder `.text(value)`
becomes `.data(value)`. `SendText::new(...)` behavior and call shape do not
change. There is no duplicate legacy field or ambiguous precedence between text
and bytes. The parent owns package/API version integration; no version fields
or lockfiles were changed in this slice.

The catalog keeps its existing `FiniteSubsetUploadDeferred` classification and
updates the note to name literal text/raw bytes and the unimplemented surfaces.
The enum label is historical catalog vocabulary, **not** a claim that finite
send-text uses an accumulator/upload handshake.

## Evidence and checks

All Rust commands used the user's `mise exec --` selection, reporting
`rustc 1.101.0-nightly (822182db4 2026-10-13)`. No toolchain pin was introduced.

| Check | Result |
| --- | --- |
| `cargo nextest run -p kitty-rc-commands -p kitty-rc-exchange --locked` | 59 passed, none skipped. |
| `cargo nextest run -p kitty-rc-verify --lib --locked` | 23 passed, none skipped. |
| `cargo test -p kitty-rc-commands --doc --locked` | Two positive and one compile-fail example passed. |
| `cargo clippy -p kitty-rc-commands -p kitty-rc-exchange --all-targets --no-deps --locked -- -D warnings` | Passed. |
| `cargo clippy -p kitty-rc-verify --all-targets --no-deps --locked -- -D warnings` | Passed. |
| `cargo check --workspace --all-targets --locked` | Passed; catches workspace consumers of the migrated public field/builder. |
| `cargo run -p kitty-rc-verify --locked -- source /home/rektide/archive/kovidgoyal/kitty` | Existing stable inventory/constructors/parsers/uploads passed; 11 new send-text handler cases passed at **each** named revision. |
| `rustfmt --check --edition 2024 --config max_width=100` on owned Rust files | Passed; explicit width avoids inherited 160-column formatting churn. |
| Rekon `check --okf` on this report, support policy and verification guide | Passed. |

The [payload goldens](/protocol/commands/tests/contracts.rs) establish binary
NUL/`ff`, UTF-8 distinction, literal backslashes, empty variants, ordinary owned
and borrowed builder inputs, and scope/paste/exclusion retention. Existing
selector tests were migrated without rewriting their expectations.
The [preparation tests](/execution/exchange/tests/send_text.rs) establish
executor-injected `no_response: true`, submission-only/non-reusable metadata,
absence of stream/async flags, an honest `Completion::Submitted`, acceptance
above the reference chunk size and continued caller-selected size rejection.

The source verifier follows its existing independent-golden methodology: Rust
defines wire inputs and expected bytes independently of this library's
serialization, and the minimal Python boundary runs the **actual named upstream
handler and sanitizer**. It records write calls on synthetic, already-selected
windows. The 11 cases cover literal UTF-8/backslashes/colon, binary NUL/`ff`,
UTF-8 through base64, binary backslashes, both empty encodings, 1025 bytes,
active exclusion, paste-auto off/on, and forced-paste sanitization.

These are not a Rust encoder/decoder mirror round trip or a copied translation
of upstream logic. They are also **not live matching, real child/TTY delivery,
dispatcher suppression, full multi-window paste conformance, or version-floor
evidence**. Existing live matrix claims remain unchanged; unchanged runtime and
live journeys were not rerun merely to confirm earlier results.

## Architectural checkpoint and next candidate

**Follow-on:** the parent selected [explicit finite broadcast messages](sendtext-broadcast0.sol61xh.md)
after its [native grammar assessment](sendtext-boundaries0.gpt6a.md). The
recommendation and exclusions below are preserved as this earlier checkpoint,
not the current selection; no local escape decoder was commissioned.

Still excluded: ANSI-C decoding, escaped constructors, nested `paste`, nested
`send_text`, broadcast `session_id`/start/end, `kitty-key`, chunked execution,
CLI and automatic cleanup. Slice B execution and Slice C CLI remain separate
future work, with prospective tracking discussed only after building. Nothing
here authorizes their implementation or ticket creation.

**Recommended next coherent candidate: an explicit, fallible escaped-text
entrance for outer RC `SendText` only**, after characterizing the native decoder
at named revisions. That transformation can produce the already-established
literal `Text` variant before infallible `Command::request`; it need not change
execution or introduce another wire encoding. This is a recommendation for the
parent's selection, not an implemented or accepted addition.

The broader proposal's grammar table is not sufficient evidence for that work:

1. The native C state's octal end-of-input path visibly invokes its digit-flush
   macro with a different radix from normal octal termination. Characterize
   exact-end versus following-character vectors with a real native oracle before
   promising Bash-equivalent or upstream-equivalent behavior.
2. Python Unicode versus Rust Unicode-scalar rules and the subsequent UTF-8
   encoding step need separate characterization. Do not infer that native
   expansion rejects every surrogate simply because a Rust `String` cannot
   contain one. This slice has not executed that C routine.
3. `options/utils.py::paste_parse` expands the nested action's remainder on the
   **server**. Expanding locally then emitting that result as an escaped remainder
   can reinterpret backslashes a second time. Design the source-versus-literal
   API and inverse encoding/validation before adding `KnownAction::paste`.
   Keep this separate from the outer RC convenience.
4. Broadcast is independently stateful: the handler updates a global map,
   cursor rendering and removal/focus callbacks; ordinary data with a session ID
   also updates membership. Submission-only start/end cannot confirm remote
   cleanup. A lifecycle-aware shape and evidence deserve their own checkpoint,
   not incidental variants added to the finite data enum now.

The [support policy](/doc/usage/support.md#gate-the-exact-profile) and
[verification guide](/tool/verification/README.md#the-two-deliberate-python-boundaries)
link back to this report. The parent owns shared README/index navigation and
the next-stage architectural review.
