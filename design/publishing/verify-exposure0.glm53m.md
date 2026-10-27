---
type: DesignAssessment
title: Should kitty-rc-verify be published to crates.io?
description: Exposure audit, external-value analysis and recommendation for the private verification orchestration crate ahead of the workspace's first publish wave.
resource: /design/publishing/verify-exposure0.glm53m.md
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-27
sources:
  - resource: "opencode://ses_e80f2f8ceffepJ3l93JH72ib3P"
    title: Commissioned assessment of publishing vs keeping kitty-rc-verify unpublished, with code-level exposure and coupling audit
  - resource: /tool/verification/Cargo.toml
    title: Package manifest with six binaries, registry-only dependencies and no publish field
  - resource: /tool/verification/README.md
    title: Self-described private verification guide and entrances table
  - resource: /tool/verification/src/main.rs
    title: CLI surface of seven subcommands and their workspace-coupled defaults
  - resource: /tool/verification/src/lib.rs
    title: Crate invariants and module inventory
  - resource: /tool/verification/src/examples.rs
    title: Cargo/mise integration proving matrix and single-fixture entrances build workspace packages
  - resource: /tool/verification/src/matrix.rs
    title: Matrix orchestration invoking workspace example build plans
  - resource: /tool/verification/src/source.rs
    title: Pinned-source oracle reading protocol/commands/src/catalog.rs from the workspace root
  - resource: /tool/verification/src/release.rs
    title: Pinned upstream release digests and revisions with provenance caveats
  - resource: /tool/verification/src/download.rs
    title: Standalone release acquisition with ureq fetch, SHA-256 verify, safe txz extraction and provenance JSON
  - resource: /tool/verification/src/fixture.rs
    title: Private fixture lifecycle and include_str! embedding of the kitten fixture
  - resource: /tool/verification/src/env.rs
    title: Ambient environment sanitation with defensive re-filtering
  - resource: /tool/verification/src/supervision.rs
    title: Owned process groups with libc killpg teardown escalation
  - resource: /tool/verification/src/child.rs
    title: Fixture-child binary contract (key, sentinel, stream, sleep)
  - resource: /tool/verification/tests/lifecycle.rs
    title: Lifecycle tests exercising the engine through the fake fixtures
  - resource: /tool/verification/tests/fixtures/fake_kitty.rs
    title: Fake kitty as a socket-binding lifecycle double that never speaks the RC protocol
  - resource: /tool/verification/tests/fixtures/fake_cli.rs
    title: Canned-response CLI stand-in for smoke journeys
  - resource: /tool/verification/tests/fixtures/fake_example.rs
    title: Environment-contract-checking example stand-in
  - resource: /tool/verification/tests/fixtures/fake_weston.rs
    title: Wayland-socket stand-in
  - resource: /tool/verification/fixtures/no_ui.py
    title: MIT-licensed original kitten fixture (no upstream code copied)
  - resource: /tool/verification/fixtures/source_oracle.py
    title: Minimal executor extracting upstream methods from a caller-provided checkout at runtime
  - resource: /Cargo.toml
    title: Workspace manifest confirming no kitty-rc-* dependency reaches tool/verification
  - resource: /README.md
    title: Workspace table naming the crypto decrypt oracle; no-MSRV-commitment statement
  - resource: /doc/usage/support.md
    title: Provisional evidence-based support policy the crate's entrances implement
  - resource: /design/library-redesign/verification/rust-tooling0.glm53m.md
    title: Conversion report covering retained Python boundaries and GPL-avoidance rationale
  - resource: /design/library-redesign/verification/live0.sol61xh.md
    title: Release digests provenance and evidence-policy contracts behind the pinned table
  - resource: /design/library-redesign/verification/entrances0.sol61m.md
    title: Adoption entrances policy putting the client guide, not the verifier, as the public surface
  - resource: /design/upstream-kitty/evolution0.gpt6a.md
    title: Upstream drift context (0.50dev HEAD) bearing on pinned-release aging
verification:
  scope: Read-only audit of tool/verification sources, manifest, fixtures, tests and the governing design/policy documents; cargo package --list --allow-dirty packaging check
  evidence: "Manifest dependency audit (no path deps), cargo package file listing (clean), grep audits for personal paths and secrets (none), subcommand-by-subcommand workspace-coupling trace through examples.rs/matrix.rs/source.rs/main.rs"
  limits: No build or publish dry-run performed; no legal review beyond the documented GPL-boundary rationale; ecosystem precedent cited from knowledge, not fresh registry research
---

# Should `kitty-rc-verify` be published?

Work context: the workspace's first crates.io publish wave (`kitty-rc` 0.6.0
facade plus 0.1.0 concern crates). `tool/verification` currently has **no
`publish` field**; it was tentatively flagged for `publish = false` on the
strength of its description ("Private Rust verification orchestration…"),
and the human suspects it may actually *want* public exposure. This wave
assesses publish-as-is / publish-with-changes / keep-unpublished.

**Recommendation: keep unpublished** — set `publish = false` in this wave,
and treat public exposure as a future extraction, not a flip of this crate.

## What the crate actually is

- A six-binary package plus lib ([Cargo.toml](../../tool/verification/Cargo.toml)):
  the `kitty-rc-verify` CLI (7 subcommands), the `kitty-rc-verify-child`
  fixture child (key publication, sentinel, byte-exact stream, sleep), and
  four test doubles `fake-kitty`/`fake-weston`/`fake-example`/`fake-cli`.
- The lib owns: private headless Weston+kitty fixture lifecycle
  ([fixture.rs](../../tool/verification/src/fixture.rs)), owned process-group
  supervision with SIGTERM/SIGINT→SIGKILL escalation and a `Drop` backstop
  ([supervision.rs](../../tool/verification/src/supervision.rs), via
  `libc::killpg`), defensive environment sanitation
  ([env.rs](../../tool/verification/src/env.rs)), cargo/mise integration that
  builds workspace examples *before* any server starts
  ([examples.rs](../../tool/verification/src/examples.rs)), the
  supported-version matrix with JSON evidence, pinned upstream release
  acquisition (SHA-256 verification, path-escape-rejecting txz extraction,
  provenance JSON with `signature_verified: false`
  ([download.rs](../../tool/verification/src/download.rs),
  [release.rs](../../tool/verification/src/release.rs))), CLI smoke journeys,
  the pinned-source oracle cases
  ([source.rs](../../tool/verification/src/source.rs)), and the kubeless
  design probe.
- Two deliberate Python boundaries: `fixtures/no_ui.py` (an original,
  MIT-licensed kitten fixture, embedded via `include_str!` at
  `fixture.rs:240`) and `fixtures/source_oracle.py` (a minimal executor that
  extracts upstream methods from a **caller-provided** checkout via
  `git show REV:path` at runtime — no GPL text ships inside the crate).
- It depends on **no `kitty-rc-*` crate at all**: every dependency
  (`base64`, `clap`, `libc`, `serde`, `serde_json`, `sha2`, `tar`,
  `thiserror`, `tokio`, `ureq 2`, `xz2 0.1`; dev: `tempfile`) is a registry
  crate, so there is no path-dep publish blocker — verified against the
  manifest and the root workspace table.

## Exposure-risk audit: nothing blocks on secrecy or law

This is the dimension the "Private" wording raised, and it comes back clean:

- **No secrets or credentials.** Fixture passwords are explicit test
  material (`test-secret`, `upload-test-secret`), redacted from all recorded
  output; the redaction is itself tested
  ([lifecycle.rs](../../tool/verification/tests/lifecycle.rs) smoke-failure
  test). The fake key is a literal `1:test-fixture-public-key`.
- **No personal or machine-bound paths.** Grep for `/home/rektide`,
  `rektide`, `~/` across `tool/verification` finds nothing. The closest is
  the `/tmp/opencode` scratch default (`main.rs:52`, `main.rs:251`,
  `source.rs:413`) — an operator-environment convention, but a plain,
  overridable absolute temp path, not private information.
- **No vendored upstream code.** The GPL boundary is explicit and sound:
  upstream constructors/parsers are executed from a user-supplied checkout at
  a pinned revision, never translated or embedded
  ([rust-tooling0](../library-redesign/verification/rust-tooling0.glm53m.md)
  §"Deliberately retained Python";
  [source_oracle.py](../../tool/verification/fixtures/source_oracle.py)).
- **Pinned digests are public data, honestly labeled.** The release table
  records upstream archive SHA-256 values "published by GitHub's release API,
  read 2026-10-14" and explicitly declines signature claims
  ([release.rs](../../tool/verification/src/release.rs),
  [live0](../library-redesign/verification/live0.sol61xh.md) §"Actual
  executable provenance").
- **Package contents are clean.** `cargo package -p kitty-rc-verify --list
  --allow-dirty` shows only manifests, README, the two Python fixtures,
  `src/`, and `tests/` — no scratch artifacts, no logs, no personal files.

So "private" in the description describes *audience intent*, not hidden
content. The publish question is therefore about value and expectations, not
safety.

## The decisive finding: the tool is workspace-coupled

Tracing each subcommand to what it needs at runtime:

| Subcommand | Works outside this repo's checkout? | Evidence |
|---|---|---|
| `finite`, `uploads` | No — runs `cargo build -p kitty-rc[kitty-rc-registry] --example …` and resolves artifacts from the workspace target dir | [main.rs](../../tool/verification/src/main.rs) `run_single_example`; [examples.rs](../../tool/verification/src/examples.rs) `single_example_build_plan` |
| `matrix` | No — same cargo build plans over workspace packages | [matrix.rs](../../tool/verification/src/matrix.rs) lines 147–173: `Cargo::detect` + `matrix_build_plans` |
| `cli-smoke` | No — builds `-p kitty-rc-cli --bin kitty-rc` | [main.rs](../../tool/verification/src/main.rs) `cli_smoke` |
| `source` | No — reads `protocol/commands/src/catalog.rs` relative to the workspace root to compare inventories | [source.rs](../../tool/verification/src/source.rs) line 353 |
| `kubeless-compare` | Technically takes any checkout path, but compares against the private `kitty-kubeless` project | [main.rs](../../tool/verification/src/main.rs) `KubelessArgs` |
| `download` | **Yes** — fully standalone network + scratch | [download.rs](../../tool/verification/src/download.rs) |

Five of seven subcommands assume the source checkout. A stranger running
`cargo install kitty-rc-verify` gets a tool whose entrances mostly fail with
cargo-metadata errors outside the repository. Publishing it as-is would ship
a public artifact that is broken-by-design for its public audience.

## The hypothesized external value mostly lives elsewhere

- **The fake fixtures are not protocol conformance doubles.**
  [fake_kitty.rs](../../tool/verification/tests/fixtures/fake_kitty.rs)
  validates a recognized fixture config, binds the RC socket, exports a fake
  public key, and launches the child — it never *accepts an RC connection or
  answers a single framed command*. `fake-cli` and `fake-example` return
  canned JSON. They are lifecycle doubles for the supervision engine
  ([lifecycle.rs](../../tool/verification/tests/lifecycle.rs)), not a
  kitty-protocol oracle. An independent implementor of the RC protocol
  cannot use them as conformance fixtures. Real conformance runs against
  real kitty releases inside headless Weston — the part that requires this
  workspace's examples.
- **The decrypt oracle is already publishing.** The externally valuable
  interop piece named in the workspace table — "a decrypt oracle for
  conformance fixtures" — lives in `access/crypto`
  ([README.md](../../README.md) workspace table), i.e. in `kitty-rc-crypto`,
  which is part of the ordinary publish set. Publishing `kitty-rc-verify`
  grants outsiders no additional access to it.
- **What remains genuinely reusable** is the release-acquisition module
  (pinned download, digest verification, parent-escape-tested extraction,
  provenance JSON — including a hand-crafted hostile-archive test) and the
  pinned-source-oracle *methodology*. Both are real, but they are ~one
  module each inside a crate whose main surface is orchestration for this
  repo. The ecosystem precedent (mock-server dev-fixture crates such as
  wiremock/mockito/httpmock; cargo-wrapping harness libs like escargot)
  supports publishing *protocol-faithful or generic* fixtures — it does not
  rescue a fixture set that does not speak the protocol.

## Naming and support expectations

- Publishing a crate whose description, README title
  ("kitty-rc-verify — private Rust verification tooling"), lib docs and CLI
  about-text all say "private" is incoherent messaging; the tentative flag
  was reading the crate's own self-description correctly.
- A crates.io page carries implied obligations this repo explicitly
  declines: the root README states there is no toolchain pin and no
  maintained Rust minimum ([README.md](../../README.md) "Building and
  testing"), and the support policy is deliberately provisional
  ([support.md](../../doc/usage/support.md)). The redesign facade is
  explicitly unstable.
- The pinned release table (0.43.1, 0.49.2, read 2026-10-14) will age —
  upstream HEAD was already 0.50dev at documentation time
  ([evolution0](../upstream-kitty/evolution0.gpt6a.md)). In-repo, that is an
  ordinary maintenance edit; published, every pin refresh requires a release,
  and strangers will read the pins as a support statement the policy
  explicitly refuses to make.
- Build burden without audience: `xz2 0.1` links the system `liblzma`
  (no `static` feature), so `cargo install` fails on machines without the
  dev package; `ureq 2` is an old major (3.x exists; still rustls-based, so
  no OpenSSL); and an install drops **six** binaries into
  `~/.cargo/bin`, four of them test doubles. `default-run` only fixes
  `cargo run` selection inside the repo.

## Options considered

1. **Publish as-is** — rejected: self-described-private artifact, 5/7
   subcommands broken outside the checkout, fixture binaries pollute
   installs.
2. **Publish with changes** (rewrite description, add `readme` field and a
   scope section, feature-gate fixture binaries, revisit `/tmp/opencode`
   default and ureq/xz2) — coherent but expensive relative to the audience it
   would serve: repo contributors run it from source anyway (`cargo run -p
   kitty-rc-verify` per the README), and external protocol implementors are
   served by `kitty-rc-crypto`'s oracle plus the published client crates,
   not by this orchestrator. Saves only the single `download` subcommand for
   strangers, at the price of permanent public-tool expectations.
3. **Keep unpublished; git deps serve the rare external user** — chosen.
   Anyone who truly wants the orchestrator can depend on the GitHub
   repository (path/git dependency); that audience is currently hypothetical,
   and a git dep carries exactly the right signal: workspace-internal tool.

## Recommendation and flip conditions

**Keep `kitty-rc-verify` unpublished** for this publish wave:

1. The CLI's value is workspace-bound by construction — five of seven
   subcommands invoke cargo against this repo's packages or read its catalog
   source, so a crates.io install is a mostly-broken tool for anyone else
   ([examples.rs](../../tool/verification/src/examples.rs),
   [source.rs](../../tool/verification/src/source.rs)).
2. The audience behind "we may want to expose it" is not actually served:
   the fakes do not speak the RC protocol, and the decrypt oracle that has
   genuine interop value already ships in `kitty-rc-crypto`
   ([README.md](../../README.md) workspace table).
3. Public-tool expectations (MSRV, semver, issue tracker, aging pinned
   releases) conflict with the repo's stated no-MSRV, provisional-evidence
   support posture ([support.md](../../doc/usage/support.md)).
4. The exposure audit found *no* secret/legal blocker — so if any flip
   condition below fires, nothing in the content needs scrubbing first.

**The answer flips if any of these become true:**

- A **protocol-speaking conformance double** emerges — a fake kitty that
  accepts DCS-framed commands, performs the X25519→AES-256-GCM handshake,
  and answers catalogued replies. Then extract the fixtures as their own
  small crate (e.g. `kitty-rc-mock`) with public-facing docs; *that* is a
  publish-worthy conformance crate, and this assessment does not cover it.
- A real external consumer appears and demonstrates git-dependency friction
  for the orchestrator itself — then publish **with changes** per option 2.
- The acquisition/oracle modules are wanted as a library — then extract a
  focused crate (which could shed `tokio` and `clap` entirely) rather than
  exposing this one.

## Manifest specifics

For the recommended path, the only change is adding to
[tool/verification/Cargo.toml](../../tool/verification/Cargo.toml) `[package]`:

```toml
publish = false
```

(The crate currently has **no** `publish` field, so without this line a
workspace-wide publish would include it.) For completeness, if a
publish-with-changes path is ever selected, the changes are: rewrite
`description` (drop "Private", state "verification orchestration for
kitty-rc workspace contributors"), add `readme = "README.md"`, optionally
`keywords`/`categories = ["development-tools::testing"]`, gate or accept the
four fixture binaries, and revisit the `/tmp/opencode` default and the
ureq-2/xz2-0.1 pins. No path-dep blocker exists in either path — the
dependency table is entirely registry crates today.

## Cross-references

- [Verification guide](../../tool/verification/README.md): the entrances this
  assessment traced.
- [Rust verification tooling conversion](../library-redesign/verification/rust-tooling0.glm53m.md):
  why the crate is shaped as private orchestration.
- [Support policy](../../doc/usage/support.md) ⇄ publish expectations: the
  provisional evidence tiers a public tool would be read against.
- [Upstream evolution](../upstream-kitty/evolution0.gpt6a.md): the drift that
  will age the pinned release table.
