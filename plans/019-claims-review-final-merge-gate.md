# Plan 019: Reconcile claims, review state, and enforce the final merge gate

Status: TODO
Priority: P0
Effort: S
Planned against: combined SHA and evidence from plan 018
Depends on: 018

## Objective

Make documentation, plan state, PR bodies, reviews, and CI describe exactly the
proved support boundary. Only then convert both PRs from draft and recommend the
prerequisite merge order.

## Current defects

- Public docs say native state is differential-tested and either tool may
  install/upgrade/uninstall broadly.
- Plans 001–008 were marked DONE using zero-second false-green jobs.
- Six of eight old cask classes were explicitly unverified; mixed mechanisms
  were mislabeled.
- #11910 has unresolved GitHub threads and neither PR has human approval.
- No combined head or combined CI currently exists.

## Files and surfaces in scope

- `docs/bootstrap/packages/brew.md`
- `plans/README.md`
- plans 003–008 completion/invalidation sections
- `plans/007-corpus-results.md`
- PR #11910 and #11915 descriptions/checklists/review threads
- branch/check status, without merging the PRs

## Two-stage claim policy

### Stage A: immediately on implementation branch

Narrow current docs and PR text to “experimental/limited interoperability.”
List supported formula lifecycle and cask mechanism boundaries. State that
unknown lifecycle/artifact behavior fails closed. Remove “1:1”, “full”, “either
engine may,” or “differential-tested” claims not backed by plan 018.

Do this before feature work is presented as merge-ready. A temporary truthful
limitation is required; do not wait for final tests while overclaims remain.

### Stage B: after plan 018

Restore only claims directly supported by combined-head evidence. If any matrix
row is UNSUPPORTED, keep that limitation explicit. “All 37 names accounted for”
must not become “all Homebrew casks supported.” Formula interoperability must
state the tested lifecycle boundary and fail-closed behavior.

## Implementation steps

1. Update old plan statuses and completion sections: preserve historical work,
   link each invalidated claim to plans 009–018, and remove false PASS wording.
2. Replace `007-corpus-results.md` with plan 018's pinned mechanism matrix, job
   URLs, exact SHAs, fixture counts, and unsupported rows.
3. Update docs with operational guarantees only: receipts alone are insufficient;
   healthy state includes shared lifecycle and topology. Explain read-only status,
   safe repair, persistent config preservation, and fail-closed limitations.
4. Update both PR bodies with prerequisite order, combined SHA, exact Homebrew
   reference, plan 018 results, essential-mac regression, known limitations, and
   AI disclosure.
5. Resolve each #11910 review thread with a direct commit/evidence link or leave
   it unresolved and keep draft. Re-request human review after the final push.
6. Confirm branch protection/checks on the exact combined-dependent heads. Old
   individual green runs are not reused.
7. Mark plans 009–019 DONE only after their own done criteria are evidenced.
   Historical 001–008 may be “implemented and reverified,” never retroactively
   treated as independent merge proof.

## Final merge gate

All conditions are mandatory:

- #11915 exact accepted head contains plans 009–012 as applicable and passes its
  formula gates.
- #11910 is a clean descendant of that exact head and contains plans 013–019.
- Combined exact-head CI is green across Linux, macOS, Windows, lint, unit, e2e,
  Homebrew differential jobs, and completion-marker checks.
- No unresolved review threads; at least required human approvals are present.
- Worktrees are clean; all commits pushed; PR base/head SHAs match evidence.
- Homebrew pin remains current or drift has been re-audited.
- Docs and corpus matrix match actual supported/unsupported behavior.
- PR bodies contain current evidence and required AI disclosure.

Recommended merge order is #11915 first, then its exact descendant #11910. If
base-branch advancement changes either tested tree, rerun required gates on the
new exact heads before merge.

## Verification

```bash
rtk git status --short
rtk git log -1 --show-signature --format=fuller
rtk mise run lint
rtk mise run docs:build
```

Also inspect GitHub API/CLI state for draft flag, exact SHAs, checks, approvals,
and unresolved threads. This plan does not authorize pressing Merge.

## Done criteria

- Every public and PR claim is no stronger than plan 018 evidence.
- Every corrective plan has exact-head evidence.
- Both PRs satisfy the final merge gate and can be marked ready for human merge.

## Stop conditions

Do not mark ready with a skipped oracle, unresolved thread, absent human approval,
conflict, dirty/unpushed head, stale Homebrew pin, or documentation overclaim.
