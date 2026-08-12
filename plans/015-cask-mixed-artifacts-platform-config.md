# Plan 015: Implement mixed cask artifacts and platform-correct configuration

Status: TODO
Priority: P0
Effort: M
Planned against: #11910 `05ccd7ab8`
Depends on: 013, 014

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
