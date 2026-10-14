---
type: Index
title: Upstream kitty compatibility research
description: "Revision-pinned research connecting kitty evolution to the Rust client's compatibility and design."
resource: /design/upstream-kitty/README.md
status: draft
author: "model:openai/gpt-6-astra#medium"
date: 2026-10-14
sources:
  - resource: "opencode://ses_ec7b73050ffeSoRWSwN2E8vc3X"
    title: "Established baseline/stable/HEAD comparison and its navigation entry"
---

# Upstream kitty

## Compatibility and evolution

- [Kitty since we built this library](evolution0.gpt6a.md): construction history,
  declared 0.43.1 baseline, latest stable 0.49.2, and pinned development HEAD;
  protocol continuity, server fixes, new commands, ls fields, existing local
  defects, and a recommended modernization sequence. Source review, not live
  interoperability certification.

## Related project context

- [Library README](../../README.md): public API and setup.
- [Historical response inventory](../../doc/info.md): local data-model starting point.
- [Historical registry proposal](../../doc/PLAN-registry.md): earlier pooling intent;
  the comparison links the actual downstream implementation.
- [Historical encryption bug](../../doc/BUG-encrypted.md): provenance for an
  already-repaired wrapping defect, distinct from current encrypted-send issues.
