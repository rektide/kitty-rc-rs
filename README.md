# kitty-rc (redesign in progress)

> Rust crates for the kitty terminal emulator's remote-control (RC) protocol,
> rebuilt around typed exchanges and a concurrent registry. Under active
> redesign; nothing here is a finished client yet.

## Current standing

This repository is in the middle of the redesign described in
[`design/library-redesign/draft1.gpt6a.md`](design/library-redesign/draft1.gpt6a.md)
(registry-first concurrent managed execution, one exchange per connection,
explicit operation ownership). The **legacy 0.5.0 client was removed**
rather than kept as a compatibility facade: its execution path had
double-encrypted sends, implicit credential/subprocess lookup, generic
long-string chunking that corrupted payloads, drifted command wire shapes,
and self-confirming tests. The removal record, disposition map, and
restoration inventory live in
[`design/library-redesign/teardown0.glm53m.md`](design/library-redesign/teardown0.glm53m.md);
the previous implementation is recoverable from jj commit `e78489cb`.

What exists today is the **verified protocol foundation** the rebuilt
execution and registry layers will sit on. There is not yet a session,
client facade, or connection pool — those are the next commissioned stage.

## Workspace layout

| Crate | Owns |
| --- | --- |
| [`protocol/schema`](protocol/schema) | Wire envelope types: `PlainRequest` (with the upstream `async` wire name), `ProtectedEnvelope`, `ResponseEnvelope` classification (terminal success/failure vs streaming acknowledgment), and the declared `SUPPORTED` kitty version `[0, 43, 1]` |
| [`protocol/frame`](protocol/frame) | Incremental DCS framing (`ESC P @kitty-cmd … ESC \`): split-frame reassembly, in-order coalesced-frame delivery with surplus retention, bounded payloads, poisoning on out-of-frame bytes, clean-EOF truncation reporting |
| [`protocol/commands`](protocol/commands) | Typed command catalog (currently the `ls` exemplar) and the salvaged output models (`WindowInfo`, `TabInfo`, `OsInstance`) with tolerant-additive decoding |
| [`access/crypto`](access/crypto) | The one outbound protection boundary: exact `1:<base85>` X25519 key parsing, X25519 → SHA-256 → AES-256-GCM sealing with RFC 1924 base85, explicit inputs only (no env/file/subprocess/clock), plus a decrypt oracle for conformance fixtures |
| [`access/key-store`](access/key-store) | The persisted pubkey record format (TSV): parsing with malformed-line reporting, rendering, newest-per-PID selection |
| [`apps/cli`](apps/cli) | The `kitty-pubkey-db` binary (`init`/`add`/`get`/`cleanup`) over the shared store library |

Deliberately absent until they carry real code: `execution/{exchange,tokio}`,
`access/{endpoint,discovery,credentials}`, `control/registry`,
`observation/listing`, and the `client` facade. Upstream wire facts baked
into the foundation crates were verified against the pinned kitty source
tree (see the teardown and design documents for provenance).

## Building and testing

```sh
cargo build            # build all crates and the kitty-pubkey-db binary
cargo nextest run      # test suite (or: cargo test)
cargo clippy --workspace --all-targets
cargo fmt --check
```

The `kitty-pubkey-db` store stays at `$XDG_STATE_HOME/kitty/pubkey.tsv`
with its existing line format (`pid\twindow_id\tpubkey\ttimestamp`);
existing files remain readable. The legacy automatic cleanup-on-add
behavior was removed — run `kitty-pubkey-db cleanup` explicitly. `scripts/enable-rc.sh`
remains for deliberately configuring a local kitty for future live
interoperability runs.

## Design and research

- [`design/library-redesign/`](design/library-redesign/README.md) — redesign
  home: accepted registry-first draft, preserved research, and the teardown
  handoff for the next implementation stage.
- [`design/upstream-kitty/evolution0.gpt6a.md`](design/upstream-kitty/evolution0.gpt6a.md) —
  revision-pinned upstream evolution, defect inventory, and support-policy
  context.
- [`doc/`](doc/) — historical plans and bug evidence kept as research
  material; not current API documentation.

## License

[MIT](LICENSE)
