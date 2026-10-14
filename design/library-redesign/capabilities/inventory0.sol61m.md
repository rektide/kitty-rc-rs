---
type: CapabilityAudit
title: Stable finite and nested kitty capability inventory
description: Revision-pinned RC inventory, actual completion contracts and finite implementation boundaries, including nested actions, kittens, config and upload exclusion.
resource: /design/library-redesign/capabilities/inventory0.sol61m.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec603e323ffeVy8bC4dQ6KgFpj"
    title: Stable and HEAD handler audit and grouped finite catalog commission
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Accepted command contracts and nested entrance direction
  - resource: /design/library-redesign/implementation0.sol61m.md#current-correction-standing
    title: Corrected audited reuse and mode interface
  - resource: /design/library-redesign/teardown0.glm53m.md#5-restoration-inventory-for-the-command-catalog
    title: Restoration inventory, corrected against stable handlers here
  - resource: https://github.com/kovidgoyal/kitty/tree/e300597364aabb4890ebdd3a5491397f858fd714/kitty/rc
    title: All 41 stable v0.49.2 command specs, client payloads and handlers directly inspected
  - resource: https://github.com/kovidgoyal/kitty/blob/e300597364aabb4890ebdd3a5491397f858fd714/kitty/options/utils.py
    title: Directly inspected nested action argument parsers
  - resource: https://github.com/kovidgoyal/kitty/blob/e300597364aabb4890ebdd3a5491397f858fd714/kitty/remote_control.py
    title: Native terminal responses, async registration and stream handshake
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/base.py
    title: StreamInFlight replacement/abort and empty-terminator behavior
  - resource: https://github.com/kovidgoyal/kitty/blob/e300597364aabb4890ebdd3a5491397f858fd714/kitty/boss.py
    title: Nested combine dispatch, interactive create_marker and resize return behavior
  - resource: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/conf/utils.py
    title: ToCmdline splits then expands tilde and server environment even for quoted tokens
verification:
  scope: Source audit, not live server or minimum-version conformance
  evidence: Stable tag peeled to e300597364aabb4890ebdd3a5491397f858fd714; HEAD 5bf3c2dc676efd5c4b4526eb135c922701565707; git diff of kitty/rc and direct handler/parser reads
  limits: Presence at a revision is not the earliest release or a selected support floor; custom nested extensions remain unknown
---

# Stable capability audit

Work anchors: `kitty-rc-proto-rs-redesign-capabilities-audit`,
`kitty-rc-proto-rs-redesign-capabilities-typed`, and the existing
`kitty-rc-proto-rs-vid` Action ticket. This is the independent audit checkpoint,
not implementation completion. [Implementation report](implementation0.sol61m.md)
will record actual coverage and tests; parent owns the shared design index.

## Revision and outcome law

Stable **v0.49.2** = `e300597364aabb4890ebdd3a5491397f858fd714`.
Inspected **0.50dev HEAD** = `5bf3c2dc676efd5c4b4526eb135c922701565707`.
The command below names its stable `kitty/rc/<name_with_underscores>.py` source.
Every row was inspected in both its protocol spec and handler/client constructor;
the handler wins where these differ. This sets no global support/version floor.

`R` = one classified terminal reply; `D` = deferred terminal reply, not a
subscription; `S` = submission without acknowledgment. `U` = upload conversation
not implemented in this finite wave. `ack` means native success without required
data, not proof of every UI/child side effect. Generic outer `no_response` can
suppress replies/errors; submission-only transports must retire. `send-key` and
`send-text` use S to match reference-client policy even though the socket server
handler itself can emit an acknowledgment when suppression is not set.

Audited built-in finite profiles may opt into **transport** reuse after a clean
terminal boundary. This never means replay safety. Arbitrary action/kitten/raw
profiles retire by default. Remote errors are classified before typed decoding;
required output fields must not become default/empty successes. Additive object
fields can be ignored. Process exit failure is successful *command data*, not
an `ok:false` RC failure.

## Complete stable RC inventory (41 commands)

Payload fields below omit injected peer/async metadata. Optional scope fields
keep upstream names, not a universal fabricated `target` field.

| Command | Payload/options | Result / mode | Boundary and intended finite coverage |
|---|---|---|---|
| action | `action` single full string, `match_window`, `self` | ack R or S | Typed known grammar plus raw action; dispatch only; retire |
| close-tab | `match`, `self`, `ignore_no_match` | ack R/S | Close requested, not child exit |
| close-window | same | ack R/S | Marks close, not disappearance observation |
| create-marker | `match`, `self`, `marker_spec` string list | ack R | text/itext/regex/iregex pairs or function; nested parser |
| detach-tab | `match`, `self`, `target_tab` | ack R | First target tab's OS window; no target creates OS window |
| detach-window | `match`, `self`, `target_tab`, `stay_in_tab` | ack R | `new` means new tab; empty means new OS window |
| disable-ligatures | `strategy` never/always/cursor, `match_window`, `match_tab`, `all` | ack R | Applies window rendering strategy |
| env | `env` dictionary | ack R | Future children; deletion key `NAME=`; values expanded on server |
| focus-tab | `match` (client also emits payload `no_response`) | ack R/S | First matching tab |
| focus-window | `match` | ack R/S | First matching window |
| get-colors | `match`, `configured` | required conf-format string R | Not JSON color map |
| get-text | `match`, `extent`, `ansi`, `cursor`, `wrap_markers`, `clear_selection`, `self` | required text string R | Nine stable extents, command output needs shell integration; can clear selection |
| goto-layout | `layout`, `match` | ack R | Full layout/prefix may be ambiguous/disabled |
| kitten | `kitten`, `args` list, `match` | absent/string/bool R | First match; installed/custom Python extension; retire |
| last-used-layout | `match`, `all` (client payload `no_response`) | ack R/S | Switch previous layout |
| launch | `args` list, `type`, `match`, `self`, `next_to`, `source_window`, cwd/title/env/var and launch options | required decimal window-ID string R, or exit-code/signal string D | Zero denotes no window; env/var/copy_env are lists; deferred child wait already implemented |
| load-config | `paths` list, `override` list, `ignore_overrides` | ack R/S | Server filesystem/config-dir resolution; broad interpretation invalidation |
| ls | `all_env_vars`, `match`, `match_tab`, `self`, `output_format` | required JSON-encoded listing string R; session format string R | Client constructor omits self/output_format but handler accepts; session only without filters |
| new-window | client translates old `window_type`/`new_tab` into launch `type`; `args`, title/cwd etc | required ID string R/S | Deprecated; use typed launch; raw entrance only |
| remove-marker | `match`, `self` | ack R | Remove window marker |
| resize-os-window | `match`, `self`, `incremental`, `action`, `unit`, width/height, `os_panel` list | ack R/S | cells/pixels; resize/toggle-fullscreen/toggle-maximized/toggle-visibility/hide/show/os-panel; WM may refuse |
| resize-window | `match`, `self`, integer `increment`, `axis` horizontal/vertical/reset | bool, absent, or layout-refusal string R | **Not** scroll amount pair; `string_return_is_error` is reference-client interpretation, not an envelope rewrite |
| run | `cmdline` list, `env` list, privilege options, base64 `data`, stream ID | required object stdout/stderr base64, exit_code/exit_status D+U | Even empty input uses shared accumulator; defer execution until exclusion lane |
| screenshot | `match`, `match_tab`, `output_path` | PNG base64 string D, or `Screenshot saved to: <absolute path>` string D | Visible target required; distinguish server-path and image profiles; introduced 0.49 |
| scroll-window | `match`, `self`, `amount` pair | ack R/S | number + l/p/u/r, start/end + null; unscroll/prompt integral |
| select-window | tab `match`, `self`, `title`, `exclude_active`, `reactivate_prev_tab` | required integer window ID D | No selection yields RC error; visual selection may never finish before deadline |
| send-key | `keys` list, `match`, `match_tab`, `all`, `exclude_active` | S | Program keyboard mode may suppress sending; no delivered-key proof |
| send-text | prefixed `data`, match fields, all/exclude, `session_id`, `bracketed_paste` | S; repeated client chunks | Finite literal text only in this wave; broadcast/stdin/file and binary streams deferred |
| set-background-image | `data`, `match`, `all`, `configured`, `layout` | ack R/S or U | Finite `-` removal / `index:N`; upload handshake and empty terminator deferred |
| set-background-opacity | `opacity`, `match_window`, `match_tab`, `all`, `toggle` | ack R | Requires dynamic_background_opacity; scoped to containing OS windows |
| set-colors | `colors` object RGB integers/null; transparent_background_colors string, match fields, all/configured/reset | ack R | Reset implies all+configured in client; no automatic local file interpretation |
| set-enabled-layouts | `layouts` list, `match`, `configured` | ack R | Client Python omits configured but handler implements it |
| set-font-size | `size`, `all`, `increment_op` +,-,*,/ | ack R | Zero reset; active/all OS window, not window match; all changes future default |
| set-os-window-title | `title`, `match`, `self` | ack R | Empty resets to active child title; introduced 0.49 |
| set-spacing | canonical `settings` map margin/padding-top/right/bottom/left -> float/null; match fields, all/configured | ack R | Null restores configured edges; shorthand expansion is client-side |
| set-tab-color | `colors` map active_fg/active_bg/inactive_fg/inactive_bg -> RGB/null, `match`, `self` | ack R | Null removes override |
| set-tab-title | `title`, `match` | ack R | Empty uses active child title |
| set-user-vars | `var` list, `match`, `self` | required string R | No vars queries, NAME removes, NAME=VALUE sets; setting can return empty string |
| set-window-logo | `data`, `position`, `alpha`, `match`, `self` | ack R/S or U | Finite `-` removal; uploads deferred |
| set-window-title | optional `title`, `temporary`, `match`, `self` | ack R | Missing title restores child title |
| signal-child | `signals` list, `match`, `self` | ack R/S | Foreground process signals, not exit wait |

## Nested action inventory and grammar

The teardown's ~65 marker types are not one Rust type per method here. Group
known grammar in `actions` enums, and expose a clearly raw complete action string.
Argument parsing is action-dependent: titles/layout/opacity use remainder string;
kitten/launch/config/color commands use shell-like splitting; integer navigation
uses integers; send_text is mode + Python/ANSI-C escapes, not raw UTF-8 args.
These are not RC payload objects. Target is first matched window via outer
`match_window`/`self`; effects may reach tab/server/OS-window beyond that target.
Nested `new_*`/`launch`/`kitten` use `to_cmdline`, which expands tilde/environment
variables **after** splitting. Quoting preserves token boundaries but is not
literal argv preservation. Use the outer RC Launch/Kitten JSON lists for literal
values. load_config_file/set_colors use shlex splitting without that helper.

| Restoration group | Stable names / argument contract | Standing |
|---|---|---|
| No-argument window/tab movement | next_tab, previous_tab, move_tab_forward/backward; next_window, previous_window, move_window_forward/backward; first_window, reset_window_sizes | Typed no-arg profiles; window/tab UI scope |
| Close/create | quit; close_tab, close_window, close_window_with_confirmation [ignore-shell]; new_tab/new_window/new_window_with_cwd with command-line | Simple closes typed; create/launch args deliberately shell-quoted; acknowledgment not lifecycle completion |
| Navigation | goto_tab integer (negative parser adds one); nth_window integer; neighboring_window left/right/top/bottom; move_window direction or integer | Typed integers/directions, preserve wire integer semantics |
| Titles/layout | set_tab_title/set_window_title remainder; goto_layout/toggle_layout remainder; next_layout integer; last_used_layout | Typed strings/layout escape; ordering caller-owned |
| OS window | toggle_fullscreen, toggle_maximized | Typed no-arg; environment-dependent UI effects |
| Clipboard | copy_to_clipboard, paste_from_clipboard, paste_selection, clear_selection, copy_or_interrupt; paste escaped remainder | Known no-arg profiles typed; raw escaped paste deferred |
| Scroll | scroll_line_up/down, scroll_page_up/down, scroll_home/end, show_scrollback; scroll_to_prompt integer plus offset; scroll_to_mark prev/next group | Typed finite grammar, show_scrollback may launch overlay |
| Markers | toggle_marker shell-split type/pairs; remove_marker; create_marker no args | RC create-marker sets specification; mappable create_marker opens interactive ask kitten, not equivalent completion |
| Mouse/debug | mouse_selection enum; legacy mouse_click_url; debug_config; dump_lines_with_attrs optional remainder | Context-specific mouse actions left raw; mouse_click_url legacy spelling not asserted; debug/raw only |
| Process/nested | send_key whitespace keys; send_text mode+escaped text; kitten shell argv; launch shell argv; signal_child whitespace signals | kitten/launch/signal typed grammar; send_text raw nested grammar initially |
| Config/style | clear_terminal action + active/all; show_kitty_doc remainder; edit_config_file; change_font_size all/current + operator/amount; set_background_opacity remainder; load_config_file shell argv; set_colors shell argv | Typed audited grammar for clear/font/config/colors; config effects broad |

`boss.combine(..., raise_error=True)` returns consumed/dispatch success. A known
grammar is not proof that a particular target consumes the action or all later
effects finish. Raw arbitrary actions and UI/no_ui kitten invocation are not
replayable or reusable merely because their outer command has one reply.

## Kitten/config installation and invalidation

`kitten` names a builtin or Python file; relative files resolve in **server**
config directory. Only first matching window is invoked. `no_ui` handler return
string or bool survives; other returns become absent data. A visible UI kitten
can complete later while RC already acknowledges dispatch. No deployment,
arbitrary structured JSON query, or named kitten availability is inferred.

`load-config.paths` are server-readable paths, relative to server config dir.
Empty paths reload previous config. `override` is a list of name=value strings;
ignore_overrides controls previously supplied overrides. Interpretations of
layout/style/key/action defaults and prior config facts can become stale after
reload. This library does not maintain such caches or automatically reconcile
them; callers acquire fresh observations explicitly.

## Deferred and version-gated work

HEAD-only `resize-window-edge` has no stable contract here. HEAD `resize-window`
adds fractional/edge behavior and HEAD detach-window adds location/create-if-
missing/tab-title; HEAD launch adds docks. They are not silently stable APIs.
Nine text extents and OS-window title/screenshot are stable; version gates are
per capability, not global wire version replacement. Earliest release is only
asserted when previously documented release evidence exists (text alternate
0.48, screenshot/OS title 0.49); other entries say observed at stable 0.49.2.

Uploads need handshake (first nonempty `stream:true`), stream ID, end marker,
bounded producer demand, deferred terminal result where appropriate, and
same-server/same-command exclusion. `StreamInFlight.handle_data` aborts the old
stream when IDs differ **even for empty new input**. A finite `RunEmpty` is not
safe alongside uploads without that lane. Proposed shared seam: an optional
command-specific exclusion key in prepared/admitted profile plus a dedicated
upload conversation; ownership remains with the separately commissioned xhigh
upload wave. No engine change is authorized by this audit.

## Cross-references

- [Draft 1 nested entrances](../draft1.gpt6a.md#6-commands-include-the-larger-surface-inside-action): architectural requirement implemented from handler facts, not legacy wrappers.
- [Teardown inventory](../teardown0.glm53m.md#5-restoration-inventory-for-the-command-catalog): historical scope; corrected env/resize facts above.
- [Upstream evolution](../../upstream-kitty/evolution0.gpt6a.md): prior version evidence, not a new support floor.
