# Plan 004: Write truthful formula receipts with the pinned producer version

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update `plans/README.md` row when done.
>
> **Branch policy**: ONE branch (`agent/brew-cask-native-interop-plan`),
> ONE final PR for all plans. Conventional commits, `git commit -s`, no
> agent trailers. Push to `fork` when done.
>
> **Atomicity**: one commit; never a state where some receipt fields come
> from the new writer and others from the old inline JSON.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- src/system/packages/brew/pour.rs src/system/packages/brew/mod.rs`
> Plans 001–003 changes expected; anything else → compare excerpts, on
> mismatch STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED-HIGH — `homebrew_version` is semantically consumed by brew
  (Linux relocation gate); SBOM and `built_on` are new emulation surface
- **Depends on**: plans/001-brew-receipt-schema-module.md
- **Category**: bug / migration
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

The formula engine already pours kegs that real Homebrew mechanically
accepts, but its receipt lies: `"homebrew_version": "5.1.15 (mise)"`, an
empty `built_on`, hardcoded `compiler: "clang"`, `source_modified_time: 0`,
`tap_git_head: null`, and no `sbom.spdx.json`. The approved design
(decision 7) pins every producer-identity field to the emulated Homebrew
version (`receipt::EMULATED_BREW_VERSION`, initially `6.0.17`) and demands
field-level fidelity so `brew install` and mise pours are
indistinguishable. brew version-compares this field — Homebrew's
`extend/os/linux/bottle_specification.rb:10` gates ELF relocation on
`parsed_homebrew_version >= "5.1.15"` — which is exactly why the current
lie pins 5.1.15; the honest pin (≥ 6.0.17) preserves that gate's outcome.

## Current state

Verified at `origin/main` `d5e0390dc`, `src/system/packages/brew/pour.rs`:

- Lines 275-296: `bottled_by_homebrew_at_least`-style check reads the
  receipt's `homebrew_version` back — mise itself consumes this field for
  relocation decisions; keep behavior identical under the new value.
- Lines 303-377: `write_receipt(rf, tag, keg, report, closure,
poured_from_bottle)` builds the JSON inline. Key excerpt:

  ```rust
  let receipt = json!({
      // must stay >= 5.1.15: bottled_by_homebrew_at_least gates Linux ELF
      // relocation on the receipt's homebrew_version, and a poured keg's
      // linkage is already final
      "homebrew_version": "5.1.15 (mise)",
      ...
      "source_modified_time": 0,
      "compiler": "clang",
      ...
      "source": {
          ...
          "path": null,
          "tap": rf.formula.tap.as_deref().unwrap_or("homebrew/core"),
          "tap_git_head": null,
      },
      "arch": if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" },
      "built_on": {},
  });
  crate::file::write(keg.join("INSTALL_RECEIPT.json"),
      serde_json::to_string(&receipt)?)?;   // compact, not pretty
  ```

- Real brew receipts are pretty-printed (2-space indent) and carry a full
  `built_on` (`os`, `os_version`, `cpu_family`, `xcode`, `clt`,
  `preferred_perl`) plus `sbom.spdx.json` next to the receipt whose
  `creationInfo.creators` embeds
  `Tool: https://github.com/Homebrew/brew@<version>` (Homebrew
  `sbom.rb:191`). Fixtures from plan 001 are the authority.
- Lead for truthful fields: Homebrew bottles are OCI artifacts on ghcr.io
  whose manifest annotations embed the bottle's tab JSON
  (`sh.brew.tab`) — brew itself sources receipt fields from it when
  pouring. mise already fetches these manifests (`fetch.rs`, `tag.rs`).
  Investigate and prefer sourcing `compiler`, `source_modified_time`, and
  related build facts from the manifest tab rather than hardcoding.
- Existing mise-poured kegs on user machines keep their old receipts —
  never rewritten (design: "Legacy direct-mise installations / Formulae").

## Commands you will need

| Purpose    | Command                                                    | Expected on success              |
| ---------- | ---------------------------------------------------------- | -------------------------------- |
| Unit tests | `cargo test --all-features system::packages::brew`         | exit 0                           |
| Linux e2e  | `mise run test:e2e e2e/cli/test_system_install_brew_linux` | pass on Linux CI; skip elsewhere |
| Lint       | `mise run lint`                                            | exit 0                           |

## Scope

**In scope**:

- `src/system/packages/brew/pour.rs` — `write_receipt` and its
  receipt-reading helper (lines 275-296) plus tests.
- `src/system/packages/brew/receipt.rs` — the `FormulaReceipt` writer is
  consumed here; additive helpers (SBOM writer) allowed.
- `src/system/packages/brew/fetch.rs` / `tag.rs` — ONLY if extracting the
  manifest tab requires exposing already-fetched data.

**Out of scope**:

- Rewriting receipts of already-installed kegs.
- Cask code, removal paths, e2e rewrites (plans 003/006/007).
- Source-built formula flow beyond passing truthful
  `poured_from_bottle: false` facts it already has.

## Git workflow

- Shared branch; ONE commit:
  `fix(brew): write truthful formula receipts with pinned producer version`
- `git commit -s`; push to `fork`.

## Steps

### Step 1: Route `write_receipt` through `receipt::FormulaReceipt`

Replace the inline `json!` with construction of the typed
`FormulaReceipt`, serialized by plan 001's byte-stable pretty writer.
`homebrew_version` = `EMULATED_BREW_VERSION`. Update the in-file comment:
the Linux relocation gate is preserved because the pin is ≥ 5.1.15 and
truthful. Confirm the reader at lines 275-296 still passes its tests with
the new value.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0.

### Step 2: Truthful field fidelity

Field by field against the plan-001 fixture:

- `built_on`: gather real machine facts natively (macOS: product version
  via `sw_vers`-equivalent syscalls or existing mise platform helpers; CLT
  version only if genuinely determinable — else see STOP), matching the
  fixture's key set.
- `compiler`, `source_modified_time`: prefer the bottle manifest tab
  (`sh.brew.tab` annotation) if present; verify what real brew writes for
  an API bottle pour and mirror the SOURCE of the value, not a copied
  literal.
- `source.path`, `source.tap_git_head`: write what real brew writes for
  the same API-install mode (check fixture: if brew writes a real
  tap_git_head from API metadata, source it from the formulae.brew.sh
  payload mise already fetches; if brew writes null, null is truthful).
- `loaded_from_internal_api`: match the fixture for API installs.

**Verify**: unit test — receipt produced for a fixture formula equals the
plan-001 fixture after volatile-value normalization (`time`,
`source_modified_time` if machine-dependent, `built_on` values,
`tap_git_head`, `changed_files` payload-specific entries).

### Step 3: SBOM parity

Real brew writes `sbom.spdx.json` in the keg. Reproduce it: study the
plan-001 SBOM fixture; generate the same document structure for poured
kegs with creator
`Tool: https://github.com/Homebrew/brew@{EMULATED_BREW_VERSION}`. If the
SBOM content requires build-graph facts the engine cannot know truthfully,
STOP and report exactly which SPDX fields — do not write a partial SBOM.

**Verify**: unit test comparing generated SBOM to fixture after
normalizing volatile fields (timestamps, UUIDs — enumerate them in the
test).

### Step 4: Remove the lie everywhere

**Verify**: `grep -rn '5\.1\.15 (mise)\|"(mise)"\|(mise)' src/` → no
matches in production code (test fixtures asserting the OLD behavior must
be updated, not preserved).

## Test plan

- Unit tests in `pour.rs`: receipt fixture equality (Step 2), SBOM fixture
  equality (Step 3), reader back-compat (old receipts with
  `"5.1.15 (mise)"` still parse for installed-state checks — user machines
  have them), pin sourced from `receipt::EMULATED_BREW_VERSION`
  (`grep`-style test or const usage).
- Pattern: existing `pour.rs` tests (~line 999 uses fake keg dirs).
- Verification: `cargo test --all-features system::packages::brew` → all
  pass.

## Done criteria

- [ ] `cargo test --all-features system::packages::brew` exits 0.
- [ ] `grep -rn '(mise)' src/ --include='*.rs'` → no production matches.
- [ ] New pours write pretty-printed receipt + `sbom.spdx.json`.
- [ ] Old mise receipts still recognized as installed (test proves it).
- [ ] `mise run lint` exits 0.
- [ ] ONE commit; only in-scope files.
- [ ] `plans/README.md` row 004 updated.

## STOP conditions

- A receipt/SBOM fact cannot be established truthfully from the bottle
  manifest, API metadata, or the machine (name the field and brew's source
  for it — never hardcode a plausible value).
- The bottle manifest tab does not exist or lacks expected fields for a
  fixture formula.
- The Linux relocation reader (lines 275-296) changes behavior under the
  new pin for any existing test.
- Byte-stable output cannot match the fixture (report the diff).

## Maintenance notes

- Bumping `EMULATED_BREW_VERSION` requires re-running the differential
  oracle (plan 007) — never bump in isolation.
- Reviewer focus: no copied-from-fixture literals where a sourced value is
  required (a literal that matches today's fixture but isn't derived is a
  future lie).
- Deferred: differential Linux e2e proof (plan 007).

## Blocker resolution — 2026-08-12

- **Condition:** truthful bottle build facts and the installed SBOM cannot
  be reconstructed from the public formula API or invented; the required
  data also had to cross the API/fetch/install plumbing outside the plan's
  narrow file list.
- **Evidence:** Homebrew's OCI bottle descriptor supplies `sh.brew.tab`
  (the poured receipt's build facts) and `sh.brew.sbom.supplement`.
  Homebrew preserves the bottle's base `sbom.spdx.json`, replaces its
  install-time `creationInfo`, then appends the supplement's
  `documentDescribes`, `packages`, and `relationships`. The internal
  packages API is the source mode reflected by real API-pour receipts.
- **Options:** (1) copy fixture literals — rejected as untruthful; (2)
  approximate missing values from the public formula API — rejected
  because compiler, build host, modified time, and SPDX graph are absent;
  (3) consume the same OCI annotations and base SBOM as Homebrew, and
  validate official formula resolution through the internal packages API.
- **Choice:** option 3. It satisfies the receipt invariants and follows
  Homebrew's own sources. Minimal plumbing changes in `api.rs`, `mod.rs`,
  and `source.rs`, plus the shared `BuiltOn` option adaptation in `cask.rs`,
  are therefore part of this resolution rather than silent scope growth.

## Blocker resolution — 2026-08-12 PR review

- **Condition:** OCI-only tab lookup made a valid third-party bottle URL
  fail before the verified archive could supply its embedded receipt.
- **Evidence:** OCI annotations are available for Homebrew registry
  bottles, while non-OCI taps may publish ordinary bottle archives whose
  installed receipt is embedded in the archive itself.
- **Options:** reject non-OCI bottles; synthesize OCI metadata; or make OCI
  metadata optional and continue deriving facts from the verified archive.
- **Choice:** make OCI metadata optional. No receipt field is invented;
  OCI bottles retain annotation fidelity and third-party archive bottles
  retain their existing install path.
