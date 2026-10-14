---
type: ImplementationReport
title: CLI shared-store adoption and deliberate cleanup
description: Shared bounded TSV I/O, byte-preserving append/init and explicit coordinated maintenance that refuses malformed data.
resource: /design/library-redesign/cli/store0.sol61m.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec5e53929ffezfWrk0e0AOF9Uc"
    title: CLI store implementation, maintenance boundary and isolated byte-preservation verification
  - resource: /design/library-redesign/implementation/access0.sol61xh.md#existing-tsv-store-preserved-not-rewritten
    title: Shared storage implementation and exact remaining CLI gap
  - resource: /access/key-store/src/storage.rs
    title: Inspected bounded read and append preservation boundary
  - resource: /apps/cli/src/main.rs
    title: Original CLI malformed-data loss and read-truncate cleanup
    revision: "jj-commit:6690b62a"
  - resource: /apps/cli/src/store.rs
    title: Current modular store CLI, explicit path selection and coordinated maintenance
verification:
  scope: CLI store commands using isolated subprocess HOME/XDG paths
  evidence: Three nextest subprocess journeys and strict targeted all-target clippy passed on mise-selected Rust
  limits: Cooperative maintenance, not a hostile-filesystem or arbitrary external-writer transaction
---

# Shared store and cleanup boundary

`kitty-pubkey-db` now uses shared `storage::read` and `storage::append`.
Append retains every prior byte, including malformed lines and an unterminated
last record. `init` uses exclusive creation, never check-then-truncate. Normal
lookup refuses malformed stores rather than selecting a misleading clean key.
Reads are bounded to 16 MiB. Three-field legacy records remain accepted.

## Explicit maintenance

`cleanup --quiescent` requires the caller to stop **all non-CLI publishers**.
CLI init/add/cleanup serialize with a create-new sidecar `pubkey.tsv.lock`;
a competing writer fails actionably, without deleting the lock. Stale locks
require deliberate inspection/removal. Shared library append is not itself a
transactional writer and does not participate in this CLI-only lock.

Cleanup refuses malformed/non-UTF-8 input before creating any replacement. It
preserves exact surviving record bytes and file permissions, writes a create-new sibling temporary
file, syncs it, checks that source bytes remain unchanged and atomically renames.
An observed concurrent change aborts replacement. This check does **not** remove
the final race with a noncooperating writer; the explicit quiescence assertion
is the safety boundary. It is not a new database protocol. No cleanup runs on
add/read/init/reconnect. Crash leftovers are not automatically pruned.
Linux procfs inspection is checked explicitly; unsupported platforms or procfs
errors do not become evidence that every stored PID is dead.

## Verified journeys

- Malformed LF/CRLF/no-final-newline and invalid UTF-8 cleanup/get fail, preserving
  every source byte.
- Init preserves existing data; append preserves malformed and legacy prefixes
  and inserts only the needed separator before the new publication.
- Cleanup without `--quiescent` fails; a competing lock blocks add/cleanup;
  successful cleanup preserves a running PID's original three-field CRLF line.
- The utility's explicit `--store` bypasses ambient path lookup; malformed
  selected PID/window environment fails while explicit flags bypass those inputs.

This completes the CLI acceptance gap of `kitty-rc-proto-rs-redesign-access-store`,
on top of the library verification recorded in the
[access report](../implementation/access0.sol61xh.md). Parent owns shared index,
root overview and tracker export updates.

The [current CLI reference](/apps/cli/README.md#shared-public-key-utility) owns
usage; [the historical publication plan](/doc/PLAN-pubkey-db.md#cleanup-logic)
explains the original hook problem but its cleanup-on-add/epoch design is
deliberately not restored. [CLI implementation](implementation0.sol61m.md)
connects this boundary to actual encrypted standalone listing.
