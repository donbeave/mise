# Brew interoperability problem reference

Date: 2026-08-22

## Purpose

This document is the canonical problem statement for replacing mise PRs
[#11915](https://github.com/jdx/mise/pull/11915) and
[#11910](https://github.com/jdx/mise/pull/11910). It separates:

1. bugs present on mise `main`;
2. useful design ideas proven by the current PRs;
3. defects and regressions introduced by those PRs; and
4. review and delivery constraints.

The current PRs are evidence and prototypes. They are not suitable merge
candidates and must not become the specification for their replacements.

## Executive summary

There are two independent user-visible bugs.

### Formula lifecycle health is incomplete

Mise can pour a formula and its dependency closure, write a native-compatible
receipt, and create expected keg links, yet still omit required post-install or
shared lifecycle state. The clearest case is `ca-certificates` in an OpenSSL
closure: mise reports the formula installed while default TLS trust is broken.

The existing installed predicate is incomplete. It treats an existing keg and
matching active link as sufficient health evidence. Correct health must also
cover required supported lifecycle effects and their repair state.

### Homebrew-owned casks are mistaken for conflicts

A healthy cask installed by Homebrew, including `1password-cli`, contains native
Homebrew metadata. Current mise checks that metadata before classifying installed
state and aborts with `Homebrew owns this cask`. A shared configuration such as
`"brew-cask:1password-cli" = "latest"` therefore cannot converge when Homebrew
already installed the cask.

The existing state model conflates presence with mise ownership. Correct
behavior must recognize healthy foreign state as installed while preserving its
producer and refusing mutation until explicit adoption.

These bugs share a Homebrew interoperability theme, but they do not share one
root cause. They should not be solved by one broad Homebrew compatibility layer.

## Current PR status

Verified from GitHub on 2026-08-22:

| PR | Head | GitHub diff | State | Checks | Review result |
|---|---|---:|---|---|---|
| #11915 | `97640cc04359fa6488706ac46bf71a46cf840b3b` | 38 files, `+26,130/-1,131` | Open draft, mergeable | 23 successful, 0 failed or pending | No approval or review decision |
| #11910 | `a1df6b7fd11c2a7a1c7bb802f9a503f98e122496` | 61 files, `+46,752/-8,111` | Open draft, mergeable | 33 successful, 0 failed or pending | No approval; maintainer refused current review shape |

Local exact-ref history shows #11915 contains 209 commits from its merge base.
#11910 contains #11915 as an ancestor and adds another 27 commits. Its cask-only
delta is still 51 files and `+21,127/-7,168`.

Maintainer feedback on #11910 is decisive:

> I don't understand what this is for but a 40k line pull request is way outside the realm of something I'd be willing to review

Green CI and mechanical mergeability do not answer this objection. They prove
only that the exact large heads pass their configured checks.

## Problem 1: formula lifecycle

### Observed failure

A formula dependency closure can be present and linked, but OpenSSL or another
consumer cannot use required default shared state. For the reported case,
OpenSSL/Node TLS trust fails unless external environment overrides compensate
for missing CA/configuration state.

### Root cause on the original base

The original backend already performed two important tasks:

- resolved and poured the complete dependency closure; and
- wrote a Homebrew-compatible `INSTALL_RECEIPT.json` and link topology.

Those are existing prerequisites, not contributions of #11915.

The enabling defect is the health invariant. `keg_installed()` checks only:

```text
keg exists AND active opt record matches requested version
```

It does not check required post-install effects, shared `etc` state, CA state,
or provenance needed for safe repair. This permits a false healthy result.

### Required invariant

Formula health should mean:

```text
verified artifact identity
+ healthy dependency closure
+ supported lifecycle effects
+ native receipt
+ expected links and topology
```

Absence, damaged mise-owned state, unsupported lifecycle semantics, and
ambiguous ownership must remain distinct states.

### Useful prototype ideas to retain

- Compile a typed lifecycle plan before mutation.
- Preflight the complete already-resolved closure.
- Include lifecycle state in installed/healthy classification.
- Distinguish repairable damage from package absence.
- Repair only effects whose mise ownership and expected identity are proven.
- Fail before mutation when required semantics are unsupported.

### #11915 defects that must not become requirements

These are defects in the proposed implementation, not the original user bug:

- Hardcoded accepted hashes for live `ca-certificates` formula/helper revisions
  create predictable outages after legitimate Homebrew updates.
- `FetchMode::Fresh` is discarded for official core metadata, while a process
  cache keeps returning the stale index. The advertised metadata-recovery retry
  therefore cannot refresh its input.
- Strict parsing of an internal signed schema can make routine additive metadata
  changes reject all formula installs without a proven schema contract.
- A pinned Homebrew 6.0.17 oracle proves one runtime only, not broad Homebrew
  compatibility.
- A live `auth.kimi.com` HTTP-status assertion introduces unrelated external
  flakiness into TLS proof.
- Formula lifecycle work is bundled with source-build expansion, sandbox and
  process frameworks, filesystem transactions/WAL, JWS/tap trust, SBOM, and
  large CI infrastructure.
- Existing support for resources, external patches, and macOS source builds is
  removed as collateral behavior.

### Formula acceptance conditions

A replacement is complete only when it proves:

1. the reported OpenSSL/CA closure works with default trust and no environment
   override;
2. lifecycle damage is detected and safely repaired when mise owns the effects;
3. unsupported or ambiguous lifecycle state fails before closure mutation;
4. at least two equivalent supported metadata revisions compile to the same
   typed semantics without a mutable live-recipe allowlist;
5. stale cached metadata can be replaced once, while repeated incoherence
   terminates without recursion or partial mutation, if the replacement uses
   that metadata path;
6. native `brew list`, `brew info`, and uninstall interoperability remains;
7. existing formula closure, receipt, bottle, source, tap, and platform behavior
   remains unless a separate approved change explicitly modifies it; and
8. TLS proof uses a local certificate chain/server.

## Problem 2: native Homebrew cask ownership

### Observed failure

Given a healthy Homebrew-installed `1password-cli` and a mise configuration that
requests `brew-cask:1password-cli`, bootstrap aborts instead of reporting the
requirement satisfied.

### Root cause on current main

Current install flow checks for Homebrew `.metadata` before installed-state
detection. The metadata is treated as a mutation conflict even when no mutation
is necessary.

The state model also lacks producer origin. A version-only `Installed` state
cannot distinguish:

- mise-installed and mise-owned;
- Homebrew-installed and Homebrew-owned; or
- ambiguous/conflicting state.

Without this distinction, recognition can accidentally grant upgrade or prune
authority.

### Required state and ownership invariant

Presence and mutation authority are separate dimensions:

| State | Status/bootstrap | Upgrade/prune |
|---|---|---|
| Healthy mise-owned | Installed/no-op | Allowed under existing rules |
| Healthy Homebrew-owned | Installed/no-op | No-op or refuse with adoption guidance |
| Explicitly adopted | Installed/no-op | Allowed under existing adoption contract |
| Damaged or ambiguous foreign state | Conflict/repair guidance | Refuse before mutation |
| Platform-inapplicable | Unavailable/skip | No mutation |

Read-only status must never convert receipts, create links, delete files, or
transfer ownership.

### Useful prototype ideas to retain

- Parse the minimal native Homebrew cask receipt/metadata.
- Validate the version directory and artifact/backlink topology needed to prove
  healthy presence.
- Return installed/no-op for healthy native state.
- Preserve a producer-origin value in installed state.
- Exercise the exact `1password-cli` bootstrap configuration.

### #11910 defects that must not become requirements

- Homebrew-owned casks can be removed by ordinary mise prune without explicit
  adoption.
- Producer origin is erased after classification, so version drift can let
  ordinary upgrade replace a Homebrew-owned cask.
- Third-party casks are rejected even though that change is unrelated to native
  recognition; docs also inaccurately claim formula-tap support was removed.
- Platform-inapplicable casks become `Missing` in status/doctor/install hints,
  while apply alone remaps them through a `brew-cask` name special case.
- Generic package-state semantics are changed to carry backend-specific policy.
- Unsafe libc `qsort` is used to imitate incidental receipt byte ordering.
- Cask work is bundled with Aqua, task, lock, HTTP, formula, ELF, and global
  configuration changes.
- Thirteen brew-specific CI cells are added to mandatory global CI.

### Cask acceptance conditions

A replacement is complete only when it proves:

1. a valid native receipt plus required topology reports installed;
2. exact Homebrew-installed `1password-cli` bootstrap succeeds without reinstall
   or ownership transfer;
3. status is read-only;
4. ordinary upgrade and prune leave Homebrew-owned state untouched;
5. explicit adoption still uses the existing adoption contract;
6. mise-owned casks remain installable, upgradable, and prunable;
7. malformed or ambiguous native state refuses mutation with precise guidance;
8. platform availability has one backend-neutral meaning consumed consistently
   by apply, status, doctor, and install hints;
9. current third-party cask behavior remains; and
10. receipt serialization is safe and deterministic where serialization is
    required.

## Why current PRs cannot be salvaged by explanation

Three separate failures exist.

### Review surface failure

Both diffs contain many independent architectural changes. A reviewer cannot
establish which changes are necessary for either reported bug or verify their
cross-platform blast radius as one unit.

### Correctness failure

The PRs introduce rolling metadata outages, ineffective cache refresh,
ownership-transfer paths, inconsistent status semantics, and collateral support
removal. These remain blockers even if diff size were accepted.

### Presentation failure

#11910 targets `main` while embedding #11915. Its title describes broad formula
and cask interoperability although its intended second concern is native cask
recognition. This hides both the requested behavior and the review boundary.

More comments or retry commits do not change any of these conditions. Replacement
artifacts are required.

## Replacement boundaries

### Formula replacement

One concern: OpenSSL/CA lifecycle health and repair for poured formulae. Preserve
existing resolution, receipt, link, source-build, and tap behavior. Split any
new generic prerequisite only when a focused prototype proves current primitives
insufficient.

Suggested title: `fix(brew): complete OpenSSL certificate lifecycle`.

### Cask replacement

One concern: recognize healthy Homebrew-owned casks as installed/no-op while
preserving producer ownership across status, apply, upgrade, and prune. Reuse
current-main adoption machinery instead of replacing it.

Suggested title: `fix(brew): recognize Homebrew-owned casks`.

### Ordering clarification

The current cask branch depends on #11915 because it was built as a stack. That
does not prove the reduced cask product fix depends on the formula lifecycle
fix. A fresh current-main cask successor may be technically independent.

Before enforcing formula-first delivery, verify whether the focused cask change
needs any formula successor API. If no dependency exists, sequencing is a review
choice, not an architecture requirement. Never carry the old stack dependency
into new branches by default.

## CI boundary

Per replacement PR:

- normal format, lint, unit, and e2e gates;
- focused Rust tests for state and ownership invariants;
- one exact-head artifact per OS/architecture, reused by same-platform jobs;
- one path-gated macOS oracle for the exact reported behavior; and
- Linux coverage only where behavior differs.

Broad Homebrew differential corpora, destructive canonical-prefix matrices, and
slow source-build/confinement tests belong in scheduled or manual workflows
unless the focused diff directly changes those paths.

## Open design decisions

These are not settled by the current PRs:

1. Which authenticated, stable metadata source supplies formula lifecycle
   semantics without evaluating arbitrary Ruby or pinning mutable live recipes?
2. What exact typed lifecycle operations are supported initially?
3. Which existing transaction primitive is sufficient for shared CA/config
   state, and what concrete gap would justify a new generic primitive?
4. Where should producer origin live so all package-state consumers preserve it
   without backend-name checks?
5. Should foreign cask upgrade/prune be silent no-op or explicit refusal with
   adoption guidance?
6. Are formula and cask successors technically independent on current main?

Direction agreement should settle these questions before another large
implementation starts.

## Source documents

- [Formula lifecycle audit](PR-11915-FORMULA-LIFECYCLE-AUDIT.md)
- [Native cask interoperability audit](PR-11910-BREW-CASK-INTEROP-AUDIT.md)
- [Review summary](PR-REVIEW-SUMMARY.md)
- [Replacement plan](BREW-PR-REPLACEMENT-PLAN.md)

Those documents retain detailed changed-file classifications and execution
history. This reference owns the stable problem definition and acceptance
boundary.
