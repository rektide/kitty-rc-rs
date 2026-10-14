---
type: Reference
title: Explicit kitty control CLI
description: Endpoint and authentication selection, finite listing/focus commands, sampled watch and deliberate shared-store maintenance.
resource: /apps/cli/README.md
status: draft
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-14 }
sources:
  - resource: "opencode://ses_ec5e53929ffezfWrk0e0AOF9Uc"
    title: CLI implementation, compiled help inspection, isolated executable journeys and shared-store maintenance
  - resource: /design/library-redesign/draft1.gpt6a.md
    title: Registry ownership, independent access sources and finite observation architecture
  - resource: /design/library-redesign/tickets0.sol61m.md#existing-backlog
    title: Preserved CLI ticket mapping and standalone adoption scope
  - resource: /observation/listing/README.md
    title: Inspected scoped acquisition, custody and fixed-delay polling contracts
  - resource: /design/library-redesign/implementation/access0.sol61xh.md
    title: Explicit discovery and credential binding APIs and remaining shared-store gap
  - resource: /design/library-redesign/capabilities/implementation0.sol61m.md
    title: Typed listing and focus profiles and strict required output
---

# kitty-rc CLI

```sh
cargo build -p kitty-rc-cli
cargo run -p kitty-rc-cli --bin kitty-rc -- --help
cargo install --path apps/cli
```

Both `kitty-rc` and `kitty-pubkey-db` are installed. APIs remain unstable; no
old stub client or separate `list_windows` engine is restored.

## Useful entrances

```sh
# Inside kitty: use its captured KITTY_LISTEN_ON, if a supported route is published.
kitty-rc ls

# Standalone: explicitly select a socket, never an assumed ./kitty.socket.
kitty-rc --to /run/user/1000/kitty.sock list-windows --json
kitty-rc --socket unix:/run/user/1000/kitty.sock active --json
kitty-rc --to /run/user/1000/kitty.sock a --tab 20
kitty-rc --to /run/user/1000/kitty.sock active-window --os-window 10
kitty-rc --to /run/user/1000/kitty.sock goto 31
kitty-rc --to /run/user/1000/kitty.sock g 31 --json
kitty-rc --to /run/user/1000/kitty.sock watch --interval-ms 500 --count 10 --json
```

Aliases: `list-windows` → `ls`/`list`; `active-window` → `active`/`a`;
`goto` → `g`; `watch` → `w`. Global flags can appear before or after the
subcommand. `--match` and `--match-tab` forward kitty listing selectors.

Plain output is readable text. `--json` lists a flat array of window rows;
active returns one row, goto returns a remote-reply confirmation, and watch
returns newline-delimited sample objects. Rows include window/OS/tab IDs,
title, PID, CWD, argv, focus/active facts and foreground process information.
Environment/user-variable maps are deliberately omitted, not assumed safe to
print. stdout contains results only; diagnostics and `--verbose` provenance
go to stderr. Exit codes: success/help `0`, operational failure `1`, parser
usage failure `2`, Ctrl-C `130`.

### Active is not always focused

Default `active-window` selects the unique terminal with `is_focused:true`
from an **unfiltered** response. Missing focus fields or multiple winners fail;
the CLI never substitutes the first per-tab `is_active` window. Empty/no-focused
responses fail distinctly from acquisition failure.

`--tab ID` instead selects that tab's unique `is_active:true` terminal.
`--os-window ID` requires that OS window's unique active tab and that tab's
unique active terminal. Missing required active facts or duplicate IDs fail.
Filtered active selection requires one of these explicit targets; omitted
windows cannot establish global focus. These are sampled facts, not a promise
about desktop/compositor state. `goto ID` sends typed `FocusWindow` with `id:ID`
and awaits its native reply; it does not fake desktop focus or retry mutation.

## Endpoint and config policy

1. Explicit `--to` (alias `--socket`) bypasses endpoint enumeration and malformed
   lower-priority route environment.
2. Selected `KITTY_LISTEN_ON` and `KITTY_PID` are captured through discovery.
3. Otherwise, endpoints from the **declared** `--config FILE` are considered.
   Multiple candidates are an actionable ambiguity; no endpoint is guessed.

`--to` accepts a native Unix path or `unix:/path`, and explicit numeric
`tcp:127.0.0.1:PORT` / bracketed IPv6 routes. Paths containing a colon should
use `unix:`. `KITTY_LISTEN_ON` and JSON endpoints use explicit route schemes.
Inherited `fd:` and abstract `unix:@...` routes fail as unsupported, not
silently replaced by a filesystem socket. Native non-Unicode explicit paths
are preserved. No glob, PID suffix guessing, ambient config search or probe.

Example declared JSON (unknown fields rejected; maximum 64 KiB):

```json
{
  "endpoints": ["unix:/run/user/1000/kitty.sock"],
  "auth": "encrypted",
  "key_store": "/home/example/.local/state/kitty/pubkey.tsv",
  "pid": 1234,
  "password_env": "MY_KITTY_PASSWORD"
}
```

Auth/key/password source selectors use **explicit CLI > selected environment >
declared JSON > defaults**. Environment selectors are `KITTY_RC_AUTH`,
`KITTY_RC_KEY_FILE` or `KITTY_RC_KEY_STORE`, `KITTY_RC_PID`, and
`KITTY_RC_PASSWORD_FILE` or `KITTY_RC_PASSWORD_ENV`. Select only one alternative
per source kind at the winning tier; a higher-tier source overrides the entire
lower-tier selector. Explicit auth bypasses malformed lower-tier auth. Paths
are literal, relative to the process working directory; no tilde expansion.

### Authentication is explicit

Default auth is `plain`; `--auth encrypted` never downgrades after errors.
Explicit credential flags with plain auth are rejected. Explicit plain auth
deliberately ignores lower-priority encrypted configuration/material.

```sh
# Published key file deliberately associated with the selected endpoint.
kitty-rc --to /selected.sock --auth encrypted \
  --key-file /private/server-key --password-file /private/password ls --json

# Existing shared TSV; explicitly declare the route/PID association.
kitty-rc --to /selected.sock --auth encrypted \
  --key-store /private/pubkey.tsv --pid 1234 --password-env MY_KITTY_PASSWORD ls

# Inside kitty, absent a key selector, use its captured route-bound public key.
# Set the selected password variable in your environment yourself.
kitty-rc --auth encrypted --password-env MY_KITTY_PASSWORD active
```

Key defaults to captured `KITTY_PUBLIC_KEY`, bound only to its captured
`KITTY_LISTEN_ON` route and PID hint. It is not usable for an unrelated explicit
endpoint. Key-file/store selection deliberately declares its selected endpoint
association; this is provenance, **not verified process identity**. Store lookup
requires a PID hint, refuses malformed records and preserves all bytes.
No helper subprocess or hidden crypto/transport environment lookup occurs.

Password defaults to the named variable `KITTY_RC_PASSWORD` only in encrypted
mode. `--password-env NAME` selects a variable, not a password value. Credential
files contain a single UTF-8 line (optional final LF/CRLF); malformed/missing
sources fail without fallback. Password values are never included in CLI Debug,
output or provenance. Avoid logging your environment externally.

## Sampled watch and lifecycle

Watch issues one finite managed `ls` at a time using `kitty-rc-listing::Poller`.
`--interval-ms` defaults to 1000; `--count` is optional. Without count, the
caller terminates with Ctrl-C. Delayed/missed slots are skipped, not burst
replayed. JSON records contain local sequence, acquisition time, scheduling
delay, active row, selected scope and sampled changes: `Returned`, `Changed`,
`NoLongerReturned`. The last is **not destruction** or lossless event evidence.
Titles are printed on every successful sample; compact change comparison only
covers membership/parent/focus/active facts.

Failed/malformed/ambiguous acquisitions stop with nonzero status; no prior
successful sample is relabeled fresh, nor is an empty sample fabricated.
The sequential CLI cannot publish out-of-order pending observations. Each
successful acquisition takes/releases its record; at most one accepted baseline
is retained. A 71-sample subprocess journey exceeds the default 64-record quota
without leaking custody. Cadence and output are caller-driven; no subscription,
resume cursor, desired-state controller or background watchdog is claimed.

`--timeout-ms` defaults to 5000 and is an absolute per-operation queue/execution
budget. `--max-response` defaults to 1 MiB (256 bytes–16 MiB allowed). Projection
is additionally bounded by the listing leaf's 16,384-window limit. Every process
explicitly shuts down its registry and stops its owned driver. Ctrl-C cancels
accepted work locally, reports retained progress/uncertainty before release,
and never claims remote rollback or replay. This custody is in-process, not
crash-durable history.

The CLI deliberately chooses the managed entrance. Library callers may instead
use low-level `Session::execute`: dropping that future invalidates its owned
transport. In contrast, dropping a managed waiter detaches interest while the
registry retains accepted work; cancel/release/shutdown are explicit owner
actions. The CLI does not expose a second direct-mode engine with different
hidden retry/cancellation behavior.

## Shared public-key utility

```sh
kitty-pubkey-db init
kitty-pubkey-db add                         # selected KITTY_PID/PUBLIC_KEY/WINDOW_ID
kitty-pubkey-db get 1234
kitty-pubkey-db --store /declared/pubkey.tsv add --pid 1234 --pubkey '1:...'

# Stop all external publishers first. This assertion is a real safety boundary.
kitty-pubkey-db cleanup --quiescent
```

Default path is `$XDG_STATE_HOME/kitty/pubkey.tsv`, falling back to
`$HOME/.local/state/kitty/pubkey.tsv`; no missing-HOME `./` guess. `--store`
selects a native path. Explicit add fields bypass their corresponding environment;
malformed selected numeric environment fails rather than disappearing silently.
Shared parser/storage preserves three/four-field TSV and unterminated records.
Init is create-new and cannot truncate an existing file. Get refuses malformed
data. Append preserves every old byte, including malformed records.

Cleanup is Linux procfs maintenance, not automatic pruning. It refuses malformed
input without rewriting, preserves surviving line bytes and permissions, and
uses a CLI-writer sidecar lock plus create-new temporary/sync/changed-byte check
and atomic rename. **Non-CLI publishers must be stopped:** library append and
unrelated writers do not honor that CLI-only lock. The final external-writer race
is not solved by a byte comparison; `--quiescent` is the explicit boundary.
Stale locks/temp files are inspected and removed deliberately, never automatically.
See [maintenance evidence](../../design/library-redesign/cli/store0.sol61m.md).

## Verification and architecture

Tests use actual binaries with `env_clear`, scratch HOME/XDG directories and
private Unix peers. No human endpoint/config/password/key files are used.

```sh
cargo nextest run -p kitty-rc-cli
cargo check -p kitty-rc-cli --all-targets
cargo clippy -p kitty-rc-cli --all-targets --no-deps -- -D warnings
cargo fmt -p kitty-rc-cli --check
```

Entrypoint binaries are thin. `interface`, `configuration`, `control`, `output`
and `store` own distinct application concerns; direct leaf dependencies expose
actual typed catalog, discovery, credential policy, registry and observation APIs.
See [implementation report](../../design/library-redesign/cli/implementation0.sol61m.md)
for native ticket standing, actual gates, legacy standalone adoption and limits.
