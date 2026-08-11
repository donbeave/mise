# Plan 002: Recognize Homebrew-installed casks in status and apply

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. On
> any STOP condition, stop and report. When done, update this plan's row in
> `plans/README.md`.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`,
> rebased onto `origin/main`), ONE final PR for all plans. Conventional
> commits, `git commit -s`, no agent trailers. Push to `fork` when done.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- src/system/packages/brew/cask.rs`
> Changes from plan 001 are expected. Any other change: compare the
> excerpts below against live code; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED — read-side only; no mutation semantics change
- **Depends on**: plans/001-brew-receipt-schema-module.md
- **Category**: bug (production bootstrap failure)
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

This is the production bug. A machine with a valid Homebrew-installed cask
(observed: Codex `0.147.0`, receipt at
`/opt/homebrew/Caskroom/codex/.metadata/INSTALL_RECEIPT.json`) declares
`brew-cask:codex` in mise bootstrap. Current mise status only accepts
mise's own `.mise-cask.toml` receipt, so it reports the cask **missing**;
apply then routes it to install, which **rejects** the same Homebrew
metadata ("Homebrew owns this cask"). Bootstrap dies on perfectly healthy
state. This plan makes status recognize valid Homebrew `.metadata` as
installed state, which makes apply a no-op for such casks — fixing the
failure without touching any write path. The design document (decision 9)
mandates this; the ownership guard itself is deleted later (plan 003), so
an interim build can never overwrite Homebrew state with the old mise-only
format.

## Current state

All excerpts verified at `origin/main` `d5e0390dc`.

- `src/system/packages/brew/cask.rs:587-615` — `BrewCaskManager::installed`
  fetches the current cask definition, then:

  ```rust
  let version = installed_cask_version(&cask, &artifacts)?;
  let state = match version {
      Some(version) => match &req.version {
          Some(requested) if version != *requested => {
              PackageState::VersionMismatch { installed: version }
          }
          _ => PackageState::Installed { version },
      },
      None => PackageState::Missing,
  };
  ```

- `cask.rs:3798-3805` — `installed_cask_version` delegates to
  `installed_cask_version_in(cask, artifacts, state_dir)` (there are two
  cfg variants: production uses `crate::dirs::STATE`, tests use
  `prefix::prefix().join(".mise-test-state")`).
- `cask.rs:3807-3850` — `installed_cask_version_in` returns `Ok(None)`
  ("missing") unless mise's own receipt exists and its recorded
  targets/pkgs still match. A Homebrew-installed cask has no mise receipt →
  `None` → `Missing`. Key excerpt:

  ```rust
  let Some(version) = installed_version(&cask.token) else {
      return Ok(None);
  };
  let version_dir = caskroom_version_dir(&cask.token, &version);
  match read_receipt(&version_dir)? {
      Some(receipt) => { /* schema/fingerprint checks on mise receipt */ }
      None => Ok(None),
  }
  ```

- `cask.rs:3706-3711` — `homebrew_metadata_present(token)` is a bare
  existence check of `Caskroom/<token>/.metadata` (used by the install
  guard at `cask.rs:290-295` and `332-339`, and by cask prune at ~4217).
  The guard STAYS in this plan.
- `PackageState` variants (`src/system/packages/mod.rs:49-70`):
  `Installed { version }`, `Missing`, `NeedsRepair { reason }` (check the
  exact field shape at line 56 before use), `VersionMismatch { installed }`,
  `Unavailable { reason }`.
- Driver behavior (`src/cli/system/driver.rs`): apply queries `installed`
  first and filters `Installed` packages out of the mutation set — so
  status recognition alone makes apply a no-op.
- Existing test exemplars in `cask.rs`: `installed_cask_version_*` tests
  from ~line 7237 (fake prefix via `MISE_SYSTEM_BREW_PREFIX`, fake state
  dir, `write_durable_file` fixtures).

Design contract for this plan (decisions 9 and 10):

- A valid Homebrew `.metadata` receipt for the requested token = installed,
  with the receipt's recorded version (opaque string). An unpinned
  (`latest`) declaration is satisfied by ANY installed version.
- Corrupt/unparseable `.metadata` receipt → `NeedsRepair` (never `Missing`,
  never an install attempt).
- A `.metadata` written by a NEWER brew than the pin parses fine as long as
  required structure is present (extra keys tolerated — plan 001 parser).

## Commands you will need

| Purpose    | Command                                            | Expected on success |
| ---------- | -------------------------------------------------- | ------------------- |
| Build      | `mise run build`                                   | exit 0              |
| Unit tests | `cargo test --all-features system::packages::brew` | exit 0              |
| Lint       | `mise run lint`                                    | exit 0              |

## Scope

**In scope**:

- `src/system/packages/brew/cask.rs` — `installed_cask_version_in` (and its
  callers' state mapping in `installed`), new unit tests.

**Out of scope** (do NOT touch):

- The ownership guard (`cask.rs:290-295`, `332-339`) — deleted in plan 003.
- Any write path (`.mise-cask.toml`, `.metadata` writing) — plan 003/005.
- `cask_ruby_bin` — plan 003.
- Formula code (`mod.rs`, `pour.rs`) — formula status already accepts
  brew-poured kegs via opt-link state (`mod.rs:236-252`).
- e2e scripts — differential coverage lands in plan 007.

## Git workflow

- Shared branch `agent/brew-cask-native-interop-plan`; one commit:
  `fix(brew-cask): recognize Homebrew-installed casks in status and apply`
- `git commit -s`; push to `fork`.

## Steps

### Step 1: Read Homebrew metadata in the installed check

In `installed_cask_version_in`, BEFORE the mise-receipt path, detect
Homebrew state: if `Caskroom/<token>/.metadata/INSTALL_RECEIPT.json`
exists, parse it via plan 001's `read_cask_receipt`:

- parse OK → determine the installed version: the receipt's
  `source.version` when the matching `Caskroom/<token>/<version>/`
  directory exists; if the receipt parses but its version directory is
  absent, treat as corrupt (below). Return that version as installed.
- parse error (`Malformed`/`MissingField`) → return a distinguishable
  error/state so `installed` maps it to `NeedsRepair` with a reason naming
  the token and the file — NOT `Ok(None)`.

The function's return type may need to grow (e.g.
`Result<CaskInstalledState>` enum: `Installed(String)`, `Absent`,
`NeedsRepair(String)`) — if so, update ALL call sites mechanically
(install path at `cask.rs:296/339`, status at `599`, and any prune/import
usage) without changing their behavior for the `Absent`/mise-receipt
cases. Keep the version string opaque end to end.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0.

### Step 2: Map states in `installed`

In `BrewCaskManager::installed` (`cask.rs:587`): Homebrew-recognized
version → `PackageState::Installed { version }` (unpinned request), or
`VersionMismatch` when a pinned `req.version` differs (existing logic
covers this once the version flows through); needs-repair → the repo's
`NeedsRepair` state with the reason. Do not special-case tokens.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0.

### Step 3: Unit tests (fake prefix)

Model after `installed_cask_version_uses_metadata_token` (~`cask.rs:8059`)
and neighbors. Under `MISE_SYSTEM_BREW_PREFIX`, create
`Caskroom/<token>/.metadata/INSTALL_RECEIPT.json` from the plan-001
fixture plus a matching version payload dir, then assert:

1. valid Homebrew receipt + version dir → `Installed` with that opaque
   version (e.g. `0.147.0`);
2. unpinned request + OLDER installed version than the current catalog →
   still `Installed` (apply must not reinstall);
3. truncated/garbage receipt JSON → `NeedsRepair`, reason mentions the
   token; never `Missing`;
4. receipt with a newer `homebrew_version` string (e.g.
   `"7.0.1-3-gdeadbee"`) and one unknown extra key → `Installed`;
5. `.metadata` present but receipt file absent → `NeedsRepair` (bare
   existence is lifecycle authority — never silently install over it);
6. no `.metadata`, no mise receipt → `Missing` (unchanged);
7. mise-receipt path behavior unchanged (existing tests keep passing).

**Verify**: `cargo test --all-features system::packages::brew` → exit 0,
new tests included.

## Test plan

Covered by Step 3 (seven cases). The Codex end-to-end regression (real
brew install → mise status installed → apply no-op) is added to the
differential e2e in plan 007 — note it there, do not add e2e here.

## Done criteria

ALL must hold:

- [ ] `cargo test --all-features system::packages::brew` exits 0 with the
      new tests.
- [ ] The seven Step-3 cases exist and pass.
- [ ] The ownership guard lines are UNCHANGED
      (`grep -n "Homebrew owns this cask" src/system/packages/brew/cask.rs`
      still matches twice in install paths).
- [ ] No write path modified: `git diff` for this plan touches no code that
      writes `.mise-cask.toml` or `.metadata`.
- [ ] `mise run lint` exits 0.
- [ ] `git status --short` — only in-scope files.
- [ ] `plans/README.md` row 002 updated.

## STOP conditions

- Excerpts above no longer match (beyond plan-001 changes).
- `PackageState::NeedsRepair`'s shape (`src/system/packages/mod.rs:56`)
  cannot carry a reason string without a contract change to
  `packages/mod.rs` — report; that file is out of scope here.
- The return-type change fans out beyond `cask.rs` call sites.
- Any test requires running real `brew` (unit layer must stay offline).

## Maintenance notes

- Plan 003 deletes the guard and the mise-receipt write; the state enum
  introduced here becomes the single cask installed-state model.
- Reviewer focus: `Missing` vs `NeedsRepair` classification — `Missing` on
  corrupt metadata would let apply reinstall over Homebrew state, the exact
  accident class the design forbids.
- Deferred: reading `.metadata` config/timestamped snapshots (needed by
  plans 003/006, not for recognition).

## Blocker resolution — 2026-08-12

- **Condition:** `PackageState::NeedsRepair` could store only an installed
  version, while this plan requires a truthful corruption reason naming the
  token and receipt path. Putting the reason in `installed` would falsify
  status JSON's `installed_version`.
- **Evidence:** `src/cli/system/status.rs` serializes the sole field as
  `installed_version`; existing formula repair states have a real version.
  The driver already pattern-matches this variant structurally and can preserve
  the installed value independently from a reason.
- **Options:** (1) overload `installed` with the reason, producing false JSON;
  (2) turn receipt corruption into an immediate error, losing the required
  per-package needs-repair state; (3) add `reason: Option<String>` to
  `NeedsRepair`, keep existing constructors at `None`, and have status expose
  the reason separately.
- **Choice:** option 3. It removes the state-model condition that conflated
  identity with diagnostics, preserves current formula behavior, and expresses
  corrupt cask state truthfully without new CLI/config surface.
