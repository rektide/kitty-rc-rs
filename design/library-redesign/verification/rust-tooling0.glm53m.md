---
type: VerificationReport
title: Rust verification tooling conversion
description: Python runners/fixtures/matrix/oracles translated to the kitty-rc-verify crate; retained Python boundaries, evidence continuity and new full-matrix observations.
resource: /design/library-redesign/verification/rust-tooling0.glm53m.md
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5bc3061ffeQLd390Klwo3Esr"
    title: Commissioned translation of project-owned verification orchestration into a Rust crate
  - resource: /design/library-redesign/verification/live0.sol61xh.md
    title: Released matrix ownership, exact binary provenance and conversion contracts
  - resource: /design/library-redesign/uploads/implementation0.sol61xh.md
    title: Released upload runner/example ownership and suppression-lane policy
  - resource: /tool/verification/README.md
    title: The maintained Rust verification guide replacing the Python entrances
  - resource: /scripts/conformance/README.md
    title: Superseded Python guide (evidence retained; paths are historical)
verification:
  scope: New kitty-rc-verify crate (35 lifecycle/unit tests), converted examples, retained-boundary oracle, real finite/uploads/matrix/source runs and converged workspace gates
  evidence: 220/220 workspace nextest, locked workspace all-target build, strict all-target clippy, fmt, doctests; live 0.48.2 finite+uploads; full 3-release encrypted+plain matrix with uploads and CLI smoke; pinned-source oracle parity
  limits: No oldest-supported guarantee, no whole-catalog/all-options claim, no HEAD promotion; plain uploads still not claimed (plain runs omit the upload harness by design)
---

# Rust verification tooling conversion

Work anchor: `kitty-rc-proto-rs-redesign-verification-live`. The human
commissioned translating "some of this verification crap" out of Python;
this report records what became Rust, what deliberately did not, and the
first actual evidence produced by the new tooling.

## What was replaced

| Retired Python | Rust replacement |
|---|---|
| `scripts/verify-isolated-kitty.py` | `kitty-rc-verify finite` |
| `scripts/verify-isolated-uploads.py` | `kitty-rc-verify uploads` |
| `scripts/verify-supported-kitty.py` | `kitty-rc-verify matrix` |
| `scripts/conformance/download-kitty.py` | `kitty-rc-verify download` (`src/download.rs`: ureq fetch, SHA-256 verify, safe tar.xz extraction, provenance JSON) |
| `scripts/conformance/cli-smoke.py` | `kitty-rc-verify cli-smoke` (`src/smoke.rs`) |
| `scripts/conformance/fixture-kitty.py`, `fixture-cargo.py` | **Deleted without successor**: the Rust runner composes kitty's config and executes prebuilt example binaries directly, so PATH adapters no longer exist. Their safety properties became Rust behavior and tests: config augmentation happens in `src/fixture.rs` before kitty starts, and the private-prefix validation lives in the examples and `tests/fixtures/fake_kitty.rs`. |
| `scripts/conformance/test_matrix.py` | Rust tests in `tool/verification` (release parsing, ambient stripping, fixture refusal, bounded behavior) |
| `design/library-redesign/capabilities/verify-source0.sol61m.py` | `kitty-rc-verify source` capabilities profile (`src/source.rs`) |
| `design/library-redesign/uploads/verify-source0.sol61xh.py` | `kitty-rc-verify source` uploads profile |
| `design/library-redesign/probes/compare0.sol61m.py` | `kitty-rc-verify kubeless-compare` (`src/kubeless.rs`) |
| Python `-c` children (key publication, sentinel, run streams) | The real Rust binary `kitty-rc-verify-child` launched by kitty inside the private window; examples receive it via `KITTY_RC_MATRIX_CHILD` |

Converted examples: `client/examples/supported_matrix.rs` (sentinel child now
the fixture-child binary; accepts the new `kitty-rc-fixture-` prefix plus the
two legacy prefixes) and `control/registry/examples/upload_conformance.rs`
(`stream`/`sleep` children). Both keep their policy comments and arguments
contracts (SOCKET KEY PASSWORD [PNG]); only the child process changed.

## Deliberately retained Python (and why it is honest)

- **`tool/verification/fixtures/no_ui.py`** — an upstream kitten. Kitty
  plugins are Python executed by kitty; a Rust executable cannot test the
  native kitten boundary. The file stays a real, inspectable fixture copied
  verbatim into each private fixture (never hidden inside a string literal).
- **`tool/verification/fixtures/source_oracle.py`** — a ~100-line executor
  that extracts named upstream methods at the pinned revision and runs them.
  Upstream constructors/parsers can only be a *semantic* oracle if the
  actual upstream code runs; translating their (GPLv3) algorithms into this
  repository would be a licensing and methodology error, and grepping source
  text would be weaker evidence pretending to be equivalent. All cases,
  comparisons, structural chunk checks and reports live in Rust; the oracle
  only executes upstream code and reports results over JSON.

Everything else — lifecycle, deadlines, teardown, decoding, errors, fixture
scheduling, paths, environment sanitation, matrix reports, downloads,
extraction safety, CLI journeys — is Rust with structured `thiserror`
sources; no error is caught merely to print it.

## Invariants preserved (with one correction)

Private headless Weston/kitty only; explicit executables (`--kitty` or
`KITTY_RC_TEST_KITTY`); no ambient endpoint/key/config/mutation — every
spawned child receives a **fully replaced** environment, defensively
re-filtered inside `fixture_environment` so even an unsanitized base cannot
leak `KITTY_*`/display entrances (a lifecycle test poisons the base and
proves the strip). Startup bounded at 30 s (the handoff's 5-second failure
record is preserved in live0). Owned groups terminate only through owned
handles; SIGTERM/SIGINT→SIGKILL escalation with a `Drop` group-kill backstop
covers cancellation and panic paths; clean teardown removes the fixture
directory, failure paths retain `process.log`. Fixture passwords are local
test material and are redacted from all recorded output (tested). Release
digests/revisions recorded; the envelope stays `[0,43,1]`.

Correction discovered during translation: fixture directories moved from the
long repo-scratch paths to `/tmp/opencode` parents (matching the retired
runners' actual behavior) because Wayland socket paths under
`.test-agent/supported-kitty/…` can exceed `sockaddr_un`'s 108-byte limit —
the fake-weston lifecycle test caught this before a real run could.

## Actual evidence produced by the Rust tooling (2026-10-14)

All with the user's `mise` selection (`rustc 1.101.0-nightly`).

- **`kitty-rc-verify finite`**: installed 0.48.2, private Weston/kitty,
  encrypted conformance passed — with the fixture key published by the Rust
  child binary.
- **`kitty-rc-verify uploads`**: installed 0.48.2, encrypted PNG
  background/logo, suppressed-terminal handshake, binary
  empty/small/exact/large run (Rust `stream` child: exact byte round trip,
  `00ff80` stderr, exit 7) and managed aliases passed.
- **`kitty-rc-verify source ~/archive/kovidgoyal/kitty`**: pinned
  `b2cb8712…` — 41-command inventory parity with the modeled catalog
  (minus deliberate raw `resize-window-edge`), 15 constructor golden cases,
  7 nested parsers, and the upload image/run chunk contracts (512-byte image
  chunks, one final empty frame, 4096-byte run chunks). This is the same
  coverage the two retired design scripts asserted, now from one entrance.
- **`kitty-rc-verify matrix --cli-smoke`**: **full matrix on all three
  releases × encrypted+plain**, extending the previous preparation
  checkpoint (which was finite-only) with the previously-unfinished pieces:

| Server | Encrypted | Plain | CLI smoke |
|---|---|---|---|
| installed 0.48.2 | conformance+matrix+uploads pass; 12 case passes, 2 documented skips (0.49 gates) | supported_matrix pass; 12/2 | pass both modes |
| 0.43.1 (cached archive) | conformance+matrix+uploads pass; 11 passes, 3 skips (0.48/0.49 gates) | supported_matrix pass; 11/3 | pass both modes |
| 0.49.2 (cached archive) | all pass incl. screenshot PNG reply+private file and OS-title | 14/0 | pass both modes |

  Encrypted **uploads now pass on all three releases** — the upload-version
  matrix the handoff listed as remaining work. Plain profiles deliberately
  run only `supported_matrix` (plain uploads remain unclaimed, as before).
  Report: `.test-agent/supported-kitty/matrix.json` with per-profile case
  records, harness results, executable/example SHA-256 digests, provenance
  (release binaries carry their published source revisions; the installed
  binary explicitly records "no source revision inferred"), the selected
  rustc report and sanitized outputs. No test password appears in any
  profile output.
- **Converged workspace gates**: `cargo fmt --all -- --check`,
  locked workspace all-target build, `cargo nextest run --workspace`
  (**220/220**, up from 185 — +35 new tests), strict
  `cargo clippy --workspace --all-targets -- -D warnings`, and workspace
  doctests all pass.

Old Python witnesses (`preparation.json`, `preparation-installed.json`,
live0's tables) remain intact as pre-conversion evidence; the new
`matrix.json` is the first post-conversion full-matrix witness. The
preparation checkpoint's 68-pass/10-skip observation and this run agree on
every overlapping profile.

## Checks not rerun / not claimed

- The matrix was not run with `--download` (cached verified archives were
  reused; the download path is exercised by unit tests plus the original
  acquisition evidence).
- Ctrl-C interruption is implemented (SIGINT to owned groups, report
  best-effort, exit 130) but not exercised in this report.
- Plain-mode uploads, whole-catalog/all-options coverage, oldest-supported
  guarantees and HEAD features remain unclaimed, per the
  [support policy](../../../doc/usage/support.md).

## Parent integration notes

- Root `README.md` still says `python3 scripts/verify-isolated-kitty.py`;
  the maintained entrance is now
  `mise exec -- cargo run -p kitty-rc-verify -- …`. Suggested replacements:
  finite/uploads/matrix lines under "Building and testing".
- The superseded [scripts/conformance guide](../../../scripts/conformance/README.md)
  is deleted with its directory; historical reports referencing `scripts/*`
  paths keep them as evidence of what ran then.
- `design/library-redesign/README.md` should link this report as the current
  verification-tooling record.
- New dependencies (`tar`, `xz2`, `ureq`, direct `libc`) were added only to
  `kitty-rc-verify`; no leaf crate changed.
- Tracker: `kitty-rc-proto-rs-redesign-verification-live` can close for the
  Rust-tooling conversion plus this full matrix/gate evidence; continuing
  matrix maintenance is ordinary follow-on work, not an open conversion.

## Cross-references

- [Support boundary](../../../doc/usage/support.md) ⇄ matrix gates.
- [Live matrix handoff](live0.sol61xh.md): pre-conversion evidence and the
  conversion contracts this work satisfied.
- [Upload implementation](../uploads/implementation0.sol61xh.md): released
  upload ownership; suppression/interruption lane comments preserved.
- [Rust verification guide](../../../tool/verification/README.md): the
  maintained entrances and lifecycle description.
