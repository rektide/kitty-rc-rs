---
type: VerificationGuide
title: Private profile-gated kitty matrix
description: Opt-in release acquisition, isolated compositor/server composition and exact per-profile evidence.
resource: /scripts/conformance/README.md
status: checkpoint
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5ccabb4ffeiXGEem6BYMTT3a"
    title: Independent release provenance, matrix harness, finite and nested live journeys
  - resource: /doc/usage/support.md
    title: Provisional profile-level support policy, not an oldest-supported guarantee
  - resource: /scripts/verify-isolated-uploads.py
    title: Reused private Weston, kitty, PNG and process teardown lifecycle
  - resource: /control/registry/examples/upload_conformance.rs
    title: Separate upload owner's live PNG and buffered run journeys
---

# Opt-in isolated verification

Work anchor: `kitty-rc-proto-rs-redesign-verification-live`.

**Checkpoint, not the selected maintained language.** Human steering now selects
Rust verification tooling. These prototype files are preserved for the
parent-assigned G conversion; do not extend Python orchestration. The exact
completed preparation evidence and Rust handoff are in the
[live report](../../design/library-redesign/verification/live0.sol61xh.md#rust-conversion-handoff).
The tiny upstream Python no-ui kitten is a native fixture boundary, not a
reason to retain Python process orchestration or embedded Python test children.

Prerequisites: the user's selected `mise` Rust, Python 3.12+, headless Weston
(`--backend=headless --renderer=pixman --fake-seat`), and x86-64 Linux kitty.
Nothing is installed into user/system locations or added to user kitty config.

```sh
# Pure preparation/ownership checks, no kitty contacted:
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/conformance -p 'test_*.py'

# Explicit network opt-in. Pinned binaries stay under ignored .test-agent/:
python3 scripts/verify-supported-kitty.py --download --cli-smoke

# Reuse acquired binaries; builds selected examples with mise, then tests
# installed kitty and acquired 0.43.1/0.49.2, encrypted and explicit plain:
python3 scripts/verify-supported-kitty.py --cli-smoke

# Exact single executable/mode, after an intentional converged build:
python3 scripts/verify-supported-kitty.py --no-build --kitty /absolute/path/to/kitty --mode encrypted
```

`--kitty` and `--mode` can repeat. `--finite-only` deliberately omits the existing
core and upload examples: useful preparation, **not full-ticket verification**.
`--report PATH` selects JSON output; default is
`.test-agent/supported-kitty/matrix.json`. Missing executables or failed fixture
startup are blockers, not passes. A case failure makes the run fail; gated
profiles record skips. Successful stderr/stdout results never print the selected
test password. The fixed local test password is not a production credential.

## Lifecycle and ownership

The matrix composes `scripts/verify-isolated-uploads.py` through private PATH
adapters. That existing runner owns Weston, kitty, socket, config, child, key
capture, valid PNG and teardown. The adapter selects an **explicit executable**,
augments only a recognized private config, and supplies private XDG config/cache/
data directories. Ambient kitty endpoints, keys, passwords, display and desktop
bus are removed. No human window is discovered or targeted. On bounded timeout
or Ctrl-C, only the process group created for this fixture is interrupted.

`fixture-cargo.py` accepts only the known runner invocation and executes prebuilt
examples, avoiding repeated compiler work while the server is alive. It runs
`conformance`, `supported_matrix`, and `upload_conformance` for encrypted fixtures;
plain fixtures run `supported_matrix`. Thus **plain uploads are not claimed**.
`--cli-smoke` adds standalone list/explicit-tab-active/goto/two finite watch
samples with the advertised 16 MiB response budget. The no-ui kitten is our
independent private fixture, not copied upstream code or a user installation.

## Evidence and gates

JSON records exact reported versions, executable/archive SHA-256, published
release source revisions, selected Rust, example and runner hashes, mode,
profile/options, measured progress/reuse and pass/skip/blocker/failure evidence.
Release digests are pinned from GitHub's release API; this does **not** claim
independent signature verification. The installed binary's source revision is
unknown unless separately supplied—never inferred from a version number.

The envelope remains `[0,43,1]`. Alternate text is selected only from 0.48;
screenshot and OS-window title only from 0.49, following documented profile
metadata. Unknown earliest availability stays unknown; successful selected
journeys on 0.43.1 do not establish whole-catalog support or a minimum guarantee.
Snapshots/title observations and binary run results are checked; action/kitten
acknowledgments alone do not imply downstream UI completion. Screenshot checks
PNG signature and actual private saved-file bytes, not visual pixel fidelity.

See [live evidence](../../design/library-redesign/verification/live0.sol61xh.md)
for the exact completed matrix and limitations, and
[support policy](../../doc/usage/support.md) for the maintained support boundary.
