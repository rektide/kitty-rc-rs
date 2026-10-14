---
type: Design
title: Kubeless reconnaissance for the kitty-rc library redesign
description: >
  Initial evidence-backed assessment of ~/src/kube-kubeless (kubeless, a soft
  fork of kube-rs decomposed into ~70 concern crates) as prior art for the
  kitty-rc redesign: request lifecycle correctness, command-specific
  streaming, and the crates-per-concern architecture.
resource: /design/library-redesign/kubeless-assessment0.glm53fx.md
status: draft
generated: { at: '2026-10-14', by: 'model:zai-coding-plan/glm-5.3-flash#max' }
sources:
  - { resource: 'opencode://ses_ec7b25aa3ffeb7vG9NyTo6O0ty', title: 'Commissioned kubeless source reconnaissance: lifecycle/streaming opportunity map, reusable-crate triage, licensing and maturity evidence' }
  - { resource: '~/src/kube-kubeless README.md', title: 'Project overview, entrance table, composition-and-boundaries, unpublished status' }
  - { resource: '~/src/kube-kubeless Cargo.toml', title: 'Workspace members, Apache-2.0, edition 2024 / rust 1.89, workspace lints, dependency table' }
  - { resource: '~/src/kube-kubeless kube/client/src/client/mod.rs', title: 'send/call_with_phase phase capture; request_events newline framing' }
  - { resource: '~/src/kube-kubeless kube/client/src/client/response.rs', title: 'InvocationError NotInvoked/Invoked; request_bytes_with_evidence; CollectedBody' }
  - { resource: '~/src/kube-kubeless kube/client/src/client/retry.rs', title: 'Replay permission, response-only RetryPolicy' }
  - { resource: '~/src/kube-kubeless kube/client/src/client/builder.rs', title: 'Default middleware stack and connector timeouts' }
  - { resource: '~/src/kube-kubeless kube/client/src/config/mod.rs', title: 'Timeout fields/defaults' }
  - { resource: '~/src/kube-kubeless kube/client/src/error.rs', title: 'Remote-Status vs transport error taxonomy' }
  - { resource: '~/src/kube-kubeless kube/client/src/client/auth/mod.rs', title: 'RefreshableToken, expiry margins, AsyncPredicate auth layer' }
  - { resource: '~/src/kube-kubeless kube/client/src/api/remote_command.rs', title: 'AttachedProcess multiplexed streaming: channels, status channel, CLOSE_CHANNEL, join/Drop discipline' }
  - { resource: '~/src/kube-kubeless kube/runtime/src/watcher.rs', title: 'Session-native watcher entrances and error taxonomy' }
  - { resource: '~/src/kube-kubeless observation/session/README.md and src/lib.rs', title: 'Pull-driven snapshot+resume session, cursor semantics' }
  - { resource: '~/src/kube-kubeless observation/backoff/README.md', title: 'Clock-optional classified-recovery backoff' }
  - { resource: '~/src/kube-kubeless work/keyed/README.md', title: 'At-most-one-active-future-per-key' }
  - { resource: '~/src/kube-kubeless control/attempt/README.md', title: 'Completed-vs-Rejected invocation outcome' }
  - { resource: '~/src/kube-kubeless control/settlement/README.md', title: 'Submit-exactly-once, unknown-effect discipline, drop-is-cancellation' }
  - { resource: '~/src/kube-kubeless operation/README.md and Cargo.toml', title: 'Conditional writes with conflict-only bounded retry; zero runtime dependencies' }
  - { resource: '~/src/kube-kubeless view/wait/README.md', title: 'Condition waits, absence-or-identity-changed settlement' }
  - { resource: '~/src/kube-kubeless runtime/README.md', title: 'Cross-recipe lifetime assembly policy' }
  - { resource: '~/src/kube-kubeless kube/core/src/request.rs', title: 'Transport-free request construction split' }
  - { resource: '~/src/kube-kubeless kube/client/src/client/response/tests.rs', title: 'Phase/head-retention/eager-collection test names' }
  - { resource: '~/src/kube-kubeless CHANGELOG.md', title: 'Upstream fork point kube-rs 4.2.0 (2026-07-22)' }
  - { resource: '/design/library-redesign', title: 'Commissioning brief: verified kitty-rc 0.5.0 review, issue list, and 97990cb2 source-state correction (parent session)' }
tags: [kubeless, kube-rs, lifecycle, streaming, crates-per-concern, reconnaissance]
---

# Kubeless reconnaissance for the kitty-rc library redesign

## Situation and commission

The kitty-rc 0.5.0 verified review found request-lifecycle defects (phase
conflation, retry-after-lost-reply hazards, unbounded heuristic framing) and a
possible need for command-specific streaming. The parent commissions crates
per individual concern: separable schema/protocol, streaming,
transport/execution, auth, discovery, credential-store, high-level client, and
CLI. This wave answers: **what does `~/src/kubeless` actually do, and what
exactly can the kitty-rc redesign take from it?**

Method: everything below was read from source, READMEs, tests, and manifests
in the local checkout; nothing is taken from the name. Revision pins:

- **kubeless**: working-copy change `prkzlsknusuo` / commit `ee984da4af6a`
  (empty working copy), parent `pzuswpllmnpw` / `e1e4b63ac504` "broaden the
  session provenance title to cover the facade wave", 2026-10-14. It is a
  colocated jj/git checkout at `~/src/kube-kubeless` (symlinked from
  `~/src/kubeless`). No `AGENTS.md` exists in that checkout; its
  `CONTRIBUTING.md` (upstream kube-rs text) requires DCO signoff.
- **kitty-rc-proto-rs**: the client.rs facts below reflect revision
  `97990cb2` ("Format examples/test-connectivity.rs") per the parent's
  verified correction. One historical finding is already fixed there:
  `client.rs` `send` dropping the inner `write_all` result now uses `?`/
  `?`-chaining around line 201. I treat that defect as **historical design
  motivation only**, not a current defect. The "159 nextest tests pass,
  strict clippy fails" claim also belongs to an earlier session snapshot; I
  ran **no** fresh test or lint checks for this document.

## What kubeless actually is

Kubeless is rektide's **experimental soft fork of kube-rs 4.2.0** (upstream
release 2026-07-22, per `CHANGELOG.md`), restructured so that the Kubernetes
client's mechanisms are "opened" into **independently useful `kubeless-*`
crates**. Root `README.md`: *"Adopt the mechanism you need without adopting a
Kubernetes resource model or a whole controller."*

Structure evidence (`Cargo.toml` workspace members, ~70 crates):

- `kube/` — the Kubernetes composition: `kube/client`, `kube/core`,
  `kube/derive`, `kube/runtime`, plus the `kube` umbrella (upstream parity
  names preserved).
- Domain families, each a directory of small crates with per-crate READMEs:
  `observation/` (session, backoff, reflector, reflection, relation, …),
  `view/` (cache, change, memory, publication, wait, …), `work/` (ready,
  keyed, execute, custody, schedule, …), `control/` (attempt, plan, pass,
  settlement, retained-actions, driver, …), `operation/`, `catalog/`,
  `runtime/`, and `all/` (the `kubeless::` omnicrate facade).
- Facade crates re-export family members behind opt-in features
  (`kubeless-control` starts with attempt/feedback; `pass`, `drive`,
  `scheduled` are feature-selected — root README "Composition and
  boundaries").
- **Discipline**: "executor-free" cores — no required async executor or task
  spawning; a crate may expose `Future`/`Stream`/`Sink` contracts but timing
  and host integration are selected separately; `kubeless-runtime` is the
  composed runtime (root README). Workspace lints: `unsafe_code = "forbid"`,
  `missing_docs = "deny"` (`Cargo.toml [workspace.lints.rust]`).

Status and publishing: all `kubeless-*` crates are **workspace-internal, not
on crates.io**; the README instructs path or git deps *"pinned to a reviewed
revision"* and warns this is not an official kube-rs/CNCF release. Workspace
version was reset to 0.1.0 with exact internal pins (`=0.1.0` path deps, e.g.
`observation/backoff/Cargo.toml`). License is **Apache-2.0** workspace-wide,
retaining upstream kube-rs authors (clux, Natalie Klestrup Röijezon, kazk) in
`[workspace.package] authors`. Edition 2024, `rust-version = "1.89.0"`.

## Relevance map to the two kitty-rc concerns

### #1 Request lifecycle correctness

**(a) Phase-preserving invocation — "definitely not sent vs unknown outcome".**
This kitty requirement has a direct, fork-local precedent.
`kube/client/src/client/mod.rs:224-241` (`call_with_phase`) wraps every
request so that failure *before* the outer service's `call` is typed
`InvocationError::NotInvoked` and failure *after* invocation is
`InvocationError::Invoked` (`kube/client/src/client/response.rs:17-33`). The
doc comment is honest about the boundary: *"Invoked means only that
Service::call was invoked. Middleware, connectors, and proxies may still fail
before network dispatch or replay internally, so this type does not establish
that bytes reached Kubernetes."* Tests pin this:
`response/tests.rs:150` `public_collection_distinguishes_invoked_service_failure`,
`:186` `public_collection_retains_outer_readiness_phase_after_buffer_closes`.

**(b) Response evidence retained on partial failure.**
`request_bytes_with_evidence` (`response.rs:44-65`) returns
`Response<CollectedBody>` where the body is `Bytes` **or** `ReadFailed` — a
body-read failure after a successful response head keeps status, version, and
headers available instead of collapsing everything into one error. This is a
generalizable shape for kitty's "reply arrived but was truncated" case.
Pinned by `response/tests.rs:124` `public_collection_retains_head_when_body_read_fails`.

**(c) Explicit-permission, response-only retry.**
The fork rewired kube-rs retry around a request extension
(`retry.rs:55-66`): `Replay::Never` / `Replay::OnRetryableResponse`, with
*"Permission is explicit: a request without this extension is not replayed."*
The `RetryPolicy` (`retry.rs:154-189`) retries **only** 429/503/504
**responses** (`:131-138`) — never service errors, i.e. never when the
outcome is unknown — and `clone_request` (`:191-210`) refuses to replay when
the body cannot be cloned (`Kind::Once`). Discovery/version calls opt in via
`request.extensions_mut().insert(retry::Replay::OnRetryableResponse)`
(`client/mod.rs:420-481`). This is exactly the answer to kitty's hazard that
*"Registry retry can't replay arbitrary mutations after lost replies"*:
make replay opt-in per request, and only on evidence the server answered.
Tests include paused-time retry-after handling (`retry.rs:304-382`).

**(d) Unknown-effect mutations are never retried — settlement instead.**
`control/settlement/README.md` (whole doc): `acknowledge_then_settle`
*"submits exactly once … neither retries a request with unknown remote
effects nor assumes that acknowledgment means settlement"*; ordinary failures
after acknowledgment retain the original receipt; *"dropping the returned
future is cancellation, and receipt recovery after cancellation requires an
external owner or durable operation record."* The generic vocabulary for
kitty's "distinguish definitely-not-sent / sent-unknown / acked / settled"
states exists here in distilled, dependency-light form.

**(e) Re-derive instead of replay for idempotent-shaped work.**
`operation/README.md`: conditional writes with *bounded* retry where **only a
native conflict** triggers retry and *"every retry performs a fresh read and
fresh preparation"*, with attempt records that keep semantic attempt counts
distinct from writer invocations, and `Outcome::NoWrite` as an explicit
caller conclusion. Kitty has no server-side compare-and-swap, but the
pattern (fresh-read-then-prepare per attempt, conflict-only retry,
attempt-observer hooks) ports to any "check then act" client operation.

**(f) Remote failure vs transport failure in the type system.**
`error.rs:11-116` keeps `Error::Api(Box<Status>)` (the server's structured
failure, with code/reason/message) distinct from `HyperError`, `Service`
(middleware), TLS, auth, and UTF-8/serde variants. Watch streams even parse a
`Status` out of the *middle* of a stream (`client/mod.rs:379-381`) so remote
failure surfaces as a typed error mid-stream. Kitty's `execute` currently
returns `Ok` for remote `ok=false` (commissioning brief); kube's
`legacy_response_text` (`response.rs:80-102`) shows the alternative: parse
the remote failure envelope first, reconstruct a failure when the envelope is
malformed but the HTTP status says error.

**(g) Auth as composable layer with refresh state.**
`auth/mod.rs`: `RefreshableToken` (`:180`) is an enum of token sources
(Exec / token-file / GCP OAuth / OIDC), applied as a tower
`AsyncPredicate` filter (`:190-206`), with expiry margins refreshed
~10s before expiry (`:140`, `:165` TEN_SEC) and file tokens re-read on a
60s cadence (`:135`, `:167`). Concurrency is a `Mutex`/`RwLock` guard around
refresh so concurrent requests don't stampede. Relevant to kitty's
token/password + encryptor state: keep credentials behind a small refresh
interface at the transport boundary, not inside each command.

**(h) Timeouts: connector-phase only by default — a gap kitty must close.**
Default stack timeouts are connect/read/write connector timeouts
(`builder.rs:226-231` via `hyper_timeout::TimeoutConnector`; config fields
`config/mod.rs:147-159`; defaults `:422-423`: connect 30s, write 295s, read
None). There is **no whole-operation deadline layer** in the default stack —
exactly the thing kitty's review says it wants. Take from kubeless the
*placement* lesson (timeouts at the connector + classification of
`TimedOut`/`UnexpectedEof` IO kinds in stream decoding,
`client/mod.rs:388-401`) but not a solution: kitty needs a new whole-op
deadline wrapper of its own.

**(i) Watch-style recovery as a typed state machine.**
`kube/runtime/src/watcher.rs:50-90` classifies source errors
(InitialListFailed / WatchStartFailed / WatchError / WatchFailed /
NoResourceVersion) and adds `ObservationProtocol` / `ReflectionProtocol`
variants for *adapter invariant violations* — wire failures and internal
bugs get different types. Recovery/backoff is factored out into
`observation/backoff`: a clock-optional state machine mapping classified
failures to `Wait`/`GiveUp` decisions over caller-supplied `Instant`s,
forgetting history on progress (README). Compare kitty's `receive` timeout
resetting per read and reconnect retaining stale auth: kubeless shows
classified-failure → policy decisions instead of ad-hoc resets.

**(j) In-flight discipline.**
`work/keyed/README.md`: `KeyedFutures` holds *at most one active future per
equal key* with occupancy checks, replacement, cancellation, and keyed
completion — a small (hashbrown-only) mechanism for the "explicit in-flight /
cancel / disconnected state" kitty wants per connection or per target.

### #2 Command-specific streaming

**(a) `AttachedProcess` — the single most transferable artifact.**
`kube/client/src/api/remote_command.rs` (451 lines) implements kubectl
exec/attach over a multiplexed WebSocket, and it mirrors kitty's situation
(binary frames tagged by channel, one terminal status, streaming data):

- Channel-tagged binary frames: `STDIN_CHANNEL=0`, `STDOUT=1`, `STDERR=2`,
  `STATUS=3`, `RESIZE=4`, and **`CLOSE_CHANNEL=255`** for per-channel EOF
  (`:286-294`), with a comment citing the Kubernetes remote-command v5
  constants. Kitty's `into_chunks` terminator bug (no terminator for small
  image uploads) is the same problem v5's close-channel solves: **EOF must
  be a first-class protocol event, not implied by connection close**.
- A **dedicated terminal-status channel** feeding a oneshot
  (`take_status`, `:262-266`): streamed data channels and the final outcome
  are separate facts — the same split kitty needs between streamed chunks
  and command completion.
- A `select!` message loop (`:296-…`) with 60s ping keepalive
  (`:315-317`), a documented graceful-close degradation (if the peer lacks
  stream-close support, closing the whole socket *"may miss out on any
  remaining stdout data"*), and stdin-EOF signaling that either sends
  `CLOSE_CHANNEL` or degrades.
- Lifecycle hygiene worth copying verbatim in spirit: `Drop` aborts the
  background task (`:118-121`), and `join` **drops all stream ends before
  awaiting** to avoid a documented duplex-buffer deadlock (`:241-260`).

**(b) Bounded newline framing with error-class mapping.**
`Client::request_events` (`client/mod.rs:345-410`) frames a streaming
response with `tokio_util` `FramedRead` + `LinesCodec` over the body,
mapping IO error kinds (client `TimedOut` → treated as end, `UnexpectedEof`
→ tolerated known-server quirk, else typed error) and guarding
`MaxLineLengthExceeded`. Kitty's "local buffer until suffix, cannot retain
surplus" receive loop should become exactly this shape: a codec over the
socket that yields whole frames and *keeps ownership of surplus bytes for
the next frame*. (Note their LinesCodec max is `usize::MAX` — unbounded
line growth is an acknowledged wart, kitty should bound it.)

**(c) Snapshot + resumable change sessions with explicit cursor semantics.**
`observation/session` (`README.md`, `src/lib.rs:1-18`): one pull-driven
`Session` behind four entrances (`paged`, `paged_from`, `streaming`,
`streaming_from`); *"a returned cursor is only the session's volatile next
resume position — not downstream acceptance, durable commitment, a value
revision, or a fence"*, and withholding the next poll *is* the backpressure
mechanism. The kube watcher (`kube/runtime/src/watcher.rs:1-10`) was rebuilt
on this: `session`/`metadata_session` preserve cursors end-to-end, while
`watcher`/`metadata_watcher` remain as pinned upstream-parity adapters that
*explicitly discard* cursors. For kitty this is the template if any
command ever gains a subscription/streaming form: protocol cursor and
consumer acceptance stay distinct.

**(d) Condition waits.** `view/wait/README.md`: executor-free condition
algebra over fresh reads with caller-owned wake streams, and
`conditions::absent_or_identity_changed` generalizing "deleted or
replaced" — useful if kitty grows wait-for-state helpers over its views.

## Architecture evidence for "crates per individual concern"

Kubeless is the strongest local existence proof that the parent's proposed
decomposition can work in Rust, with concrete conventions:

- **Lean cores with adapters at the edges**: dependency evidence —
  `kubeless-operation` has **zero** runtime dependencies (dev-only `futures`);
  `control/attempt`: `pin-project`; `work/keyed`: `hashbrown`;
  `observation/session`: `futures` (std) + `thiserror`;
  `observation/backoff`: session + optional `tokio`;
  `control/settlement`: `futures-core` + `view-wait`.
- **Facades + omnicrate**: family facades with opt-in features and an `all`
  omnicrate (`kubeless::`) as the one-stop namespace (root README entrance
  table; `all/README.md`). Facades are *"import conveniences, not mandatory
  dependency centers."*
- **Transport-free request construction**: `kube/core/src/request.rs` builds
  `http::Request`s from params with no client present (builder struct is
  literally `Request { url_path }`), and typed Api methods live in
  `kube/client/src/api/`. That is the same schema/protocol vs
  transport/execution split the parent proposes for kitty.
- **Foreign-consumer probes**: `probes/**` crates exercise mechanisms
  without Kubernetes types (`probes/observation/model-only`,
  `probes/work/custody-host`, `probes/operation/foreign-conditional`, …),
  enforcing that generics stay substrate-free.
- **Verification story**: `justfile` targets `clippy`, `deny`
  (cargo-deny, `deny.toml`), `test`/`test-all-features`/`test-doc`
  (nextest, `.config/nextest.toml`), `coverage` (tarpaulin.toml),
  `test-external-probes`, `minimal-versions`; `e2e/` boots a real API
  server (Dockerfile + `boot.rs`). Kitty can adopt the shape: per-concern
  probes + one integration harness against a real kitty instance.

**Cautionary evidence**: recent jj history (`lvpypkznnqqo` "naming
assessment wave", `xknmxtrrzyks`, `uqmzluyolprz`, `pnoprukpytuw`) shows
significant rename churn across ~70 crates — the cost of fine-grained
naming without stable consumers. Kitty has *no* consumer-compatibility
obligation (per commissioning brief), so it can afford this once, but the
doc should budget naming as real work.

## Concrete reuse vs inspiration vs gaps

**Potentially reusable as dependencies** (executor-free, kube-free, tiny
deps; all Apache-2.0, unpublished → git-pin to a reviewed revision):

| Crate | What it gives kitty | Deps (evidence) |
| --- | --- | --- |
| `kubeless-control-attempt` | Typed `Completed(O)` vs `Rejected(R)` invocation outcome | pin-project |
| `kubeless-control-settlement` | Submit-exactly-once + ack≠settlement + drop-is-cancellation contracts | futures-core, view-wait |
| `kubeless-operation` | Conditional attempt loop: fresh read+prepare per retry, conflict-only bounded retry, attempt records | **none** |
| `kubeless-observation-session` | Snapshot+resume session machinery, cursor semantics | futures(std), thiserror |
| `kubeless-observation-backoff` | Classified-failure → Wait/GiveUp state machine (clock optional) | session (+optional tokio) |
| `kubeless-work-keyed` | At-most-one-active-future-per-key in-flight set | hashbrown |
| `kubeless-view-wait` | Condition waits over fresh reads | (view family) |

Whether to *depend* on these or re-derive is a real fork: they are
unpublished, moving (`0.1.0` lockstep, rename waves), rust 1.89/edition
2024. The *contracts* are small enough to re-derive under MIT inspiration;
the *machinery* (session, operation) is more than trivial. Decision for the
next wave.

**Architectural inspiration only** (HTTP/Kubernetes-specific or tied to the
hyper/tower stack): `InvocationError`/`call_with_phase` phase capture;
`Replay`-extension retry policy; `CollectedBody` evidence retention;
`AttachedProcess` channel protocol; `request_events` codec framing; the
default-stack composition order in `builder.rs:249-303` (base-uri → retry →
auth → headers → trace → client); discovery's cached-vs-oneshot split
(`discovery/mod.rs`); kubeconfig file loading (`config/file_loader.rs`) as a
credential-store analog (kitty's actual credential store should follow its
own platform conventions, not kubeconfig YAML).

**Gaps — kubeless has nothing for**: Unix-socket transports and DCS-style
framing; kitty's challenge/response encryption (its auth layer assumes HTTP
headers and expiry, not per-message encryption — kitty's "encrypt ONCE"
fix is kitty's to design); whole-operation deadlines (absent even here);
terminal-protocol terminators beyond the v5 close-channel *concept* (the
wire bytes are Kubernetes-specific).

## Dependencies, runtime assumptions, maturity, licensing (as actually relevant)

- **Runtime**: the kube composition is tokio- + hyper- + tower-bound
  (`workspace.dependencies`; `AttachedProcess` spawns, auth locks on tokio
  primitives). The reusable list above is deliberately executor-free — it
  compiles and runs under any executor or none.
- **Rust floor**: edition 2024, rust-version 1.89.0 — newer than kitty's
  current MSRV posture; re-deriving avoids taking that floor.
- **Licensing**: kubeless is Apache-2.0 including retained upstream
  kube-rs authorship; kitty-rc-proto-rs is MIT (license added in
  `74182f2e` "…add MIT license file"). MIT project incorporating
  Apache-2.0 code is fine with notice/NOTICE preservation; inspiration-only
  carries no obligation. Contributions back to kubeless require DCO
  signoff (`CONTRIBUTING.md`).
- **Supply chain**: workspace-internal crates are explicitly not published;
  consuming them means a pinned git dep to a reviewed revision (their own
  README's advice). `deny.toml` (cargo-deny) and `tarpaulin.toml` exist;
  tests run under nextest.
- **Maturity**: the kube composition inherits kube-rs 4.2.0 maturity and
  its test depth (mock-tower tests in `client/mod.rs:599-794`, phase tests
  in `response/tests.rs`, paused-time retry tests, e2e envtest-style
  harness). The `kubeless-*` decomposition itself is young (0.1.0 lockstep
  reset, active rename waves this month) — treat contracts as stable-ish,
  packaging as in-flux.

## Uncertainties and suggested deeper questions for the next wave

1. **Where is the truthful "sent" boundary?** `Invoked` ≠ bytes-on-wire
   (their own doc admits middleware may fail or replay internally). For
   kitty, which must distinguish definitely-not-sent from unknown-outcome
   *at a socket*, the honest boundary is around the actual write. Question:
   should kitty model three phases (constructed / handed-to-transport /
   bytes-written) instead of kubeless's two?
2. **Phase capture vs retry layer ordering**: in the default stack,
   RetryLayer sits *below* the outer service the phase is captured at
   (`builder.rs:249-303`), so a replayed request reports one `Invoked`
   phase for many dispatches. If kitty ever adds safe-replay, how should
   phase evidence expose attempt counts (see `operation`'s semantic
   attempt records)?
3. **Fork-local vs upstream**: `call_with_phase`,
   `request_bytes_with_evidence`, `Replay`, and the session-native watcher
   appear to be fork-local restructurings (upstream kube-rs 4.x has
   `request_text` but seemingly not the evidence/phase API). Verifying
   against upstream kube-rs would show which ideas are proven upstream vs
   single-checkout experiments — worth a look at
   `~/archive/kube-rs/kube` if present before citing them as upstream
   practice.
4. **Whole-operation deadlines**: confirm nothing in the stack provides
   them (tower-http timeout layer is not in the default stack). Kitty will
   need its own deadline wrapper; design it once at the execution boundary
   covering construct→write→await→decode.
5. **Bounded framing**: `LinesCodec` max is `usize::MAX` here; kitty must
   pick a real max DCS-frame size and a surplus-retaining decode strategy.
   Is there prior art in `tokio_util` codecs or elsewhere in the checkout
   for bounded, coalescing frame readers?
6. **Dependency vs re-derive** for the executor-free contracts list —
   factor in rust-floor, unpublished pins, and rename churn.
7. **Credential store**: kubeconfig file loading is the only credential
   *store* prior art here (plus exec-plugin refresh); kitty's discovery +
   credential-store boundaries likely need different platform sources.

## Sources needed to prime the Astra medium agent

- `kube/client/src/client/mod.rs` (esp. 218-241, 345-410),
  `response.rs`, `retry.rs`, `builder.rs`, `response/tests.rs` — phase
  capture, evidence retention, retry permission, framing.
- `kube/client/src/api/remote_command.rs` — multiplexed command streaming
  with terminal-status channel and per-channel EOF.
- `control/settlement/README.md`, `control/attempt/README.md`,
  `operation/README.md` — unknown-effect discipline, invocation outcomes,
  conditional attempts.
- `observation/session/README.md` (+ `src/lib.rs`, `protocol.rs`),
  `observation/backoff/README.md`, `work/keyed/README.md` — resumable
  streams, recovery policy, in-flight sets.
- `kube/core/src/request.rs` vs `kube/client/src/api/` — schema/protocol
  vs transport split.
- Workspace `Cargo.toml`, root `README.md`, `justfile`, `deny.toml` —
  packaging, licensing, verification story.

All kubeless citations are at checkout `ee984da4af6a` (2026-10-14); kitty-rc
references follow the parent-verified `97990cb2` state.
