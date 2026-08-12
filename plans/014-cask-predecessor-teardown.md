# Plan 014: Execute replayable predecessor teardown on upgrade/reinstall

Status: TODO
Priority: P0
Effort: L
Planned against: #11910 `05ccd7ab8`
Depends on: 013

## Objective

Parse and validate all installed-receipt teardown behavior before installation,
then execute the predecessor's uninstall phases with Homebrew successor-aware
semantics during upgrade/reinstall. Never install a cask whose future required
teardown mise cannot replay.

## Current defects

- Upgrade reads prior direct targets but never runs the installed receipt's
  uninstall actions. The existing executor is used only by prune.
- Install records arbitrary `uninstall_artifacts`, while the executor supports
  only `pkgutil`, `delete`, `quit`, and `launchctl`.
- Current corpus needs unsupported actions: `script` for cloudflare-warp and
  orbstack; `signal` for jetbrains-toolbox and zoom.
- Reusing prune wholesale would be wrong: upgrade/reinstall must preserve
  successor-owned artifacts and must not execute zap by default.

## Files in scope

- `src/system/packages/brew/cask.rs`
- typed cask lifecycle module if extraction improves exhaustive matching
- cask API/receipt fixtures and e2e tests

## Required architecture

Compile install, uninstall, and zap metadata into a closed typed lifecycle plan
before download or mutation. Store only a receipt representation that the same
version of mise can parse for future teardown. Unknown action, field, platform
guard, interpolation, or executable behavior makes the cask unavailable before
installation.

Operation semantics are explicit:

- Upgrade/reinstall: predecessor uninstall phases with successor context;
  preserve paths/packages/services claimed by the successor where Homebrew does.
- Prune/uninstall: full uninstall phases and artifact removal.
- Zap: separate explicit behavior; never run implicitly during upgrade/prune
  unless repository product semantics intentionally request zap.

## Implementation steps

1. Introduce exhaustive enums for uninstall actions and action-specific typed
   payloads. Parse the complete future teardown plan during install preflight.
2. Determine pinned Homebrew 6.0.17 semantics for `pkgutil`, `delete`, `quit`,
   `launchctl`, `script`, and `signal`, including successor filtering, missing
   targets, timeout, exit status, environment, and privilege boundaries.
3. Implement every action required by the pinned 37-cask mechanism matrix, with
   command confinement and path ownership checks. If truthful execution is not
   possible on a platform, mark affected casks unavailable before any mutation.
4. Read the predecessor's installed native receipt at upgrade/reinstall. Do not
   substitute current upstream metadata: removal must reflect what was installed.
5. Compare predecessor and successor prepared plans to compute successor-aware
   teardown. Execute it at the Homebrew-equivalent point in plan 013's durable
   transaction. Record every action before and after execution.
6. Share typed parsing/execution primitives with prune, but keep distinct
   operation policies. Unknown recorded actions return NeedsRepair/manual
   guidance; they never become silent no-ops.
7. Preserve diagnostic output and exact action failure. Do not continue into
   successor activation after a required teardown failure.

## Required tests

- Current codexbar-shaped predecessor invokes `quit` during upgrade.
- cloudflare-warp/orbstack `script` and jetbrains-toolbox/zoom `signal` either
  execute with verified Homebrew parity or fail availability before mutation.
- Successor retains an identical app/helper/service/package when Homebrew would
  preserve it.
- Upgrade never executes zap-only delete/rmdir actions.
- Prune executes full uninstall exactly once and leaves Homebrew-equivalent
  post-state.
- Installed receipt differs from live upstream API: predecessor receipt wins.
- Unknown action at fresh install => zero mutation. Unknown legacy action =>
  NeedsRepair, no destructive partial teardown.
- Interrupted action outcome follows plan 013 recovery policy and never blindly
  repeats.

## Verification

```bash
rtk cargo test --bin mise system::packages::brew::cask
rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow
rtk mise run lint
```

## Done criteria

- Upgrade/reinstall executes predecessor uninstall behavior with successor
  semantics before activation.
- Every newly recorded uninstall action is provably replayable by mise.
- Unsupported casks fail closed before download/mutation.
- Prune and upgrade policies cannot accidentally invoke zap.

## Stop conditions

Do not approximate a lifecycle command with path deletion. Do not fetch current
metadata to reinterpret an installed predecessor. Do not use shell strings when
the metadata provides executable/argv structure.
