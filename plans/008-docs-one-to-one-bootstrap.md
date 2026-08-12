# Plan 008: Document the 1:1 Homebrew-compatible bootstrap

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update `plans/README.md` row when done.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`),
> ONE final PR for all plans. Conventional commits, `git commit -s`, no
> agent trailers. Push to `fork` when done.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- docs/bootstrap/packages/brew.md`
> Changes beyond plans 001–007 → compare against live file, STOP on
> mismatch.

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: plans/001–007 (documents shipped behavior only)
- **Category**: docs
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

After plans 001–007, the brew/brew-cask engine's contract changed
fundamentally: engine-installed packages are fully manageable by real
Homebrew and vice versa. The docs currently describe the old mise-private
ownership model (including claims that Homebrew must not own declared
casks). Wrong docs would push users back into the exact mixed-state
confusion the engineering removed.

## Current state

- `docs/bootstrap/packages/brew.md` at `origin/main` `d5e0390dc` — read it
  fully before editing (it gained content on main vs the older baseline:
  cask cookbook link `73e9a53da`, prune notes from #11810, font notes).
  It describes mise-owned cask state, the "Homebrew owns this cask"
  refusal, and `.mise-cask.toml` semantics — all superseded.
- Documentation conventions: mise docs live under `docs/`, VitePress
  markdown; URL paths follow directory layout
  (`mise.jdx.dev/bootstrap/packages/brew.html` — AGENTS.md "URL
  Structure": always full path matching `docs/`). Generated docs are
  produced by `mise run render` tasks — this page is hand-written; run
  render only if changed CLI help demands it (plans 001–007 add no CLI
  surface, so it should not).
- The authoritative behavior statements to document are the design
  document's "Executive decision" and "Required behavior" sections
  (`plans/brew-cask-native-homebrew-interoperability.md`).

## Commands you will need

| Purpose                                | Command         | Expected on success |
| -------------------------------------- | --------------- | ------------------- |
| Lint (includes markdown/format checks) | `mise run lint` | exit 0              |
| Docs dev preview (optional)            | `mise run docs` | serves locally      |

## Scope

**In scope**:

- `docs/bootstrap/packages/brew.md`

**Out of scope**:

- Any `src/`, `e2e/`, or CI file.
- Other docs pages (link updates only if this page's anchors are
  referenced elsewhere — `grep -rn "packages/brew" docs/` first and fix
  only broken links).
- Generated docs (no CLI help changed).

## Git workflow

- Shared branch; ONE commit:
  `docs(brew): document 1:1 Homebrew-compatible bootstrap`
- `git commit -s`; push to `fork`. After this plan, all eight plans are
  complete — open the single PR to `main` (never `release`) per the
  README's branch policy, including the corpus results file
  (`plans/007-corpus-results.md`) summary in the PR body and the
  AI-disclosure line required by AGENTS.md.

## Steps

### Step 1: Rewrite the contract sections

State plainly, replacing contradicting text:

- the engine installs Homebrew packages natively and produces on-disk
  state identical to real Homebrew, verified against the pinned Homebrew
  version (name the pin's source: the receipt module constant, without
  promising a specific number in prose);
- mixing `brew` and mise on the same machine is supported in both
  directions, including uninstall: either tool may install, list,
  upgrade, or uninstall regardless of which tool installed;
- a package already installed by Homebrew satisfies its mise declaration —
  status shows installed, apply changes nothing;
- apply ensures presence; upgrade is explicit and mirrors brew semantics —
  `auto_updates` casks are skipped (no implicit `--greedy`);
- prune removes packages exactly as `brew uninstall` would;
- for legacy mise-only casks (pre-1:1 installs), status only validates;
  apply converts provable state; invalid or unprovable state remains
  NeedsRepair without mutation and includes the one-line fix from plan 005;
- Homebrew installation is NOT required for the engine to work; when brew
  is present, both tools share state;
- `MISE_SYSTEM_BREW_PREFIX` is test-only; no custom-prefix support.

Remove every claim that mise-installed packages are unsupported by
`brew list`/`upgrade`, every mention of `.mise-cask.toml` as the ownership
record, and the "Homebrew owns this cask" error documentation.

**Verify**:
`grep -n "mise-cask.toml\|Homebrew owns this cask" docs/bootstrap/packages/brew.md`
→ no matches.

### Step 2: Verify links and lint

`grep -rn "packages/brew" docs/` — fix only links broken by heading
changes. Then `mise run lint`.

**Verify**: `mise run lint` → exit 0.

## Test plan

Docs-only; verification is the two grep gates plus lint.

## Done criteria

- [ ] Step 1 grep gate passes (no stale ownership-model text).
- [ ] Page accurately reflects the eight shipped behaviors listed above.
- [ ] Legacy status is documented as read-only validation; only apply may
      convert provable state, while invalid/unprovable state remains
      NeedsRepair without mutation.
- [ ] `mise run lint` exits 0.
- [ ] ONE commit; only in-scope files; `plans/README.md` row 008 updated.
- [ ] Single PR opened per Git workflow (all plans DONE).

## STOP conditions

- A behavior this page must document did not actually ship as specified
  (report the gap against the responsible plan instead of documenting the
  intent).
- The page documents features out of this effort's scope (taps without API
  metadata, services) in ways that now contradict shipped behavior in
  OTHER sections — report; do not expand scope silently.

## Maintenance notes

- When `EMULATED_BREW_VERSION` bumps, no docs change is needed (prose
  avoids the literal number by design).
- Reviewer focus: no overclaim — the page must not promise support for
  taps without published API metadata or `brew services`.

## Blocker resolution — 2026-08-12 PR documentation review

- **Condition:** legacy conversion wording implied status mutation and the
  coexistence promise did not explicitly require valid compatible state.
- **Evidence:** design decision 2 makes status read-only; corrupt or unprovable
  state is classified NeedsRepair without mutation.
- **Options:** retain ambiguous prose, weaken the implementation contract, or
  state validation-on-status and conversion-on-apply precisely.
- **Choice:** align design and user docs with the implemented read-only status
  boundary and qualify interoperability by valid Homebrew-compatible state.
