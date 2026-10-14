---
type: Research
title: "Kitty remote control since kitty-rc 0.5.0: baseline, stable, and development HEAD"
description: "Revision-pinned upstream evolution and its compatibility implications for this Rust client, separating new capabilities from pre-existing implementation defects."
resource: /design/upstream-kitty/evolution0.gpt6a.md
status: draft
author: "model:openai/gpt-6-astra#medium"
date: 2026-10-14
sources:
  - resource: "opencode://ses_ec7b73050ffeSoRWSwN2E8vc3X"
    title: "Commissioned source/history comparison, baseline recovery, compatibility analysis, and document verification"
  - resource: "https://github.com/rektide/kitty-rc-rs/commit/17dc33180c283bb7974daafde28cc4ec8920c0c9"
    title: "Local 0.5.0 release commit, 2026-02-04"
  - resource: "https://github.com/rektide/kitty-rc-rs/tree/97990cb2c64ed34deb53e932ef10fbf2b831c9ea"
    title: "Local source state inspected, including October cleanup"
  - resource: "https://github.com/kovidgoyal/kitty/tree/7f24dd68c98d8f4712457560f25e8056cce52ec8"
    title: "Kitty v0.43.1, the client's stated protocol target"
  - resource: "https://github.com/kovidgoyal/kitty/tree/b2cb871254fbdac19a085188813c8dfb258b95cd"
    title: "Kitty v0.49.2, latest stable at retrieval"
  - resource: "https://github.com/kovidgoyal/kitty/tree/5bf3c2dc676efd5c4b4526eb135c922701565707"
    title: "Kitty master at retrieval; changelog labels next version 0.50.0 future"
  - resource: "https://api.github.com/repos/kovidgoyal/kitty/releases/latest"
    title: "Latest stable release metadata retrieved 2026-10-14"
  - resource: "https://api.github.com/repos/kovidgoyal/kitty/commits/master"
    title: "Master revision verified against GitHub on 2026-10-14"
  - resource: "https://github.com/rektide/zooming-kittens/blob/462e7918a7d4d40aef011f0b4b168f203ad557d6/src/kitty/registry.rs"
    title: "Existing downstream pool and font-operation retries"
verification:
  method: "Read-only git history and source comparison; AST extraction of every RC protocol_spec and selected ls serializers; release API checks; local document/link checks"
  scope: "Source-level compatibility assessment, not live interoperability certification; no library implementation or test-suite execution in this session"
---

# Kitty remote control since we built this library

**The wire protocol has stayed remarkably stable; the larger gaps are command
coverage, richer discovery data, server lifecycle fixes, and defects already
present in our implementation.** Updating every version field to the latest kitty
would not fix those defects and would unnecessarily reject older servers.

The most useful modernization sequence is: make existing requests and replies
correct, preserve connection/request lifecycle state, then add stable upstream
capabilities. HEAD already contains useful workspace and resizing features, but
they are not part of the current stable release.

**Scope of compatibility:** upstream-version interoperability is the concern
here. The human explicitly declares all local interfaces unstable, with no API
compatibility obligation to any consumer, including zooming-kittens. A major
typed command/response redesign and substantial concern-based decomposition
(potentially separate crates for protocol, transport/lifecycle, commands,
discovery, and authentication) are authorized directions. Existing consumers
provide usage evidence, not constraints requiring legacy interfaces to survive.
This upstream assessment does not select the eventual crate boundaries.

## 1. What “when we built” actually means

There are three different baselines, which should not be collapsed into one date:

| Coordinate | Evidence and meaning |
| --- | --- |
| Initial implementation | Local history starts 2026-01-07. Protocol framing landed in `604b406`; streaming in `4bd5b38`; response structs in `c6244d1` on January 9. |
| Last substantial API construction | Authentication, single-connection behavior, and public-key lookup landed February 2. The **0.5.0 release commit is `17dc33180c283bb7974daafde28cc4ec8920c0c9`, February 4, 2026**, after the bon-builder/to_message refactor. The March 15 `6069aa6` commit only catches up Cargo.lock. |
| Declared upstream protocol target | **Kitty v0.43.1**, `7f24dd68c98d8f4712457560f25e8056cce52ec8`, released October 1, 2025. Local `94c3ab58c77bd28d8da358a9dfa2799842704a66` explicitly says it updates encryption's kitty version to 0.43.1 “to match current kitty version.” |

That statement records the developer's target, not the newest upstream release
then available: **v0.45.0 already existed on December 24, 2025**, before both the
initial build and 0.5.0. Its peeled commit is
`c26b7705307f7a832b8be4150da38866945c7abc`. Therefore changes introduced in 0.44–0.45
are target lag already present at construction; 0.46 onward is subsequent
released evolution. This document compares 0.43.1 forward and marks that boundary.
It does not claim to know the exact installed kitty binary used for development.

The local source inspection is pinned to
`97990cb2c64ed34deb53e932ef10fbf2b831c9ea`, October 13, 2026. Its cleanup commits do
not constitute an upstream protocol update. In particular, the old dropped-write
Result problem **was fixed** before this inspection: `client.rs:199–201` now uses
`??`; the 0.5.0 release source used only `?`. Do not carry that old finding forward
as an unfixed current defect.

### Targets retrieved October 14, 2026

| Target | Immutable revision | Status |
| --- | --- | --- |
| Latest stable, **v0.49.2** | `b2cb871254fbdac19a085188813c8dfb258b95cd` | GitHub latest-release API: published `2026-10-01T02:57:33Z`, `prerelease=false`. |
| Latest master/HEAD | `5bf3c2dc676efd5c4b4526eb135c922701565707` | GitHub commit API and local archive agree; commit time `2026-10-13T15:35:16Z`. Changelog calls the next release **0.50.0 [future]**. |

Upstream evidence came from `/home/rektide/archive/kovidgoyal/kitty`, which was
available at execution time, with GitHub metadata independently checking the tip.
No corresponding `~/wiki/kovidgoyal` directory was present. “Stable” and “HEAD” below
always mean these pinned snapshots, not moving branches.

## 2. Protocol, encryption, and sockets

### Stable envelope; version is not feature negotiation

`docs/rc_protocol.rst` is **byte-for-byte unchanged between 0.43.1 and 0.49.2**.
The envelope remains `ESC P @kitty-cmd <JSON> ESC \\`, including over sockets.
`cmd`, array-valued `version`, `payload`, `no_response`, and `kitty_window_id`
retain their meanings. Async and streaming envelope fields also predate our build.
[Baseline protocol][base-protocol]; [stable protocol][stable-protocol].

The server's `handle_cmd` compares the first two version components with its own;
a newer client major/minor is rejected. It does not advertise capabilities or
transform an old payload into a new one. An old version value also does not make a
new command available on an old server. See [stable dispatcher][stable-rc].

Locally, `src/command.rs:12–19` selects `[0,43,1]`, encryption's reconstructed
envelope uses that same array at `src/client.rs:178–191`, and ActionCommand still
uses `[0,14,2]` at `src/commands/action.rs:23`. The latter resembles the upstream
documentation example rather than a defensible separate support policy.
`src/encryption.rs:126–132` independently emits a string `"0.43.1"`; the client
reconstructs the envelope and discards that value. This is an existing inconsistent
representation, not a changed upstream requirement.

**Implication:** centralize the declared compatibility target, keep explicit
minimum-version/capability information for new operations, and test against actual
older servers. A global bump to 0.49 or future 0.50 is not a compatibility repair.

### Encryption version is still 1

Both endpoints retain `RC_ENCRYPTION_PROTOCOL_VERSION = '1'` in
`kitty/constants.py`. The algorithm is still X25519 → SHA-256(shared secret) →
AES-256-GCM; binary fields use RFC-1924-style base85. Password and nanosecond
timestamp belong inside the encrypted original command at its top level, not
inside command payload. The server still rejects timestamps over five minutes
away. Responses are not made confidential by this request encryption.

Actual changes are implementation hardening:

* Stable 0.47 introduces constant-time password lookup and fixes a partial socket
  write buffer-compaction condition in `b39f88c6a` (April 3). This is not a new key
  format or cipher negotiation. [Security/robustness change][security-change].
* HEAD's `parse_cmd` returns immediately when an encrypted request has no sender
  public key, rather than merely logging and attempting decryption. Valid client
  requests need no new field. [HEAD parser][head-rc].

Our local `src/encryption.rs:98–132` implements the same primitive family, but this
research has not independently run cross-language crypto vectors. Its algorithm
choice alone does not certify the full encrypted send path.

### Socket lifecycle: important fixes, no switch to “one frame per connection”

Already in 0.43.1, `has_complete_peer_command`/`dispatch_peer_command` in
`kitty/child-monitor.c` dispatch completed frames before EOF and retain remaining
input bytes. Persistent use is not newly forbidden. Python's reference client
half-closes its write side; the Go client sends framed commands, reads responses,
then closes. `tools/cmd/at/socket_io.go` is unchanged between baseline and stable
and explicitly keeps a queue of parsed responses. [Baseline server][base-peer];
[stable Go client][stable-socket].

Changes worth recognizing in diagnostics:

| First stable | Change | Consumer consequence |
| --- | --- | --- |
| 0.46.0 | `6ac4b8464`, February 9: keep peers with an outstanding async response from being pruned after input finishes. | Real server-side async/half-close repair, shortly after our release. An old-server failure here is not necessarily a Rust serialization bug. [Commit][async-peer-change]. |
| 0.47.0 | Correct `<` versus `>` in remaining-response `memmove` after a partial send. | Large replies are more reliable; exercise large `ls`, get-text, and screenshot responses. [Commit][security-change]. |
| 0.49.0 | `c85bcda29`: determine UNIX peer-credential support separately for single-instance and RC listeners. | Same-user protection is made listener-specific, particularly for Linux abstract single-instance sockets. Same-user checks for RC UNIX sockets already existed before 0.43.1; this is not a new blanket ban. [Commit][peer-credentials]. |
| HEAD only | `b9b7e0b29`, October 8: nonblocking accepted peers, EAGAIN/EWOULDBLOCK handling, resilient transient accept errors, locking/cleanup repairs. | One non-reading client should no longer stall all RC traffic; descriptor exhaustion no longer necessarily kills the listener. Not yet in 0.49.2. [Commit][nonblocking-change]. |

The peer input buffer's 64-KiB growth limit is present at both endpoints. It does
not authorize arbitrary JSON-field chunking: obey each command's streaming
contract. A blocked or timed-out client should not automatically replay a mutation
just because the newer server is more robust; delivery can still be ambiguous.

## 3. Released command changes

An AST-based inventory of **every `kitty/rc/*.py` protocol_spec** finds two added
commands between baseline and stable, no removed command definitions, and only
four existing command specifications with changed text/fields. Behavioral and
generated CLI changes require reading beyond those specs; those are included
below. Large formatting-only diffs in July are not protocol redesigns.

| First stable | Change since 0.43.1 | Library/design effect |
| --- | --- | --- |
| **0.44.0; already available when built** | `state:focused_os_window` matches all windows/tabs in the focused OS window. Launch wait-for-child callback setup repaired for overlays. | Existing string match fields can express the new selector. This is broader than selecting one focused pane. Waiting for a launched child remains an async operation, not a new feature introduced in 0.49. [Matching][stable-base], [launch][stable-launch]. |
| **0.46.0** | `scroll-window` accepts fractional **line** scrolling in addition to fractional pages. A page-scrolling regression was later fixed in **0.48.0** (`368753682`). | Represent amount as the upstream pair `[number-or-start/end, unit]`; allow fractional lines but keep prompt/unscroll restrictions. Account for intermediate upstream versions when diagnosing scrolling. [Implementation][stable-scroll]. |
| **0.47.0** | `set-background-image`: `centered` and `cscaled` RC layout choices; preconfigured image selection via `data: "index:N"`, including signed increments. `configured` with an index is rejected; clearing configured images clears the set. | Add explicit image-upload / remove / index-selection forms. Index selection is not a streaming upload. The old spec's literal unexpanded layout placeholder also becomes a real f-string; compare executable behavior, not just generated prose. [Implementation][stable-background]. |
| **0.47.0** | `launch` adds `os_window_position`. | Optional string field; placement remains platform/window-manager dependent. Existing requests do not require it. [Launch][stable-launch]. |
| **0.48.0** | `get-text` adds `alternate` and `alternate_scrollback`. | Useful for examining the other screen buffer, including scrollback when available. Existing free-form extent strings can already carry them, but typed documentation/tests should expose them. [Implementation][stable-get-text]. |
| **0.49.0** | New **`screenshot`** with `match`, `match_tab`, `output_path`; async completion. | A substantial new typed operation: visible OS window/tab/pane → PNG base64 string, or server-side file write with textual result. Hidden tabs/panes are rejected. A supplied path belongs to the machine running kitty, not necessarily the client. [Implementation][stable-screenshot]. |
| **0.49.0** | New **`set-os-window-title`**. | Distinct from setting an individual terminal window's title. Empty/omitted title restores automatic active-window tracking. Match through contained windows. [Implementation][stable-os-title]. |

The spec addition of `last_non_empty_output` is **documentation catch-up**: the
0.43.1 `get_text.py` handler and CLI choices already supported it. Similarly,
`run`, session output from `ls`, `wait_for_child_to_exit`, OS panels, and the
`action` command are not new since our baseline.

No existing RC `options_spec` default-value changes were found in this stable
comparison. `ls` still defaults to JSON, get-text to screen, and background image
layout to configured. There are meaningful behavior changes without changing
defaults: for example, `set-font-size`'s schema remains unchanged while **0.49's
zoom ceiling becomes `max(configured_size * 10, 256)` points**, previously
`configured_size * 5`; the minimum remains four points. This matters directly for
zooming consumers and is not a reason to alter the font-size wire payload.
[Font-size change][font-limit-change].

Other relevant CLI evolution is not a Rust wire requirement: the Go `set-colors`
parser gains functionality while the server color payload specification remains
unchanged; relative password-file paths are resolved under kitty's config
directory. If we later emulate `kitten @` CLI behavior, use those client-side
implementations as well as server schemas.

## 4. `ls`: richer state, mostly additive compatibility

Stable `ls` still returns JSON **as a string inside response `data`**, not as a new
wire array. The existing `parse_response_data` at `src/commands/window.rs:117–123`
handles both a string and an already-decoded Value. `ls --output-format=session`
already existed in 0.43.1; it needs a different result interpretation and is not
exposed by the current LsCommand. [Stable ls][stable-ls].

Inspecting the actual producers, rather than `ls.py` alone, gives these additions:

| First stable | Level / fields added | Use |
| --- | --- | --- |
| **0.45.0; already available when built** | Window `neighbors`; OS window `active_tab_history` | Directional navigation and recent-tab behavior. |
| **0.46.2** | Window and tab `title_overridden` | Distinguish configured/user titles from automatic titles. |
| **0.47.0** | Window `session_name`, `last_focused_at`, `needs_attention`, `has_activity_since_last_focus` | Session grouping, focus ordering, bell/activity UI. `last_focused_at` is from a monotonic clock, not a Unix timestamp; don't compare it across different kitty processes. |
| **HEAD only** | Window `program_status` | List of program-status records generated by the new OSC 7501 support; separate from shell `last_cmd_exit_status`. |

Sources: [stable Window.as_dict][stable-window], [stable TabManager.list_tabs][stable-tabs],
[stable Boss.list_os_windows][stable-boss], [HEAD Window.as_dict][head-window],
[HEAD ProgramStatus.as_list][head-status]. No existing keys were removed in these
three serializers in the examined range. HEAD's program-status data is provisional;
it should not be sold as a stable feature yet.

Our `WindowInfo`, `TabInfo`, and `OsInstance` (`src/commands/window.rs:14–115`) ignore
unknown fields, so these additions generally disappear rather than breaking
deserialization. They are useful coverage gaps. This is not a full `ls`
compatibility guarantee: layout-specific nested objects have their own shapes,
and our `LayoutOpts` is narrowly typed (`bias/full_size: i32`, `mirrored: String`,
lines 40–48). A representative corpus across layouts/platforms is more useful
than merely adding the new top-level fields. Preserve flexible extension data
where the upstream structure is open-ended.

## 5. Development HEAD: prepare interfaces, do not call these stable

The 0.49.2 → pinned HEAD command inventory adds **`resize-window-edge`** and changes
launch, detach-window, and resize-window. [HEAD release notes][head-changelog].

* **Docked windows:** launch types `window-dock` and `tab-dock`, with `dock_edge`,
  `dock_size`, and `dock_skip_focus`. Layout/navigation consumers must distinguish
  ordinary panes from attached UI; merely adding enum strings is not the whole
  user-facing feature. [HEAD launch][head-launch].
* **Named workspaces:** `launch.focus_existing`, `location=by-title`, and
  detach-window's `tab_title`, `location`, `create_if_missing`, plus
  `target_tab=@title`. The latter searches visible tabs in the active OS window
  and respects `tab_bar_filter`; it is not a global unique-name registry.
  [HEAD detach-window][head-detach].
* **Fractional resizing:** resize-window gains finite `[0,1]` `fraction` (zero
  preserves ordinary resizing) and `restore_entry_layout` for reset.
  [HEAD resize-window][head-resize].
* **Edge resize sessions:** resize-window-edge has start/state/select/move/reset,
  edge selection, and optional divider identity. Result `data` is another
  JSON-encoded string, with `edges`, `selected`, `divider`, `status`, and `failed`.
  Divider tokens reflect current layout identity and are unsuitable as durable
  IDs. Reset can fail when topology changed. [HEAD edge command][head-edge].
* **Lifecycle robustness and program status:** see sections 2 and 4. These affect
  existing consumers even if no new command builder is added.

## 6. Existing local defects are not evidence of upstream breakage

The following checks compare the inspected Rust source against **0.43.1 itself**.
They should be repaired before claiming that new upstream features work.

| Local evidence at `97990cb2` | Baseline contract / consequence |
| --- | --- |
| `src/commands/action.rs:6–23` serializes `action` plus separate `args`, then sends `send_key`. | Baseline `kitty/rc/action.py` expects command **`action`** and one action string containing its arguments. Baseline send-key expects a list of key strings. Both the command selection and action-argument representation need correction. |
| `src/commands/window.rs:206–207` uses `keys: String`; `src/commands/process.rs:67–117` uses string args, map env/var, boolean copy_env, and unrenamed `window_type`. | Baseline send-key/launch use lists; launch's wire field is `type`, env/var are string lists, copy_env carries a list, not the CLI boolean. RunCommand (`process.rs:20–36`) has similar list-vs-string/map discrepancies. These did not arise in 0.49. |
| `src/protocol.rs:21–22` exports `async_id` without a serde rename. | Baseline wire field is **`async`**. Dispatcher internally calls it async_id only after decoding. Its round-trip test (lines 291–296) proves self-consistency, not server compatibility. |
| `src/protocol.rs:98–164` detects any long string and chunks it into `data`, omitting the other payload fields. | Streaming is command-specific. Background image/run have particular binary encodings, sizes, metadata and terminators; send-text has its own chunk semantics. A small image still needs its end marker. Arbitrary long match/title strings cannot be converted into streaming data. Lossy byte-to-UTF-8 conversion can also alter content. |
| `src/client.rs:240–249` encrypts, then calls `send`, which encrypts again at line 196. | Encryption wraps the original command once. `send_all` and thus `send_command` remain incorrect for authenticated use, independent of upstream drift. |
| `src/client.rs:262–267` sends without reading and does not force no_response; execute (`235–238`) always reads. | Fire-and-forget must suppress replies; response-producing requests must consume their replies. Executing a no_response request should not wait for a response that was explicitly disabled. Otherwise pooled request/reply state becomes misaligned. |
| `src/client.rs:206–232` allocates a fresh receive buffer, looks only for an ending suffix, and resets the timeout on each read. | Multiple frames can arrive together; cancellation can discard a partial frame. Introduce retained parsing state and deliberate request deadlines/cancellation semantics. A stream acknowledgment is not the command's final result. |
| `src/client.rs:235–238` returns decoded `ok:false` as Rust Ok; `LsCommand::parse_response` (`window.rs:153–158`) returns empty data when absent. | The envelope distinguishes transport success from remote command failure. Preserve that distinction explicitly; do not mistake rejected ls for an empty instance. Returning raw envelopes is usable if callers check them consistently. |

Baseline source anchors: [action][base-action], [send-key][base-send-key],
[launch][base-launch], [run][base-run], [dispatcher][base-rc],
[streaming implementation][base-stream]. In particular, the upstream Go client
consumes an initial streaming acknowledgment and then the final response; the
Rust response type (`src/protocol.rs:204–209`) has no stream discriminator.

The historical [encryption bug note](../../doc/BUG-encrypted.md) explains a real
earlier misplaced-password/timestamp issue, repaired by local `967378a` on
February 2. Current `client.rs:168–174` places those fields correctly. That note
must not be read as the current code, nor confused with the remaining double
encryption path. Its proposed workaround and repository URL are historical text,
not the recommendations of this assessment.

## 7. Discovery, credentials, and downstream implications

The upstream model remains **endpoint plus credentials**, not “PID implies a
standard socket and key file.” Kitty provides `KITTY_LISTEN_ON` and
`KITTY_PUBLIC_KEY`; its Go client supports UNIX/TCP/fd endpoints and TTY fallback.
The configurable listen address, fd transport, per-process public key, and scoped
remote-control authorization already predate this comparison. Stable config
still defaults `allow_remote_control` to no and `listen_on` to none.
[Stable client setup][stable-main]; [child environment][stable-child].

Our `KittyBuilder::from_pid` (`src/client.rs:48–54`) encodes this project's
`kitty-{pid}.sock` convention, not universal upstream discovery. The executable
`kitty-pubkey-db`, PID extraction, and `~/.config/kitty/key.pub` fallback are local
integration policy (`client.rs:84–141`, `encryption.rs:73–95`). They cannot establish
that a key belongs to an arbitrary supplied endpoint. Reconnect currently replaces
the socket but retains its Encryptor (`client.rs:269–278`); a restarted instance or
reused PID needs deliberate credential/instance identity handling.

Keep transport addressing, instance identity, and credential acquisition separate
in the redesigned interfaces. Plain path support can remain useful; endpoint
expansion should be deliberate. Upstream evidence supports decomposing these
concerns, while the choice of shared lifecycle abstractions and crate boundaries
belongs to the independently commissioned redesign work.

The local [registry proposal](../../doc/PLAN-registry.md) is historical. An actual
pool already exists in [zooming-kittens' registry][consumer-registry], including
PID mapping, reaping and retries. At lines 244–303 it sends successive +1/-1 font
mutations, and the surrounding retry loop can repeat operations after a partial
failure. Modern server fixes do not make these increments idempotent. A lost
reply after execution leaves an unknown result; replay can double-apply already
successful increments. The consumer does explicitly inspect `response.ok`, so the
raw-envelope design is not automatically mishandled by this caller.

The new focus/session/activity fields offer concrete value to that consumer;
the higher font-size ceiling changes its effective behavior without any schema
change. Those are more immediate benefits than adding every future HEAD command.

## 8. Recommended landing sequence and verification boundary

1. **Restore baseline fidelity:** correct wire names/types, single encryption,
   reply suppression/consumption, and command-specific streaming. Use upstream
   request/response fixtures rather than only Rust encode/decode round trips.
2. **Make lifecycle part of the interface:** a typed command abstraction should
   associate request encoding, result decoding, async/stream policy and expected
   response behavior. Replace the existing API freely where needed. Preserve buffered frames; distinguish command rejection,
   transport failure and delivery-unknown results. Test cancellation and reconnect
   before relying on pooled clients.
3. **Add stable information and capabilities:** richer ls structs, screenshot,
   OS-window titles, alternate-screen text, background selection and window
   position. Keep optional additions compatible with older servers; document
   first supported versions and explicit fallbacks.
4. **Track HEAD as a separate target:** docks, named workspace operations, edge
   resizing and program status need their own fixtures and actual server testing
   when adopted. They do not require upgrading the envelope design wholesale.

Useful future interoperability matrix: baseline 0.43.1, build-era newest 0.45.0,
stable 0.49.2, and the pinned HEAD if adopting unreleased features. Include plain
and encrypted ls, remote rejection, no-response commands, split/coalesced frames,
large text output, small and multi-chunk images, async cancel, launch wait,
server restart and interrupted non-idempotent mutations. Exercise more than one
layout and distinguish same-user UNIX sockets from other transport modes.

**Verified here:** git history and tag ancestry; GitHub stable/master metadata;
whole command-spec inventory; selected handler/default/client comparisons; actual
ls serializer deltas; server peer lifecycle changes; local file/line evidence;
document links. First-release labels above were checked with tag containment,
not inferred from author dates (some commits were merged much later).

**Not verified here:** running kitty binaries, runtime permissions/prompts,
cross-language encryption vectors, all nested layout schemas, all OS/backend
behavior, or every mappable action. No tests/builds were run for this documentation
task. Broader rendering, font, keyboard and graphics protocol changes matter to
terminal applications but do not by themselves change this RC envelope. This is
a source-grounded migration assessment, not an interoperability certification.

## Cross-references and reproducibility

* [Research index](README.md) is the entry point for this comparison.
* [Library README](../../README.md) explains the public surface; its broad streaming
  and async support claims need to be evaluated against section 6's wire evidence.
* [Response inventory](../../doc/info.md) is the historical local field list;
  section 4 gives the upstream additions and the compatibility distinction.
* [Public-key database proposal](../../doc/PLAN-pubkey-db.md) and
  [password plan](../../doc/PLAN-password.md) record how the current local policy
  arose; neither is an upstream discovery specification.
* [Registry proposal](../../doc/PLAN-registry.md) records earlier intent;
  [the downstream implementation][consumer-registry] supplies current evidence.

The ignored `.test-agent/upstream-kitty/inspect.py` script used Python AST parsing
on `git show REV:path` text without importing/executing upstream modules. To
reproduce the essential comparison, inspect the three exact revisions above with
`git diff BASE STABLE -- docs/rc_protocol.rst kitty/rc kitty/remote_control.py`,
then inspect `kitty/window.py`, `kitty/tabs.py`, `kitty/boss.py`,
`kitty/child-monitor.c`, and `tools/cmd/at/`. Use `git tag --contains COMMIT
--sort=version:refname 'v0.*'` for first released containment. The scratch script
is an aid; all load-bearing results and immutable evidence links are in this file.

[base-protocol]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/docs/rc_protocol.rst
[stable-protocol]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/docs/rc_protocol.rst
[base-rc]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/remote_control.py
[stable-rc]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/remote_control.py
[head-rc]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/remote_control.py
[base-peer]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/child-monitor.c
[stable-socket]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/tools/cmd/at/socket_io.go
[security-change]: https://github.com/kovidgoyal/kitty/commit/b39f88c6a
[async-peer-change]: https://github.com/kovidgoyal/kitty/commit/6ac4b8464
[peer-credentials]: https://github.com/kovidgoyal/kitty/commit/c85bcda29
[nonblocking-change]: https://github.com/kovidgoyal/kitty/commit/b9b7e0b29
[stable-base]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/base.py
[stable-launch]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/launch.py
[stable-scroll]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/scroll_window.py
[stable-background]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/set_background_image.py
[stable-get-text]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/get_text.py
[stable-screenshot]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/screenshot.py
[stable-os-title]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/set_os_window_title.py
[font-limit-change]: https://github.com/kovidgoyal/kitty/commit/18bddffcc
[stable-ls]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/ls.py
[stable-window]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/window.py
[stable-tabs]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/tabs.py
[stable-boss]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/boss.py
[head-window]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/window.py
[head-status]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/program_status.py
[head-changelog]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/docs/changelog.rst
[head-launch]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/launch.py
[head-detach]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/detach_window.py
[head-resize]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/resize_window.py
[head-edge]: https://github.com/kovidgoyal/kitty/blob/5bf3c2dc676efd5c4b4526eb135c922701565707/kitty/rc/resize_window_edge.py
[base-action]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/rc/action.py
[base-send-key]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/rc/send_key.py
[base-launch]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/rc/launch.py
[base-run]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/rc/run.py
[base-stream]: https://github.com/kovidgoyal/kitty/blob/7f24dd68c98d8f4712457560f25e8056cce52ec8/kitty/rc/base.py
[stable-main]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/tools/cmd/at/main.go
[stable-child]: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/child.py
[consumer-registry]: https://github.com/rektide/zooming-kittens/blob/462e7918a7d4d40aef011f0b4b168f203ad557d6/src/kitty/registry.rs
