# 05 — Why mise's formula engine differs from Homebrew

## Questions

- Why is mise's current `brew:` engine an independent implementation instead of
  using Homebrew's Ruby engine and state machine?
- For every formula-engine difference already catalogued in chapters 01 and 02,
  is the current behavior a documented constraint, a documented intentional
  choice, an incremental gap, a regression, or unexplained?
- Which differences have historical evidence of intent, and which have only an
  observable implementation effect with no recorded rationale?

Informs: mise-bootstrap-brew-interop

## Method

- Read-only history analysis of [`jdx/mise`](https://github.com/jdx/mise) at
  current `main`, commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`.
  Method: filtered full-history clone outside the working repository; `git log
  --follow`, `git log -S`, and `git blame` over
  `src/system/packages/brew/{api,resolve,pour,source,prefix,maintenance}.rs`,
  `shim.rb`, the shared package-manager trait, and the Brew documentation.
- Read the merged primary pull requests that introduced or materially changed
  the formula engine: [#10326](https://github.com/jdx/mise/pull/10326),
  [#10346](https://github.com/jdx/mise/pull/10346),
  [#10364](https://github.com/jdx/mise/pull/10364),
  [#10375](https://github.com/jdx/mise/pull/10375),
  [#10383](https://github.com/jdx/mise/pull/10383),
  [#10606](https://github.com/jdx/mise/pull/10606),
  [#10618](https://github.com/jdx/mise/pull/10618),
  [#11320](https://github.com/jdx/mise/pull/11320),
  [#11330](https://github.com/jdx/mise/pull/11330),
  [#11371](https://github.com/jdx/mise/pull/11371), and
  [#11665](https://github.com/jdx/mise/pull/11665). Pull-request bodies,
  maintainer-authored commits, code comments, tests, and current documentation
  were treated as evidence; generated review summaries were not used as motive
  evidence. Merged PR prose is treated as an accepted project artifact, not as
  proof that jdx personally authored every stated rationale.
- Read-only history analysis of [`Homebrew/brew`](https://github.com/Homebrew/brew)
  at `24f8780838c054a92ebf58b09455acbb236731bd`, plus historical and current
  `homebrew-core` formula revisions. Direct API responses for `ca-certificates`
  and `openssl@3` were fetched on 2026-08-22. No installation or mutation test
  was run; runtime interoperability remains outside this chapter's method.
  Those live API URLs support only the stated current result; unlike the pinned
  source links, they are not immutable historical snapshots.
- Labels mean: **DOCUMENTED_CONSTRAINT** = an explicit surrounding architecture
  or supported-platform boundary; **DOCUMENTED_INTENTIONAL** = the differing
  behavior itself was explicitly selected; **INCREMENTAL_GAP** = the intended
  compatibility surface is incomplete and history shows piecemeal extension or
  repair; **REGRESSION** = previously implemented behavior was later broken;
  **UNEXPLAINED** = no primary source states why. A label never substitutes for
  the separately stated implementation effect.

Vetted: 2026-08-22

## Findings

### What was the actual design goal?

- The first formula engine was explicitly introduced as **“brew without brew,”**
  not as a Rust front-end to an installed Homebrew. Its stated purpose was to
  install machine-global Homebrew formulae on a bootstrap target where Homebrew
  may not exist: fetch static API metadata, resolve dependencies client-side,
  pour bottles directly, and create Homebrew-compatible filesystem state. The
  current module still says it never invokes `brew`, directly owns bottle pour
  and source build, and only claims compatibility of the resulting kegs and
  receipts. — [mise `mod.rs:1-18`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L1-L18), [originating PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- The canonical prefix was the load-bearing interoperation choice. The
  originating PR states that shared-library bottles and their dependency
  closures depend on fixed canonical paths; current documentation repeats that
  this is why they cannot be served from ordinary per-project mise backends.
  Thus mise reused Homebrew's artifacts and namespace while replacing the
  program that normally owns their lifecycle. — [mise Brew docs
  `brew.md:222-226`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L222-L226), [originating PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- Consequently, the large design difference is not accidental: official
  Homebrew loads formula objects and runs its Ruby installer, while mise
  reimplemented selected observable stages behind the generic mise system
  package-manager contract. What is not established is that every smaller
  semantic difference was consciously selected. The historical record instead
  separates a documented no-Homebrew architecture from many later omissions
  and repairs. — [mise package-manager trait
  `mod.rs:135-216`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L135-L216), [Homebrew installer
  `formula_installer.rb:987-1034`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L987-L1034) (confidence: HIGH for the architecture; MED for the absence of finer-grained intent)

### Does history show a complete alternate design or incremental accretion?

- It shows rapid incremental accretion. The originating change explicitly scoped
  itself to current core bottles and excluded taps, casks, services, Intel Macs,
  and source builds. On June 12, #10364 added a limited source DSL shim, #10375
  first delegated third-party taps to an installed `brew`, and #10383 replaced
  that delegation with direct tap API consumption while adding casks. Import
  and prune arrived June 24 in #10606, and formula-prune ownership was reversed
  the next day in #10618. — [PR #10326](https://github.com/jdx/mise/pull/10326),
  [PR #10364](https://github.com/jdx/mise/pull/10364),
  [PR #10375](https://github.com/jdx/mise/pull/10375),
  [PR #10383](https://github.com/jdx/mise/pull/10383),
  [PR #10606](https://github.com/jdx/mise/pull/10606),
  [PR #10618](https://github.com/jdx/mise/pull/10618) (confidence: HIGH)
- Later fixes repeatedly copied missing official semantics after concrete
  failures: recognizing Homebrew directory links (#11320), correct one-hop link
  ownership in prune (#11330), linked-keg records and repair state (#11371), and
  text relocation in `:any_skip_relocation` bottles (#11665). This establishes
  a pattern of compatibility being extended and corrected feature by feature;
  it does not prove an unstated motive for any remaining gap. — [PR
  #11320](https://github.com/jdx/mise/pull/11320), [PR
  #11330](https://github.com/jdx/mise/pull/11330), [PR
  #11371](https://github.com/jdx/mise/pull/11371), [PR
  #11665](https://github.com/jdx/mise/pull/11665) (confidence: HIGH)

### Status ledger: metadata, API, taps, and versions

- **Core metadata — DOCUMENTED_INTENTIONAL / DOCUMENTED_CONSTRAINT.** Current
  mise fetches per-formula JSON from `formulae.brew.sh`, deserializes only fields
  its direct engine uses, and holds results in its HTTP client's process cache.
  This was explicitly chosen so bootstrap does not require a Homebrew checkout
  or CLI. Official Homebrew can also load core API data, but materializes a full
  Formula model and retains a Ruby-source path when needed; mise's smaller
  schema is the boundary of its independent engine. — [mise `api.rs:13-47`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/api.rs#L13-L47), [mise `api.rs:138-177`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/api.rs#L138-L177), [Homebrew `api/formula.rb:18-116`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/api/formula.rb#L18-L116), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- **Third-party taps — DOCUMENTED_INTENTIONAL / DOCUMENTED_CONSTRAINT.** #10375
  initially routed non-core formulae through real `brew`; #10383 deliberately
  removed that fallback to preserve no-Homebrew bootstrap. Current mise accepts
  only GitHub raw bases that publish `api/formula/<name>.json`, whereas official
  Homebrew installs a tap Git checkout and loads its Formula Ruby through
  Formulary. The
  narrower supported tap shape is therefore intentional and documented, not an
  unexplained failure to discover Homebrew's tap mechanism. — [mise
  `api.rs:148-177`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/api.rs#L148-L177), [mise Brew docs
  `brew.md:23-48`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L23-L48), [Homebrew
  `tap.rb:639-761`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/tap.rb#L639-L761), [Homebrew
  `formulary.rb:702-766`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formulary.rb#L702-L766), [PR #10375](https://github.com/jdx/mise/pull/10375), [PR #10383](https://github.com/jdx/mise/pull/10383) (confidence: HIGH)
- **Version selection — DOCUMENTED_CONSTRAINT.** The generic configuration can
  represent pins, but mise rejects formula version values because its direct
  path consumes only the currently published stable bottle; versioned formula
  names such as `postgresql@17` remain names. Official Homebrew has richer
  installed-version and spec selection, but arbitrary historical formula
  version installation was explicitly outside this engine's artifact source.
  — [mise `mod.rs:76-85`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L76-L85), [mise Brew docs
  `brew.md:416-419`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L416-L419), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)

### Status ledger: dependency closure

- **Client-side closure — DOCUMENTED_INTENTIONAL.** Mise recursively consumes
  API runtime dependencies, adds build dependencies only for source-built
  formulae, applies the selected bottle/host variation, canonicalizes aliases,
  breaks cycles, and returns dependency-first order. Both the origin and source
  follow-up describe this as work mise must do because it does not call
  Homebrew. — [mise `resolve.rs:37-59`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/resolve.rs#L37-L59), [mise
  `resolve.rs:77-234`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/resolve.rs#L77-L234), [PR #10326](https://github.com/jdx/mise/pull/10326), [PR #10364](https://github.com/jdx/mise/pull/10364) (confidence: HIGH)
- **Official dependency satisfaction and replacement — UNEXPLAINED.** Mise's
  resolver has no model for Homebrew build options,
  receipt-supplied minimum compatible runtime dependency versions, or
  transactional replacement of an existing dependency keg. It checks whether
  the exact current keg directory exists and installs missing closure members.
  The no-Homebrew architecture explains why mise needs its own resolver, but no
  reviewed primary source explains why these specific official predicates and
  rollback semantics were omitted. — [mise `mod.rs:100-121`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L100-L121), [Homebrew
  `dependency.rb:62-139`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/dependency.rb#L62-L139), [Homebrew
  `formula_installer.rb:879-948`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L879-L948) (confidence: HIGH for the effect; MED that the enumerated history supplied no rationale)

### Status ledger: bottle fetch, selection, relocation, and pour

- **Direct bottle path — DOCUMENTED_INTENTIONAL / DOCUMENTED_CONSTRAINT.** Mise
  independently selects a supported tag, downloads and SHA-256 verifies the
  bottle, extracts inside the Cellar, relocates text and binaries, re-signs
  changed Mach-O files, writes a receipt, and links. This is the core
  no-Homebrew implementation, deliberately limited to arm64 macOS and
  x86_64/arm64 Linux. — [mise `tag.rs:1-64`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/tag.rs#L1-L64), [mise
  `pour.rs:191-270`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L191-L270), [mise Brew docs
  `brew.md:328-355`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L328-L355), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- **Exact pour parity — INCREMENTAL_GAP.** The engine's stated goal for this
  stage is to mirror `brew pour`, but the history contains concrete missed
  semantics later repaired. #11665 fixed a case where mise skipped all
  relocation for `:any_skip_relocation`, although Homebrew skips only binary
  linkage relocation and still rewrites text. #11320 and #11330 fixed Homebrew
  link-layout assumptions that broke upgrade and prune. These are verified
  incremental gaps, not evidence that deviation was desired. — [mise current
  `pour.rs:232-246`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L232-L246), [PR #11665](https://github.com/jdx/mise/pull/11665), [PR #11320](https://github.com/jdx/mise/pull/11320), [PR #11330](https://github.com/jdx/mise/pull/11330) (confidence: HIGH)
- **Homebrew's broader bottle eligibility — DOCUMENTED_CONSTRAINT.** Mise does
  not expose Homebrew's force-source, build-bottle, compiler, options, or
  formula-specific `pour_bottle?` command surface; it picks a compatible bottle
  or enters its own source fallback. The bootstrap engine intentionally models
  current declarative package convergence, not all `brew install` modes. The
  exact omission of each individual eligibility predicate has no separate
  rationale. — [mise `source.rs:33-58`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/source.rs#L33-L58), [Homebrew
  `formula_installer.rb:255-303`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L255-L303), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH for the scoped command model; MED for per-predicate intent)

### Status ledger: source builds and the Ruby shim

- **Own Ruby shim — DOCUMENTED_INTENTIONAL / DOCUMENTED_CONSTRAINT.** Source
  builds were added specifically to preserve “Homebrew formulae without
  Homebrew.” Mise provisions Ruby through mise, downloads commit-pinned Formula
  Ruby and checksummed source, evaluates a locally implemented DSL subset, runs
  `install` and `post_install`, writes the same partial receipt, and links via
  its Rust path. Unsupported install-time helpers and unverified/VCS source
  forms deliberately fail loudly rather than produce a misleading canonical
  keg. — [mise `source.rs:1-10`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/source.rs#L1-L10), [mise
  `source.rs:60-94`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/source.rs#L60-L94), [mise
  `shim.rb:854-907`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/shim.rb#L854-L907), [PR #10364](https://github.com/jdx/mise/pull/10364) (confidence: HIGH)
- **Full Formula DSL/build environment parity — DOCUMENTED_CONSTRAINT.** #10364
  expressly lists missing language helpers and superenv compiler shims as known
  limitations; current docs still promise only common formula shapes. This is a
  deliberate supported subset, unlike the unadvertised bottle post-install gap
  below. — [mise Brew docs
  `brew.md:357-385`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L357-L385), [mise Brew docs
  `brew.md:412-415`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L412-L415), [PR #10364](https://github.com/jdx/mise/pull/10364) (confidence: HIGH)

### Status ledger: receipts and producer identity

- **Homebrew-shaped receipts — DOCUMENTED_INTENTIONAL.** Interoperation with a
  later-installed real Homebrew was an explicit launch requirement. Mise
  serializes `INSTALL_RECEIPT.json` itself, including request/dependency status,
  bottle/source status, runtime deps, changed files, source tap, architecture,
  and a mise-marked Homebrew version. Current docs claim `brew
  list/upgrade/uninstall` compatibility. — [mise `pour.rs:299-375`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L299-L375), [mise Brew docs
  `brew.md:248-255`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L248-L255), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH for intent and static format)
- **Exact receipt parity — UNEXPLAINED.** Mise overwrites the
  bottle's native tab with a hand-built subset: for example, `tap_git_head` is
  null, `built_on` is empty, and it emits obsolete `installed_as_dependency`
  beside current `installed_on_request`. Official Homebrew records more source,
  build, platform, and API provenance. The historical rationale establishes
  compatibility intent but does not explain each omitted field, and this method
  did not execute Homebrew against a mise-written receipt. — [mise
  `pour.rs:336-374`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L336-L374), [Homebrew
  `tab/tab.rb:367-419`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/tab/tab.rb#L367-L419), [Homebrew
  `tab.rb:25-89`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/tab.rb#L25-L89) (confidence: HIGH for static difference; MED for runtime compatibility)
- **Formula producer conflation — DOCUMENTED_INTENTIONAL.** Unlike the cask
  engine, formula status intentionally treats the shared prefix as the source
  of truth regardless of whether mise or real Homebrew created a keg. The
  original PR promised reciprocal visibility, and current comments explicitly
  preserve it. This explains why formula presence has no producer ownership
  distinction; it does not by itself justify every mutation made later through
  that shared state. — [mise `mod.rs:236-251`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L236-L251), [mise Brew docs
  `brew.md:248-265`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L248-L265), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)

### Status ledger: prefix and filesystem ownership

- **Canonical shared prefix and bootstrap — DOCUMENTED_INTENTIONAL /
  DOCUMENTED_CONSTRAINT.** Mise creates Homebrew's standard prefix layout and
  may elevate for `mkdir`/`chown`, because the engine must work before Homebrew
  is installed and bottles require canonical paths. Current docs state that the
  one-time setup mirrors Homebrew's installer; subsequent installs are ordinary
  user file operations. — [mise `prefix.rs:1-41`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/prefix.rs#L1-L41), [mise
  `prefix.rs:167-243`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/prefix.rs#L167-L243), [mise Brew docs
  `brew.md:241-246`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L241-L246), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- **Recursive ownership repair of an existing prefix — UNEXPLAINED.** Current
  code also runs recursive `chown -R` when an existing prefix is judged
  non-writable, not only when it is newly created. The user documentation says
  the elevation is for an absent prefix, and the reviewed history supplies no
  separate rationale for transferring an existing tree's ownership. Official
  Homebrew separately expects keg/public-link paths to be writable and refuses
  ordinary root execution; the cited code does not show Homebrew recursively
  taking ownership during formula installation.
  — [mise `prefix.rs:181-230`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/prefix.rs#L181-L230), [Homebrew
  `keg.rb:143-185`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/keg.rb#L143-L185), [Homebrew
  `brew.sh:159-200`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/brew.sh#L159-L200) (confidence: HIGH for behavior and documentation mismatch; MED that the enumerated sources state no finer rationale)

### Status ledger: links and active records

- **Compatible link shape — DOCUMENTED_INTENTIONAL.** Mise creates `opt`, public
  prefix links, and the linked-keg record, protects foreign occupied paths, and
  rolls back links it replaced on failure. The intent is explicitly to produce
  a tree a real Homebrew recognizes, not to invent a mise-only link namespace.
  — [mise `pour.rs:546-643`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L546-L643), [mise Brew docs
  `brew.md:248-267`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L248-L267), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- **Exact Homebrew link semantics — INCREMENTAL_GAP.** Initial compatibility
  omitted Homebrew directory symlinks and its linked-keg record. #11320 repaired
  directory-link ownership/expansion, and #11371 added the record plus a narrow
  repair state. That history labels the remaining implementation as a parity
  port still corrected by focused cases, not a deliberately different link
  model. Official Homebrew still has richer overwrite, link-failure, and
  partial-link state behavior than mise's all-or-error link transaction. — [PR
  #11320](https://github.com/jdx/mise/pull/11320), [PR #11371](https://github.com/jdx/mise/pull/11371), [Homebrew
  `keg.rb:490-580`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/keg.rb#L490-L580), [Homebrew
  `formula_installer.rb:1195-1286`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L1195-L1286) (confidence: HIGH)

### Status ledger: finish, `.bottle/etc`, `.bottle/var`, and post-install

- **Bottle finish omission — UNEXPLAINED.** Current bottle
  success ends after receipt and linking. It does not restore `.bottle/etc` or
  `.bottle/var`, run structured `post_install_steps`, or run Formula Ruby
  `post_install`; source builds happen to run `post_install` inside the shim.
  Official Homebrew makes shared-file restoration and post-install separate
  finish stages after linking. The origin described parity only for “pour
  time,” but no primary source says these lifecycle effects should be omitted
  from a working installed formula. — [mise `mod.rs:170-210`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L170-L210), [mise
  `pour.rs:245-270`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L245-L270), [mise
  `shim.rb:886-902`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/shim.rb#L886-L902), [Homebrew
  `formula_installer.rb:987-1017`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L987-L1017), [Homebrew
  `formula_installer.rb:1390-1427`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L1390-L1427), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH for behavior; MED that the enumerated history contains no intended omission)
- **The API did not force this omission — UNEXPLAINED.** Homebrew added the
  structured install-step framework and Formula `post_install_steps` API before
  mise's originating engine landed. Mise's current Formula schema still has no
  `post_install_steps` field. However, the two load-bearing formulas were only
  converted from legacy Ruby later: `openssl@3` on July 6 and `ca-certificates`
  on July 20. Thus, by inference, current structured data can supply a lifecycle
  plan, but mise still needs a compatible executor; there is no historical
  evidence that mise evaluated and rejected that implementation. — [Homebrew install-steps framework commit
  `4c1998c7`](https://github.com/Homebrew/brew/commit/4c1998c75753bba578f895953237c0b3ffc2497d), [Homebrew Formula steps commit
  `003b9e24`](https://github.com/Homebrew/brew/commit/003b9e24920dc382267869fbcb7d5c4cbe09a193), [mise
  `api.rs:13-47`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/api.rs#L13-L47), [openssl conversion
  `e484941c`](https://github.com/Homebrew/homebrew-core/commit/e484941cdf071d644d5b9d83622f7f6cf5ef96c8), [ca-certificates conversion
  `dc8516ba`](https://github.com/Homebrew/homebrew-core/commit/dc8516ba23fc4c5585df20289b2eec17e775a0c9) (confidence: HIGH)

### Status ledger: installed health

- **Active `opt` as cross-producer presence — DOCUMENTED_INTENTIONAL.** Mise
  intentionally uses a live `opt/<name>` target as the basic installed
  predicate so a Homebrew-installed formula satisfies mise configuration and a
  disconnected Cellar remnant does not mask retry. #11371 later added a narrow
  `NeedsRepair` state for disagreement between `opt` and the linked-keg record.
  — [mise `mod.rs:236-251`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L236-L251), [PR #11371](https://github.com/jdx/mise/pull/11371) (confidence: HIGH)
- **Lifecycle-aware health — UNEXPLAINED.** Status does not
  parse the receipt, inspect dependency readiness, or verify shared/post-install
  artifacts. Official Homebrew also has no one durable generic
  lifecycle-complete flag, but it distinguishes keg presence, receipt-bearing
  installation, `opt`, and public linked state. No history source states that a
  successful link should certify formula runtime health; current simplicity
  dates to the original engine and was only partially expanded for link-record
  repair. — [mise `mod.rs:236-251`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L236-L251), [Homebrew
  `formula.rb:942-1076`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula.rb#L942-L1076), [Homebrew
  `dependency.rb:62-139`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/dependency.rb#L62-L139) (confidence: HIGH for the effect; MED for unavailable motive)

### Status ledger: OpenSSL and CA trust

- **Missing default trust — UNEXPLAINED, not a verified regression.** At the
  time mise's engine launched, `ca-certificates` already required Ruby
  `post_install` to generate `etc/ca-certificates/cert.pem`, and `openssl@3`
  already required Ruby `post_install` to point `etc/openssl@3/cert.pem` at it.
  Current Homebrew expresses the same outcomes as structured API steps. Mise's
  bottle path executes neither generation nor symlink, yet its active-link
  status reports both formulae installed. — [historical `ca-certificates.rb:18-30,113-114`](https://github.com/Homebrew/homebrew-core/blob/d19f58c6320e12d769a8b299ec2de947e289faa8/Formula/c/ca-certificates.rb#L18-L30), [historical `ca-certificates.rb:100-114`](https://github.com/Homebrew/homebrew-core/blob/d19f58c6320e12d769a8b299ec2de947e289faa8/Formula/c/ca-certificates.rb#L100-L114), [historical `openssl@3.rb:136-144`](https://github.com/Homebrew/homebrew-core/blob/84e4e32eecfd4abb49960ea23ddf90a64d94bc62/Formula/o/openssl%403.rb#L136-L144), [current `ca-certificates` API](https://formulae.brew.sh/api/formula/ca-certificates.json), [current `openssl@3` API](https://formulae.brew.sh/api/formula/openssl%403.json), [mise `mod.rs:170-210`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L170-L210) (confidence: HIGH)
- The chronology rules out calling this a newly introduced Homebrew behavior:
  the required formula effects predate mise's engine. It also does not satisfy
  the evidence threshold for **REGRESSION**, because no reviewed commit or test
  shows that the mise bottle path ever performed these effects and later lost
  them. The verified status is an original lifecycle gap whose current metadata
  representation became simpler after launch. — [mise origin commit
  `173403d5`](https://github.com/jdx/mise/commit/173403d57ec5e0c7e25589e801354525051783e7), [historical `ca-certificates`](https://github.com/Homebrew/homebrew-core/blob/d19f58c6320e12d769a8b299ec2de947e289faa8/Formula/c/ca-certificates.rb#L22-L28), [historical `openssl@3`](https://github.com/Homebrew/homebrew-core/blob/84e4e32eecfd4abb49960ea23ddf90a64d94bc62/Formula/o/openssl%403.rb#L141-L144) (confidence: MED within the enumerated implementation history)

### Status ledger: upgrade and old kegs

- **Install-current-bottle as upgrade — DOCUMENTED_INTENTIONAL.** The shared
  manager contract defaults upgrade to install, and #10346 explicitly selected
  that path for Brew because both operations resolve the current bottle and
  repoint links. — [mise package trait
  `mod.rs:193-200`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L193-L200), [mise
  `mod.rs:276-295`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L276-L295), [PR #10346](https://github.com/jdx/mise/pull/10346) (confidence: HIGH)
- **Removing the previous keg — UNEXPLAINED, not a verified regression.**
  #10346 and current docs say the new keg replaces the old. Current pour removes
  only an already-existing directory for the *same new version* before rename;
  earlier version directories remain, and formula prune inventories only active
  links. Inspection of the introduction revision shows the same same-version
  removal, so no implementation was found that later regressed. This is a
  documented promise not delivered by the code, with no rationale for the
  discrepancy. — [mise current `pour.rs:249-270`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L249-L270), [mise current
  `pour.rs:166-189`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L166-L189), [mise Brew docs
  `brew.md:390-397`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L390-L397), [implementation at upgrade introduction
  `b83edbfa`](https://github.com/jdx/mise/blob/b83edbfa7098e4395bc6ae7da41dfc3273e0354b/src/system/packages/brew/pour.rs#L126-L149), [mise
  `maintenance.rs:97-142,175-190`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/maintenance.rs#L97-L142), [PR #10346](https://github.com/jdx/mise/pull/10346) (confidence: HIGH for the effect; MED for unavailable motive)

### Status ledger: prune provenance and uninstall

- **Inventory-based prune, including Homebrew-owned formulae —
  DOCUMENTED_INTENTIONAL.** #10606 first implemented ledger-only removal of
  mise-installed/adopted kegs. Maintainer-authored #10618 deliberately removed
  that provenance ledger the next day and made config/tracked-config closure
  the source of truth, explicitly including formulae installed by real
  Homebrew. Current code and docs preserve that exact choice. This difference
  from producer-aware cask pruning is therefore not accidental. — [PR
  #10606](https://github.com/jdx/mise/pull/10606), [PR #10618](https://github.com/jdx/mise/pull/10618), [mise
  `maintenance.rs:145-190`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/maintenance.rs#L145-L190), [mise Brew docs
  `brew.md:296-307`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L296-L307) (confidence: HIGH)
- **Inactive old keg cleanup — UNEXPLAINED.** The deliberate inventory
  model starts from active `opt` links, so inactive versions left by upgrade are
  invisible and survive prune. Neither #10618 nor current docs describe this as
  desired; their stated unit is linked formulae. — [mise
  `maintenance.rs:97-142`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/maintenance.rs#L97-L142), [mise
  `maintenance.rs:175-190`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/maintenance.rs#L175-L190), [PR #10618](https://github.com/jdx/mise/pull/10618) (confidence: HIGH)
- **No mise formula uninstall command — DOCUMENTED_CONSTRAINT / UNEXPLAINED.**
  The generic package-manager contract exposes
  installed/install/upgrade; removal is the separate declarative `packages
  prune` workflow. #10606 framed that workflow against Bundle cleanup. This
  documents the available declarative surface, but not an explicit rejection of
  an imperative uninstall operation. The compatible receipt is intended to let
  actual `brew uninstall` operate on a mise keg.
  — [mise package-manager trait
  `mod.rs:140-216`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L140-L216), [mise Brew docs
  `brew.md:248-253`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L248-L253), [PR #10606](https://github.com/jdx/mise/pull/10606) (confidence: HIGH)

### Status ledger: supported platforms

- **arm64 macOS and x86_64/arm64 Linux only — DOCUMENTED_CONSTRAINT.** The
  origin explicitly excluded Intel Macs; current availability checks and docs
  continue that boundary. Bottle tag selection is hard-coded for those hosts,
  with source fallback when a supported host has no usable bottle. Official
  Homebrew supports Intel macOS too, but no source reviewed here gives a deeper
  reason than the initial implementation scope and its canonical-prefix design.
  — [mise `mod.rs:214-230`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L214-L230), [mise
  `tag.rs:16-64`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/tag.rs#L16-L64), [mise Brew docs
  `brew.md:228-239`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L228-L239), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)

### Status ledger: failure propagation and rollback

- **Fail-loud source subset — DOCUMENTED_INTENTIONAL.** The source-builder
  design explicitly rejects unsupported install-time helpers and unverifiable
  source shapes before mutation because an incorrect keg in the canonical
  prefix is worse than no keg. — [mise `source.rs:60-94`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/source.rs#L60-L94), [mise
  `shim.rb:854-859`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/shim.rb#L854-L859), [PR #10364](https://github.com/jdx/mise/pull/10364) (confidence: HIGH)
- **Remove a keg after link failure — DOCUMENTED_INTENTIONAL.** Mise explicitly
  says it must never leave a half-installed keg, removes the current keg when
  linking fails, and returns the error. Official Homebrew intentionally keeps a
  successfully poured/built but unlinked keg for ordinary link conflicts and
  marks the command failed. This is a real, documented semantic difference in
  failure-state policy. — [mise `pour.rs:249-269`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L249-L269), [Homebrew
  `formula_installer.rb:1195-1286`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L1195-L1286), [PR #10326](https://github.com/jdx/mise/pull/10326) (confidence: HIGH)
- **Abort the formula batch on first failed closure member — UNEXPLAINED.** Mise
  iterates the resolved closure sequentially and returns
  immediately on the first pour/source error; completed dependency kegs remain.
  Official Homebrew's top-level install and upgrade loops can mark one formula
  failed and continue independent requested formulae. The source follow-up
  notes sequential execution as a property of the generic driver, but gives no
  rationale for treating all requested roots as one failure domain. — [mise
  `mod.rs:166-210`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L166-L210), [Homebrew
  `install.rb:391-405`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/install.rb#L391-L405), [Homebrew
  `upgrade.rb:532-565`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/upgrade.rb#L532-L565), [PR #10364](https://github.com/jdx/mise/pull/10364) (confidence: HIGH for effect; MED that the enumerated history contains no choice rationale)

### Consolidated classification

| Difference | Current classification | Evidence-backed explanation |
| --- | --- | --- |
| Independent Rust bottle engine | DOCUMENTED_INTENTIONAL, DOCUMENTED_CONSTRAINT | Bootstrap was designed to install formulae with no Homebrew present. |
| Core API subset | DOCUMENTED_INTENTIONAL, DOCUMENTED_CONSTRAINT | Direct static metadata replaces Homebrew object loading for the supported surface. |
| Direct-API-only third-party taps | DOCUMENTED_INTENTIONAL, DOCUMENTED_CONSTRAINT | A brief real-`brew` delegation was deliberately removed by #10383. |
| Current-version-only package values | DOCUMENTED_CONSTRAINT | The artifact source publishes current bottles; versioned formula names remain names. |
| Client dependency resolver | DOCUMENTED_INTENTIONAL | Required by the no-Homebrew architecture. |
| Missing official dependency predicates/transaction | UNEXPLAINED | Observable omission; no reviewed motive. |
| Direct fetch/relocate/codesign/pour | DOCUMENTED_INTENTIONAL | Core launch surface. |
| Exact pour/link parity defects | INCREMENTAL_GAP | Multiple later PRs repaired concrete mismatches. |
| Limited Formula Ruby shim | DOCUMENTED_INTENTIONAL, DOCUMENTED_CONSTRAINT | Preserves no-Homebrew source builds and intentionally fails beyond its DSL subset. |
| Hand-built compatible receipt | DOCUMENTED_INTENTIONAL | Real Homebrew visibility was a launch goal. |
| Partial receipt fields | UNEXPLAINED | Static difference; no per-field rationale or runtime proof. |
| Shared formula producer state | DOCUMENTED_INTENTIONAL | Either Homebrew or mise active links satisfy formula presence. |
| Canonical prefix and initial bootstrap | DOCUMENTED_INTENTIONAL, DOCUMENTED_CONSTRAINT | Required by Homebrew bottles without an installed Homebrew. |
| Recursive chown of an existing prefix | UNEXPLAINED | Current behavior exceeds the narrower user documentation. |
| Homebrew-shaped links/records | DOCUMENTED_INTENTIONAL, INCREMENTAL_GAP | Compatibility intended; records and link cases added later. |
| Missing bottle finish/shared-file/post-install | UNEXPLAINED | Not implemented; no source says omission is desired. |
| `opt`-based basic health | DOCUMENTED_INTENTIONAL | Enables cross-producer convergence and retries after unlinked remnants. |
| Missing lifecycle/dependency health | UNEXPLAINED | Link repair expanded health once; lifecycle state remains absent. |
| Missing OpenSSL/CA trust effects | UNEXPLAINED | Required effects predate mise; no evidence they ever worked on bottle path. |
| Install-current-bottle upgrade | DOCUMENTED_INTENTIONAL | Selected by shared trait and #10346. |
| Old keg survives upgrade/prune | UNEXPLAINED | Contradicts PR/docs promise from introduction; no prior working implementation found. |
| Formula prune mutates Homebrew-owned kegs | DOCUMENTED_INTENTIONAL | #10618 deliberately replaced provenance-ledger pruning with inventory pruning. |
| No mise uninstall command | DOCUMENTED_CONSTRAINT, UNEXPLAINED | Generic declarative surface uses prune; no source explicitly rejects an imperative command. |
| No Intel macOS | DOCUMENTED_CONSTRAINT | Explicit launch/current platform boundary; deeper reason unrecorded. |
| Fail-loud source limitations | DOCUMENTED_INTENTIONAL | Avoids silently wrong canonical kegs. |
| Delete keg on link failure | DOCUMENTED_INTENTIONAL | Mise chooses retryable absence; Homebrew keeps an unlinked keg. |
| Abort formula batch on first error | UNEXPLAINED | Current loop behavior; no reviewed rationale. |

- **No current regression was found in the named histories inspected for this
  formula ledger.** The OpenSSL lifecycle and old-keg cleanup defects existed in
  the earliest relevant implementation inspected. Later link and relocation
  problems are verified bugs, but the cited history shows their fixes rather
  than a current reappearance. This is a bounded finding, not an exhaustive
  proof over all formula history. — method: `git show`/`git blame` at the introduction commits
  `173403d57ec5e0c7e25589e801354525051783e7` and
  `b83edbfa7098e4395bc6ae7da41dfc3273e0354b`, followed through current
  `6f52dcdf99e282ef7a7db68c81301fa4618d0f79` (confidence: MED within the named histories)

## Dead ends and contradictions

- “Mise differs because Homebrew metadata could not express lifecycle” is
  contradicted by history. Homebrew's general Formula step API predates mise's
  engine, and current formula API records expose the exact OpenSSL/CA operations.
  The individual core formulae did remain legacy Ruby until July, so structured
  availability was not uniform at mise launch. — [Homebrew framework commit
  `4c1998c7`](https://github.com/Homebrew/brew/commit/4c1998c75753bba578f895953237c0b3ffc2497d), [Formula API commit
  `003b9e24`](https://github.com/Homebrew/brew/commit/003b9e24920dc382267869fbcb7d5c4cbe09a193), [current `ca-certificates` API](https://formulae.brew.sh/api/formula/ca-certificates.json), [current `openssl@3` API](https://formulae.brew.sh/api/formula/openssl%403.json) (confidence: HIGH)
- “Mise intentionally implements all these differences” is unsupported. The
  no-Homebrew engine, canonical prefix, direct taps, limited source DSL,
  inventory prune, and selected failure policies are explicit. The lifecycle,
  health, cleanup, batch, and exact receipt omissions have no equivalent
  decision record. Treating code shape alone as motive would overstate the
  evidence. — evidence set: [originating PR #10326](https://github.com/jdx/mise/pull/10326), [source PR #10364](https://github.com/jdx/mise/pull/10364), [tap PR #10383](https://github.com/jdx/mise/pull/10383), [prune PR #10618](https://github.com/jdx/mise/pull/10618) (confidence: HIGH)
- “Upgrade replaces the old keg” is stated in #10346 and current documentation,
  but contradicted by current and introduction-time pour code. This is a
  promise/implementation contradiction, not evidence of a later regression. —
  [PR #10346](https://github.com/jdx/mise/pull/10346), [mise Brew docs
  `brew.md:390-397`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L390-L397), [introduction-time
  `pour.rs:126-149`](https://github.com/jdx/mise/blob/b83edbfa7098e4395bc6ae7da41dfc3273e0354b/src/system/packages/brew/pour.rs#L126-L149) (confidence: HIGH)
- The current docs claim real Homebrew `list`, `upgrade`, and `uninstall` work
  against mise kegs. Static receipt and topology correspondence supports the
  design intent, but this chapter did not run those commands; it cannot elevate
  the claim to runtime proof. — [mise Brew docs
  `brew.md:248-255`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L248-L255), [mise
  `pour.rs:299-375`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/pour.rs#L299-L375) (confidence: MED)

## Open unknowns

- No primary maintainer statement was found explaining why bottle finish,
  `.bottle/etc`/`.bottle/var`, and post-install were omitted even though the
  engine claimed Homebrew-compatible installed results. The observable cause is
  clear; the historical motive remains unknown.
- No primary source was found choosing active-link presence as sufficient
  runtime health after lifecycle effects became available in API metadata.
- No primary source explains why existing-prefix repair recursively transfers
  ownership, beyond the general goal of making the canonical prefix writable.
- No primary source explains first-error termination across independent formula
  roots, or the mismatch between promised and actual old-keg replacement.
- Runtime behavior of official Homebrew against current mise-written receipts,
  and the exact effects of mise prune on a mixed-producer prefix, remain to be
  validated experimentally; this chapter establishes design and static status,
  not live compatibility.
