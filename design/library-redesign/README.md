---
type: DesignIndex
title: Kitty remote-control library redesign
description: Human-directed clean-slate API and crate design, with independent lifecycle and reuse research before synthesis.
resource: /design/library-redesign/README.md
status: draft
generated:
  by: "model:openai/gpt-6-astra#medium"
  at: 2026-10-13
sources:
  - resource: "opencode://ses_ec7cd6e45ffeqAP7k1wV3STT72"
    title: Human redesign direction, research sequence, and initial architecture draft
  - resource: /design/library-redesign/draft0.gpt6a.md
    title: Independent parent proposal before research integration
  - resource: /design/upstream-kitty/evolution0.gpt6a.md
    title: Completed upstream study, baseline and summary inspected for navigation
---

# Library redesign

**Find the best API and architecture. All current interfaces are unstable, and
no consumer imposes a compatibility obligation.** The human favors a substantial
restructure with independently composable concern crates, explicit lifecycle,
and separated discovery/authentication steps.

## Initial position

- [Initial architecture draft](draft0.gpt6a.md): typed commands/responses, a
  concern-crate workspace, explicit setup and exchange lifecycles, cancellation
  and outcome semantics, command-specific streaming, and verification strategy.
- [Earlier registry plan](../../doc/PLAN-registry.md): historical motivation;
  pooling and retries need reconsideration under the new lifecycle model.
- [Earlier public-key store plan](../../doc/PLAN-pubkey-db.md): historical
  standalone authentication context and a candidate integration to redesign.

## Commissioned research

These are commissions, not findings. Completed documents will be linked after
they are read and integrated into this map.

| Inquiry | Agent | Role |
| --- | --- | --- |
| Upstream kitty evolution | Astra medium | Establish build baseline and compare latest stable/HEAD behavior and schemas |
| Kubeless reconnaissance | FX | Map concrete lifecycle/streaming reuse opportunities and uncertainties |
| Kubeless research and design | Astra medium, after FX | Verify the reconnaissance and develop integration choices |
| Tmax lifecycle comparison | G | Reconstruct ownership and lifecycle, identify useful mechanisms and excessive complexity |

### Available upstream evidence

- [Kitty evolution from the build baseline](../upstream-kitty/evolution0.gpt6a.md):
  February 2026 library release, declared kitty 0.43.1 target, latest stable 0.49.2,
  and pinned development HEAD. Its opening assessment distinguishes a stable
  core protocol from command growth, server fixes, and local pre-existing bugs.
  This is evidence for the eventual synthesis; the parent draft has not been
  rewritten around its recommendations.

The parent draft is independent of this wave. After the research is available,
discuss tradeoffs with the human **before writing the next architecture draft**.
No library implementation is commissioned by this research sequence.
