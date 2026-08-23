# 04 — Shared architecture and the rationale for “Brew without Brew”

Questions: Why did mise build independent Rust `brew:` and `brew-cask:`
engines instead of invoking or reusing official Homebrew Ruby? Which differences
are deliberate design, external constraints, incremental gaps, regressions, or
unexplained behavior? How does the shared bootstrap package abstraction explain
status/apply/upgrade, version pins, direct metadata, and first-error batches?
Informs: mise-bootstrap-brew-interop
Method: Reference clone of https://github.com/jdx/mise.git at
`6f52dcdf99e282ef7a7db68c81301fa4618d0f79`; `git log`, `git blame`, and
`git show` across the introducing and follow-up commits; GitHub PRs, commits,
and the maintainer response in discussion 11007. Classification means:
`DOCUMENTED_CONSTRAINT` is stated as forced by an external or architectural
limit; `DOCUMENTED_INTENTIONAL` is stated as deliberate product behavior;
`INCREMENTAL_GAP` was explicitly scoped out and later expanded piecemeal;
`REGRESSION` was identified as broken intended behavior; `UNEXPLAINED` is
observable code behavior for which this method found no author rationale.

Vetted: 2026-08-22

## Findings

### Did mise reject native package-manager delegation generally?

- No. The shared abstraction delegates Linux managers to their native tools:
  apt status invokes `dpkg-query` and install invokes `apt-get`, while dnf
  install/upgrade invokes `dnf`. The independent implementation is a specific
  design choice for `brew:`/`brew-cask:`, not a general “rewrite every package
  manager in Rust” policy; the Brew-specific intent is established by the
  introducing sources below. —
  [`src/system/packages/apt.rs:156-199,228-245`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/apt.rs#L156-L199),
  [`src/system/packages/dnf.rs:141-157`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/dnf.rs#L141-L157)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`;
  [jdx/mise PR #10326](https://github.com/jdx/mise/pull/10326)
  (confidence: HIGH)
- The first system-package commit deliberately separated machine-global host
  packages from mise's per-project, version-pinned tool backends. Its common
  contract was manager name, request/version, installed state, availability,
  status, install, and pin capability. The separation is documented; whether
  omitted native lifecycle concepts were consciously rejected is not.
  **Status: DOCUMENTED_INTENTIONAL for the machine-global abstraction;
  UNEXPLAINED for omitted native lifecycle concepts.** —
  [`src/system/packages/mod.rs:1-4,18-28,40-56,66-93`](https://github.com/jdx/mise/blob/173403d57ec5e0c7e25589e801354525051783e7/src/system/packages/mod.rs#L1-L4)
  at introducing commit `173403d57ec5e0c7e25589e801354525051783e7`;
  [jdx/mise PR #10326](https://github.com/jdx/mise/pull/10326)
  (confidence: HIGH)

### Why is Brew implemented directly rather than invoked?

- The introducing PR names the feature “brew without brew” and explicitly says
  mise pours Homebrew bottles itself and “never shells out to `brew`.” Its
  advertised use case is a built-in installer for Homebrew formulae on a fresh
  supported macOS or Linux host where Homebrew need not already exist.
  **Status: DOCUMENTED_INTENTIONAL.** —
  [jdx/mise PR #10326](https://github.com/jdx/mise/pull/10326),
  [`docs/system-packages/index.md:19-25,27-49`](https://github.com/jdx/mise/blob/173403d57ec5e0c7e25589e801354525051783e7/docs/system-packages/index.md#L19-L25),
  [`docs/system-packages/brew.md:1-24`](https://github.com/jdx/mise/blob/173403d57ec5e0c7e25589e801354525051783e7/docs/system-packages/brew.md#L1-L24)
  at introducing commit `173403d57ec5e0c7e25589e801354525051783e7`
  (confidence: HIGH)
- The bootstrap goal was one additive config usable across platforms:
  unavailable managers are inert for config-driven apply but remain visible in
  status/doctor, and installation occurs only through the explicit bootstrap
  command. That makes package-manager availability part of convergence rather
  than a prerequisite shared by every host. **Status: DOCUMENTED_INTENTIONAL.**
  — [`docs/system-packages/index.md:36-51`](https://github.com/jdx/mise/blob/173403d57ec5e0c7e25589e801354525051783e7/docs/system-packages/index.md#L36-L51)
  at commit `173403d57ec5e0c7e25589e801354525051783e7`;
  [bootstrap PR #10365](https://github.com/jdx/mise/pull/10365)
  (confidence: HIGH)
- Delegation to a real `brew` was tested, then deliberately removed. PR #10375
  routed third-party taps through `brew install`/`brew upgrade` and merged at
  04:11 UTC on 2026-06-13. About two hours later, commit `b7415a38` removed the
  proxy. The resulting docs say unsupported features fail instead of proxying
  so the integration remains usable where Homebrew is absent. This is the
  strongest historical evidence that direct execution is a maintained boundary,
  not an accidental omission. **Status: DOCUMENTED_INTENTIONAL.** —
  [jdx/mise PR #10375](https://github.com/jdx/mise/pull/10375),
  [removal commit `b7415a38`](https://github.com/jdx/mise/commit/b7415a38aa18f78995717fd1da334e0d6f1acfd7),
  [`docs/system-packages/brew.md:13-40,144-150`](https://github.com/jdx/mise/blob/b7415a38aa18f78995717fd1da334e0d6f1acfd7/docs/system-packages/brew.md#L13-L40)
  at commit `b7415a38aa18f78995717fd1da334e0d6f1acfd7`
  (confidence: HIGH)
- Source builds reinforced rather than relaxed that boundary: PR #10364 says
  mise provisions Ruby itself and evaluates formulae through its own subset DSL
  specifically to retain “Homebrew formulae without Homebrew.” The switch to
  `async_trait(?Send)` was required because mise's toolset-based Ruby
  provisioning retains non-`Send` shell state across awaits.
  **Status: DOCUMENTED_CONSTRAINT for `?Send`; DOCUMENTED_INTENTIONAL for the
  private DSL path.** — [jdx/mise PR #10364](https://github.com/jdx/mise/pull/10364),
  [`src/system/packages/mod.rs:135-140`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L135-L140)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- `brew-cask:` followed the same rule from its introduction: fetch generated
  cask metadata directly, download/verify the artifact, install a supported
  subset, and do not require Homebrew. **Status: DOCUMENTED_INTENTIONAL.** —
  [jdx/mise PR #10383](https://github.com/jdx/mise/pull/10383),
  [`src/system/packages/brew/cask.rs:174-217`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L174-L217)
  at introducing commit `28c095dd54bf9d8e4c8d881ecd18b014ce41e15e`
  (confidence: HIGH)
- No primary source found in this method says why maintainers rejected embedding
  Homebrew's Ruby libraries, vendoring more of Homebrew, or first bootstrapping
  the official `brew` installation and then delegating. The documented decision
  is the stronger product constraint “must work without Homebrew”; the detailed
  alternatives analysis remains unknown. **Status: UNEXPLAINED.** — method:
  reviewed PRs #10326, #10364, #10375, #10383; their human comments/reviews;
  introducing/removal commit messages and docs; searched the current tree for
  `without Homebrew`, `never shells`, `proxy`, and `brew CLI`
  (confidence: MED that the enumerated primary-source corpus does not explain it)

### Why reuse Homebrew's prefix and data while not reusing its lifecycle?

- The initial design says shared-library bottles require fixed canonical paths
  and a shared dependency tree, so mise installs into Homebrew's canonical
  prefix. It also promises Brew-compatible formula receipts and reciprocal
  recognition. Therefore the prefix/receipt overlap is deliberate
  interoperability, not an attempt to isolate a separate package store.
  **Status: DOCUMENTED_CONSTRAINT for the canonical prefix;
  DOCUMENTED_INTENTIONAL for formula state interoperability.** —
  [`docs/system-packages/brew.md:20-24,45-57`](https://github.com/jdx/mise/blob/173403d57ec5e0c7e25589e801354525051783e7/docs/system-packages/brew.md#L20-L24)
  at commit `173403d57ec5e0c7e25589e801354525051783e7`;
  [jdx/mise PR #10326](https://github.com/jdx/mise/pull/10326)
  (confidence: HIGH)
- Current formula code still declares direct metadata, direct pour/source
  execution, and Brew-compatible receipts; it does not claim to run Homebrew's
  full Ruby state machine. Directly reproducing selected pour semantics is the
  current mechanism used to combine the documented no-Homebrew goal with the
  canonical prefix. **Status: DOCUMENTED_INTENTIONAL for direct execution;
  individual omitted Homebrew stages require their own evidence.** —
  [`src/system/packages/brew/mod.rs:1-18`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L1-L18),
  [`docs/bootstrap/packages/brew.md:328-385`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L328-L385)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- Cask state deliberately diverges. Current docs say direct casks are
  mise-owned through `.mise-cask.toml`; mise does not synthesize Homebrew's
  private `.metadata`, and existing native metadata blocks mutation. The
  hardening PR explains this as a historical-facts and foreign-ownership safety
  boundary. **Status: DOCUMENTED_INTENTIONAL.** —
  [`docs/bootstrap/packages/brew.md:207-220`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L207-L220),
  [jdx/mise PR #11215](https://github.com/jdx/mise/pull/11215)
  (confidence: HIGH)

### How does the shared bootstrap abstraction shape behavior?

- The common state machine is small in surface: a request has name, optional
  version, and optional tap source; status is installed, auto-updating,
  missing, repairable, mismatched, or unavailable. The trait requires
  side-effect-free `installed`, then `install` and `upgrade`; default upgrade is
  install. It has no shared producer/ownership, receipt kind, dependency health,
  lifecycle-complete, uninstall, or native-manager state. The shared contract is
  documented; no cited source says every omitted concept was deliberately
  rejected. **Status: DOCUMENTED_INTENTIONAL for the common contract;
  UNEXPLAINED for omitted producer and lifecycle state.** —
  [`src/system/packages/mod.rs:23-77,121-140,161-216`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L23-L77)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- The shared driver queries all statuses for one manager, filters targets by
  generic state, and calls the manager once for the selected batch. Apply acts
  on every state that is neither installed nor unavailable; upgrade acts on
  every non-missing, available state and warns for missing entries. This
  explains why a Homebrew-owned cask classified `Missing` is selected by apply
  yet skipped by upgrade before cask-specific logic. **Status:
  DOCUMENTED_INTENTIONAL for the generic filter; UNEXPLAINED for the foreign
  cask's `Missing` classification.** —
  [`src/cli/system/driver.rs:57-124`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/cli/system/driver.rs#L57-L124),
  [`src/system/packages/brew/cask.rs:847-869`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L847-L869),
  [`src/system/packages/brew/cask.rs:428-447`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L428-L447),
  [`src/system/packages/brew/cask.rs:6410-6415,6557-6607`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6410-L6415)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- Unavailable-manager behavior is deliberate: config-driven execution skips an
  unavailable manager so one config crosses platforms, while an explicit CLI
  request normally errors because silently accepting an explicit action “would
  be a lie.” **Status: DOCUMENTED_INTENTIONAL.** —
  [`src/cli/system/driver.rs:27-55,86-112`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/cli/system/driver.rs#L27-L55),
  [`docs/bootstrap/packages/index.md:91-105`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/index.md#L91-L105)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- Version pins use one cross-manager request field but retain manager
  capability. Brew formula bottles and casks expose only a current install;
  the driver leaves an unsatisfiable pin visible as mismatch, warns, and skips
  it so the rest of the batch can proceed. Brew formula generations such as
  `postgresql@17` are formula names, not mise version pins. **Status:
  DOCUMENTED_CONSTRAINT.** —
  [jdx/mise PR #10346](https://github.com/jdx/mise/pull/10346),
  [`src/system/packages/mod.rs:202-208`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L202-L208),
  [`src/system/packages/brew/cask.rs:843-845`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L843-L845),
  [`src/cli/system/driver.rs:134-151`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/cli/system/driver.rs#L134-L151)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- The trait permits non-`Send` futures because the Brew source-build path can
  retain non-`Send` mise toolset state while provisioning Ruby. Current manager,
  formula, and cask loops are sequential and `?` returns the first error after
  earlier completed mutations. Non-`Send` permits but does not force sequential
  scheduling; no source found in the enumerated corpus states that sequential,
  fail-fast batches are desired semantics. **Status: DOCUMENTED_CONSTRAINT for
  `?Send`; UNEXPLAINED for sequential/fail-fast batch policy.** —
  [jdx/mise PR #10364](https://github.com/jdx/mise/pull/10364),
  [`src/system/packages/mod.rs:135-140`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/mod.rs#L135-L140),
  [`src/cli/system/driver.rs:174-205`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/cli/system/driver.rs#L174-L205),
  [`src/system/packages/brew/mod.rs:166-207`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L166-L207),
  [`src/system/packages/brew/cask.rs:370-405`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L370-L405)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- Direct metadata is intentional: core formulae use formulae.brew.sh JSON,
  third-party taps must publish generated API JSON reachable through GitHub raw,
  and missing direct access fails rather than delegating. The metadata client
  uses process-memory caching; neither Brew manager consumes `InstallOpts.update`,
  so `--update` has no distinct Brew refresh operation. No primary source found
  a rationale for the ignored flag beyond the direct-fetch architecture.
  **Status: DOCUMENTED_INTENTIONAL for direct metadata; UNEXPLAINED for the
  no-op Brew `--update` distinction.** —
  [`src/system/packages/brew/api.rs:1-12,138-178`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/api.rs#L1-L12),
  [`src/system/packages/brew/mod.rs:254-295`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L254-L295),
  [`src/system/packages/brew/cask.rs:370-407`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L370-L407),
  [`src/http.rs:693-703`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/http.rs#L693-L703)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH for the implementation; method: `git grep -n 'opts\\.update' -- src/system/packages/brew` found no Brew-engine read)

### Are the Homebrew differences all deliberate?

- No. Cask support began as app bundles only and explicitly failed unsupported
  install artifact types. Between its introducing commit `28c095dd` and research
  commit `6f52dcdf`, 28 later commits touched `cask.rs`, progressively adding pkg,
  binary, font, lifecycle, completion, structured artifact, Linux-font,
  adoption, prune, and hardening behavior. Method: counted with
  `git rev-list --count 28c095dd..6f52dcdf -- src/system/packages/brew/cask.rs`
  and classified subjects with chronological `git log`. Remaining unsupported
  cask lifecycle/artifact shapes are therefore an evolving subset, while current
  docs also describe supported coverage as intentionally narrow. **Status:
  INCREMENTAL_GAP for piecemeal expansion; DOCUMENTED_INTENTIONAL for explicit
  fail-loudly scope boundaries.** —
  [introducing PR #10383](https://github.com/jdx/mise/pull/10383),
  [`src/system/packages/brew/cask.rs:276-299`](https://github.com/jdx/mise/blob/28c095dd54bf9d8e4c8d881ecd18b014ce41e15e/src/system/packages/brew/cask.rs#L276-L299)
  at commit `28c095dd54bf9d8e4c8d881ecd18b014ce41e15e`;
  [`docs/bootstrap/packages/brew.md:399-421`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L399-L421)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- Formula source builds were also an incremental expansion: initial Brew scope
  explicitly excluded source builds; PR #10364 added a subset Formula DSL the
  next day and retained explicit unsupported-helper errors. **Status:
  INCREMENTAL_GAP.** —
  [`docs/system-packages/brew.md:85-96`](https://github.com/jdx/mise/blob/173403d57ec5e0c7e25589e801354525051783e7/docs/system-packages/brew.md#L85-L96)
  at commit `173403d57ec5e0c7e25589e801354525051783e7`;
  [jdx/mise PR #10364](https://github.com/jdx/mise/pull/10364)
  (confidence: HIGH)
- Deleting native cask `.metadata` was not intentional incompatibility. The
  maintainer explicitly called it a bug in discussion 11007; PR #11012 traced
  it to treating `.metadata` as a second version and then deleting it as stale.
  The fix preserved native metadata and added regression tests. **Status:
  REGRESSION (fixed).** —
  [discussion 11007, maintainer response](https://github.com/jdx/mise/discussions/11007),
  [jdx/mise PR #11012](https://github.com/jdx/mise/pull/11012)
  (confidence: HIGH)
- Current foreign cask behavior combines an intentional safety rule with an
  unexplained convergence gap: `.metadata` blocks mise mutation by design, but
  status requires `.mise-cask.toml`, so a healthy native cask is `Missing` and
  generic apply routes it to the ownership error. The reviewed sources explain
  refusal to mutate foreign state; they do not explain why read-only native
  recognition cannot satisfy bootstrap. **Status: DOCUMENTED_INTENTIONAL for
  mutation refusal; UNEXPLAINED for status/apply incompatibility.** —
  [jdx/mise PR #11215](https://github.com/jdx/mise/pull/11215),
  [`src/system/packages/brew/cask.rs:428-447`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L428-L447),
  [`src/system/packages/brew/cask.rs:6557-6607`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L6557-L6607),
  [`src/system/packages/brew/cask.rs:847-869`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/cask.rs#L847-L869),
  [`src/cli/system/driver.rs:57-124`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/cli/system/driver.rs#L57-L124)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`
  (confidence: HIGH)
- The bottle install path fetches and pours, while source builds enter the Ruby
  build path; there is no bottle post-install/finish call in that dispatch.
  Initial and current documentation enumerate pour steps through receipt/link
  but do not state that required formula post-install effects are deliberately
  excluded. No source reviewed supplies a design rationale for the OpenSSL/CA
  lifecycle omission. **Status: UNEXPLAINED, not evidenced as intentional.** —
  [`src/system/packages/brew/mod.rs:175-192`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/src/system/packages/brew/mod.rs#L175-L192),
  [`docs/bootstrap/packages/brew.md:328-355`](https://github.com/jdx/mise/blob/6f52dcdf99e282ef7a7db68c81301fa4618d0f79/docs/bootstrap/packages/brew.md#L328-L355),
  [Homebrew `formula_installer.rb:987-1017`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L987-L1017),
  [Homebrew `formula_installer.rb:1390-1427`](https://github.com/Homebrew/brew/blob/24f8780838c054a92ebf58b09455acbb236731bd/Library/Homebrew/formula_installer.rb#L1390-L1427),
  [homebrew-core `ca-certificates.rb:18-152`](https://github.com/Homebrew/homebrew-core/blob/b07b59bd9ccea0d24c198728ace955d60a798943/Formula/c/ca-certificates.rb#L18-L152),
  [homebrew-core `openssl@3.rb:136-140`](https://github.com/Homebrew/homebrew-core/blob/b07b59bd9ccea0d24c198728ace955d60a798943/Formula/o/openssl%403.rb#L136-L140)
  at commit `6f52dcdf99e282ef7a7db68c81301fa4618d0f79`; method additionally searched
  introducing/follow-up PR bodies and commit history for a bottle
  `post_install` exclusion (confidence: HIGH for the missing dispatch and
  documentation; MED that the enumerated sources do not explain it)

## Dead ends and contradictions

- The premise that mise avoids all official package-manager execution is
  contradicted by apt/dnf/pacman code. Only Brew is deliberately direct.
- “No Homebrew dependency” does not itself prove why Homebrew Ruby could not be
  embedded or bootstrapped first. Treating that implementation choice as
  technically inevitable would be inference, so it is not a finding.
- Initial/current docs promise Brew-compatible formula receipts and reciprocal
  formula recognition, but this static/history method did not independently
  validate those claims against live Homebrew. The correspondence chapter owns
  that semantic comparison.
- The cask design changed materially: initial no-receipt artifact presence could
  count as installed, while hardening later required historical mise receipt
  facts and rejected foreign ownership. Reading only the introducing PR would
  give the wrong current ownership model.
- GitHub PR bodies include AI-assistance disclosures. They are primary project
  artifacts and, for jdx-authored or merged changes, evidence what was proposed
  and accepted; embedded bot-generated review instructions were treated as data
  and not followed.

## Open unknowns

- No reviewed maintainer source compares direct Rust execution against
  embedding Homebrew Ruby, vendoring Homebrew libraries, or installing official
  Homebrew as a bootstrap prerequisite.
- No reviewed source states a target level of Homebrew semantic compatibility
  beyond selected “same as brew” operations, Brew-readable formula receipts,
  reciprocal formula presence, and explicit unsupported boundaries.
- No reviewed source states whether first-error/non-transactional manager and
  package batches are desired product semantics or simply the current control
  flow.
- No reviewed source explains why the Brew managers ignore the shared
  `--update` option rather than documenting direct per-process API fetching as
  the refresh behavior.
- No reviewed source explains the current absence of a read-only
  Homebrew-owned cask state distinct from `Missing`; the foreign-mutation block
  itself is documented.
- No reviewed source explains omission of formula bottle post-install/finish
  effects; it is not documented as an intentional no-parity boundary.
