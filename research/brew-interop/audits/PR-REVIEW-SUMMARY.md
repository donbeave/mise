# Brew PR review summary

Date: 2026-08-22

## Refresh verification

Rechecked against GitHub and freshly fetched `origin/main` at
`6f52dcdf99e282ef7a7db68c81301fa4618d0f79`.

- PR heads remain `97640cc` (#11915) and `a1df6b7` (#11910).
- Both remain open drafts and mechanically mergeable. `gh pr checks` reports 23 passed/0 failed for #11915 and 33 passed/0 failed for #11910. GraphQL reports 38/38 and 30/30 review threads resolved, respectively.
- That does **not** by itself satisfy review gates: #11910 retains a top-level Codex P1 that GitHub does not count as a review thread. Current head patches its exact apply failure, but related status/doctor semantics remain inconsistent; any reduced successor needs fresh full review. Neither PR has an approval or `reviewDecision`; #11915 has no human maintainer review, while #11910 has jdx's explicit scope refusal.
- #11915 still merges from `f000a690`; #11910's current merge base with `main` is `7a94d1b47` after upstream synchronization.
- Current main's #12074 adoption work and #12222 content-drift fix still do not recognize a healthy Homebrew-owned `.metadata` installation. Main still raises `Homebrew owns this cask` before installed-state detection.
- The broken `mise-formula-lifecycle/.git` worktree pointer remains. Formula evidence was verified through the intact interop clone's exact `origin/pr-11915` ref.

Coverage includes every changed file and all goal-facing, architecture, CI,
compatibility, and regression hunks against authoritative merge bases.
Per-file tables classify the full diff; decisive findings carry exact line
citations. The formula diff contains 485 hunks; the cask-only second delta
contains 1,403. Repeating one row for every added line would not add stronger
evidence.

## Decision

- PR #11915: **draft; replace or split before review**.
- PR #11910: **draft; replace after #11915's reduced successor merges**.
- Both PRs remain draft. Verification: GitHub reports `isDraft: true` for both; no state mutation was needed in this continuation.
- Do not request another maintainer review on current branches.
- No empty successor PR was created. Keep both current PRs draft until focused replacement branches and their required evidence exist; then expose only the formula successor for review first.

This conclusion is not based on effort or perceived value. Current changes fail correctness, compatibility, reviewability, and goal-fit requirements.

## Decisive evidence

| PR | Current head | Base | Size | Commits | Result |
|---|---|---|---:|---:|---|
| #11915 | `97640cc04359fa6488706ac46bf71a46cf840b3b` | `main` | +26,130/-1,131; 38 files | 209 | Clean/green, but unsafe update coupling and excessive scope |
| #11910 | `a1df6b7fd11c2a7a1c7bb802f9a503f98e122496` | `main` | +46,752/-8,111; 61 files | 232 total | Clean/green, but maintainer refused current review shape |

Maintainer statement on #11910:

> I don't understand what this is for but a 40k line pull request is way outside the realm of something I'd be willing to review

Source: <https://github.com/jdx/mise/pull/11910#issuecomment-5364856639>

PR #11915 adds 26k lines and 209 commits for an OpenSSL lifecycle defect. Its core model is directionally correct, but current implementation adds several independent security, metadata, sandbox, source-build, recovery, and CI systems. It is not a minimum fix.

PR #11910's second-stack delta is still +21,127/-7,168 across 51 files. `cask.rs` alone changes +16,420/-6,499. Its ownership recognition solves the reported `1password-cli` failure structurally, but current PR also removes third-party cask support, inaccurately documents third-party formula support as removed, and changes generic package-state semantics.

## What jdx's comment requires

The rejection is about both comprehension and review surface. More explanation on the existing
40k-line PR cannot fix either. Required response is a different artifact:

#11910's broad title (`feat(brew): interoperate with Homebrew formulae and
casks`) accurately reflects its combined `main` diff, but hides the intended
second-stack cask fix. That presentation directly creates the confusion jdx
reported.

1. Do not defend or reopen the current stack for review.
2. Agree on the narrow formula lifecycle boundary in Discussion/Discord before more code.
3. Build one current-main formula successor whose title, body, tests, and diff all describe only OpenSSL/CA lifecycle health and repair.
4. Merge it first.
5. Build the cask successor from that merged `main`; show only native Homebrew ownership recognition/no-op and ownership guards.
6. Request review only after focused diffs, ordinary CI, focused oracles, and automated review are clean.

Copy-ready acknowledgement, but post only when successor direction/work exists:

> Agreed. The current stack is too broad to review. I am keeping both PRs draft and replacing them with focused current-main changes: first formula lifecycle health/repair for OpenSSL/CA only; then, after that merges, native Homebrew cask recognition/no-op with ownership preserved. I will not request review until each diff is small, isolated, and fully tested.
>
> *AI-assisted — Tool: Codex; model: openai/gpt-5; version: unavailable.*

## User problems

### OpenSSL installed by `mise brew`

Current #11915 proves one exact dependency closure and lifecycle at its pinned head. Important correction from the prior report: its merge-base already resolved and poured the full dependency closure (`src/system/packages/brew/mod.rs:87,170-190`) and already wrote a Homebrew-compatible receipt (`src/system/packages/brew/pour.rs:299-375`). Those are prerequisites, not contributions of this PR. Actual defect is that base `keg_installed()` accepted only keg + active-link topology (`pour.rs:31-52`) and never checked required post-install/shared state. The PR correctly introduces lifecycle health and repair, but bundles that fix with many independent systems.

It does **not** provide sustainable compatibility. `src/system/packages/brew/lifecycle.rs` pins accepted `ca-certificates` recipe/helper hashes. A legitimate upstream formula update becomes unsupported until mise ships another audited pin. That replaces incomplete OpenSSL installs with predictable future install outages.

Its advertised stale-metadata recovery is also ineffective for official core.
`formula_with_mode()` discards `FetchMode` (`src/system/packages/brew/api.rs:441-443`),
while `internal_formula()` always reuses one process-wide `OnceCell`
(`:630-632,734-760`). The OCI identity-miss retry requests `Fresh`
(`src/system/packages/brew/mod.rs:244-268`) but receives the same cached index.
This is a concrete correctness bug, not only excess scope.

Required replacement: generic typed lifecycle support derived from authenticated metadata, bounded to supported operations, with fail-before-mutation for unsupported operations. Test at least two supported formula revisions. Use local TLS proof; do not depend on `auth.kimi.com` returning HTTP 405.

### Homebrew-owned `brew-cask:1password-cli`

Current #11910 fixes root bug class: installed state is inferred from shared native Caskroom metadata, not only a mise-private receipt. Valid native install becomes `Installed`, so bootstrap becomes no-op rather than throwing ownership error.

It also introduces a separate cross-platform status regression. `installed()` now reports an absent platform-inapplicable cask as `Missing`; only apply later converts it through a `name == "brew-cask"` special case. `status`, doctor, and install hints consume the uncorrected state, so a shared macOS/Linux config can be skipped by apply on Linux yet still be falsely reported missing. Availability classification must remain backend-neutral and consistent for every caller.

The broad oracle also proves behavior beyond the requested no-op: after Homebrew
installs a cask, mise prune removes that Homebrew-owned state. Native read-side
recognition is required; automatic takeover/removal is not. A reduced successor
must keep foreign ownership read-only unless user invokes an existing explicit
adoption contract.

Upgrade has the same ownership flaw. `InstalledCaskState::Installed` stores only
version, erasing whether Homebrew or mise owns it. Version drift bypasses the
no-op and feeds the native receipt into predecessor teardown, so
`mise bootstrap packages upgrade` can replace a Homebrew-owned cask without
adoption. Ownership origin must survive classification; foreign upgrade must
no-op or refuse until explicit adoption.

Current main's merged adoption feature (#12074) does not fully solve this case: `cask.rs` rejects existing `.metadata` before installed detection. Minimum correction on current main:

1. Validate native Homebrew Caskroom metadata and topology in `installed()`.
2. Return installed/no-op for healthy Homebrew ownership.
3. Retain install refusal when mutation would take over foreign ownership.
4. Do not convert or rewrite Homebrew receipts during read-only status.
5. Add exact `1password-cli` bootstrap regression using the essential-mac configuration shape.

#12074 provides adoption configuration, receipt fields, and self-update
behavior; it does not provide a producer-origin model for native Homebrew
state. Successor must add that distinction explicitly.

This should require a focused cask change, fixture/unit coverage, and one narrow bootstrap e2e—not a 24k-line cask subsystem.

## Direction fit

Positive, preserve:

- Rust-first direct installation; no runtime `brew` dependency.
- Typed lifecycle plan before mutation.
- Formula health includes artifact identity, dependency closure, shared lifecycle effects, receipt, links, and topology.
- Existing closure resolution and native receipt writing remain intact; do not present them as new work.
- Read native Homebrew state for interoperability.
- Fail before mutation when operation or ownership is unsupported.
- Repair state distinct from healthy installed state.

Negative, remove or split:

- Partial reimplementation of Homebrew internals in 9.4k-line lifecycle and 24k-line cask modules.
- Live recipe/helper digest allowlists embedded in mise.
- Formula fix coupled to JWS clients, third-party tap trust, SBOM, source-build DSL, WAL, process groups, descriptor filesystem APIs, Landlock, and seccomp.
- Cask fix coupled to Aqua, task config, lock, HTTP, and generic package-state behavior.
- #11910 removes third-party cask support while still resolving configured GitHub formula taps from immutable commits; docs incorrectly say both are rejected. Same-repository commit pinning gives immutability, not an independent trust root.
- Cask apply/status semantics disagree across callers because availability is remapped only in the apply driver.
- Brew-specific mandatory CI matrix with up to 120-minute jobs on unrelated changes.
- Unsafe libc `qsort` for receipt ordering.
- #11915 removes formula resources/external patches and macOS source builds; #11910's own delta removes third-party casks and changes formula-tap requirements. All are collateral behavior changes.

Recent accepted cask work shows maintainer-compatible shape: #11962 changed 2
files (`+3,415/-1,167`), #11963 3 files (`+1,294/-16`), #11964 2 files
(`+738/-17`), #12074 15 files (`+723/-89`), and #12222 2 files
(`+155/-20`). Direction is Rust-native and incremental. Current monolithic
presentation, not Rust-first interop itself, conflicts with that pattern.

## Stack result

`97640cc` is an ancestor of `a1df6b7`. Both merge-tree checks are conflict-free. Mechanical stack order is valid.

Presentation is not valid: both PRs target `main`, so #11910 exposes #11915 plus its own changes. The predecessor branch exists only in the contributor fork, so it cannot be selected as the base of a PR into `jdx/mise`; true simultaneous stacking would require a maintainer-owned intermediate branch. Publish the reduced cask successor only after formula merges, then branch/rebase it from new `main`.

## CI result

All current GitHub checks are green. This proves exact fixtures at exact heads only. It does not prove minimality, future Homebrew compatibility, absence of removed behavior, or maintainer reviewability.

Minimum required CI:

- Focused Rust unit tests for typed lifecycle/state/receipt invariants.
- One exact-head artifact per OS/architecture, reused by all same-platform brew jobs.
- One path-gated macOS OpenSSL/CA lifecycle and repair oracle using local TLS.
- One focused native-cask ownership/bootstrap no-op oracle including `1password-cli`.
- Linux bottle/source coverage only where platform behavior differs, integrated into existing e2e matrix.
- Broad differential Homebrew corpus as scheduled/manual validation, not mandatory for unrelated PRs.

## Final recommendation

Do not salvage current PRs by adding more commits. First settle reduced formula
lifecycle boundary in a short Discussion/Discord proposal, as required for
non-obvious direction by `docs/contributing.md:9-20`. Preserve useful tests and
design invariants, then recreate small branches from current main. Merge formula
lifecycle first. Publish/rebase native cask recognition second. Full sequence
lives in `BREW-PR-REPLACEMENT-PLAN.md`.
