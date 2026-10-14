---
type: ImplementationReport
title: Audited finite capability catalog and nested Action implementation
description: Actual grouped finite command coverage, typed outputs, source-executed golden cases, mode/reuse boundaries and accurate partial-ticket standing.
resource: /design/library-redesign/capabilities/implementation0.sol61m.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#medium"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec603e323ffeVy8bC4dQ6KgFpj"
    title: Stable catalog audit, finite/nested implementation, pinned upstream fixture execution and workspace verification
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Direct source inventory and command-specific boundaries
  - resource: /design/library-redesign/draft1.gpt6a.md#6-commands-include-the-larger-surface-inside-action
    title: Accepted nested entrance and typed-output architecture
  - resource: /design/library-redesign/implementation0.sol61m.md#current-correction-standing
    title: Existing corrected Command modes, audited pooling and classification contract
  - resource: /design/upstream-kitty/evolution0.gpt6a.md#3-released-command-changes
    title: Release-specific capability evidence without a global support-floor change
  - resource: https://github.com/kovidgoyal/kitty/tree/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc
    title: Stable command constructors, result handlers and specs inspected and exercised
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/options/utils.py
    title: Nested action parsers inspected and selected pure parsers directly executed
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/boss.py
    title: Dispatch-only acknowledgment, interactive marker action and resize behavior
verification:
  scope: Pure finite catalog plus existing workspace composition, not new live kitty or upload conformance
  evidence: 30 command tests, 15 actual upstream payload constructors, 7 actual upstream parsers, 124 workspace nextest tests, locked all-target check, strict clippy, formatting and one compiled builder doctest
  limits: No new live terminal mutations, no upload implementation, no earliest-version assertion where evidence is unknown, no selected global support floor
---

# Finite capability implementation

## Outcome and standing

The broad finite catalog is implemented in `protocol/commands`, using **grouped
domain families**, not the removed ~65 one-method marker wrappers. The
[complete stable inventory](inventory0.sol61m.md) covers **41 commands**, plus an
explicit provisional HEAD-only entry. Typed finite profiles cover 39 stable
names fully or as explicit finite subsets; `new-window` remains deprecated/raw,
and `run` remains upload-deferred with a standalone strict result decoder.
No compatibility facade for the old wrong serializers was restored.

Checkpoints: `02be4bd8` independent inventory; `8a38a51d` grouped finite catalog,
source verifier and strict decoding. The audit initially identified annotated
tag object `e300597364aabb4890ebdd3a5491397f858fd714` as a commit; the correction
uses its actual peeled stable commit **`b2cb871254fbdac19a085188813c8dfb258b95cd`**
for public source URLs and catalog metadata. Contents inspected were the same
stable tree, not a different revision. HEAD stays
`5bf3c2dc676efd5c4b4526eb135c922701565707` (0.50dev).

`kitty-rc-proto-rs-redesign-capabilities-audit` is closed for the inventory.
The existing `kitty-rc-proto-rs-vid` Action ticket can close for this implemented,
verified source-backed grammar and raw entrance. **The broad typed ticket stays
open:** its buffered run execution acceptance is not complete until the upload
wave provides accumulator exclusion and its conversation. A source/output
decoder is not claimed as a working run operation.

## Public entrances

All entrances implement the existing `Command` associated output/class/mode
interface, without exchange/tokio/registry edits. `client::commands` already
reexports the whole crate; no facade edit was needed.

| Module | Public profiles | Output / notable native constraints |
|---|---|---|
| root | Ls, GetText, FocusWindow, LaunchWait, SessionText, RawFinite | Prior first slice retained; nine text extents/options; session output is separate unfiltered string profile |
| actions | Action, ActionSpec::{Known,Raw}, KnownAction, SimpleAction, grammar enums | Unit dispatch ack; action full string + match_window/self; all retire |
| window | WindowCommand/WindowOperation, ResizeWindow, SelectWindow, SendText, SendKey, UserVars, MarkerSpec | Ack group; resize bool/absent/refusal result; selected WindowId; text/key submission only; vars required string |
| tab | TabCommand/TabOperation, TabColor | Focus/close/title/detach/layout/enabled-layouts/tab-colors ack group |
| appearance | Appearance/AppearanceOperation, SetFontSize, GetColors, FontChange | Ligatures, opacity, colors, spacing; image remove/index finite; fonts active/all OS windows; colors conf-format string |
| configuration | LoadConfig, Env, Kitten, KittenResult | Server paths/list overrides; future-child env dictionary; first-window kitten return absent/string/bool |
| process | Launch, LaunchResult, Screenshot, SaveScreenshot, RunOutput | Launch decimal ID or zero; PNG bytes versus server-path confirmation; strict buffered binary run result decoder only |
| targets | WindowId, TabId, WindowTarget, TabTarget, WindowSet, Direction, Layout, Assignment, Rgb, Opacity, Finite, ServerPath | Domain IDs/selectors; open match/layout escape; validated RGB/finite numeric domains; nonempty server save path |
| catalog | CAPABILITIES, capability, Coverage, Completion, pinned revisions | Observed stable contracts and explicit deferred/deprecated/HEAD boundaries; known earliest release metadata is not global support selection |

`Submission<C: CanSubmitWithoutReply>` deliberately suppresses the eligible
profile's reply and retires its transport. Its unit completion means local
outbound submission only. The command builders and ordinary Rust struct/enums
are both usable; the root doctest compiles a typed Action and deliberate config
submission.

Known nested profiles include integer tab/window navigation, directional
movement, titles/layouts, resizing/confirmation/detachment, scroll/prompt/mark
navigation, marker grammar, clear-terminal modes, font operations, opacity,
new-process/launch argv, named kitten argv, config paths/colors argv and signals.
No-arg vocabulary covers the common tab/window, clipboard, fullscreen,
scrollback, marker/debug/config actions. Context-dependent mouse operations,
nested escaped paste/send-text/send-key and other unmodeled action grammar
remain honestly available through `ActionSpec::Raw` rather than alleged parsed
profiles. Outer RC literal SendText/SendKey are implemented.

Important grammar distinction: title/layout use unquoted remainder strings;
kitten/launch/new-process use shell token quoting **and then upstream server
tilde/environment expansion** through `to_cmdline`. Quoting preserves token
boundaries, not exact literal values after expansion. Use RC Launch/Kitten JSON
lists for literal argv. Config/color action parsers use shlex without that
post-split helper. Negative `goto_tab` wire values are not transformed twice;
the native parser owns its backwards-compatible adjustment.

## Honest outcome and reuse boundaries

- A reply-bearing built-in finite profile opts into audited transport reuse,
  never replay. Submission-only profiles, raw action/finite/kitten and custom
  marker functions default to retirement. No action effects are scheduled or
  ordered across connections by this pure crate.
- Action ack reports dispatch consumption, not completed UI/child lifetime.
  `create_marker` mappable action opens an interactive ask kitten, whereas the
  outer RC `create-marker` sets supplied pattern/function data directly.
- Kitten only models the outer native string/bool/absent result. No deployment,
  availability, arbitrary structured return or UI completion is inferred.
- Required listing IDs and OS/tab child containers now fail when absent/null
  instead of fabricating empty success. The existing Option<u64> public field
  representation remains for current observation composition, but the parser
  enforces identity presence. Optional descriptive fields and additive fields
  remain tolerant. GetText/launch/select/screenshot/run required data is strict.
- Resize needed a subtle source correction: `string_return_is_error` is a
  **reference-client interpretation**, not a native envelope rewrite. The
  server can return `ok:true` with a layout-refusal string. `ResizeResult::Rejected`
  preserves that semantic failure distinct from RC `ok:false`, bool and absent
  reset results. Process nonzero exit is likewise native result data, not RC
  failure. Execution still classifies native envelope errors before decoding.
- Empty screenshot server paths would change result kind to image data;
  `ServerPath` rejects that accidental profile mismatch. PNG output validates
  base64 and PNG signature, not full image-file integrity.
- `load-config` can invalidate caller-maintained config/layout/style/action
  interpretations. This crate owns no caches and invents no automatic refresh,
  cleanup or convergence.

## Upload boundary and precise next seam

`run` empty stdin is **not** a safe no-upload shortcut: it still calls shared
`StreamInFlight.handle_data`, and a different stream ID aborts the old stream.
No `RunEmpty` command was shipped. `RunOutput::decode` requires stdout/stderr
standard base64 and both signed exit fields, preserves arbitrary bytes, and
tolerates additive result fields for the future run profile.

The separate xhigh wave must add same-server/same-command exclusion keyed at
admission/preparation and a real upload conversation with handshake, chunk/end,
deferred completion where appropriate, quotas and cancellation evidence. Even
empty run input must acquire that lane. Proposed command-profile seam is an
optional explicit exclusion key plus the dedicated upload profile; this agent
stopped at that consequential shared-engine boundary instead of changing it.

Finite background remove/index and logo removal never enter the accumulator.
PNG upload, run stdin, file/stdin/broadcast text conversations, and HEAD
resize-edge/dock/named-workspace functionality remain unimplemented. The catalog
marks these boundaries instead of claiming all features of a familiar command
name. Supported-version live matrix remains the native support ticket's work.

## Verification actually performed

Toolchain: user-selected `mise exec -- rustc --version` returned
`rustc 1.101.0-nightly (822182db4 2026-10-13)` for nightly-2026-10-14. No new
rust-toolchain pin or rust-version metadata was introduced.

- `mise exec -- cargo nextest run -p kitty-rc-commands`: **30/30 passed**.
- `mise exec -- cargo clippy -p kitty-rc-commands --all-targets -- -D warnings`: passed.
- `python3 design/library-redesign/capabilities/verify-source0.sol61m.py /home/rektide/archive/kovidgoyal/kitty`: passed. It executes 15 actual pinned source constructors and 7 actual nested action parsers through AST without importing native kitty or mutating any terminal; verifies exact stable catalog membership. This is not mere serializer roundtrip testing.
- Final `mise exec -- cargo check --workspace --all-targets --locked`: passed.
- Final `mise exec -- cargo nextest run --workspace`: **124/124 passed**, including concurrent access/observation work visible at this gate; no skipped tests.
- Final `mise exec -- cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `mise exec -- cargo test --workspace --doc`: passed, including one new builder doctest.
- `mise exec -- cargo fmt --all --check`: passed.

No new live kitty mutations, no delegated subagents, no push/squash/jj edit.
Root shared manifest/lock contain the narrow `base64 = "0.22"` addition mixed
with concurrent access work; those shared files are deliberately left for
parent integration, not wholesale committed by this agent. Owned commits
include only protocol/commands and this capability documentation subtree.

## Cross-references

- [Inventory](inventory0.sol61m.md) ⇄ this report: complete source mapping and actual implementation standing.
- [Source verifier](verify-source0.sol61m.py): reproducible independent golden-case evidence.
- [Command contract tests](../../../protocol/commands/tests/contracts.rs): typed output/negative/domain/mode fixtures and metadata completeness.
- [Draft 1](../draft1.gpt6a.md#6-commands-include-the-larger-surface-inside-action): nested capability architecture realized without a giant wrapper zoo.
- [First implementation corrections](../implementation0.sol61m.md#review-driven-corrections): shared engine classification and transport reuse obligations reused, not rebuilt here.
