---
type: DesignAssessment
title: Send-text core first, explicit broadcast next, escape grammar by choice
description: Source and executable review of the Slice-A sketch, correcting double expansion and escape-compatibility claims and ordering the finite command work before prospective tickets.
resource: /design/library-redesign/capabilities/sendtext-boundaries0.gpt6a.md
status: draft
generated:
  by: "model:openai/gpt-6-astra#xhigh"
  at: 2026-10-28
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Human commissions Sol xhigh implementation under Astra stewardship, requests ticket discussion after building; parent verifies native escape and nested-action boundaries
  - resource: /design/library-redesign/capabilities/sendtext0.glm53m.md
    title: Prior Slice-A sketch under review, preserved as its author's proposal
  - resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
    title: Reviewed finite implementation and Sol's recommendation; native parent findings favor broadcast before selecting an escape grammar
  - resource: /design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md
    title: Reviewed required-ID finite broadcast controls and independently executed lifecycle evidence after the second implementation slice
  - resource: /protocol/commands/src/window.rs
    title: Existing literal finite send-text entrance and submission-only contract
  - resource: /protocol/commands/src/actions.rs
    title: Existing typed and raw nested-action entrances
  - resource: /tool/verification/fixtures/source_oracle.py
    title: Existing source executor supplies Python globals but does not execute the C escape expander
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/data-types.c
    title: Stable escape-expander source inspected locally with git show
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/utils.py
    title: Stable nested action parsers, whitespace handling and UTF-8 conversion inspected locally
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/window.py
    title: Archived HEAD send_text keyboard-mode dispatch and paste configuration/effect paths inspected locally
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/action.py
    title: Archived HEAD action dispatch acknowledges consumption, not child receipt
  - resource: /design/library-redesign/verification/live0.sol61xh.md
    title: Existing pinned-release acquisition and provenance account
verification:
  scope: Native escape expansion, UTF-8 conversion and parse_key_action under locally cached kitty 0.43.1 and 0.49.2 interpreters; no terminal server or child-input effect exercised
  evidence: Both releases executed the parent-authored probe through +runpy; selected outputs and reproduction command are recorded below
  limits: No full Bash conformance, all-release claim, native action-effect test, or automatic validation of the earlier sketch
---

# Send-text: choose the useful core before the grammar

## Direction and authority

The human asked Astra to drive Sol xhigh through useful work and clarified:
**build first, then discuss epics/tickets before creating them**, including work
left uncovered. The first commissioned implementation is the finite literal
text/raw-bytes command core. The earlier [GLM sketch](sendtext0.glm53m.md) is
input to that work, with the corrections below.

**First checkpoint:** Sol's [finite core](sendtext-core0.sol61xh.md) is built
and reviewed: literal text/raw bytes, builder migration and preparation/source
evidence. Sol proposed a characterized escaped-text convenience as the next
candidate. The parent selects explicit broadcast messages next instead:
the native findings below make grammar choice material, while broadcast adds
an independent native capability. This is a scoped implementation decision;
the broader grammar alternatives remain for human discussion.

**Second checkpoint:** [broadcast messages](sendtext-broadcast0.sol61xh.md) are
built and reviewed. `BroadcastId`, optional `SendText.broadcast` and separate
`BroadcastControl` start/end messages preserve the content/lifecycle distinction.
Native lifecycle scenarios establish the stateful effects under explicit
injected boundaries. Final convergence passed 248 workspace tests, eight
doctests, strict Clippy and formatting. The next action is human discussion of
prospective tracking, rather than automatically implementing the rest of the
sketch.

Recommended order:

1. **Literal Unicode and arbitrary bytes:** distinct data variants, direct
   constructors and usable builders; existing submission-only execution.
2. **Explicit finite broadcast messages:** assess the session identity and
   start/data/end shape against upstream, then expose that native capability
   with caller-owned lifecycle. This does not require a chunked transport.
3. **Escape interpretation:** select a grammar deliberately before promising a
   compatibility helper. The existing sketch's promised equivalence is false.
4. **Nested paste/send_text:** establish literal encoding, target/mode and
   effect semantics before expanding the action vocabulary.

The [command-specific upload work](../uploads/implementation0.sol61xh.md)
already resolved the empty-run accumulator problem. Execution conversations
and CLI source acquisition remain the separate B/C directions the human plans
to fork. None of this assessment creates or accepts a ticket.

## Corrections that change the implementation

### A local decoder is unnecessary for the finite core

Rust callers can already supply `"\n"` or `"\u{ff}"` as actual Unicode, and
raw bytes can express `0xff` without pretending it is UTF-8. `SendText::new`
should continue to mean literal text. Raw strings such as `r"\n"` should remain
backslash plus `n`.

Changing public `text: String` into a typed `data` field is an API revision:
`new` callers may remain source-compatible while struct literals and
`.text(...)` builders change. The version marker and migration examples must
say so. The 1024-byte value in the reference generator is a client chunking
choice, not a limit to impose on a finite command constructor.

### The sketch's paste constructor expands twice

Kitty parses `paste <remainder>` through its own escape expander. Expanding the
input locally and then interpolating that result into the action string
changes meaning on the second pass. Native reproduction in **both 0.43.1 and
0.49.2**:

| Step | Value (JSON string notation) |
| --- | --- |
| Caller escape source | `"\\\\n"` |
| One local expansion | `"\\n"` — literal backslash and `n` |
| Sketch's emitted action | `"paste \\n"` |
| Server's parsed argument | `"\n"` — newline |

There is also whitespace loss: native `parse_key_action("paste  leading ")`
produces `"leading"`. `"paste "` has **zero arguments**, not one empty string.
Direct remainder interpolation therefore fails even without a backslash.

If we add typed paste, prefer **literal input plus an encoder** whose output
is decoded once by the server. Check the complete action parser, not only
`paste_parse`, because outer whitespace handling matters. Fixed-width numeric
escapes can protect whitespace and backslashes without a general decoder.
Empty input needs its own explicit contract. Preserve `ActionSpec::Raw` for
caller-authored native syntax.

### The documented grammar does not describe the native edge cases

The parent executed the real native functions, rather than reimplementing them
as an oracle. Both cached releases returned these results:

| Escape source | Native expansion | Consequence |
| --- | --- | --- |
| `\xff` | U+00FF; UTF-8 bytes `c3 bf` | This is Unicode interpretation, not raw byte `ff`. |
| `\123` at end | U+0123 | The sketch's ordinary octal promise does not hold. |
| `\123z` | `Sz` | A following non-digit changes the numeric interpretation. |
| `\1` at end | empty | A valid-looking terminal numeric escape can disappear. |
| `\1z` | U+0001 followed by `z` | End-of-input behavior is material. |
| `\12` at end / `\12z` | U+0012 / newline followed by `z` | The discrepancy also occurs with two digits. |
| `\x` / `\xZ` | empty / `Z` | Missing digits are consumed, not rejected. |
| `\ud800` | A Python surrogate string | Expansion succeeds; subsequent UTF-8 encoding raises. |
| `\U00110000` | `SystemError` | Failure occurs during Unicode construction. |

The [stable C source](https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/data-types.c)
explains the octal/end-of-input discrepancy. A Rust `String` also cannot
represent the surrogate intermediate that Python accepts. The sketch's claim
that all non-scalars fail in the expander conflates separate stages.

There are two coherent future products: a documented, deliberately strict
Unicode escape grammar, or a version-qualified kitty compatibility adapter
with a defined Rust-side treatment of unrepresentable values. Neither can
honestly claim both exact native success behavior and generic Bash semantics.
Select from an actual caller need; raw text/bytes do not need this decision.

The current Python source oracle cannot independently answer the C question:
its extracted parser depends on `defines.expand_ansi_c_escapes`, which its
scope does not provide. A replacement Python decoder would test our own
assumptions again. Execute native kitty or a separately built pinned native
component when validating a future escape contract.

## Broadcast identity deserves an explicit boundary

The native broadcast identifier groups cursor-render bookkeeping. It is not
an acknowledgment, delivery receipt, connection token or upload exclusion
lane. A data message may create/update its session without a preceding start;
an explicit end affects the session's recorded membership as well as selected
windows. These are stateful effects despite each payload being finite.

A typed shape should make identity available on data and require an identity
for lifecycle operations. A nonempty validated identifier avoids accidentally
using the server's empty-name slot. Keep scope/exclusion options visible and
require the caller to submit end deliberately. No destructor should send
network cleanup, and closing a socket is not proof of remote cleanup. Inspect
the origin-window hook separately from match-selected recipient windows.

For naming, distinguish the **broadcast ID**, the **start/end message**, and a
future **running conversation owner**. Naming all three `Session` would collide
with the existing transport session and imply ownership the command does not
provide. The exact Rust shape belongs in Sol's next architectural checkpoint.

## Why nested send_text remains its own decision

Native `send_text` inside `action` differs from RC `send-text`:

- It takes a keyboard-mode expression plus escape-encoded text, then converts
  expanded Unicode to UTF-8. That is not an arbitrary byte input entrance.
- `Window.send_text` checks current keyboard mode; unmatched mode or empty
  input returns without writing. A typed mode vocabulary needs this semantic
  audit, not just a string enum inferred from examples.
- Outer action targeting selects a window for dispatch; it is not the RC
  multi-window `WindowSet` behavior.
- `paste` enters `paste_with_actions`: configured filtering, confirmations,
  newline handling and bracketed-paste behavior can change or delay effects.
  It is not another spelling for byte-exact `send-text`.
- An action acknowledgment establishes dispatch/consumption, not child receipt
  or later UI completion.

The useful future investigation is a **literal nested-text encoder and effect
contract**. A full general-purpose escape decoder is not its prerequisite.

## Evidence and reproduction

The ignored parent scratch `.test-agent/sendtext-boundaries/probe.py` imports
`expand_ansi_c_escapes` and `parse_key_action` inside each cached release's
`+runpy` interpreter. It uses parent-authored inputs, copies no upstream
implementation, and starts no terminal server:

```sh
.test-agent/supported-kitty/0.49.2/app/bin/kitty +runpy 'import runpy; runpy.run_path(".test-agent/sendtext-boundaries/probe.py", run_name="__main__")'
```

The same command with `0.43.1` reproduced all rows above. Existing acquisition
provenance identifies sources `b2cb871254fbdac19a085188813c8dfb258b95cd` and
`7f24dd68c98d8f4712457560f25e8056cce52ec8`, respectively. This is native
parser/expander evidence, not a live child-input journey or whole-version
support claim. The durable cases above remain available without scratch.

## Connections and next discussion

- [Slice-A sketch](sendtext0.glm53m.md): predecessor proposal; the double
  expansion, grammar and API-compatibility claims are corrected here.
- [Capability inventory](inventory0.sol61m.md#nested-action-inventory-and-grammar):
  the existing distinction between RC payloads and nested parser grammar.
- [Support policy](/doc/usage/support.md): exact-profile evidence remains the
  useful unit for extensions.
- [Design home](../README.md): navigation for the implementation and preserved
  proposals.

After implementation review, discuss a send-text capability epic with the
human: completed finite types, selected broadcast surface, B's bounded
conversation, C's source acquisition, targeted live evidence, and optional
escape/nested-action research. Establish which are worth tracking before
creating work items; an uncovered possibility is not automatically a task.
The [post-build discussion](sendtext-followthrough0.gpt6a.md) now supplies that
concrete scope and the choices for the human.
