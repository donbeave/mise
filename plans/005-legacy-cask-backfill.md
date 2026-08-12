# Plan 005: Backfill provable legacy mise casks to Homebrew metadata

> **2026-08-13 audit override — acceptance invalidated.** The conversion proof
> does not cover Homebrew moved-artifact topology, full target ownership, or
> recoverable transaction state. Use plans 013, 017, and 018. Their branch,
> commit-trailer, drift, and done criteria override this document.

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update `plans/README.md` row when done.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`),
> ONE final PR for all plans. Conventional commits, `git commit -s`, no
> agent trailers. Push to `fork` when done.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- src/system/packages/brew/cask.rs`
> Plans 001–003 changes expected; anything else → compare excerpts, on
> mismatch STOP.

## Status

- **Current acceptance:** INVALIDATED by the 2026-08-13 deep audit

- **Priority**: P2
- **Effort**: M
- **Risk**: HIGH — writes lifecycle authority for payloads installed
  earlier; the July 2026 backfill failure is the direct precedent
- **Depends on**: plans/001, 002, 003
- **Category**: migration
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

Existing machines hold casks installed by the old engine: payload plus
`.mise-cask.toml`, no Homebrew `.metadata`. After plan 003, new installs
write Homebrew metadata — but legacy casks would stay invisible to brew
forever. The approved design (decision 2) converges them via a **gated
backfill**: convert to the full truthful receipt set only when truth is
provable; otherwise report needs-repair with a one-line fix. The July 2026
attempt (`99e2e50a5`, retired by `b91389ddd`) backfilled EMPTY tabs and is
the anti-pattern: backfill without provable truth is forbidden. The
operator's real machines carry ~24 such casks — this is the rollout path
that keeps their bootstraps converging instead of erroring after upgrade.

## Current state

Verified at `origin/main` `d5e0390dc`, `src/system/packages/brew/cask.rs`:

- Legacy receipt (`.mise-cask.toml`) shape — written by the old engine
  (writer at ~3907-3931): `schema_version` (≤3), `version`, `apps`,
  `binaries`, `fonts`, `completions`, `pkg_ids`, `targets` (list of
  `CaskTargetRecord { path, fingerprint }`), `prune_safe`,
  `prune_blocker`.
- `cask_target_record_matches` (~3933) re-fingerprints a recorded target
  and compares — this is the existing fingerprint check the gate reuses.
- Legacy read entry: `read_receipt(&version_dir)` used by
  `installed_cask_version_in` (3807-3850); plan 002 kept this path.
- After plan 003, the truthful writers exist:
  `receipt::CaskReceipt` writer, config writer, timestamped snapshot
  writer, and the `uninstall_artifacts` derivation from a fetched cask
  definition.

Gate conditions (design decision 2 — ALL must hold to backfill):

1. the mise legacy receipt exists and parses (schema_version ≤ 3);
2. every recorded target fingerprint still matches the installed payload
   (`cask_target_record_matches` all true; recorded pkg ids still
   installed);
3. the cask definition for the INSTALLED version is obtainable: the
   receipt's `version` equals the CURRENT catalog version of the cask
   (uninstall directives are version-specific; a drifted version cannot be
   described truthfully).

On success: write the full `.metadata` receipt set (same writers as plan
003, `time` = backfill time, facts from the fetched definition + the
verified on-disk state) and delete `.mise-cask.toml`. On any failed
condition: `NeedsRepair` with a one-line instruction; mutate nothing.

## Commands you will need

| Purpose    | Command                                            | Expected on success |
| ---------- | -------------------------------------------------- | ------------------- |
| Unit tests | `cargo test --all-features system::packages::brew` | exit 0              |
| Lint       | `mise run lint`                                    | exit 0              |

## Scope

**In scope**:

- `src/system/packages/brew/cask.rs` — backfill logic in the
  status/apply path (where plan 002's state model distinguishes legacy
  state), plus tests.

**Out of scope**:

- Payload mutation of any kind (no re-download, no re-stage, no artifact
  moves).
- Formula code; removal paths; e2e (007).
- Any new CLI flag/command — backfill rides existing status/apply.

## Git workflow

- Shared branch; ONE commit:
  `fix(brew-cask): backfill provable legacy mise casks to Homebrew metadata`
- `git commit -s`; push to `fork`.

## Steps

### Step 1: Classify legacy state in the installed check

Extend plan 002's state model: when no `.metadata` exists but
`.mise-cask.toml` does, classify as `LegacyMise { receipt }` instead of
absent. Status maps it to `Installed` ONLY after a successful backfill (or
to `NeedsRepair` on gate failure) — decide the mapping in Step 2; the
classification itself must not mutate anything.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0.

### Step 2: Implement the gated backfill

During status/apply evaluation of a `LegacyMise` cask (under the caskroom
lock):

1. fetch the current cask definition (existing `fetch_cask`);
2. check gate condition 3 (receipt.version == definition.version);
3. check gate condition 2 (all fingerprints match; pkg ids installed);
4. all pass → write `.metadata` receipt set via plan 003 writers
   (`uninstall_artifacts` from the fetched definition; `built_on`/`arch`
   real; `installed_on_request: true`), then remove `.mise-cask.toml`,
   then report `Installed { version }`;
5. any fail → `NeedsRepair` with reason exactly one line, e.g.
   `brew-cask:<token>: legacy mise install cannot be converted (installed
1.2.3 != catalog 1.3.0); reinstall with either 'brew install --cask
<token>' or mise apply after uninstalling`.

Order matters: the receipt set must be fully written before
`.mise-cask.toml` is removed, so an interruption leaves a convertible
state, never an orphaned payload.

**Verify**: unit tests below.

### Step 3: Offline behavior

`fetch_cask` needs the network. If the fetch fails during status, report
the legacy cask as `NeedsRepair` with a "could not verify against catalog"
reason — never `Missing`, never a silent pass-through, never a mutation.

**Verify**: unit test with an unreachable API base (existing test
patterns stub the API base URL).

## Test plan

Unit tests (fake prefix + fake state dir, modeled on
`installed_cask_version_*` tests):

- provable case: legacy receipt + matching fingerprints + version equals
  catalog → `.metadata` set written (receipt bytes match plan-001 writer),
  `.mise-cask.toml` gone, state `Installed`;
- version drift (receipt 1.0.0, catalog 1.1.0) → `NeedsRepair`, nothing
  written, `.mise-cask.toml` intact;
- fingerprint mismatch (edited app) → `NeedsRepair`, nothing written;
- unparseable legacy receipt → `NeedsRepair`;
- pkg-id recorded but not installed → `NeedsRepair`;
- fetch failure → `NeedsRepair` with catalog-verification reason;
- idempotence: running status twice after successful backfill is a clean
  `Installed` no-op (second run takes the plan-002 Homebrew path).

Verification: `cargo test --all-features system::packages::brew` → all
pass.

## Done criteria

- [ ] All seven test cases exist and pass.
- [ ] Backfill writes the complete receipt set before removing
      `.mise-cask.toml` (test asserts intermediate order via injected failure
      or file inspection).
- [ ] No payload file is ever touched by backfill (test asserts payload
      mtimes/hashes unchanged).
- [ ] `mise run lint` exits 0.
- [ ] ONE commit; only in-scope files; `plans/README.md` row 005 updated.

## STOP conditions

- The gate as specified cannot distinguish a case you encounter (e.g. a
  legacy receipt schema older than the fingerprint fields — schema_version
  1 has no `targets`): report; do NOT loosen the gate to "targets exist".
  Legacy receipts without fingerprints are NOT provable → `NeedsRepair`.
- Truthful `uninstall_artifacts` cannot be derived for the installed
  version (definition mismatch mid-flight).
- Backfill would need to move, rename, or rewrite any payload file.
- The one-line repair message cannot name a safe manual path for some
  artifact class (report the class).

## Blocker resolution — 2026-08-12 verification review

- **Condition:** fingerprint validation did not prove that the legacy receipt inventory was complete.
- **Evidence:** a receipt omitting a current catalog artifact could pass because only recorded targets were checked.
- **Options:** trust the partial receipt, derive metadata from catalog alone, or require exact per-class target and package-id equality before backfill.
- **Choice:** require exact inventory equality plus existing fingerprints/version/package checks; mismatch remains untouched as `NeedsRepair`.

## Blocker resolution — 2026-08-12 read-only status audit

- **Condition:** status called the mutating reconciliation path, contradicting controlling invariant 4 and the required non-mutating live validation.
- **Evidence:** `installed()` reached `write_homebrew_metadata` and removed `.mise-cask.toml`; apply already has a distinct reconciliation path.
- **Options:** keep mutating status, suppress live validation, or validate truth read-only in status and reserve conversion for apply.
- **Choice:** status validates and reports a provable legacy install as installed without mutation; apply performs the gated metadata conversion. Decision 2 is clarified accordingly.

## Blocker resolution — 2026-08-12 Linux nightly isolation

- **Condition:** final CI's Linux nightly job failed because the successful
  backfill test created `/Applications/Legacy.app` on the runner.
- **Evidence:** job `93979059823` reported `Permission denied` at that exact
  path; the test's bare app target bypassed its temporary Homebrew prefix.
- **Options:** grant runner permission, skip the test on Linux, or give the
  fixture an explicit `$HOMEBREW_PREFIX/Applications` target.
- **Choice:** use the explicit prefix-relative target. It exercises the same
  backfill behavior while keeping every payload and fingerprint inside the
  disposable test directory; focused and full Homebrew suites pass.

## Maintenance notes

- This path becomes dead code once the legacy fleet converges; mark it
  clearly (`// legacy .mise-cask.toml backfill — remove when fleet
converged`) so a future cleanup can delete reading+backfill together.
- Reviewer focus: mutation ordering (receipts-before-removal) and that no
  gate failure path writes anything.
- Deferred: operator-facing aggregate report of all NeedsRepair casks in
  one run — existing status output already lists every package; no new
  surface.

## Blocker resolution — 2026-08-12

- **Condition:** schema 0/1 receipts and schema 2/3 receipts with no target
  records cannot prove payload identity; catalog lookup can also fail
  before the current definition is available.
- **Evidence:** the legacy schema defaults `targets` to empty, so neither
  current catalog artifacts nor mere path existence can reconstruct the
  historical installed bytes. The existing fingerprint function can prove
  only recorded targets. Package receipts remain independently verifiable
  through `pkgutil` on macOS.
- **Options:** (1) infer targets from today's catalog — rejected because it
  invents history; (2) accept target existence — rejected because edited
  payloads pass; (3) classify all legacy receipts, convert only schema 2/3
  records with non-empty matching fingerprints, matching package receipts,
  and an equal catalog version; otherwise return one-line `NeedsRepair`.
- **Choice:** option 3. Offline catalog failure also returns `NeedsRepair`
  without mutation. Successful conversion atomically writes the complete
  Homebrew metadata set before deleting the legacy receipt, and tests prove
  payload hash and mtime remain unchanged.
