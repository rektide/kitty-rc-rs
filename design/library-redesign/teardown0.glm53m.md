---
type: ImplementationHandoff
title: Legacy teardown disposition and workspace foundation handoff
description: Inventory of every legacy execution path with salvage decisions, persisted-data behavior, restoration priorities for the removed command surface, and the committed foundation handed to the exchange/registry implementation stage.
resource: /design/library-redesign/teardown0.glm53m.md
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec71e81e3ffercjuzWSBbKFI5w"
    title: Teardown ownership: legacy inspection, disposition map, workspace restructure, foundation crates, and handoff authoring
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Accepted registry-first direction governing crate grouping and slice boundaries
  - resource: /design/library-redesign/kubeless-design0.gpt6a.md
    title: Verified exchange semantics and the one outbound protection boundary
  - resource: /design/library-redesign/kubeless-adaptation1.gpt6a.md
    title: Concurrent registry requirements the next implementation stage must satisfy
  - resource: /design/upstream-kitty/evolution0.gpt6a.md
    title: Defect inventory and upstream wire facts used to classify salvage
  - resource: "https://github.com/kovidgoyal/kitty/tree/5bf3c2dc676efd5c4b4526eb135c922701565707"
    title: Pinned upstream archive directly inspected for envelope, encryption, framing, action, send-key, and ls payload facts
  - resource: /src/client.rs
    title: Removed legacy connection, implicit credential lookup, double-encryption send path (predecessor e78489cb)
  - resource: /src/protocol.rs
    title: Removed mixed envelope representation and heuristic generic chunking (predecessor e78489cb)
  - resource: /src/commands/action.rs
    title: Removed repetitive action wrappers with send_key/as-args wire defect (predecessor e78489cb)
  - resource: /src/bin/kitty-pubkey-db.rs
    title: Removed self-contained store CLI whose behavior moves to access/key-store plus apps/cli (predecessor e78489cb)
verification:
  scope: Documentary inventory written before and updated after the teardown it describes; code claims verified by direct reads at predecessor e78489cb and upstream archive reads
  evidence: Upstream facts pinned in the source list were re-verified in this session; gates and their results are recorded in the final section
  limits: No live kitty interoperability run in this stage; restoration notes for drifted commands describe upstream contracts, not newly implemented behavior
---

# Legacy teardown disposition and handoff

## 1. Standing, authorization, and recovery

The human authorized a broad teardown followed by rebuilding on the accepted
[draft 1](draft1.gpt6a.md) direction: "a g to probably delete a crap ton is
probably the first right start... then start into it with sol m, building".
This document is the teardown stage's owned record: what is removed, what is
refactored into new crates, what functionality leaves the source tree but stays
in scope for restoration, and the exact handoff the next stage (Sol: first
end-to-end typed exchange plus the concurrent registry slice) builds on.

No consumer compatibility obligations exist, including zooming-kittens. No
pushes occur. Nothing here accepts or finalizes the new public API; the
foundation is deliberately smaller than the full draft-1 map.

**Recovery point.** The teardown begins from predecessor commit `e78489cb`
("Synthesize registry-first architecture after lifecycle and control
discussion", working copy `17cf2999`, empty). Any removed file can be recovered
with `jj restore --from e78489cb <path>` or browsed via `jj file list -r e78489cb`.
The removal commits name their paths explicitly.

## 2. Defects that justify removal rather than repair

Each removed execution-path file is classified against defects verified by the
[upstream evolution report](../upstream-kitty/evolution0.gpt6a.md#6-existing-local-defects-are-not-evidence-of-upstream-breakade)
and re-confirmed this session. The architecture, not individual lines, is the
problem: typed inputs collapsed into an untyped message, message fields
independently controlled response behavior, and the caller chose the response
parser afterward.

| Legacy location (at e78489cb) | Defect | Disposition |
| --- | --- | --- |
| `src/client.rs:240-249` `send_all` | Encrypts, then `send()` encrypts again — authenticated sends are double-encrypted | Removed; one protection boundary lives in `kitty-rc-crypto` |
| `src/client.rs:117-142` | Spawns `kitty-pubkey-db` subprocess inside `connect` and implicitly resolves credentials | Removed; no implicit lookup or helper subprocess in new code |
| `src/client.rs:262-267` `send_command` | Sends without reading any reply; `execute` (235-238) always reads even for `no_response` | Removed; exchange ownership is the next stage's core |
| `src/client.rs:206-233` `receive` | Fresh per-call buffer, suffix-scan only, per-read timeout reset; coalesced/partial frames and cancellation discard state | Removed; `kitty-rc-frame` owns persistent bounded decoding with surplus retention |
| `src/client.rs:235-238` | Decoded `ok:false` returned as `Ok`; `LsCommand::parse_response` maps missing data to empty success | Removed; response classification (`ResponseEnvelope::classify`) and `ls_from_response` make missing data an error |
| `src/protocol.rs:98-165` | Any payload string over 4096 chars becomes generic `data` chunks, dropping other payload fields; lossy UTF-8; `chunk_num` is not an upstream field | Removed; streaming is command-specific and belongs to audited command profiles |
| `src/protocol.rs:21-22` | `async_id` serialized without serde rename; wire field is `async`. Its round-trip test proved self-consistency only | Replaced by `PlainRequest` with `#[serde(rename = "async")]` and upstream-derived fixtures |
| `src/protocol.rs:180-180` | Version literals scattered (`[0,43,1]`, `[0,14,2]`) with no policy home | Replaced by `ProtocolVersion` and one declared `SUPPORTED` constant in `kitty-rc-schema` |
| `src/commands/action.rs:18-25` | Wraps ~65 mappable actions but sends command `send_key` with a structured `{action, args}` payload; upstream `action` command takes command `action` with payload `{action: "goto_tab 1", self, match_window}` | Removed; inventory preserved in §5 for typed action-profile restoration |
| `src/commands/window.rs` `SendKeyCommand.keys: String`, `process.rs` string `args`/map `env`, boolean `copy_env`, unrenamed `window_type` | Upstream uses list-valued `keys`, launch `args` list, string-list `env`/`var`, list `copy_env`, wire field `type` | Removed; per-command restoration notes in §5 |
| `src/encryption.rs:52-63` | Accepts any key ≥ 32 bytes, silently truncating | Replaced by exact-length parsing in `kitty-rc-crypto` |
| `src/encryption.rs:73-96` | Implicit `KITTY_PUBLIC_KEY`/`~/.config/kitty/key.pub` reads inside crypto | Removed; crypto receives explicit inputs only |
| `src/encryption.rs:126-132` | Emits stray `"version": "0.43.1"` string (discarded by client reconstruction) | Removed; `ProtectedEnvelope.version` is the protocol array |
| `src/encryption.rs` tests | Mutate process env globals (`unsafe { std::env::set_var }`), race-prone | Replaced by pure-input crypto tests with an independent decrypt oracle |
| `src/client.rs` tests, `protocol.rs` chunk tests, command `to_message` tests | Mostly mirror serializers; the async-id test enshrined the wrong wire name | Replaced by upstream-fixture tests; mirror tests not migrated |

Additional removals on standing policy: examples with hardcoded sockets and a
committed real password (`examples/test_font.rs`, `examples/test-encrypted-simple.rs`),
a stub CLI whose subcommands print "not yet implemented"
(`src/main.rs`), a duplicate of that stub's one working path
(`src/bin/list_windows.rs`), and the automatic cleanup-on-add epoch machinery in
the pubkey store (automatic prune removed; explicit `cleanup` remains).

## 3. Disposition map

Legend: **preserve** (kept in tree), **refactor** (semantics moved into a new
crate, design corrected), **remove** (deleted; scope preserved in §5 if not
otherwise covered).

| Path | Decision | Destination / notes |
| --- | --- | --- |
| `src/client.rs` | remove | Exchange/transport deliberately absent this stage; Sol owns `execution/exchange` + `execution/tokio` |
| `src/command.rs` | remove | Builder ceremony for the removed message type |
| `src/protocol.rs` | refactor | Envelope types → `protocol/schema`; framing → `protocol/frame`; chunking dropped entirely |
| `src/commands/mod.rs` | remove | Module tree replaced by `protocol/commands` (exemplar only, this stage) |
| `src/commands/window.rs` | refactor (partial) | `ls` payload + all output models (WindowInfo family) → `kitty-rc-commands`; the other 15 builders removed, inventoried in §5 |
| `src/commands/action.rs` | remove | Inventory in §5; nested action profiles are future `protocol/commands` work |
| `src/commands/process.rs`, `style.rs`, `tab.rs`, `layout.rs` | remove | Inventory in §5 |
| `src/encryption.rs` | refactor | Primitive chain (X25519→SHA-256→AES-256-GCM, RFC 1924 base85) → `kitty-rc-crypto` as pure functions with explicit inputs + decrypt oracle |
| `src/error.rs` | remove | Per-crate error enums in the new crates; no grab-bag error crate |
| `src/lib.rs` | remove | No facade crate until there is something to facade (Sol) |
| `src/main.rs` | remove | Dead/stub CLI |
| `src/bin/list_windows.rs` | remove | Duplicate diagnostic |
| `src/bin/kitty-pubkey-db.rs` | refactor | Record model/parse/render → `kitty-rc-key-store`; CLI behavior → `apps/cli` binary `kitty-pubkey-db` |
| `examples/*` | remove | Diagnostics enshrining wrong payloads; one contains a committed password |
| `scripts/enable-rc.sh` | preserve | Useful deliberate local kitty setup for later live tests; unrelated to library API |
| `doc/*.md`, `design/**` | preserve | Historical evidence and accepted direction; this file is the only new design-tree addition |
| `LICENSE`, `AGENTS.md`, `.beads`, `.test-agent/` | preserve | License, local agent boilerplate, work-tracking, gitignored scratch |
| `Cargo.toml`, `Cargo.lock` | refactor | Root becomes a virtual workspace manifest; lock regenerated with the new crates |
| `README.md` | refactor | Rewritten to truthful current standing in the final stage commit |

## 4. Persisted user data and the key store

The pubkey store is user data and survives with its format intact.

- Format (unchanged): TSV lines `pid\twindow_id\tpubkey\ttimestamp`, file
  `$XDG_STATE_HOME/kitty/pubkey.tsv` (default `~/.local/state/kitty/pubkey.tsv`);
  `window_id` may be empty; records without timestamp parse with `0`.
- Library behavior (`kitty-rc-key-store`): parse records, **report malformed
  lines with line numbers instead of silently dropping them**, render records,
  and select the newest record for a PID by timestamp. No filesystem access, no
  liveness checks, no deletion.
- CLI behavior (`apps/cli`, binary name `kitty-pubkey-db`): `init`, `add`, `get`,
  `cleanup` preserved. `add` still falls back to `KITTY_PID`/`KITTY_PUBLIC_KEY`/
  `KITTY_WINDOW_ID`. The removed legacy behavior is the **automatic
  cleanup-on-add epoch trigger** (`pubkey-check.epoch`, 24h re-check): automatic
  pruning contradicts the no-automatic-prune standing rule. `cleanup` remains an
  explicit user action (keeps records whose `/proc/<pid>` exists).
- The `epoch` file is no longer written. Existing `pubkey.tsv` and even existing
  epoch files are left untouched on disk; nothing erases user data.
- Design note for the later `access/{credentials,discovery}` slice: PID-guessed
  sockets and stored keys are lookup hints with provenance, not endpoint
  identity; a store read must not become a hidden downgrade path.

## 5. Restoration inventory for the command catalog

Functionality that leaves the tree in this stage but remains in scope. The next
stages restore these as typed commands against upstream fixtures — never by
reusing the old serializers as oracles. Wire facts cited here were re-verified
against the pinned upstream archive at `5bf3c2dc` (kitty HEAD, docs unchanged
since 0.43.1 for these surfaces).

### 5.1 RC commands removed (builder existed)

| Upstream command | Legacy builder | Known wire drift to fix on restoration |
| --- | --- | --- |
| `ls` | `LsCommand` | None known; payload `{all_env_vars, match, match_tab, self?}`; restored now in `kitty-rc-commands` |
| `send-text` | `SendTextCommand` | Upstream `data` may repeat per chunk; bracketed_paste values differ; audit against `send_text.py` |
| `send-key` | `SendKeyCommand` | `keys` must be a **list** of strings (`{'keys': args, ...}`), not one string |
| `close-window` | `CloseWindowCommand` | Check `self`/`match` against `close_window.py` |
| `create-marker` / `remove-marker` | builders existed | Audit |
| `detach-window` / `detach-tab` | builders existed | Audit (HEAD adds `location`, `create_if_missing`, `tab_title`) |
| `focus-window` | builder existed | Audit |
| `get-text` | `GetTextCommand` | Audit; 0.48 adds `alternate`, `alternate_scrollback`; payload includes `extent` |
| `new-window` | `NewWindowCommand` | Audit |
| `resize-window` | `ResizeWindowCommand` | Amount is upstream pair `[number-or-start/end, unit]`; fractional lines since 0.46; HEAD adds `fraction` |
| `scroll-window` | `ScrollWindowCommand` | Same amount-pair shape |
| `select-window` | `SelectWindowCommand` | Async command: envelope field `async`; server response carries no correlation id |
| `set-window-logo` / `set-window-title` | builders existed | Audit |
| `run` | `RunCommand` | `cmdline` is a list; streaming input is base64 chunks with per-command sizes (Go/Python readers use 4096 raw bytes), stream id, `async` id, explicit input terminator; output is buffered stdout/stderr in async `data` |
| `kitten` | `KittenCommand` | `args` is a list; handler runs on first matching window; returns only string/boolean results |
| `launch` | `LaunchCommand` | `args` list; env/var string lists; `copy_env` list; wire field `type`; 0.47 adds `os_window_position`; HEAD adds docks |
| `env` | `EnvCommand` | Upstream env is string-list based CLI; audit payload |
| `set-user-vars` | builder existed | Audit |
| `load-config` | `LoadConfigCommand` | Audit; affects broad server state |
| `resize-os-window` | builder existed | Audit |
| `disable-ligatures` | builder existed | Audit |
| `signal-child` | builder existed | Audit |
| `set-background-opacity` | builder existed | Audit |
| `set-background-image` | `SetBackgroundImageCommand` | Streaming upload contract: first nonempty data frame requests stream acceptance (`stream:true` + `stream_id`), await `{"ok":true,"stream":true}`; later frames same stream id without re-handshake; empty `data` frame ends input; `data:"index:N"`/`"-"` are finite ops; ~512 raw PNG bytes per chunk documented |
| `set-colors` / `get-colors` | builders existed | Audit |
| `set-font-size` | `SetFontSizeCommand` | Payload `{size, all, increment_op}`; scope semantics (active/all windows) differ per `Boss.change_font_size`; 0.49 raises zoom ceiling to `max(configured*10, 256)` |
| `set-spacing` / `set-tab-color` | builders existed | Audit |
| `close-tab` / `detach-tab` / `focus-tab` / `set-tab-title` | builders existed | Audit |
| `goto-layout` / `last-used-layout` / `set-enabled-layouts` | builders existed | Audit |
| `action` | (misimplemented, §2) | Payload `{action: "goto_tab 1", self, match_window}` — one string with args joined by spaces, command `action` |
| *not previously present* | — | Stable upstream additions worth typed builders later: `screenshot` (0.49, async, PNG string or server path), `set-os-window-title` (0.49), `get-text` alternates (0.48); HEAD-only: `resize-window-edge`, dock launches, named workspaces |

### 5.2 Nested mappable-action vocabulary (from removed `src/commands/action.rs`)

The marker types enumerated the mappable-action surface kitty exposes through
the `action` command. Names preserved as restoration scope (typo "mouse_click_url"
as written in legacy source; verify against upstream action tables when typing):
quit; new_tab, close_tab, next_tab, previous_tab, goto_tab, set_tab_title,
detach_tab, move_tab_forward, move_tab_backward; new_window, new_window_with_cwd,
close_window, close_window_with_confirmation, next_window, previous_window,
move_window_forward, move_window_backward, nth_window, first_window,
set_window_title, resize_window, reset_window_sizes, move_window,
neighboring_window, toggle_fullscreen, toggle_maximized; copy_to_clipboard,
paste, paste_from_clipboard, paste_selection, clear_selection,
copy_or_interrupt; goto_layout, next_layout, last_used_layout, toggle_layout;
scroll_line_up, scroll_line_down, scroll_page_up, scroll_page_down, scroll_home,
scroll_end, scroll_to_prompt, show_scrollback; create_marker, remove_marker,
scroll_to_mark, toggle_marker; mouse_click_url, mouse_selection;
debug_config, dump_lines_with_attrs; send_key, send_text, kitten, launch,
signal_child, clear_terminal, show_kitty_doc, edit_config_file,
set_background_opacity, change_font_size, load_config_file, set_colors.

Per draft 1 §6: an action acknowledgment may acknowledge dispatch rather than
all later effects; known actions deserve typed profiles with scope/ordering;
arbitrary action strings keep an escape hatch without inferred semantics.

### 5.3 Output models preserved

`WindowInfo`, `TabInfo`, `OsInstance`, `ProcessInfo` and the layout/tab-group
helper structs survive in `kitty-rc-commands::outputs` (tolerant of unknown
additive fields). Upstream serializer additions to consider when expanding
coverage: window `neighbors` (0.45), `title_overridden` (0.46.2),
`session_name`/`last_focused_at`/`needs_attention`/`has_activity_since_last_focus`
(0.47), OS window `active_tab_history` (0.45), HEAD-only `program_status`.
`last_focused_at` is monotonic-clock; never compare across kitty processes.

## 6. Workspace established this stage

Root becomes a virtual Cargo workspace. Only crates with real code this stage
exist; the rest of the draft-1 map is deliberately absent, not stubbed.

| Crate | Owns | Key interfaces (this stage) |
| --- | --- | --- |
| `protocol/schema` (`kitty-rc-schema`) | Wire envelope types, version vocabulary, response classification | `PlainRequest` (with `async` rename), `ProtectedEnvelope`, `ResponseEnvelope` + `classify`, `ProtocolVersion` + `SUPPORTED` `[0,43,1]` |
| `protocol/frame` (`kitty-rc-frame`) | Incremental DCS framing, exact boundaries, surplus retention, bounds | `FrameDecoder::{new(max_payload), extend, next_frame, finish}`, `frame(payload)` encoder |
| `protocol/commands` (`kitty-rc-commands`) | Typed command exemplar and output models | `Ls` (`to_request`), `outputs::*`, `ls_from_response` (missing data is an error) |
| `access/crypto` (`kitty-rc-crypto`) | One-envelope protection, key parsing, pure inputs | `parse_server_public_key` (exact 32 bytes, `1:` prefix), `encrypt_request(value, password, timestamp_ns, key, rng)`, `decrypt_envelope` oracle |
| `access/key-store` (`kitty-rc-key-store`) | Persisted pubkey record format | `PubkeyRecord`, `parse_store` (malformed reported), `render_record`, `latest_for_pid` |
| `apps/cli` (`kitty-rc-cli`) | Key-store utility binary | `kitty-pubkey-db` `init`/`add`/`get`/`cleanup` |

Deliberately absent until they carry real code: `execution/{exchange,tokio}`,
`access/{endpoint,discovery,credentials}`, `control/registry`,
`observation/listing`, `client` facade. Verified upstream facts baked into the
foundation: DCS framing `\x1bP@kitty-cmd … \x1b\\`; `no_response` semantics;
stream acknowledgment `{"ok":true,"stream":true}`; encrypted envelope fields
`{version:[…], iv, tag, pubkey, encrypted}` (RFC 1924 base85, 12-byte IV,
16-byte tag, 32-byte keys; `enc_proto` only when protocol ≠ 1); password +
nanosecond timestamp injected at the top level of the plaintext command; 5-minute
server timestamp window; server rejects client major/minor newer than itself.

## 7. Handoff to the exchange/registry stage (Sol)

Build **the first end-to-end typed exchange and the concurrent registry
together**, per draft 1 §12: registry-first, not deferred. Concretely:

1. `execution/exchange` (runtime-independent) + `execution/tokio` (socket
   owner): one exchange per connection; whole-operation deadline; write-start
   evidence; transport retirement on cancellation/ambiguity; no-response mode
   retires the connection; coalesced frames processed in order via
   `kitty-rc-frame`. The outbound path is exactly:
   `typed payload -> PlainRequest -> plain OR encrypt once (kitty-rc-crypto) -> frame -> bytes`.
   Transport code must never receive a value it could encrypt twice.
2. `control/registry`: accepted operations owned by the registry (survive
   dropped waiters), bounded admission/leases, explicit cancel, driver
   replacement without automatic replay. Three-connection isolation fixture
   from draft 1 §3.
3. Use `Ls` in `kitty-rc-commands` as the first typed command; extend the
   catalog from §5 with upstream fixtures, not legacy serializers.
4. Setup stays composition (draft 1 §7): explicit endpoint, credentials, and
   key provenance; no implicit environment or subprocess lookup anywhere.

Open decisions deliberately left: admission commit mechanism, custody component
versus retained host, capacity/fairness defaults, retained-output ownership,
Rust minimum (workspace currently `1.85`, untested below), kitty support matrix
(declared `[0,43,1]`, untested against real servers), whether `kitty_window_id`
needs string tolerance (currently `u64` per Go client), and ls `self` payload
behavior (official client message_to_kitty omits it; server reads it).

## 8. Verification record

Filled at the end of the teardown stage; commands and results in the final
commit's message and here. (Pre-commit state of this section: pending.)

Risks carried forward: no live-kitty conformance yet for the foundation crates
(upstream facts are source-verified only); `protocol/commands` carries one
exemplar by design, so catalog breadth is intentionally a later stage's work.
