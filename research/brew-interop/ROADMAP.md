# Make mise bootstrap converge through reviewable Brew interoperability fixes

- **Status**: SHAPING
- **Slug**: mise-bootstrap-brew-interop
- **Created**: 2026-08-22
- **Plan**: — (`plan/` once planned) · **Verified**: — (`verification/` once run)

## Intent

Make `mise bootstrap` usable as the single bootstrap entrypoint for the private
[`essential-mac`](https://github.com/donbeave/essential-mac) repository.

Replace the oversized mise PRs
[#11915](https://github.com/jdx/mise/pull/11915) and
[#11910](https://github.com/jdx/mise/pull/11910) with small, focused portions
that jdx can understand and review. Preserve the useful evidence and structural
ideas from those PRs without carrying their scope, regressions, or stacked
history into the replacements.

When this ships, a fresh Mac and an existing Homebrew-managed Mac can run the
essential-mac bootstrap repeatedly. Healthy Homebrew-owned casks satisfy the
configuration without ownership transfer, and mise-poured formulae include and
repair the lifecycle state required for working default TLS.

## Vocabulary

- **Formula lifecycle effect**: A supported post-install or shared-state effect
  required for a poured formula to work, beyond the keg, receipt, and active
  links.
- **Homebrew-owned cask**: A cask whose healthy native Homebrew receipt and
  topology establish Homebrew as producer and mutation owner.
- **mise-owned package state**: Package state installed or explicitly adopted
  through mise and therefore eligible for mise-managed upgrade, repair, or
  prune under existing rules.
- **Recognition**: Read-only classification of existing package state. It does
  not transfer ownership or grant mutation authority.
- **Explicit adoption**: Existing deliberate contract that transfers package
  management authority to mise. Recognition alone is not adoption.
- **Successor PR**: A new current-main pull request containing one focused
  concern. It is not a trimmed continuation of either oversized draft.
- **Formula fix**: Complete formula lifecycle outcome required by this item. It
  may span multiple successor PRs; research determines their boundaries.
- **Cask fix**: Complete native-cask compatibility outcome required by this
  item. It may span multiple successor PRs; `1password-cli` is only one fixture.

## Decisions

- 2026-08-22 — **Do not try to obtain review for the current #11915 or #11910
  diffs.** Their review surface and bundled correctness regressions cannot be
  fixed through more explanation or retry commits.
- 2026-08-22 — **Deliver small, independently understandable upstream PRs.**
  Each PR must focus on one mise bootstrap failure and preserve surrounding
  supported behavior.
- 2026-08-22 — **Treat existing PRs as prototypes and evidence, not as the new
  specification.** Useful invariants and focused tests may survive; unrelated
  architecture and regressions must not.
- 2026-08-22 — **Keep package presence separate from mutation ownership.** A
  healthy Homebrew-owned cask can satisfy bootstrap without becoming
  mise-owned.
- 2026-08-22 — **Define formula health beyond poured files and links.** Required
  supported lifecycle effects are part of healthy installed state.
- 2026-08-22 — **Include a temporary essential-mac bridge while upstream fixes
  await merge and release.** Bootstrap must work before uncertain upstream
  delivery completes; remove the bridge only after a released mise containing
  both fixes passes repeated essential-mac bootstrap verification.
- 2026-08-22 — **Prioritize the OpenSSL formula lifecycle successor before the
  cask successor.** The only observed `brew:` formula failure in essential-mac
  is the OpenSSL/CA lifecycle defect, and restoring working formula bootstrap is
  the first priority.
- 2026-08-22 — **Cover every cask declared by essential-mac, not only
  `1password-cli`.** `1password-cli` is a concrete regression fixture; success
  means the ownership model works across the repository's complete cask set.
- 2026-08-22 — **Bound formula scope to the complete current essential-mac
  `brew:` set and working OpenSSL/CA default trust.** Include dependency
  lifecycle operations, including Node operations, only when that actual closure
  requires them; do not pursue general Homebrew lifecycle parity.
- 2026-08-22 — **Research formula lifecycle metadata sources before selecting
  one.** Current evidence makes the public API a candidate, but there is not yet
  enough detail to choose it confidently over other authenticated,
  bottle-bound approaches.
- 2026-08-22 — **Do not preselect a fixed formula PR count or architecture.**
  Research current mise design first, then split work into the smallest clear
  review units; each PR must state exactly what it does and remain easy for jdx
  to review.
- 2026-08-22 — **Set a soft production-diff target from recent jdx-reviewed
  mise PR evidence.** Do not invent an arbitrary line cap before research;
  always require one clear responsibility and split every independent piece.
- 2026-08-22 — **Close oversized draft PRs #11915 and #11910.** Preserve their
  exact heads and audit evidence first, but do not leave unmergeable drafts open
  as active delivery artifacts.
- 2026-08-22 — **Research current mise/Homebrew behavior before defining full
  compatibility semantics.** Compare only capabilities already implemented by
  mise's `brew:` and `brew-cask:` bootstrap engines with official Homebrew Ruby
  behavior; do not expand research into unrelated Homebrew features.

## Capabilities

- Break the Brew interoperability work into successor PRs whose titles, bodies,
  diffs, and tests each describe one concern.
- Derive formula PR boundaries from current-main architecture evidence, diff
  size, and one clear responsibility per review unit rather than from the old
  implementation.
- Start successor branches from current upstream mise `main`; do not preserve
  the old stack merely because #11910 contains #11915 as an ancestor.
- Preserve exact old heads and audits as evidence for tests and design details.
- Provide a temporary downstream bridge that makes essential-mac bootstrap
  converge while preserving mixed Homebrew and mise ownership.
- Remove the temporary bridge after a released mise containing both upstream
  fixes passes repeated essential-mac bootstrap verification.
- Recognize a healthy native Homebrew cask receipt, version directory, and
  required artifact/backlink topology.
- Report a healthy Homebrew-owned cask as installed for status and bootstrap.
- Make bootstrap/apply a no-op for every healthy Homebrew-owned cask declared by
  essential-mac; use `brew-cask:1password-cli` as one exact regression fixture.
- Preserve producer origin after installed-state classification.
- Keep status inspection read-only: no receipt conversion, link creation,
  deletion, or ownership transfer.
- Keep ordinary upgrade and prune from mutating Homebrew-owned state until
  explicit adoption.
- Refuse malformed, ambiguous, or conflicting foreign cask state before
  mutation with precise guidance.
- Keep mise-owned cask install, upgrade, prune, and existing explicit adoption
  behavior working.
- Give platform-inapplicable casks one backend-neutral classification consumed
  consistently by apply, status, doctor, and install hints.
- Preserve current third-party cask behavior.
- Extend formula health from keg-plus-active-link presence to verified artifact
  identity, healthy dependency closure, supported lifecycle effects, native
  receipt, and expected links/topology.
- Reuse existing formula dependency resolution, bottle verification, native
  receipt, source-build, and link behavior instead of presenting them as new.
- Compile typed formula lifecycle operations before mutation.
- Preflight lifecycle semantics for the complete already-resolved closure.
- Fail before any closure mutation when a required operation, path, helper, or
  ownership state is unsupported or ambiguous.
- Apply required shared CA/OpenSSL state so default OpenSSL and Node TLS work
  without environment overrides.
- Distinguish absent formula state, healthy state, repairable damaged mise-owned
  state, unsupported state, and ambiguous ownership.
- Detect and repair missing mise-owned lifecycle effects without repouring a
  healthy keg or rewriting a healthy native receipt.
- Bind supported lifecycle inputs and executable helpers to verified bottle or
  formula identity.
- Cover every lifecycle operation required by the current essential-mac formula
  set while excluding unrelated Homebrew lifecycle parity.
- Support equivalent lifecycle semantics across at least two metadata revisions
  without a hardcoded allowlist for one mutable live formula revision.
- Preserve native `brew list`, `brew info`, and uninstall interoperability.
- Use focused unit/fixture coverage and one narrow existing macOS e2e path for
  each behavior instead of a mandatory broad Brew CI redesign.
- Keep broad differential corpora, destructive canonical-prefix matrices, and
  slow source-build/confinement tests scheduled or manual unless a focused PR
  directly changes those paths.
- Retain ordinary mise project format, lint, unit, and e2e gates.
- Reuse one exact-head artifact per OS/architecture across same-platform Brew
  tests when such an artifact is required.

## Screens

## Flows

### Existing Homebrew-owned cask

1. `mise bootstrap` reads a `brew-cask:` declaration.
2. Cask backend reads and minimally validates native Homebrew metadata and
   topology.
3. Healthy foreign state is classified with Homebrew producer origin.
4. Status and bootstrap report the requirement satisfied.
5. Apply, ordinary upgrade, and prune leave the foreign state unchanged.
6. Mutation becomes available only through explicit adoption.
7. Ambiguous or damaged foreign state fails before mutation with repair or
   adoption guidance.

### Poured formula lifecycle

1. Mise resolves the complete formula dependency closure using existing
   behavior.
2. Every required lifecycle plan is parsed, typed, identity-bound, and
   preflighted before closure mutation.
3. Existing machinery downloads, verifies, and pours bottles.
4. Supported shared defaults and post-install effects are applied.
5. Lifecycle postconditions join receipt and link topology in health
   classification.
6. Missing recorded mise-owned effects become repairable state.
7. Bootstrap repairs only those missing effects and rechecks health.
8. Unsupported semantics or conflicting foreign content fail without partial
   mutation.

### Upstream delivery

1. Preserve old heads and audit evidence, then close both oversized drafts and
   stop development on them.
2. Confirm each focused boundary against current mise main.
3. Obtain maintainer direction for non-obvious formula lifecycle semantics
   before publishing broad implementation.
4. Publish and complete the focused OpenSSL/CA formula lifecycle successor PR
   series, with count and boundaries determined by research.
5. Publish the cask successor PR series covering the complete essential-mac
   cask set, split only where independent review units exist.
6. Create each successor from current main and keep it focused, green, and free
   from the other successor's diff.
7. Address automated feedback before asking jdx for review.
8. Verify released mise against essential-mac bootstrap, including a repeated
   run on fresh and Homebrew-managed machines.

## Data & integrations

- **essential-mac configuration**: `mise.toml` declares formulae and casks,
  including 22 `brew-cask:` entries; `"brew-cask:1password-cli" = "latest"`
  is one representative declaration (`mise.toml:35-78`).
- **mise upstream**: Successor changes target
  [`jdx/mise`](https://github.com/jdx/mise) current `main`.
- **Successor independence**: Current-main cask work needs no formula code,
  generic `PackageState` change, shared-driver special case, doctor change,
  install-hint change, or broad receipt serializer. Formula-first ordering was
  only an artifact of the old stacked GitHub diff
  (`docs/mise/CURRENT-STATE-EVIDENCE.md:67-94,209-216`). This evidence applies
  to the native-ownership recognition successor; a separately justified
  platform-applicability successor may touch existing doctor or install-hint
  consumers.
- **Current formula metadata boundary**: mise already consumes the public
  `formulae.brew.sh` API for bottle URL and SHA-256, and current records expose
  structured `post_install_steps`
  (`docs/mise/CURRENT-STATE-EVIDENCE.md:136-140,177-188`).
- **Observed lifecycle operations**: The essential-mac closure currently needs
  a bottle-contained `run` for `ca-certificates`, forced `symlink` for
  `openssl@3`, and Node's `mkdir_p`, guarded recursive `remove`, recursive
  `copy`, forced `symlink`, and declared source globs
  (`docs/mise/CURRENT-STATE-EVIDENCE.md:142-175`).
- **Mixed live cask ownership**: Almost every configured cask has native
  Homebrew metadata while Ghostty is mise-owned, so any temporary bridge must
  classify and preserve both producers rather than bulk-transfer ownership
  (`docs/mise/CURRENT-STATE-EVIDENCE.md:51-65`).
- **Archived formula prototype**: #11915 head
  `97640cc04359fa6488706ac46bf71a46cf840b3b` changes 38 files by
  `+26,130/-1,131` and contains 209 commits from its merge base.
- **Archived cask stack**: #11910 head
  `a1df6b7fd11c2a7a1c7bb802f9a503f98e122496` contains #11915, exposes 61
  changed files and `+46,752/-8,111` against `main`, and adds a cask-only delta
  of 51 files and `+21,127/-7,168`.
- **Review state on 2026-08-22**: Both old PRs are open drafts, mechanically
  mergeable, green at their exact heads, and have no maintainer approval.
  #11910 has an explicit maintainer refusal based on comprehension and review
  surface. Green checks do not prove merge fitness.
- **Homebrew cask state**: Native `.metadata/INSTALL_RECEIPT.json`, version
  directories, and required artifacts/backlinks provide read-only presence and
  producer evidence.
- **Current Homebrew-owned cask command behavior**: mise status requires its
  own `.mise-cask.toml`, so native Homebrew state is classified `Missing`;
  bootstrap apply selects that missing request and the cask installer then
  refuses the native `.metadata` ownership boundary. Explicit upgrade filters
  out `Missing`, warns to run apply, leaves the cask unchanged, and normally
  exits successfully without reaching the cask manager's upgrade path
  ([mise `driver.rs:159-197`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/cli/system/driver.rs#L159-L197),
  [mise `cask.rs:428-447`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L428-L447),
  [mise `cask.rs:6557-6607`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6557-L6607)).
- **Homebrew formula state**: Bottle identity, dependency closure, native
  receipt, active links, shared `etc`/CA state, and supported post-install
  effects together provide health evidence.
- **Existing adoption contract**: Reused for explicit ownership transfer; not
  replaced by recognition.
- **CI**: Existing mise gates plus focused Rust and macOS interoperability
  evidence.

## References

- `/Users/donbeave/Projects/donbeave/mise/BREW-INTEROP-PROBLEM-REFERENCE.md`
  — canonical source problem statement, rejected-PR defects, acceptance
  conditions, replacement boundaries, and open design decisions.
- [`docs/mise/CURRENT-STATE-EVIDENCE.md`](../../docs/mise/CURRENT-STATE-EVIDENCE.md)
  — essential-mac reproduction and current-main evidence.
- [`docs/mise/DELIVERY-PLAN.md`](../../docs/mise/DELIVERY-PLAN.md) — existing
  delivery proposal to test during shaping.
- [`docs/mise/research/PR-11915-FORMULA-LIFECYCLE-AUDIT.md`](../../docs/mise/research/PR-11915-FORMULA-LIFECYCLE-AUDIT.md)
  — full formula changed-file and correctness audit.
- [`docs/mise/research/PR-11910-BREW-CASK-INTEROP-AUDIT.md`](../../docs/mise/research/PR-11910-BREW-CASK-INTEROP-AUDIT.md)
  — full cask changed-file and ownership audit.
- [`docs/mise/research/PR-REVIEW-SUMMARY.md`](../../docs/mise/research/PR-REVIEW-SUMMARY.md)
  — combined PR and maintainer-review evidence.
- [`docs/mise/research/BREW-PR-REPLACEMENT-PLAN.md`](../../docs/mise/research/BREW-PR-REPLACEMENT-PLAN.md)
  — earlier replacement sequence retained as historical evidence.
- [`mise.toml`](../../mise.toml) — actual essential-mac bootstrap declarations.
- [jdx/mise#11915](https://github.com/jdx/mise/pull/11915) — oversized formula
  lifecycle prototype.
- [jdx/mise#11910](https://github.com/jdx/mise/pull/11910) — oversized stacked
  Brew interoperability prototype and maintainer refusal.

## Research

- [Mise/Homebrew bootstrap engine correspondence](../../research/mise-homebrew-bootstrap-engine-parity/README.md)
  — establishes current `brew:`/`brew-cask:` architecture, official Homebrew
  semantics for their implemented overlap, and the historical rationale/status
  of each known divergence; informs compatibility and focused successor
  boundaries.

## Must not

- MUST NOT ask jdx to review either current oversized diff — maintainer already
  rejected #11910's comprehension and review surface.
- MUST NOT add more merge-main, retry-only, review-bot, or CI-expansion commits
  to the old branches — they do not change the rejected artifact shape.
- MUST NOT combine formula lifecycle and cask ownership into one broad Homebrew
  compatibility layer — they are separate bug classes.
- MUST NOT carry old stacked ancestry into successor branches without a proven
  code dependency — it makes the second PR expose the first.
- MUST NOT claim existing dependency-closure resolution or native formula
  receipt/link behavior as new work — mise already had them.
- MUST NOT expand the formula successor into general Homebrew lifecycle parity —
  its product boundary is the current essential-mac `brew:` set and working
  OpenSSL/CA default trust.
- MUST NOT lock the roadmap to a speculative number or shape of formula PRs
  before current mise architecture is researched.
- MUST NOT pin accepted lifecycle behavior to mutable live formula or helper
  digests — legitimate Homebrew updates would cause rolling outages.
- MUST NOT accept unsupported formula lifecycle operations and fail after
  partial mutation — complete closure preflight must fail first.
- MUST NOT evaluate arbitrary Ruby or execute helpers found through ambient
  `PATH` as part of a typed lifecycle fix.
- MUST NOT overwrite conflicting or foreign formula shared state during repair.
- MUST NOT use a live public HTTP status such as `auth.kimi.com` as the TLS
  oracle — use a local certificate chain and server.
- MUST NOT couple the formula fix to a new internal JWS client, whole-record
  schema redesign, OCI retry system, source-build overhaul, SBOM, WAL, sandbox,
  process-group, or filesystem framework unless a focused prototype proves a
  required primitive is missing and it is split independently.
- MUST NOT remove current resources, external patches, macOS source builds,
  bottle behavior, formula-tap support, or platform support as collateral
  formula work.
- MUST NOT treat native cask recognition as ownership transfer.
- MUST NOT upgrade, prune, uninstall, rewrite, or replace Homebrew-owned cask
  state without explicit adoption.
- MUST NOT erase producer origin into a version-only installed state.
- MUST NOT make status mutate receipts, links, files, or ownership.
- MUST NOT change generic package-state semantics or shared drivers through a
  `brew-cask` manager-name special case.
- MUST NOT remove third-party cask support or inaccurately document supported
  formula taps as removed.
- MUST NOT use unsafe libc ordering to reproduce incidental receipt byte layout
  when safe deterministic semantic compatibility is sufficient.
- MUST NOT bundle Aqua, task, lock, config, HTTP, formula, ELF, or global CI
  changes into the cask successor.
- MUST NOT treat green CI or mechanical mergeability as proof of reviewability,
  future Homebrew compatibility, ownership safety, or preserved behavior.

## Quality bar

- A fresh Mac can complete `mise bootstrap --yes` from essential-mac twice.
- A Mac with healthy Homebrew-owned casks can complete the same command twice.
- Every healthy Homebrew-owned cask declared by essential-mac satisfies status
  and bootstrap without reinstall or ownership transfer; `1password-cli`
  remains an exact regression fixture.
- Native cask status is read-only; ordinary upgrade and prune leave every
  Homebrew-owned file, receipt, version directory, and artifact unchanged.
- Explicitly adopted and mise-owned casks remain maintainable through existing
  behavior.
- Malformed, multiple-version, conflicting, or ambiguous native cask state
  fails before mutation with precise guidance.
- Platform-inapplicable casks have consistent apply, status, doctor, and install
  hint behavior.
- A poured OpenSSL/CA dependency closure works with default OpenSSL and Node TLS
  without `SSL_CERT_FILE`, `NODE_EXTRA_CA_CERTS`, or equivalent overrides.
- Formula lifecycle damage is detected and only proven mise-owned effects are
  repaired.
- Repair preserves healthy keg bytes/inode, bottle identity, native receipt,
  and active-link topology.
- Unsupported lifecycle semantics and repeated metadata incoherence terminate
  safely without recursion or partial prefix mutation when those paths are
  part of the successor.
- At least two supported metadata revisions prove lifecycle compatibility
  without a mutable live-recipe allowlist.
- Current formula, bottle, source-build, third-party tap, cask, adoption, Linux,
  macOS, and Windows behavior remains unless separately approved.
- Current stable Homebrew can read relevant formula/cask state and perform
  expected cleanup on disposable test machines.
- TLS proof uses a local CA/server and tests real default trust behavior.
- Each successor PR contains one understandable concern, no unrelated files,
  ordinary project gates, focused regression evidence, and a description that
  distinguishes existing behavior from new behavior.
- Successor PR production diffs fit the evidence-based soft size target derived
  from recent jdx-reviewed mise work; exceeding it requires an explicit proof
  that remaining changes are inseparable.
- Cask successor does not contain formula changes; formula successor does not
  contain cask changes.
- Released mise containing required fixes is verified against the actual
  essential-mac configuration before any temporary workaround is removed.

## Open questions

- Given the settled no-mutation-before-adoption rule, should ordinary
  upgrade of a healthy Homebrew-owned cask be a silent no-op or an explicit
  refusal with adoption guidance? **Recommendation:** exit successfully but
  visibly report that Homebrew owns the cask and that mutation requires
  Homebrew or explicit mise adoption; bootstrap stays error-free without
  pretending an upgrade occurred.
- Given the same ownership rule, should prune silently preserve an unconfigured
  Homebrew-owned cask or explicitly report that it was excluded from removal?
  **Recommendation:** preserve it, exit successfully, and include the exclusion
  in prune output so the ownership boundary is observable.
- Does the supported package inventory freeze when successor fixtures are cut,
  or track every essential-mac change until release verification?
  **Recommendation:** freeze each successor's fixture inventory at branch cut,
  then run a separate release-gate check against the latest essential-mac set.
- What should happen when mise encounters a damaged Homebrew-owned formula whose
  keg still satisfies current active-link presence but required lifecycle state
  is unhealthy? **Recommendation:** report ownership-specific unhealthy state
  and direct repair through Homebrew; do not let mise repair foreign shared
  state without an explicit ownership-transfer contract.
- Does formula lifecycle scope include source-built formulae when their existing
  shim already runs `post_install`? **Recommendation:** change source behavior
  only if the frozen essential-mac closure proves an observable source-path gap;
  otherwise keep the successor bottle-focused.
- What milestone ends “formula first” and permits cask implementation to start?
  **Recommendation:** allow cask research and fixture work in parallel, but do
  not publish cask successor PRs until the formula successor boundary is agreed
  with maintainers and its first focused PR is reviewable.

## Open research questions

- Which engine-level cask differences are actually exercised by the complete
  22-cask essential-mac set and therefore belong in successor scope?
- Which available formula lifecycle metadata sources can provide authenticated,
  version-tolerant, bottle-bound structured semantics without arbitrary Ruby or
  mutable live-recipe pins, and what are each source's trust, schema, freshness,
  offline, compatibility, and mise-review-scope trade-offs?
- Which exact typed lifecycle operations are required by the complete current
  essential-mac formula closure under the selected metadata source?
- What is the minimum current-main cask diff that recognizes and preserves
  Homebrew ownership while reusing existing adoption and prune guards?
- Which essential-mac formula dependency closures require structured lifecycle
  operations, and which operation shapes occur in current Homebrew metadata?
- Which current mise APIs and transaction primitives can support the formula
  successor without new generic frameworks?
- How does current mise formula architecture naturally decompose metadata,
  planning, apply, health/repair, tests, and documentation into small reviewable
  PRs, and what dependencies force any pieces to stay together?
- What production-diff sizes, file counts, responsibility boundaries, and
  review outcomes characterize recent jdx-reviewed or merged mise PRs, and what
  soft target follows for these successors?
- Which existing transaction primitive is sufficient for shared CA/config
  state, and what concrete missing capability would justify a separate generic
  prerequisite?
- Where can cask producer origin live so all relevant state consumers retain it
  without generic backend-name checks?
- Which focused existing mise e2e jobs can host the cask and formula regression
  proofs without adding new mandatory build pipelines?
- What maintainer direction and contribution constraints apply to each proposed
  successor boundary?

## Deferred

## Remaining

- Close #11915 and #11910 after confirming their cited exact heads remain
  recoverable from the recorded commit IDs.
- Research the open factual questions and return their evidence to this item.
- Resolve the open product decisions before finalization and planning.

## Run

— (`goal/` once planned)
