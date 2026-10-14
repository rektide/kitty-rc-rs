---
type: VerificationGuide
title: Rust private verification tooling
description: Isolated fixture lifecycle, supported-version matrix, release acquisition, CLI smoke and pinned source oracles in one Rust CLI.
resource: /tool/verification/README.md
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5bc3061ffeQLd390Klwo3Esr"
    title: Commissioned Rust replacement of the Python verification runners, fixtures and matrix orchestration
  - resource: /design/library-redesign/verification/live0.sol61xh.md
    title: Released Python matrix evidence, exact provenance and Rust-conversion handoff contracts
  - resource: /design/library-redesign/uploads/implementation0.sol61xh.md
    title: Released upload runner and example ownership; suppression/interruption lane policy
  - resource: /doc/usage/support.md
    title: Provisional evidence-based support boundary; no oldest-release guarantee
  - resource: /scripts/conformance/README.md
    title: Superseded Python conformance guide whose entrances this crate replaces
---

# kitty-rc-verify — private Rust verification tooling

Work anchor: `kitty-rc-proto-rs-redesign-verification-live`.

One crate owns the whole verification lifecycle that previously lived in
`scripts/*.py`: bounded process supervision, private headless fixtures, the
supported-version matrix, pinned release acquisition, standalone CLI smoke
journeys, the kubeless design probe and the pinned source-contract oracle.
Run everything through the user's selected Rust:

```sh
mise exec -- cargo run -p kitty-rc-verify -- <subcommand>
```

Prerequisites for live runs: headless Weston (`weston --backend=headless`) and
an explicit kitty executable. Nothing is installed into user/system
locations, no user kitty config is touched, and no ambient terminal, key or
endpoint is ever selected.

## Entrances

| Subcommand | What it does |
|---|---|
| `finite` | One private Weston+kitty fixture; runs the encrypted `conformance` example (direct/managed reuse, concurrent launch-wait/text/focus). |
| `uploads` | One private fixture with an independently generated valid PNG; runs the encrypted `upload_conformance` example (PNG background/logo, suppressed-terminal handshake, binary run journeys, managed aliases). |
| `matrix` | Opt-in supported-version matrix: selected executables × protection modes, prebuilt examples, JSON evidence report. `--download`, `--kitty` (repeat), `--mode` (repeat), `--finite-only`, `--cli-smoke`, `--no-build`, `--report`, `--deadline`. |
| `download` | Acquire pinned upstream releases (0.43.1, 0.49.2) into ignored `.test-agent/supported-kitty` with digest verification, safe extraction and provenance JSON. Already-acquired matching archives are reused without network access. |
| `cli-smoke SOCKET KEY PASSWORD` | Standalone `kitty-rc` CLI journeys (list, explicit-tab active, goto, two finite watch samples) against one existing private fixture. |
| `source CHECKOUT` | Pinned upstream source contracts at revision `b2cb871254…`: catalog inventory, 15 constructor golden cases, 7 nested parsers, upload chunking contracts. |
| `kubeless-compare KUBELESS_DIR` | The design comparison probe (disposable scratch package only). |

Typical matrix usage (cached release binaries are reused, not re-downloaded):

```sh
mise exec -- cargo run -p kitty-rc-verify -- matrix --cli-smoke
mise exec -- cargo run -p kitty-rc-verify -- matrix --finite-only --kitty /usr/bin/kitty --mode plain
```

## Lifecycle and ownership

Each fixture owns one headless Weston and one kitty process, each in its own
process group, plus a unique `kitty-rc-fixture-*` directory holding
`kitty.conf`, private XDG config/cache/data, the RC socket, the published key
and a combined process log. Every spawned child gets a **fully replaced**
environment: ambient `KITTY_*`, `DISPLAY`, `WAYLAND_DISPLAY`,
`XDG_RUNTIME_DIR` and desktop-bus variables are stripped defensively even if
a caller passes an unsanitized base. Startup is bounded (default 30 s — the
retired 5-second bound failed once) and teardown is guaranteed on every
path: normal exits use SIGTERM→SIGKILL per owned group, deadlines/Ctrl-C use
SIGINT→SIGKILL, and a `Drop` backstop group-kills anything a cancelled or
panicking future leaves behind. Clean teardown removes the fixture
directory; failure paths keep it (with `process.log`) for diagnosis.

The examples are compiled **before** any server starts and executed by path;
the retired PATH-adapter scripts (`fixture-kitty.py`, `fixture-cargo.py`)
have no successor because the Rust runner composes every invocation itself.
Fixture children (key publication, sentinel, raw stream, sleep) are the real
Rust binary `kitty-rc-verify-child` launched by kitty inside the private
window — the examples receive its path via `KITTY_RC_MATRIX_CHILD`. The
conformance examples' private-endpoint validation recognizes the
`kitty-rc-fixture-` prefix (and the two legacy prefixes of the retired
Python runners).

## Evidence and gates

Finite journeys and CLI smoke run in both protection modes; the upload
harness runs in encrypted mode only, so plain uploads are never claimed.
The matrix report keeps the retired runner's JSON shape for witness
comparability: exact reported versions, executable/archive SHA-256, published
source revisions, selected `rustc --verbose`, example digests, per-profile
case records, harness results and full sanitized output. Missing executables,
failed startup and exceeded deadlines are blockers, never passes. The test
password is fixture-local and redacted from every recorded output. The wire
envelope remains `[0,43,1]`; no oldest-supported, whole-catalog or HEAD claim
is made — see the [support policy](../../doc/usage/support.md).

## The two deliberate Python boundaries

1. [`fixtures/no_ui.py`](fixtures/no_ui.py) — an upstream **kitten** (kitty
   plugins are Python, loaded by kitty itself). It is copied verbatim into
   each private fixture and exercises the native string/bool/argv kitten
   boundary; a Rust executable cannot test that boundary.
2. [`fixtures/source_oracle.py`](fixtures/source_oracle.py) — the minimal
   executor that extracts and runs upstream constructors/parsers at the
   pinned revision. Executing upstream GPL code is what makes upstream a
   *semantic* oracle; translating its algorithms into this repository would
   be both a licensing and a methodology error. Every case, comparison and
   report lives in Rust (`src/source.rs`).

## Tests

`tests/lifecycle.rs` exercises the engine with Rust fixture binaries
(`fake-kitty`, `fake-weston`, `fake-example`, `fake-cli`) — real process
supervision, environment sanitation, deadline bounds, exit propagation and
matrix evidence with zero Python. Unit tests cover version parsing, config
profiles, the generated PNG's chunk/CRC/scanline validity, release tables,
extraction path safety (including a hand-crafted parent-escape archive) and
report redaction.

## Layout

| Path | Owns |
|---|---|
| `src/main.rs` | CLI surface |
| `src/fixture.rs` | Private fixture lifecycle |
| `src/supervision.rs` | Owned process groups, bounded teardown |
| `src/matrix.rs` | Version/mode orchestration and evidence |
| `src/smoke.rs` | CLI journeys |
| `src/download.rs` | Release acquisition, digests, safe extraction, provenance |
| `src/source.rs` | Pinned source cases and comparisons |
| `src/child.rs` + `src/childbin.rs` | The fixture-child binary and its contract |
| `src/kubeless.rs` | Design comparison probe |
| `src/env.rs`, `src/png.rs`, `src/release.rs`, `src/report.rs`, `src/examples.rs` | Sanitized environments, PNG generation, release coordinates, evidence records, cargo/mise integration |
