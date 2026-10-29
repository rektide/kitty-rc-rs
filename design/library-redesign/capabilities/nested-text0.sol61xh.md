---
type: ImplementationReport
title: Literal nested paste and send_text
description: Two typed nested text actions, invariant-preserving nonempty literals and modes, native full-parser evidence and unchanged dispatch-only retirement semantics.
resource: /design/library-redesign/capabilities/nested-text0.sol61xh.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e7a023118ffeR098xNZniVpVG5"
    title: Implements the human-selected two nested literal conversions, Rust-generated native parser checks and workspace verification under parent stewardship
  - resource: /design/library-redesign/capabilities/sendtext-boundaries0.gpt6a.md
    title: Parent's double-expansion correction and literal-encoding route, not a local decoder specification
  - resource: /protocol/commands/src/actions.rs
    title: Validated text, nonempty keyboard-mode vocabulary, literal constructors and internal canonical encoder
  - resource: /protocol/commands/tests/nested_text.rs
    title: Independent full-wire goldens and explicit opt-in native parser test
  - resource: /protocol/commands/tests/fixtures/nested_text.py
    title: Executes the actual native full parser on Rust-generated actions without translating upstream code
  - resource: /execution/exchange/tests/nested_text.rs
    title: Preparation, retirement, submission suppression and dispatch reply classification checks
  - resource: /doc/usage/support.md
    title: Exact-profile evidence policy; no whole-release or new live-matrix claim
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/utils.py
    title: Inspected full parse_key_action, paste_parse, send_text_parse, parse_send_text_bytes and alias resolution
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/definition.py
    title: Inspected documented normal, application, kitty, combination and all keyboard gates
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/keys.py
    title: Inspected keyboard_mode_name precedence for extended protocol and DECCKM
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/window.py
    title: Inspected send_text, paste, paste_with_actions and paste_text effect boundaries
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/boss.py
    title: Inspected dispatch_action and combine consumption, alias resolution and failure behavior
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/action.py
    title: Inspected first selected window, dispatch-only success and unconsumed Unknown action error
verification:
  scope: Independent Rust wire/preparation tests, actual full native parser in cached kitty 0.43.1 and 0.49.2, and workspace gates
  evidence: 255 workspace tests and 11 doctests passed; strict all-target Clippy, explicit 100-column formatting and owned OKF checks passed; 2970 native full-parser roundtrips plus four empty-shape checks passed
  limits: Parser execution is not alias dispatch, UI, socket or child-input conformance; no new live-support range or minimum release asserted
---

# Literal nested paste and send_text

Built only the human-selected two conversions. No local escape decoder, stream,
CLI acquisition, `kitty-key`, ticket, replay/reuse rule, remote push or release
was added. Parent-owned navigation, READMEs and version coordination remain
separate. Implementation checkpoint: `4ba4835d`.

## API and literal boundary

All public additions live in [actions.rs](/protocol/commands/src/actions.rs):

| Surface | Contract |
| --- | --- |
| `ActionText::new(literal) -> Result<Self, EmptyActionText>` | Reject only empty strings. Private storage, immutable `as_str()`, no unchecked default or deserialization entrance. |
| `KnownAction::paste(literal) -> Result<Self, EmptyActionText>` | Constructs `Paste(ActionText)`; accepts literal Unicode, not native escape source. |
| `KnownAction::send_text(modes, literal) -> Result<Self, EmptyActionText>` | Constructs `SendText { modes: SendTextModes, text: ActionText }`. |
| `SendTextModes` | `All` or any nonempty subset: `Normal`, `Application`, `Kitty`, `NormalApplication`, `NormalKitty`, `ApplicationKitty`, `NormalApplicationKitty`. No empty/duplicate/unknown mode state. |

```rust
use kitty_rc_commands::actions::{Action, KnownAction, SendTextModes};
use kitty_rc_commands::targets::WindowId;

let paste = Action::new(KnownAction::paste(r" \n ").unwrap());
let send = Action::builder()
    .action(KnownAction::send_text(SendTextModes::NormalApplication, "hello\n").unwrap())
    .target(WindowId(7))
    .build();
```

The internal encoder emits **each Unicode scalar as fixed-width `\UXXXXXXXX`**
(lowercase hex). Kitty performs the sole decoding pass. Thus literal `r"\n"`
emits `\U0000005c\U0000006e`, not a newline escape; `" x "` emits
`\U00000020\U00000078\U00000020`. This protects edge whitespace from the
full parser's `strip`/`split` and avoids native octal edge behavior. NUL,
whitespace-only text, noncharacters and all valid Rust scalars remain accepted
without trimming or normalization. Encoding uses ten ASCII bytes per scalar;
no compact/minimal escape representation is promised.

Empty input is a deliberate narrow library restriction: native `"paste "`
parses with **zero arguments**, while `"send_text all "` parses as empty mode
and bytes. Constructors neither emit missing arguments nor exploit malformed
zero-digit escapes or hidden no-ops. Public variants require validated text;
the private-field compile-fail doctest checks that boundary. Existing
`ActionSpec::Raw` remains verbatim caller-authored grammar. Added enum variants
also require downstream exhaustive Rust matches to be updated.

## Source-backed effect contract

At pinned stable `b2cb871254fbdac19a085188813c8dfb258b95cd`:

- **Nested `send_text`** decodes Unicode to UTF-8, then checks the selected
  window's keyboard mode. Normal/application are DECCKM states; enabled kitty
  keyboard encoding takes precedence. Combinations serialize in
  normal/application/kitty order; `All` serializes as `all`.
- `Window.send_text` returns passthrough (`True`) without writing for a mode
  mismatch or empty native text, and returns `False` after requesting a write.
  `Boss.dispatch_action` treats passthrough as unconsumed; `combine` can return
  false and RC action then reports **`Unknown action: ...`**, even for a known
  action name. No local mode probing or fabricated success is introduced.
- **Paste** invokes `paste_with_actions`: filtering, confirmations, dangerous
  control handling and newline settings can change, defer or cancel effects.
  `paste_text` additionally sanitizes bracketed paste or normalizes newlines.
  Parser-literal preservation is **not** byte-exact child input.
- Alias configuration can replace dispatch. Outer `Action.target` still selects
  one dispatch window (the first native match), not RC `SendText`'s multi-window
  scope. Normal actions await a dispatch acknowledgment; optional
  `Submission(action)` suppresses it. **Both retire**, neither grants replay
  safety, and neither acknowledgment nor submission establishes child receipt.

These are source-audit conclusions, not newly executed dispatch/child journeys.

## Evidence and reproduction

[Wire tests](/protocol/commands/tests/nested_text.rs) specify complete strings
and payloads independently: backslash/newline double-expansion regressions,
octal-looking and malformed-looking literal text, edge/Unicode whitespace,
NUL/control/scalar boundaries, all eight mode choices, validated variant
construction, all outer target forms and unchanged raw input.
[Exchange tests](/execution/exchange/tests/nested_text.rs) check reply versus
suppressed preparation, retirement, no upload metadata, unit dispatch success,
honest local submission and preservation of native `Unknown action` failures.

The ignored native test sends **actual Rust-generated action strings** and
literal expected values to kitty's `+runpy` interpreter. The tiny
[fixture](/protocol/commands/tests/fixtures/nested_text.py) calls real
`parse_key_action`, including outer stripping/splitting and the native C
expander; it contains no decoder mirror or copied upstream implementation.

| Cached release | Acquisition source revision | Full-parser checks |
| --- | --- | --- |
| 0.43.1 | `7f24dd68c98d8f4712457560f25e8056cce52ec8` | 165 paste + 1,320 send_text roundtrips; two empty shapes |
| 0.49.2 | `b2cb871254fbdac19a085188813c8dfb258b95cd` | Same; all passed |

The 165 literals cover all ASCII, Unicode whitespace, scalar boundaries,
backslashes, controls and mixed data; each send-text literal runs all eight
mode choices. Reproduce from the repository root:

```sh
KITTY_RC_NATIVE_KITTY="$PWD/.test-agent/supported-kitty/0.43.1/app/bin/kitty" \
  mise exec -- cargo test -p kitty-rc-commands --test nested_text native_full_parser -- --ignored --nocapture
```

Repeat with `0.49.2`. Executables must be explicitly supplied; ordinary tests
skip this check. No terminal server, download, user configuration mutation,
alias dispatcher or child is involved. This evidence does not extend the
[live matrix or support policy](/doc/usage/support.md).

Converged gates (all through `mise exec --`):

| Check | Result |
| --- | --- |
| Targeted commands nextest / commands doctests / new exchange tests | 56 / 8 / 2 passed; native opt-in skipped by ordinary nextest |
| `cargo nextest run --workspace` | 255 passed; one intentional native skip |
| `cargo test --workspace --doc` | 11 passed, including compile-fail invariants |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo fmt --all -- --config max_width=100 --check` | Passed; explicit native project width avoids inherited 160-column churn |

The [parent boundary assessment](sendtext-boundaries0.gpt6a.md) supplies the
double-expansion correction this implementation follows. The separate
[RC finite text/binary core](sendtext-core0.sol61xh.md) and
[broadcast messages](sendtext-broadcast0.sol61xh.md) retain their own contracts;
neither is extended by these nested conversions.
