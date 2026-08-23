# mise Homebrew fix delivery plan

Date: 2026-08-22
Status: canonical plan

## Completion condition

Work is complete only when all of the following are true:

1. A fresh Mac can run `mise bootstrap --yes` twice from essential-mac.
2. A Mac with healthy Homebrew-owned casks can run the same command twice.
3. `brew-cask:1password-cli` is recognized without ownership transfer.
4. Status, apply, upgrade, and prune never mutate Homebrew-owned cask state.
5. Poured `ca-certificates`, `openssl@3`, Node, and their consumers have all
   required shared lifecycle state.
6. Missing mise-owned lifecycle effects are detected and repaired without
   repouring or rewriting healthy kegs and receipts.
7. Unsupported operations and ambiguous ownership fail before mutation.
8. Existing source builds, third-party taps, formula installs, mise-owned
   casks, platform skipping, and adoption behavior remain supported.
9. Each upstream pull request contains one understandable concern and ordinary
   project gates pass without a bundled CI redesign.
10. essential-mac no longer needs its temporary bridge after a released mise
    containing both fixes is verified.

## Delivery overview

```text
essential-mac temporary bridge
        |
        +-- immediate reliable bootstrap

current mise main
        |
        +-- PR 1: recognize Homebrew-owned casks
        |
        +-- direction agreement for public post_install_steps
                |
                +-- PR 2: apply + health-check + repair formula lifecycle

released mise with both fixes
        |
        +-- essential-mac bridge removal
```

The two upstream branches are independent. PR 1 should be made ready first
because it is smaller and directly removes the current bootstrap blocker.
Formula discussion and local prototyping can proceed concurrently, but no
second giant stacked branch should be published.

## Phase 0: preserve old evidence, stop old development

- Keep exact heads `a1df6b7` and `97640cc` reachable for test and design
  salvage.
- Do not add commits, merge-main updates, CI expansions, or review-bot loops to
  PR #11910 or PR #11915.
- Do not ask jdx to re-review either old draft.
- Create every successor from freshly fetched upstream main.
- Repair or replace the broken local formula worktree rather than trusting its
  missing parent worktree metadata.
- Treat the archived reports as evidence, not as branches to trim in place.

## Phase 1: temporary essential-mac bridge

Goal: make this repository usable immediately without pinning either oversized
mise draft.

### Cask bridge

Temporarily remove base-profile casks from `[bootstrap.packages]` and express
the same desired list in Rust `essential-mac gaps`. The gap must classify each
cask before acting:

1. Native Homebrew `.metadata` and healthy version topology:
   preserve it and report success.
2. A valid mise `.mise-cask.toml` receipt:
   preserve it and report the temporary mixed-owner state; do not ask Homebrew
   to adopt or replace it.
3. No recognized installation:
   install with the real Homebrew CLI.
4. Both receipts, multiple versions, malformed metadata, or conflicting target:
   fail before mutation with an exact repair message.

This classifier prevents the bridge from blindly handing mise-owned Ghostty to
Homebrew while also avoiding the current native Homebrew conflicts.

Keep the desired cask list in one Rust constant or typed catalog so the
temporary config deletion cannot silently omit an application. Add a parity
test comparing the bridge catalog to the removed TOML declarations captured in
the change.

### Formula bridge

After package apply, validate:

- `etc/ca-certificates/cert.pem` exists, is readable, and contains
  certificates;
- `etc/openssl@3/openssl.cnf` exists;
- `etc/openssl@3/cert.pem` is the expected symlink and resolves to the CA
  bundle;
- default OpenSSL trust succeeds without `SSL_CERT_FILE` or other overrides.

If and only if those postconditions are unhealthy and Homebrew recognizes both
formula receipts, run:

```bash
brew postinstall ca-certificates openssl@3
```

Then verify all postconditions again and fail bootstrap if any remain broken.
Never uninstall, reinstall, relink a keg, rewrite a receipt, or remove user
content from this bridge.

### Bridge verification

Test on disposable macOS machines representing:

- clean prefix;
- current mixed ownership;
- healthy native cask;
- healthy mise cask;
- malformed/dual ownership;
- missing CA bundle;
- missing OpenSSL certificate link;
- healthy state where every repair path must remain a no-op.

Run bootstrap twice. Snapshot cask receipts, version directories, artifact
targets, formula keg inode, formula receipt, and `opt` topology before and
after the second run.

### Bridge stop condition

The bridge is temporary. It must carry a link to this plan and explicit removal
conditions. It must not become a permanent second package architecture.

## Phase 2: upstream cask successor

Suggested title:

```text
fix(brew): recognize Homebrew-owned casks
```

### Scope

- Add a private producer-aware installed-state classifier.
- Parse only native facts needed to prove token, version, source tap, and
  artifact topology.
- Tolerate unrelated additive receipt fields.
- Treat a healthy native Homebrew installation as installed for status and
  bootstrap.
- Make apply and ordinary upgrade no-op for healthy Homebrew ownership,
  including catalog version drift.
- Reclassify after the Caskroom lock and before any mutation.
- Keep native status inspection read-only.
- Preserve existing native `.metadata` prune guard.
- Preserve current explicit adoption behavior; recognition alone never adopts.
- Return a conflict state for ambiguous or malformed native topology.

### Explicit non-goals

- no formula changes;
- no native receipt conversion or serializer parity;
- no foreign upgrade, uninstall, or prune;
- no generic `PackageState` reinterpretation;
- no manager-name special cases in shared drivers;
- no doctor or install-hint policy rewrite;
- no third-party cask removal or tap-trust policy;
- no Aqua, task, lock, config, or HTTP changes;
- no broad cask DSL rewrite;
- no workflow matrix changes.

### Required tests

1. Valid native `1password-cli` receipt, version directory, `op` target, and
   completion topology classify as `HomebrewOwned("2.39.0")`.
2. Status reports installed without writing any file.
3. Apply is no-op when native version equals catalog version.
4. Apply and upgrade remain no-op when native version differs from catalog.
5. Prune never selects Homebrew-owned state.
6. Malformed receipt, missing version, multiple versions, or target mismatch
   returns conflict before mutation.
7. Current mise-owned install, upgrade, prune, and self-updating behavior stay
   unchanged.
8. Explicit adoption still follows the current-main contract.
9. macOS/Linux platform-unavailable behavior remains consistent for apply,
   status, doctor, and install hints.
10. Focused macOS e2e installs `1password-cli` using Homebrew, runs status,
    apply, upgrade, and prune through exact-head mise, then proves the native
    tree and receipt are unchanged.

### Expected review shape

Production changes should stay inside the cask backend, with small fixture/unit
coverage, one focused existing e2e extension, and exact ownership docs. If the
implementation begins changing formula, shared package semantics, generic CLI
consumers, or CI architecture, stop and rescope.

## Phase 3: formula direction agreement

Before publishing formula code, post one short Discussion or Discord proposal:

> A verified bottle can currently be marked installed while its structured
> post-install/shared `etc` state is absent, which leaves OpenSSL without
> default CA trust. May mise consume the public formula API's typed
> `post_install_steps` under its existing API-plus-bottle-SHA trust boundary,
> preflight the full closure before mutation, and repair only recorded
> mise-owned effects? Unknown operations and opaque Ruby would fail before
> mutation. This would not change source builds, taps, casks, sandboxing, or CI
> architecture.

Direction agreement must cover:

- use of public `post_install_steps`;
- supported initial operation set;
- bottle-contained helper execution boundary;
- lifecycle manifest and repair ownership;
- fail-before-mutation behavior for unsupported operations.

Do not introduce an internal JWS client merely to prototype this direction.
The existing public metadata and verified-bottle boundary is the smallest
coherent proposal.

## Phase 4: upstream formula lifecycle successor

Suggested title:

```text
fix(brew): apply and repair formula post-install steps
```

Fresh application and existing-state repair must ship together. An apply-only
PR would not repair the already-poured unhealthy keg that triggered this work.

### Data model

Extend the existing public formula API model with:

- `post_install_defined`;
- a tagged typed `post_install_steps` representation;
- bounded path specs, guards, and source-glob declarations used by supported
  operations.

Unknown top-level formula fields remain tolerated as today. Unknown operation
types, unknown semantic fields inside a supported operation, unresolved
templates, unsupported path roots, or opaque `post_install_defined = true`
must fail during preflight.

### Initial supported operation set

Support every structured operation required by the current essential-mac base
closure:

- `mkdir_p`;
- guarded `remove`;
- recursive `copy`;
- `symlink`, including bounded declared source globs;
- `run` only when the executable resolves inside the checksum-verified keg and
  all inputs/outputs resolve inside explicitly permitted roots;
- activation of bottle `.bottle/etc` and `.bottle/var` defaults using
  create-if-absent semantics that never overwrite existing user state.

No generic Ruby evaluation. No arbitrary executable search through `PATH`.
No helper or formula-source digest allowlist tied to one live revision.

### Planning and mutation order

1. Resolve the complete existing formula closure.
2. Select every bottle and its existing SHA-256 identity.
3. Parse and compile every lifecycle plan.
4. Validate operation types, templates, path roots, helpers, guards, and
   ownership.
5. If any closure member is unsupported, fail with zero prefix mutation.
6. Download and verify bottles through existing machinery.
7. Pour kegs and activate shared defaults.
8. Apply typed post-install operations.
9. Verify postconditions.
10. Persist a small mise lifecycle manifest inside the keg.
11. Publish normal links/healthy state only after lifecycle success.

Reuse existing closure, bottle verification, native receipt, relocation,
source-build, and link code. Do not claim them as new behavior.

### Lifecycle manifest

Record only what offline health and safe repair require:

- schema version;
- formula name and package version;
- bottle SHA-256;
- normalized lifecycle-plan digest;
- mise-owned shared paths and expected kinds;
- exact symlink targets;
- helper identity as part of the verified keg;
- successful operation completion.

Do not rewrite Homebrew's native receipt with private lifecycle fields.

### Health and repair

Status must traverse requested formula dependency health. A requested root is
not healthy when a transitive lifecycle effect is missing.

For mise-owned state:

- exact existing directory/copy/symlink effects are healthy;
- a missing recorded idempotent effect is `NeedsRepair`;
- conflicting or foreign content is conflict, not repair;
- repair applies only the missing effect;
- repair preserves keg inode, bottle bytes, native receipt, and active link.

For native Homebrew-owned formula state, inspection remains read-only. mise
must not repair or adopt it implicitly.

Legacy mise kegs without a lifecycle manifest require one bounded migration:
compile the current supported plan, validate bottle/receipt identity, inspect
shared state, then either record healthy state or repair missing mise-owned
effects. Ambiguous provenance fails with guidance.

### Required unit and fixture tests

1. Two `ca-certificates` revisions compile equivalent supported run plans.
2. `openssl@3` compiles the expected CA symlink plan.
3. Node compiles `mkdir_p`, guarded remove, recursive copy, forced symlinks,
   and source globs.
4. Unknown operation type fails before mutation.
5. Unknown semantic operation field fails closed while unrelated formula fields
   remain tolerated.
6. Opaque Ruby post-install fails before mutation.
7. Complete-closure preflight rejects one unsupported dependency before any keg
   or shared path is created.
8. Helper execution cannot escape the verified keg or permitted output roots.
9. Shared defaults create missing paths but never overwrite existing values.
10. Missing CA bundle or OpenSSL cert link becomes `NeedsRepair`.
11. Repair restores only the missing effect and preserves keg/receipt/link
    identity.
12. Conflicting foreign output is not overwritten.
13. Requested root status reflects unhealthy transitive dependencies.
14. Existing formula, source-build, third-party tap, Linux, macOS, and Windows
    unit suites remain green.

### Focused macOS oracle

Extend an existing macOS e2e path; do not create a new build pipeline.

- Use an essential-mac-shaped root such as `brew:agent-browser` or
  `brew:wget`, exercising Node/OpenSSL/CA closure as applicable.
- Install exact-head mise into a disposable canonical prefix.
- Assert CA bundle, OpenSSL config, cert symlink, Node/npm links, and requested
  executables.
- Generate a local CA and server certificate.
- Run a local HTTPS server.
- Prove OpenSSL and a real consumer connect using default trust, without
  `SSL_CERT_FILE`, `NODE_EXTRA_CA_CERTS`, or a public internet endpoint.
- Delete one recorded effect.
- Prove status reports repair.
- Re-run bootstrap and prove only that effect is restored.
- Prove the formula keg and native receipt remained unchanged.
- Confirm current stable Homebrew can still read `list`/`info` and perform
  cleanup on the disposable runner.

### Explicit non-goals

- no internal signed-index/JWS client;
- no OCI descriptor identity system or recursive `Fresh` retry;
- no embedded live recipe/helper hashes;
- no whole Homebrew internal schema mirror;
- no trust-anchor rotation system;
- no source-build DSL overhaul;
- no formula resource or external-patch removal;
- no sandbox, seccomp, Landlock, process-group, or descriptor-filesystem
  framework;
- no generic WAL/recovery subsystem;
- no SBOM system;
- no cask behavior;
- no third-party tap trust-policy change;
- no CI architecture change.

## Phase 5: review and CI discipline

For each successor:

1. Verify `git diff --stat upstream/main...HEAD` represents one concern.
2. Remove every unrelated changed file.
3. Use logical commits without merge-main or review-retry noise.
4. Run format, Clippy, focused unit tests, and ordinary project gates.
5. Run the focused macOS e2e manually or through the existing test tranche.
6. Address every actionable automated review finding.
7. Open a draft only when the diff, tests, and description already exist.
8. Mark only the cask successor ready first.
9. Make the formula successor ready after direction agreement and focused
   evidence.

Behavior PRs must not add a dedicated Homebrew build matrix. If maintainers
later want permanent path-gated brew jobs or a differential corpus, propose
that as a CI-only change after behavior merges.

## Phase 6: old PR cleanup and maintainer communication

When the cask successor exists and is green, reply to jdx:

> Agreed—the current PR combines independent bugs with broad Homebrew-parity
> and CI work, hiding its purpose. I replaced it with a current-main cask-only
> ownership no-op; the OpenSSL/CA lifecycle work will be a separate PR after
> confirming the narrow public-metadata boundary.
>
> *AI-assisted — Tool: Codex; model: openai/gpt-5; version: unavailable.*

Then:

- link #11910 to the cask successor and close it;
- link #11915 to the formula successor and close it only after the replacement
  is public;
- never require reviewers to compare old and new histories;
- preserve old exact heads locally until successors merge.

No GitHub reply, thread resolution, closure, or PR creation should occur
without explicit user authorization at execution time.

## Phase 7: remove the essential-mac bridge

After a released mise includes both successors:

1. Install that release through the normal new-Mac bootstrap path.
2. Test a clean machine and a machine with native Homebrew cask ownership.
3. Restore casks to `[bootstrap.packages]`.
4. Remove the temporary Rust cask fallback.
5. Remove the `brew postinstall` formula repair bridge.
6. Keep an essential-mac regression that runs bootstrap twice and checks
   cask ownership plus OpenSSL default trust.
7. Update `MISE-FIRST.md` with the minimum fixed mise release.

The cleanup is one focused essential-mac PR. Completion is not proven while
the repository still depends on dual package paths.

## Final acceptance checklist

### Casks

- [ ] Native Homebrew `1password-cli` reports installed.
- [ ] Bootstrap succeeds without reinstall or adoption.
- [ ] Status is read-only.
- [ ] Upgrade does not take over native ownership.
- [ ] Prune never removes native ownership.
- [ ] Malformed ownership fails before mutation.
- [ ] Mise-owned casks retain normal install/upgrade/prune behavior.
- [ ] Platform-unavailable behavior remains consistent across callers.

### Formulae

- [ ] All essential-mac closure lifecycle steps preflight before mutation.
- [ ] `ca-certificates` builds a valid shared CA bundle.
- [ ] `openssl@3` has config, cert directory, and correct default cert link.
- [ ] Node post-install shared state is complete.
- [ ] Default local TLS works without environment overrides.
- [ ] Missing effects become repair state.
- [ ] Repair preserves keg and native receipt identity.
- [ ] Unknown or opaque lifecycle fails before mutation.
- [ ] Two supported formula revisions pass.
- [ ] Existing formula/source/tap behavior remains supported.

### Delivery

- [ ] Each upstream diff contains one concern.
- [ ] No unrelated CI, sandbox, source, trust, or package-state systems.
- [ ] Ordinary gates and focused oracles pass.
- [ ] Automated review findings are addressed.
- [ ] Old drafts are linked and closed only after replacements exist.
- [ ] Released mise passes clean and existing-machine essential-mac bootstrap.
- [ ] Temporary bridge is removed.
