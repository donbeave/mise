# Plan 013: Make cask activation owned, transactional, and recoverable

Status: IN PROGRESS
Priority: P0
Effort: L
Planned against: #11910 `05ccd7ab8`
Depends on: 009
Implementation start: #11910 `279530a1e33814c7c6a49aca6198ba67efb124b3`
Implementation commit: `bca8747bd362679a6c97d72f3e0001539730f011`

Drift check (2026-08-13): the activation code still builds one
`current_targets` vector from apps, binaries, completions, and fonts. Apps are
already installed publicly before `ArtifactLinkTransaction::begin` moves every
entry in that vector to backup; the activation closure recreates links/fonts
but not apps, so commit still deletes the only app copy. The journal remains a
completed-string list rather than an executable recovery state machine.

## Objective

Replace the current mixed app/link/backup flow with a typed artifact activation
transaction that proves ownership before mutation, matches Homebrew moved
artifact topology, and recovers safely after interruption.

## Current defects

- `cask.rs:427-504`: app is installed live, added to `current_targets`, moved to
  transaction backup, never recreated, then deleted by `commit`. A successful
  app install can finish with no app.
- `swap_app` and generic target backup accept any existing target. Fresh install
  can destroy a manually installed app, unrelated binary/symlink, or user font.
- Apps/fonts are duplicated in Caskroom and public locations. Homebrew's moved
  artifact places the payload publicly and makes the Caskroom source a backlink.
- The serialized journal is never consumed for recovery. Pending maps to Absent,
  so apply may rerun pkg/app/hook work of unknown outcome.
- The predecessor Caskroom backup can be deleted before outer activation commit.

## Files in scope

- `src/system/packages/brew/cask.rs`
- cask receipt/state helpers if extraction is needed
- cask unit fixtures and focused e2e tests

Do not add lifecycle action breadth here; plan 014 owns teardown semantics.

## Required architecture

Compile raw cask artifacts into a `PreparedCaskTransaction` before download or
mutation. It contains:

- complete receipt inventory;
- activation effects only;
- artifact lifecycle kind (`Moved`, `Symlinked`, generated wrapper/completion,
  installer, metadata-only);
- source and public target;
- predecessor ownership proof;
- expected post-state and health predicate;
- undo/recovery boundary and irreversibility classification.

`receipt_targets` and `activation_targets` are distinct. The transaction may
backup only a target it recreates or deliberately removes in the same durable
phase.

## Implementation steps

1. Split complete artifact inventory from activation work. Fix the P0 by keeping
   a newly installed app outside generic link backup unless an app activation
   step recreates it before commit.
2. Build a target-claim table before any preflight/pkg/app mutation. Accept only:
   absent target; target provably owned by the exact installed predecessor; or
   target already in the expected successor post-state. Any other target is a
   zero-mutation collision error identifying owner evidence and path.
3. Apply the ownership rule uniformly to app, font, binary, completion, wrapper,
   manpage-ready targets, regular files, directories, and symlinks. Do not follow
   a foreign symlink while checking ownership.
4. Implement Homebrew moved topology for apps/fonts: public payload plus
   Caskroom backlink. Validate backlink direction, entry type, target containment,
   and predecessor identity. Replace duplicate-content fingerprint assumptions.
5. Represent generated binaries/completions separately from moved artifacts;
   preserve their correct Homebrew topology.
6. Turn the journal into a durable phase machine. Persist intent and fsync before
   each externally visible phase; persist completion immediately after. Pending
   state is `NeedsRepair`, never Absent.
7. Define recovery per phase. Reversible filesystem activation may resume or
   roll back from recorded backups. Unknown-result installer/hook phases never
   auto-replay; return exact manual/reinstall guidance.
8. Retain predecessor Caskroom and public-target backups until app, external
   targets, metadata, and journal commit all succeed. Cleanup is the final phase.
9. Ensure status validates expected topology read-only and diagnoses a precise
   broken phase.

## Required tests

- App install regression: app exists after commit; Caskroom backlink, metadata,
  status, and receipt inventory are correct.
- Inject failure after every durable phase, then restart status/apply. Prove no
  app disappearance, lost predecessor, or blind irreversible replay.
- Fresh-install collisions for manual app, regular binary, foreign symlink,
  directory, and font all fail before mutation.
- Same-cask predecessor targets upgrade successfully; already-correct successor
  topology is idempotent.
- Real-Homebrew moved app/font fixtures are recognized without rewriting and can
  enter plan 014 teardown.
- Malicious Caskroom backlink escaping permitted roots is rejected without
  following or removing its target.

## Verification

Local proof at `bca8747bd362679a6c97d72f3e0001539730f011`:

- `rtk cargo test --bin mise system::packages::brew::cask` — 169 passed.
- `rtk cargo test --bin mise system::packages::brew` — 226 passed.
- `rtk cargo clippy --workspace --all-features --all-targets -- -D warnings` —
  zero errors (one pre-existing linker warning).
- Regressions cover successful app activation retaining both public app and
  Caskroom backlink, protected predecessor replacement, foreign target
  rejection, activation rollback, durable pending-phase reporting, and
  installed-receipt topology independent of current catalog metadata.

Hosted macOS differential proof remains required by plans 017–018, so this plan
stays IN PROGRESS despite local implementation passing.

```bash
rtk cargo test --bin mise system::packages::brew::cask
rtk mise run test:e2e e2e/cli/test_system_install_brew_macos_slow
rtk mise run lint
```

The macOS body must carry plan 009's completion marker. Real parity remains plan
018; focused fixtures here prove the transaction invariants.

## Done criteria

- Successful app install cannot delete its app.
- No unowned target is replaced.
- Apps/fonts use and validate Homebrew moved topology.
- Pending state is recoverable NeedsRepair with phase detail.
- Predecessor backups survive until the entire transaction commits.

## Stop conditions

Never auto-delete or replace a target with ambiguous ownership. Never rerun an
irreversible action whose previous completion is unknown. Never weaken topology
validation to accept both duplicate and moved shapes as equivalent; legacy
conversion must be explicit and ownership-gated.
