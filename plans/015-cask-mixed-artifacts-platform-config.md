# Plan 015: Implement mixed cask artifacts and platform-correct configuration

Status: IN PROGRESS
Priority: P0
Effort: M
Planned against: #11910 `05ccd7ab8`
Depends on: 013, 014
Implementation start: #11910 `279530a1e33814c7c6a49aca6198ba67efb124b3`
Implementation commit: `bca8747bd362679a6c97d72f3e0001539730f011`

Drift check (2026-08-13): `manpage` remains in the non-installing allowlist and
is absent from receipt/status/prune topology. `native_cask_config` still emits
macOS application/font/audio paths unconditionally on Linux.

## Objective

Implement `manpage` as a complete installing artifact and derive native receipt
configuration from the same platform directory model used for activation. A
mixed-artifact cask must install, report, upgrade, and remove all mechanisms.

## Current defects

- `manpage` is classified as non-installing and silently skipped. Ghostty
  currently contains an app, two manpages, and three completions, so calling it
  “App-only” hid the defect.
- Homebrew-target derivation also omits manpages, leaving real-brew external
  links during mise prune.
- `cask.rs:4539-4567` writes macOS paths on Linux. Homebrew 6.0.17 Linux uses
  `~/.config/apps`, XDG fonts, `~/.vst`, and `~/.vst3`.

## Files in scope

- `src/system/packages/brew/cask.rs`
- cask config/target helper extracted from it if useful
- Ghostty and Linux config fixtures/tests

## Implementation steps

1. Add a single effective cask-directory object derived from platform, prefix,
   home, and XDG environment. It is the sole source for artifact targets,
   ownership validation, native `config.json`, status, and teardown.
2. Match pinned Homebrew 6.0.17 defaults. macOS retains its app/font/VST paths.
   Linux uses `~/.config/apps`, `${XDG_DATA_HOME:-~/.local/share}/fonts`,
   `~/.vst`, `~/.vst3`, and verified Homebrew locations for remaining fields.
3. Parse `manpage` into a typed artifact. Validate filename/section consistency,
   legal manual section, source containment, and exact target directory before
   mutation.
4. Stage/activate manpages through plan 013's owned transaction with Homebrew
   symlink topology. Include them in receipt inventory, status, upgrade claims,
   predecessor comparison, and plan 014 teardown.
5. Remove the silent “non-install artifact” classification. Unsupported
   manpage shapes fail during preflight, not after other artifacts install.
6. Update corpus fixtures to represent Ghostty as a mechanism combination, not
   a single app label. Plan 018 owns the complete matrix/oracle result.

## Required tests

- Ghostty-shaped app + two manpages + bash/zsh/fish completions: every artifact
  exists with correct topology; status healthy; upgrade and prune remove only
  owned targets.
- Invalid section, target escape, foreign existing manpage, and source symlink
  escape fail before any artifact mutation.
- Receipt/config snapshot on macOS matches real brew for every emitted field.
- Linux tests set custom `XDG_DATA_HOME`; actual font/app/VST targets and emitted
  config are identical and real-brew uninstall removes the same paths.
- No macOS-only path appears in a Linux receipt.

## Verification

Local proof at `bca8747bd362679a6c97d72f3e0001539730f011`:

- Manpages participate in stage, activation, receipt inventory, installed
  topology, predecessor claims, and prune. Invalid sections/escapes fail.
- Ghostty-shaped unit coverage executes app + two manpages + bash/fish/zsh
  completions. `plans/007-corpus-results.md` now pins all 37 names as a
  multi-mechanism matrix. The 2026-08-13 drift audit advanced the observed tap
  head to `38e49d2d9b9113d2384550124f9dca83323c73a8`. Canonical API digests exclude
  analytics, generation time, and the tap-global head while retaining each
  cask's exact `ruby_source_checksum` and all operational fields.
- One platform directory model drives target paths and native config. Linux
  tests cover `~/.config/apps`, XDG fonts, `~/.vst`, and `~/.vst3`, and reject
  macOS path leakage.
- Adopted Homebrew receipts use their installed artifacts, not current catalog
  artifacts. Relevant custom Homebrew directories fail as NeedsRepair instead
  of being silently reinterpreted through mise defaults.
- Focused cask tests: 169 passed; all brew tests: 226 passed; Clippy: zero
  errors.

Pinned real-brew round trips remain required by plan 018; matrix rows are
explicitly oracle-pending, so this plan stays IN PROGRESS.

```bash
rtk cargo test --bin mise system::packages::brew::cask
rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow
rtk mise run test:e2e e2e/cli/test_system_install_brew_linux
rtk mise run lint
```

## Done criteria

- Manpage has full install/status/upgrade/prune behavior.
- Ghostty is tested as a mixed mechanism set.
- Receipt configuration and actual target calculation share one platform model.
- Linux contains no macOS default paths.

## Stop conditions

Do not silently skip an installing artifact. Do not emit a directory merely
because Homebrew has a field for it; prove its current 6.0.17 platform default.
