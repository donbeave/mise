# Plan 006: Align prune and uninstall post-state with brew uninstall

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update `plans/README.md` row when done.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`),
> ONE final PR for all plans. Conventional commits, `git commit -s`, no
> agent trailers. Push to `fork` when done.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- src/system/packages/brew src/cli/system/prune.rs`
> Plans 001–005 changes expected; anything else → compare excerpts, on
> mismatch STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH — removal code executes destructive actions; parity
  errors delete wrong things or leave residue brew chokes on
- **Depends on**: plans/001, 003, 004
- **Category**: bug / migration
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

Install-side 1:1 (plans 003/004) is half the contract. If mise removal
leaves different residue than `brew uninstall` — stale `.metadata`,
unexecuted uninstall directives, half-torn kegs — mixed machines
accumulate divergence and the round-trip oracle (plan 007) cannot pass.
The approved design (decision 5) puts formula prune, cask prune
(jdx/mise#11810), and cask uninstall/reinstall teardown all under the same
bar: post-state identical to what real `brew uninstall` leaves.

## Current state

Verified at `origin/main` `d5e0390dc`:

- Formula prune: `src/system/packages/brew/maintenance.rs:150-161`
  `apply_prune_plan` — dry-run prints `remove brew:<name>@<version>`, then
  `unlink_and_remove_keg(candidate)` per candidate;
  `unlink_and_remove_keg` at 226-243 removes public links and
  `file::remove_all(&candidate.keg)`. Existing tests at 443+ (fake
  prefix).
- Cask prune (arrived with #11810): plan builder in `cask.rs` (skips
  Homebrew-owned via `.metadata` existence check near 4217:
  `reason: "Homebrew owns this cask"`, requires exactly one installed
  version, requires a parseable mise ownership receipt);
  `apply_cask_prune_plan` at `cask.rs:4328`, `_in` variant at 4332. CLI:
  `src/cli/system/prune.rs` — `--manager brew|brew-cask` (line 24),
  `run_brew` confirms then `apply_prune_plan` (76-92), `run_brew_cask`
  builds plan, warns skips, confirms, `apply_cask_prune_plan` (98-138).
- Reinstall teardown: the install path replaces existing app/binary/font
  targets (`previous_binary_targets` etc., `cask.rs:341-345` area) — after
  plan 003 the metadata refresh must accompany it.
- What real `brew uninstall` does (parity target — verify each against a
  real-brew fixture run in plan 007's harness, but implement from the
  recorded `uninstall_artifacts` now):
  - executes the recorded `uninstall`/artifact directives of the INSTALLED
    version (quit/launchctl/pkgutil-forget/delete/binary+completion link
    removal…), from the receipt — not from today's catalog definition;
  - removes `Caskroom/<token>/<version>/` and, when no versions remain,
    the token dir INCLUDING `.metadata`;
  - formula: unlinks opt/linked/public links, removes the keg, removes the
    empty rack; `zap` is NOT part of plain uninstall — never execute zap
    directives during prune/uninstall.

Design constraints in force: dry-run previews without mutation; failures
abort before the first removal; version strings stay opaque; `NeedsRepair`
/ Homebrew-owned-skip semantics of the prune planner change — after plans
002/003 the state model is unified, so cask prune candidates are
"engine-recognized installed casks not in config", regardless of origin,
EXCEPT that prune must still refuse when the receipt (Homebrew or legacy)
cannot be read (never delete what you cannot classify).

## Commands you will need

| Purpose    | Command                                                    | Expected on success |
| ---------- | ---------------------------------------------------------- | ------------------- |
| Unit tests | `cargo test --all-features system::packages::brew`         | exit 0              |
| Linux e2e  | `mise run test:e2e e2e/cli/test_system_install_brew_linux` | pass on Linux CI    |
| Lint       | `mise run lint`                                            | exit 0              |

## Scope

**In scope**:

- `src/system/packages/brew/maintenance.rs` — formula removal parity.
- `src/system/packages/brew/cask.rs` — cask prune planner/apply +
  uninstall/reinstall teardown reading `uninstall_artifacts`.
- `src/cli/system/prune.rs` — only if planner/apply signatures change;
  preserve CLI UX exactly (flags, dry-run output shape, confirmation).

**Out of scope**:

- Prune candidate-selection SEMANTICS beyond the origin unification stated
  above (what is "configured" stays as-is).
- `brew autoremove`-style dependency cleanup — never.
- Import (`src/cli/system/import.rs`) — reads receipts, no removal; only
  adjust if plan 001 readers replace private parsing it does today.
- e2e rewrites (plan 007).

## Git workflow

- Shared branch; ONE commit:
  `fix(brew): align prune and uninstall post-state with brew uninstall`
- `git commit -s`; push to `fork`.

## Steps

### Step 1: Cask teardown executes recorded directives

Implement a teardown function that consumes the RECEIPT
(`receipt::CaskReceipt.uninstall_artifacts` for Homebrew/new-format state;
the legacy mise receipt's recorded targets for still-unconverted state):
remove binary/completion links it recorded, execute recorded `uninstall`
directives (pkgutil-forget, launchctl, delete — each only for entries the
receipt records; quarantine: match brew), remove the version dir, remove
the token dir + `.metadata` when it was the last version. NEVER execute
`zap` entries. Unknown directive kinds inside `uninstall_artifacts` →
abort before any deletion with a classified error.

**Verify**: unit tests with fixture receipts (binary cask, app cask,
pkg cask) assert the exact removal set and refusal on unknown directives.

### Step 2: Wire cask prune + reinstall to the teardown

`apply_cask_prune_plan` uses Step 1's teardown per candidate; planner
accepts engine-recognized installed casks of either origin (drop the
"Homebrew owns this cask" skip; keep the unreadable-receipt and
multi-version skips). Reinstall (install over existing version) refreshes
metadata per plan 003 and tears down replaced targets via the same
recorded-facts logic. Dry-run output format unchanged
(`remove brew-cask:<token>@<version>`); failures abort before first
removal (build the full action list, validate, then execute).

**Verify**: `cargo test --all-features system::packages::brew` → exit 0;
updated prune tests cover both origins.

### Step 3: Formula removal parity

Compare `unlink_and_remove_keg` post-state to real brew uninstall
(fixture-driven): opt link, `var/homebrew/linked/<name>`, public prefix
links, keg dir, and the RACK (`Cellar/<name>/`) when empty — plus any
state real brew removes that the current code leaves (verify against a
real-brew uninstall ls-diff captured alongside plan-001 fixtures; commit
that listing as a testdata file). Align exactly; keep the multi-version
behavior (only the candidate version's keg).

**Verify**: existing maintenance tests updated + new parity test against
the captured listing → `cargo test --all-features system::packages::brew`
exit 0.

## Test plan

- Unit: teardown per artifact class (binary/app/pkg/font), unknown
  directive refusal, `.metadata` removed only with last version, zap never
  executed (fixture with zap entries asserts they remain untouched),
  dry-run mutates nothing (hash the tree before/after), abort-before-first
  removal on a poisoned second candidate.
- Pattern: `maintenance.rs` tests 443+ and cask prune tests near the
  planner.
- Verification: `cargo test --all-features system::packages::brew` → all
  pass.

## Done criteria

- [ ] Teardown consumes recorded receipts, never today's catalog.
- [ ] Zap directives are never executed by any removal path
      (test-enforced).
- [ ] Cask prune handles both origins; unreadable receipt → skip/refuse,
      no deletion.
- [ ] Formula removal matches the captured real-brew post-state listing.
- [ ] Dry-run paths byte-identical output to current CLI; no mutation.
- [ ] `cargo test --all-features system::packages::brew` and
      `mise run lint` exit 0.
- [ ] ONE commit; only in-scope files; `plans/README.md` row 006 updated.

## STOP conditions

- A recorded `uninstall` directive kind cannot be executed natively
  without shelling out to `brew` (name it; e.g. `signal`, `script`
  stanzas — report before implementing partial support).
- Real brew's uninstall post-state (fixture listing) removes something the
  engine cannot safely identify from the receipt.
- Parity would require executing a directive on a path outside the
  Homebrew prefix that the receipt does not explicitly record.
- CLI UX (flags/prompts/output) would have to change.

## Maintenance notes

- Any new artifact type added to install (plan 003 note) must extend
  teardown in the same commit — reviewers should reject install-side
  additions without removal-side parity.
- Reviewer focus: the refuse-on-unknown path — silent skips of unknown
  directives would leave residue and pass tests that only count removals.
- Deferred: `brew uninstall --zap` equivalence (no mise surface exists;
  out of scope by design).

## Blocker resolution — 2026-08-12

- **Condition:** a disposable local Homebrew prefix was unavailable for
  the requested uninstall listing, and recorded uninstall metadata can
  contain action kinds that must not be silently skipped.
- **Evidence:** Homebrew's `Keg#unlink` and formula uninstall code define
  the formula post-state; cask receipts contain the installed version's
  `uninstall_artifacts`, including direct artifacts and `uninstall`
  directives. `zap` is a separate artifact and is excluded from plain
  uninstall. The source-derived listing is committed under `testdata/`;
  plan 007 supplies the disposable differential execution.
- **Options:** (1) continue using only legacy mise target receipts —
  rejected because Homebrew-origin state remains unremovable; (2) fetch
  today's catalog — rejected because uninstall facts are version-specific;
  (3) pre-validate and execute the installed receipt's direct targets plus
  native `pkgutil`, `delete`, `quit`, and `launchctl` actions, refusing
  unknown kinds before any removal.
- **Choice:** option 3. Both origins now share the prune path, all
  candidates are validated before the first mutation, metadata is removed
  with the final version, zap is ignored by construction, and formula
  racks are removed only when empty.

## Blocker resolution — 2026-08-12 verification review

- **Condition:** JSON receipts with uninstall flight blocks or relative `delete` paths could enter teardown; direct targets were removed before fallible recorded actions.
- **Evidence:** Homebrew stores the Ruby cask source when flight blocks exist (`cask/installer.rb`), so JSON cannot replay them; receipt paths are destructive authority.
- **Options:** silently skip unsupported facts, fetch today's catalog, or fail closed before mutation and run validated installed-receipt actions before direct targets.
- **Choice:** fail closed for flight blocks and non-absolute/non-normalized delete paths; validate every candidate first; execute recorded actions before direct target removal.

## Blocker resolution — 2026-08-12 PR review

- **Condition:** uninstall pre/postflight hooks were detected only during
  removal, and structured `*_steps` receipt keys were not classified as
  unsupported, allowing install side effects before an inevitable refusal.
- **Evidence:** Homebrew represents these hooks separately from replayable
  uninstall artifacts; native execution would require unsupported script
  semantics or invoking brew.
- **Options:** ignore the hooks; implement a partial interpreter; or reject
  every flight-hook representation during pre-install validation and again
  when consuming installed receipts.
- **Choice:** fail closed at both boundaries. Catalog validation now rejects
  singular and structured pre/postflight keys before mutation; receipt
  teardown independently rejects structured step keys before removal.
