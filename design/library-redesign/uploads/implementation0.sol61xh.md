---
type: ImplementationReport
title: Bounded command-specific uploads and exact managed accumulator lanes
description: PNG background/logo and buffered async run execution, independent source-backed fixtures, explicit alias coordination and interruption policy, and private-headless evidence.
resource: /design/library-redesign/uploads/implementation0.sol61xh.md
status: draft
generated:
  by: "model:openai/gpt-6.1-sol#xhigh"
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec5ea232fffeiw0Rb2oUZbPbED"
    title: Parent-commissioned upload/run implementation, native source audit, bounded ownership corrections, scripted and private-headless verification
  - resource: /design/library-redesign/draft1.gpt6a.md#specialty-constraints
    title: Registry-first concurrency with command-specific shared-accumulator exclusion
  - resource: /design/library-redesign/kubeless-design0.gpt6a.md#2-what-kittys-streaming-protocol-actually-requires
    title: Source-backed first-chunk handshake, EOF, suppression and async-result requirements
  - resource: /design/library-redesign/capabilities/inventory0.sol61m.md
    title: Stable finite/nested catalog and deferred upload/run boundaries actually read
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Predecessor finite catalog, strict RunOutput decoder, and deliberately open typed run acceptance
  - resource: /design/library-redesign/implementation/access0.sol61xh.md
    title: Access handoff and preserved setup-generation, stale-lease, rejected-input and retirement contracts
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/base.py
    title: Directly read singleton StreamInFlight, replacement on differing IDs, explicit empty terminator and 128-MiB ceiling
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/run.py
    title: Directly read repeated run options, raw 4096-byte source reads, buffered stdin and final stdout/stderr/exit callback
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/set_background_image.py
    title: Directly read documented 512-byte PNG chunks, repeated options and finite remove/index bypass
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/rc/set_window_logo.py
    title: Directly read logo 2048-byte specification, Python 512-byte generator, self/alpha/position and EOF behavior
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/tools/cmd/at/set_window_logo.go
    title: Directly read actual Go logo/background 2048-raw-byte reader divergence and first-frame-only Stream flag
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/tools/cmd/at/run.go
    title: Directly read Go run source EOF frame and raw 4096-byte bound
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/tools/cmd/at/socket_io.go
    title: Directly read handshake-before-next-chunk and terminal suppression ordering
  - resource: https://github.com/kovidgoyal/kitty/blob/b2cb871254fbdac19a085188813c8dfb258b95cd/kitty/remote_control.py
    title: Directly read envelope stream/async injection, stream acknowledgment and no_response ordering
  - resource: https://github.com/kovidgoyal/kitty/tree/5bf3c2dc676efd5c4b4526eb135c922701565707
    title: Selected stable-to-HEAD diff confirms upload/base/run unchanged; background prose and unrelated decrypt guard differ
verification:
  scope: Pure profiles, actual pinned constructor execution, scripted Unix conversations and local managed scheduling, with an isolated installed-kitty journey
  evidence: Targeted upload/exchange/registry tests and workspace gates recorded below; private Weston/kitty 0.48.2 image/run conformance passed
  limits: Not a complete supported-version live matrix, external-client exclusion, remote rollback, live process output stream or hard memory/preemption sandbox
---

# Uploads and exact accumulator lanes

## Implemented outcome and standing

`protocol/commands::upload` now contains executable **BackgroundPng, LogoPng and
Run** profiles. Run is no longer decoder-only: bounded stdin is sent as a native
upload conversation and its associated output is the catalog's strict binary
`RunOutput`. There is no `RunEmpty` bypass. Empty run input still owns the same
managed run lane, even though it needs no first-nonempty handshake.

This completes the upload capability and remaining typed-run implementation
acceptance; supported-version conformance remains the separately owned live
matrix. Earlier [catalog inventory](../capabilities/inventory0.sol61m.md) and
[finite implementation](../capabilities/implementation0.sol61m.md) describe their
preserved predecessor stage, not an assertion that uploads are still absent.

Native work anchors: `kitty-rc-proto-rs-redesign-capabilities-upload` and
`kitty-rc-proto-rs-redesign-capabilities-typed`. Checkpoints `351418b0` and
`0fb3e612` preserve the initial engine/lane stage and subsequent source-budget,
destructor and group-freezing refinements. `d0e3c30e` preserves bounded native-header
classification, fuller image fixtures and the explicit binary runner hook.
Completion is not human API acceptance.

## Independent public entrances

| Concern | Entrance | Ownership/meaning |
|---|---|---|
| Pure profiles | `BackgroundPng`, `LogoPng`, `Run`, sealed `UploadCommand`, `UploadKey` | Typed immutable metadata, output decoder, audited raw-byte bound and command key |
| Pure preparation | `prepare_upload`, consuming `PreparedUpload::frame(...).protect(...)` | Bounded metadata, stable random IDs, per-frame fresh protection; no source reads or body-sized preparation |
| Direct execution | `Session::upload(command, UploadSource, execution::UploadOptions)` | Caller owns one conversation; cancellation retires transport; `last_evidence()` remains inspectable |
| Managed execution | `Registry::submit_upload(reservation, endpoint, command, source, registry::UploadOptions)` | Same synchronous acceptance cut and detachable `Operation<C>` as finite work |
| Explicit aliases | `coordination_group(endpoint)`, `assign_alias(endpoint, group)` | Owner-declared same-server association, never inferred from path/PID/key/generation equality |
| Interrupted lane | `lane_claim`, `lane_inventory`, `resolve_lane(claim, AllowNewStreamToSupersede)` | Exact occurrence decision to permit a NEW stream, not replay or remote cleanup |
| Retained unpolled input | `take_rejected_upload(token)` | Frozen metadata plus original source after queued cancellation/deadline/setup rejection; original occurrence stays spent |

The two upload option types are deliberately separate: direct options include
Session framing/deadline limits; managed options include existing operation
deadline/response reservation plus the cumulative raw input limit. Leaf crates
are fully usable without the facade. Existing finite execution signatures remain
source-compatible. Parent-owned facade/index work can re-export the new source,
option and lane types with unambiguous names.

## Command-native conversation, not string splitting

| Profile | Maximum raw bytes/frame | Terminal contract |
|---|---:|---|
| Background PNG | 512 | Native unit reply, or explicitly suppressed terminal |
| Logo PNG | 2048 | Native unit reply, or explicitly suppressed terminal |
| Run | 4096 | One async result after process exit: stdout/stderr base64 plus signed exit fields |

Background follows its conservative documented/Python bound, not the Go client's
2048-byte background reader. Logo follows its specification and Go raw-byte bound;
Python happens to send 512. Run's observed readers confirm 4096 is a **raw byte**
bound, not a 4096-character base64 split. Per-frame protection/framing expansion
is bounded separately.

The executor owns these transitions:

1. Bounded initial source read; PNG requires its eight-byte signature. Empty or
   invalid-prefix PNG fails locally before transport write. Signature validation
   is not whole-image integrity validation; kitty owns that terminal interpretation.
2. Only the first nonempty frame sets `stream:true`. Read and validate the stream
   acknowledgment before consuming any remaining source input. Negative/malformed
   acknowledgment stops demand.
3. Preserve options and the same random `stream_id` in every payload/envelope.
   Run preserves one independent random `async` ID through all chunks and EOF.
4. Drive bounded later source reads and pending frame writes while noticing early
   inbound refusals. A coalesced acknowledgment plus early failure is consumed in
   order before more source demand. Partial inbound framing lives on the connection,
   not in canceled read futures.
5. Only real AsyncRead EOF emits **one** `data:""` marker. Run with initially empty
   input sends that marker without a fictitious handshake. Source errors or total
   size failure after partial upload never finalize truncated stdin.
6. Read the classified terminal result, unless an audited image submission profile
   suppresses it. Run has no suppression implementation; a compile-fail doctest
   verifies this restriction. Suppression never removes the required handshake.

`UploadSource` is an owned `AsyncRead + Unpin + Send`, not a generic string chunk
iterator. Standard short reads are data; a zero read is native source EOF. This
avoids mistaking an empty producer chunk for EOF. Byte sources are explicit known
buffers; reader sources can have unknown total length and are never collected into
a full body. Binary and Unicode-containing stdin remain exact bytes before base64.
Target/options remain exact typed payload values, without ambient target discovery.

Finite `prepare` also refuses native accumulator commands disguised as raw/custom
one-frame inputs, including empty run. Finite image `-`/`index:` operations remain
available and do not touch the accumulator. Other raw/custom finite profiles retain
their default unaudited-retirement policy; arbitrary raw upload conversations need
separately owned transport, not a bypass of managed invariants.

## Lanes, generations and native uncertainty

Each registered endpoint initially owns a local `CoordinationGroupId`. Known
aliases explicitly join one existing group before their old group has accepted
work. A group's membership freezes when **any** member accepts work. New unused
aliases may join it; used groups cannot be silently split by endpoint, credential
or binding-generation changes. There is no implicit canonicalization or PID-based
server-identity claim.

The exclusion key is `(group, UploadKey)`, not endpoint-wide serialization.
Eligible work is selected **before** taking socket/class capacity. A blocked alias
upload consumes only its bounded accepted/input/output reservations, not an active
execution slot. Short work, text acquisition, another command's upload and another
endpoint remain eligible under the existing quotas. Per-endpoint quotas remain on
`EndpointId`; a coordination group is not an invented physical-server quota.

Claims include the exact operation token and driver occurrence. Successful native
terminal completion or proven `NotWritten` completion releases the matching claim.
Stale completion/release cannot unblock a successor. A typed output-decode failure
after a valid native success remains a different fact from unknown wire completion.
Source/transport cancellation, negative early reply, driver loss or suppressed
terminal result after writing retain the lane conservatively.

Inspect **both** outcome custody and `lane_inventory()`: a known remote refusal
or successful local submission can leave accumulator ownership unknown even when
`Summary::unresolved` is false for the command's native outcome. `release(token)`
refuses while its lane remains retained. `resolve_lane` requires the exact claim
and a non-running operation. `AllowNewStreamToSupersede` grants deliberate use of
a new stream ID, which may replace a native incomplete accumulator; it does not
send cleanup, rerun the old operation or establish child rollback.

The initial policy holds run's lane through terminal child completion. The source
clears its accumulator before launching the child, but the wire has no distinct
final-input-consumed acknowledgment. Locally writing EOF alone cannot order a new
socket's stream after the old EOF was consumed. Holding longer is conservative
without globally serializing unrelated work. Suppressed image submission likewise
has no terminal proof; its held claim requires the explicit native policy above.

This is local coordination within one Registry and its clones. Independent
registries and external clients can interfere; neither random IDs nor closing a
socket prove server accumulator cleanup. No automatic replay, reconciliation,
alias merging, lane pruning or cleanup was added.

The [access handoff](../implementation/access0.sol61xh.md) remains intact: queued
stale preparation is retained with its exact rejected setup basis; already leased
work keeps old setup; stale completions never enter newer pools; socket retirement
reserves slots until actual outside-lock destruction. Group identity is independent
of those setup coordinates.

## Resource and deadline boundaries

Metadata and maximal encoded/protected expansion are checked before acceptance.
Managed input reservation includes conservative JSON-tree expansion for repeated
metadata, one raw chunk, one protected pending frame, fixed executor future/read
buffer storage, and all source-declared ownership. A layout test checks the upload
future's fixed storage against its preparation reservation without polling it.
Inbound bytes/frame count and total raw upload bytes are separately bounded.
Native response classification uses `IgnoredAny` for success data rather than
expanding an untrusted JSON body inside the driver. Typed output allocation belongs
to the sole consuming caller take, as in the existing registry.

`UploadSource::bytes` counts actual Vec capacity. `UploadSource::reader` requires
an honest caller-declared retained-memory bound, including hidden buffers, future
allocations and owned error causes; arbitrary Rust readers cannot be introspected
or sandboxed. When a source error moves into native outcome custody, its declared
input reservation follows it until take/release. Queued cancellation, deadline and
setup rejection retain unpolled sources under quota until explicit take/release.
Caller-owned reader/error destructors run outside the registry lock; a reentrant
destructor test checks this ownership boundary.

One absolute deadline covers queue, source reads, connection, fresh per-frame
protection, writes, handshake, later chunks, EOF and terminal response. Bounded
synchronous preparation checks the same deadline. It is cooperative, not a hard
preemption promise for arbitrary blocking readers/custom decoders. Managed typed
decoding happens later in the caller's sole take, not as an independently timed
resident task. `last_evidence().input_complete()` means locally written EOF, never
verified accumulator cleanup.

## Verification actually performed

Toolchain: user-selected `mise exec --` nightly-2026-10-14; no Rust/MSRV pin added.

- [Pure fixtures](../../../execution/exchange/tests/upload_profiles.rs) verify
  native metadata, command-specific raw/base64 bounds, stable IDs, exact EOF payload,
  finite-upload rejection and independently decrypted fresh protection per frame.
- [Pinned constructor oracle](verify-source0.sol61xh.py) directly executes three
  stable source constructors through AST, without native kitty imports or terminal
  I/O. It verifies repeated options, exact binary chunks, Python image bounds and
  run's raw bound. Library implementation is independent, not GPL translation.
- [Scripted Unix tests](../../../execution/tokio/tests/uploads.rs) cover empty/small/
  exact/large run, both image bounds with small/exact/over-bound inputs, one EOF,
  handshake demand gating, suppression, refusal, coalesced early error, late producer
  failure, cancellation and one absolute trickled/stalled-source deadline.
- [Fault cases](../../../execution/tokio/tests/upload_faults.rs) reject malformed
  handshake and late cumulative limit without finalizing partial input.
- [Managed journeys](../../../control/registry/tests/uploads.rs) verify alias/empty-
  run conflicts, unrelated short/text/other-endpoint progress, generation-independent
  lanes, retained interrupted policy, source/error byte custody and outside-lock drop.
  Unit tests exercise stale occurrence release against an actual successor claim.
- Targeted exchange/tokio/registry run after bounded-header refinement: **53/53
  passed**. Final shared workspace gate, including the subsequently added managed
  suppression policy test and concurrent CLI/check work: **185/185 passed**, strict
  workspace clippy, locked all-target check, formatting and workspace doctests
  passed. Run suppression's compile-fail doctest passed. These shared check results
  do not attribute other authors' implementations to this session.
- `mise exec -- python3 scripts/verify-isolated-uploads.py` passed with **installed
  kitty 0.48.2** under a new private Weston instance. Actual encrypted background/
  logo PNG uploads, suppressed-terminal handshake, empty/1/4096/16385-byte run with
  exact binary stdout/stderr and exit 7, and managed alias/text execution passed.
  The script creates an independent valid PNG with computed CRC/zlib data. No human
  terminal, ambient RC endpoint or user config was selected or mutated.

### Human-directed Rust verification migration

After these gates, the human rejected further Python verification proliferation
and commissioned G to migrate orchestration to Rust. The Python runner/oracle are
**checkpointed evidence and temporary migration inputs**, not a newly selected
long-term verification stack. This session stopped extending them and released
those paths plus `control/registry/examples/upload_conformance.rs` to that work.
The core Rust profiles/executor/registry/tests are complete independently.

G's exact inputs and assertions:

- Private runner input: optional `KITTY_RC_TEST_KITTY` binary path; otherwise PATH's
  kitty. Private password config, Unix socket and Weston environment are created
  under `/tmp/opencode`; ambient RC/Wayland/Display fields are removed. Cargo is
  launched under inherited user-selected mise Rust.
- Rust example: package `kitty-rc-registry`, example `upload_conformance`, arguments
  `SOCKET KEY PASSWORD PNG`. It checks encrypted background/logo native replies,
  suppressed image handshake, run input sizes 0/1/4096/16385, exact binary stdout,
  stderr `[0,255,128]`, exit code 7/status 1792, then managed aliases plus text and
  empty final lane inventory.
- PNG fixture: independent 1x1 RGBA, zlib scanline plus computed chunk CRCs. Preserve
  a real valid image, not merely a signature pretending to be a valid PNG.
- All `python3 -c` environment-export/echo/sleep helpers here are replaceable Rust
  child modes, **not native kitty plugins**. Replace the example's Python target
  program too when removing Python fixture dependencies; do not embed Python in
  Rust and describe that as translation.
- Source oracle input: local kitty archive, pinned commit
  `b2cb871254fbdac19a085188813c8dfb258b95cd`. Its actual-method execution is genuinely
  Python because those upstream methods are Python. A pure Rust replacement should
  preserve native-source/reference-client evidence and documented Python/Go chunk
  divergence, rather than translating a stub and claiming actual upstream method
  execution. No unavoidable custom native `.py` plugin exists in this slice.

Keep the original scripts/fixtures and this evidence recoverable through commits;
the migration owner decides new maintained Rust paths. Parent-owned supported-
version matrix and final converged gates remain separate work.

The live runner accepts `KITTY_RC_TEST_KITTY=/absolute/path/to/kitty`, otherwise
PATH's kitty, and prints its actual version. It keeps private config, runtime dir,
socket and Wayland display. Example:

```sh
KITTY_RC_TEST_KITTY=/path/to/kitty mise exec -- python3 scripts/verify-isolated-uploads.py
python3 design/library-redesign/uploads/verify-source0.sol61xh.py /home/rektide/archive/kovidgoyal/kitty
```

Installed 0.48.2 evidence is **not** a stable 0.49.2/HEAD live assertion, a selected
global support floor, complete native accumulator-interference matrix, visible
image-rendering proof or live stdout event API. The separately commissioned live
matrix owns those supported-version claims. Parent owns tracker export, root
README/shared index and optional facade re-export naming. No push/squash/jj edit,
subagent spawn, tracker migration or restoration of other authors' work occurred.

## Cross-references

- [Upload home](README.md) ⇄ this report: current entrance and evidence map.
- [Draft 1 specialty constraints](../draft1.gpt6a.md#specialty-constraints): adopted
  narrow-lane architecture, preserving concurrency as the primary registry payoff.
- [Kitty-native exchange research](../kubeless-design0.gpt6a.md#2-what-kittys-streaming-protocol-actually-requires): source precedent implemented here rather than a universal message splitter.
- [Finite catalog stage](../capabilities/implementation0.sol61m.md#upload-boundary-and-precise-next-seam): predecessor whose run decoder/open acceptance this slice completes.
- [Access generations](../implementation/access0.sol61xh.md#registry-setup-ownership-and-exact-generations): preserved setup boundaries, not a new server identity or upload lane.
- [Private runner](../../../scripts/verify-isolated-uploads.py) and [Rust journey](../../../control/registry/examples/upload_conformance.rs): reusable isolated verification hook for the parent-owned live matrix.
