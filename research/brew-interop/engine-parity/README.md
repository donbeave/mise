# Mise/Homebrew bootstrap engine correspondence

Informs: [mise-bootstrap-brew-interop](../ROADMAP.md)

Vetted: 2026-08-22

## Scope

This research compares only behavior already exposed by mise's `brew:` and
`brew-cask:` system-package managers through `mise bootstrap` and related
package commands. It does not inventory unrelated Homebrew commands or propose
general Homebrew parity. Evidence is pinned to jdx/mise
`6f52dcdf99e282ef7a7db68c81301fa4618d0f79`, Homebrew/brew
`24f8780838c054a92ebf58b09455acbb236731bd`, and Homebrew/homebrew-core
`b07b59bd9ccea0d24c198728ace955d60a798943`; each chapter records its method
and exact citations.

## Conclusions

### Current design

Mise implements two independent Rust engines. They use Homebrew prefixes,
metadata, bottle/cask payloads, and some receipt conventions, but they do not
execute Homebrew's Ruby install state machine for their normal paths:

- `brew:` resolves formula metadata and dependency closure, selects a bottle
  or source build, pours/builds a keg, writes `INSTALL_RECEIPT.json`, creates
  links, and decides health mainly from the active `opt` keg. Its bottle path
  stops after pour/link and does not run Homebrew's finish/post-install stage.
  ([current mise formula engine](01-current-mise-engines.md#what-does-brew-install-or-pour))
- `brew-cask:` parses cask JSON itself, installs a supported artifact subset,
  runs supported lifecycle operations through Rust plus a limited Ruby shim,
  and records ownership in `.mise-cask.toml`. Status, upgrade, and prune are
  driven by that private receipt. ([current mise cask engine](01-current-mise-engines.md#what-metadata-dependency-and-artifact-behavior-does-brew-cask-implement))
- `mise bootstrap` supplies shared orchestration: configured requests,
  install/status/upgrade/prune dispatch, platform filtering, sequential
  execution, and first-error propagation. The engines own package-specific
  state and mutations. ([bootstrap entrypoints and driver](01-current-mise-engines.md#what-public-paths-exercise-these-engines))

Therefore these are Homebrew-shaped bootstrap engines, not a wrapper around
`brew` and not the official Ruby implementation translated one-for-one.

### Why the design differs

The controlling upstream rationale is **“brew without brew.”** Mise designed
bootstrap to install Homebrew artifacts on a fresh machine where Homebrew may
not exist. That requires mise to fetch metadata, resolve dependencies, execute
install steps, record state, and decide health itself. A real-`brew` proxy for
unsupported tap operations briefly merged in #10375, then jdx removed it about
two and a half hours later; the replacement documentation says unsupported
operations must fail rather than delegate so the engine remains independent of
Homebrew. Source builds and casks subsequently kept that same boundary.
([shared architecture and history](04-shared-architecture-rationale.md#why-is-brew-implemented-directly-rather-than-invoked))

That explains **why separate Rust/Ruby-shim machinery exists**. It does not
prove that every semantic difference is intentional. The reviewed history
supports this classification:

| Difference class | Research status | Meaning for successor PRs |
|---|---|---|
| Direct installation without Homebrew; direct API metadata; canonical formula prefix; limited fail-loudly DSLs | Documented intentional design or constraint | Preserve unless maintainers explicitly change the product boundary. |
| Formula/cask capability arriving in small follow-up changes | Incremental gap | Treat only demonstrated missing behavior as a focused extension. |
| Formula bottle finish, OpenSSL/CA health, old-keg cleanup, exact dependency/receipt semantics | Unexplained | No evidence these differences were deliberately chosen; show the concrete incorrect outcome and fix the narrow responsible layer. |
| Cask host variation/language/platform selection, aggregate phase order, dependency-safe prune, native-cask read-only recognition, batch continuation | Unexplained | Do not present as rejected Homebrew semantics; first prove exposure in essential-mac, then isolate one responsibility. |
| Native cask `.metadata` deletion and app-content-drift reinstall | Fixed regressions | Evidence that “independent engine” never makes incompatible behavior correct by definition. |
| Mise-owned cask receipt, refusal to mutate Homebrew-owned casks, conservative prune, `auto_updates` skip/adoption rules | Documented intentional design | Preserve producer ownership; convergence must not silently take over foreign state. |

No reviewed maintainer source explains why embedding Homebrew Ruby, vendoring
its libraries, or bootstrapping official Homebrew first was rejected in detail.
The evidence establishes the stronger product boundary—operation without
Homebrew—not a complete alternatives analysis. ([formula rationale](05-formula-design-rationale.md),
[cask rationale](06-cask-design-rationale.md))

### Formula overlap: material differences

| Existing capability | mise `brew:` | Official Homebrew consequence |
|---|---|---|
| Installed state | Active `opt/<name>` resolving to a keg is the main predicate. | Homebrew separates latest/any-version installed, linked, opt-linked, and dependency-satisfied states. A keg can exist while link or post-install failed. ([formula state](02-homebrew-formula-correspondence.md#what-is-the-official-formula-state-model)) |
| Bottle finish | Verifies, relocates, receipts, renames, and links; no formula Ruby or post-install for bottles. | Homebrew finish restores `.bottle/etc` and `.bottle/var`, then runs declarative/Ruby post-install. ([official install flow](02-homebrew-formula-correspondence.md#what-is-the-official-install-flow)) |
| OpenSSL/CA trust | No generic bottle-finish lifecycle branch; active links can look healthy while trust files are absent. | `ca-certificates` generates the shared bundle; `openssl@3` post-install links its default `cert.pem` to that bundle. ([OpenSSL/CA lifecycle](02-homebrew-formula-correspondence.md#what-is-the-post-install-lifecycle-including-opensslca-trust-state)) |
| Source build | Executes a limited in-repository Formula DSL shim and its `post_install`. | Loads upstream formula Ruby and runs build/post-install in Homebrew's own process and sandbox model. ([metadata and source](02-homebrew-formula-correspondence.md#what-are-the-source-and-tap-semantics)) |
| Receipt | Writes a Homebrew-shaped `INSTALL_RECEIPT.json`; static shape is known. | Live interoperability with current Homebrew remains unproven, and Homebrew's receipt/state model carries more distinctions. ([mise receipt](01-current-mise-engines.md#what-does-brew-install-or-pour)) |
| Upgrade/removal | Upgrade relinks a new active keg but leaves inactive old kegs; prune enumerates active `opt` links without producer gating. | Homebrew has separate upgrade cleanup, dependency checks, pin/keep rules, keg-scoped unlink, and uninstall behavior. ([upgrade](02-homebrew-formula-correspondence.md#what-is-the-official-upgrade-flow), [uninstall](02-homebrew-formula-correspondence.md#what-is-the-official-uninstall-flow)) |
| Failure | Mise stops the manager batch at first error; earlier completed dependencies remain. | Several official install/upgrade failures mark the run failed and continue; link/post-install failure can preserve a keg plus an explicit recovery path. ([failure boundaries](02-homebrew-formula-correspondence.md#where-are-the-official-failure-boundaries)) |

The OpenSSL defect is architectural, not an `openssl@3` special-case mystery:
the bottle path has no equivalent of Homebrew's finish stage, while mise health
does not represent required lifecycle effects. The current roadmap bounds the
product fix to the essential-mac formula closure rather than all
Homebrew formula behavior. The gap existed when the engine was introduced and
no inspected source explains it as intentional; it is not a verified later
regression. ([mise lifecycle contradiction](01-current-mise-engines.md#dead-ends-and-contradictions),
[formula history](05-formula-design-rationale.md#status-ledger-openssl-and-ca-trust))

### Cask overlap: material differences

| Existing capability | mise `brew-cask:` | Official Homebrew consequence |
|---|---|---|
| Metadata selection | Directly deserializes a narrower public API record; no general `variations`, language localization, or platform `depends_on` model. | Homebrew merges current OS/architecture variation and selected language before constructing artifacts. ([platform and language](03-homebrew-cask-correspondence.md#how-do-platform-architecture-and-language-semantics-correspond)) |
| Installed receipt | Requires one version and a valid `.mise-cask.toml`, recorded targets, and pkg receipts. | Requires native `.metadata` caskfile topology; it does not recognize mise's receipt. ([receipt topology](03-homebrew-cask-correspondence.md#how-does-homebrew-detect-installation-and-what-is-its-native-receiptversion-topology)) |
| Homebrew-owned state | Detects `.metadata` as foreign, reports the request missing, then refuses apply; prune skips it. | Native Homebrew regards the same cask as installed. Thus a healthy Homebrew cask currently makes mise bootstrap fail rather than converge. ([cross-recognition](03-homebrew-cask-correspondence.md#what-concrete-ownership-and-cross-recognition-rules-exist)) |
| Install lifecycle | Supports many shared artifact types, but phase order, quarantine handling, shim coverage, and rollback boundaries differ. | Official artifact classes and native flight blocks define different ordering and quarantine/rollback behavior. ([artifacts and phase order](03-homebrew-cask-correspondence.md#how-does-official-homebrew-interpret-cask-api-metadata-and-artifacts), [fetch and quarantine](03-homebrew-cask-correspondence.md#what-are-the-corresponding-fetch-checksum-and-staging-semantics)) |
| Upgrade | Version drift reinstalls mise-owned non-auto-update casks; current API `auto_updates=true` becomes a no-op after installed-state validation. | A named `brew upgrade <cask>` uses greedy selection and may upgrade `auto_updates` or `latest` casks; Homebrew can inspect live app versions. ([upgrade correspondence](03-homebrew-cask-correspondence.md#how-do-upgrade-auto_updates-and-version-drift-correspond)) |
| Removal | No direct uninstall; prune removes only unchanged, exclusively claimed, prune-safe recorded targets and skips side-effectful artifact classes. It does not preserve cask dependency closure. | Homebrew uninstall executes installed artifact uninstall metadata and dependent checks; zap and Bundle cleanup are separate stronger operations. ([uninstall and prune](03-homebrew-cask-correspondence.md#what-correspondences-exist-for-uninstall-zap-prune-and-cleanup)) |
| Batch failure | First cask error stops mise's manager batch after earlier commits. | Homebrew install/reinstall normally reports a failed cask and continues the batch. ([failure boundaries](03-homebrew-cask-correspondence.md#where-are-the-failure-boundaries)) |

### Meaning of “compatible with Homebrew”

Within this roadmap's settled boundaries, the compatibility target is scoped
observable equivalence: for operations mise already claims to provide and
packages in the essential-mac set, bootstrap must reach the same healthy
user-visible outcome without errors or unsafe ownership changes. That does not
require adding every Homebrew feature. Current mise does not meet that target
for poured OpenSSL/CA state or healthy Homebrew-owned casks.
([formula evidence](02-homebrew-formula-correspondence.md#what-is-the-post-install-lifecycle-including-opensslca-trust-state),
[cask evidence](03-homebrew-cask-correspondence.md#what-concrete-ownership-and-cross-recognition-rules-exist))

One boundary must remain explicit: “same as Homebrew” cannot simultaneously
mean that ordinary mise upgrade mutates a Homebrew-owned cask and that producer
ownership is preserved until explicit adoption. The linked roadmap has already
settled the latter. Research therefore supports convergence for status/apply,
then either a silent no-op or an explicit adoption-required result for
upgrade/prune; choosing between those two user experiences remains a product
decision.

## Candidate directions within settled scope

### Formula lifecycle finish slice

Add a bounded finish/lifecycle plan for the already-resolved essential-mac
formula closure, then include lifecycle effects in health/repair. Advantage:
addresses the OpenSSL root cause at the missing architectural boundary and can
reuse existing bottle, receipt, dependency, and link machinery. Trade-off:
metadata trust, versioning, transaction, and operation coverage must be proven
before deciding PR boundaries. ([formula current engine](01-current-mise-engines.md#what-does-brew-install-or-pour), [official lifecycle](02-homebrew-formula-correspondence.md#what-is-the-post-install-lifecycle-including-opensslca-trust-state))

### Producer-aware native-cask recognition

Teach status/apply to classify a healthy native Homebrew receipt as satisfied
while retaining Homebrew producer origin; reuse the existing foreign-mutation
guard and adoption contract. Advantage: directly fixes mixed-machine bootstrap
without reimplementing native uninstall/upgrade. Trade-off: native health must
be defined narrowly enough to avoid accepting damaged or ambiguous state, and
upgrade/prune UX still needs the open no-op-versus-refusal decision.
([ownership boundary](03-homebrew-cask-correspondence.md#what-concrete-ownership-and-cross-recognition-rules-exist))

### Mise-owned cask semantic convergence

Independently close only essential-mac-exposed gaps in metadata variation,
platform selection, install phases, or upgrade behavior for mise-owned casks.
Advantage: preserves producer boundaries and allows one-responsibility PRs.
Trade-off: package-specific exposure across all 22 casks must be measured first;
the engine-level differences alone do not prove every gap affects the current
repository. ([package-specific unknown](03-homebrew-cask-correspondence.md#open-unknowns))

## Ruled out by current scope

- Full Homebrew feature parity: unrelated official capabilities were excluded
  from the question and are forbidden by the formula roadmap boundary.
- Treating Homebrew and mise cask receipts as interchangeable: each engine
  recognizes only its own topology, and mise deliberately treats `.metadata`
  as foreign ownership. ([receipt contradiction](03-homebrew-cask-correspondence.md#dead-ends-and-contradictions))
- Transparent mise mutation of Homebrew-owned casks: it conflicts with the
  settled producer-ownership/adoption boundary in the linked roadmap.
- Reusing either oversized draft as the delivery unit: this research describes
  current-main architecture; it does not change the existing review evidence
  that #11915 and #11910 must be replaced by focused successors.

## Open unknowns and disposition

- Formula lifecycle metadata trust and versioning: separate research before
  selecting a source or successor architecture.
- Exact lifecycle operations required by the current essential-mac formula
  closure: evaluate after choosing the metadata source.
- Exposure of cask variation/language/phase gaps across all 22 essential-mac
  casks: package-by-package research before defining cask successor slices.
- Live Homebrew behavior against mise-written formula receipts and
  mise-occupied Caskroom paths: destructive interop experiments were outside
  this static comparison; cover on disposable test machines if a successor
  depends on reciprocal mutation.
- Review-size target and natural current-main PR boundaries: separate research
  into recent jdx-reviewed PRs plus focused architecture decomposition.

## Chapters

- [01 — Current mise Brew bootstrap engines](01-current-mise-engines.md)
- [02 — Official Homebrew formula correspondence](02-homebrew-formula-correspondence.md)
- [03 — Official Homebrew cask correspondence](03-homebrew-cask-correspondence.md)
- [04 — Shared architecture and “Brew without Brew” rationale](04-shared-architecture-rationale.md)
- [05 — Formula design rationale and divergence status](05-formula-design-rationale.md)
- [06 — Cask design rationale and divergence status](06-cask-design-rationale.md)
