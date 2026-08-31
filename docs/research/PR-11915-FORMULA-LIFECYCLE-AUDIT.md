# PR #11915 — formula lifecycle audit

PR: <https://github.com/jdx/mise/pull/11915>  
Head: `97640cc04359fa6488706ac46bf71a46cf840b3b`  
Base merge point: `f000a6906231d36124be9758ae30ec294951fcfd`  
State after audit: draft

## Refresh verification

Verified again with fetched upstream main `6f52dcdf9` and exact PR ref
`97640cc`. GitHub reports open/draft/mergeable; `gh pr checks` reports 23
passed/0 failed and GraphQL reports 38/38 review threads resolved. Diff remains 38 files,
`+26,130/-1,131`, 485 hunks, and 209 commits including 50 merge commits from
merge-base `f000a690`.

The named local worktree is not a usable Git worktree: its `.git` file points at
missing parent metadata. All source and history evidence below comes from the
intact interop clone's exact `origin/pr-11915` ref. This does not affect GitHub
head evidence, but should be repaired before replacement implementation.

## Verdict

Request changes. Recreate as several independently reviewable PRs. Current exact-head tests are green, but core compatibility and scope requirements are not satisfied.

jdx's #11910 refusal applies to this prerequisite's presentation too: #11915 is
already 26k added lines and 209 commits. A second explanatory comment cannot
make that diff understandable. Agree on the lifecycle boundary first, then
replace this branch with a current-main successor containing one concern.

## Root cause and correct boundary

Original backend already resolved and poured the full dependency closure (`f000a690:src/system/packages/brew/mod.rs:87,170-190`) and already wrote a Homebrew-compatible `INSTALL_RECEIPT.json` (`f000a690:src/system/packages/brew/pour.rs:299-375`). Prior report incorrectly credited those capabilities to this PR.

Actual bug-enabling invariant was narrower: `keg_installed()` treated an existing keg plus matching active `opt` record as healthy (`f000a690:src/system/packages/brew/pour.rs:31-52`). It did not model or validate post-install effects, shared `etc`/CA state, or lifecycle repair provenance. `ca-certificates` could therefore look installed while OpenSSL default trust remained unusable.

Correct invariant:

`healthy = verified artifact identity + healthy dependency closure + supported lifecycle effects + native receipt + expected links/topology`

Closure and receipt terms already existed. Minimum goal change is to add lifecycle planning, application, health, and repair without replacing those working subsystems.

Good structural pieces:

- `src/system/packages/brew/mod.rs:104-120`: repair-only path distinguishes damage from absence.
- `src/system/packages/brew/mod.rs:135-145,369-384`: compile/preflight lifecycle before mutation.
- `src/system/packages/brew/mod.rs:409-428`: installation state includes lifecycle health.
- `src/system/packages/brew/lifecycle.rs:77-111`: typed operations instead of arbitrary Ruby execution.

These ideas should survive. Current surrounding implementation should not.

## Blocking correctness findings

### PR description overclaims existing behavior

The PR body lists complete dependency-closure resolution and native receipt/link
topology as changes. Both existed at the merge-base. This weakens reviewer trust
and hides the actual delta. Any successor description must distinguish existing
prerequisites from new lifecycle semantics and claim only behavior proved by its
focused diff.

### Live upstream hashes create rolling outage

`src/system/packages/brew/lifecycle.rs:25-32,59-71,3162-3170` allowlists one `ca-certificates` formula revision and helper digest. Normal Homebrew updates invalidate the allowlist. OpenSSL dependency installation then fails until a new mise release adds a new pin.

Required:

- Derive generic supported lifecycle operations from authenticated metadata.
- Bind operation inputs to verified bottle/formula identity.
- Reject unsupported operations before any closure mutation.
- Test two different signed formula revisions with equivalent supported semantics.
- If generic support is impossible, package a stable versioned helper whose identity is tied to bottle metadata; do not pin mutable live formula source in mise code.

### Fresh metadata retry reuses stale core index

`src/system/packages/brew/api.rs:441-443` explicitly discards `FetchMode` for
official core. `internal_formula()` caches the signed index permanently in
`INTERNAL_FORMULAE` (`:630-632,734-760`). When OCI descriptor identity and API
metadata disagree, `src/system/packages/brew/mod.rs:244-268` recursively retries
with `FetchMode::Fresh`, but the retry cannot fetch a new core index.

Required:

- Make core `Fresh` bypass or atomically replace the process cache.
- Prove cached incoherence followed by a changed signed index succeeds once.
- Prove repeated incoherence fails without recursion or partial mutation.

### Internal schema is too brittle

`src/system/packages/brew/api.rs:487-579` models Homebrew internal signed schema with strict unknown-field rejection. Routine additive upstream metadata can reject all formula installs. Embedded trust material also needs rotation behavior.

Required:

- Prefer a stable public source/schema when it carries required lifecycle data.
- Otherwise define an explicit upstream schema/version contract. Unknown semantic fields must fail closed; tolerate an additive field only when upstream evidence proves it cannot affect install semantics.
- Define named trust-anchor source, overlap, rotation, and revocation behavior; test key transition rather than an unspecified fallback.
- Preserve signed fields used for identity and policy without accepting ambiguous variants.

### Oracle overclaims compatibility

The macOS oracle job at `.github/workflows/test.yml:188-239` pins Homebrew 6.0.17 and its runtime SHA. Passing it proves compatibility with one historical runtime, not Homebrew generally. Similar values in Linux jobs are provenance sentinels; those jobs do not install a Homebrew runtime.

`e2e/cli/test_system_install_brew_formula_lifecycle_macos_slow:79-83` depends on live `auth.kimi.com` and exact HTTP 405. Network/application behavior can fail unrelated PRs.

Required:

- Test current stable Homebrew plus a pinned minimum/fixture schema.
- Use local TLS server/certificate chain to prove default OpenSSL and Node trust.
- Narrow docs and PR claims to supported/tested lifecycle semantics.

## Changed-file audit

Classification: **keep** = required concept; **reduce** = retain only narrowed goal-facing behavior; **split** = valid independent work; **remove** = not required for OpenSSL goal; **replace** = concept needed but implementation should be redesigned; **conditional** = retain only if reduced code needs it; **test-only** = retain only with reduced implementation. Compound labels apply both actions.

| File | Delta | Classification | Audit |
|---|---:|---|---|
| `.github/workflows/test.yml` | +210/-1 | reduce | Three dedicated builds/jobs duplicate shared CI and run on `full-ci`; use path-gated shared artifacts/matrix. |
| `Cargo.lock` | +1 | conditional | Keep only dependency required by reduced typed lifecycle. |
| `Cargo.toml` | +1 | conditional | Same; no dependency for split subsystems. |
| `docs/bootstrap/packages/brew.md` | +11/-9 | keep | Document exact supported semantics and failure boundary; remove broad parity claims. |
| `e2e/cli/brew_oracle_guard.sh` | +123 | split/test | Useful destructive-test guard; reusable test-infrastructure PR if still needed. |
| `e2e/cli/test_brew_oracle_env_sanitization` | +8 | split/test | Keep with guard infrastructure, not formula behavior unless directly needed. |
| `e2e/cli/test_brew_oracle_guard` | +39 | split/test | Same. |
| `e2e/cli/test_system_install_brew_formula_lifecycle_macos_slow` | +176 | keep/rewrite | Keep OpenSSL/CA install, repair, native recognition/uninstall. Replace live endpoint. |
| `e2e/cli/test_system_install_brew_linux` | +59/-7 | reduce | Keep only Linux bottle behavior that differs from macOS. |
| `e2e/cli/test_system_install_brew_source_slow` | +40/-4 | split | Source build is separate scope. |
| `e2e/run_all_tests` | +12 | remove/split | Global harness change not intrinsic to formula lifecycle. |
| `e2e/run_test` | +58/-3 | remove/split | Global harness/oracle mechanics belong test infrastructure. |
| `src/cli/bootstrap.rs` | +6/-3 | reduce | Preserve status/repair presentation only if generic package contract requires it. |
| `src/cli/exec.rs` | +9 | split | Process-control plumbing unrelated to OpenSSL lifecycle. |
| `src/cli/run.rs` | +9 | split | Same. |
| `src/cli/system/driver.rs` | +6/-2 | reduce | Keep generic repair handling only; no brew-name special cases. |
| `src/cli/system/status.rs` | +2/-2 | reduce | Keep precise repair reason without backend coupling. |
| `src/cmd.rs` | +525/-21 | split | Process group/execution security framework; independent cross-platform PR. |
| `src/file.rs` | +1319 | split | FD-bound cleanup/restore/WAL primitives; independent API and tests first. |
| `src/sandbox/landlock.rs` | +482/-58 | split | Linux source confinement, not bottle/OpenSSL lifecycle. |
| `src/sandbox/macos.rs` | +51/-13 | split | Shared sandbox changes need independent regression proof. |
| `src/sandbox/mod.rs` | +647/-6 | split | Shared sandbox API has all-backend blast radius. |
| `src/sandbox/seccomp.rs` | +506/-17 | split | Linux source confinement; separate. |
| `src/system/packages/brew/api.rs` | +1688/-26 | reduce/split | Keep minimum typed formula/lifecycle metadata. JWS/tap client and generalized schema require separate design PR. |
| `src/system/packages/brew/cask.rs` | +1 | remove | Blank formatting noise; no formula behavior. |
| `src/system/packages/brew/fetch.rs` | +462/-11 | reduce/split | Existing flow already verified bottle SHA-256. Keep only verified bottle identity input lifecycle needs; split OCI descriptor, FD-retention, and generalized trust work. |
| `src/system/packages/brew/lifecycle.rs` | +9453 | replace | Core concept required; module size and hardcoded recipes unacceptable. Split parse/plan/execute/state modules with bounded schema. |
| `src/system/packages/brew/maintenance.rs` | +381/-41 | reduce | Keep lifecycle-aware repair/removal only. General recovery framework separate. |
| `src/system/packages/brew/mod.rs` | +268/-61 | keep/reduce | Preserve closure preflight, health model, repair path. |
| `src/system/packages/brew/pour.rs` | +6145/-610 | replace/reduce | Keep receipt/link/shared-state transaction changes needed by lifecycle. Closure ownership is in `mod.rs`/`resolve.rs`; extract generalized WAL/recovery. |
| `src/system/packages/brew/resolve.rs` | +461/-45 | reduce/split | Closure already existed. Keep only lifecycle-preflight changes; third-party GitHub tap authentication is separate. |
| `src/system/packages/brew/sbom.rs` | +407 | split | Provenance/SBOM useful but not required for OpenSSL fix. |
| `src/system/packages/brew/shim.rb` | +123/-80 | split | Source build/lifecycle interpreter scope; avoid expanding Ruby alongside Rust-first bottle fix. |
| `src/system/packages/brew/source.rs` | +2421/-109 | split | Source builds, patches, requirements, confinement are independent. |
| `src/system/packages/mod.rs` | +1 | keep | Repair reason integration acceptable if generic and minimal. |
| `src/system/resources.rs` | +3/-2 | reduce | Retain only if generic resource behavior required. |
| `src/task/task_executor.rs` | +9 | split | Process-control plumbing unrelated. |
| `src/test.rs` | +7 | conditional | Test helper only if reduced tests use it without global behavioral change. |

## CI architecture

Current jobs at `.github/workflows/test.yml:180-358` add separate macOS formula, Linux bottle, and Linux source pipelines. They compile exact head independently and gate full CI. This makes brew uniquely expensive versus other backends.

Replacement:

1. Unit-test lifecycle compilation/state transitions in normal Rust test job.
2. Build one exact-head artifact per OS/architecture and reuse it across same-platform brew jobs.
3. Run one path-gated macOS Homebrew interoperability oracle.
4. Put Linux bottle/source cases into existing e2e tranche; retain separate source job only if required host confinement cannot run there.
5. Keep destructive `/opt/homebrew` operations only on disposable runners with guard verification.

## Breaking-change audit

- Future `ca-certificates` revisions: likely install failure due embedded hashes.
- Future additive Homebrew metadata: likely parsing failure due strict internal schema.
- Third-party taps: GitHub-only canonical resolution adds unsupported repository/rate-limit/private-token cases; unrelated to goal.
- Formula resources/external patches and all macOS source builds: existing support is removed in this PR's own delta (`docs/bootstrap/packages/brew.md:373-383`), unrelated to OpenSSL lifecycle. Preserve it or deprecate separately under mise policy.
- Shared command/file/sandbox paths: defaults appear compatible and CI passes, but change surface is too broad to prove no regressions inside this PR.
- Current exact test fixtures: pass, including OpenSSL/CA, repair, Homebrew list/info/uninstall, Linux bottle/source, Windows, lint, and e2e.

## Acceptance call

Approval likelihood is low. Bots reporting green/clean cannot make 26k lines and 209 commits maintainable. Replace; do not add more repair commits to this history.

No new local exact-head test is claimed: the copied build artifact referenced a
deleted temporary schema path. GitHub's green exact-head workflow is the
authoritative execution proof; source inspection proves the rolling pin/cache
defects above.
