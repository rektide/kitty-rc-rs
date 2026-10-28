---
type: WorkProposal
title: Send-text after the finite build — epic and ticket discussion
description: Completed finite data and broadcast work, proposed follow-on scope and dependencies, and choices for the human before native tracker items are created.
resource: /design/library-redesign/capabilities/sendtext-followthrough0.gpt6a.md
status: proposed
generated:
  by: "model:openai/gpt-6-astra#xhigh"
  at: 2026-10-28
sources:
  - resource: "opencode://ses_e7b98a17dffeeKi0YEszIrjBvx"
    title: Human directs building before ticketing and discussion before creation; Astra reviews both Sol xhigh slices and proposes bounded follow-through
  - resource: /design/library-redesign/capabilities/sendtext0.glm53m.md
    title: Original A/B/C decomposition and proposed epic name
  - resource: /design/library-redesign/capabilities/sendtext-core0.sol61xh.md
    title: Completed finite text/raw-byte core and migration evidence
  - resource: /design/library-redesign/capabilities/sendtext-broadcast0.sol61xh.md
    title: Completed finite broadcast messages, stateful behavior and converged gates
  - resource: /design/library-redesign/capabilities/sendtext-boundaries0.gpt6a.md
    title: Native escape and nested-action findings that change the proposed order
  - resource: /client/README.md
    title: Integrated public usage, migration and lifecycle boundaries
  - resource: /doc/usage/support.md
    title: Current exact-profile source/live evidence boundary
  - resource: /Cargo.toml
    title: Updated command-dependent workspace version requirements
---

# Send-text after the finite build

**For discussion, not created tracker items.** The human requested implementation
first, then discussion of epics/tickets before creating any. The two selected
finite slices are complete and parent-reviewed. B and C remain the forkable
directions already identified by the human.

## What is built

| Result | Evidence and public entrance |
| --- | --- |
| Literal text / arbitrary bytes | `SendTextData::{Text, Bytes}`, `SendText::new/bytes`, `.data(...)` builders; [core report](sendtext-core0.sol61xh.md). |
| Explicit broadcast messages | Nonempty `BroadcastId`, optional `SendText.broadcast`, required-ID `BroadcastControl::start/end`; [broadcast report](sendtext-broadcast0.sol61xh.md). |
| Native-contract verification | Eleven finite handler cases and three fresh-state broadcast scenarios at each named stable/HEAD revision; real upstream code with documented injected boundaries. |
| Integrated checks | 248 workspace tests, eight doctests, strict all-target Clippy and formatting passed. Parent reviewed the code and evidence rather than rerunning those gates. |
| Public guidance and source version markers | [Usage/migration](/client/README.md#finite-text-and-byte-input); commands 0.3, facade 0.8, exchange/tokio/registry/check/CLI 0.2 and listing 0.3. The normal dependency closure resolves a single commands crate. These are source revision markers; no new registry publication occurred. |

The public migration changes `text` / `.text(...)` to `data` / `.data(...)`;
full struct literals also need `broadcast` or update defaults. Literal
`SendText::new` behavior stays intact. The command-dependent package versions
move together because their public command/operation types must share identity.

The [boundary review](sendtext-boundaries0.gpt6a.md) is another concrete result:
native probes disproved the earlier Bash-equivalence and paste-constructor
claims. Preserving that evidence avoids building a misleading convenience API.

## Proposed epic

Retain the earlier candidate name
`kitty-rc-proto-rs-redesign-capabilities-sendtext`: **Deliberate text, byte and
broadcast input over kitty remote control.**

Purpose: let callers express input faithfully, bound its execution, and
understand what an observed result establishes. Finite values are the foundation;
source consumption, ordered local writes and CLI policy have separate owners.

Recommended tracking shape: one epic with the built work summarized in its
description/notes, linked to the two implementation reports. If retrospective
closed children are useful, split them into `-finite` and `-broadcast`; their
completion/evidence is already recorded, so these are a tracking preference.
Do not reopen the original redesign roadmap simply because a follow-on exists.

## Concrete prospective children

### `-live` — Observe finite input in an isolated real kitty

**Why first:** the new API has strong source/preparation evidence, but new actual
child-input journeys would increase confidence more directly than more type
vocabulary. It can proceed independently of B/C.

Scope:

- Extend existing private fixtures with a bounded raw-mode receiver, so binary
  assertions observe child bytes rather than interpreting `get-text` screen
  rendering as a binary oracle.
- Exercise literal backslashes/Unicode, NUL/non-UTF-8 bytes, explicit targets and
  selected bracketed-paste modes. Record exact server/version/protection profile.
- Keep local `Submitted` distinct from independently observed child receipt.
- A broadcast lifecycle witness may be a later subprofile; source-mode callback
  invocation is not evidence of real socket-origin/event behavior.

Acceptance: exact named journeys pass under isolation, with failure/teardown
evidence and support-policy updates limited to what ran.

### `-conversation` — B: bounded multi-message send-text execution

Scope:

- Explicit bounded source consumption and ordered writes on one owned transport,
  usable through direct and managed entrances; preserve operation ownership,
  deadline, cancellation and retained-progress semantics.
- Fresh per-message protection at the existing protection boundary, appropriate
  input/wire accounting and transport retirement; no automatic replay after
  uncertain writes.
- Decide byte chunk sizing and bracketed-paste boundary semantics from native
  behavior. The reference's 1024-byte generator is evidence, not a universal
  protocol limit.
- Choose whether broadcast start/data/end composition belongs in this initial
  conversation or a subsequent owner. Ordered local writes still do not prove
  remote receipt or successful cleanup.

Acceptance: bounded-source, interruption/deadline and exact wire-sequence
evidence, with explicit outcomes and a native journey for the chosen profile.
No server upload accumulator/exclusion lane is inferred from the word stream.

### `-cli` — C: explicit send-text source acquisition

Scope:

- `kitty-rc send-text` using established explicit endpoint/protection policy,
  window/tab selection, exclusion and bracketed-paste options.
- Finite literal input can use the built core immediately; file/piped-stdin
  streaming should compose with B. TTY input mode/termination is an additional
  policy choice, not implied by supporting a pipe.
- Choose positional-text interpretation deliberately. Literal text requires no
  decoder; reference-compatible escape syntax depends on the grammar decision
  below. Do not quietly switch the Rust API's literal contract.

Acceptance: executable journeys for each selected source and error path,
accurate help, bounded input, and no implicit credential discovery or retries.
Use a dependency on B only for the streaming portion actually selected.

## Optional inquiries, not compulsory implementation tasks

| Candidate | Decision/result worth commissioning |
| --- | --- |
| Escape convenience | Which caller needs it, and whether to expose a strict Unicode grammar or version-qualified kitty compatibility, including malformed numeric input and non-scalar policy. Native end-of-input quirks make this a real choice. |
| Literal nested paste / `send_text` | A full-action encoder and effect contract: preserve backslashes, whitespace and empty-input meaning; distinguish paste configuration effects from keyboard-mode-gated sending. A general local decoder is not a prerequisite. |
| `kitty-key:` | A separate audit of event encoding and per-window keyboard-mode behavior, after text/byte use establishes demand. |

The human previously identified a possible Astra assessment for nested actions.
The [current review](sendtext-boundaries0.gpt6a.md#why-nested-send_text-remains-its-own-decision)
supplies starting evidence; deciding to expand that assessment need not block B/C.

## Questions to settle before creating tickets

1. Does one send-text epic with completed work summarized in its notes fit, or
   would two retrospective closed children be useful?
2. Should isolated live input evidence be the next selected task, alongside the
   separately forked B/C work?
3. Which optional inquiry deserves an actual work item now? My recommendation is
   to keep escape/nested-action possibilities linked in the epic until a caller
   need or a chosen assessment gives one a concrete outcome.

Search existing tracker state before applying this proposal, especially after
the planned B/C forks. The names above are candidates, not reservations or proof
that no other session has since created the work.
