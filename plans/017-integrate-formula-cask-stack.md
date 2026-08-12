# Plan 017: Build one semantically integrated formula/cask stack

Status: TODO
Priority: P0
Effort: M
Planned against: #11910 `05ccd7ab8`, #11915 `b94b6b1c1`
Depends on: 012, 014, 015, 016

## Objective

Produce one mergeable head containing all formula and cask corrections. Resolve
the existing conflicts around shared abstractions, then run non-destructive
local gates before the full plan 018 oracle.

## Current defect

Both PRs target `main`. Merging #11915 then #11910 conflicts in:

- `src/system/packages/brew/api.rs`
- `src/system/packages/brew/pour.rs`
- `src/system/packages/brew/source.rs`

Those files own lifecycle metadata, receipt provenance, and finalization order.
Separate exact-head green checks cannot prove the manual conflict resolution or
the combined product.

## Prerequisite branch policy

1. Complete formula plans 010–012 on #11915's branch and obtain its exact
   reviewed head.
2. Complete independent cask plans 013–016 on #11910's branch without copying
   formula code around the conflict.
3. Rebase #11910 onto the exact accepted #11915 head. Do not merge `main` into
   either branch as a substitute for dependency ordering.
4. Record old heads, new prerequisite head, rebase base, and resulting combined
   SHA in both PR bodies and this plan's completion section.

## Semantic conflict rules

- `api.rs`: one authoritative formula API type supplies plan 010's typed
  lifecycle preparation and plan 011's provenance. Cask API changes remain
  orthogonal. No duplicated raw lifecycle fields.
- `pour.rs`: one state machine distinguishes provenance, prepares lifecycle
  before mutation, finalizes once, and supports lifecycle-only repair. Preserve
  #11910 receipt/SBOM/linked-keg behavior and #11915 etc/var/post-install behavior.
- `source.rs`: source builds write verified formula snapshots and enter the same
  finalizer. They never take bottle provenance and never skip lifecycle.
- Shared receipt/version constants have one owner. Do not retain two emulation
  pins or two definitions of installed health.
- All new enums are exhaustively matched. Do not silence conflicts with fallback
  branches, `unreachable!`, or duplicated legacy paths.

## Implementation steps

1. Before rebase, run focused formula and cask unit suites on their respective
   heads and archive results with exact SHAs.
2. Rebase and resolve the three known conflicts deliberately. Search for all
   remaining conflict markers and duplicated symbols/state paths.
3. Trace each operation end-to-end on the combined tree:
   OCI bottle, archive bottle, source formula, lifecycle-only repair, fresh app
   cask, cask upgrade, cask prune, real-brew cask adoption.
4. Verify status stays read-only and no production call invokes `brew`.
5. Run formatting, Clippy, unit tests, focused e2e, Windows compilation/tests,
   and source invariant greps. Fix only regressions caused by this stack; record
   unrelated baseline failures exactly.
6. Push combined head and ensure #11910 explicitly depends on the new #11915
   exact head. Individual old CI is historical only.

## Verification

```bash
rtk git diff --check
rtk rg -n '^(<<<<<<<|=======|>>>>>>>)' src e2e .github plans docs
rtk cargo test --bin mise system::packages::brew
rtk cargo clippy --workspace --all-features --all-targets -- -D warnings
rtk mise run test:e2e e2e/cli/test_system_install_brew_linux
rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow
rtk mise run test:e2e e2e/cli/test_system_install_brew_formula_lifecycle_macos_slow
rtk mise run lint
```

Use the repository's normal Windows gate. Never execute destructive oracle
bodies locally; plan 009 guards must stop them before mutation.

## Required invariants

- No production `brew` process execution.
- Formula Installed means complete finalization/health, not receipt shape.
- Cask Installed means complete artifact topology and committed journal.
- Every mutation plan is validated fully before its first side effect.
- Every receipt field has authoritative provenance.
- Status cannot create, repair, relink, download, or lock.

## Done criteria

- #11910 is a clean descendant of the exact accepted #11915 head.
- `api.rs`, `pour.rs`, and `source.rs` have one coherent architecture.
- A single combined SHA is pushed and all non-destructive/focused gates pass.
- No merge conflict or duplicated fallback behavior remains.
- Plan 018 references this combined SHA as its only test subject.

## Stop conditions

Do not accept an ours/theirs resolution without tracing both PR invariants. Do
not call either PR merge-ready yet; combined operational oracle proof belongs to
plan 018.
