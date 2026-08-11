# Make the mise brew/brew-cask engine one-to-one with Homebrew

> **Executor instructions:** Read this plan completely before editing code.
> The internal brew/brew-cask engine is retained and must produce on-disk
> state byte-identical to what real Homebrew produces. Production mise never
> executes the `brew` CLI. Real `brew` runs only inside tests as the
> differential oracle. If a STOP condition occurs, report it instead of
> inventing a fallback.
>
> **Rebase first:** implementation starts from a branch rebased onto current
> `origin/main` (which now includes cask prune #11810, font casks, platform
> filters). After rebasing, re-verify every current-state excerpt below —
> they were captured at baseline `14888188d` — and run:
>
> ```sh
> rtk git diff --stat 14888188d..HEAD -- \
>   src/system/packages/brew src/system/packages/mod.rs \
>   src/cli/system docs/bootstrap/packages/brew.md \
>   e2e/cli/test_system_install_brew_linux \
>   e2e/cli/test_system_install_brew_macos_slow
> ```
>
> Stop if an in-scope symbol no longer matches what this plan describes.

## Status

- **Status:** design APPROVED by maintainer 2026-08-12 after question-driven
  review; ready for implementation on explicit instruction
- **Priority:** P1
- **Effort:** XL
- **Risk:** HIGH — engine must reproduce Homebrew's private on-disk state
  exactly; emulation errors can surface as destructive `brew
upgrade`/`uninstall` misbehavior
- **Depends on:** none
- **Category:** correctness / compatibility / architecture
- **Planned at:** commit `14888188d`, 2026-08-05; direction revised
  2026-08-12
- **mise branch:** `agent/brew-cask-native-interop-plan`
- **Homebrew evidence verified at:** local `6.0.17-4-gb94d9e9` (source
  `b94d9e9f94`); emulation pin at implementation start: `6.0.17`

## Design review progress

This section is the durable record of the maintainer review. Preserve both
accepted decisions and unresolved questions so implementation does not silently
replace product policy with executor assumptions.

### Review session: 2026-08-12

#### Triggering production evidence

An `essential-mac` AI bootstrap using mise `2026.8.4` failed on an existing
Homebrew-installed Codex cask:

```text
brew-cask:codex
mise ERROR brew-cask:codex: Homebrew owns this cask; remove it with Homebrew before installing it with mise
```

The machine had a valid Homebrew cask receipt at
`/opt/homebrew/Caskroom/codex/.metadata/INSTALL_RECEIPT.json`; `brew info --cask
codex` reported version `0.147.0` installed on request. The downstream config
correctly declared `brew-cask:codex`, but current mise status ignored Homebrew
metadata, classified the cask as missing, then its install path rejected the
same metadata as conflicting ownership.

This proves the problem is not Codex-specific. Any valid Homebrew-owned cask can
hit the same status/install contradiction when added to mise bootstrap state.

#### Direction accepted

> **Superseded 2026-08-12 later in this session:** the maintainer
> subsequently revised the direction to keep the internal engine with full
> 1:1 write emulation — see "DECIDED 2026-08-12: keep engine, full 1:1
> write emulation". This subsection stays as the historical record of the
> interim delegation acceptance.

The maintainer accepted the plan's single-writer direction for further design
review:

- Homebrew is the sole formula/cask state writer.
- mise retains `brew:` and `brew-cask:` as declarative orchestration namespaces.
- Existing valid Homebrew installations satisfy mise status and apply without
  mutation.
- Fresh mise apply delegates installation to canonical Homebrew.
- No token-specific Codex workaround, synthetic Homebrew metadata, or generic
  uninstall/reinstall migration belongs in the core fix.
- The review continues as a question-driven design process. Decisions and new
  evidence must be recorded in this section before implementation begins.

#### Live contract verification

Current local Homebrew `6.0.17` was checked against the proposed query contract:

- `HOMEBREW_NO_AUTO_UPDATE=1 brew info --json=v2 --installed` returned one
  object containing separate `formulae` and `casks` arrays.
- The live inventory contained 108 formulae and 10 casks.
- Codex appeared in `.casks` with token `codex` and opaque installed version
  `0.147.0`.
- Therefore the proposed authoritative inventory query can recognize the exact
  state that current mise rejects.

The plan branch remains at its planned baseline `14888188d`. At the time this
paragraph was first written, the scoped source files had no diff against
`origin/main`. **Superseded 2026-08-12 (restart verification): `origin/main`
has since diverged in the scoped files — see "Verification refresh:
2026-08-12 (restart session)" below.** The plan file itself is intentionally
uncommitted while this review evolves.

#### Required correction: noninteractive Homebrew mutations

> **Moot as of the 2026-08-12 architecture decision:** production mise never
> invokes the `brew` CLI, so no delegated mutation can prompt. Retained as
> evidence; the ask-mode facts remain relevant to test harnesses that drive
> real `brew` (pass `--yes` or run without a TTY there).

Homebrew `6.0.17` makes ask mode the default for `install` and `upgrade` and
supports `--yes` / `--no-ask`. The observed bootstrap already paused at:

```text
==> Do you want to proceed with the upgrade? [y/n]
```

The current plan's mutation argv omits `--yes`. Delegation must not add a second
interactive confirmation after mise's package driver has already performed its
own confirmation policy. The final design must define when the Homebrew adapter
passes `--yes`, update exact-argv tests, and prove unattended `mise bootstrap
--yes` cannot block on Homebrew's ask prompt.

#### Required correction: legacy mise-only cask rollout

> **Resolved 2026-08-12 by open-decision 2's gated backfill** (provable
> truth → convert to full receipts; otherwise needs-repair with a one-line
> instruction). Retained as the record of the rollout requirements that the
> backfill design answers.

The safe core behavior remains: a `.mise-cask.toml` installation without valid
Homebrew metadata is not silently adopted, deleted, reinstalled, or represented
with fabricated metadata. It returns `NeedsRepair` before mutation.

This is safe but not yet an operational rollout plan. Existing downstream
machines may contain many mise-only casks. Before implementation is considered
ready, this plan must define:

- how status inventories every affected cask in one run;
- the exact actionable recovery guidance shown to operators;
- whether any narrowly proven `brew install --cask --adopt` eligibility class is
  allowed, and under whose explicit confirmation;
- how disposable tests prove target identity and rollback behavior;
- the rollout gate preventing a mise upgrade from turning a previously
  converged bootstrap into a sequence of unexplained `NeedsRepair` failures.

Universal automatic migration remains rejected unless Homebrew publishes a
supported registration/import transaction.

#### Verification refresh: 2026-08-12 (restart session)

Repository facts re-verified from a fresh session before continuing the
interview:

- Branch `agent/brew-cask-native-interop-plan` is at baseline `14888188d`. The
  only uncommitted change is this plan file. The drift-check diff
  `14888188d..HEAD` over the scoped paths is empty.
- `origin/main` has moved to `d5e0390dc` and **now diverges from the baseline
  in 14 scoped files** (`git diff origin/main HEAD -- <scoped paths>`: 113
  insertions, 1645 deletions relative to main — i.e. main gained ~1.6k lines).
  Relevant main commits since baseline: `e8163e022` platform filters for
  packages (#11809), `73f11c627` **prune mise-owned brew casks (#11810)**,
  `b7f38fe98` font artifacts from git URLs (#11781), `c637392a4` font casks on
  Linux (#11758), `881c7daa2` extensionless DMG detection (#11692),
  `70fb487b2`/`f24659ab7` relocation fixes, `5415017d0` flatpak packages
  (#11757), `dc0e012d0` jobs fix, `73e9a53da` cask cookbook docs.
- Consequences: (a) the plan's current-state excerpts and line numbers
  describe baseline `14888188d` only; before implementation the branch must be
  rebased onto main and every excerpt re-verified (existing known correction,
  now with concrete evidence); (b) `cask.rs` on main is now 8,296 lines and
  ships a **cask prune** path, so the "Cask prune/import … do not exist today"
  out-of-scope claim is stale — see new open decision 5; (c) the inbound bug
  is still present on main: the ownership guard (`cask.rs:290`, `cask.rs:332`
  "Homebrew owns this cask"), receipt-based `installed_cask_version`, and
  `.mise-cask.toml` writes all persist, and main's new cask prune also skips
  Homebrew-owned casks via `homebrew_metadata_present` (`cask.rs:4428`) while
  still deleting mise-owned cask state directly.
- mise confirmation policy verified at baseline: `src/cli/system/driver.rs:136-138`
  prompts before mutation unless `--dry-run`, `--yes`, or stderr is
  unattended; `src/cli/system/prune.rs:82` and multiple `install.rs` sites use
  the same `prompt::confirm`. So every delegated Homebrew mutation is already
  behind mise's own confirmation policy.

Homebrew evidence refreshed from local `6.0.17-4-gb94d9e9` (source checkout at
`b94d9e9f94`, newer than the plan's recorded `6.0.15` baseline):

- `install`, `reinstall`, and `upgrade` accept `-y, --no-ask, --yes`; ask mode
  is the default. `uninstall` has no ask mode.
- `HOMEBREW_NO_ASK` env: "If set, do not enable default ask mode. This does
  not disable an explicit `--ask`." The `--no-ask` switch is env-bound
  (`env: :no_ask`, `cmd/install.rb:51`).
- `Homebrew::Ask.confirm?` (`Library/Homebrew/ask.rb:13`) returns immediately
  without prompting when stdin **or** stdout is not a TTY. So non-TTY CI never
  blocks; the observed production hang happened because mise ran `brew` with
  an inherited interactive terminal.
- `ask_prompt_needed?` (`install.rb:689-708`) prompts only when the plan
  includes packages beyond the named requests, or when dependencies /
  upgradeable dependants are present; otherwise ask mode only prints the plan.
  A single dependency-free install may not prompt — but any dependency does,
  so delegation cannot rely on this.
- Version availability: Homebrew `5.1.15` does **not** recognize
  `--yes`/`--no-ask` argv (only opt-in `--ask`); passing `--yes` to a 5.x brew
  is an unknown-option error. `HOMEBREW_NO_ASK` **does** exist in `5.1.15`
  ("do not enable ask mode from `$HOMEBREW_ASK` or the `$HOMEBREW_DEVELOPER`
  default"), so the env var is honored on 5.x and 6.x, and unknown
  `HOMEBREW_*` vars are ignored harmlessly by even older versions.
- In 6.x, `--ask` is deprecated and env-bound to `HOMEBREW_ASK`;
  `conflicts "--ask", "--no-ask"` is declared. Whether a user-exported
  `HOMEBREW_ASK=1` plus a mise-passed `--yes` argv triggers a CLI conflict
  error is **unverified**; if the argv route is chosen this must be tested, or
  the adapter must scrub `HOMEBREW_ASK` from the child environment.

#### Clarification requested by maintainer: how the providers work today (2026-08-12)

Before answering the ask-mode question, the maintainer asked whether current
mise calls Homebrew under the hood or is a fully internal brew-compatible
engine. Verified answer from baseline source (unchanged in substance on
current main):

- **Formula provider is fully native.** `src/system/packages/brew/mod.rs:1-18`
  states "mise never shells out to brew." Metadata comes from formulae.brew.sh
  (`api.rs`, `resolve.rs`), bottles from ghcr.io (`fetch.rs`, `tag.rs`), and
  `pour.rs` extracts, relocates (`relocate.rs`, `macho.rs`, `elf.rs`), links
  kegs, and writes a brew-shaped `INSTALL_RECEIPT.json` claiming
  `"homebrew_version": "5.1.15 (mise)"`. Formulae without bottles are built
  from source through a mise-managed Ruby evaluating mise's own Formula-DSL
  shim (`source.rs`, `shim.rb`). Installed-state checks read mise's own
  opt/link records, not any Homebrew query.
- **Cask provider is likewise native.** `cask.rs` (241 KB) downloads
  artifacts itself, handles DMG/zip/pkg via `ditto`/`xattr`/`pkgutil`, runs
  cask lifecycle hooks by executing mise's `cask_shim.rb` against the
  fetched `cask.rb` with a discovered Ruby, writes its own `.mise-cask.toml`
  receipt, and refuses to install when Homebrew `.metadata` exists.
- **Exactly one external `brew` invocation exists in both providers:**
  `cask_ruby_bin()` (`cask.rs:780-784`) runs `brew ruby -e 'print
RbConfig.ruby'` from PATH solely to locate a Ruby interpreter for the shim,
  falling back to PATH `ruby`, then mise-provisioned Ruby. No package query,
  install, upgrade, link, or removal ever calls the `brew` executable.

Conclusion recorded: current mise is a parallel brew-compatible installer
engine, not an orchestrator. That dual-writer architecture is the enabling
condition for the observed status/install contradiction, which is why the
accepted direction replaces the engines with delegation instead of patching
the guard.

#### Maintainer direction revision requested: keep the internal engine (2026-08-12)

When asked the ask-mode suppression question, the maintainer instead revised
the direction under review. Recorded statement (paraphrased from interview):

- Keeping mise's **full internal engine is critical**; do not replace it with
  delegation to the `brew` executable.
- The goal is that `mise` package/cask install and `brew install` are
  **one-to-one interchangeable**: installing with either tool must produce the
  same result, and mixing the two tools on the same machine must work.
- The maintainer views `upgrade` as secondary; the **install process** is
  where the incompatibility lives and is the focus.
- The observed production failure is precisely this incompatibility: mise
  install and brew install currently cannot be mixed.

Status of this revision: it **contradicts the previously accepted
single-writer delegation direction** (Executive decision, Required behavior,
Steps 1–7). Those sections are now under re-review and must not be
implemented or rewritten until the architecture decision below is answered.
Open decision 1 (Homebrew ask-mode suppression) is **parked**: it only exists
if some delegation to the `brew` executable remains.

Evidence assembled for the architecture decision (discoverable facts, not
policy):

- Write-side cask emulation was already attempted and retired in repository
  history, within ~34 hours in July 2026: `bd2fe92bd` (2026-07-22, "write
  Homebrew .metadata so brew list/upgrade work"), `99e2e50a5` ("backfill
  metadata receipts"), `a47633fc2` ("formula-style brew identity without
  status lie"), then `b91389ddd` (2026-07-23, "retire synthetic metadata and
  harden pours").
- The ADR introduced by `b91389ddd`
  (`docs/dev/brew-cask-decision-record.md`, since removed from the tree)
  concluded: "Homebrew's installed cask marker (`.metadata` + tab) is
  **lifecycle authority**, not identity. Writing synthetic empty-tab metadata
  made mise claim authority it could not describe and allowed Homebrew to
  recover teardown from the live API." Its support matrix intentionally
  declared MiseOwned casks "brew list/upgrade: unsupported".
- What failed in July was an **empty/synthetic** tab. A truthful complete
  receipt would need Homebrew's private `Tab` schema (formula and cask),
  `.metadata` layout, and the adjacent `sbom.spdx.json`; Homebrew treats
  these as internal (`Cask::Caskroom` internal; cask metadata moved to JSON
  in July 2026 with the top-level receipt authoritative). Errors in an
  emulated receipt surface later as **destructive misbehavior** when real
  `brew upgrade`/`uninstall` consumes them.
- The formula side is different today: mise pours write brew-shaped keg
  receipts that current Homebrew mechanically accepts (`mod.rs` documents
  this), while still falsifying `"homebrew_version": "5.1.15 (mise)"`. Their
  universal acceptance by `brew upgrade`/`uninstall` is unproven
  (integration-corpus item), and the format remains private.
- Read-side recognition (mise treating valid Homebrew-installed state as
  installed) is achievable through the versioned public CLI
  (`brew info --json=v2 --installed`) regardless of which write-side
  architecture is chosen; the observed bootstrap failure needs at minimum
  this read-side fix.

#### DECIDED 2026-08-12: keep engine, full 1:1 write emulation

**Decision (maintainer):** mise keeps its full internal install engine for
both formulae and casks. Instead of delegating to the `brew` executable, the
engine must produce state **one-to-one identical with what `brew install`
produces**, so that mise-installed and brew-installed packages are fully
interchangeable: either tool may install, list, upgrade, or uninstall a
package regardless of which tool installed it. The install path is the focus;
upgrade is secondary.

**Rationale:** maintainer policy — the internal engine is critical to mise's
brew/brew-cask providers; the incompatibility between mise install and brew
install is the actual bug to fix, not the existence of the engine.

**Rejected alternatives (2026-08-12):**

- Full delegation to the `brew` executable (this plan's previous executive
  decision) — rejected because it removes the internal engine.
- Split architecture (formulae native, casks delegated) — rejected for the
  same reason on the cask side.
- Read-side-only recognition — rejected because mise-installed casks would
  remain invisible to `brew list`/`upgrade`, failing the mixing goal.

**Consequences accepted with this decision:**

- Overturns the July 2026 ADR (`b91389ddd`) that retired synthetic cask
  metadata, and supersedes this plan's delegation-based Executive decision,
  Required behavior, Homebrew contract, Steps 1–7, Done criteria, and several
  Rejected approaches. Those sections are pending full reconciliation after
  the interview completes; until then they carry a superseded banner and must
  not be executed.
- mise commits to writing **truthful, complete** Homebrew receipts (formula
  `Tab`/`INSTALL_RECEIPT.json`, cask `.metadata` + tab, and whatever adjacent
  files brew requires, e.g. `sbom.spdx.json`) in Homebrew's private,
  unversioned formats, and to tracking upstream format changes release by
  release.
- The July failure mode (synthetic **empty** tab claiming authority without
  lifecycle facts) is not re-adopted: the new bar is a receipt carrying the
  same lifecycle facts brew itself would have written. Emulation errors can
  surface as destructive `brew upgrade`/`uninstall` misbehavior, so the
  design must include a compatibility oracle and drift detection before any
  release.
- The current false `"homebrew_version": "5.1.15 (mise)"` receipt field is
  incompatible with the truthfulness bar and must be resolved by a later
  decision in this interview.

**New open questions raised by this decision** are appended to the list
below (compatibility oracle, receipt truth spec, read-side recognition
mechanism, ownership-guard removal semantics, legacy `.mise-cask.toml`
backfill, Homebrew version-drift policy, cask prune alignment, uninstall
directive fidelity).

#### DECIDED 2026-08-12: verification method and acceptance corpus

**Maintainer answers (question 6, plus a reinforcing direction statement):**

1. Verification must be **consistent with the repository's existing
   verification conventions** — do not introduce a foreign testing style. If
   the repo verifies via integration tests, use integration tests; if via
   fixtures, use fixtures.
2. The best proof is an **integration differential comparison of two
   outputs**: freshly install a package with real `brew`, freshly install the
   same package with the mise engine, and compare the results. This is
   critical. Real brew and real mise bootstrap may be used during the
   implementation phase for verification.
3. Reinforced direction: production mise **never calls the `brew` CLI** — the
   internal brew/brew-cask engine performs everything, and it must behave
   one-to-one with the original Homebrew installation process for every
   package and cask. (Real `brew` remains permitted inside tests as the
   differential oracle.)
4. **Acceptance corpus:** `https://github.com/donbeave/essential-mac` (local
   checkout `/Users/donbeave/Projects/donbeave/essential-mac`). Every `brew:`
   and `brew-cask:` declaration in that repository must work correctly and
   one-to-one whether installed via `brew install` or via mise bootstrap.

**Verified current repo verification conventions (discoverable facts):**

- E2E: bash scripts under `e2e/cli/` with `assert*` helpers.
  `test_system_install_brew_linux` (70 lines) runs only as root on Linux CI,
  requires network, installs real `brew:xz` through the mise engine into
  `/home/linuxbrew/.linuxbrew`, asserts on real filesystem state (linked keg
  symlinks, receipt mtime, inodes), and covers status/apply/repair/import/
  prune. `test_system_install_brew_macos_slow` (68 lines) runs only on macOS
  CI, installs the real `hiddenbar` cask via the engine, asserts on
  `/Applications` and Caskroom paths, and exercises reinstall/permission
  recovery. No existing test compares against real-brew-produced state; real
  `brew` is currently never executed in tests.
- Unit tests: fake-prefix fixtures via `MISE_SYSTEM_BREW_PREFIX` (used in
  `pour.rs`, `cask.rs`, `maintenance.rs`, `prefix.rs`).

**Decision recorded for question 6:** the 1:1 oracle is a differential
integration suite in the existing e2e style: on a disposable CI host with
real Homebrew present, install fixture packages once with real `brew` and
once with the mise engine (fresh state each side), then compare the resulting
keg/Caskroom trees and receipts (volatile fields normalized) and prove
lifecycle round-trips (mise-installed → `brew list/info/upgrade/uninstall`
works; brew-installed → mise status reports installed and apply is a no-op).
The essential-mac corpus defines the compatibility bar for release
("every package, every cask from this repository").

**Corpus snapshot (2026-08-12, 31 formulae / 37 casks):**

- Formulae: agent-browser, ansible, bash, bat, buf, dockutil, gh, git-lfs,
  git-standup, gogcli, gomplate, gradle, helm, just, kimi-code, kotlin,
  kubectx, kubernetes-cli, linode-cli, mas, maven, mise, msedit, nushell,
  opencode, opentofu, rtk, starship, watch, wget, xh.
- Casks: 1password, 1password-cli, bartender, chatgpt, claude, claude-code,
  cleanshot, cloudflare-warp, codex, codexbar, font-jetbrains-mono,
  font-jetbrains-mono-nerd-font, ghostty, google-chrome, grammarly-desktop,
  grok-build, handbrake-app, jetbrains-toolbox, kimi, little-snitch, notion,
  opencode-desktop, orbstack, plex-media-server, sketch, speechify-voice-ai,
  sublime-text, superwhisper, surge, tableplus, tor-browser, transmission,
  visualvm, vlc, yaak, zed@preview, zoom.
- Hard compatibility classes inside the corpus: pkg-installer casks
  (cloudflare-warp, little-snitch, zoom), system-extension/privileged-helper
  apps (little-snitch, orbstack), fonts, a versioned cask (`zed@preview`),
  auto-updating apps (1password, google-chrome), and the nested/wrapper cases
  the corpus itself annotates (vlc, yaak — jdx/mise#11164).
- essential-mac's own convention notes (mise.toml:27-28) currently prefer the
  mise engine when API metadata exists and "real Homebrew CLI only for
  third-party taps without API metadata".

**Sub-question resolved 2026-08-12 — oracle cadence:** follow how mise
itself already verifies the brew/brew-cask implementations: the committed CI
gate stays in the existing convention (small-fixture e2e in `e2e/cli/`,
upgraded to differential brew-vs-mise comparisons plus round-trips). The
full essential-mac corpus is an **implementation-phase local verification**,
and it is deduplicated by mechanism: classify the 37 casks by
artifact/lifecycle class and test one representative per class — "no point
to test them all if all of them are using the same approach; we only test
those which are different to each other" (maintainer). Implementation task
recorded: derive the equivalence classes from each cask's artifact stanza
set (app-only, binary+completions, pkg installer, font, versioned
`@preview`, `auto_updates` app, nested/wrapper app, privileged
helper/system extension) and enumerate representatives before the
differential pass.

### Open decisions

0. ~~Architecture~~ — **decided 2026-08-12**: keep engine + full 1:1 write
   emulation (see decision record above).
1. _(Closed as moot 2026-08-12 — no delegated mutations remain in the chosen
   architecture.)_ Homebrew ask-mode suppression for delegated mutations.
2. ~~Legacy mise-only casks~~ — **decided 2026-08-12: gated backfill on
   apply.** A legacy cask (`.mise-cask.toml`, no `.metadata`) is converted
   to full truthful Homebrew receipts during status/apply only when truth is
   provable: the mise receipt exists, artifact fingerprints still match the
   installed payload, and the cask definition for the installed version is
   obtainable (installed version equals current catalog version — uninstall
   directives are version-specific). On success the engine writes the
   complete `.metadata` receipt set and removes `.mise-cask.toml`. If any
   condition fails (auto-updated app, fingerprint mismatch), the cask
   reports needs-repair with a one-line reinstall instruction; nothing is
   mutated. Distinction from the retired July backfill (`99e2e50a5`): that
   wrote empty synthetic tabs; this writes complete truthful receipts and
   refuses when truth cannot be established. Formula side needs no
   migration: existing mise kegs already carry brew-shaped receipts that
   real Homebrew accepts; they are left untouched. Rejected: unconditional
   backfill (version-mismatched destructive directives — July failure
   class), manual-repair-only (turns every existing bootstrap machine into
   repair errors after upgrade).
3. _(Closed as moot 2026-08-12 — `--adopt` was a delegation-path tool; under
   emulation the equivalent problem is receipt backfill, question 2.)_
4. _(Closed as moot 2026-08-12 — native writers are retained by decision 0.)_
5. ~~Prune/removal alignment~~ — **decided 2026-08-12: removal is in
   scope.** Formula prune, cask prune (#11810, arriving via the rebase onto
   main), and the cask uninstall/reinstall paths must all satisfy the same
   differential bar: mise removal leaves post-state identical to
   `brew uninstall`, including execution of the cask's recorded uninstall
   directives (launchctl/pkgutil/binary/completions) and `.metadata`
   teardown. Round-trip oracle tests depend on this. Rejected: install-only
   scope (mixed machines would accumulate divergent residue and round-trip
   verification could not pass).
6. ~~Compatibility oracle~~ — **decided 2026-08-12**: differential
   integration suite in existing e2e style plus lifecycle round-trips;
   essential-mac is the acceptance corpus (see decision record above).
   Remaining sub-question: CI cadence/placement of the full-corpus run.
7. ~~Receipt truth specification~~ — **decided 2026-08-12 (producer-identity
   policy):** every producer-identity field (`homebrew_version` in formula
   and cask receipts, the SBOM creator string) is pinned to the exact
   Homebrew version the engine currently emulates and is differential-CI
   verified against (today `6.0.17`), with no mise marker anywhere; the pin
   is bumped only when the differential oracle re-verifies against a newer
   Homebrew. The current falsified `"5.1.15 (mise)"` is removed. Supporting
   facts: brew semantically version-compares this field
   (`extend/os/linux/bottle_specification.rb:10` gates relocation on
   `parsed_homebrew_version >= "5.1.15"`); the SBOM creator embeds the same
   version (`sbom.rb:191`). Rejected: a `"(mise)"` provenance suffix
   (breaks byte-level 1:1 and the indistinguishability goal); dynamically
   mirroring brew's latest release (claims unvalidated behavior).
   Completeness bar implied by decision 0: all fields real brew writes —
   formula tab (arch, built_on, compiler, changed_files,
   runtime_dependencies, source with tap_git_head, timestamps),
   `sbom.spdx.json`, cask `.metadata` receipt including truthful
   `uninstall_artifacts`/zap directives derived from the cask DSL,
   `config.json`, and the timestamped cask-definition snapshot
   (`.metadata/<version>/<timestamp>/Casks/<token>.json`) — verified
   structures sampled 2026-08-12 from real Homebrew 6.0.x installs
   (`ada-url` keg, `codex` Caskroom).
8. ~~Read-side recognition~~ — **decided 2026-08-12: strict zero brew CLI in
   production.** mise never executes the `brew` CLI for anything: status
   parses Cellar/Caskroom on-disk state natively (formula receipts/opt links
   already native; reading cask `.metadata` is new work), all writes are 1:1
   native, and the sole existing call site (`cask_ruby_bin`'s `brew ruby`,
   `cask.rs:780-784`) is replaced with mise's own Ruby discovery. Real
   `brew` is executed only inside tests as the differential oracle.
   Consequence: the read side also depends on private on-disk formats, so
   the drift policy (decision 10) must cover read parsing as well as
   writing. Rejected alternatives: read-only `brew info --json=v2` when brew
   is present (two read paths, presence-dependent behavior); keeping the
   `brew ruby` exception (inconsistent with the rule).
9. ~~Ownership guard semantics~~ — **decided 2026-08-12: brew-equivalent
   semantics.** The "Homebrew owns this cask" guard (`cask.rs:290`/`332`)
   and the mise-receipt-only status check are deleted; one state model
   serves both origins because the on-disk state is identical. Status: any
   valid installed state of the requested kind reports installed; `latest`
   declarations are satisfied by any installed version (apply remains
   presence reconciliation). Apply/install over an installed package is a
   no-op mirroring brew's "already installed". Upgrade reproduces brew
   upgrade semantics, including skipping `auto_updates` casks (no implicit
   `--greedy`). Partial/corrupt state (e.g. Caskroom version directory
   without a valid receipt) reports needs-repair and is never silently
   mutated. This is the direct fix for the Codex bootstrap failure.
   Rejected: origin tracking (reintroduces a second metadata source — the
   original bug's enabling condition); keeping the guard until emulation
   ships (leaves production broken meanwhile).
10. ~~Homebrew drift policy~~ — **decided 2026-08-12: schema-strict,
    version-tolerant.** The engine parses Homebrew on-disk state strictly
    against the structures it knows. A newer `homebrew_version` string alone
    is never an error (user machines auto-update brew; a version lock would
    break bootstrap within days of each brew release). Unknown or missing
    required structure, or an unparseable receipt, is an error and blocks
    mutation of that package — clean absence stays distinguishable from
    corruption/unknown. Drift detection is the scheduled differential CI
    run against current Homebrew stable; a failing oracle is a drift alarm,
    the emulation pin is bumped only after re-verification, and no release
    ships with a failing oracle. Rejected: hard version lock (routine
    breakage), best-effort continue (destructive-accident class; violates
    the never-mutate-unknown invariant).

### Restart handoff prompt

Copy this prompt into a fresh agent started from
`/Users/donbeave/Projects/donbeave/mise`:

```text
Continue work on:

plans/brew-cask-native-homebrew-interoperability.md

State as of 2026-08-12: the maintainer design interview is COMPLETE and the
plan has been reconciled around the decided architecture. Do not reopen
settled decisions unless the maintainer asks. Read the plan completely —
especially "Design review progress" (decisions 0-10) and the normative
sections — then read repository AGENTS.md.

Decided architecture (controlling):

- Keep mise's internal brew/brew-cask engine; production mise never executes
  the brew CLI (the cask.rs `brew ruby` lookup is removed too).
- The engine reads and writes Homebrew's real on-disk state 1:1: truthful
  complete receipts (formula tab + SBOM, cask .metadata set), pinned
  producer version (initially 6.0.17), brew-equivalent status/apply/
  upgrade/removal semantics.
- Schema-strict version-tolerant reads; gated provable-truth backfill for
  legacy mise-only casks; removal parity in scope; differential
  brew-vs-mise e2e oracle in the existing e2e conventions; essential-mac
  corpus (equivalence-class reduced) is the implementation-phase
  verification bar.

Current state: plan file is the only uncommitted change on branch
agent/brew-cask-native-interop-plan (baseline 14888188d). origin/main has
diverged (cask prune #11810, font casks, platform filters) — implementation
must rebase first and re-verify excerpts.

Next actions in order:

1. If the maintainer has not yet approved the reconciled plan, present the
   decision summary and ask for approval; store the answer in the plan.
2. On approval: commit the plan file (docs/plan commit, conventional
   message, git commit -s).
3. Implementation only when explicitly instructed, following the plan's
   step/commit sequence and STOP conditions.

Never mutate the operator's real installed apps during any verification.
```

## Executive decision

The mise-internal brew/brew-cask engine is retained as the installer for
`brew:` and `brew-cask:` bootstrap packages, and it is upgraded so the state
it reads and writes is **one-to-one identical** with what real Homebrew
produces. There is one installation format because both installers emit the
same format, verified continuously against real Homebrew.

Decided contract (maintainer interview, 2026-08-12):

1. **Zero brew CLI in production.** mise never executes the `brew`
   executable — not for mutations, not for status queries, not for helper
   discovery (the current `brew ruby` call is removed). Real `brew` runs
   only inside tests as the differential oracle.
2. **1:1 write emulation.** Engine installs/upgrades/removals leave state
   byte-identical (after normalizing volatile fields) to real Homebrew:
   Cellar kegs with complete truthful `INSTALL_RECEIPT.json` and
   `sbom.spdx.json`, linked-keg/opt state, Caskroom payloads with the full
   `.metadata` receipt set (top-level `INSTALL_RECEIPT.json` including
   truthful `uninstall_artifacts`, `config.json`, timestamped cask-definition
   snapshot).
3. **Producer identity is the emulation pin.** Every producer-identity field
   (`homebrew_version`, SBOM creator) carries the exact Homebrew version the
   engine is differential-CI verified against (initially `6.0.17`), bumped
   only after re-verification. No mise marker anywhere; the falsified
   `"5.1.15 (mise)"` is removed.
4. **Brew-equivalent semantics.** Any valid installed state of the requested
   kind satisfies status regardless of which tool wrote it; `latest`
   declarations are satisfied by any installed version; apply is presence
   reconciliation; install over installed is a no-op; upgrade mirrors brew
   upgrade semantics including skipping `auto_updates` casks; removal
   executes the recorded uninstall directives and metadata teardown exactly
   as `brew uninstall` would.
5. **Schema-strict, version-tolerant reads.** A newer `homebrew_version` in
   on-disk state is never an error by itself; unknown or corrupt structure
   blocks mutation of that package with a clear error.
6. **Mixing is the acceptance bar.** Every `brew:`/`brew-cask:` declaration
   in the essential-mac corpus must work correctly whether installed by
   `brew install` or by the mise engine, interchangeably, in both
   directions, including uninstall.

No new mise command, subcommand, flag, mode, ownership selector, or
configuration key is added. Existing `status`, `apply`, `upgrade`, `import`,
`prune`, and `[bootstrap.brew.taps]` surfaces are sufficient.

## Direction lineage

This plan has carried three directions; the maintainer decision of
2026-08-12 is controlling:

1. **v1 — native engine with mise-private ownership** (July 2026, ADR
   `b91389ddd`): engine wrote mise-only receipts; Homebrew-owned casks were
   rejected; `brew list/upgrade` on mise-installed casks was declared
   unsupported. This produced the observed production contradiction — a
   valid Homebrew cask reported missing by status and rejected by install.
2. **v2 — full delegation to the canonical `brew` executable** (this plan's
   earlier executive decision): rejected by the maintainer on 2026-08-12
   because the internal engine is a critical mise capability.
3. **v3 — native engine with full 1:1 Homebrew state emulation** (current):
   keeps the engine, removes the dual-format condition by making the format
   identical, and verifies identity continuously against real Homebrew.

The v1 failure evidence (empty synthetic tabs claiming lifecycle authority
they could not describe — `bd2fe92bd`→`b91389ddd`) remains binding: v3 never
writes a receipt whose lifecycle facts it cannot state truthfully. The
delegation arguments recorded in the review history remain the honest
statement of v3's standing risk: Homebrew's formats are private and
churning, so v3 is only sound while the differential oracle and drift
policy hold.

## Required behavior

### Supported steady-state matrix

| Starting state                                       | mise status                    | mise apply                             | mise upgrade                               | ordinary Homebrew afterward           |
| ---------------------------------------------------- | ------------------------------ | -------------------------------------- | ------------------------------------------ | ------------------------------------- |
| Valid Homebrew formula, installed before declaration | installed                      | exact no-op                            | engine upgrade, brew-identical result      | fully supported                       |
| Valid Homebrew cask, installed before declaration    | installed                      | exact no-op                            | engine upgrade, brew-identical result      | fully supported                       |
| Formula absent                                       | missing                        | engine install, brew-identical result  | later engine upgrade                       | fully supported                       |
| Cask absent                                          | missing                        | engine install, brew-identical result  | later engine upgrade                       | fully supported                       |
| Installed through the new engine                     | installed                      | no-op                                  | engine or `brew upgrade` — interchangeable | indistinguishable from `brew install` |
| Legacy mise-only cask, truth provable                | installed after gated backfill | backfill receipts, no payload mutation | normal                                     | fully supported after backfill        |
| Legacy mise-only cask, truth not provable            | needs repair                   | no mutation, one-line fix instruction  | no mutation                                | untouched                             |
| Homebrew state corrupt/unparseable                   | error for that package         | no mutation                            | no mutation                                | untouched                             |

`apply` means presence reconciliation, not update. An installed older version
satisfies an unpinned `"latest"` declaration. Only the existing explicit
`mise bootstrap packages upgrade` operation requests an upgrade. Upgrade
mirrors brew semantics, including skipping `auto_updates` casks (no implicit
`--greedy` behavior).

### Non-negotiable invariants

1. Production mise never executes the `brew` CLI for any purpose. Real
   `brew` runs only inside tests as the differential oracle.
2. Engine-written state is complete and truthful: every receipt field, SBOM,
   uninstall directive, and metadata file carries facts the engine actually
   established — never placeholders, never empty synthetic tabs.
3. Producer-identity fields carry the pinned emulated Homebrew version,
   bumped only after differential re-verification.
4. Any valid installed state of the requested kind satisfies status,
   regardless of which tool wrote it. Status is read-only and never
   elevates.
5. Package kind is always explicit; same-named formulae and casks are never
   guessed.
6. Treat every version string as opaque. No semver sorting, normalizing, or
   newest-version inference.
7. Reads are schema-strict and version-tolerant: newer `homebrew_version`
   alone is never an error; unknown/corrupt structure blocks mutation of
   that package. Clean absence stays distinguishable from failure.
8. Removal executes the recorded uninstall directives and metadata teardown
   exactly as `brew uninstall` would; no partial teardown, no leftover
   `.metadata`.
9. Legacy mise-only state is backfilled only under the provable-truth gate;
   otherwise preserved untouched with needs-repair reporting.
10. Never run destructive migration tests on a developer's installed apps.

## Homebrew on-disk contract the engine emulates

The engine reads and writes Homebrew's real on-disk structures. They are
private and unversioned; the differential oracle plus the drift policy are
what make relying on them sound. Structures verified 2026-08-12 against
Homebrew `6.0.17`:

### Canonical prefixes

| Platform           | Prefix                       |
| ------------------ | ---------------------------- |
| macOS arm64        | `/opt/homebrew`              |
| Linux x86_64/arm64 | `/home/linuxbrew/.linuxbrew` |

`MISE_SYSTEM_BREW_PREFIX` remains a test-only override, never documented as
custom-prefix support.

### Formula state (Cellar)

- Keg: `<prefix>/Cellar/<name>/<version>/` with `INSTALL_RECEIPT.json` and
  `sbom.spdx.json`.
- Receipt keys (observed, `6.0.13+`): `aliases`, `arch`, `built_as_bottle`,
  `built_on` (os, os_version, cpu_family, xcode, clt, preferred_perl),
  `changed_files`, `compiler`, `homebrew_version`, `installed_on_request`,
  `loaded_from_api`, `loaded_from_internal_api`, `poured_from_bottle`,
  `runtime_dependencies`, `source` (tap, tap_git_head, spec, versions,
  path), `source_modified_time`, `time`, `unused_options`, `used_options`.
- brew consumes `homebrew_version` semantically
  (`extend/os/linux/bottle_specification.rb:10` gates relocation on
  `>= 5.1.15`); the SBOM creator string embeds the same version
  (`sbom.rb:191`).
- Link state: `opt/<name>` symlink, `var/homebrew/linked/<name>`, public
  prefix links — already produced natively; identity confirmed by the
  oracle.

### Cask state (Caskroom)

- Payload: `Caskroom/<token>/<version>/...`.
- Metadata: `Caskroom/<token>/.metadata/` containing the authoritative
  top-level `INSTALL_RECEIPT.json`, `config.json`, and the timestamped
  definition snapshot
  `.metadata/<version>/<timestamp>/Casks/<token>.json`.
- Cask receipt keys (observed): `arch`, `built_on`, `homebrew_version`,
  `installed_on_request`, `loaded_from_api`, `loaded_from_internal_api`,
  `runtime_dependencies`, `source` (tap, tap_git_head, version, path),
  `time`, `uninstall_artifacts`, `uninstall_flight_blocks`.
- `uninstall_artifacts` records the version-specific uninstall/zap
  directives (binary links, generated completions, zap targets…). Real brew
  executes these destructively at uninstall time — they must be derived
  truthfully from the cask DSL of the installed version.
- The July 2026 format change (installed metadata moved to JSON, top-level
  receipt authoritative) is the floor: the engine emulates the current
  format only, never resurrects pre-JSON layouts.

### Taps

`[bootstrap.brew.taps]` remains configuration. The engine keeps its current
native tap-metadata consumption (published API metadata for fully qualified
`owner/tap/name` requests). Taps without published API metadata stay outside
the engine's scope, as today.

## No Homebrew prerequisite, no new mise CLI

The engine performs installation itself, so a Homebrew installation is not
required for `brew:`/`brew-cask:` bootstrap to work — unchanged from current
behavior. When real Homebrew is present, both tools operate on the same
state interchangeably; that is the point of this plan. mise never installs
Homebrew itself, and no new command, flag, mode, or setting is added:
existing `status`, `apply`, `upgrade`, `import`, and `prune` carry the whole
contract.

## Legacy direct-mise installations

### Formulae created by current mise

Existing mise-poured kegs already carry brew-shaped receipts that real
Homebrew mechanically accepts. They are left untouched — no receipt rewrite,
no repour. New pours write the corrected truthful receipts (decision 7).
The differential corpus determines whether any legacy pour class is rejected
by real brew; such a case reports needs-repair, never silent repair.

### Casks created by current mise (gated backfill — decision 2)

A `.mise-cask.toml` cask without Homebrew `.metadata` is converted during
status/apply to the full truthful receipt set **only when truth is
provable**:

- the mise legacy receipt exists and parses;
- artifact fingerprints still match the installed payload;
- the cask definition for the installed version is obtainable — installed
  version equals current catalog version, because uninstall directives are
  version-specific.

On success the engine writes the complete `.metadata` receipt set and
removes `.mise-cask.toml`. If any condition fails (auto-updated app,
fingerprint mismatch, version drift), the cask reports needs-repair with a
one-line reinstall instruction and nothing is mutated. The July `99e2e50a5`
empty-tab backfill remains the anti-pattern: backfill without provable truth
is forbidden.

## Current state in this repository

Excerpts captured at baseline `14888188d`; re-verify after the rebase onto
`origin/main` (which adds cask prune #11810, font casks, platform filters —
`cask.rs` grows to 8,296 lines there, with the same bug pattern at
`cask.rs:290`/`332`/`4428`).

### Formula engine: near-1:1 already, but lies about its producer

`src/system/packages/brew/mod.rs:1-18` documents the native pour design.
`BrewManager::install_via_pour` resolves APIs and dependencies, downloads or
builds formulae, bootstraps the prefix, relocates binaries, and links kegs.
Real Homebrew mechanically accepts these kegs today.

`src/system/packages/brew/pour.rs:302-379` writes the receipt with the
falsified producer value:

```rust
"homebrew_version": "5.1.15 (mise)",
```

`BrewManager::installed` at `src/system/packages/brew/mod.rs:236-251` uses
mise's opt/link records. Receipt-field fidelity against real brew
(`changed_files`, `built_on`, SBOM presence) has never been verified — the
differential oracle decides what else needs correction.

### Cask engine: separate format, exact inbound bug

At `src/system/packages/brew/cask.rs:237-250`, install fetches current
metadata, parses mise-supported artifacts, then rejects any Homebrew
metadata:

```rust
if homebrew_metadata_present(&cask.token) {
    bail!("... Homebrew owns this cask ...");
}
```

At `src/system/packages/brew/cask.rs:531-551`, status calls
`installed_cask_version`; at `3618-3660` that helper requires mise's own
receipt/fingerprints. A normal Homebrew cask has neither, so mise reports it
missing — then install rejects it. The cask engine writes `.mise-cask.toml`
instead of Homebrew `.metadata`, which is the dual-format condition this
plan removes.

One PATH-brew call exists: `cask_ruby_bin` (`cask.rs:780-784`) runs
`brew ruby` to locate a Ruby interpreter. It must be replaced by mise's own
Ruby discovery under the zero-brew-CLI rule.

### Existing commands already support the target UX

`src/cli/system/driver.rs:90-164` already queries manager state before
mutation, filters installed packages out of apply, sends present packages to
upgrade, and re-queries afterward. `src/system/packages/mod.rs:84-127`
provides the manager contract. No command surface changes: the engines'
state model changes underneath.

### Removal paths must meet the same bar

`src/system/packages/brew/maintenance.rs:150-161` and
`src/cli/system/prune.rs:66-89` unlink and delete formula kegs; main's
jdx/mise#11810 adds cask prune. Under decision 5, every removal path must leave
post-state identical to `brew uninstall`, including executing recorded
`uninstall_artifacts` and tearing down `.metadata` — verified by the
round-trip oracle.

Import stays read-only; it may now read the same truthful receipts the
engine writes (identity and `installed_on_request`), still without the brew
CLI.

## Commands the executor will need

| Purpose                | Command                                                             | Expected result                                             |
| ---------------------- | ------------------------------------------------------------------- | ----------------------------------------------------------- |
| Targeted unit tests    | `rtk cargo test --all-features system::packages::brew`              | exit 0                                                      |
| Linux formula e2e      | `rtk mise run test:e2e e2e/cli/test_system_install_brew_linux`      | all assertions pass                                         |
| macOS formula/cask e2e | `rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow` | all assertions pass on macOS CI; intentional skip elsewhere |
| Lint/fix before commit | `rtk mise run lint-fix`                                             | exit 0; stage resulting relevant fixes                      |
| Lint                   | `rtk mise run lint`                                                 | exit 0                                                      |
| Full gate              | `rtk mise run ci`                                                   | exit 0                                                      |

Never execute e2e scripts directly. Use the mise tasks shown above.

## Scope

### In scope

- `src/system/packages/brew/cask.rs` — read Homebrew `.metadata` as installed
  state; write the full truthful `.metadata` receipt set on install/upgrade;
  delete the ownership guard; brew-equivalent already-installed/upgrade
  semantics; gated legacy backfill; uninstall/reinstall post-state parity;
  replace the `brew ruby` interpreter lookup.
- `src/system/packages/brew/pour.rs` — truthful formula receipts: pinned
  `homebrew_version`, field fidelity per the oracle, SBOM parity.
- A shared receipt module under `src/system/packages/brew/` for the emulation
  pin, receipt/metadata schemas, schema-strict parsers, and writers used by
  formula, cask, import, and prune paths.
- `src/system/packages/brew/mod.rs`, `maintenance.rs`, `prefix.rs`,
  `src/cli/system/prune.rs`, `src/cli/system/import.rs` — status/import/prune
  reading the same truthful state; removal parity (formula and cask prune).
- Existing brew/cask unit tests and both e2e suites, upgraded to the
  differential brew-vs-mise form; CI workflow changes only as needed to run
  them (macOS runners already ship Homebrew).
- `docs/bootstrap/packages/brew.md` and generated docs affected by changed
  help.

### Out of scope

- New commands, subcommands, flags, modes, settings, or ownership selectors.
- Installing Homebrew, or requiring it at runtime.
- Deleting the engine or delegating any production operation to the brew CLI.
- Taps without published API metadata (unchanged current limitation).
- Homebrew services.
- Registry changes.
- Private downstream repositories or user-machine mutation experiments.
- Token-specific branches for any application corpus.

## Git workflow

Use coherent, reviewable commits. Do not push or open a PR unless instructed.
Every commit uses conventional commit format with DCO signoff
(`git commit -s`).

Recommended sequence (after rebasing onto current `origin/main`):

1. `refactor(brew): add shared Homebrew receipt schema module with emulation pin`
2. `fix(brew-cask): recognize Homebrew-installed casks in status and apply`
3. `fix(brew-cask): write full truthful Homebrew metadata on install`
4. `fix(brew): write truthful formula receipts with pinned producer version`
5. `fix(brew-cask): backfill provable legacy mise casks to Homebrew metadata`
6. `fix(brew): align prune and uninstall post-state with brew uninstall`
7. `test(brew): add differential brew-vs-mise e2e oracle`
8. `docs(brew): document 1:1 Homebrew-compatible bootstrap`

Sequencing rule: commit 2 (read-side recognition) fixes the production
bootstrap failure on its own and must not depend on write-side changes.
Write-side switches (3 and 4) are each atomic: never commit an engine that
half-writes the new format. The ownership guard is deleted in commit 3, not
commit 2, so an interim build cannot overwrite Homebrew state with the old
mise-only format.

## Implementation steps

### Step 1: Shared receipt schema module with emulation pin

Add a module under `src/system/packages/brew/` owning:

1. the emulation pin constant (initially `6.0.17`) used for every
   producer-identity field;
2. typed models of the formula receipt, cask `.metadata` receipt,
   `config.json`, and the timestamped definition snapshot;
3. schema-strict parsers: unknown/missing required structure or unparseable
   JSON yields a classified error, never `Missing`; newer
   `homebrew_version` strings pass through as opaque data;
4. writers producing byte-stable output (key order/formatting matched to
   real brew output so the differential diff stays clean);
5. fixture-based unit tests using receipts captured from real Homebrew
   `6.0.17` (record the generating brew version in each fixture header).

**Verify:** `rtk cargo test --all-features system::packages::brew`

### Step 2: Read-side recognition (fixes the production failure)

`BrewCaskManager::installed`/status accepts a valid Homebrew `.metadata`
receipt for the requested token as installed state, with the recorded opaque
version. `latest` declarations are satisfied by any installed version.
Legacy `.mise-cask.toml` detection is retained. Partial/corrupt state
reports needs-repair. The ownership guard stays in place this step, so the
old writer cannot clobber Homebrew state before Step 3 lands; guarded
installs are unreachable for healthy state because status now filters them
out of apply.

Formula status is already effectively shared-state; add tests proving
brew-installed formulae report installed.

The observed Codex scenario becomes an e2e regression test: Homebrew-owned
cask + declaration → status installed, apply no-op.

**Verify:** unit tests plus
`rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow`

### Step 3: Cask write-side 1:1 (atomic)

In one commit, the cask engine writes the complete truthful `.metadata`
receipt set on install/reinstall/upgrade — top-level `INSTALL_RECEIPT.json`
(including `uninstall_artifacts` derived from the installed version's DSL,
`source` with tap/tap_git_head, timestamps, pinned `homebrew_version`),
`config.json`, and the timestamped `Casks/<token>.json` snapshot — and stops
writing `.mise-cask.toml` for new installs. The ownership guard and the
mise-receipt-only version check are deleted; already-installed behaves like
brew ("already installed" no-op); upgrade skips `auto_updates` casks.
Replace `cask_ruby_bin`'s `brew ruby` call with mise-native Ruby discovery.

**Verify:** unit tests; macOS differential e2e (Step 7 harness may land
early in reduced form): brew-install vs engine-install of the same cask
compare equal after normalization; engine-installed cask survives
`brew list/info/upgrade/uninstall`.

### Step 4: Formula receipt truth (atomic)

`pour.rs` writes the pinned `homebrew_version` (removing `"5.1.15 (mise)"`),
SBOM creator parity, and any field corrections the differential diff
demands (`built_on`, `changed_files`, `source.tap_git_head`, timestamps).
Existing installed kegs are not rewritten.

**Verify:** unit tests; Linux differential e2e: brew-poured vs engine-poured
keg for the same bottle compare equal after normalization.

### Step 5: Gated legacy cask backfill

Implement decision 2's provable-truth gate in status/apply: fingerprints
match + installed version equals current catalog version → write full
receipt set, remove `.mise-cask.toml`; otherwise needs-repair with a
one-line reinstall instruction. Never mutate payloads during backfill.

**Verify:** unit tests covering: provable case converges; version-drifted
auto-update case reports needs-repair; fingerprint mismatch reports
needs-repair; backfilled cask then passes `brew uninstall` round-trip in
the differential e2e.

### Step 6: Removal parity (formula + cask prune, uninstall paths)

Formula prune, cask prune (#11810 via rebase), and cask uninstall/reinstall
teardown leave post-state identical to `brew uninstall`: execute recorded
`uninstall_artifacts`, remove the Caskroom version dir and `.metadata`, and
match brew's keg/rack/link teardown. Prune candidate selection keeps its
current meaning; dry-run previews without mutation; failures abort before
the first removal.

**Verify:** unit tests plus round-trip e2e — engine install → `brew
uninstall` clean, and brew install → engine prune/uninstall leaves the same
post-state real brew leaves.

### Step 7: Differential oracle e2e

Upgrade both existing e2e suites to the differential form, keeping the
repository's e2e conventions (bash + `assert*`, run via mise tasks):

- Linux: real Homebrew present in the CI container; one formula installed
  via brew and via the engine (fresh state each side); normalized diff of
  keg tree + receipt + SBOM; round-trips both directions; import/prune
  assertions preserved.
- macOS: same for one small app cask and one font cask (fixtures from the
  corpus equivalence classes); round-trips both directions.
- Normalization list is explicit in the harness: timestamps, `built_on`
  machine facts, `source.path` cache location, file mtimes. Everything else
  must match byte-for-byte.
- Implementation-phase local verification: classify the 37 essential-mac
  casks by artifact/lifecycle class, run the differential pass on one
  representative per class, and record the class table plus results in the
  PR description.

**Verify:** `rtk mise run test:e2e e2e/cli/test_system_install_brew_linux`
and `rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow`

### Step 8: Documentation

Update `docs/bootstrap/packages/brew.md` to state: the engine installs
Homebrew packages natively and produces state identical to real Homebrew
(verified against the pinned brew version); mixing `brew` and mise on the
same machine is supported in both directions, including uninstall; apply
ensures presence, upgrade is explicit and mirrors brew semantics
(`auto_updates` casks are skipped); legacy mise-only casks converge via the
gated backfill or report a one-line repair path; the test-only prefix
override promises no custom-prefix support. Remove claims that
mise-installed packages are unsupported by `brew list/upgrade`.

**Verify:** `rtk mise run lint`

## Mandatory tests

### Unit tests (fixture-based, offline)

Receipt fixtures captured from real Homebrew `6.0.17`, generating version
recorded per fixture. Cover:

- formula receipt parse/write round-trip byte-stable;
- cask `.metadata` receipt, `config.json`, timestamped snapshot round-trip;
- schema-strict failures: unknown structure, missing required keys,
  malformed JSON → classified error, never `Missing`;
- newer `homebrew_version` strings accepted as opaque data;
- opaque versions containing commas, `@`, revision suffixes, non-semver
  text;
- status matrix: brew-installed, engine-installed, absent, partial/corrupt,
  legacy mise-only (provable and unprovable backfill gates);
- `uninstall_artifacts` derivation from cask DSL matches real-brew fixture
  receipts for each artifact class;
- pinned producer version appears in every producer-identity field; no
  `"(mise)"` marker anywhere;
- backfill writes nothing when any truth condition fails;
- removal teardown plan matches the recorded `uninstall_artifacts`.

### Differential oracle e2e (real brew, disposable CI)

Real Homebrew present; record brew and mise versions in logs.

For one formula (Linux) and one app cask plus one font cask (macOS):

1. fresh-install via real `brew`; capture normalized state snapshot A
   (keg/Caskroom tree, receipts, SBOM, links);
2. clean removal via `brew uninstall`;
3. fresh-install the same package via the mise engine; capture normalized
   snapshot B;
4. assert A == B byte-for-byte after the explicit normalization list
   (timestamps, `built_on` machine facts, `source.path`, mtimes);
5. round-trip: engine-installed package passes `brew list/info/upgrade/
uninstall` cleanly; brew-installed package satisfies mise status/apply as
   a no-op and mise removal leaves brew-identical post-state;
6. mise status and apply run twice against installed state and change no
   hashes.

Implementation-phase local verification: the essential-mac corpus reduced to
artifact/lifecycle equivalence classes, one representative per class,
results recorded in the PR. Never run destructive tests against a
developer's real installed applications; corpus verification uses fresh
disposable state only, and the operator's live machines are read-only
evidence.

## Done criteria

All must hold:

- [ ] Existing valid Homebrew formulae and casks report `installed` in mise,
      and applying their declarations performs no mutation (Codex regression
      test green).
- [ ] Engine-installed formulae and casks pass real
      `brew list/info/upgrade/uninstall` cleanly.
- [ ] Differential oracle: brew-install vs engine-install of each e2e
      fixture compares byte-identical after the explicit normalization list.
- [ ] Removal parity: engine prune/uninstall post-state equals
      `brew uninstall` post-state for the fixtures.
- [ ] Producer-identity fields carry the pinned emulated version everywhere;
      `"5.1.15 (mise)"` and `.mise-cask.toml` writes are gone from new
      installs.
- [ ] Production code contains zero `brew` CLI invocations (including the
      former `brew ruby` lookup).
- [ ] Legacy mise-only casks converge via the provable-truth backfill or
      report needs-repair with the one-line instruction; nothing is silently
      deleted, reinstalled, or fabricated.
- [ ] Schema-strict reads: corrupt/unknown state blocks mutation of that
      package; newer brew versions alone never error.
- [ ] Implementation-phase corpus verification recorded: essential-mac
      equivalence-class table plus per-representative differential results.
- [ ] No new mise CLI/config surface was added.
- [ ] Version strings remain opaque; no new semver ordering exists.
- [ ] Targeted unit and Linux/macOS e2e tests pass.
- [ ] `rtk mise run lint` passes.
- [ ] `rtk mise run ci` passes.
- [ ] `rtk git status --short` shows no files outside the approved scope
      except repository-generated docs required by the render process.

## STOP conditions

Stop and report; do not improvise if:

- a receipt or metadata fact cannot be established truthfully from what the
  engine actually did (never write a placeholder to pass the diff);
- the differential oracle reveals a state difference that cannot be closed
  without reproducing undocumented Homebrew behavior the engine does not
  implement (report the gap; do not normalize it away);
- real Homebrew rejects or mismanages engine-written state for any fixture;
- a legacy backfill would proceed despite a failed truth condition, or would
  require deleting a user app, rerunning a pkg/installer, or fabricating
  version facts;
- a compatibility test would touch non-fixture user applications, security
  software, networking, virtualization, login items, or user data;
- any production path would invoke the `brew` CLI;
- status cannot distinguish clean absence from corruption/unknown failure;
- a package name, tap name, or URL would pass through shell evaluation;
- any step introduces mise-side version ordering;
- an in-scope file has drifted enough that the current-state excerpts no
  longer describe it;
- a verification command fails twice after one focused correction.

## Rejected approaches

### Full delegation to the canonical brew executable

Rejected by the maintainer 2026-08-12: the internal engine is a critical
mise capability. The delegation design remains recorded in the review
history as the fallback if 1:1 emulation proves unsustainable.

### Delete only the cask ownership guard (without 1:1 state)

Rejected. Removing the guard while the engine still writes a mise-only
format lets mise overwrite a Homebrew payload while stale Homebrew
lifecycle metadata survives. Guard deletion is only valid together with
full truthful metadata writing (Step 3).

### Synthetic or empty receipt metadata

Rejected — already failed in production (July 2026, `bd2fe92bd` →
`b91389ddd`). A receipt is written only when every lifecycle fact in it is
true. This includes unconditional legacy backfill from current catalog
definitions when the installed version differs.

### Route based on which manager originally installed the package

Rejected. Origin tracking reintroduces a second metadata source — the
enabling condition of the original bug class. One format, one state model.

### `brew install --cask --adopt` migration for legacy casks

Rejected as the migration path (it was a delegation-era tool and re-runs
installers/stages current content). Superseded by the gated truthful
backfill.

### Automatically install Homebrew

Moot and rejected: the engine requires no Homebrew installation, and mise
never runs Homebrew's installer.

### Hard version lock / best-effort parsing

Rejected drift policies (decision 10): a version lock breaks users within
days of each brew release; best-effort continue on misunderstood lifecycle
state is the destructive-accident class.

## Maintenance notes

- Review future brew-manager changes by asking whether the written state
  would still diff clean against real Homebrew. The differential oracle is
  the arbiter; a change that cannot pass it does not ship.
- When Homebrew changes a private format, the sequence is: oracle fails →
  reproduce the change in the engine → re-verify → bump the emulation pin.
  Never compensate with placeholder fields or normalization-list widening
  that hides real divergence.
- Keep receipt schemas, writers, and parsers in the shared module so formula
  and cask handling cannot drift apart.
- If Homebrew later publishes supported registration/import or receipt
  schemas, prefer them over reverse-engineered structures and shrink the
  private surface the engine tracks.
- The delegation design (v2) is the documented fallback if the oracle
  becomes unmaintainable; do not partially delegate single operations ad
  hoc.
