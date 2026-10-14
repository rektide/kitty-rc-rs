---
type: Index
title: Command-specific upload implementation and evidence
description: Current PNG/run entrances, exact local accumulator coordination, and bounded verification sources.
resource: /design/library-redesign/uploads/README.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5ea232fffeiw0Rb2oUZbPbED"
    title: Upload/run implementation, ownership and verification map
  - resource: /design/library-redesign/uploads/implementation0.sol61xh.md
    title: Actual implemented conversations, resource boundaries and native uncertainty policy
---

# Uploads

- ⇄ [Implementation and decisions](implementation0.sol61xh.md): BackgroundPng,
  LogoPng, buffered async Run, direct/managed entrances, explicit aliases and exact
  interrupted-lane policy. Includes verification and supported-version limits.
- Checkpointed pinned-source oracle: the retired `verify-source0.sol61xh.py`
  verified actual stable constructors without native imports or terminal
  mutation; `kitty-rc-verify source` (tool/verification) retained the same
  coverage at the same pinned revision.
- Private live runner: superseded by `kitty-rc-verify uploads`
  (explicit `KITTY_RC_TEST_KITTY` override, private
  Weston/kitty/config/socket and the PNG/run Rust example are now
  orchestrated by the Rust crate in tool/verification).

The human directed Rust verification orchestration after the successful core gates.
G owns that migration; [the handoff](implementation0.sol61xh.md#human-directed-rust-verification-migration)
records exact inputs, assertions and replaceable helpers. No further Python
verification expansion is selected here.

Work: `kitty-rc-proto-rs-redesign-capabilities-upload` and the remaining run
acceptance of `kitty-rc-proto-rs-redesign-capabilities-typed`.
