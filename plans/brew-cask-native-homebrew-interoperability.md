# Native mise ↔ Homebrew cask interoperability

Status: research and implementation plan  
Researched: 2026-08-02  
mise baseline: `832623e202ff7be9e3a735c4ba1437d578d3066c` (`origin/main`)  
Homebrew source baseline: `1f3abf43a3560d01d2f9bfa6fed5cbba208fded4`

## Executive conclusion

The current guard is correct about one danger but implements the wrong model:

```rust
if homebrew_metadata_present(&cask.token) {
    bail!(
        "brew-cask:{}: Homebrew owns this cask; remove it with Homebrew before installing it with mise",
        cask.token
    );
}
```

It prevents mise from overwriting a Homebrew installation while leaving stale
Homebrew lifecycle metadata. However, it also makes a valid installed cask look
missing to declarative bootstrap. This breaks migration of an existing Mac to
`mise bootstrap` and caused the observed failure on `brew-cask:bartender`.

The desired model is not “dispatch mutations to whichever manager installed the
app.” It is a **shared, Homebrew-compatible on-disk cask contract implemented in
native Rust**:

1. A valid Homebrew installation satisfies mise `status` and `apply`.
2. Mise can natively upgrade it without executing Homebrew or Ruby.
3. Fresh mise installs write the metadata Homebrew needs to list, upgrade, and
   uninstall the cask later.
4. Mise keeps `.mise-cask.toml` as an additional integrity/rollback receipt; it
   no longer treats Homebrew `.metadata` as mutually exclusive ownership.
5. Structured JSON API metadata is the default lifecycle input and every new
   capability is native Rust. The existing Ruby shim remains a frozen,
   explicitly legacy bridge only for casks whose API metadata contains a
   legacy flight-hook marker.

This reaches formula-like interoperability while retaining mise's native pour.
It is larger than deleting the guard: detection, historical uninstall metadata,
transactional metadata writes, legacy migration, and compatibility tests must
land together.

## Fixed command-surface constraint

This work changes implementation behind commands mise already has. It must not
add a command, subcommand, flag, mode, ownership selector, adoption workflow,
repair UI, import path, prune path, or uninstall command.

The only mise entry points in scope are the existing package operations:

- `mise bootstrap packages status`;
- `mise bootstrap packages apply` (and `mise bootstrap` calling it);
- `mise bootstrap packages upgrade`;
- their existing `brew:` and `brew-cask:` manager selection/configuration.

`brew:` formula installation is the compatibility reference and a regression
gate; its receipt behavior is already correct. Most implementation work belongs
to `brew-cask:` so the same existing commands produce an installation with the
same observable filesystem, metadata, lifecycle, and later official-Homebrew
behavior as `brew install --cask`.

Official `brew list/info/outdated/upgrade/uninstall` commands appear below only
as black-box compatibility verifiers. Mise does not gain corresponding new
commands and does not delegate to them.

## Requirements derived from the workflow

The feature is complete only when all these flows work:

| Starting state | `mise ... status` | `mise ... apply` | `mise ... upgrade` | Later official `brew` |
|---|---|---|---|---|
| Absent | missing | native install | native install | sees/manages result |
| Installed by Homebrew | installed | no-op | native Rust upgrade | still sees/manages result |
| Installed by current mise | installed | no-op | migrates during native upgrade | sees result after migration |
| Installed by new mise | installed | no-op | native Rust upgrade | sees/manages result |
| Corrupt/ambiguous records | unhealthy/error | fail before mutation | fail before mutation | untouched |

“Installed by either manager” must describe compatibility, not an irreversible
owner choice. Requiring uninstall/reinstall, shelling out to `brew`, evaluating a
cask Ruby file, or silently ignoring an old version does not satisfy the goal.

## Current mise implementation

### Formula path already has the right vision

`src/system/packages/brew/mod.rs` installs bottles directly into the canonical
prefix without Homebrew. `src/system/packages/brew/pour.rs` writes a compatible
`INSTALL_RECEIPT.json`, so Homebrew recognizes mise-poured formulae. Formula
status reads canonical prefix state regardless of which program poured it.

Casks should follow this architecture: native execution plus a compatible
receipt, not manager delegation.

### Cask status conflates presence with mise ownership

At `src/system/packages/brew/cask.rs:531`, `BrewCaskManager::installed` fetches
current API metadata and calls `installed_cask_version`. That function accepts
only a version directory with `.mise-cask.toml`, a completed journal, valid
fingerprints, and installed pkg receipts. A normal Homebrew cask has no mise
receipt, so it becomes `PackageState::Missing`.

`install_one` then checks only whether any `.metadata` filesystem object exists
and aborts. Result: status says missing, apply refuses to install.

### Mise's direct lifecycle is mostly native, but not Ruby-free

Native Rust already handles:

- app bundles;
- binaries and command wrappers;
- fonts;
- declared/generated completions;
- macOS `.pkg` installers and pkgutil receipt checks;
- structured preflight/postflight `move`, `remove`, `run`, and
  `terminate_process` steps;
- target containment, checksum verification, transactional activation,
  rollback journals, durable receipts, and content fingerprints.

But `execute_lifecycle_hook`, `cask_ruby_bin`, `fetch_cask_rb`, and
`cask_shim.rb` still download and evaluate checksum-bound Ruby for legacy
`preflight` and `postflight` blocks. This is not the default for normal casks:
it is reached only when `has_lifecycle_hook` finds an explicit legacy marker.
The correct near-term design keeps that path as frozen compatibility while all
new structured capabilities are implemented in Rust.

Do not grow the embedded Ruby DSL. Structured metadata is typed, auditable,
path-validatable, and compatible with direct argv execution. Remove the bridge
only after measured compatibility gates show its remaining value is zero or an
accepted breaking change.

### Catalog-wide Ruby impact measurement

The official `https://formulae.brew.sh/api/cask.json` catalog was measured on
2026-08-02 at Homebrew cask tap head
`9853cf07d2bde789664d30f15c22c1ef6620a847`:

| Metric | Count |
|---|---:|
| Total official casks | 7,671 |
| Casks with any legacy flight marker | 9 |
| Legacy `preflight` markers | 1 |
| Legacy `postflight` markers | 7 |
| Legacy `uninstall_preflight` markers | 1 |
| Legacy `uninstall_postflight` markers | 4 |
| Structured `preflight_steps` artifacts | 32 |
| Structured `postflight_steps` artifacts | 58 |
| Structured `uninstall_preflight_steps` artifacts | 28 |
| Structured `uninstall_postflight_steps` artifacts | 19 |

The nine legacy casks are:

| Cask | Legacy markers | Current mise impact if shim is removed |
|---|---|---|
| `adobe-creative-cloud` | uninstall-postflight | no current install-path change; mise already rejects its unsupported `installer` artifact |
| `distroav` | postflight, uninstall-preflight | loses a working pkg postflight that creates OBS plugin links |
| `docker-desktop` | postflight, uninstall-postflight | loses conditional kubectl-link creation |
| `hummingbird` | postflight | current shim fails because `set_ownership` is not implemented; no working support to preserve |
| `libcblite` | postflight, uninstall-postflight | already rejected because generic `artifact` is unsupported |
| `libcblite-community` | postflight, uninstall-postflight | already rejected because generic `artifact` is unsupported |
| `meshlab` | postflight | loses case-sensitive-filesystem app symlink repair |
| `multipass` | postflight | loses arm64 completion symlink creation |
| `rodecaster` | preflight | loses wildcard pkg rename before installation |

Therefore a complete immediate deletion removes source download/Ruby lookup for
all casks but produces an actual regression for approximately five currently
working official installs: `distroav`, `docker-desktop`, `meshlab`, `multipass`,
and `rodecaster`. Third-party tap impact cannot be measured from the official
catalog and may be larger.

This measurement also corrects a misleading source-field signal: every current
official API record has `ruby_source_path`, not only hook-bearing casks. Mise
must gate Ruby strictly from legacy artifact markers, never from source-path
presence.

### Structured Rust coverage is the larger current gap

Homebrew currently serialises 15 cask step types:

```text
copy, delete_keychain_certificate, inreplace, mkdir_p, move,
move_children, move_contents, remove, run, set_ownership,
set_permissions, symlink, terminate_process, touch, write
```

Mise implements only `move`, `remove`, `run`, and `terminate_process`. Across
install-time structured hooks, 46 official casks use at least one unsupported
step type. Another 32 use only the four nominally supported types, but
`miniconda` and `miniforge` still fail schema parity because their `move` and
`remove` steps contain `guards`/`overwrite` fields the current parser rejects.
Thus only 30 current structured install-hook casks fit the present parser's
type/field surface.

There is also a semantic mismatch: current mise defaults
`terminate_process.must_succeed` to `false`; its behavior must track the
canonical Homebrew schema and tests exactly rather than assuming a default.
Supported type names alone do not prove parity—fields, guards, tokens, default
bases, overwrite behavior, uninstall inversion, privilege, and error policy all
matter.

The authoritative porting reference is current Homebrew
`Library/Homebrew/install_steps.rb` plus its tests, not the old mise Ruby shim.
Homebrew's own migration plan reports that the structured DSL can represent all
official legacy hooks after tap conversion, but the public cask head still has
nine legacy casks. Mise should follow the released API, not unpublished/local
migration branches.

### Current mise receipt remains valuable

The schema-2 `.mise-cask.toml` records target paths, pkg IDs, and full content
fingerprints. Homebrew's receipt is designed for reconstruction and uninstall;
it does not provide mise's drift proof. Keep both receipts, but give them
different meanings:

- `.metadata`: shared Homebrew compatibility and historical lifecycle facts;
- `.mise-cask.toml`: optional mise integrity extension for the version mise
  activated.

Both existing is therefore normal, not a conflict. If official Homebrew later
upgrades the cask, its normal version replacement removes the old version's
`.mise-cask.toml`; mise then recognizes the installation through `.metadata`.

## Homebrew's actual metadata contract

Evidence comes from the installed Homebrew source under
`/opt/homebrew/Library/Homebrew`, not inferred directory names.

### Installed detection

`cask/caskroom.rb` recognizes a cask when this exists:

```text
<prefix>/Caskroom/<token>/.metadata/<opaque-version>/<timestamp>/Casks/
  <token>.json
  <token>.internal.json   # legacy
  <token>.rb              # legacy/uninstall flight blocks
```

It chooses the lexically latest timestamp and derives installed version from the
parent version directory. Version is opaque; mise must not use semver ordering.
Symlinked token directories are rejected by Homebrew.

### Modern installed cask file is small

`Cask::Installer#save_caskfile` writes `<token>.json` for casks without legacy
uninstall flight blocks. `Cask#to_installed_json_hash` currently stores only
`url_specs.only_path` when needed; otherwise the JSON is `{}`. It stores an
explicit empty `artifacts` array only when no uninstall artifacts exist.

Homebrew reconstructs version from the directory, then loads exact uninstall
artifacts from the top-level receipt. When absent, it may fall back to today's
API, but mise must not depend on that fallback because current definitions can
differ from the installed version.

### Top-level receipt

The file is:

```text
<prefix>/Caskroom/<token>/.metadata/INSTALL_RECEIPT.json
```

`Cask::Tab#to_json` contains these fields:

```json
{
  "homebrew_version": "...",
  "loaded_from_api": true,
  "loaded_from_internal_api": true,
  "uninstall_flight_blocks": false,
  "installed_on_request": true,
  "time": 0,
  "runtime_dependencies": null,
  "source": {
    "tap": "homebrew/cask",
    "tap_git_head": "...",
    "version": "...",
    "path": "..."
  },
  "arch": "arm64",
  "uninstall_artifacts": [],
  "built_on": null
}
```

Homebrew tolerates unknown/missing informational fields, but compatibility must
be proven against its loader and commands. Mise should write truthful values:

- a mise identifier/version in a new non-authoritative extension field if
  desired, never falsely claim a Homebrew version;
- `loaded_from_api`/`loaded_from_internal_api` matching source;
- `installed_on_request = true` for configured roots;
- Unix epoch installation time;
- architecture from the target system;
- tap and `tap_git_head` from fetched metadata;
- exact opaque installed version;
- exact uninstall artifacts preserved from the API used for installation;
- `uninstall_flight_blocks = false` for the native-only accepted set.

Before implementation, add a Homebrew-side compatibility fixture to determine
whether omitting `homebrew_version`/`built_on` is supported. Do not invent fake
Homebrew build data merely to match shape.

### Upgrade transaction ordering

Homebrew's own upgrade flow:

1. Loads the installed definition and receipt.
2. Uninstalls old artifacts and backs up old staged files plus version metadata.
3. Stages new payload and installed JSON.
4. Installs new artifacts.
5. Rolls back old artifacts and metadata on failure.
6. Purges old staging after success.
7. Writes the new top-level receipt only after rollback is no longer needed.

Mise should preserve the same invariant: never publish new shared metadata until
payload activation succeeds, and never remove the last recoverable old metadata
before rollback is impossible.

## Correct native Rust state model

Replace `homebrew_metadata_present(bool)` with one authoritative classifier.
Names are illustrative:

```rust
enum CaskInstallState {
    Absent,
    HomebrewCompatible(HomebrewCaskRecord),
    MiseLegacy(MiseCaskRecord),
    CompatibleWithMiseIntegrity {
        homebrew: HomebrewCaskRecord,
        mise: MiseCaskRecord,
    },
    UnmanagedPayload,
    Corrupt(CaskStateProblem),
}
```

`HomebrewCaskRecord` must contain canonical token, opaque version, installed
caskfile path/type, parsed top-level tab, and exact raw uninstall artifacts.
`MiseCaskRecord` retains existing receipt validation.

Classification rules:

- validate the canonical token after API alias/old-token resolution;
- reject symlinked token, `.metadata`, version, timestamp, Casks, caskfile, and
  receipt objects;
- require all paths to remain beneath the canonical Caskroom token directory;
- parse JSON with size/depth limits;
- accept `.rb` only as detectable legacy state; never evaluate it;
- a `.rb` record with uninstall flight blocks is readable for presence but not
  natively upgradeable until exact structured uninstall data is available;
- both receipts are healthy only when token/version/targets agree;
- a payload with neither receipt is unmanaged and must not be overwritten;
- pending journals or `.upgrading`/`.mise-*` remnants make state unhealthy until
  the appropriate recovery path resolves them.

All existing cask status, apply, and upgrade paths must use this classifier.
Independent boolean checks will recreate policy drift. Repair, adoption,
uninstall, import, and prune command surfaces are explicitly out of scope.

## Native install and upgrade design

### Phase 1: presence interoperability

Make a valid `HomebrewCompatible` state return `Installed`. `apply` must become
an idempotent no-op and preserve every byte of Homebrew metadata. This alone
fixes adding bootstrap declarations to existing machines.

Do not require installed version to equal today's API version for `latest`.
`apply` ensures presence; explicit `packages upgrade` performs upgrades.

### Phase 2: shared receipt writer for fresh mise installs

Add typed Rust structures for installed JSON and cask tab. Keep raw API
`artifacts` on `Cask`; derive and validate the exact uninstall subset before any
mutation. Never copy unknown artifact types blindly into a privileged receipt.

After native payload activation succeeds:

1. Write `.mise-cask.toml` into the activated version directory.
2. Create a unique timestamped temporary metadata tree under the token.
3. Write `<token>.json` and fsync it.
4. Write a temporary top-level `INSTALL_RECEIPT.json` and fsync it.
5. Validate both by reading them through mise's parser.
6. Atomically rename the timestamped tree into place.
7. Atomically replace the top-level receipt.
8. Fsync each parent directory.
9. Remove obsolete metadata only after activation and receipt commit.

Temporary names must be unique per process/transaction. Never use one shared
`.tmp` path. Journal metadata steps so interrupted commits can roll forward or
restore the previous tree deterministically.

### Phase 3: native upgrade of a Homebrew installation

Before mutation, load the historical Homebrew record. Validate that every old
uninstall artifact is understood by native Rust. The implementation must support
the old record, not merely today's API.

Construct one transaction containing:

- old staged version and `.metadata/<old-version>` backup;
- old top-level receipt backup;
- old external targets derived from historical artifacts;
- new staged payload, external targets, mise receipt, installed JSON, and tab.

Then:

1. Validate the complete old and new lifecycle plans.
2. Download/checksum/extract new payload without mutation.
3. Stop/quit processes only through supported structured directives.
4. Back up old payload, metadata, receipt, and replaceable external targets.
5. Run native uninstall/transition actions required for old artifacts.
6. Activate new payload and links/installers.
7. Write both compatible and mise receipts.
8. Verify final targets and reread shared metadata.
9. Commit; then remove backups and stale versions.
10. On any failure, remove partial new state and restore old payload, targets,
    metadata, and receipt.

Do not call `brew`, `brew ruby`, system Ruby, or a mise-managed Ruby. Do not
parse/evaluate `.rb`.

### Phase 4: legacy mise migration

Current mise installations have `.mise-cask.toml` but no `.metadata`.

- `status` and `apply` remain valid without mutation.
- On the next successful explicit upgrade/reinstall, write compatible metadata
  for the new version.
- No new repair command or flag is introduced. Existing explicit upgrade may
  migrate the receipt only when the original API payload/uninstall facts are
  provably recoverable; otherwise it fails before mutation. It must not
  reconstruct historical uninstall state from today's changed API.
- Never silently synthesize metadata during read-only status.

After migration, official Homebrew must list, upgrade, and uninstall the cask.

## Removing Ruby from brew-cask

The native-only implementation must remove:

- `CASK_SHIM_RB`;
- `execute_lifecycle_hook`;
- `cask_ruby_bin`;
- `fetch_cask_rb`;
- `ensure_cask_shim`;
- `src/system/packages/brew/cask_shim.rb`;
- tests whose purpose is Ruby DSL evaluation.

Keep `ruby_source_path` and checksum fields only if another non-execution use is
required; otherwise remove them from the cask model.

At metadata preflight:

- structured `preflight_steps`/`postflight_steps`: parse and execute natively;
- legacy `preflight`/`postflight`: reject before download/mutation;
- legacy uninstall flight blocks in an installed record: status can report
  installed, but native upgrade must stop with an actionable unsupported error;
- unknown structured step/directive: reject the entire plan before mutation.

This is a deliberate support boundary, not a fallback. Native mise must never
silently skip lifecycle code.

## Historical uninstall artifact support

Homebrew receipts can contain app, binary, font, completion, pkg-related
uninstall declarations, `uninstall`, `zap`, and structured uninstall flight
steps. Native upgrade normally needs transition/uninstall actions, but must not
perform `zap`: zap deletes user data and is not part of upgrade.

Build a typed parser with separate policies:

- **presence facts**: targets/pkg IDs proving installation;
- **upgrade transition**: quit/launchctl/delete/pkgutil actions required to
  replace old version safely;
- **ordinary uninstall**: future scope;
- **zap**: explicitly excluded from apply/upgrade.

Only allow contained or explicitly approved absolute system paths. Expand `~`
against the real invoking user only in commands whose semantics require it.
Execute direct argv; never shell strings. Apply the existing sudo policy.

If an old Homebrew record contains unsupported or ambiguous uninstall behavior,
status/apply still succeed as presence operations, but upgrade must fail before
mutation and name the unsupported stanza. Compatibility must not mean unsafe
approximation.

## Private acceptance corpus

### Bootstrap flow

The private downstream configuration is used only as the operator's real list
of applications requiring full support. Its declarative bootstrap runs
`mise bootstrap --yes`; therefore one foreign-owned cask aborts the entire
package phase. Its base configuration has 20 casks; all profiles/optional
declarations total 39 unique casks. Mise must not depend on that repository at
runtime or in public tests; reproduce each shape with public/disposable
fixtures.

### Current 39-cask matrix

The matrix below was fetched from the official cask API on 2026-08-02. Versions
are evidence only and will change; implementation must treat them as opaque.

| Cask | Current artifact families |
|---|---|
| `1password` | app, uninstall, zap |
| `1password-cli` | binary, generated completions, zap |
| `bartender` | app, uninstall, zap |
| `chatgpt` | app, uninstall, zap |
| `claude` | app, uninstall, zap |
| `claude-code` | binary, zap |
| `cleanshot` | app, uninstall, zap |
| `cloudflare-warp` | pkg, uninstall, zap |
| `codex` | binary, generated completions, zap |
| `codexbar` | app, binary, uninstall, zap |
| `firefox@developer-edition` | app, zap |
| `font-jetbrains-mono` | font |
| `font-jetbrains-mono-nerd-font` | font |
| `ghostty` | app, binary completions, manpage, zap |
| `google-chrome` | app, zap |
| `grammarly-desktop` | app, uninstall, zap |
| `grok-build` | binary, generated completions, zap |
| `handbrake-app` | app, zap |
| `jetbrains-toolbox` | app, uninstall, zap |
| `kimi` | app, zap |
| `little-snitch` | app, zap |
| `notion` | app, uninstall, zap |
| `opencode-desktop` | app, zap |
| `orbstack` | app, binary, completions, structured postflight, uninstall, zap |
| `plex-media-server` | app, binary, uninstall, zap |
| `raindropio` | app, zap |
| `sublime-text` | app, binary, uninstall, zap |
| `superwhisper` | app, uninstall, zap |
| `surge` | app, binary, uninstall, zap |
| `tableplus` | app, zap |
| `tor-browser` | app, zap |
| `transmission` | app, zap |
| `tunnelblick` | app, uninstall, structured uninstall-preflight, zap |
| `visual-studio-code` | app, binary, uninstall, zap |
| `visualvm` | app, zap |
| `vlc` | app, command wrapper, zap |
| `yaak` | app, zap |
| `zed@preview` | app, binary, generated completions, uninstall, zap |
| `zoom` | pkg, structured postflight, uninstall, zap |

No current cask in this private matrix exposes a legacy Ruby `preflight` or `postflight`
artifact. Therefore removing Ruby execution does not reduce present downstream
coverage. `tunnelblick` adds the important missing test case: structured
uninstall-preflight metadata must be parsed for native upgrades.

### Observed mixed state

The failing machine showed valid Homebrew metadata for at least:

- `bartender`;
- `google-chrome`;
- `notion`;
- `grammarly-desktop` (canonical config token is `grammarly-desktop`).

These must satisfy base bootstrap without deletion. The test must discover live
state rather than assume these remain installed; deterministic coverage belongs
in isolated fixtures.

### Read-only live control, before destructive comparison

On 2026-08-02, with Homebrew `6.0.14-38-g1f3abf4` and mise
`2026.8.0 macos-arm64`, the configured inventory excluding Ghostty and Codex CLI
had this state:

| State | Count | Casks |
|---|---:|---|
| Homebrew metadata only | 3 | `chatgpt`, `claude`, `opencode-desktop` |
| Both Homebrew metadata and `.mise-cask.toml` | 2 | `grok-build`, `kimi` |
| Mise receipt only | 25 | all other currently installed test targets |
| Not installed | 7 | `firefox@developer-edition`, `handbrake-app`, `jetbrains-toolbox`, `raindropio`, `tor-browser`, `transmission`, `visual-studio-code` |

This mixed state already proves several important differences:

- `brew info --cask` says **Not installed** for mise-only casks even though their
  payloads and Caskroom version directories exist.
- `brew list --cask <token>` can still exit zero for those same casks because it
  can enumerate their Caskroom payload. Therefore that command alone is not a
  valid installed-state oracle.
- `brew list --cask --versions` fails globally at the first mise-only directory:
  `Error: Cask '1password' is not installed.` One incompatible entry breaks
  inventory of otherwise valid Homebrew casks.
- `brew doctor` reports 26 corrupt Caskroom directories (25 test targets plus
  excluded Ghostty) and says they cannot be upgraded as-is.
- Homebrew correctly reports old/current versions and `brew outdated --cask`
  for Homebrew-compatible `chatgpt`, `claude`, `opencode-desktop`, `grok-build`,
  and `kimi`.
- The dual-receipt `grok-build` and `kimi` directories are accepted by Homebrew.
  This confirms `.mise-cask.toml` can coexist as an ignored integrity extension;
  `.metadata` does not need to mean exclusive Homebrew ownership.
- Conversely, `MISE_ENV=ai mise bootstrap packages status --json` reports the
  Homebrew-only `chatgpt`, `claude`, and `opencode-desktop` as `missing`, while
  it reports dual-receipt `grok-build` and `kimi` as installed from their mise
  receipts. This is direct runtime proof that current mise ignores valid
  Homebrew presence rather than an API/token-resolution failure.

These observations strengthen the shared-contract design. Merely making mise
accept Homebrew metadata fixes only half the breakage: the 25 mise-only entries
continue corrupting Homebrew's global cask view until mise writes compatible
metadata.

### Recovered prior synthetic-metadata prototype

Repository history contains an earlier implementation and its subsequent
retirement. It must be treated as experiment evidence, not copied blindly:

| Commit | Result |
|---|---|
| `bd2fe92bd` | wrote `.metadata`, a synthetic tab, config, and installed marker |
| `99e2e50a5` | added no-download backfill for mise-only receipts |
| `a47633fc2` | clarified formula-style identity intent |
| `b91389ddd` | removed the writer because lifecycle facts were incomplete |

The prototype used `homebrew_version: "5.1.15 (mise)"`. It first constructed a
partial uninstall list from mise's parsed app/binary/font/pkg subset, then
changed to an empty list. The partial list was unsafe because Homebrew treats a
non-empty receipt list as authoritative and would omit artifact types mise did
not record. The empty list was also insufficient: Homebrew recovered teardown
from the live API, which can differ from the version actually installed and
fails offline.

Live remnants of that prototype still exist for `grok-build` and `kimi`:

- both are accepted by current Homebrew and current mise;
- both retain `.mise-cask.toml` beside Homebrew metadata;
- `kimi` has an exact-looking app uninstall artifact in the tab;
- `grok-build` has `uninstall_artifacts: []`, while its installed cask JSON now
  contains binary, generated-completion, and zap artifacts.

That `grok-build` JSON is evidence of Homebrew's online recovery/migration, not
evidence that the original Rust writer was complete. It demonstrates the core
historical-integrity bug: a later API definition can populate lifecycle facts
that were never captured at install time.

The prototype also wrote metadata after removing stale versions, without one
transaction spanning payload, external targets, mise receipt, versioned
metadata, and the top-level tab. The new design must solve that atomicity gap.

Reusable lessons from the old work:

- Homebrew mechanically accepts a Rust-written installed marker and tab.
- `.mise-cask.toml` can safely coexist as an ignored extension.
- Write the installed caskfile last so partial metadata is not advertised as
  installed.
- Backfill must require strong mise receipt evidence; never adopt a bare
  Caskroom directory.
- Presence-gate success is not lifecycle parity.
- Exact historical raw/typed uninstall facts, transactional commit, offline
  behavior, and real Homebrew uninstall/upgrade tests are mandatory.
- Do not claim a fake Homebrew version. Determine a truthful producer extension
  and the minimum Homebrew-compatible tab fields empirically.

### Destructive integration experiment protocol

The requested real-machine experiment is valuable because fixture-only tests
cannot expose differences in package receipts, app copies, permissions,
extended attributes, generated links, launch services, or privileged installer
behavior. It must be performed one cask at a time, not by removing the complete
desktop stack at once.

Scope after explicit operator confirmation:

- test the 30 currently installed configured casks above;
- exclude `ghostty` and `codex` completely;
- do not install the seven currently absent optional/profile casks unless the
  operator separately expands scope;
- never use `brew uninstall --zap`, mise zap, or manual deletion of user data;
- preserve application preferences, accounts, databases, and documents;
- stop immediately if a cask cannot be restored before moving to the next one.

The 30-target run includes disruptive security/network/pkg software:
`1password`, `1password-cli`, `little-snitch`, `cloudflare-warp`, `orbstack`,
`tunnelblick`, `zoom`, and `plex-media-server`. Their uninstall/install steps may
stop daemons, unload helpers or network extensions, require sudo/UI approval,
and temporarily interrupt networking or virtualization. Confirmation must name
this scope; a general request to “remove most apps” is insufficient authority
for these side effects.

#### Per-cask sequence

For each confirmed token:

1. Close the app normally and record running processes/services.
2. Capture the current control state described below.
3. Run `brew reinstall --cask --force <token>` using the latest Homebrew. This
   gives every target a clean official installation even when its starting
   state is mise-only or dual-receipt.
4. Capture the complete Homebrew result and verify `brew info`, `brew list`,
   `brew outdated`, and `brew doctor` agree it is healthy.
5. Run `brew uninstall --cask <token>` without `--zap` and verify payload,
   external links, package receipts expected to be removed, and Caskroom
   metadata behavior.
6. Install the same current API definition with the locally built latest-main
   mise binary through a one-cask temporary bootstrap config. Do not run the
   entire external package set.
7. Capture the complete mise result with the same collector.
8. Compare normalized manifests, ignoring only expected producer/time/cache
   fields. Every other difference becomes a finding.
9. Verify the app/CLI actually launches or reports a version, and verify mise
   status/apply idempotence.
10. Verify official Homebrew's `info`, `list`, `outdated`, and `doctor` behavior
    against the mise result. With current mise this is expected to fail; record
    exact output.
11. Leave the confirmed final state installed through mise. If mise installation
    fails, immediately restore with `brew reinstall --cask --force <token>` and
    record the blocker.

Do not batch uninstall targets. A one-at-a-time transaction limits outage and
provides a known restore command after every failure.

#### Evidence collector

Capture before, Homebrew-installed, Homebrew-uninstalled, and mise-installed
states with the same schema:

- exact tool versions and git SHAs;
- API JSON SHA-256, token, opaque version, URL, archive checksum, tap git head,
  artifacts, dependencies, and lifecycle steps;
- recursive Caskroom tree: relative path, entry type, mode, uid/gid, size,
  symlink target, and content hash for receipt/small metadata files;
- installed cask JSON and `INSTALL_RECEIPT.json`, normalized and raw hashes;
- `.mise-cask.toml` and journal state;
- application/target paths, copy-versus-symlink behavior, bundle identifier,
  `CFBundleShortVersionString`, `CFBundleVersion`, and nested app targets;
- `codesign --display --verbose=4` identity/team/designated requirement and
  `codesign --verify --deep --strict` result;
- quarantine and other relevant xattrs (`xattr -lr`), ACLs, flags, owner, and
  permissions;
- binary/wrapper/completion/font/manpage symlinks and resolved targets;
- `pkgutil --pkg-info` and `pkgutil --files` for recorded package IDs;
- launchd labels, privileged helpers, system/network extensions, and running
  processes named by structured uninstall metadata;
- `brew info/list/outdated/doctor` output and exit codes;
- `mise bootstrap packages status --json`, apply, second apply, and upgrade
  output/exit codes;
- application or CLI smoke result appropriate to the artifact.

Do not hash or copy user configuration/data directories named only in `zap`.
Record that they still exist, if needed, without reading their contents.

Normalize only these known nondeterministic values during comparison:

- install timestamps and timestamp-directory names;
- producer version/build metadata;
- cache/source paths that do not affect lifecycle reconstruction;
- archive extraction mtimes when target contents and signatures match.

Never normalize away artifacts, uninstall directives, target paths, pkg IDs,
permissions, symlink topology, signatures, xattrs, or lifecycle behavior.

#### Comparison dimensions and expected current differences

| Dimension | Official Homebrew | Current mise | Required mise result |
|---|---|---|---|
| Installed cask JSON | timestamped `.metadata/.../Casks/<token>.json` | absent | compatible native JSON |
| Top-level tab | `.metadata/INSTALL_RECEIPT.json` | absent | compatible native tab |
| Mise integrity receipt | absent | `.mise-cask.toml` | keep as extension |
| Homebrew `info` | Installed | Not installed | Installed |
| Global `brew list --cask --versions` | healthy | can abort | healthy |
| `brew doctor` | healthy cask | corrupt Caskroom warning | healthy cask |
| Mise status | currently missing for Homebrew-only | installed | installed for both |
| App target | Homebrew artifact semantics | native mise copy/swap | behaviorally equivalent |
| Caskroom staging | Homebrew extracted/staged layout | mise-selected payload subset | sufficient for both lifecycles |
| Uninstall history | exact historical artifacts | mise target/pkg subset | exact structured history plus integrity |
| Legacy hooks | Ruby-supported | Ruby shim | reject unless structured; no Ruby |
| Upgrade | Ruby Homebrew transaction | native mise transaction | native, shared metadata, rollback-safe |

#### Experiment stop conditions

Stop the destructive run and restore the current cask when:

- sudo or a GUI approval cannot be completed safely;
- uninstall would require `--zap` or delete user data;
- a network/security extension warns that reboot is required before reinstall;
- the app contains irreplaceable local-only state outside documented zap paths;
- Homebrew reinstall fails or does not produce healthy metadata;
- mise cannot reinstall the current version;
- smoke verification fails after either installation;
- the next step would require manual deletion outside the token's documented
  artifact set.

### Downstream impact is validation evidence, not mise implementation scope

One downstream consumer currently works around missing cask interoperability by
disabling Codex's Homebrew startup updater. That demonstrates the user-visible
effect of the missing shared receipt, but downstream cleanup is outside this
plan and belongs after a released mise version proves the new contract.

## Implementation sequence

### 1. Add read-only Homebrew metadata types and classifier

Files: `src/system/packages/brew/cask.rs`, preferably split into a focused
`cask/receipt.rs` module if the current file would grow further.

Implement strict parsers and the shared state enum. Wire only `status` first.
Do not change install/upgrade mutation yet.

Verification:

```sh
rtk cargo test --all-features system::packages::brew::cask::tests
```

Required fixtures: modern empty JSON, `url_specs`, explicit artifacts, legacy
internal JSON, legacy Ruby marker, missing/malformed tab, old token, opaque
version with commas/@, symlinks, traversal, dual compatible receipts, mismatched
dual receipts, pending transaction, unmanaged payload.

### 2. Make apply presence-idempotent

Return installed for valid Homebrew metadata and no-op in `install_one`. Snapshot
the fixture tree before/after status, apply, and dry-run; bytes, modes, and
symlink targets must match. Keep corrupt states fail-closed.

This is the minimum fix for the reported bootstrap failure and may be shipped as
the first reviewable commit, but not advertised as full bidirectional lifecycle
compatibility until later steps land.

### 3. Implement compatible metadata writer in Rust

Add typed installed JSON/tab structures, uninstall-artifact derivation, atomic
writer, fsync, journal integration, and reread verification. Fresh mise installs
write both receipts. Reject legacy/unknown lifecycle metadata before download.

Compatibility gate must use a disposable prefix and the real current Homebrew:

```sh
brew list --cask <fixture>
brew info --cask <fixture>
brew uninstall --cask <fixture>
```

Homebrew must consume the Rust-written state without repair warnings. Use a
harmless fixture cask and isolated targets; never user applications.

### 4. Implement native historical transition and upgrade

Parse the installed receipt's exact historical artifacts. Add transactional
backup/restore of both metadata systems and external targets. Support every
artifact family required by the 39-cask matrix, including structured uninstall
steps, before claiming coverage of the external matrix.

Test Homebrew-install → mise-upgrade → Homebrew-upgrade/uninstall and the reverse
sequence. No subprocess may execute `brew` or Ruby during mise operations; test
the command runner/program list.

### 5. Remove the Ruby shim

Delete all shim code and tests. Add explicit rejection tests for legacy Ruby
hooks. Search gate:

```sh
rg -n 'cask_shim|brew.*ruby|cask_ruby_bin|fetch_cask_rb|execute_lifecycle_hook' \
  src/system/packages/brew
```

Expected: no matches associated with brew-cask execution.

### 6. Validate the private 39-cask acceptance matrix

Build the changed mise binary. Read the 39-token inventory from the private
configuration as acceptance input only. Validate every API definition
through parsing, status, install, upgrade, Homebrew inspection, and uninstall in
disposable fixtures. Run a mixed-state bootstrap with a generated temporary
mise config. Do not edit, commit, test, or otherwise mutate the private
repository.

Verification in mise:

```sh
rtk mise run lint-fix
rtk mise run lint
rtk cargo test --all-features system::packages::brew::cask::tests
rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow
rtk mise run ci
```

## Mandatory test matrix

### State and security

- absent;
- valid Homebrew modern JSON/tab;
- valid legacy Homebrew record readable for presence;
- valid legacy mise receipt;
- compatible Homebrew + mise receipts;
- version/token/target mismatch;
- malformed JSON and oversized/deep JSON;
- symlink at every metadata level;
- traversal/NUL/absolute API-controlled component;
- stale journal/backup/temp tree;
- unmanaged Caskroom payload;
- aliases and old tokens;
- opaque versions: comma, `@`, `latest`, non-semver.

### Lifecycle sequences

- Homebrew install → mise status/apply no-op;
- Homebrew install → mise native upgrade → Homebrew list/info/uninstall;
- mise fresh install → Homebrew list/info/upgrade/uninstall;
- legacy mise install → mise upgrade/migration → Homebrew visibility;
- repeated apply and repeated upgrade idempotence;
- failed download/checksum/extraction/preflight/pkg/postflight/receipt write;
- process interruption at every journal checkpoint;
- rollback restores payload, external targets, both receipts, and visibility;
- old receipt contains an unsupported stanza: failure before mutation;
- legacy Ruby hook: failure before mutation;
- zap is never executed during apply/upgrade.

### Private 39-cask acceptance coverage

- exactly 39 unique configured/optional casks at the researched revision;
- all current artifact definitions parse without Ruby;
- base 20-cask mixed-owner apply succeeds;
- all profiles parse and dry-run;
- pkg cases: Cloudflare WARP and Zoom;
- structured steps: OrbStack, Tunnelblick, Zoom;
- wrapper/nested target: VLC and Surge;
- fonts, completions, generated completions;
- Codex-class self-updaters can invoke Homebrew successfully after a mise pour.

## Safety invariants

1. Classification/status is read-only and never elevates.
2. Validate the entire old and new lifecycle plan before first mutation.
3. No shell evaluation; commands use direct argv.
4. No Ruby execution or Homebrew delegation.
5. No semver assumptions; versions are opaque strings.
6. Never infer historical uninstall behavior solely from today's API.
7. Never execute zap during apply/upgrade.
8. Never follow metadata symlinks or escape the canonical prefix/approved target
   roots.
9. New shared receipt is published only after payload activation succeeds.
10. Failure leaves one complete, usable old or new state—not a mixture.
11. Official Homebrew compatibility is proven by its real read/upgrade/uninstall
    commands against disposable fixtures.
12. Unsupported lifecycle semantics fail before mutation; no best-effort skip.

## Repository and PR boundaries

Implementation scope is the mise repository only. Expected production/test/doc
paths are:

- `src/system/packages/brew/cask.rs`, split into focused cask metadata modules
  when appropriate;
- removal of `src/system/packages/brew/cask_shim.rb` after structured native
  coverage is complete;
- brew-cask unit/e2e fixtures and macOS CI workflow paths;
- `docs/bootstrap/packages/brew.md` and generated CLI documentation when
  behavior text changes.

Out of scope:

- any edit, commit, generated artifact, workaround cleanup, or documentation
  change in downstream repositories;
- new commands, subcommands, flags, modes, or configuration keys;
- mise-side uninstall, repair, adoption, import, prune, or handoff workflows;
- registry entries;
- formula behavior;
- delegating operations to Homebrew or Ruby;
- token-specific branches for the external 39-cask inventory.

The public mise PR must not identify the downstream repository or its private
path. Express motivation generically: existing Homebrew-installed casks
should satisfy declarative bootstrap and direct mise pours should be
Homebrew-compatible.

Given review size, implement as one branch but use coherent commits:

1. `fix(brew-cask): recognize Homebrew cask receipts`
2. `feat(brew-cask): write Homebrew-compatible receipts`
3. `feat(brew-cask): upgrade compatible casks natively`
4. `refactor(brew-cask): remove Ruby lifecycle shim`
5. `docs(brew-cask): document shared cask lifecycle state`

Whether maintainers prefer one PR or stacked PRs is a submission decision. The
final behavior must not be split such that a released intermediate version
overwrites foreign state or claims interoperability prematurely.

All commits require DCO signoff and:

```text
Co-authored-by: Codex <codex@openai.com>
```

## Rejected approaches

### Delete only the guard

Rejected. It overwrites payload while retaining stale Homebrew uninstall and
version metadata.

### Treat Homebrew installs as present but never upgrade them

Useful first phase, incomplete final behavior. User can bootstrap, but cannot
continue lifecycle operations from either tool.

### Shell out to `brew upgrade --cask`

Rejected. It depends on Homebrew/Ruby, violates native mise architecture, and
does not make fresh mise installs visible to Homebrew.

### Keep exclusive owner routing

Rejected as final design. It preserves two incompatible receipt worlds and
cannot provide mise → Homebrew continuity.

### Automatically delete Homebrew metadata and adopt

Rejected. It destroys the only historical uninstall record and converts a safe
migration into implicit ownership transfer.

### Write only a minimal caskfile marker

Rejected. Homebrew also relies on the top-level tab/uninstall artifacts.
Partial metadata can make a cask appear installed while leaving uninstall or
upgrade unable to clean historical artifacts.

### Continue evaluating Ruby with a shim

Rejected. It creates an unbounded compatibility/security surface and conflicts
with the native Rust requirement. Structured API operations are the support
boundary.

## STOP conditions during implementation

Stop and redesign/report rather than improvise if:

- current Homebrew rejects truthful receipts without a forged
  `homebrew_version` or `built_on` value;
- Homebrew requires execution of an installed `.rb` to list/uninstall any of the
  39 target casks;
- historical uninstall artifacts cannot be represented safely from stored JSON;
- a required current cask exposes lifecycle behavior only through Ruby source;
- rollback cannot preserve the original Homebrew record byte-for-byte;
- compatibility tests require touching real user applications or metadata;
- a token-specific implementation branch appears necessary;
- native support would need to silently skip an unknown privileged operation.

## Definition of done

- The original `Homebrew owns this cask` error is gone because valid metadata is
  recognized, not because the safety check was weakened.
- Existing official Homebrew casks satisfy mise bootstrap with zero mutation.
- Mise upgrades those casks entirely in Rust.
- Fresh/migrated mise casks are consumable by official Homebrew.
- No brew-cask code executes Ruby or delegates to Homebrew.
- Corrupt, legacy-unsupported, and ambiguous state fails before mutation.
- All 39 externally derived cask shapes pass the native metadata/lifecycle
  matrix in mise-owned disposable fixtures.
- The real base bootstrap succeeds with mixed installation history.
- Unit, e2e, lint, CI, Homebrew compatibility, rollback, and idempotence gates
  all pass.
