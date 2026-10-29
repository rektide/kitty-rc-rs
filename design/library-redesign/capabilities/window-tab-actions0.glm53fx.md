---
type: ImplementationReport
title: Window/tab helpers and named clipboard buffers
description: Ten converted nested actions across zero-argument helpers, cwd-inheriting process kinds and raw-remainder buffer names, with validation mirroring the native parser's edge behavior.
resource: /design/library-redesign/capabilities/window-tab-actions0.glm53fx.md
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e79b8beeaffeJKwxQA8AlDHWiu"
    title: Implements the human-selected window/tab helper and clipboard buffer conversion slice with native grammar verification and workspace gates
  - resource: /design/library-redesign/capabilities/conversion-coverage0.gpt6a.md
    title: Commissioning review whose conventional-candidate table selected this family
  - resource: /protocol/commands/src/actions.rs
    title: Extended SimpleAction, NewProcess, BufferName and KnownAction wire encoding
  - resource: /protocol/commands/tests/window_tab_actions.rs
    title: Wire goldens, remainder-grammar distinction, validation and opt-in native parser test
  - resource: /protocol/commands/tests/fixtures/window_tab_actions.py
    title: Executes the native full parser on Rust-generated actions in explicit +runpy
  - resource: /client/README.md
    title: Added the named clipboard buffers usage subsection
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/tabs.py
    title: Inspected close_other_windows_in_tab and move_window_to_top implementations
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/boss.py
    title: Inspected close_other_tabs_in_os_window, new_tab_with_cwd, new_os_window_with_cwd, copy_to_buffer and paste_from_buffer handlers plus dispatch_action ordering
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/utils.py
    title: Inspected remainder parsers, shlex_parse membership and parse_key_action strip/split
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/window.py
    title: Inspected copy_ansi_to_clipboard, copy_last_command_output, show_last_command_output and CwdRequest cwd sourcing
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/actions.py
    title: Inspected get_all_actions registration over Window/Tab/Boss @ac decorators
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/action.py
    title: Inspected the RC action command's single-string combine dispatch
verification:
  scope: Independent Rust wire goldens, actual native full parser and documented-action registry on cached kitty 0.43.1/0.49.2, and workspace gates
  evidence: 274 workspace tests plus 14 doctests passed with strict all-target Clippy and 100-column formatting; 38 library-generated strings and 49 scratch-oracle checks (32 wire, 10 registry membership, 7 edge shapes) passed against the native parser
  limits: Parser execution and registry enumeration are not alias dispatch, UI, clipboard, process-creation or child-effect conformance; no live-support range or minimum release asserted
---

# Window/tab helpers and named clipboard buffers

Built the human-selected families from the
[coverage review](conversion-coverage0.gpt6a.md): window/tab helpers,
cwd-inheriting process kinds and the named clipboard buffer pair. No ordinal
`second_window`..`tenth_window` wrappers (existing `KnownAction::NthWindow`
already expresses them), no `scroll_prompt_to_top` option modeling, no nested
`send_key`, launch-option grammars or output/listing fields. Implementation
checkpoint: `f7831240`.

## What was converted

| Surface | Wire name(s) | Native anchor (kitty 0.49.2) |
| --- | --- | --- |
| `SimpleAction::CloseOtherWindowsInTab` | `close_other_windows_in_tab` | `tabs.py` Tab method, zero-arg |
| `SimpleAction::MoveWindowToTop` | `move_window_to_top` | `tabs.py` Tab method, zero-arg |
| `SimpleAction::CloseOtherTabsInOsWindow` | `close_other_tabs_in_os_window` | `boss.py` Boss method, zero-arg |
| `SimpleAction::CopyAnsiToClipboard` | `copy_ansi_to_clipboard` | `window.py` Window method, zero-arg |
| `SimpleAction::CopyLastCommandOutput` | `copy_last_command_output` | `window.py` Window method, zero-arg |
| `SimpleAction::ShowLastCommandOutput` | `show_last_command_output` | `window.py` Window method, zero-arg |
| `NewProcess::TabWithCwd` | `new_tab_with_cwd` | `boss.py` `*args: str`, `_create_tab` |
| `NewProcess::OsWindowWithCwd` | `new_os_window_with_cwd` | `boss.py` `*args: str`, `_new_os_window` |
| `KnownAction::CopyToBuffer(BufferName)` | `copy_to_buffer <name>` | `boss.py` handler; `options/utils.py` remainder parser |
| `KnownAction::PasteFromBuffer(BufferName)` | `paste_from_buffer <name>` | `boss.py` handler; `options/utils.py` remainder parser |

Public constructors: `BufferName::new` and `KnownAction::copy_to_buffer` /
`KnownAction::paste_from_buffer`, all returning
`Result<_, InvalidBufferName>` (`Empty` or `EdgeWhitespace`). Variant
additions require downstream exhaustive matches to update, as with the
nested-text pair.

## Three nested grammars, kept distinct

`action` is one outer string carrying several different argument grammars;
this slice touches three of them, and conflating them is the main correctness
risk the types prevent:

- **Escape-encoded** (`paste`, `send_text`): every Unicode scalar becomes
  fixed-width `\UXXXXXXXX`; kitty performs the sole decoding.
- **Shell-quoted argv** (`new_*` kinds): POSIX single-quote tokens preserve
  argv boundaries through `to_cmdline`; upstream then performs server-side
  tilde/environment expansion after splitting. The `_with_cwd` kinds join the
  existing `NewProcess` treatment unchanged (`options/utils.py` registers all
  three `*_with_cwd` names under one `shlex_parse`).
- **Raw remainder** (`copy_to_buffer`/`paste_from_buffer`): the parser
  returns `[rest]` verbatim — the whole string after the action name is one
  argument. `BufferName` therefore encodes as `format!("copy_to_buffer {name}")`
  with no quoting and no escape encoding; internal spaces, tabs, newlines,
  backslashes, quotes and shell metacharacters stay literal.

### Why edges are validated

The full parser applies `strip()` to the action string and `split(None, 1)`
for the remainder. A leading whitespace run in the name joins the separator;
trailing whitespace — including the `\u{1c}`–`\u{1f}` separator controls and
NEL, which Rust's `char::is_whitespace` does not cover — is stripped. A name
with either would parse back as a **different** buffer than the caller named.
`BufferName` rejects empty names and such edges (`InvalidBufferName`) instead
of silently addressing another buffer; interior characters of every one of
those classes remain accepted and literal. This mirrors `ActionText`'s
accept-only-what-survives principle where the escape encoder made everything
survive, and adapts it to a grammar with no escaping.

`clipboard` and `primary` stay ordinary accepted names with a doc-comment
distinction rather than typed recognition: the wire grammar is identical,
the special semantics live in the native handlers (`set_clipboard_string` /
`set_primary_selection` versus kitty's in-memory named buffers), and a typed
split would duplicate the existing `CopyToClipboard`/`PasteSelection`
vocabulary for what upstream models as plain argument data.

## Dispatch-scope caveats

`Boss.dispatch_action` tries Boss methods first (with `window_for_dispatch`
set from the RC action's single matched window), then the tab and window of
that window. Consequences audited from source:

- `close_other_tabs_in_os_window` is a Boss method using
  `active_tab_manager_with_dispatch`: with an RC target window it closes
  other tabs in **that window's OS window**; without one, the active OS
  window. The commissioning note ("acts on the active OS window") is the
  no-target case.
- `close_other_windows_in_tab` and `move_window_to_top` are Tab methods
  operating on the dispatch window's tab.
- `copy_ansi_to_clipboard`, `copy_last_command_output` and
  `show_last_command_output` are Window methods acting on the dispatch (or
  active) window's selection/command output. The last two require shell
  integration to have reported command output; `copy_last_command_output`
  copies the last **non-empty** output while `show_last_command_output`
  shows the **last-run** output.
- The `_with_cwd` kinds source the child working directory from the
  dispatch/fallback window via OSC 7 shell-integration reporting
  (`CwdRequest`), falling back to kitty's process-cwd heuristics without it.
- Dispatch acknowledgment remains the only success claim: it is not proof a
  selection was nonempty, a buffer held text, or a child was spawned.

**Version caveat:** nine of the ten names are registered documented actions
on both cached releases (0.43.1 and 0.49.2). `copy_last_command_output` was
added upstream between them — present in 0.49.2, absent from 0.43.1's
registry (its bare name still parses zero-arg there, but dispatch would
fail). The variant doc comment records this.

## Evidence and reproduction

[Wire tests](/protocol/commands/tests/window_tab_actions.rs) pin complete
strings and payloads independently: all six helper names, empty and quoted
argv for the cwd kinds (including family consistency with
`new_window_with_cwd`), fifteen remainder names kept literal, empty/edge
rejection with interior acceptance, all outer target forms and the unchanged
dispatch-only contract. The opt-in native test sends **Rust-generated**
strings to kitty's `+runpy` through the tiny
[fixture](/protocol/commands/tests/fixtures/window_tab_actions.py), which
calls the real `parse_key_action`.

The commissioned scratch oracle
(`.test-agent/window-tab-actions/verify.py`, gitignored, with README) runs
the same parser plus the `get_all_actions()` registry through an explicit
cached executable, using an encoder mirror bounded to this slice's three
formats and probing the edge shapes that justify validation.

| Cached release | Library-generated roundtrips | Scratch oracle |
| --- | --- | --- |
| 0.43.1 | 38 wire cases + 4 edge shapes passed | 32 wire + 7 edge passed; membership records `copy_last_command_output` absent |
| 0.49.2 | Same; all passed | 32 wire + 10 membership + 7 edge passed, zero failures |

Reproduce the opt-in test from the repository root:

```sh
KITTY_RC_NATIVE_KITTY="$PWD/.test-agent/supported-kitty/0.49.2/app/bin/kitty" \
  mise exec -- cargo test -p kitty-rc-commands --test window_tab_actions native_full_parser -- --ignored --nocapture
```

Converged gates (all through `mise exec --`):

| Check | Result |
| --- | --- |
| `cargo test --workspace` | 274 passed, 0 failed, 2 intentional native skips |
| `cargo test --workspace --doc` | 14 passed, including compile-fail invariants |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo fmt --all -- --config max_width=100 --check` | Passed |

Public usage is documented in the
[client guide](../../../client/README.md#named-clipboard-buffers-and-window-helpers).
The [coverage review](conversion-coverage0.gpt6a.md) retains the remaining
families; this slice does not advance them.
