---
type: ResearchComparison
title: tmax lifecycle reconstruction and what kitty-rc should and should not take from it
description: Grounded reconstruction of how tmax connects, executes, correlates, bounds, cancels, and shuts down; comparison against the kitty-rc redesign direction with concrete costs, minimal useful patterns, and triggers for heavier machinery.
resource: /design/library-redesign/tmax-lifecycle0.glm53m.md
tags: [kitty-rc, tmax, lifecycle, redesign, comparison]
status: draft
generated:
  at: 2026-10-14
  by: model:zai-coding-plan/glm-5.3#max
updated:
  at: 2026-10-14
  by: model:zai-coding-plan/glm-5.3#max
  reason: parent review corrections — async response correlation, ownership model, purity boundaries, schema tolerance, connection-vs-instance identity, no auto-reconciliation, preflight limits, transport invalidation on interrupted exchanges, HEAD-vs-v0.43.1 citation precision (see Correction addendum)
sources:
  - resource: opencode://ses_ec7b1e11effeOrmZ7s7gR2AXJY
    title: Commissioned tmax lifecycle research and independent comparison wave (this document)
  - repository: ~/src/tmax @ jj change mlruqyvwwzqp (commit ba681a561d71)
    title: tmax checkout read statically; line references are to this checkout, 2026-10-14
  - resource: ~/src/tmax/README.md
    title: tmax entrance, contracts summary, feature table
  - resource: ~/src/tmax/doc/library/README.md
    title: Library guide — evidence, failure/resource semantics, deadlines
  - resource: ~/src/tmax/doc/library/execution-modes.md
    title: Direct vs persistent execution capability matrix
  - resource: ~/src/tmax/doc/library/ordering.md
    title: Cooperative correlation contract and its assumptions
  - resource: ~/src/tmax/src/endpoint/mod.rs
    title: Endpoint config, socket resolution, one-shot execution
  - resource: ~/src/tmax/src/protocol/native/command.rs
    title: One-shot native command conversation and error taxonomy
  - resource: ~/src/tmax/src/session/native.rs
    title: Persistent session construction phases and supervisor
  - resource: ~/src/tmax/src/session/mod.rs
    title: Session surface, admission, close/shutdown
  - resource: ~/src/tmax/src/session/request.rs
    title: Request/reply-stream consumption, completion receipts, deadlines
  - resource: ~/src/tmax/src/session/{lifecycle,limits,evidence,result}.rs
    title: Session outcome, budgets, positions, failure result types
  - resource: ~/src/tmax/src/operation/query.rs
    title: TypedOperation/ReplyDecoder/Prepared vocabulary
  - resource: ~/src/tmax/src/control_mode/{transport/sync.rs,transport/sync.rs limits,delivery/cell.rs,attach/gate.rs}
    title: Thread architecture, bounded delivery cell, startup gate
  - resource: ~/src/tmax/design/sync-deadlines/implementation0.sol61.md
    title: Deadline map and verification burden record
  - resource: ~/src/tmax/design/saturated-budget/implementation0.sol61.md
    title: Honest saturation failure record (23/192 timeouts at 16-way)
  - resource: ~/src/tmax/design/cleanup-ownership/decision0.sol61.md
    title: Cleanup ownership decision proposal and failure table
  - resource: ~/src/tmax/design/annealing/README.md
    title: Annealing index — cost of convergence in design documents
  - repository: kitty-rc src @ git 97990cb2 (jj rvzszymznvwz; later commits touch design/ only)
    title: kitty-rc client/protocol/error sources re-read for the comparison
  - repository: ~/archive/kovidgoyal/kitty @ git 5bf3c2dc6 (nightly-131-g5bf3c2dc6)
    title: inspected kitty HEAD remote_control.py — response construction and async handling, read for the correlation correction
  - repository: ~/archive/kovidgoyal/kitty @ tag v0.43.1 (e128af470f7c)
    title: kitty v0.43.1 remote_control.py — same response-correlation facts verified at the release tag
---

# tmax lifecycle reconstruction and what kitty-rc should and should not take from it

## Situation, ask, and method

The parent is redesigning kitty-rc 0.5.0 (Tokio `UnixStream`, bon/Serde command
structs, untyped `KittyMessage` hop, manually chosen parsers) toward a typed
Command→Response API with one authoritative serialization/encryption/write/receive
pipeline, an incremental frame decoder, bounded buffers, explicit deadlines and
unknown-outcome semantics, one exchange at a time initially, and crates per
concern. I was commissioned to reconstruct how `~/src/tmax` works and its
lifecycle, and to extract insight without reflexively adopting or dismissing it.
The user explicitly warned tmax may be overengineered.

Method: static reading of tmax source, docs, tests structure, and design records
(revisions in frontmatter; tmax's colocated git HEAD is beads-churn, so the jj
change id is authoritative). I did not run tmax's suites and claim no fresh
verification of tmax code. Concurrent same-wave documents in this directory and
the upstream-kitty review were deliberately not read. kitty-rc findings below
were re-verified against `src/` at git `97990cb2`; the parent has since
confirmed `client.rs:201` now propagates the inner write result (`??`), so the
earlier "ignores inner write_all result" review item is historical motivation
only, not a live defect. Items I re-verified as still live at `97990cb2`:
`send_all` double encryption (`client.rs:240-252` encrypts, then `send`
(`client.rs:195-204`) encrypts again); `send_command` neither reads a reply nor
forces `no_response` (`client.rs:262-267`); `receive` scans with
`buffer.ends_with(SUFFIX)`, assumes one frame, resets the timeout per read, and
drops partial bytes on timeout (`client.rs:206-233`); `protocol.rs:98-165`
chunks any long payload string into `data`, dropping sibling payload keys,
corrupting UTF-8 via `from_utf8_lossy` on byte chunks, and streaming without
any framing terminator guarantee; `execute` returns `Ok` for `ok:false`
responses (`client.rs:235-238`). The 159-tests/Clippy figures from the earlier
review pertain to an earlier snapshot and are not cited as current.

## tmax at a glance

tmax is a fork of `tmux-interface-rs` reworked into a native tmux control
client ("no subprocess"). Scale, read 2026-10-14:

| Area | Lines | Notes |
|---|---|---|
| `src/commands|options|formats|variables|copy_mode|styles|target` | ~88k | inherited upstream builders, largely untouched |
| `src/control_mode` | ~29k | the persistent engine: driver, transport threads, protocol/correlation, subscriptions, delivery |
| `src/session` | ~8k | public persistent surface (builder, requests, receipts, subscriptions, limits, lifecycle) |
| `src/operation` + `src/query` | ~6.6k | transport-independent typed-operation vocabulary, 17 declared operations |
| `src/protocol` (incl. `native/`) | ~4.8k | one-shot imsg client, context, diagnostics |
| `src/endpoint` | 0.6k | endpoint config + direct one-shot execution |
| `src/error`, `src/server` | ~2.7k | taxonomy; optional explicit server ownership |
| design/ + doc/ records | 150+ files | decision proposals, implementation reports, acceptance gates |

So roughly **50k lines of new engine code serve 17 typed operations** — about
10× the entire current kitty-rc codebase (5.4k). That ratio is the first cost
datum, and most of it buys capabilities kitty's remote-control protocol does
not need (below).

## Reconstructed lifecycle

### Endpoint: discovery and configuration without a connection

`Tmux` (`src/endpoint/mod.rs:49-109`) is immutable-by-convention configuration:
socket selection (`-S` path / `-L` name), environment overrides, a network
timeout, and an output bound. It holds no live connection. Socket resolution
mirrors tmux's own client (`resolved_socket_path`, `mod.rs:172-210`): `-S` as
given; else `$TMUX` comma-truncated; else `<TMUX_TMPDIR|/tmp>/tmux-<uid>/<name>`
with the endpoint's configured environment honored over the ambient one.
`validate_socket` (`mod.rs:231-264`) rejects sockaddr-length overflows **before
any spawn**, converting a late cryptic failure into immediate typed evidence.
Client context (merged raw-byte environment, cwd, `TERM`) is frozen once at
connect (`client_context`, `mod.rs:297-324`).

Direct and persistent execution share this vocabulary deliberately:
"**One operation vocabulary does not make the execution modes interchangeable**"
(`doc/library/execution-modes.md` header) — distinct lifecycles, not one
optional-field outcome.

### Direct (one-shot) execution — the path analogous to kitty-rc's use case

`Tmux::run`/`query` (`mod.rs:114-151`) lowers one command through one bounded
native connection (`src/protocol/native/command.rs`):

1. **Validate everything before the first byte** — socket path length, argv
   encoding, context, bounds (`command.rs` module doc; `endpoint/mod.rs:266-294`).
2. One absolute deadline covers **connect → identify → command → mediation →
   exit acknowledgement** (30s default, `Tmux::default`). No per-read timeout
   resets.
3. Exactly one `MSG_COMMAND` conversation; bounded stdout/stderr captured in
   memory; client-mediated file requests declined `EBADF`; stdin uploads
   declined. The connection is deliberately never reused (audited tmux marks
   unattached command clients `CLIENT_EXIT`).
4. Typed error taxonomy (`command.rs:79-141`): `ServerConnection{socket_path}`
   (connect-stage, retains the path) is distinct from later `Io` loss, which is
   "never rewritten as a missing server"; `DeadlineExceeded`; `NotTmuxPeer`;
   `VersionReply`; `OutputOverflow`; a nonzero native return code is
   `Rejected` with a `TmuxDiagnostic` — **never a successful outcome**.

`Tmux::query<Q: TypedOperation>` executes the preparation and runs its decoder
strictly: "malformed expected rows fail the entire query, never return partial
data" (`mod.rs:124-151`).

### Persistent session construction

`SessionBuilder::connect` (`src/session/native.rs:701-842`; contract in
`src/session/mod.rs:134-162`):

- **Phase 0, pre-byte validation:** limits, startup timeout, attach argv,
  socket resolution, client context — every pure check runs before contacting
  the server.
- **Build proof:** default is one bounded `display-message -p '#{version}'`
  mapped through a source-audited build table; `attested_version("3.6")`
  (fallible) skips the query. Unaudited/malformed/shutting-down peers get
  honest typed errors; **no implicit server start, no child fallback**.
- **Committed identify transaction:** after the first control-identify byte,
  the handshake completes forward — recoverable pressure is retried through
  DONE "to avoid a known audited-server crash" if the client drops mid-S2.
  Consequence stated plainly in the API docs: *a permanently wedged peer can
  block construction indefinitely in that narrow window*.
- **Engine startup** (10s default): an attach-gate state machine
  (`control_mode/attach/gate.rs:17-23`) — Startup → MarkerCompatibility →
  CapabilityProbe → OutputBaseline. Failure returns
  `NativeConstructionFailure::EngineStartup` preserving the typed gate phase,
  disposition, retained transport fault, preserved timeout budget, and the
  **actual abandon-close receipt** — "never a claim that tmux admitted or
  acknowledged the already-delivered attach command"
  (`src/session/native.rs:359-471`).
- A single supervisor thread owns the connection for the session's lifetime and
  later runs the `EXITING → EXITED → close` conversation
  (`native.rs:573-576`, `CHANNEL_B_POLL` 250ms, close budget 5s).

### Running engine: threads, admission, bounds

`SyncTransport` (`control_mode/transport/sync.rs:34-96`) spawns reader and
writer threads with **structural interrupts** — `shutdown(2)` on owned socket
ends releases blocked workers; there is no flag-polling or kill race. Two
bounded ingress lanes (reader events 256, API admission 64) with per-turn work
batches (64/32) prevent either side from starving the other. Fixed input
framing limits: 8 MiB per line, 32 MiB tracked retained chunks.

Every complete line gets a checked `ProtocolPosition` (connection-scoped
ordinal) before parsing and fanout (`src/session/evidence.rs:42-75`) — framing,
body, marker, unselected, and malformed lines all consume positions, so
"visible gaps do not prove loss."

Admission (`Session::stream/submit_group`, `mod.rs:242-313`) **blocks until the
driver acknowledges**; capacity exhaustion surfaces as typed
`Full`/`ByteFull` refusals, never unbounded queueing. Typed preparations
(`PreparationPolicy::LibraryDeclared`) are checked against the connection's
observed `Capabilities` *before* reserving ingress.

### Request lifecycle: submit → consume → complete

- `submit(q)` returns a `Request` owner; `query(q)` is exactly
  `submit(q)?.wait()` (`mod.rs:225-230`).
- The caller-side decoder runs on **the consumer's thread** — `ReplyDecoder`
  has no `Send`/`'static` bound; "executors must run user decoding in the
  consuming caller's context, not on their protocol driver"
  (`src/operation/mod.rs:1-6`). The driver only routes crate-owned bytes.
- The consumption deadline is fixed at submission (`now + limits.timeout`,
  30s default); expiry is **checked when consuming, not by a background timer**
  (`src/session/request.rs:687-694`). `wait_until` can only *cap* it, never
  extend it (`request.rs:833-836`).
- Success is `Completed<T>` — decoded value plus `Completion`
  (`status`/`proof`/`group` evidence). `CompletionProof` separates
  `WireEnd`/`WireError` (tmux actually emitted a boundary) from
  `NoWireBoundary`; even `Success` "does not prove that a caller-side decoder
  succeeded, and cancellation/loss never proves that a mutation did not
  execute" (`request.rs:27-56`).
- Failure preserves stage: `SubmissionFailure` (nothing admitted, **no
  fabricated receipt**) vs `RequestFailure` (admitted; carries a lightweight
  `CompletionReceipt`) vs `AttemptFailure` wrapping both
  (`src/session/result.rs:28-157`). A pending receipt may complete later; an
  early-loss receipt is immutable and never upgraded.
- Receipts are cloneable, non-cancelling observers that do not keep the
  request, mailbox, or session alive (`request.rs:163-202`).
- Cancellation (`cancel`, drop) is nonblocking and **abandons delivery, not
  execution** (`#[must_use]` owners; `doc/library/README.md` "Failure and
  resource semantics").

### Correlation: why the biggest subsystem exists

tmux control mode multiplexes many commands, notifications, and output over
one line-oriented stream **with no request IDs**. tmax's answer is cooperative
framing (`doc/library/ordering.md`): the application line and a synthetic
fence line `tic-correlation-fence-<nonce>` are written under one bounded
reservation; the server's parse-error block for the fence becomes a private
delimiter; the group owns correlation until the fence or connection end. The
contract is explicit that this is cooperation, not authentication — a complete
imitation of the marker is indistinguishable; there is no atomicity or
`PIPE_BUF` guarantee. Around this sits the ledger, block evidence, group
progress, outside-line retention, and cardinality checks
(`CompletionStatus::ReplyCardinality` for zero/multiple candidate blocks).

**kitty-rc does not need any of this machinery, but not because kitty "has
IDs."** The kitty RC protocol has **no response correlation at all**: in the
inspected kitty HEAD (nightly-131-g5bf3c2dc6), sync responses are
`{'ok', 'data'|'error'}` dicts with no request identifier
(`~/archive/kovidgoyal/kitty/kitty/remote_control.py:260-264`), and async
completions sent by `send_response_to_client` likewise do not echo the `async`
id — the id is only a server-side key into `active_async_requests` used for
dedupe and cancellation gating (`remote_control.py:472-484`, registration at
`227-247`; note the wire field is `async`, `remote_control.py:468`). The same
facts hold at release tag **v0.43.1** (`send_response_to_client` at
`remote_control.py:460-471`, sync shape at `:251-253`, registration `:218-235`,
wire field `:218,456`). A client receiving a late frame cannot attribute it to
a particular outstanding request. Therefore **one outstanding exchange at a
time is not merely a safe first shape — it is the only sound default** given
the protocol; correlating multiple concurrent requests would require an
upstream protocol change (echoing a request id in responses), not a
client-side pending map.

### Shutdown and cleanup

- `close()` stops admission and structurally interrupts the transport —
  nonblocking, does not join threads, does not kill the server
  (`mod.rs:357-362`).
- `shutdown(timeout)` waits for a retained `SessionOutcome`: first close cause,
  typed cleanup evidence (`NativeClose::ExitAcknowledged` vs `PeerLost` vs
  `DeadlineExceeded`), and the honest statement that local `SHUT_RD`
  interruption means **no peer-EOF or complete-drain claim**
  (`src/session/lifecycle.rs:38-56`). If the wait times out, the reaper
  continues and "retrying is safe."
- Last-handle drop initiates close without joining; requests and receipts do
  not keep the session alive (`mod.rs:196-208`).
- Post-timeout cleanup policy is codified after real investigation: "a
  cleanup `kill-pane` timeout with no native completion leaves the mutation
  outcome unknown; it does not authorize blind resubmission"
  (`doc/library/README.md`, "Failure and resource semantics").

### Error taxonomy

`OperationError` carries domain evidence (`TmuxDiagnostic` with command, body,
stderr, return code, truncation flag — `src/error/operation.rs:11-39`).
Everything else projects to one coarse `TerminalCause`
(`error/operation.rs:70-107`): Completed / Rejected / NotAdmitted / LocalClose
/ PeerExit / ConnectionLost / ProtocolFailure / DriverFailure / Timeout /
InvalidInput / Cancelled / Loss{Capacity|Malformed|SequenceExhausted} /
Unavailable. The doc comment repeats the core honesty rule: "timeout,
cancellation, loss, and connection failure do not prove a mutation did not
execute."

## What each mechanism bought, and what it cost

tmax's complexity is not arbitrary — nearly every subsystem traces to a hard
tmux fact. The exercise for kitty-rc is matching mechanism to protocol fact:

| tmax mechanism | Forced by | kitty-rc analog needed? | Verdict |
|---|---|---|---|
| Typed ops: `TypedOperation{Output, Decoder}` + opaque `Prepared<Q>` binding syntax+decoder+requirements+semantics | wanting one vocabulary across two lifecycles | **Yes** — matches accepted typed Command→Response direction exactly | Adopt the shape, skip the sealed semantic-marker zoo initially |
| Validate-before-first-byte (socket length, argv, config) | late cryptic child failures | Yes — cheap | Adopt |
| One absolute deadline per conversation | per-stage timeouts compound badly | Yes — fixes kitty-rc's per-read timeout reset | Adopt |
| Strict decoding: malformed rows fail whole op; zero/multiple candidates are errors | silent partial data is worse than failure | Yes — fixes `ls`-missing-data→empty-success pattern | Adopt |
| Stage-preserving failures (refused vs admitted vs timeout; no fabricated receipts) | honest retry policy | Yes — minimal version: `UnknownOutcome` after send | Adopt (small) |
| "Timeout/cancel ≠ non-execution; no blind retry" | mutation safety | **Yes — this is exactly the zooming-kittens double-increment hazard** | Adopt as documented contract |
| Bounded lines/bodies/mailboxes + typed capacity errors | untrusted/broken peers | Small version: max response bytes per exchange | Adopt (small) |
| `ConnectionId` provenance token, explicitly *not* incarnation proof | cached handles across reconnects | Useful as a **local connection token** for the registry problem; cheap; says nothing about server-instance identity | Adopt concept |
| Engine threads + condvar delivery cells + wakers | sync-first design, later async need | **No** — kitty-rc is Tokio-first; `select`/`timeout`/cancellation are native | Skip |
| Cooperative fences, positions, group ledger, block evidence | tmux has no request IDs and multiplexes | **No machinery** — but note kitty has no response correlation either, which *justifies* one-exchange-at-a-time rather than obviating it | Skip machinery, keep the serial-exchange constraint |
| Subscriptions, notification policy, output routing, format resources | tmux push channels | **No** — kitty RC socket has no push channel | Skip (revisit only if a sidecar adopts one) |
| Audited build table + fail-closed versioning | tmux protocol changes 3.2–3.7c | No — kitty versions degrade gracefully at runtime; keep a floor + runtime error | Skip |
| S2 forward-completion liveness exception | audited tmux crash window | No known kitty analog | Skip |
| Evidence receipts as non-cancelling observers | pipelined consumers wanting late proof | Only if concurrent interleaved consumers appear | Defer |

Costs observed in the record, for calibration:

- **Verification burden:** ~900 tests plus per-version isolated server
  fixtures, compiler pinning, snapshot contracts — with recorded traps (the
  mise PATH shadowing that silently ran nightly instead of 1.96.1 and failed a
  snapshot gate; `design/sync-deadlines/implementation0.sol61.md:95-166`).
- **Saturation honesty, not safety:** under 16-way load, 23/192 cleanups still
  time out; the disposition is documented policy ("no finite allowance is
  saturation-safe"), suites stay serial
  (`design/saturated-budget/implementation0.sol61.md:37-60`). Bounded engines
  did not fix load behavior; they made failure legible.
- **Sync/async straddle:** C1 async admission futures still await a blocking
  sync engine; "Awaiting admission does not make those owners' `wait`…asynchronous" (`doc/library/README.md:133-146`); C2 remains a proposal. A
  Tokio-native design dissolves this entire problem class on day one.
- **Interface learning cost:** 409-line README + 692-line library guide of
  contract prose; six-plus evidence types; `#[non_exhaustive]` everywhere.
  Honest, but heavy for consumers that just want to run commands.
- **Process weight:** 150+ design docs, staged decision proposals, acceptance
  gates. Valuable for a protocol-forensics project; proportional for a
  5k-line client library it is not.

## Semantics mapping for kitty-rc (conservative shape)

| Concern | Recommended kitty-rc semantics | tmax precedent |
|---|---|---|
| Ownership | Exclusive-owner client: one `Exchange` future per command, borrowing `&mut client` across awaits is fine — no `Arc`/split-half prescription. The real constraint is cancellation hygiene, not borrow shape: dropping an exchange mid-receive leaves unknown stream state, and with no response correlation there is no way to resynchronize. An interrupted in-flight exchange therefore **invalidates the transport** — the connection must be discarded and a later exchange requires a fresh one. Merely dropping the user-space buffered input is *not* safe: replies to the interrupted command can still arrive on the socket, cannot be attributed, and no protocol-proven resynchronization exists to justify reuse | `Request` owner; `#[must_use]`; drop cancels delivery |
| Tasks | No resident engine task; each exchange owns its receive until frame complete | (tmax's threads exist only because it is sync; do not copy) |
| Shutdown | Drop the stream; explicit `close()` optional; reconnect yields a fresh **local connection token** (bookkeeping identity for this client's connection) — it is *not* proof of a new server instance; the same kitty may still own that socket | `close()` nonblocking; `Shared::drop` closes (`mod.rs:196-200`); `ConnectionId` is connection identity, explicitly not instance identity |
| Cancellation | Dropping the exchange future cancels the *wait*, never implies the command did not run | cancel "abandons delivery, not tmux execution" |
| Backpressure | One exchange at a time initially → none needed; `max_response_bytes` bound with typed overflow error | `reply_bytes` bound → `ResponseTooLarge` |
| Deadlines | One deadline per exchange spanning send+receive; connect gets its own; no per-read resets | direct 30s covers connect→exit-ack |
| Errors | `Connect{path}`, `Encode`, `Encrypt`, `Send`, `Framing`, `Timeout`, `TooLarge`, `Remote{error}`, `Decode` — plus `UnknownOutcome{after: Send|None}` marker on post-send failures | stage taxonomy; `ServerConnection` retains path; timeout ≠ rollback |
| Retry | Core never retries side-effects; reconnect is a new local connection with unproven server-instance identity; idempotent retry only for connect itself | "does not authorize blind resubmission" |

## Options with triggers

**Conservative (recommended first slice).** Typed `Command` trait with
associated `Response` and per-command payload encoder/decoder; pure frame
codec (prefix/ST suffix, incremental scan, surplus retained for the next
frame, multiple frames per read handled); crypto envelope applied exactly once
in one pipeline; one-shot executor with a single per-exchange deadline; the
error taxonomy above including `UnknownOutcome`; strict parsers (missing data
is an error); command-specific streaming for the few large-payload commands
rather than a generic length heuristic. No resident tasks, no pending maps, no
buffers beyond one frame + one response bound. Est. ~1.5–2.5k lines.

**Middle (add when triggered).**
- *Trigger: async commands (`async` id) actually used by consumers* → still
  one outstanding exchange at a time; responses do not echo the id
  (`remote_control.py:472-484`), so a pending map cannot attribute late frames
  and multiple concurrent async commands are uncorrelatable without an
  upstream protocol extension. A library can only serialize them; raising
  this with upstream (echo the request id in responses) is the real fix.
- *Trigger: a cached-client registry (zooming-kittens)* → fresh local
  connection tokens per connection plus caller-chosen revalidation before
  reuse (see recommendation 6); registry policy stays with the consumer.
- *Trigger: interleaved concurrent exchanges on one socket* → not supportable
  by the current protocol semantics (no response correlation); if upstream
  ever adds it, re-read tmax's receipts/observer split and bounded lanes
  before inventing a correlation layer.

**Richer (only under strong evidence).** Persistent subscriptions/events
(only if kitty grows a push channel over this socket or a sidecar protocol is
adopted); version attestation tables (only if kitty RC starts breaking
compatibility across versions the way tmux does); full evidence objects
(only if multiple consumers need protocol-level proof rather than results).

## Crate boundaries

tmax stayed one crate with feature gates (`client`, `control_mode`,
`server-lifecycle`) plus ~88k legacy lines — its layering is visible in
modules, not crates. For kitty-rc the separation the parent sketched maps
cleanly onto tmax's *internal* seams, with the engine mostly deleted:

| kitty-rc crate/module | tmax precedent | content |
|---|---|---|
| discovery | `endpoint` socket resolution | **pure core**: socket-name parsing, pid extraction from `kitty-<pid>.sock`, selection rules, env-var *names*; **I/O edge** (kept separate): enumerating runtime dirs/sockets, reading environment, liveness checks. The pure selection logic is isolatable and table-testable; the enumeration itself is I/O and should not be laundered into "pure" |
| credentials | — (tmax has none) | same split: pure key/password *parsing and validation*; I/O edge performs pubkey-db queries, env reads, file reads |
| envelope (pure) | — | encrypt/decrypt one payload; no transport |
| wire/codec (pure) | `operation::invocation` + framing constants | frame encode/decode, incremental scanner, typed command/response types |
| transport/execution | `protocol::native/command.rs` discipline | connect, one-exchange executor, deadlines, error mapping |
| client (facade) | `endpoint::Tmux` + `Session` surface | builder, typed `execute`, raw escape hatch |
| cli | — | thin |

The `Prepared<Q>` lesson applies at crate level too: bind a command's wire
form, decoder, and semantics **together** in one opaque value so a caller
cannot mix a request with the wrong response parser — the current kitty-rc
"manually chosen parser after untyped message" flow is precisely the
mistake `Prepared` prevents.

## Concrete recommendations

1. Adopt the `TypedOperation` trio shape (associated `Response`, fresh decoder
   per execution, opaque preparation binding encode+decode) — it is the
   single highest-value pattern tmax offers and it fits the accepted design
   direction verbatim.
2. Adopt the direct-path discipline, with a preflight scope caveat: validate
   everything *checkable without consuming the input* before connecting
   (socket path, config, static payload shape, frame-size bounds); one
   absolute deadline spanning the whole exchange; connect-stage errors retain
   the socket path and are never conflated with later I/O loss; server-side
   `ok:false` is a `Remote` error, never `Ok`. For incrementally produced
   input (streamed `send-text` from a reader), full pre-validation is
   impossible without buffering the stream: preflight the framing envelope,
   let later source errors surface mid-exchange, and classify them as
   `UnknownOutcome{after: Send}` — the command may be partially delivered.
3. Write the no-retry/unknown-outcome contract into the crate docs from day
   one (tmax's phrasing is a good model: "timeout, cancellation, loss, or
   disconnect does not mean a mutation did not execute").
4. Schema-aware decoders: **required** response data missing (or malformed)
   is an error — this retires the `ls`-empty-success class (tmax:
   `ReplyCardinality`, malformed-row-fails-all). Unknown *additive* fields are
   tolerated per command/version schema: kitty grows response payloads across
   releases, and rejecting unrecognized fields would break the client against
   newer servers. Strictness is about absence and shape of promised data, not
   about novelty.
5. Do not build threads, mailboxes, fences, subscriptions, or attestation
   tables. None of tmax's drivers for them exist in the kitty protocol as
   currently understood. If upstream evolution contradicts this (the parallel
   upstream review owns that question), revisit with the middle-tier triggers.
6. For the registry hazard, offer **explicit caller-directed revalidation,
   not automatic reconciliation**: unsolicited auto-reconcile is forbidden by
   the project's own change discipline, and no generic remedy exists. Where
   observable state exists (window/tab listings), the library should make
   re-reading cheap and let the *caller* compare and decide after an ambiguous
   outcome or reconnect. Where no observable state exists (e.g. relative
   font-size increments), the outcome is simply unknown; the honest options
   are caller-chosen (accept ambiguity, re-read via a queryable equivalent,
   or escalate to the user) — the library must not paper over this.
7. Keep an execution-modes table (one screen, like
   `doc/library/execution-modes.md`) contrasting one-shot vs any future
   persistent mode, rather than optional-field conflation.

## Open questions for the parent discussion

- Inspected kitty source (HEAD and v0.43.1) settles that responses carry no
  request identifier (`remote_control.py:260-264, 472-484`; v0.43.1
  `:251-253, 460-471`), so one outstanding exchange is the
  only sound default. Remaining upstream question: is FIFO ordering of
  pipelined *sync* responses on one socket ever guaranteed (event-loop
  serialization), and could upstream be asked to echo a request id so async
  completions become attributable? (Upstream review owns this.)
- What are the exact response frames for streamed/chunked payloads (e.g.
  `send-text`, images) — is there a documented terminator/ack per chunk? The
  current generic chunking is broken in four ways (see Situation); the fix
  should be command-specific and evidence-based.
- Should `version` be a negotiated floor or per-command? Current code hardcodes
  `[0,43,1]`, `[0,14,2]`, `[0,15,0]` in different places — pick one policy.
- Is a "raw escape hatch" response ever encrypted? If encryption is
   all-or-nothing per connection, the pipeline can encrypt exactly once at a
   single choke point (fixing the double-encrypt class permanently).
- Do we want a cheap local connection token exposed for consumers like
  zooming-kittens, or leave identity to the consumer? (tmax's
  `ConnectionId`-is-equality-only, connection-not-instance discipline is the
  right scope if yes.)

## Correction addendum (2026-10-14, after parent review)

The initial version (jj `wsloqquworzs`) was checkpointed; these corrections
are grounded in inspected kitty HEAD (and, where noted, verified at v0.43.1)
read after review. Deltas:

1. **Async correlation removed.** The claim that `async_id` "tags late
   responses" and enables "a small pending-map" was unsupported: responses
   never echo any request identifier (inspected kitty HEAD
   `~/archive/kovidgoyal/kitty/kitty/remote_control.py:472-484`; sync shape at
   `:260-264`; wire field is `async`, `:227,468`; same verified at v0.43.1 at
   `:460-471`, `:251-253`, `:218,456`). One outstanding exchange is
   justified by the *absence* of general response correlation, and the pending
   map / one-outstanding-async-limit middle-tier trigger was replaced by
   serialization plus an upstream protocol-change ask.
2. **Ownership prescription removed.** The `Arc`-split / "not borrowed across awaits"
   prescription was unjustified and conflicted with the minimal exclusive-owner
   model; borrowing `&mut client` across awaits is fine. The real constraint
   is cancellation hygiene: a cancelled read leaves unknown stream state that
   cannot be resynchronized without request ids, so an interrupted in-flight
   exchange invalidates the transport itself — the connection is discarded,
   not merely its buffered input, since further replies may still arrive on
   the socket; no resync claim is currently justified.
3. **Purity boundaries split.** "discovery (pure, no I/O)" and "credentials
   (pure pubkey lookup)" were contradictory; both now carry a pure
   parsing/selection core and a separate I/O edge (enumeration, env/file/db
   access).
4. **Decoder strictness scoped.** "missing/extra data is an error" is now
   "required data missing is an error; unknown additive fields are tolerated
   per command/version schema" — strict about promised-data absence, tolerant
   of upstream additions.
5. **Connection vs instance identity disentangled.** Earlier text correctly
   described tmax's token as connection-not-instance identity; later text
   ("reconnect = new instance identity") conflated them. A reconnect yields a
   fresh *local connection token*; server-instance identity remains unproven.
6. **No automatic reconciliation.** The "revalidation over replay" remedy now
   respects the project rule against unsolicited auto-reconciliation and is
   scoped to explicit, caller-directed revalidation where observable state
   exists; effects without observable state (font increments) honestly remain
   unknown-outcome, caller decides.
7. **Preflight scope limited.** "Validate everything before the first byte"
   now applies to what is checkable without consuming the input; incrementally
   produced input gets envelope preflight plus mid-exchange source errors
   classified as `UnknownOutcome{after: Send}`.
8. **Transport invalidation, not buffer discard.** "marks the connection
   unusable or discards its buffered input" offered a buffer-cleanup
   alternative that is unsound: replies to the interrupted command can still
   arrive on the socket after the user-space buffer is dropped, and without
   response correlation they cannot be attributed or safely drained. The
   requirement is now invalidation/discard of the transport itself after an
   interrupted in-flight exchange, and the "explicit resync decision" option
   was removed — no resync mechanism is protocol-proven.
9. **Citation precision.** `5bf3c2dc6` is the inspected kitty HEAD, not a
   release "baseline"; references were renamed accordingly, and the response-
   correlation facts were separately verified at release tag v0.43.1
   (`e128af470f7c`) with their own line citations.

## Reading pointers (tmax)

- `README.md` — entrance + "Contracts worth knowing" is the densest honest
  summary anywhere in the repo.
- `doc/library/execution-modes.md` — the one-table lifecycle contrast; a good
  template for kitty-rc docs.
- `doc/library/README.md` §"Failure and resource semantics" — deadline
  domains, bounds, and the no-retry policy in implemented form.
- `doc/library/ordering.md` — what cooperative correlation costs and assumes;
  read as a *cautionary* example of protocol-fact-driven complexity.
- `src/session/request.rs:27-160` — completion/proof/receipt design in ~130
  readable lines.
- `src/operation/query.rs:9-77` — the typed-operation core worth copying.
- `src/endpoint/mod.rs` + `src/protocol/native/command.rs` — the one-shot
  discipline most analogous to kitty-rc's needs.
- `design/saturated-budget/implementation0.sol61.md` and
  `design/sync-deadlines/implementation0.sol61.md` — calibration on what
  verification actually costs and what machinery does not fix.
