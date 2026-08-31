# Brew PR replacement plan

## Stop condition

Both reported failures are fixed on current main-derived branches; existing supported brew/brew-cask behavior remains; focused and ordinary project gates pass; each PR contains one reviewable concern; second PR diff excludes first after merge.

## Immediate owner action

1. Leave #11915 and #11910 draft. Both already report `isDraft: true`; no state change is needed.
2. Do not ask jdx to revisit either current diff. His comment identifies the artifact shape as unacceptable, not a missing explanation inside it.
3. Do not open empty replacement PRs. First get direction agreement, then build and verify the focused formula successor locally.
4. Once formula successor is focused and green, post one concise acknowledgement to jdx and link the replacement.
5. Keep cask successor local until formula successor merges; then rebase from new `main`, verify its isolated diff, and publish it.

Suggested acknowledgement when formula successor exists:

> Agreed. The current stack is too broad to review. I replaced it with a focused current-main formula lifecycle fix for OpenSSL/CA only. Native cask recognition will follow separately after this merges, with Homebrew ownership kept read-only unless explicitly adopted.
>
> *AI-assisted — Tool: Codex; model: openai/gpt-5; version: unavailable.*

## Phase 0 — preserve evidence

- Keep #11915 and #11910 draft.
- Keep successor work local until its focused diff and required tests pass. Do not create empty replacement PRs merely to reserve numbers.
- Tag/archive exact heads `97640cc` and `a1df6b7` for test/design salvage.
- Do not add more merge/retry/review-bot commits.
- Repair or recreate local `mise-formula-lifecycle` worktree: its `.git` points at missing `/Users/donbeave/Projects/donbeave/mise/.git/worktrees/mise-formula-lifecycle.C1IwAs`. Until repaired, use exact refs in the intact interop clone.
- Before implementation, post a short formula-lifecycle boundary proposal in GitHub Discussions or mise Discord. `docs/contributing.md:9-20` explicitly requires direction agreement for non-obvious work before review.

## Phase 1 — direction agreement and narrow prototype

Create a local prototype from current upstream `main`. Do not publish or request
review yet. Use it to prove the minimum boundary and whether existing primitives
are sufficient.

Direction proposal should fit one short post:

- Problem: a poured `brew:openssl@3` closure can look installed while required CA/OpenSSL shared lifecycle state is absent, so default TLS fails.
- Existing behavior to preserve: dependency closure, bottle verification, native receipts, links, source builds, and tap support.
- Proposed boundary: typed Rust preflight/apply/health/repair for only supported formula lifecycle operations; unsupported operations fail before mutation.
- Non-goals: source-build/sandbox overhaul, new recovery framework, SBOM, third-party trust redesign, or cask changes.
- Ask: confirm this boundary matches mise direction before replacement implementation proceeds.

Scope:

1. Preserve existing dependency-closure resolution/pour behavior; do not rewrite or claim it as new.
2. Preserve existing Homebrew-compatible receipt/link topology; extend only lifecycle health evidence.
3. Represent only required supported lifecycle operations as typed Rust values.
4. Preflight lifecycle for the complete already-resolved closure before mutation.
5. Install shared `ca-certificates`/OpenSSL config state transactionally.
6. Detect lifecycle damage and repair only mise-owned effects.
7. Fail closed before mutation for unsupported operations or ambiguous ownership.
8. Make `FetchMode::Fresh` truly reacquire official core metadata after an OCI identity miss.

Exclude:

- Third-party tap trust/JWS redesign unless existing current-main APIs cannot supply authenticated metadata.
- Linux source build overhaul, Ruby shim expansion, SBOM.
- Generic WAL/recovery framework beyond minimum existing transaction primitive.
- New command/process-group/file/sandbox frameworks.
- Cask changes.

Correctness gates:

- No hardcoded live formula/helper digest allowlist.
- Cached API/OCI incoherence followed by one fresh signed index succeeds; repeated incoherence terminates safely.
- Two supported `ca-certificates` metadata revisions pass same typed plan.
- Unknown semantic metadata fails closed; additive fields are tolerated only when an upstream schema contract proves they are non-semantic.
- Local TLS validates OpenSSL and Node default trust without environment overrides.
- Damage detection/repair and native `brew list/info/uninstall` proof.
- Existing formula, source, third-party tap, Linux, macOS, and Windows behavior unchanged unless explicitly scoped.

Prototype stop: focused tests pass and any prerequisite gap is demonstrated by
a concrete missing primitive, not preference.

## Phase 2 — optional independent prerequisites

Only create when Phase 1 proves existing primitives insufficient. Extract,
review, and merge these before publishing the formula successor:

- Descriptor-safe filesystem transaction primitive.
- Process-group execution primitive.
- Sandbox API extension.
- Formula recovery WAL.
- Authenticated third-party tap metadata.
- Source-build confinement/SBOM.

Each gets independent API contract, cross-platform tests, and no brew behavior bundled unless necessary to demonstrate consumer use.

## Phase 3 — formula lifecycle successor

Rebase/rebuild the Phase 1 prototype on current `main` after every required
Phase 2 prerequisite merges.

- Keep only formula lifecycle model/apply/health/repair, focused tests, and exact docs.
- Use small logical commits: model/parser, apply/state, integration, tests/docs. No merge/retry-only commits.
- Open draft only after focused diff and gates exist.
- Mark ready only after automated feedback is addressed.
- Merge before publishing final cask branch for review.

Suggested title: `fix(brew): complete OpenSSL certificate lifecycle`.

## Phase 4 — native cask recognition successor

Create from `main` after formula successor merges. Reuse #12074's adoption
configuration and receipt/self-update machinery; add an explicit producer-origin
model because #12074 does not provide one for native Homebrew state.

Scope:

1. Parse minimal native Homebrew cask receipt/metadata.
2. Validate version directory, artifacts/backlinks needed to establish healthy presence.
3. Report healthy Homebrew-owned cask as installed.
4. Make bootstrap no-op; do not acquire ownership.
5. Preserve mutation refusal for ambiguous/conflicting state.
6. Keep explicit adoption separate and unchanged.
7. Add exact `1password-cli`/essential-mac regression.
8. Preserve one backend-neutral platform-availability state consumed consistently by apply, status, doctor, and install hints.
9. Never prune or mutate Homebrew-owned state without explicit adoption.
10. Preserve producer origin in installed state so foreign upgrades no-op or refuse with adoption guidance.

Exclude:

- Formula changes.
- Aqua/task/lock/config/HTTP changes.
- Generic `PackageState` reinterpretation or backend-name checks.
- Apply-only availability remapping that makes other status consumers disagree.
- Removal of taps/resources/patches/macOS source builds.
- Third-party cask removal or formula-tap trust-policy changes.
- Unsafe libc sorting and byte-for-byte incidental receipt emulation.
- Broad cask DSL rewrite.

Commit shape: receipt parser, installed/no-op behavior, focused tests/docs.

Suggested title: `fix(brew): recognize Homebrew-owned casks`.

## Phase 5 — CI design

Required per-PR gates:

- Existing format/lint/unit/e2e project gates.
- Changed-path classification.
- One exact-head artifact per OS/architecture, reused by all same-platform brew jobs.
- Formula PR: one macOS OpenSSL lifecycle oracle; Linux case only for distinct source/bottle behavior.
- Cask PR: one macOS native ownership/no-op oracle including `1password-cli`.

Scheduled/manual:

- Baseline/current Homebrew differential corpus.
- Large formula/cask package matrix.
- Destructive canonical-prefix uninstall/cleanup matrix.
- Slow source-build/confinement matrix.

This preserves strong evidence without slowing unrelated backend changes.

## Phase 6 — completion audit

Before ready-for-review, prove every item:

- `git diff --stat upstream/main...HEAD` matches one concern.
- No unrelated changed files.
- No hardcoded mutable upstream recipe/helper identity.
- PR body credits existing closure/receipt behavior as prerequisite, not new work.
- No removed supported behavior in docs or tests.
- Status paths are read-only.
- Unsupported/ambiguous state fails before mutation.
- OpenSSL default CA/TLS works and repairs.
- Native Homebrew `1password-cli` makes `mise bootstrap` succeed without reinstall/ownership transfer.
- Native Homebrew `1password-cli` remains untouched by ordinary mise upgrade and prune until explicitly adopted.
- Existing mise-owned install/upgrade/prune remains valid.
- Apply, status, doctor, and install hints agree for platform-inapplicable casks.
- Focused gates and ordinary project gates pass.
- PR descriptions state exact tested boundary, head, and stacked order.
- Formula PR merged first.
- Cask branch rebased on merged main; GitHub diff contains only cask successor.

## GitHub cleanup

After successors exist and checks pass:

1. Create successor PRs as drafts only when their diffs are already focused and testable.
2. Link each old draft to its replacement.
3. Mark only formula successor ready first.
4. Prefer keeping the cask successor local until formula merges. A fork-only predecessor cannot be the base of a PR into `jdx/mise`; simultaneous publication would expose the combined diff unless a maintainer creates an upstream intermediate branch.
5. Close #11915 only after formula replacement is public and preserves discussion/history.
6. After formula successor merges, rebase/retarget cask successor to `main`, verify focused diff, then mark ready.
7. Close #11910 only after cask replacement is public.
8. Prefer one ready PR at a time for this sequence.
9. Never force a reviewer to compare old and new giant histories.

Current repository automation does not exempt drafts from age closure. Pinned
`jdx/pr-closer` v1.2.0 closes healthy PRs after 30 full days by default and does
not inspect draft state. Expect automatic closure around 2026-09-11 for #11910
and 2026-09-12 for #11915 if they remain open. Do not rely on draft status as
long-term archival storage; preserve exact heads locally.
