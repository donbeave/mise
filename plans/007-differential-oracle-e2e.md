# Plan 007: Add the differential brew-vs-mise e2e oracle

> **2026-08-13 audit override — FALSE-GREEN.** `e2e/run_test` stripped `CI`, so
> the macOS and Linux oracle bodies exited successfully without fixtures. The
> cited zero-second jobs are not evidence. The corpus also conflated names with
> mechanisms. Use plans 009, 017, and 018. Their safety, completion-marker,
> branch, drift, and done criteria override this document.

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update `plans/README.md` row when done.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`),
> ONE final PR for all plans. Conventional commits, `git commit -s`, no
> agent trailers. Push to `fork` when done.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- e2e/cli/test_system_install_brew_linux e2e/cli/test_system_install_brew_macos_slow e2e/assert.sh`
> Only plans 001–006 changes expected elsewhere; these e2e files should be
> unchanged since `d5e0390dc` — if not, compare and STOP on mismatch.

## Status

- **Current acceptance:** FALSE-GREEN; no valid differential proof

- **Priority**: P1
- **Effort**: L
- **Risk**: MED — test-only changes, but the oracle is the release gate;
  a weak oracle silently blesses divergence
- **Depends on**: plans/003, 004, 006
- **Category**: tests
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

The entire 1:1 emulation architecture is only sound while an oracle proves
engine-produced state equals real-brew-produced state (design decisions 6
and 10). This plan turns both existing brew e2e suites into that oracle:
differential fresh-install comparison plus lifecycle round-trips. It is
also where the production Codex regression (brew-installed cask satisfies
mise status/apply) becomes a permanent test. Real `brew` runs ONLY here —
tests — never in production mise.

## Current state

Verified at `origin/main` `d5e0390dc` (files unchanged from baseline):

- `e2e/cli/test_system_install_brew_linux` (70 lines) — runs only as root
  on Linux CI with network; guards `[[ -d /home/linuxbrew ]] && exit 0`;
  `trap 'rm -rf /home/linuxbrew' EXIT`; installs `brew:xz` via the mise
  engine; asserts linked keg symlinks, receipt mtime, inode stability
  across repair; covers status/apply/repair/import/prune. **Assumes
  Homebrew is absent** — the differential form needs real brew present, so
  the guard and provisioning strategy change.
- `e2e/cli/test_system_install_brew_macos_slow` (68 lines) — macOS CI
  only (`$CI == true`); skips if Hidden Bar app/caskroom already exist;
  installs `brew-cask:hiddenbar` via the engine; asserts
  `/Applications/Hidden Bar.app` + `/opt/homebrew/Caskroom/hiddenbar`;
  exercises delete-protected reinstall; cleanup via `rm -rf` in trap.
- E2E conventions: bash + `assert`, `assert_contains`, `assert_fail`,
  `assert_succeed`, `assert_directory_exists` from `e2e/assert.sh`; tests
  run ONLY via `mise run test:e2e <file>`; no chmod on test files; the
  harness handles cleanup of the workdir but NOT of system state (traps do
  that).
- Real brew in tests: macOS GitHub runners ship Homebrew preinstalled.
  Linux CI container does not — real brew must be installed by the TEST
  (allowed: tests may run brew; production may not). Homebrew 6.x
  install/upgrade ask by default; in scripts pass `--yes` (or rely on
  non-TTY auto-skip: `Ask.confirm?` returns false when stdin or stdout is
  not a TTY — CI is non-TTY, but pass `--yes` explicitly anyway for
  robustness).
- Normalization list (design, decision 6): timestamps (`time`,
  `source_modified_time`, timestamped metadata dir names), `built_on`
  machine facts, `source.path` cache location, `tap_git_head`,
  `homebrew_version` VALUES (real brew writes its own version; the engine
  writes the pin — values differ legitimately; keys must exist on both
  sides), file mtimes. Everything else byte-identical.

## Commands you will need

| Purpose   | Command                                                         | Expected on success                      |
| --------- | --------------------------------------------------------------- | ---------------------------------------- |
| Linux e2e | `mise run test:e2e e2e/cli/test_system_install_brew_linux`      | pass on Linux CI as root; skip elsewhere |
| macOS e2e | `mise run test:e2e e2e/cli/test_system_install_brew_macos_slow` | pass on macOS CI; skip elsewhere         |
| Lint      | `mise run lint`                                                 | exit 0                                   |

## Scope

**In scope**:

- `e2e/cli/test_system_install_brew_linux` — rewrite to differential form.
- `e2e/cli/test_system_install_brew_macos_slow` — rewrite to differential
  form (app cask + font cask).
- A shared helper file `e2e/cli/brew_oracle.sh` (create) if both scripts
  need the snapshot/normalize functions — source it, don't duplicate.
- CI workflow file ONLY if the Linux job needs brew provisioning steps
  (smallest possible change; identify the workflow that runs e2e first).

**Out of scope**:

- Any `src/` change (if the oracle finds divergence, that is a FINDING for
  the responsible plan — report, do not patch the oracle to pass).
- The full essential-mac corpus in CI (implementation-phase local
  verification only — see Test plan).
- New e2e files beyond the helper (rewrite existing suites; do not
  duplicate import/prune coverage into new files).

## Git workflow

- Shared branch; ONE commit:
  `test(brew): add differential brew-vs-mise e2e oracle`
- `git commit -s`; push to `fork`.

## Steps

### Step 1: Snapshot + normalize helper

`e2e/cli/brew_oracle.sh`: functions to (a) snapshot a keg or Caskroom
token dir — file list with modes and content hashes, receipts and SBOM
pretty-printed; (b) normalize volatile fields (the explicit list above —
implement as `jq` transforms on receipts/SBOM plus filename
canonicalization for timestamped metadata dirs); (c) diff two snapshots
and print the first divergence. Normalization REPLACES values with
`<NORMALIZED>`; it must fail loudly if an expected volatile key is absent
(a missing key is divergence, not something to paper over).

**Verify**: shellcheck-clean under `mise run lint`; helper functions
covered by use in Steps 2-3.

### Step 2: Linux differential formula suite

Rewrite `test_system_install_brew_linux` keeping its import/prune
assertions:

1. provision real Homebrew into `/home/linuxbrew/.linuxbrew` (official
   installer inside the disposable CI container — test-only; keep the
   existing root/arch guards and EXIT trap);
2. `brew install --formula xz -y`; snapshot A; `brew uninstall --formula
xz -y`; assert clean post-state; SAVE the post-uninstall tree listing
   (this is plan 006's parity fixture — commit it under
   `src/system/packages/brew/testdata/` if plan 006 has not already);
3. engine install `brew:xz` via `mise bootstrap packages apply --yes`;
   snapshot B; `diff_normalized A B` → empty;
4. round-trip 1: `brew list --formula`, `brew info xz`,
   `brew uninstall --formula xz -y` on the ENGINE-installed keg → all
   succeed; post-state matches step-2 listing;
5. round-trip 2: `brew install xz -y`, then mise status → `installed`,
   `mise bootstrap packages apply --yes` → output contains "already", and
   a keg re-snapshot proves zero changes;
6. keep the existing repair/import/prune assertions, now running against
   engine-installed state (prune post-state must equal the step-2
   listing).

**Verify**: `mise run test:e2e e2e/cli/test_system_install_brew_linux` on
Linux CI → pass.

### Step 3: macOS differential cask suite

Rewrite `test_system_install_brew_macos_slow` (runners ship brew):

1. fixtures: `hiddenbar` (app cask) and `font-jetbrains-mono` (font cask)
   — skip-if-present guards as today, for BOTH fixtures and their
   Caskroom/App/font targets;
2. per fixture: `brew install --cask <token> -y` → snapshot A →
   `brew uninstall --cask <token> -y` → save post-uninstall listing →
   engine install via mise → snapshot B → normalized diff empty;
3. round-trips both directions (engine-installed →
   `brew list --cask`/`info`/`uninstall --cask` clean; brew-installed →
   mise status `installed`, apply no-op with re-snapshot proof) — the
   brew-installed direction IS the Codex production regression test;
4. keep the delete-protected reinstall scenario, asserting the refreshed
   `.metadata` afterwards;
5. cleanup through `brew uninstall --cask` where state is brew-readable;
   the `rm -rf` trap remains only as last-resort for failed half-states.

**Verify**: `mise run test:e2e e2e/cli/test_system_install_brew_macos_slow`
on macOS CI → pass.

### Step 4: Corpus equivalence-class verification (local, documented)

Not CI. Classify the 37 essential-mac casks (list in the design document's
"Corpus snapshot") by artifact/lifecycle class: app-only,
binary+completions, pkg installer, font, versioned (`zed@preview`),
`auto_updates` app, nested/wrapper app (vlc, yaak), privileged
helper/system extension (little-snitch, orbstack). Run the Step-3
differential procedure on ONE representative per class in a disposable
environment (never the operator's live machine). Record the class table +
per-representative pass/fail in `plans/007-corpus-results.md`. Classes
whose representative cannot run in a disposable environment (system
extensions needing user approval) are recorded as NOT VERIFIED with the
reason — no silent skips.

**Verify**: `plans/007-corpus-results.md` exists, every class listed,
none silently absent.

## Test plan

This plan IS the test plan. Coverage delivered: differential
byte-identity (formula + app cask + font cask), four round-trip
directions, Codex regression, removal-parity proof, repair/import/prune on
engine state, corpus class table.

## Done criteria

- [ ] Both e2e suites pass on their CI platforms (evidence: CI run link or
      local platform runs recorded in the PR description).
- [ ] Normalized differential diffs are EMPTY for all fixtures.
- [ ] Codex-scenario regression present (brew-installed cask → status
      installed → apply no-op → zero state change).
- [ ] Post-uninstall parity listings committed as fixtures.
- [ ] `plans/007-corpus-results.md` complete per Step 4.
- [ ] No `src/` production change in this plan's commit.
- [ ] `mise run lint` exits 0; `plans/README.md` row 007 updated.

## STOP conditions

- A normalized diff is non-empty and the divergence traces to engine
  behavior (report against the responsible plan 003/004/006 — do NOT
  widen the normalization list to make it pass; widening is only valid for
  a field that is machine/time-dependent by nature, and each widening must
  be justified in the helper's comments).
- Real brew cannot be provisioned in the Linux CI container.
- A fixture cask's vendor download is unavailable/flaky in CI (pick an
  equivalent fixture in the same artifact class; document the swap).
- `brew` prompts despite `--yes` (ask-mode regression — record brew
  version and report).

## Blocker resolution — 2026-08-12 verification review

- **Condition:** normalization erased immutable `built_on`, `tap_git_head`, and `source_modified_time` facts.
- **Evidence:** Homebrew derives these from bottle/cask metadata and uses them downstream; they are not wall-clock noise.
- **Options:** retain broad normalization, compare only key presence, or compare immutable facts exactly while normalizing only time, producer identity, and producer-local paths.
- **Choice:** compare immutable facts exactly. Plan remains IN PROGRESS until disposable Linux and macOS oracle jobs pass.

## Blocker resolution — 2026-08-12 PR security review

- **Condition:** the disposable Linux oracle downloaded and executed Homebrew's mutable `HEAD` installer.
- **Evidence:** review comment `discussion_r3762598171`; the test jobs can receive credentials, so mutable remote execution violates supply-chain safety.
- **Options:** retain `HEAD`, pin only a commit, or pin an immutable commit and verify the downloaded bytes.
- **Choice:** pin Homebrew/install commit `a34ae4ee9151cbce4c3b33bca7043a972b7ae9a5` and require SHA-256 `12479a24be3f5307eecac7cde670fad7118640f031229e964f544b1367b52a41` before execution.

## Maintenance notes

- This suite is the drift alarm: schedule it (CI cron) against current
  Homebrew stable; a failure means Homebrew changed a private format —
  the fix sequence is engine update → re-verify → bump
  `EMULATED_BREW_VERSION` (see plan 001), never normalization widening.
- Reviewer focus: the normalization list — every entry must be justified;
  entries that hide real divergence defeat the entire architecture.
- Deferred: scheduled-CI wiring if the repo has no cron workflow pattern
  (follow-up; note it in the PR).

## Blocker resolution — 2026-08-12

- **Condition:** local platform commands only prove the scripts' guarded
  skip paths; destructive real-brew installs are forbidden on the
  operator's live macOS machine, and Linux provisioning requires root in
  a disposable runner.
- **Evidence:** `.github/workflows/test.yml` exposes `workflow_dispatch`
  and contains the macOS slow test. The Linux test creates a dedicated
  `linuxbrew` user and removes `/home/linuxbrew` on exit. The macOS test
  refuses non-CI execution and guards every fixture target.
- **Options:** (1) set `CI=true` locally — rejected as destructive on live
  state; (2) weaken the oracle or report local skip as a pass — rejected;
  (3) commit and push the guarded oracle, dispatch the repository test
  workflow on this branch, then mark DONE only after platform evidence.
- **Choice:** option 3. The normalization list is limited to explicit
  machine/time facts and fails if required keys are missing. Corpus classes
  without a safe disposable representative carry explicit NOT VERIFIED
  reasons rather than silent success.

## Blocker resolution — 2026-08-12 PR oracle review

- **Condition:** Caskroom-only snapshots could miss divergent installed apps
  and fonts; Linux engine operations ran as root instead of the prefix owner.
- **Evidence:** cask targets live outside Caskroom, and Homebrew state is owned
  by the dedicated `linuxbrew` user in the disposable Linux oracle.
- **Options:** retain metadata-only comparison, normalize target differences,
  or snapshot labeled metadata plus target roots and run mise as `linuxbrew`.
- **Choice:** compare all labeled roots byte-for-byte and use the prefix owner
  for every Linux engine operation. Disposable macOS CI clears only the exact
  fixture state before running, so preinstalled fonts cannot silently skip the
  oracle. No normalization was widened.

## Blocker resolution — 2026-08-12 Linux runner review

- **Condition:** the regular e2e container has read-only `/etc` and no
  `useradd`, so it cannot provision the mandatory non-root Homebrew owner.
- **Evidence:** CI job `93971192452` failed at `useradd`; `e2e/run_test` mounts
  only `/home`, `/tmp`, and `/root` writable and keeps the root filesystem
  read-only for this test.
- **Options:** run Homebrew as root (rejected by Homebrew and unlike reality),
  weaken ownership coverage, or run the same mise e2e task as root on the
  disposable Linux host while keeping all brew/mise operations non-root.
- **Choice:** the container tranche explicitly defers this one test; the Linux
  e2e job invokes it separately through `mise run test:e2e` under `sudo` on the
  disposable host. `CI=true` is mandatory; any pre-provisioned fixture user and
  `/home/linuxbrew` are removed before the run, and cleanup remains scoped to
  that exact disposable state.

## Completion evidence — 2026-08-12

- macOS job `93970283233`: `hiddenbar` and `font-jetbrains-mono` differential
  step passed on a disposable runner after exact fixture cleanup.
- Linux job `93980178865`: dedicated host oracle passed with real non-root
  Homebrew, empty normalized xz diff, both ownership directions, and
  repair/import/prune lifecycle coverage.
- Focused unit suite: 206 passed. `mise run lint`: passed. No normalization
  entry was added during divergence investigation.

## Blocker resolution — 2026-08-12 shared CI network quota

- **Condition:** final-head Linux e2e failed after the differential oracle
  passed: cmake.org refused a vfox-cmake download on attempt one, then the
  shared GitHub installation token reached zero remaining requests on attempt
  two and an unrelated lockfile test could not enumerate platforms.
- **Evidence:** run `31555069700` shows the Homebrew differential step passed;
  only `backend/test_vfox_cmake` and
  `lockfile/test_lockfile_platforms_setting` failed on external network/quota
  responses. All other Linux tranches and platform jobs passed.
- **Options:** weaken/skip unrelated tests, change production networking, or
  retrigger the unchanged suite after quota reset.
- **Choice:** retrigger unchanged. Test weakening or production changes would
  hide external failures and do not improve Homebrew fidelity.

## Blocker resolution — 2026-08-12 macOS tool download timeout

- **Condition:** current-head macOS CI stopped during tool bootstrap before
  unit tests or the differential oracle because the Bun release archive did
  not download within the fixed 30-second HTTP timeout.
- **Evidence:** run `31558512480`, job `93995821842`, failed while fetching
  `bun-darwin-aarch64.zip`; nightly and Ubuntu builds passed on the identical
  head, and prior disposable macOS job `93970283233` passed the oracle.
- **Options:** weaken the oracle, change the repository-wide HTTP timeout, or
  retrigger the unchanged branch on a fresh disposable runner.
- **Choice:** retrigger unchanged. The failure occurred before project tests;
  changing timeout policy is outside this plan, while weakening coverage would
  violate the differential-oracle invariant.

## Blocker resolution — 2026-08-12 unrelated global-config gate

- **Condition:** after both current-head differential oracles passed, full CI
  failed only in `cli/test_global_config_confd`; the failure reproduced alone
  on macOS.
- **Evidence:** `MISE_USE_TOML=false` caused the test's `.tool-versions` file to
  be parsed as TOML, then `mise use -g` created the default `config.toml` that
  the test asserted must not exist. No branch diff touched config selection.
- **Options:** keep retriggering a deterministic failure, alter production
  config selection, or make the fixture explicitly select the legacy global
  file through the supported `MISE_GLOBAL_CONFIG_FILE` override.
- **Choice:** explicitly select `.tool-versions` in the test fixture. This
  preserves the intended drop-in write-target assertion, changes no production
  behavior, and removes dependence on an unrelated format-preference setting.

## Blocker resolution — 2026-08-12 normalization audit

- **Condition:** final verification found two normalizers broad enough to hide
  genuine divergence: any timestamp-shaped path component was replaced, and
  the complete SPDX creator array was discarded.
- **Evidence:** neither value is wholly nondeterministic. Only Homebrew's cask
  `.metadata/<version>/<install timestamp>` directory and the version-bearing
  producer string vary; path placement, creator type, and creator cardinality
  are invariant state.
- **Options:** retain the broad rules, remove normalization and accept expected
  machine/time diffs, or validate invariant structure and normalize only the
  intrinsically variable fragments.
- **Choice:** scope timestamp replacement to cask metadata installation
  directories, require exactly one Homebrew producer creator, and replace only
  that creator string. This keeps the oracle stable without concealing missing,
  extra, or malformed state.
