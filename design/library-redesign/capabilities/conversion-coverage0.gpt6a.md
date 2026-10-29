---
type: CapabilityAssessment
title: Remaining command, action and output conversions
description: A source-backed return to ordinary library coverage work, distinguishing the outer RC inventory from the larger nested-action and output-model surfaces.
resource: /design/library-redesign/capabilities/conversion-coverage0.gpt6a.md
status: draft
generated:
  by: "model:openai/gpt-6-astra#xhigh"
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Human redirects attention to unfinished conversions, selects literal nested paste/send_text, and deprioritizes CLI and bounded streaming; parent inventories conventional gaps
  - resource: /protocol/commands/src/catalog.rs
    title: Outer command coverage labels and exact supported-profile notes
  - resource: /design/library-redesign/capabilities/nested-text0.sol61xh.md
    title: Reviewed completion of the selected literal paste/send_text conversion after the baseline inventory
  - resource: https://github.com/rektide/kitty-rc-rs/blob/36866a00d3e3/protocol/commands/src/actions.rs
    title: Frozen pre-nested-text Rust mapping used for the discovery comparison
  - resource: /protocol/commands/src/process.rs
    title: Launch options with remaining embedded string grammars
  - resource: /protocol/commands/src/outputs.rs
    title: Current listing models deliberately ignore several stable additive fields
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Historical outer-command and nested-action restoration inventory
  - resource: /design/library-redesign/teardown0.glm53m.md
    title: Preserved legacy action names and output-model restoration context
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/actions.py
    title: Native get_all_actions inventory executed through cached kitty 0.49.2
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/utils.py
    title: Named-buffer remainder parsers, nested key splitting and scroll-prompt boolean grammar inspected locally
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/boss.py
    title: Clipboard-buffer semantics, new-tab-with-cwd and close-other-tabs implementations, plus OS listing history
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/tabs.py
    title: Close-other-windows and move-window-to-top implementations, plus title-overridden listing output
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/window.py
    title: Clipboard/command-output helpers and stable listing fields inspected locally
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/launch.py
    title: Stable launch protocol spec distinguishes embedded grammars from plain string data
verification:
  scope: Native documented-action enumeration and frozen Rust mapping scan, followed by manual inspection of representative gaps and stable output/source contracts
  evidence: Kitty 0.49.2 enumerated 142 documented action names; the discovery scan recognized 61 names in the pre-nested-text Rust mapping
  limits: Name-level discovery is not option coverage, proof that every action is useful over RC, or an exhaustive conversion plan
---

# Remaining conversions

The human's correction is right: **there is substantial ordinary library
coverage work left**. Completing the broad outer-command inventory did not
complete kitty's much larger mappable-action surface, embedded option grammars
or listing models. The selected next conversion is literal nested `paste` and
`send_text`. CLI expansion and bounded streaming are not current priorities.

## What the earlier completion actually covered

The [outer RC inventory](inventory0.sol61m.md#complete-stable-rc-inventory-41-commands)
accounts for 41 stable command names. The [current catalog](/protocol/commands/src/catalog.rs)
has typed profiles for most of them, preserves deprecated `new-window` as raw,
and identifies partial/conversation/version-specific surfaces. Uploads and the
recent finite data/broadcast additions subsequently filled several original gaps.

But **`action` is one outer command containing a much larger nested language**.
Typing that envelope and a selection of actions never implied that all mappable
actions were converted. Likewise, a typed `Launch` struct can still expose some
option grammars as strings, and a typed listing can tolerate fields it does not
yet expose.

Native `kitty.actions.get_all_actions()` under cached kitty **0.49.2** returned
**142 documented names**. A lightweight scan of `SimpleAction`, `KnownAction`
and their helper mappings at Rust snapshot `36866a00` recognized **61** of them.
The ignored discovery script is `.test-agent/sendtext-boundaries/action_inventory.py`.
These are navigation counts, not a completeness percentage: some missing names
are aliases, UI/platform-specific actions or alternate spellings of capabilities
already expressible with another typed action.

## Conventional conversion candidates

The following examples are absent from that pre-change mapping and have native
contracts worth converting. They are possible next slices, not newly assigned
work or tickets.

| Family | Examples | Work that remains |
| --- | --- | --- |
| Literal nested text — selected now | `paste`, `send_text` | Literal encoding through the full action parser, explicit empty-input handling, typed keyboard-mode selection; preserve dispatch semantics. |
| Nested keys | `send_key` | Reuse existing typed key vocabulary where valid, account for whitespace-split nested grammar and deliberate raw shortcuts. Outer RC `SendKey` already exists. |
| Named clipboard buffers | `copy_to_buffer`, `paste_from_buffer` | Typed selection for named buffers and special `clipboard`/`primary` names; preserve remainder-parser behavior and native paste effects. |
| Clipboard/output helpers | `copy_ansi_to_clipboard`, `copy_last_command_output`, `show_last_command_output` | Source-audited no-argument variants, with shell-integration and UI-effect requirements documented. |
| Window/tab helpers | `close_other_windows_in_tab`, `move_window_to_top`, `close_other_tabs_in_os_window`, `new_tab_with_cwd` | Ordinary enum/helper extensions; audit actual active/dispatch scope and creation argv grammar rather than assuming identical targeting. |
| Existing action options | e.g. `scroll_prompt_to_top` boolean option | Model actual native parameter grammar; a name-only enum is not always the complete action. |

Named-buffer parsers return their remainder directly; their handlers distinguish
`clipboard`, `primary` and other buffer names. They should not be shell-quoted
or run through the text-action escape encoder. Different nested grammars remain
different even when all of them ultimately occupy the same `action` string.

The fixed ordinal names `second_window` through `tenth_window`, for example,
should be evaluated against existing `KnownAction::NthWindow` before adding
redundant wrappers. Mouse-context and platform-specific families deserve a
separate usefulness judgment rather than bulk conversion to inflate coverage.

## Output-model catch-up is also useful library work

[outputs.rs](/protocol/commands/src/outputs.rs) deliberately ignores stable
fields that upstream already emits. Candidate additions include:

- Window `neighbors`, `session_name`, `title_overridden`, `last_focused_at`,
  `needs_attention` and `has_activity_since_last_focus`.
- Tab `title_overridden` and OS-window `active_tab_history`.

This would let callers use existing listing information without raw JSON or
new execution machinery. Preserve tolerant additive decoding and account for
older servers' missing fields. `last_focused_at` is process-local monotonic
time, not a wall-clock value comparable across kitty instances.

## Embedded option grammars still have room for types

[Launch](/protocol/commands/src/process.rs) has typed targets, launch kind,
location, stdin source and several other options, but still accepts string
grammars for spacing, colors, markers, panel settings and OS-window position.
Existing appearance/marker types are possible building blocks after checking
the launch-specific spelling and accepted alternatives. Titles, paths and argv
remain legitimate string data; replacing every `String` is not a coverage goal.

## Recommended direction

Finish the selected literal nested-text pair, then choose a compact ordinary
conversion family or listing-model catch-up from the examples above. Tie each
slice to native parser/output evidence and a usable public constructor. The
[earlier send-text follow-through proposal](sendtext-followthrough0.gpt6a.md)
is historical discussion; its streaming/CLI sequence is not the user's current
priority or a prerequisite for this work.

**Implementation update:** the [selected nested text pair](nested-text0.sol61xh.md)
has landed and passed review. The 61-name discovery count above deliberately
remains bound to its earlier snapshot; the two literal actions now have typed
entrances. Next selection can return to the remaining conventional families
rather than implicitly advancing to streaming or CLI work.
