# Plan 003: Write full truthful Homebrew metadata on cask install

> **2026-08-13 audit override — acceptance invalidated.** The historical
> implementation exists, but app activation can delete the installed app,
> external-target ownership is unproven, moved-artifact topology diverges from
> Homebrew, and transaction recovery is incomplete. Do not execute this file as
> a current completion plan. Use plans 013, 015, 017, and 018. Their branch,
> commit-trailer, drift, and done criteria override this document.
>
> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update `plans/README.md` row when done.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`),
> ONE final PR for all plans. Conventional commits, `git commit -s`, no
> agent trailers. Push to `fork` when done.
>
> **Atomicity**: this plan is ONE commit. Never leave an intermediate
> commit where the guard is deleted but full metadata is not yet written,
> or where both `.mise-cask.toml` and `.metadata` are written.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- src/system/packages/brew/cask.rs`
> Changes from plans 001–002 are expected; anything else → compare
> excerpts, on mismatch STOP.

## Status

- **Current acceptance:** INVALIDATED by the 2026-08-13 deep audit

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH — brew will later execute what this plan records
  (`uninstall_artifacts`) destructively; wrong facts damage user machines
- **Depends on**: plans/001-brew-receipt-schema-module.md,
  plans/002-cask-read-side-recognition.md
- **Category**: bug / migration
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

After plan 002, mise recognizes brew-installed casks; but mise-installed
casks are still invisible to real Homebrew because the engine writes a
mise-private `.mise-cask.toml` instead of Homebrew's `.metadata` receipt
set. The approved design (decisions 0, 7, 9) requires engine installs to be
indistinguishable from `brew install --cask`: full truthful `.metadata`
(receipt with `uninstall_artifacts`, `config.json`, timestamped definition
snapshot), no mise marker, guard removed. The July 2026 failure
(`bd2fe92bd`→`b91389ddd`) wrote EMPTY synthetic tabs and was retired; the
bar here is receipts carrying the same lifecycle facts brew itself would
have written — derived from the cask definition the engine actually
installed. A receipt fact you cannot establish truthfully is a STOP, not a
placeholder.

## Current state

Verified at `origin/main` `d5e0390dc`:

- `src/system/packages/brew/cask.rs:283-345` — install path: fetches cask,
  parses artifacts, ownership guard (`homebrew_metadata_present` →
  `bail!("... Homebrew owns this cask ...")` at 290-295, re-checked under
  the caskroom lock at 332-339), stages, installs artifacts, then writes
  the mise receipt.
- `cask.rs:~3907-3931` — the mise receipt writer builds `CaskReceipt {
schema_version: 3, version, apps, binaries, fonts, completions, pkg_ids,
targets (fingerprints), prune_safe, prune_blocker }` and
  `write_durable_file(&caskroom.join(".mise-cask.toml"), ...)`.
- `cask.rs:648-651` — `upgrade` simply calls `install`.
- `cask.rs:895-913` — `cask_ruby_bin()` shells out to PATH
  `brew ruby -e 'print RbConfig.ruby'` to find a Ruby for the cask shim;
  falls back to PATH `ruby`, then `source::ruby_bin()`. The design's
  zero-brew-CLI rule requires removing the `brew` branch.
- Zap/uninstall stanzas: the artifact parser recognizes `"zap"`
  (`cask.rs:4752` match arm); tests exercise zap shapes
  (`cask.rs:6108, 6794-6830`). `auto_updates` appears in fixtures
  (`cask.rs:5684`) — the engine already models it (added in jdx/mise#11084
  / #11107); verify how before wiring upgrade behavior.
- Target receipt structure to produce (verified against real Homebrew
  6.0.17, fixtures from plan 001):
  - `Caskroom/<token>/.metadata/INSTALL_RECEIPT.json` — keys
    `homebrew_version` (= `receipt::EMULATED_BREW_VERSION`),
    `loaded_from_api: true`, `loaded_from_internal_api` (match what real
    brew writes for API installs — confirm from fixture),
    `uninstall_flight_blocks`, `installed_on_request`, `time`,
    `runtime_dependencies`, `source` (`tap`, `tap_git_head`, `version`,
    `path`), `arch`, `built_on`, `uninstall_artifacts` (ordered one-key
    objects mirroring the cask DSL: artifact stanzas that brew records —
    `binary`, `generate_completions_from_executable`, `uninstall`, `zap`,
    …; capture the exact selection/order rule from the plan-001 fixture
    and from a second fixture with a `pkg`+`uninstall` cask before
    implementing).
  - `.metadata/config.json` — compact JSON, `default` map holding the
    engine's actual effective directories (appdir = `/Applications`, etc.
    — the values the engine really used, not copies of someone else's
    machine).
  - `.metadata/<version>/<timestamp>/Casks/<token>.json` — the cask
    definition JSON the engine fetched and installed, verbatim; timestamp
    dir `YYYYMMDDhhmmss.mmm` from the install wall clock.

## Commands you will need

| Purpose    | Command                                                         | Expected on success              |
| ---------- | --------------------------------------------------------------- | -------------------------------- |
| Unit tests | `cargo test --all-features system::packages::brew`              | exit 0                           |
| macOS e2e  | `mise run test:e2e e2e/cli/test_system_install_brew_macos_slow` | pass on macOS CI; skip elsewhere |
| Lint       | `mise run lint`                                                 | exit 0                           |

## Scope

**In scope**:

- `src/system/packages/brew/cask.rs` — install finalize path, guard
  removal, mise-receipt writer replacement, `cask_ruby_bin`, unit tests.
- `src/system/packages/brew/receipt.rs` — only additive helpers needed by
  the writer (e.g. `uninstall_artifacts` builder).

**Out of scope**:

- Legacy `.mise-cask.toml` READING (needed by plan 005's backfill and by
  002's state model — keep it).
- Removal/teardown paths (plan 006).
- Formula code (plan 004). e2e rewrites (plan 007).

## Git workflow

- Shared branch; ONE commit:
  `fix(brew-cask): write full truthful Homebrew metadata on install`
- `git commit -s`; push to `fork`.

## Steps

### Step 1: `uninstall_artifacts` derivation

In `receipt.rs` (or a cask-side builder), derive `uninstall_artifacts`
from the parsed cask definition (`Cask` + `CaskArtifacts`), reproducing
exactly the stanza set and ORDER real brew records. Establish the rule
empirically: take two plan-001-style fixtures (one binary-only cask like
`codex`, one app+pkg/uninstall cask) and match brew's selection. Every
entry must describe something the engine actually installed or the DSL
actually declares (zap targets come from the DSL verbatim). If the engine
does not parse a stanza brew records (e.g. `uninstall` directives beyond
pkg ids), extend the artifact parser to carry it through verbatim as
`serde_json::Value` — do not drop or invent entries.

**Verify**: unit test comparing derived `uninstall_artifacts` against both
fixtures byte-for-byte.

### Step 2: Write the `.metadata` set on install

In the install finalize path (where `.mise-cask.toml` is written today,
`cask.rs:~3907-3931`): write instead, under the existing caskroom lock —

1. `.metadata/INSTALL_RECEIPT.json` via `receipt::CaskReceipt` writer —
   all fields truthful: `time` = now, `installed_on_request` = the request
   really was explicit (bootstrap declarations are on-request), `source` =
   the tap/version/path facts of the definition the engine fetched
   (`tap_git_head`: only if genuinely known — see STOP), `arch`/`built_on`
   = real machine facts gathered natively, `homebrew_version` =
   `EMULATED_BREW_VERSION`.
2. `.metadata/config.json` — the engine's effective cask config dirs.
3. `.metadata/<version>/<timestamp>/Casks/<token>.json` — the fetched
   definition, verbatim bytes.

Stop writing `.mise-cask.toml` for new installs. Reinstall/upgrade over an
existing version must refresh the receipt set the way brew does (new
timestamped snapshot dir).

**Verify**: `cargo test --all-features system::packages::brew` → exit 0;
new unit test asserts the three files exist with expected content under a
fake prefix after a simulated finalize.

### Step 3: Delete the guard; brew-equivalent no-op

Remove both `homebrew_metadata_present` bails (`cask.rs:290-295`,
`332-339`). Semantics after removal (decision 9): if the requested cask is
already installed (per plan 002's state model, either origin), install is
an "already installed" no-op; partial/corrupt state is `NeedsRepair` and
never mutated. Keep `homebrew_metadata_present` only if still referenced
by cask prune (plan 006 rework) — otherwise delete it.

**Verify**: `grep -n "Homebrew owns this cask" src/system/packages/brew/cask.rs`
→ no matches in production code paths (prune skip-reason string may remain
until plan 006).

### Step 4: Upgrade semantics

`upgrade` currently aliases `install`. Ensure the combined path now: skips
casks whose definition declares `auto_updates` unless the installed
version is absent (mirror `brew upgrade` without `--greedy`); uses the
same receipt-refresh as Step 2. Locate the existing `auto_updates`
modeling (fixture at `cask.rs:5684`, logic from jdx/mise#11084/#11107) and
reuse it — do not invent a second flag.

**Verify**: unit test — `auto_updates` cask + installed older version +
upgrade ⇒ no mutation, no receipt change.

### Step 5: Remove the `brew ruby` call

In `cask_ruby_bin()` (`cask.rs:895-913`), delete the `file::which("brew")`
branch entirely. Order becomes: PATH `ruby` → `source::ruby_bin()`. Do not
add any new brew invocation anywhere.

**Verify**:
`grep -rn "which(\"brew\")\|Command::new(brew)" src/system/packages/brew/`
→ no matches.

## Test plan

Unit tests (fake prefix, offline), modeled on existing `cask.rs` tests:

- finalize writes receipt + config + snapshot; bytes of receipt match the
  plan-001 writer (volatile fields normalized);
- `uninstall_artifacts` matches both captured fixtures;
- install over existing installed cask (brew-origin fixture from plan 002)
  → no-op, no file changes;
- corrupt `.metadata` → `NeedsRepair`, install refuses;
- `auto_updates` upgrade skip;
- no `.mise-cask.toml` created by a fresh install
  (`assert!(!caskroom.join(".mise-cask.toml").exists())`).

Differential e2e proof (brew-install vs engine-install byte comparison,
`brew uninstall` round-trip of an engine-installed cask) lands in plan 007;
a reduced macOS-only harness MAY be added early here if plan 007 has not
started, but the full oracle remains 007's deliverable.

## Done criteria

- [ ] `cargo test --all-features system::packages::brew` exits 0 incl. new
      tests.
- [ ] Fresh engine install writes `.metadata` set and no `.mise-cask.toml`.
- [ ] Guard removed; already-installed is a no-op.
- [ ] `grep -rn "Command::new(brew)\|\"brew\", \"ruby\"\|which(\"brew\")" src/` →
      no production matches.
- [ ] `mise run lint` exits 0.
- [ ] ONE commit; `git status --short` clean of out-of-scope files.
- [ ] `plans/README.md` row 003 updated.

## STOP conditions

- A receipt fact cannot be established truthfully (e.g. `tap_git_head` for
  API-fetched definitions: check what real brew writes in the fixture — if
  brew records a real head the engine does not know, report; if brew writes
  `null` for the same install mode, `null` is the truthful value).
- The `uninstall_artifacts` selection/order rule cannot be reproduced from
  fixtures (ambiguous ordering, unknown stanza).
- brew's receipt for API installs contains fields whose values the engine
  cannot know natively (report field + brew's source of it).
- The guard removal exposes an install path that would mutate
  partial/corrupt state (must be `NeedsRepair` first — fix state model, or
  STOP if that requires plan-002 rework).
- Any step would keep dual writes (`.mise-cask.toml` AND `.metadata`).

## Maintenance notes

- This commit is the compatibility keystone: reviewers should diff the
  written receipt against a real `brew install --cask` receipt for the
  same cask/version and demand byte identity (volatile fields aside).
- Future cask-DSL features must extend `uninstall_artifacts` derivation in
  the same commit that installs the new artifact type.
- Deferred: `.metadata` teardown on uninstall (plan 006); legacy backfill
  (plan 005).

## Blocker resolution — 2026-08-12

- **Condition:** The planned public per-cask API source could not truthfully
  emit Homebrew's stable `loaded_from_internal_api: true`, and the planned
  verbatim definition snapshot contradicted the emulated Homebrew source.
- **Evidence:** Homebrew 6.0.17 `api.rb` says the internal API is always used;
  `cask_loader.rb` marks that loader internal; `cask/installer.rb#save_caskfile`
  writes `to_installed_json_hash` (normally `{}`), not the fetched package
  definition. `ArtifactSet#to_a` also re-sorts artifacts; a real-brew oracle
  showed the public font list requires MRI's equal-class endpoint permutation
  before its receipt matches.
- **Options:** (1) keep the public API and write `false` (truthful but fails the
  differential contract); (2) claim `true` while using public data (untruthful,
  forbidden); (3) consume the same platform-specific internal packages API,
  derive its cask struct, record `true`, and reproduce `save_caskfile`.
- **Choice:** Option 3. Official casks now use the internal packages endpoint
  with no silent public fallback; third-party taps retain their truthful public
  API path. The snapshot writer follows `save_caskfile`, and receipt artifact
  ordering follows `ArtifactSet`. This is the only option satisfying truthful
  provenance and the differential oracle.

## Blocker resolution — 2026-08-12 PR fidelity review

- **Condition:** a fixture-derived eight-item endpoint swap had no stable
  Homebrew semantic basis, and unresolved dependencies were rejected only
  after payload download.
- **Evidence:** `ArtifactSet` defines class ordering but no endpoint rule;
  equal-class API order is already deterministic. Runtime dependencies are a
  receipt prerequisite and need no payload facts.
- **Options:** retain the heuristic; emulate Ruby implementation accidents; or
  preserve API order and validate dependencies before fetch.
- **Choice:** preserve source order and reject unresolved dependencies before
  any download or install side effect. The differential oracle remains arbiter.
