---
type: DesignProposal
title: Send-text deferred surface - typed data encodings, escape grammar, epic sketch
description: Concrete Slice-A proposal for send-text data encodings, broadcast sessions, ANSI-C escape expansion and the nested paste action, with the immediate/further epic breakdown; execution conversation and CLI fork as separate work.
resource: /design/library-redesign/capabilities/sendtext0.glm53m.md
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-28
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Scoped the deferred-surface direction from the docs corpus and drafted this Slice-A sketch with epic/ticket breakdown for human discussion
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Deferred send-text/broadcast/clipboard boundaries this sketch picks up
  - resource: /protocol/commands/src/window.rs
    title: Current finite literal SendText constructor being extended
  - resource: /protocol/commands/src/actions.rs
    title: SimpleAction clipboard vocabulary and KnownAction remainder grammar
  - resource: /protocol/commands/src/catalog.rs
    title: send-text FiniteSubsetUploadDeferred coverage metadata to revise
  - resource: "file:///home/rektide/archive/kovidgoyal/kitty/kitty/rc/send_text.py"
    title: Upstream client generator, data encodings and broadcast session lifecycle read at archived HEAD
  - resource: "file:///home/rektide/archive/kovidgoyal/kitty/kitty/data-types.c"
    title: expand_ansi_c_escapes state machine with exact digit limits and default handling
  - resource: "file:///home/rektide/archive/kovidgoyal/kitty/kitty_tests/datatypes.py"
    title: Upstream escape-expansion test vectors adopted for verification
  - resource: "file:///home/rektide/archive/kovidgoyal/kitty/kitty/options/utils.py"
    title: Nested send_text/paste action parsers and their error fallback policy
  - resource: /design/publishing/verify-exposure0.glm53m.md
    title: Published-crate context; commands crate is already public surface
---

# Send-text deferred surface: Slice A sketch

**Situation.** The redesign roadmap is closed and the first crates.io wave is
published (`kitty-rc` 0.7.0 facade plus the 0.1.0/0.2.0 concern crates,
2026-10-27). The chosen next direction is the deferred command surface from
the [capability inventory](inventory0.sol61m.md#deferred-and-version-gated-work):
send-text beyond one literal submission, and the deferred nested clipboard
`paste` remainder. The `RunEmpty` lane question named in the same section is
already settled — the [uploads wave](../uploads/implementation0.sol61xh.md)
made every `run`, including empty input, own the same accumulator lane, and no
bypass remains.

**Present ask.** This wave sketches **Slice A only** — the pure command-types
layer in `kitty-rc-commands` — concretely enough to discuss. The execution
conversation (Slice B) and the CLI subcommand (Slice C) fork into their own
tickets; this document proposes the epic shape and the immediate/further split.
`kitty-key:` encoding is deliberately deferred. Typing the nested `send_text`
action grammar is *not* proposed here; the human suggested a separately
commissioned assessment wave (Astra high) for that world, likely after the core
lands.

## Upstream contract (source-pinned)

Read at the archived upstream checkout pinned at HEAD
`5bf3c2dc676efd5c4b4526eb135c922701565707` (0.50dev) — the same HEAD the
inventory inspected; the send-text protocol spec is unchanged across the
audited stable range.

### `data` encodings ([send_text.py](file:///home/rektide/archive/kovidgoyal/kitty/kitty/rc/send_text.py))

| Wire prefix | Meaning | Server behavior |
|---|---|---|
| `text:<utf8>` | literal text | written to each matched window |
| `base64:<std>` | raw bytes | base64-decoded, written to each matched window |
| `kitty-key:<b64>` | window-system key event | decoded and encoded per window (deferred here) |
| `session:start` / `session:end` | broadcast session lifecycle | cursor-render bookkeeping in a server-global `sessions_map` keyed by `session_id` |

`session_id` is **orthogonal** to the data encoding: any data payload carrying
a `session_id` creates/updates that session and marks its windows; `start`/`end`
add/remove windows and the unfocused-cursor rendering. Match fields
(`match`, `match_tab`, `all`, `exclude_active`) select windows per payload;
`bracketed_paste` wraps written bytes when enabled (or `auto` + program mode).

### Reference client shape

- Positional text goes through ANSI-C escape expansion, then **1024-raw-byte
  base64 chunks**, each chunk a complete send-text command emitted
  sequentially by a generator (`disallow_responses` — no reply ever).
- `--stdin` on a pipe and `--from-file` produce the same base64 chunks from
  raw bytes; tty stdin is an interactive `text:` mode ending on `^D` — a CLI
  concern, not a library one.

### ANSI-C escape grammar ([data-types.c](file:///home/rektide/archive/kovidgoyal/kitty/kitty/data-types.c))

Exact state machine, implemented from the documented bash ANSI-C semantics:

| Escape | Expansion |
|---|---|
| `\a \b \e \E \f \n \r \t \v` | 7, 8, 27, 27, 12, 10, 13, 9, 11 |
| `\cX` | `X & 0x1f` (so `\c ` → NUL); trailing `\c` at end-of-input stays literal `\c` |
| `\xHH` | up to **2** hex digits |
| `\uHHHH` | up to **4** hex digits |
| `\UHHHHHHHH` | up to **8** hex digits |
| `\NNN` | up to **3** octal digits |
| `\\ \' \" \?` | literal character |
| any other `\z` | **preserved** as `\z` (backslash kept) |

Numeric escapes end at the digit limit or the first non-matching digit (that
digit is then reprocessed as normal input). Values outside the Unicode scalar
range (including surrogates) make upstream raise; its nested-action parsers
catch, log, and substitute an empty string ([utils.py](file:///home/rektide/archive/kovidgoyal/kitty/kitty/options/utils.py)).

**Wire subtlety worth documenting in the API:** expansion operates on Unicode;
`\xff` becomes U+00FF, which is **two UTF-8 bytes** on the `text:`/wire path.
Raw non-UTF-8 bytes require the `base64:` encoding. Our constructors should
make that distinction structural.

**GPL boundary:** no upstream code is translated. The grammar is implemented
from its documented semantics; conformance is checked against upstream
executed at the pinned revision through the existing source-oracle methodology.

## Proposed API (concrete, for discussion)

### 1. Escape module — `protocol/commands/src/ansi_c.rs`

```rust
/// Expands bash ANSI-C quoting escapes per the documented grammar.
/// Success cases match upstream expand_ansi_c_escapes exactly;
/// invalid scalar values error where upstream raises.
pub fn expand_ansi_c_escapes(src: &str) -> Result<Cow<'_, str>, AnsiCEscapeError>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("escape expands to non-scalar value U+{0:X}")]
pub struct AnsiCEscapeError(pub u32);
```

### 2. `SendText` data encodings — [window.rs](../../../protocol/commands/src/window.rs)

The enum mirrors **wire encodings**, not input modes:

```rust
pub enum SendTextData {
    /// `text:` literal UTF-8; no escape interpretation.
    Text(String),
    /// `base64:` arbitrary raw bytes.
    Bytes(Vec<u8>),
    /// `session:start` / `session:end` broadcast lifecycle.
    Session(BroadcastSession),
}

pub enum BroadcastSession { Start, End }

pub struct SendText {
    pub data: SendTextData,
    /// Any data variant may carry one; server creates/updates the session.
    pub session_id: Option<String>,
    pub windows: WindowSet,
    pub exclude_active: bool,
    pub bracketed_paste: BracketedPaste,
}
```

Entrances (existing `SendText::new` stays literal `Text`, so current callers
are unchanged):

```rust
SendText::new("literal")                    // Text
SendText::escaped(r"a\eb")?                 // ANSI-C expansion HERE -> Text(expanded)
SendText::bytes(vec![0xff, 0x00])           // Bytes (raw, no UTF-8 requirement)
SendText::session_start(id) / ::session_end(id)
```

**Why constructor-time expansion (recommendation):** `Command::request()` is
infallible, so a fallible interpretation must surface at construction.
Storing an "escaped" variant would either store unexpanded source (ambiguous
state) or expanded text (indistinguishable from `Text` on the wire). The
`escaped` entrance is a checked convenience producing the identical wire form.

### 3. Nested clipboard `paste` — [actions.rs](../../../protocol/commands/src/actions.rs)

```rust
pub enum KnownAction {
    // ...
    /// `paste <expanded remainder>` — ANSI-C escapes interpreted here.
    Paste(String),
}
// wire(): format!("paste {text}")
// Constructor: KnownAction::paste(src)? expands via ansi_c::expand_ansi_c_escapes.
```

**Divergence to accept deliberately:** upstream's config-parser fallback is
log-error-and-send-empty; a typed constructor fails loudly instead, because
the caller is composing intent directly, not parsing a config file.

### 4. Catalog and documentation updates

- [catalog.rs](../../../protocol/commands/src/catalog.rs) `send-text` row:
  `FiniteSubsetUploadDeferred` note becomes "literal/escaped/bytes/session
  finite; chunked conversation deferred" (coverage value unchanged until B).
- README command-construction section gains an escaped/bytes example;
  [support policy](../../../doc/usage/support.md) gains the escape/base64
  profile-gating note (typed constructor existence is not live evidence).
- Source-oracle: extend the pinned-source methodology with escape-expansion
  vectors executed against the upstream checkout (no code copied).

## Verification plan

- Unit tests adopting the upstream vector table
  ([datatypes.py](file:///home/rektide/archive/kovidgoyal/kitty/kitty_tests/datatypes.py)
  `test_expand_ansi_c_escapes`, 16 cases) plus our additions: `\U0011FFFF`
  and `\ud800` error cases, trailing `\c`, unknown-escape preservation,
  digit-limit boundaries, `\cX` control masking.
- Existing gates: `cargo nextest run`, clippy, fmt; `kitty-rc-verify source`
  after the oracle extension.

## Design decisions to settle

1. **Expansion point** — constructor-time into `Text` (recommended, argued
   above) vs a stored `Escaped` variant.
2. **Error policy** — fail loudly on invalid scalars (recommended) vs
   upstream's nested-parser silent fallback.
3. **Session shape** — `SendTextData::Session` variant + orthogonal
   `session_id` field (recommended) vs dedicated command types.
4. **Naming** — `SendTextData` vs `TextData`/`SendTextEncoding`;
   module name `ansi_c.rs`.
5. **Chunk constants** — the 1024-byte chunk size belongs to the B
   conversation, not the commands crate (flagged so B does not re-litigate).

## Epic and ticket proposal

Epic `kitty-rc-proto-rs-redesign-capabilities-sendtext` (matches the
`-upload`/`-typed` naming):

| Item | Scope | Standing |
|---|---|---|
| **A. types** | This sketch: escape module, `SendTextData`/sessions, `KnownAction::paste`, catalog/doc updates, oracle vectors | Immediate, this proposal |
| **B. conversation** | Multi-request submission-only conversation: bounded byte source → 1024-byte base64 chunks → sequential send-text requests on one transport; retirement vs reuse decision (submission-only cannot prove a clean reply boundary — retire is the honest default, reference client reuse documented as divergence); registry entrance; **no upload lane** with the no-accumulator rationale recorded | Forked ticket |
| **C. CLI** | `kitty-rc send-text` subcommand: literal/`--stdin` (pipe + tty `^D`)/`--from-file`, match options, bracketed-paste | Forked ticket |
| kitty-key encoding | Window-system key event `data` prefix | Deferred, needs its own audit |
| Nested `send_text` action typing | mode + escaped-text grammar | Deferred; candidate for a commissioned Astra-high assessment wave after core lands |
| Live evidence | send-text → get-text round trip in the private matrix | Further work |

## Cross-references

- [Capability inventory](inventory0.sol61m.md): the deferred boundaries and
  the stable send-text row this implements.
- [Uploads implementation](../uploads/implementation0.sol61xh.md): the
  conversation/lane precedent B will follow and deliberately not need here.
- [Publishing assessment](../../publishing/verify-exposure0.glm53m.md): the
  commands crate is already published surface; these additions will need a
  version bump and review before the next release.
- [Client guide](../../../client/README.md): entrance documentation to extend
  on acceptance.
