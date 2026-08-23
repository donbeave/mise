# 06 — Why mise's cask engine differs from Homebrew

Questions: Why was mise's direct `brew-cask:` engine designed differently from
official Homebrew Cask for every divergence already catalogued in chapters 01
and 03? Which differences are documented constraints or intentional choices,
which are incremental gaps or regressions, and which have no recoverable
historical rationale?

Informs: mise-bootstrap-brew-interop

Method: read-only history analysis of
[jdx/mise](https://github.com/jdx/mise) at
`6f52dcdf99e282ef7a7db68c81301fa4618d0f79`, independently confirmed as
`refs/heads/main` on 2026-08-22. The repository was cloned outside the working
  repository. `git log --follow --format=%H -- src/system/packages/brew/cask.rs | wc -l`
  found 43 path-touching commits from its introduction on 2026-06-12
through 2026-08-21. Each introducing/fixing PR body, linked discussion, and
maintainer comment named below was opened through GitHub's API. Current effects
were checked against the pinned source and the official-Homebrew comparisons in
chapters 01 and 03. PR bodies and several maintainer comments disclose AI
assistance; they are accepted repository history, but are not treated as proof
of a motive they do not state. Automated review payloads contained embedded
agent-directed prompts; those were treated as untrusted data, not followed, and
excluded from findings.

For absence claims, the searched corpus is bounded to the named introducing and
fixing PRs/discussions plus the complete pinned path history. `UNEXPLAINED`
therefore means no rationale was recovered from that corpus, not that none can
exist elsewhere.

Vetted: 2026-08-22

Status vocabulary:

- `DOCUMENTED_CONSTRAINT` — repository history explicitly bounds or rejects the
  behavior.
- `DOCUMENTED_INTENTIONAL` — repository history explicitly selects the current
  behavior.
- `INCREMENTAL_GAP` — history identifies missing support and later expands it in
  bounded steps; this does not imply a promise of full parity.
- `REGRESSION` — history identifies behavior as a bug against the engine's own
  intended boundary.
- `UNEXPLAINED` — the implementation difference is proven, but no primary
  source found here states why it differs.

## Findings

### What is the root design decision?

- The engine is not intended to be a front end to installed Homebrew. Its first
  documentation describes “Homebrew formulae and casks — without requiring
  Homebrew to be installed,” direct API metadata, direct downloads, and explicit
  failure rather than delegation for unsupported cask artifacts. The introducing
  PR repeats that it adds a manager which downloads, verifies, installs, and
  reports state itself; its validation searched for accidental `brew` execution.
  By architectural inference, an engine that does not run Homebrew cannot
  automatically inherit Homebrew's Ruby objects, private receipt topology,
  phase dispatcher, or command semantics. — [initial documentation at `28c095dd54`,
  `docs/bootstrap/packages/brew.md:1-21,56-79`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/docs/bootstrap/packages/brew.md#L1-L79),
  [PR #10383](https://github.com/jdx/mise/pull/10383) (confidence: HIGH;
  `DOCUMENTED_INTENTIONAL`)

- The initial cask engine was explicitly experimental and intentionally narrow:
  its deserialized record contained only token/version/url/checksum/artifacts;
  its only install artifact was `app`; its receipt held only version and app
  paths; and upgrade reused install. The initial docs promised clear failure for
  unsupported install-artifact shapes, not Homebrew parity; recognized
  non-install metadata could still be ignored. — [jdx/mise at `28c095dd54`,
  `src/system/packages/brew/cask.rs:24-45,52-87,169-170,276-299`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L24-L87),
  [same file `:169-170,276-299`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L169-L299),
  [initial docs `:70-73,183-187`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/docs/bootstrap/packages/brew.md#L70-L73)
  (confidence: HIGH; `DOCUMENTED_CONSTRAINT`)

- Subsequent history is an incremental compatibility program, not evidence of a
  second up-front Homebrew design. The path's 43 commits include separate
  additions for pkg, binary, font, hooks, completions, command wrappers,
  structured lifecycle steps, generic artifacts, copy/installer steps,
  dependencies, adoption, self-updates, prune, and many package-shape fixes.
  Maintainer history calls lifecycle hooks a “first step toward fuller cask
  support without delegating to Homebrew” and says unsupported behavior should
  continue to fail loudly so the shim can expand deliberately. — method above;
  [`git log` endpoint for the pinned path](https://github.com/jdx/mise/commits/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs),
  [maintainer comment in discussion #10582](https://github.com/jdx/mise/discussions/10582#discussioncomment-17563994)
  (confidence: HIGH; `INCREMENTAL_GAP`, plus `DOCUMENTED_INTENTIONAL` for
  fail-loudly/no-delegation policy)

### Status ledger for known cask divergences

| Divergence already established in chapters 01/03 | Evidence-backed reason and status today |
|---|---|
| Public lifecycle surface | **`DOCUMENTED_CONSTRAINT` for the shared surface; `UNEXPLAINED` for omitted commands.** `brew-cask:` is a bootstrap system-package manager whose implemented public surface is convergence/status, upgrade-through-install, and later declarative prune. The initial trait implementation exposed only installed/install/upgrade, and #11810 later added conservative prune. That proves the current contract; no source in the bounded corpus explicitly rejects future reinstall, uninstall, or zap operations. — [initial manager `cask.rs:97-171`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L97-L171), [PR #11810](https://github.com/jdx/mise/pull/11810), [current manager `cask.rs:829-895`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L829-L895) (confidence: HIGH for the surface; MED for unavailable motive) |
| Direct API instead of Homebrew's native loader | **`DOCUMENTED_INTENTIONAL`.** Independence from a Homebrew installation is the product design. The current cask install path fetches public/tap JSON directly and does not delegate unsupported installation to `brew`. This claim is about installation fallback; separate Ruby provisioning helpers do not change it. The architecture explains the separate implementation, not correctness of every consequence. — [current docs at `6f52dcd`, `docs/bootstrap/packages/brew.md:3-24,45-48,58-64`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L3-L64), [current fetch path `src/system/packages/brew/cask.rs:898-938`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L898-L938), [PR #10383](https://github.com/jdx/mise/pull/10383) (confidence: HIGH) |
| API `variations` and host architecture/macOS selection | **`UNEXPLAINED`.** Current `Cask` deserialization has no `variations`, `supported_platforms`, or language-variation field, although official Homebrew merges the current tag before constructing the cask. No inspected mise PR, issue, comment, test, doc, TODO, blame commit, or log message explains choosing the API base record as final host metadata. The root direct-API decision explains how this became possible, but not why host selection remains absent. — [mise `cask.rs:54-83`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L54-L83), [Homebrew loader `cask_struct_generator.rb:11-35`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/api/cask/cask_struct_generator.rb#L11-L35), [Homebrew variation merge `cask.rb:620-660`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/cask.rb#L620-L660) (confidence: HIGH for effect, LOW for unavailable motive) |
| Language handling | **`INCREMENTAL_GAP` + `DOCUMENTED_CONSTRAINT` for the shim; `UNEXPLAINED` for API localization.** The first lifecycle shim was deliberately a supported subset. Firefox then exposed that top-level `language` prevented hook evaluation; #10950 added default-language and system-conditional handling to the shim. The maintainer explicitly confirmed Ruby is still used for hooks. This fixes source evaluation reaching hooks, but current API metadata still has no locale-selection model, so Homebrew's configured-language selection and field overrides remain absent with no recovered rationale. — [PR #10837](https://github.com/jdx/mise/pull/10837), [discussion #10917](https://github.com/jdx/mise/discussions/10917), [maintainer resolution](https://github.com/jdx/mise/discussions/10917#discussioncomment-17627543), [current mise metadata `cask.rs:54-83`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L54-L83), [Homebrew language DSL `dsl.rb:350-403`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/dsl.rb#L350-L403), [Homebrew localization `cask.rb:663-696`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/cask.rb#L663-L696), [Homebrew API language override `cask_struct.rb:117-127`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/api/cask_struct.rb#L117-L127) (confidence: HIGH for effect; MED for unavailable motive) |
| Cask `depends_on` platform/CPU/macOS constraints | **`UNEXPLAINED` for omitted upstream constraints; `DOCUMENTED_CONSTRAINT` for mise's own platform gate.** Mise retains only formula/cask dependency names from upstream metadata. It separately allows macOS and a documented font-only subset on Linux; docs say this boundary will expand with portable artifact implementations. No source found explains not deserializing or enforcing upstream minimum/maximum macOS, Linux, or arch requirements. — [mise metadata `cask.rs:71-92`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L71-L92), [mise platform gate `cask.rs:5030-5062`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L5030-L5062), [current docs `brew.md:159-175`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L159-L175), [Homebrew requirements `depends_on.rb:16-30`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/dsl/depends_on.rb#L16-L30) (confidence: HIGH for effect, LOW for unavailable motive) |
| Supported artifact subset | **`DOCUMENTED_CONSTRAINT` + `INCREMENTAL_GAP`.** Initial support intentionally stopped at apps. #10587 called simple pkg without custom choices a “smaller, safer first step”; #10837 added supported hooks; the #11962–#11964 stack added structured symlink/generic/copy/installer/dependency work. The current parser also supports binaries, fonts, completions, and wrappers, but those introductions are not attributed to that stack here. Current docs still enumerate unsupported shapes and require explicit failure rather than delegation. — [PR #10587](https://github.com/jdx/mise/pull/10587), [PR #10837](https://github.com/jdx/mise/pull/10837), [stack #11962](https://github.com/jdx/mise/pull/11962), [#11963](https://github.com/jdx/mise/pull/11963), [#11964](https://github.com/jdx/mise/pull/11964), [current parser `cask.rs:4944-5027`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L4944-L5027), [current docs `brew.md:177-205`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L177-L205) (confidence: HIGH) |
| Artifact phase ordering | **`UNEXPLAINED`.** Homebrew orders typed artifacts through `ArtifactSet`; mise manually sequences installer, app, pkg, font/wrapper/generic, hooks, binaries and completions. The hook and later artifact PRs explain individual prerequisites, but none states why the final aggregate ordering differs at the `pkg`/app and postflight/link boundaries. — [mise install sequence `cask.rs:556-653`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L556-L653), [Homebrew phase order `abstract_artifact.rb:71-123`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/artifact/abstract_artifact.rb#L71-L123), [PR #10837](https://github.com/jdx/mise/pull/10837), [PR #11964](https://github.com/jdx/mise/pull/11964) (confidence: HIGH for effect, LOW for unavailable motive) |
| Download strategy and checksum rules | **`DOCUMENTED_INTENTIONAL` for direct download/verification; `UNEXPLAINED` for the stricter missing-checksum rule and partial strategy coverage.** Direct download and checksum verification were core promises from #10383. Current mise accepts SHA-256 or `no_check` and otherwise fails; official Homebrew can warn and continue for some non-official casks and supports a broader strategy model. No source found states why mise chose the stricter missing-checksum behavior beyond the general direct-engine safety posture. — [initial `cask.rs:174-217`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L174-L217), [current mise `cask.rs:973-1051`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L973-L1051), [Homebrew checksum rules `download.rb:42-76,214-249`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/download.rb#L42-L76), [PR #10383](https://github.com/jdx/mise/pull/10383) (confidence: HIGH for effect, MED for safety rationale) |
| Git downloads and `only_path` staging topology | **`INCREMENTAL_GAP` for Git support; `UNEXPLAINED` for the different staged topology.** #11781's title and merged diff establish Git URL, branch, and selected-subdirectory support after the archive-only engine; its empty body supplies no rationale. Current mise moves the selected directory's children to the extraction root; Homebrew keeps `only_path` as the artifact-resolution base. No primary source in the bounded corpus explains the topology difference. — [PR #11781](https://github.com/jdx/mise/pull/11781), [mise `cask.rs:973-1023`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L973-L1023), [Homebrew Git strategy `git_download_strategy.rb:303-314`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/download_strategy/git_download_strategy.rb#L303-L314), [Homebrew artifact base `relocated.rb:56-62`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/artifact/relocated.rb#L56-L62) (confidence: HIGH for effect, LOW for unavailable motive) |
| Quarantine semantics | **`DOCUMENTED_INTENTIONAL`.** #10626 deliberately strips quarantine from the downloaded archive and installed app to address “damaged app” Gatekeeper failures after the old copy path lost macOS metadata. That is a package-fix rationale, not an assertion that Homebrew also strips quarantine; official Homebrew instead quarantines the cache and propagates it. — [PR #10626](https://github.com/jdx/mise/pull/10626), [mise at introducing commit `82abac4f14`, archive removal `cask.rs:270-276`](https://github.com/jdx/mise/blob/82abac4f1461d6aba72160c1a75d41f3a150c07b/src/system/packages/brew/cask.rs#L270-L276), [same commit, app removal `cask.rs:363-368`](https://github.com/jdx/mise/blob/82abac4f1461d6aba72160c1a75d41f3a150c07b/src/system/packages/brew/cask.rs#L363-L368), [current mise `cask.rs:1037-1044`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L1037-L1044), [Homebrew download `download.rb:35-77,92-129`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/download.rb#L35-L77) (confidence: HIGH) |
| Limited Ruby Cask DSL shim | **`DOCUMENTED_INTENTIONAL` + `DOCUMENTED_CONSTRAINT` + `INCREMENTAL_GAP`.** #10837 explicitly chose a mise-owned, checksum-bound shim and no Homebrew fallback; unsupported DSL fails loudly. #10950 expanded it after real casks failed. Current docs retain that supported-subset contract. — [PR #10837](https://github.com/jdx/mise/pull/10837), [maintainer comment](https://github.com/jdx/mise/discussions/10582#discussioncomment-17563994), [PR #10950](https://github.com/jdx/mise/pull/10950), [current source fetch/execute `cask.rs:1097-1212`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L1097-L1212), [current docs `brew.md:189-205`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L189-L205) (confidence: HIGH) |
| Mise receipt instead of native `.metadata` | **`DOCUMENTED_INTENTIONAL`.** A mise receipt existed from the first implementation. #11215 later made the ownership decision explicit: `.mise-cask.toml` owns direct pours, while mise must not create or imitate Homebrew's private metadata; the receipt records historical facts and transaction safety. Homebrew's installed predicate searches its own `.metadata` caskfile topology, proving reciprocal recognition is absent. — [initial receipt `cask.rs:41-45,408-418`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L41-L45), [current receipt/status `cask.rs:6557-6720`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6557-L6720), [Homebrew `caskroom.rb:46-62`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/caskroom.rb#L46-L62), [PR #11215](https://github.com/jdx/mise/pull/11215), [current docs `brew.md:207-220`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L207-L220) (confidence: HIGH) |
| Native `.metadata` preservation/refusal | **`REGRESSION` historically; `DOCUMENTED_INTENTIONAL` today.** Mise originally counted `.metadata` as another version, reinstalled, and deleted it as stale; the maintainer identified that as a direct-engine bug and #11012 preserved the directory. #11215 then hardened preservation into a pre-mutation refusal rather than cross-manager takeover. — [report #11007](https://github.com/jdx/mise/discussions/11007), [maintainer diagnosis](https://github.com/jdx/mise/discussions/11007#discussioncomment-17651553), [PR #11012](https://github.com/jdx/mise/pull/11012), [current refusal `cask.rs:428-447,529-542`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L428-L447), [PR #11215](https://github.com/jdx/mise/pull/11215) (confidence: HIGH) |
| Conflict recognition | **`INCREMENTAL_GAP` for mise conflict support; `UNEXPLAINED` for asymmetric recognition.** #11964 added conflict checks with dependency installation. Current mise treats any non-hidden Caskroom version as a conflicting cask, including Homebrew state; official Homebrew resolves conflicts through its native `.metadata`-backed installed predicate, so a mise-only receipt is not reciprocal. No inspected source states why these predicates differ beyond the separate receipt architecture. — [PR #11964](https://github.com/jdx/mise/pull/11964), [mise conflict check `cask.rs:449-456,6392-6407`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L449-L456), [Homebrew conflict check `installer.rb:234-255`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/installer.rb#L234-L255), [Homebrew installed predicate `cask.rb:246-252`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/cask.rb#L246-L252) (confidence: HIGH for effect, LOW for unavailable motive) |
| Installed-health model | **`DOCUMENTED_INTENTIONAL` today; `REGRESSION` for the replaced fingerprint rule.** #11215 intentionally made completed receipt plus content fingerprints the health proof, because status must use install-time facts, not today's API. That made any app content drift look missing and caused reinstallation/TCC loss. #12222 explicitly fixed this: app/font health is now presence/kind-based; symlinks retain destination and resolution checks; fingerprints remain only for adopt/prune safety. — [PR #11215](https://github.com/jdx/mise/pull/11215), [PR #12222](https://github.com/jdx/mise/pull/12222), [current health `cask.rs:6557-6606,6730-6757`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6557-L6606), [current docs `brew.md:207-220`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L207-L220) (confidence: HIGH) |
| Adoption and target ownership | **`DOCUMENTED_INTENTIONAL`.** #12074 deliberately adds adoption into mise ownership: non-self-updating apps require identical content; `auto_updates` apps may be adopted as-is; mise records metadata-only ownership without a duplicate Caskroom app. It is not adoption of Homebrew `.metadata`, which remains an earlier refusal. Current code separately implements target validation and receipt claims within mise's safety model. — [PR #12074](https://github.com/jdx/mise/pull/12074), [current adopt path `cask.rs:523-542,1255-1342`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L523-L542), [current docs `brew.md:92-129`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L92-L129) (confidence: HIGH) |
| Reinstall, upgrade, and `auto_updates` | **`DOCUMENTED_INTENTIONAL` for ordinary mise behavior; `INCREMENTAL_GAP` historically; `UNEXPLAINED` for no forced same-version reinstall.** Upgrade has always reused install convergence, so a healthy current-version cask is a no-op. #11107 initially only stopped the shim rejecting `auto_updates`; #12074 later made current API `auto_updates` authoritative, allowed receipt drift, and explicitly skipped ordinary mise upgrades. This differs from Homebrew's named greedy upgrade behavior by stated choice. No primary source found a direct-live-bundle version detector or a forced reinstall design for mise. — [initial upgrade `cask.rs:169-170`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L169-L170), [PR #11107](https://github.com/jdx/mise/pull/11107), [PR #12074](https://github.com/jdx/mise/pull/12074), [current install/upgrade `cask.rs:436-447,829-895`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L436-L447), [Homebrew named-cask upgrade `upgrade.rb:39-79`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/upgrade.rb#L39-L79), [docs `brew.md:120-129`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L120-L129) (confidence: HIGH for effect; LOW for unavailable reinstall motive) |
| No full uninstall; conservative prune | **`INCREMENTAL_GAP` + `DOCUMENTED_INTENTIONAL` for the safe prune subset; `UNEXPLAINED` for no imperative uninstall.** Discussion #11784 says cask import/prune had been left out of scope and asks what removal semantics are safe. The maintainer accepted only a conservative ownership boundary. #11810 persists install-time eligibility, revalidates, skips pkg/lifecycle/Homebrew/legacy/drift/shared/transaction cases, and never executes zap or reconstructs uninstall from current API. This proves intentional conservative prune, but not an explicit rejection of a general uninstall operation. — [discussion #11784](https://github.com/jdx/mise/discussions/11784), [maintainer scope](https://github.com/jdx/mise/discussions/11784#discussioncomment-17952959), [PR #11810](https://github.com/jdx/mise/pull/11810), [current blockers `cask.rs:6609-6641`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6609-L6641), [current prune `cask.rs:6921-7196`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6921-L7196), [docs `brew.md:309-326`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L309-L326) (confidence: HIGH for prune; MED for unavailable uninstall motive) |
| Dependency install versus dependency-safe prune | **`INCREMENTAL_GAP` for installation; `UNEXPLAINED` for prune.** #11964 added API formula/cask dependencies and conflicts as a bounded cask-shape expansion. Current install recursively installs them. The later/current prune planner uses configured names, ownership receipts, target claims, and safety gates but no dependent-cask graph; neither #11810 nor #11964 explains this asymmetry. — [PR #11964](https://github.com/jdx/mise/pull/11964), [current dependency path `cask.rs:428-486`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L428-L486), [current prune planner `cask.rs:6921-7146`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6921-L7146) (confidence: HIGH for effect, LOW for unavailable prune motive) |
| Linux/platform behavior | **`DOCUMENTED_CONSTRAINT` + `INCREMENTAL_GAP`.** Casks began macOS-only. #11758 added only font casks on Linux, and current docs explicitly call that an initial portable subset which may expand. This is mise's own portability policy, not Homebrew's cask platform-requirement semantics. — [initial availability `cask.rs:97-109`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L97-L109), [PR #11758](https://github.com/jdx/mise/pull/11758), [current gate `cask.rs:5030-5062`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L5030-L5062), [docs `brew.md:159-175`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L159-L175) (confidence: HIGH) |
| Transaction/rollback model | **`INCREMENTAL_GAP` + `DOCUMENTED_INTENTIONAL`.** Initial installs used a temp Caskroom but directly replaced app targets with no durable journal. #11215 identified missing validation and historical-state boundaries, then selected a durable journal, delayed final receipt, rollback, fingerprints, and producer isolation. #11962/#11963 extended target backup/recovery for structured writes. This is a separate safety model because mise executes a typed subset directly. By inference from the prune blockers and external-command boundary, vendor side effects from pkg or installer commands are not covered by a complete filesystem transaction; the sources do not claim full rollback parity. — [initial install `cask.rs:52-87,255-273`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L52-L87), [PR #11215](https://github.com/jdx/mise/pull/11215), [PR #11962](https://github.com/jdx/mise/pull/11962), [PR #11963](https://github.com/jdx/mise/pull/11963), [current activation `cask.rs:556-713`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L556-L713), [prune blockers `cask.rs:6609-6641`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6609-L6641) (confidence: HIGH for implemented transaction; MED for inferred boundary) |
| Batch failure propagation | **`UNEXPLAINED`.** The first manager loop returned immediately on the first error, and current code retains that behavior. No inspected history states that stopping a mise manager batch is a deliberate Homebrew divergence. Official Homebrew rescues per-cask install failures and continues. — [initial mise loop `cask.rs:138-167`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L138-L167), [current mise loop `cask.rs:386-406`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L386-L406), [Homebrew install loop `cmd/install.rb:455-473`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cmd/install.rb#L455-L473) (confidence: HIGH for effect, LOW for unavailable motive) |

### What does history establish—and not establish—about “why”?

- It establishes one controlling rationale: mise wants Homebrew artifacts without
  requiring Homebrew. It also establishes three deliberate policies inside that
  architecture: unsupported shapes fail rather than delegate, direct casks use
  mise-owned receipts rather than imitating private Homebrew metadata, and
  destructive cleanup fails closed unless mise can prove ownership and unchanged
  targets. — [PR #10383](https://github.com/jdx/mise/pull/10383),
  [maintainer comment](https://github.com/jdx/mise/discussions/10582#discussioncomment-17563994),
  [PR #11215](https://github.com/jdx/mise/pull/11215),
  [maintainer prune scope](https://github.com/jdx/mise/discussions/11784#discussioncomment-17952959)
  (confidence: HIGH)

- It does **not** establish that every semantic difference was chosen. Variation
  selection, upstream platform constraints, aggregate phase ordering,
  dependency-aware prune, and batch continuation have observable differences but
  no recovered rationale. Calling those “intentional design” would infer motive
  from code shape, so this chapter labels them `UNEXPLAINED`. — evidence in the
  corresponding ledger rows (confidence: MED for absence within the bounded
  corpus; LOW for any hypothetical motive)

- History includes genuine regressions, not merely unsupported parity: deletion
  of Homebrew `.metadata` and app-content drift triggering replacement were both
  reported, diagnosed, and fixed as bugs. Therefore “independent engine” does not
  make any non-Homebrew behavior intentional by default. — [discussion
  #11007](https://github.com/jdx/mise/discussions/11007), [PR
  #11012](https://github.com/jdx/mise/pull/11012), [PR
  #12222](https://github.com/jdx/mise/pull/12222) (confidence: HIGH)

## Dead ends and contradictions

- The proposition “mise is intended to call official Homebrew whenever exact
  behavior matters” is contradicted by the introduction and later maintainer
  statement: direct operation without Homebrew and explicit unsupported failure
  were deliberate. — [PR #10383](https://github.com/jdx/mise/pull/10383),
  [maintainer comment](https://github.com/jdx/mise/discussions/10582#discussioncomment-17563994)
  (confidence: HIGH)

- The proposition “the public cask JSON is already the correct current-host,
  current-language cask” is contradicted by Homebrew's generator/loader. It is a
  base record plus variations and localization, while mise currently reads only
  the base fields. — [Homebrew `cask_struct_generator.rb:11-35`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/api/cask/cask_struct_generator.rb#L11-L35),
  [Homebrew `cask.rb:620-696`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/cask.rb#L620-L696),
  [mise `cask.rs:54-83`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L54-L83)
  (confidence: HIGH)

- The proposition “own receipt topology was an accidental temporary file” is
  contradicted by #11215 and current docs, which explicitly define it as mise's
  lifecycle ownership boundary and prohibit imitating private `.metadata`. — [PR
  #11215](https://github.com/jdx/mise/pull/11215), [current docs
  `brew.md:207-220`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L207-L220)
  (confidence: HIGH)

- GitHub PR/discussion prose often says “Homebrew-compatible” for one local
  behavior, while current source proves broader non-parity. Those phrases cannot
  support a claim of whole-engine compatibility; this chapter uses them only for
  the named behavior. — examples: [PR #11215](https://github.com/jdx/mise/pull/11215),
  [PR #12074](https://github.com/jdx/mise/pull/12074), and the exact comparison
  ledger above (confidence: HIGH)

## Open unknowns

- No primary source found states whether full variation/language/platform
  selection is intended future work, rejected scope, or simply not yet reported.
  The current absence and official requirement are proven; motive is unknown. —
  [mise `cask.rs:54-92`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L54-L92),
  [Homebrew `cask.rb:620-696`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/cask/cask.rb#L620-L696)
  (confidence: HIGH for status)

- No primary source found explains the different aggregate artifact phase order,
  first-error batch stop, or missing dependent-cask protection in prune. These are
  not safely classifiable as deliberate. — [mise sequence
  `cask.rs:556-653`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L556-L653),
  [batch loop `:386-406`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L386-L406),
  [prune plan `:6921-7146`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6921-L7146)
  (confidence: HIGH for status)

- No maintainer statement found defines an end-state compatibility contract for
  direct casks. The evidence defines independence, fail-loudly expansion, and
  producer safety; it does not say whether supported operations should eventually
  match every observable Homebrew result. That remains a product decision, not a
  research finding. — [PR #10383](https://github.com/jdx/mise/pull/10383),
  [maintainer comment](https://github.com/jdx/mise/discussions/10582#discussioncomment-17563994),
  [PR #11215](https://github.com/jdx/mise/pull/11215) (confidence: HIGH)
