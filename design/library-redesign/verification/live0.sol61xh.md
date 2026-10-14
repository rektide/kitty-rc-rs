---
type: VerificationReport
title: Isolated release-profile live matrix
description: Actual binary provenance, private finite/nested conformance and final integration gate evidence.
resource: /design/library-redesign/verification/live0.sol61xh.md
status: preparation
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5ccabb4ffeiXGEem6BYMTT3a"
    title: Independent binary provenance, finite/nested live matrix and checkpoint for human-directed Rust runner conversion
  - resource: /doc/usage/support.md
    title: Provisional evidence-based support policy, no oldest guarantee
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Stable audited catalog and documented capability-specific release gates
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Typed finite and nested profiles with explicit output/reuse boundaries
  - resource: /design/library-redesign/implementation0.sol61m.md#review-driven-corrections
    title: Corrected reusable-session and managed pool live predecessor
  - resource: /design/library-redesign/uploads/implementation0.sol61xh.md
    title: Separate upload owner's conversation, source quotas and live image/run harness
  - resource: https://api.github.com/repos/kovidgoyal/kitty/releases/tags/v0.43.1
    title: Actual upstream x86-64 asset URL and published SHA-256, read 2026-10-14
  - resource: https://api.github.com/repos/kovidgoyal/kitty/releases/tags/v0.49.2
    title: Actual upstream stable x86-64 asset URL and published SHA-256, read 2026-10-14
verification:
  scope: Preparation checkpoint; finite/nested profiles exercised, final converged upload and workspace gates pending
  limits: No oldest-supported guarantee, no whole catalog/all-options claim, no untested HEAD promotion
---

# Isolated release-profile conformance

Work anchor: `kitty-rc-proto-rs-redesign-verification-live`.

## Preparation checkpoint

The checkpointed [matrix prototype](../../../scripts/verify-supported-kitty.py) and
its [guide](../../../scripts/conformance/README.md) select exact binaries without
touching user installation/config. Acquisition uses ignored
`.test-agent/supported-kitty/`; no broad archive fetching or source build was
needed. Parent owns the shared design index/root README/tracker export.

The preliminary finite matrix exercised baseline **0.43.1**, installed **0.48.2**
and source-audited stable **0.49.2** in encrypted and explicit plain modes. One
initial installed/encrypted fixture exceeded the predecessor runner's 5-second
startup wait before publishing a key; it made **no protocol claim**. Composition
now uses the upload owner's 30-second-startup private runner rather than editing
the other owner's script. The new composition's targeted installed/encrypted
finite run passed. **Human steering now selects maintained Rust verification
tooling.** Python orchestration is preserved as a checkpoint for G's conversion,
not selected as the maintained entrance. Final combined upload/CLI matrix and
converged gates await that conversion and the upload owner's release signal;
this ticket is not yet closed.

## Actual executable provenance

All `--version` strings reported `kitty <version> created by Kovid Goyal`.

| Server | Executable | SHA-256 |
|---|---|---|
| Installed 0.48.2 | `/usr/bin/kitty` | `6d345df83951b65ee51436c826cc6ddf045868f25a0571b04170482d71497f35` |
| Baseline 0.43.1 | `.test-agent/supported-kitty/0.43.1/app/bin/kitty` | `24507383f3e540c8ca7f7932a036994b73d833ea8c4a8975c545721a5490308a` |
| Stable 0.49.2 | `.test-agent/supported-kitty/0.49.2/app/bin/kitty` | `855d26487561265eef00a1c44056c74b5df3b988a252d8bdec07231b0b8d0a6e` |

Release artifacts, not locally installed replacements:

| Version | Source tag's peeled commit (archived source) | Full upstream archive SHA-256 |
|---|---|---|
| 0.43.1 | `7f24dd68c98d8f4712457560f25e8056cce52ec8` | `f2026ef8cec6fd21b58e3ae501f707c2fc922ddb9b3e79685917fc60197449fb` |
| 0.49.2 | `b2cb871254fbdac19a085188813c8dfb258b95cd` | `d573618b911e9c461bd421b96c13c74c7f1cb2f1ac9c327818d4ff84366cf5c6` |

Download URLs are
`https://github.com/kovidgoyal/kitty/releases/download/v<VERSION>/kitty-<VERSION>-x86_64.txz`.
Both downloaded archive digests matched the actual release API metadata. This
records HTTPS publisher provenance and content integrity, **not independently
verified signatures**. Installed 0.48.2 source/package revision is not known;
the version and executable hash are the supplied evidence. The asset archive
hash binds bundled libraries; the installed launcher hash alone does not bind
all distribution files.

## Prepared case profiles

`client/examples/supported_matrix.rs` uses explicit private Unix endpoints and
the unchanged `[0,43,1]` envelope. Each JSON case records profile, duration and
native-output checks; the fixture record supplies direct/managed option limits.
The same private initial window supplies target
IDs; no target comes from a human environment. Each fixture owns separate
Weston/kitty/config/key/child/XDG dirs, and tears down only its own processes.

- Five repeated direct listings preserve one connection token.
- Five managed listings preserve one idle transport token, then a launched child
  writes a private sentinel and sleeps 1.5 seconds. Focus and screen text finish
  **while the long operation is still Running**, with `OutboundComplete` progress,
  before its exact exit-string `7`. Three concurrent leases return three idle
  transports. Timings are observations, not throughput/parallel-server promises.
- An impossible private window ID produces classified `Failure::Remote` with
  nonempty native bytes; a clean boundary remains reusable for the next listing.
- Finite window title/user-vars are observed through listing; nested title is
  also observed and its dispatch transport retires.
- Scoped colors are read back as `#112233`; private config reload override is
  independently read back as configured `#223344`.
- Independent no-ui kitten returns exact literal argv (spaces, dollar expression,
  Unicode) as native string and native bool. Nested kitten token boundaries are
  observed through a private file, without claiming no server-side expansion.
- Suppressed focus returns local submission only, retires its transport, and a
  separately explicit listing succeeds. No server-effect proof is inferred.
- Alternate text profile is gated to 0.48+, without asserting unseen alternate
  content. OS-window title and screenshot are gated to 0.49+. Stable screenshot
  binary reply and actual saved private file both have PNG signature/nonempty
  bytes. OS-title acknowledgment does not prove window-manager title display.

The plain profile selects `allow_remote_control yes`; encrypted profile selects
password mode and only the private child's published key. This is independent
native kitty decryption evidence, not a same-module crypto round trip.

## Verification currently performed

- User-selected `mise exec -- rustc --version --verbose`: nightly-2026-10-14,
  `rustc 1.101.0-nightly`, commit `822182db4c4a1827256881d8c7aa685f0bcf2b11`
  (2026-10-13), x86_64-unknown-linux-gnu. No pin/install/MSRV change.
- Preparation unit tests: **4/4 passed**, no kitty contacted. They verify release
  parsing, removal of ambient terminal/credential/display/bus entrances, refusal
  of non-private config before writes and refusal of arbitrary cargo invocation.
- Owned Rust example targeted build and formatting passed.
- Preliminary finite profiles passed on all three releases and both modes,
  after the installed/encrypted startup blocker was resolved by runner composition.
  Scratch witnesses: `preparation.json`, `preparation-installed.json`.

Across those two witnesses the six version/mode profiles have 68 passing case
records (including private-fixture/setup records) and 10 deliberate feature-gate
skips. This is a combined **preparation** observation, not one converged final
matrix. Observed focus/text durations while the sentinel child remained pending:

| Server | Encrypted short work | Plain short work | Screenshot |
|---|---|---|---|
| 0.43.1 | 7 ms | 8 ms | skipped by 0.49 gate |
| 0.48.2 | 13 ms | 9 ms | skipped by 0.49 gate |
| 0.49.2 | 7 ms | 2 ms | 3891-byte PNG reply and private saved PNG, both modes |

No runtime compatibility failure was observed in those selected profiles. The
5-second startup failure was a fixture-readiness blocker, resolved by existing
runner composition; it is not hidden or misclassified as a protocol rejection.

## Rust conversion handoff

Released to the parent-assigned G agent after this explicit-path checkpoint:
`scripts/verify-supported-kitty.py`, all `scripts/conformance/` files, and
`client/examples/supported_matrix.rs` if its embedded child needs conversion.
This report remains evidence-owned here until the parent coordinates final
updates; `doc/usage/support.md` has **not** been modified. No other owner's
uncommitted runner/core/CLI source is included in this checkpoint.

Important contracts to preserve, not reasons to retain Python orchestration:

1. **No ambient targeting.** Explicit release executable, private Weston/kitty
   process ownership, config/runtime/key/socket/XDG isolation, checked private
   fixtures, bounded startup, owned teardown, Ctrl-C/deadline handling. Initial
   runner startup 5 seconds was insufficient once; upload runner uses 30 seconds.
2. **Actual coordinates.** Reuse the already acquired archives/extracted apps
   and `provenance.json` in ignored scratch. Record exact versions/hashes/source
   revisions, user Rust, executable harness hashes and profile-level JSON outcomes.
   No HEAD/minimum/all-catalog claims; leave the envelope `[0,43,1]` unchanged.
3. **Independent Rust journeys.** `supported_matrix` receives socket/key/local
   test password and uses `KITTY_RC_MATRIX_VERSION`, `KITTY_RC_MATRIX_PROTECTION`
   and `KITTY_RC_MATRIX_MARKER`. The marker is only fixture text, never a human
   window selector. `conformance` receives the same three arguments;
   `upload_conformance` additionally receives the independent valid PNG path.
4. **Finite evidence already exists.** Preserve all recorded cases above,
   especially sentinel-before-short-work ordering, actual long `Running`
   status/progress, direct/managed token equality, clean remote rejection,
   nested/kitten retirement, config/color observations, no-response semantics
   and exact profile gates. G can replace the runner-facing private-prefix
   validation coherently; the current prefixes are `kitty-rc-live-` and
   `kitty-private-upload-`, socket name `rc.sock`.
5. **Use Rust fixtures, not Python strings.** The current sentinel child is a
   Python `-c` string inside the Rust example. Existing runners publish a key
   through another Python child. These are replaceable by a Rust fixture-child
   binary (publish env key, print marker, create sentinel, sleep/exit; binary
   stdin/stdout cases if useful). Existing upload example also invokes Python
   children; coordinate that released example with the upload owner/parent.
6. **One genuinely native Python surface.** `scripts/conformance/no-ui.py` is an
   independent upstream **Python kitten** loaded by kitty, not orchestration.
   It verifies native string/bool results, literal outer argv and nested token
   boundaries via an owned file. Keeping this tiny fixture is honest; pretending
   a Rust executable tests the same Python-kitten boundary would not be.
7. **Prepared CLI smoke, not executed yet.** The checkpointed helper selects
   explicit key/password sources and the 16 MiB advertised response budget,
   then list/explicit-tab-active/goto/two watch samples. Translate it to Rust,
   run the actual final binary, and preserve valid no-focused sample semantics.
8. **Final work still required.** After upload release, build final Rust tooling
   and run the three versions with encrypted and explicit plain finite profiles.
   Repeat encrypted PNG background/logo, suppressed-terminal handshake,
   binary empty/1/4096/16385-byte run and managed alias/text profiles through
   `upload_conformance`; plain uploads were not exercised. Run converged selected
   `mise` formatting, locked workspace all-target check, nextest, strict all-target
   clippy and workspace doctests including CLI. This agent has **not run those
   final gates or closed the live ticket**.

The four prototype preparation tests can become Rust tests for release parsing,
ambient-source removal, refusal of non-private config before writes and rejection
of arbitrary runner invocation. Do not needlessly preserve the PATH/cargo adapter
mechanism if the Rust lifecycle runner can directly execute prebuilt examples.

## Boundaries and cross-references

Selected successful 0.43.1 profiles do **not** establish oldest availability,
whole-catalog compatibility or all option variants. Feature-specific skips
preserve the support policy; no envelope bump or silent downgraded success.
No HEAD binary/source promotion is part of this verification.

- [Support boundary](../../../doc/usage/support.md) ⇄ final profile evidence:
  wire baseline, stable source audit and exercised server coordinates stay distinct.
- [Upload report](../uploads/implementation0.sol61xh.md): source/scripted fault
  coverage and exact binary live profiles; the final matrix will record which
  of those native journeys were actually repeated per release.
- [CLI report](../cli/implementation0.sol61m.md): watch absence and response-budget
  corrections; optional private live smoke tests their converged standalone path.
- [Capability inventory](../capabilities/inventory0.sol61m.md): omitted commands,
  unknown earliest availability, nested dispatch and profile-specific gates.
