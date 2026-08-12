# Plan 018: Prove operational parity with real Homebrew

Status: IN PROGRESS
Priority: P0
Effort: L
Planned against: combined SHA produced by plan 017
Depends on: 017

Execution start (2026-08-13):

- exact combined starting SHA: `ca03fc901f49c9d8a264672c77d5dce4e66af156`;
- exact prerequisite #11915 SHA:
  `a5046918a091a421289405a9ac380717e3145ffb`;
- official `6.0.17` tag revalidated with
  `rtk git ls-remote https://github.com/Homebrew/brew.git refs/tags/6.0.17`:
  `4dacfe77a24dead72de749c0876028b77b99cd04`;
- drift found before mutation: the macOS job used the runner's unpinned Homebrew
  checkout, while the Linux marker claimed `not-installed` after installing and
  executing a live Homebrew checkout. Neither runtime identity is valid plan
  018 proof.

Execution continuation (2026-08-13):

- formula prerequisite advanced to
  `300551418f7a4fc2b4579eac2993eff7f62c8aa5` after exact-head source proof
  exposed an Ubuntu GCC provenance parser defect and the general container E2E
  exposed an unsafe-directory Git identity probe; the pinned macOS oracle then
  exposed the isolated harness omitting fixed `/usr/sbin` and `/sbin`, which
  prevented its package-receipt no-mutation probe from invoking `pkgutil`;
- combined stack was restacked without conflict; current code-proof SHA is
  `0fd2adeae546d4ae9f5cd2cabfcd63960342b039`. The previous exact evidence head
  `7db9c28bb62e7868930227f03f6c7793c8ac675d` failed before cask mutation because
  the isolated harness omitted the fixed system path containing `pkgutil`;
- first pinned macOS execution proved runtime `6.0.17` at exact SHA
  `4dacfe77a24dead72de749c0876028b77b99cd04`, then correctly failed because
  `brew uninstall` in 6.0.17 does not accept the stale oracle's `--yes` flag.

## Objective

Run non-skippable, disposable, pinned differential tests on the combined head.
Prove runtime and ownership equivalence—not merely receipt JSON shape—for formula
and cask mechanisms in both ownership directions.

## Ground rules

- Use plan 009's explicit disposable capability and completion markers.
- Pin Homebrew `6.0.17` source/behavior and record exact API/tap payload digests.
- Freeze tested formula/cask versions, artifact checksums, and tap/API snapshots
  for deterministic comparisons. Any explicit current-live probe records drift
  separately and cannot silently change the pinned oracle expectation.
- Real `brew` appears only in e2e oracle code.
- A name appearing in a corpus is not coverage. A mechanism executes or is
  explicitly unsupported and rejected before mutation.
- Normalize only nondeterministic facts such as timestamps and temp roots. Never
  normalize entry type, link target, path, permissions, receipt field presence,
  lifecycle output, package receipt, or runtime result.

## Cask mechanism matrix

Replace `plans/007-corpus-results.md` single-label classes with one pinned row per
cask and Boolean/details columns for every observed mechanism, including:

- moved app, nested/wrapper app, binary, generated completion, manpage, font;
- pkg installer, preflight/postflight, launchctl, quit, signal, script, pkgutil,
  delete, auto-update, versioned token, privileged/system extension;
- install/uninstall/zap distinction and interactive approval requirement.

A cask may have many mechanisms. Pin payload/tap revision and extraction date.
Choose at least one real oracle representative per supported mechanism and each
important interaction (for example app+manpage+completions, pkg+signal, app+quit,
font+Linux XDG). For an unsafe/interactive mechanism, add a preflight fixture
proving fail-closed zero mutation; label it UNSUPPORTED, never equivalent.

Ghostty must be classified and tested as app + two manpages + three completions.
Hidden Bar cannot represent it. Include the original 37 names, but report
coverage by mechanism and combination.

## Cask ownership directions

For each selected representative, execute as safe/applicable:

1. mise install → real brew info/list/upgrade or reinstall/uninstall;
2. real brew install → mise status/apply/upgrade/prune;
3. failure/recovery injection for reversible fixtures;
4. foreign target collision and predecessor ownership cases.

Compare Caskroom topology, public targets, backlinks, entry kinds, metadata JSON,
installed caskfile, tab, download metadata, permissions, package/service state,
upgrade residue, and final uninstall post-state. Run Linux config/XDG assertions
separately from macOS.

## Formula corpus

Use disposable macOS canonical `/opt/homebrew` for bottles whose paths/runtime
depend on the canonical prefix. Include at minimum:

- `ca-certificates`;
- `openssl@3`;
- a formula with persistent `etc` content;
- a formula with persistent `var` content;
- a formula with nontrivial typed `post_install`;
- an archive/non-OCI bottle fixture;
- a forced source build fixture;
- an already-current unsupported-lifecycle dependency fixture such as the
  relevant `postgresql@17` metadata shape.

For supported real formulae, execute both directions:

1. mise install → real brew info/list/doctor/postinstall/upgrade/uninstall;
2. real brew install → mise status/apply/upgrade/prune.

Compare Cellar contents and entry kinds, opt, `var/homebrew/linked`, public links,
shared `etc`/`var`, receipts, SBOM, formula snapshot, generated post-install
state, runtime behavior, and post-uninstall state. Preserve user-modified
persistent files through upgrades in both engines.

## Mandatory essential-mac regression

Configure only `brew:kimi-code = "latest"`. Do not declare its dependencies.
After mise convergence at canonical prefix, assert readable OpenSSL config and
CA bundle, direct OpenSSL TLS verification, default Node fetch to the Kimi OAuth
endpoint, and the installed Kimi runtime/login path far enough to prove TLS.

No `SSL_CERT_FILE`, `NODE_EXTRA_CA_CERTS`, `NODE_OPTIONS`, proxy CA injection, or
TLS verification disable may be present. Record environment key names only;
never upload tokens, response bodies, or secrets.

Then exercise:

- real Homebrew accepting/upgrading/postinstalling/uninstalling the mise-created
  dependencies;
- a fresh real-Homebrew-created chain read by mise as healthy/Noop;
- root-only status detecting a deliberately removed dependency lifecycle output;
- plan 012 lifecycle-only repair preserving keg/receipt/link inodes;
- default Node/OpenSSL TLS recovery after repair.

## Mandatory legacy migration regression

Construct old mise state with valid keg+opt, missing linked-keg, `.bottle/etc`
still in keg, missing shared `etc`, absent lifecycle record, and skipped
post-install. Status must identify each missing lifecycle phase without writes.
Apply repairs only provable effects. An unprovable old default or generated
output must yield an exact reinstall requirement and zero unsafe mutation.

## CI proof requirements

Every job records:

- combined mise SHA;
- Homebrew version/source SHA;
- API/tap fixture digests;
- platform/image;
- named fixtures and mechanism count;
- completion marker from plan 009;
- raw and normalized snapshot artifacts, with secrets excluded.

The workflow fails if expected test names, fixture counts, assertions, or marker
are absent. Zero-second or empty-log success is invalid. Retry may address
network flakes but may not skip a fixture or reuse an unverified stale prefix.

## Verification commands

```bash
rtk cargo test --bin mise system::packages::brew
rtk mise run test:e2e e2e/cli/test_system_install_brew_linux
rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow
rtk mise run test:e2e e2e/cli/test_system_install_brew_formula_lifecycle_macos_slow
rtk mise run ci
```

The dedicated GitHub jobs, not a developer machine, execute destructive bodies.

## Done criteria

- Combined exact head passes Linux, macOS, Windows, lint, unit, e2e, and every
  dedicated non-skipped oracle.
- Both ownership directions pass for supported formula/cask mechanisms.
- Root-only Kimi chain passes default TLS without overrides.
- Legacy lifecycle damage is diagnosed read-only and repaired/rejected safely.
- All 37 casks have pinned multi-mechanism accounting; each supported mechanism
  has real execution proof, and unsupported mechanisms fail closed.
- Corpus result document contains job URLs and completion-marker evidence.

## Stop conditions

Do not count fixture/unit coverage as real operational equivalence. Do not label
an interactive destructive mechanism supported merely because it cannot run in
CI. Do not normalize away a structural mismatch.
