# Plan 001: Add the shared Homebrew receipt schema module with the emulation pin

> **2026-08-13 audit override — historical implementation.** The schema/pin
> work remains useful, but this plan is not independent merge evidence. Reverify
> it on plan 017's combined head. `plans/README.md` overrides this file's old
> one-branch and “no agent trailers” instructions.

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in "STOP conditions" occurs, stop and report — do
> not improvise. When done, update this plan's row in `plans/README.md`.
>
> **Branch policy**: all plans in this directory are implemented on ONE
> branch (`agent/brew-cask-native-interop-plan`, rebased onto current
> `origin/main`) and delivered as ONE pull request. Do not create a
> per-plan branch. Commit with conventional commits and DCO signoff
> (`git commit -s`), no agent trailers. Push to the `fork` remote
> (`donbeave/mise`) after the plan completes.
>
> **Drift check (run first)**:
> `git diff --stat d5e0390dc..HEAD -- src/system/packages/brew`
> If files under `src/system/packages/brew/` changed since `d5e0390dc`
> beyond commits made by earlier plans in this directory, compare the
> "Current state" excerpts below against live code; on a mismatch, STOP.

## Status

- **Current acceptance:** IMPLEMENTED; REVERIFY ON COMBINED HEAD

- **Priority**: P1
- **Effort**: M
- **Risk**: MED — foundation for every other plan; wrong schema modeling
  propagates everywhere
- **Depends on**: none
- **Category**: tech-debt / migration foundation
- **Planned at**: commit `d5e0390dc` (origin/main), 2026-08-12

## Why this matters

The design document (`plans/brew-cask-native-homebrew-interoperability.md`,
maintainer-approved 2026-08-12) requires mise's internal brew/brew-cask
engine to read and write Homebrew's on-disk receipt state one-to-one with
real Homebrew, without ever executing the `brew` CLI in production. Today
the formula receipt is hand-built inline in `pour.rs` (with a falsified
producer version) and the cask engine uses a mise-private `.mise-cask.toml`
format. This plan creates the single shared module that owns the Homebrew
receipt schemas, the pinned emulated Homebrew version, schema-strict
parsers, and byte-stable writers — so formula, cask, backfill, import, and
prune code (plans 002–006) all speak one format and cannot drift apart.

## Current state

Files (all paths relative to repo root):

- `src/system/packages/brew/pour.rs` — formula pour; `write_receipt` at
  line 303 hand-builds the receipt JSON inline; line 340 writes the
  falsified `"homebrew_version": "5.1.15 (mise)"`; lines 366-369 write
  `"arch"` and an **empty** `"built_on": {}`; line 372 writes compact JSON
  via `serde_json::to_string`.
- `src/system/packages/brew/cask.rs` — cask engine (8,296 lines on
  `d5e0390dc`); `fn homebrew_metadata_present` at line 3706 only checks
  that `Caskroom/<token>/.metadata` exists; no code parses its contents.
- `src/system/packages/brew/mod.rs` — module root; declares the `brew`
  submodules; new module gets registered here.
- Existing unit tests in `cask.rs` (from ~line 7237) and `pour.rs` use
  fake-prefix fixtures via `MISE_SYSTEM_BREW_PREFIX` — follow that pattern.

Verified structure of real Homebrew `6.0.17` on-disk receipts (sampled
2026-08-12 from live installs; these facts are inlined so you do not need a
Homebrew machine to model them — but fixtures must be captured from real
brew, see Step 3):

Formula `Cellar/<name>/<version>/INSTALL_RECEIPT.json` — pretty-printed
JSON, 2-space indent (Ruby `JSON.pretty_generate` style). Top-level keys
observed: `homebrew_version` (e.g. `"6.0.13-8-g536cb45"` — a git-describe
string), `used_options`, `unused_options`, `built_as_bottle`,
`poured_from_bottle`, `loaded_from_api`, `loaded_from_internal_api`,
`installed_on_request`, `installed_as_dependency`, `changed_files`, `time`
(unix seconds), `source_modified_time`, `compiler`, `aliases`, `arch`
(`"arm64"`/`"x86_64"`), `runtime_dependencies` (array of objects),
`source` (`spec`, `versions`, `path`, `tap`, `tap_git_head`), `built_on`
(object: `os`, `os_version`, `cpu_family`, `xcode`, `clt`,
`preferred_perl`). The keg also contains `sbom.spdx.json`; its
`creationInfo.creators` embeds `Tool: https://github.com/Homebrew/brew@<homebrew_version>`
(Homebrew source `Library/Homebrew/sbom.rb:191`).

Cask `Caskroom/<token>/.metadata/INSTALL_RECEIPT.json` — pretty-printed
JSON, 2-space indent. Keys observed: `homebrew_version`,
`loaded_from_api`, `loaded_from_internal_api`, `uninstall_flight_blocks`
(bool), `installed_on_request`, `time`, `runtime_dependencies` (object),
`source` (`tap`, `tap_git_head`, `version`, `path`), `arch`, `built_on`
(same shape as formula), `uninstall_artifacts` (array of one-key objects
mirroring the cask DSL's artifact/uninstall/zap stanzas, e.g.
`{"binary": ["bin/codex"]}`,
`{"generate_completions_from_executable": [...]}`,
`{"zap": [{"rmdir": "~/.codex"}]}`).

Cask `.metadata/config.json` — **compact** JSON (no whitespace), keys
`default`, `env`, `explicit`; `default` holds the Cask::Config directory
map (`appdir`, `fontdir`, `languages`, …).

Cask `.metadata/<version>/<timestamp>/Casks/<token>.json` — the cask
definition snapshot as fetched from the API at install time; timestamp
directory format is `YYYYMMDDhhmmss.mmm` (e.g. `20260807033635.774`).

Semantic consumers you must not break: Homebrew version-compares
`homebrew_version` (`extend/os/linux/bottle_specification.rb:10` gates
Linux relocation on `parsed_homebrew_version >= "5.1.15"`), which is why
the current lie pins `5.1.15`. The design decision replaces it with the
real emulated version pin.

Repo conventions: error handling uses `eyre::Result` + `bail!`; files are
written through `crate::file` helpers; unit tests live in `#[cfg(test)] mod
tests` blocks in the same file. Match `src/system/packages/brew/cask.rs`
test style (fake prefix via `MISE_SYSTEM_BREW_PREFIX`).

## Commands you will need

| Purpose             | Command                                            | Expected on success |
| ------------------- | -------------------------------------------------- | ------------------- |
| Build               | `mise run build`                                   | exit 0              |
| Targeted unit tests | `cargo test --all-features system::packages::brew` | exit 0              |
| Lint                | `mise run lint`                                    | exit 0              |
| Lint fix            | `mise run lint-fix`                                | exit 0              |

## Scope

**In scope** (the only files you should modify):

- `src/system/packages/brew/receipt.rs` (create)
- `src/system/packages/brew/mod.rs` (register the module only)
- `src/system/packages/brew/testdata/` (create — captured fixtures)

**Out of scope** (do NOT touch):

- Any behavior change in `pour.rs`, `cask.rs`, `maintenance.rs` — wiring
  happens in plans 002–006.
- The `brew ruby` call in `cask.rs` (plan 003).
- Any CLI file under `src/cli/`.

## Git workflow

- Branch: `agent/brew-cask-native-interop-plan` (shared across all plans;
  rebase onto `origin/main` first if plan 001 is the first to run).
- One commit: `refactor(brew): add shared Homebrew receipt schema module with emulation pin`
- `git commit -s`; no agent trailers. Push to `fork` when done.

## Steps

### Step 1: Create the module skeleton and pin

Create `src/system/packages/brew/receipt.rs` and register `pub mod
receipt;` in `src/system/packages/brew/mod.rs`. Define:

```rust
/// The Homebrew version whose on-disk state this engine emulates and is
/// differential-verified against. Bump ONLY after the differential oracle
/// (e2e) passes against the newer Homebrew.
pub const EMULATED_BREW_VERSION: &str = "6.0.17";
```

Every producer-identity field in later plans reads this constant; nothing
else may hardcode a Homebrew version string.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0
(compiles, existing tests unaffected).

### Step 2: Typed models with strict-but-tolerant parsing

Model these types with `serde`:

- `FormulaReceipt` — required fields exactly the observed formula keys
  above. Use correct types (`u64` for `time`, `bool` flags, nested
  `Source`, `BuiltOn`, `RuntimeDependency`).
- `CaskReceipt` — required fields exactly the observed cask keys above;
  `uninstall_artifacts: Vec<serde_json::Value>` (one-key objects preserved
  verbatim — their internal shape is cask-DSL-specific and must round-trip
  byte-identically).
- `CaskConfig` — `default`/`env`/`explicit` maps preserved as
  `serde_json::Value`.

Parsing rules (design decision 10 — schema-strict, version-tolerant):

- Missing required key, wrong type, or unparseable JSON → a classified
  error (`ReceiptError` enum: `Malformed`, `MissingField`, `Io`), never
  treated as "not installed".
- Unknown EXTRA keys are tolerated and preserved: add
  `#[serde(flatten)] extra: serde_json::Map<String, serde_json::Value>` to
  each struct so a receipt written by a newer brew round-trips without
  loss. Do NOT use `deny_unknown_fields`.
- `homebrew_version` is an opaque `String`; never parse or compare it
  numerically.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0.

### Step 3: Capture fixtures from real Homebrew

On a machine with real Homebrew ≥ 6.0.17 (macOS CI runner or the
implementation host), capture VERBATIM fixture files into
`src/system/packages/brew/testdata/`:

- one formula `INSTALL_RECEIPT.json` (small formula, e.g. install `xz`
  fresh via `brew install xz` in a disposable environment — tests may run
  real brew, production code may not);
- that keg's `sbom.spdx.json`;
- one cask `.metadata/INSTALL_RECEIPT.json`, `config.json`, and one
  timestamped `Casks/<token>.json` snapshot (small app cask);
- record in a `testdata/README.md` the exact `brew --version` output that
  generated each fixture.

If you cannot run real brew anywhere, STOP and report — fixtures must be
captured, not hand-written.

**Verify**: files exist; `testdata/README.md` names the generating brew
version for each.

### Step 4: Byte-stable writers

Implement writers:

- `FormulaReceipt::to_json_bytes()` / `CaskReceipt::to_json_bytes()` —
  pretty JSON, 2-space indent, key order identical to the captured
  fixtures (Ruby `JSON.pretty_generate` layout). Use `serde_json` with a
  custom pretty formatter (`serde_json::ser::PrettyFormatter::with_indent(b"  ")`)
  and struct field order matching the fixture key order.
- `CaskConfig::to_json_bytes()` — compact JSON (no spaces), matching the
  fixture.

Round-trip test: parse each captured fixture, re-serialize, and assert the
bytes equal the fixture **after normalizing volatile values only** (replace
the values — not the keys — of `time`, `source_modified_time`,
`tap_git_head`, `built_on` members, `source.path`, and `homebrew_version`
in both sides before comparing). If byte equality cannot be reached because
Ruby's generator formats something `serde_json` cannot reproduce (e.g.
float formatting), STOP and report the exact byte difference.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0,
including new round-trip tests.

### Step 5: Reader helpers for on-disk state

Add path-based readers used by later plans:

- `read_formula_receipt(keg: &Path) -> Result<FormulaReceipt, ReceiptError>`
- `read_cask_receipt(caskroom_token_dir: &Path) -> Result<CaskReceipt, ReceiptError>`
  (reads `.metadata/INSTALL_RECEIPT.json`)
- `read_cask_config(...)`, plus a helper returning the newest timestamped
  metadata dir for a version **by directory-name string ordering of the
  timestamp format only** (timestamps `YYYYMMDDhhmmss.mmm` sort
  lexicographically; this is not version ordering).

Unit tests: valid fixture parses; truncated JSON → `Malformed`; missing
`uninstall_artifacts` key → `MissingField`; extra unknown key → parses and
round-trips.

**Verify**: `cargo test --all-features system::packages::brew` → exit 0.

## Test plan

- New tests in `receipt.rs` `#[cfg(test)]` block: fixture parse, byte
  round-trip (formula, cask, config), each error classification, unknown-key
  tolerance, opaque `homebrew_version` (a value like
  `"7.1.2-99-gabcdef0"` parses fine).
- Pattern: existing fixture-style tests in `cask.rs` (~line 7237 onward).
- Verification: `cargo test --all-features system::packages::brew` → all
  pass including the new tests.

## Done criteria

ALL must hold:

- [ ] `cargo test --all-features system::packages::brew` exits 0.
- [ ] `mise run lint` exits 0 (run `mise run lint-fix` first if needed).
- [ ] `grep -rn "deny_unknown_fields" src/system/packages/brew/receipt.rs`
      → no matches.
- [ ] `grep -rn "6\.0\.17" src/system/packages/brew/ --exclude-dir=testdata | grep -v receipt.rs`
      → no matches (pin lives only in the module).
- [ ] `git status --short` shows only the in-scope files modified/created.
- [ ] `plans/README.md` row 001 updated.

## STOP conditions

Stop and report (do not improvise) if:

- Real-brew fixtures cannot be captured anywhere (no environment with
  Homebrew ≥ 6.0.17).
- Byte-stable serialization cannot match the captured fixtures exactly
  after volatile-value normalization.
- The live receipts contain keys or shapes not listed in "Current state"
  that look load-bearing (report them; the schema spec must be extended
  deliberately, not guessed).
- `mod.rs` module registration conflicts with a rebase-introduced module of
  the same name.

## Blocker resolution — 2026-08-12 verification review

- **Condition:** the single-pin grep also matched the required fixture provenance README.
- **Evidence:** production writers reference `receipt::EMULATED_BREW_VERSION`; the fixture README must preserve exact generating `brew --version` output.
- **Options:** delete provenance (violates fixture requirements), obscure the recorded version (untruthful), or scope the producer-constant gate outside `testdata/`.
- **Choice:** exclude `testdata/`; fixture provenance is evidence, not a producer-identity source.

## Maintenance notes

- Every future Homebrew format change lands here first: oracle fails →
  update models/writers → re-verify → bump `EMULATED_BREW_VERSION`.
- Reviewers: check that no other file hardcodes a Homebrew version and that
  volatile-field normalization happens only in tests, never in production
  writers.
- Deferred: SBOM generation (plan 004 decides how `sbom.spdx.json` is
  produced; this plan only ships the fixture and creator-string knowledge).

## Blocker resolution — 2026-08-12

- **Condition:** the mandatory lint gate failed on the committed executor
  plans and changed byte-exact real-Homebrew fixtures: Prettier reported all
  plan Markdown files and added fixture EOF newlines; markdownlint parsed the
  controlling design's bare `#11810` text as a second top-level heading.
- **Evidence:** `mise run lint-fix` named `plans/001..008`, `plans/README.md`,
  and the controlling design as Prettier inputs; markdownlint reported
  `MD025` at the bare issue reference. Source behavior and receipt fixtures
  were unaffected.
- **Options:** (1) skip the lint gate or accept formatter-altered fixtures,
  violating mandatory verification or byte identity; (2) format only touched
  plan files, leaving the same repository-wide lint failure; (3) format the
  executor-plan Markdown corpus, disambiguate the issue as `jdx/mise#11810`,
  and exclude captured receipt fixtures from Prettier like existing
  `test/data` fixtures.
- **Choice:** option 3. It preserves every hard invariant and fixture byte
  identity, changes no runtime behavior, makes repository lint reproducible,
  and keeps the issue reference semantically exact.

### Standalone foundation dead-code gate

- **Condition:** `mise run lint` denied `dead_code` warnings because plan 001
  deliberately lands the shared schema before plans 002–004 consume it.
- **Evidence:** cargo named only the new receipt module's constant, models,
  readers, and writers; all are exercised by plan-001 tests and have explicit
  consumers in the next plans.
- **Options:** (1) violate plan ordering by combining consumer behavior into
  this commit; (2) make the binary's private `system` module public solely to
  suppress reachability warnings; (3) temporarily allow Rust `dead_code` on
  the receipt module registration, then remove it when consumers land.
- **Choice:** option 3. It is scoped to this staged foundation, adds no Clippy
  exclusion, no runtime behavior, and no public surface. Plan 002 must remove
  it when the first production reader is wired; plans 003–004 consume the
  remaining writers and models.

## Blocker resolution — 2026-08-12 PR schema review

- **Condition:** optional native probes omitted `built_on` keys instead of
  preserving Homebrew's stable receipt shape.
- **Evidence:** captured receipts contain the full key set; absence of a
  machine value is represented by JSON null.
- **Options:** omit absent keys, invent values, or serialize nulls.
- **Choice:** serialize nulls and fail closed on unsupported operating systems.
