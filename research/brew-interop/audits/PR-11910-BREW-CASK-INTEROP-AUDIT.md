# PR #11910 — native Homebrew interop audit

PR: <https://github.com/jdx/mise/pull/11910>  
Head: `a1df6b7fd11c2a7a1c7bb802f9a503f98e122496`  
Required ancestor: #11915 `97640cc04359fa6488706ac46bf71a46cf840b3b`  
State after audit: draft

## Refresh verification

Verified again with current upstream main `6f52dcdf9` and exact head `a1df6b7`.
GitHub reports open/draft/mergeable; `gh pr checks` reports 33 passed/0 failed
and GraphQL reports 30/30 review threads resolved. Formula ancestor `97640cc`
is proven; direct
second-stack delta remains 51 files, 1,403 hunks, `+21,127/-7,168`, 27 commits
including 6 merge commits. GitHub's
combined `main` view remains 61 files, `+46,752/-8,111`.

Inline-thread count is not the full review gate. A top-level Codex P1 about
cross-platform cask state remains visible and is not an inline thread. Current
head patches its exact apply failure, but related status/doctor semantics remain
inconsistent; a reduced successor needs fresh full review.
`docs/contributing.md:17-20` requires AI feedback addressed before maintainer
review. No approval or review decision exists; jdx explicitly refused current
review shape.

Top-level Codex feedback disposition:

| Finding | Current-head disposition |
|---|---|
| P1 platform skip | Apply failure patched through a `brew-cask` driver special case; status, doctor, and install hints remain structurally inconsistent. |
| P2 formula deletion tombstones | Patched with `.mise-delete-*` naming and `.mise-*` filtering (`src/file.rs:210-214`, `src/system/packages/brew/pour.rs:1704-1712`). |
| P2 cached response filename | Patched with an effective-filename sidecar (`src/system/packages/brew/cask.rs:1961-2006`). |

Resolved feedback does not remove the independent ownership, scope, docs, and
CI blockers below.

## Verdict

Core ownership fix is correct. Current PR is not acceptable: second-stack delta remains +21,127/-7,168 across 51 files; it contains unrelated breaking changes and generic coupling. Maintainer explicitly refused the current 40k-line review shape. Recreate after reduced formula lifecycle merges.

## Root cause and structural fix

Bug class: ownership and installed state were inferred from producer-private mise receipts instead of shared native prefix state.

Good implementation evidence:

- `src/system/packages/brew/cask.rs:8928-8976`: detects `.metadata`, reads native receipt, validates version directory/topology, returns `Installed`.
- `src/system/packages/brew/cask.rs:8891-8905`: healthy native install is no-op.
- `src/system/packages/brew/cask.rs:1354-1394`: status reports installed state.
- `e2e/cli/test_system_install_brew_macos_slow:85-86,303-315`: includes pinned `1password-cli` and Homebrew-to-mise no-op/prune flow.

This directly removes the reported `Homebrew owns this cask` failure for a healthy native install.

Correct reduced behavior:

- Read and validate native metadata.
- Treat healthy native ownership as installed for status/bootstrap.
- Never mutate/convert foreign state during status.
- Never take ownership merely because state is recognized.
- Never upgrade or prune foreign Homebrew state without explicit adoption.
- Refuse conflicting install mutation with precise guidance.
- Permit explicit adoption only through existing adoption contract.

## Blocking findings

### Third-party support regression and inaccurate docs

`src/system/packages/brew/cask.rs:1466-1487` rejects every non-official cask.
That is a breaking change in #11910's own second-stack delta and is unrelated to
recognizing a healthy Homebrew-owned cask.

Formula tap support is not removed. `src/system/packages/brew/resolve.rs:458-465`
still resolves a configured third-party tap to a GitHub commit, and
`src/system/packages/brew/api.rs:1125-1216` reads formula metadata from that
immutable snapshot. `docs/bootstrap/packages/brew.md:23-29` therefore
incorrectly says both third-party formulae and casks are rejected. Requiring an
explicit GitHub URL and pinning a commit improves transport consistency, but it
does not provide an independent artifact identity: the same repository owner
controls formula metadata, URL, and checksum.

The resources/external-patches/macOS-source-build removal visible in #11910's
combined GitHub diff belongs to prerequisite #11915, not #11910's second delta.
It remains a blocker for the stack, but attribution matters for replacement
scope.

Restore current-main third-party cask behavior in the successor and preserve
existing formula-tap behavior unchanged. If stronger tap trust or removal is
required, design it separately and follow mise deprecation policy.

### Unsafe, nondeterministic receipt ordering

`src/system/packages/brew/cask.rs:10198-10245` calls libc `qsort` to imitate byte ordering. Equal-key order is unspecified; Windows uses a different stable path. Result can vary by libc/platform. Unsafe FFI is not needed for semantic compatibility.

Use safe deterministic Rust ordering. Validate Homebrew-readable semantics, not incidental native byte layout.

### Generic package-state coupling

`src/system/packages/mod.rs:63-126` and `src/cli/system/driver.rs:120-150` change unavailable/skip behavior into unsupported/fail behavior and special-case manager identity. This leaks cask ownership policy into shared package orchestration.

Keep platform availability and unsafe installed-state distinct. Model native/foreign/conflicting state inside brew-cask. Generic driver should consume a backend-neutral action/status contract without matching `brew-cask` by name.

### Cross-platform status/doctor regression

`src/system/packages/brew/cask.rs:1299-1304,1354-1379` now classifies offline
absence as `Missing` without platform/catalog classification. Only apply remaps
that state using a manager-name special case at
`src/cli/system/driver.rs:123-140`. Standalone status consumes `installed()`
directly and makes `--missing` fail (`src/cli/system/status.rs:54-82,136-137`);
doctor and install hints likewise count every non-installed state
(`src/cli/doctor/mod.rs:521-531,744-751`, `src/cli/install.rs:222-237`).

Result: docs promise shared macOS/Linux cask config can skip unsupported casks
(`docs/bootstrap/packages/brew.md:169-174`), but only apply skips them; status,
doctor, and hints falsely report them missing. Restore one backend-neutral
`Unavailable`/`Unsupported` classification at manager boundary and make all
callers consume it consistently.

### Excess CI

`.github/workflows/test.yml:181-565,867-965` adds 13 brew-specific check cells
to global `full-ci`: one macOS build, two canonical baseline/current cells,
eight 120-minute formula/cask corpus cells, and two Linux cells.

Keep broad differential corpus scheduled/manual. Required PR CI needs one
exact-head artifact per OS/architecture reused across same-platform jobs,
focused unit tests, one macOS native-ownership oracle, and only
platform-essential Linux coverage.

### Read recognition expands into foreign-state replacement/removal

The reported problem requires healthy Homebrew ownership to become installed/no-op.
The oracle goes further: Homebrew installs the cask, then an empty mise config
drives `mise ... prune` to remove it
(`e2e/cli/test_system_install_brew_macos_slow:285-315` and
`e2e/cli/test_system_install_brew_interop_casks_macos_slow:130-150`). Native
receipt candidates are indexed and removed at
`src/system/packages/brew/cask.rs:11563-11768,11864-11935`. That is ownership
takeover/removal, not native read-side interop. Keep Homebrew-owned state
read-only unless user explicitly adopts it through current main's adoption
contract.

`docs/bootstrap/packages/index.md:174-179` documents this destructive foreign
prune behavior, contradicting the narrower ownership summary at line 142. The
successor must document read-only recognition and explicit adoption consistently.

Upgrade is also unsafe for the requested ownership boundary.
`InstalledCaskState::Installed` retains only a version and erases producer
origin (`src/system/packages/brew/cask.rs:8876-8889`). `upgrade()` always passes
`upgrading = true` without adoption options (`:1322-1336`); version drift then
bypasses `existing_install_noop()` (`:8891-8905`) and uses the native receipt as
the predecessor teardown plan (`:734-787`). A normal upgrade can therefore
uninstall/replace a Homebrew-owned cask without explicit adoption.

Required fix: carry ownership origin in installed state. Healthy foreign state
remains visible and no-op for apply; upgrade must no-op or return precise
adoption guidance. Only explicit adoption may transfer later upgrade/prune
authority.

## Second-stack changed-file audit

This table audits `97640cc..a1df6b7`, not GitHub's combined `main..a1df6b7` view.

| File/group | Classification | Audit |
|---|---|---|
| `.github/workflows/test.yml` | reduce | Consolidate and path-gate; do not make all PRs pay broad brew oracle cost. |
| `.prettierignore`, `hk.pkl` | remove | Tooling churn unrelated to ownership fix. |
| `docs/bootstrap/packages/brew.md` | keep/rewrite | Document native recognition/adoption; do not claim formula taps are removed while code supports them or delete cask support in this fix. |
| `docs/bootstrap/packages/index.md` | conditional | Keep only necessary config/user-facing behavior; remove foreign-prune claims that contradict ownership safety. |
| `docs/dev-tools/mise-lock.md` | remove/split | Lock behavior is unrelated. |
| `docs/tasks/task-configuration.md` | remove | Task behavior unrelated. |
| `e2e/cli/brew_interop_macos_packages.tsv` | reduce/test | Useful corpus; mandatory PR CI should select minimal stable cases. |
| `e2e/cli/brew_oracle.sh` | test infrastructure | Preserve reusable semantic comparisons after scope reduction. |
| `e2e/cli/brew_oracle_guard.sh` | test infrastructure | Preserve only if dedicated destructive oracle remains. |
| `e2e/cli/test_lock_global` | remove | Lock regression unrelated to cask ownership. |
| `e2e/cli/test_system_install_brew_cask_appdir_slow` | keep/test | Relevant if appdir semantics touched. |
| `e2e/cli/test_system_install_brew_cask_gcloud_macos_slow` | conditional | Backlink/resource lifecycle is larger than native recognition; separate if needed. |
| `e2e/cli/test_system_install_brew_interop_casks_macos_slow` | keep/reduce | Retain Homebrew-to-mise no-op, reverse direction, prune/ownership invariants. |
| `e2e/cli/test_system_install_brew_interop_formulae_macos_slow` | move | Formula proof belongs #11915 or separate formula interop PR. |
| `e2e/cli/test_system_install_brew_macos_slow` | keep/reduce | Add exact `1password-cli` reproduction; avoid broad unrelated corpus. |
| `e2e/cli/test_system_install_brew_source_slow` | remove | Formula source behavior unrelated. |
| `e2e/run_test` | remove/split | Harness change belongs test infrastructure. |
| `e2e/tasks/*` | remove | Task global config changes unrelated. |
| `src/backend/aqua.rs` | remove | Unrelated backend regression/churn. |
| `src/cli/bootstrap.rs` | reduce | Keep generic convergence behavior only. |
| `src/cli/doctor/mod.rs` | remove/reduce | No cask-specific manager-name coupling in generic doctor. |
| `src/cli/install.rs` | remove/reduce | Same. |
| `src/cli/lock.rs` | remove | Unrelated. |
| `src/cli/system/driver.rs` | replace/reduce | Remove manager-name special case; use backend-neutral state/action. |
| `src/cli/system/status.rs` | reduce | Preserve stable JSON and precise status without generic semantic changes. |
| `src/config/mod.rs` | remove/regression | Unrelated stale delta reverts lockfile-discovery caching and changes `backend::alias_backends()` back to `backend::list()`; drop it. |
| `src/http.rs` | remove | Broad HTTP change unrelated. |
| `src/system/packages/brew/api.rs` | reduce | Keep fields needed to parse native cask identity and supported artifacts; remove unrelated formula-tap trust redesign. |
| `src/system/packages/brew/cask.rs` | replace/reduce | Core file, but +16,420/-6,499 is not minimum. Extract receipt parser and small recognition path. |
| `src/system/packages/brew/elf.rs` | move | Formula relocation belongs #11915 or separate. |
| `src/system/packages/brew/fetch.rs` | reduce | Retain only fetch behavior required by cask path. |
| `src/system/packages/brew/lifecycle.rs` | prerequisite/remove | Formula lifecycle file should not remain in second PR diff. |
| `src/system/packages/brew/maintenance.rs` | reduce | Keep cask-aware prune/removal ownership guards. |
| `src/system/packages/brew/mod.rs` | keep/minimal | Module registration only. |
| `src/system/packages/brew/pour.rs` | prerequisite/move | Formula pour changes disappear after prerequisite merge. |
| `src/system/packages/brew/receipt.rs` | keep | Extracted native receipt representation is appropriate; narrow public API. |
| `src/system/packages/brew/source.rs` | prerequisite/remove | Formula source changes do not belong here. |
| `src/system/packages/brew/tag.rs` | conditional | Keep only tag behavior required for native cask receipt identity. |
| `src/system/packages/brew/testdata/*` | keep/reduce | Keep documented captured receipts tied to focused parser tests; remove opaque unused one-line fixtures. |
| `src/system/packages/mod.rs` | replace | Avoid global unsupported semantics for cask-specific ownership. |
| `src/system/resources.rs` | reduce | Keep only if artifact/resource ownership needs shared invariant. |
| `src/test.rs` | conditional | Test helper only. |

Files visible only because GitHub base is `main` and #11915 is embedded—`Cargo.*`, `cmd.rs`, `file.rs`, sandbox files, formula lifecycle/resolve/SBOM/shim/source, exec/run/task executor, formula tests—must disappear from #11910 review after prerequisite merge/rebase.

## Current-main interaction

Main merged #12074 for adoption configuration/receipt/self-update behavior,
#12222 for content-drift semantics, and several small cask follow-ups. They are
directionally aligned and much smaller. #12074 does not preserve producer
origin for native Homebrew state. Current
`origin/main:src/system/packages/brew/cask.rs:428-442` still bails on existing
`.metadata` before installed detection, so none solve healthy Homebrew-owned
`1password-cli` without ownership transfer.

Build reduced fix on current main after formula prerequisite. Reuse #12074's
adoption configuration and receipt machinery, then add an explicit producer
origin model and native recognition; do not replace its implementation wholesale.

## Tests required before ready

1. Fixture/unit: valid native receipt + version/topology returns installed.
2. Fixture/unit: malformed/ambiguous native state does not mutate and returns precise conflict/repair state.
3. Status purity: no receipt conversion, deletion, link creation, or ownership transfer.
4. Exact bootstrap config including `brew-cask:1password-cli`; Homebrew-installed state produces success/no-op.
5. Upgrade/prune: foreign Homebrew ownership remains untouched.
6. Mise-installed cask remains maintainable using mise receipt.
7. Explicit adoption still follows #12074 contract.
8. Safe deterministic receipt serializer tests, including null native-probe fields and unsupported OS error.
9. Shared macOS/Linux config: apply, status, doctor, and install hints agree on platform-inapplicable casks without backend-name checks.

The exact downstream declaration is
`/Users/donbeave/Projects/donbeave/essential-mac/mise.toml:57`:
`"brew-cask:1password-cli" = "latest"`. Use that configuration shape in the
focused regression; no synthetic ownership conversion is needed.

## Stack acceptance

Current ancestry is correct and conflict-free. Current presentation is not. Formula successor must merge first. Then create/rebase cask successor on new main so only its focused delta appears. Do not reopen #11910 for review in current form.

Eight stacked files are exact current-main synchronization blobs rather than
cask work (`docs/dev-tools/mise-lock.md`, `docs/tasks/task-configuration.md`,
`e2e/cli/test_lock_global`, three `e2e/tasks/*` files,
`src/backend/aqua.rs`, `src/cli/lock.rs`). `src/config/mod.rs` is not an exact
sync blob: it differs from current main by `+1/-28`, reverting lockfile-discovery
caching and changing `backend::alias_backends()` back to `backend::list()`.
That is an unrelated stale regression. Removing all nine unrelated files still
leaves 42 files, `+20,872/-7,095`; upstream synchronization does not explain the
oversized review surface.

No new local exact-head test is claimed. GitHub's green exact-head workflow is
the authoritative execution proof; source and E2E inspection establish the
ownership/no-op behavior and regressions above.
