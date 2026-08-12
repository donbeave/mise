# Implementation plans — brew/brew-cask shared-prefix interoperability

Deep merge-readiness audit: 2026-08-13.

- PR #11910 audited head: `05ccd7ab87ac5c243aa8a17ea7888eb8f161e6c3`.
- Prerequisite PR #11915 audited head: `b94b6b1c12041a84b317b8b9e4cfb720a60ccbd4`.
- Homebrew reference: release `6.0.17` and its pinned source behavior.
- Verdict: **DO NOT MERGE either PR.** The individual green suites are not
  evidence for the product that would ship. The branches conflict, the
  differential jobs skipped their bodies, and both formula and cask state
  machines have merge-blocking correctness gaps.

The original controlling design is
[brew-cask-native-homebrew-interoperability.md](brew-cask-native-homebrew-interoperability.md).
Plans 001–008 remain implementation history, not current acceptance proof.
Plans 009–019 are the corrective execution graph and override incompatible
completion claims in the older plans.

## Problem being solved

Mise's `brew:` and `brew-cask:` backends install natively into a shared Homebrew
prefix. Before this stack, formula lifecycle state was incomplete and casks were
mise-owned through `.mise-cask.toml`, so real Homebrew and mise could disagree
about installed health, ownership, upgrade, and removal.

- #11915 adds formula `.bottle/etc`, `.bottle/var`, and typed post-install
  lifecycle handling.
- #11910 adds native formula receipts/SBOM/linked state plus cask `.metadata`,
  Homebrew-owned state recognition, legacy conversion, and receipt-driven
  teardown.

Direction is correct. Required contract is stronger than compatible receipt
shape: both engines must preserve the same operational state, including shared
configuration, generated lifecycle outputs, artifact topology, user edits, and
safe successor-aware removal.

## Branch and PR policy

The historical one-branch policy no longer describes reality. Preserve the
two-PR prerequisite relationship:

1. Correct #11915 on its formula-lifecycle branch.
2. Rebase #11910 onto the exact accepted #11915 head.
3. Resolve `api.rs`, `pour.rs`, and `source.rs` semantically, not with an
   ours/theirs merge.
4. Treat only the resulting combined SHA and its full CI/oracle suite as
   merge-readiness evidence.

Both PRs stay draft until plan 019 passes. Commits must be conventional,
created with `git commit -s`, and include:

`Co-authored-by: Codex <codex@openai.com>`

PRs target `main`, never `release`. Push exact plan checkpoints. Any GitHub
content written with AI help must append the repository-required disclosure.

## Execution order and status

Historical rows retain what was built while explicitly losing acceptance:

| Plan | Historical scope                        | Current status                            |
| ---- | --------------------------------------- | ----------------------------------------- |
| 001  | Shared receipt schema and emulation pin | IMPLEMENTED; REVERIFY ON COMBINED HEAD    |
| 002  | Read Homebrew-installed cask state      | IMPLEMENTED; REVERIFY ON COMBINED HEAD    |
| 003  | Write native cask metadata              | ACCEPTANCE INVALIDATED; see 013, 015, 018 |
| 004  | Formula receipts and SBOM               | ACCEPTANCE INVALIDATED; see 011, 012, 018 |
| 005  | Legacy cask conversion                  | ACCEPTANCE INVALIDATED; see 013, 018      |
| 006  | Removal parity                          | ACCEPTANCE INVALIDATED; see 014, 018      |
| 007  | Differential oracle                     | FALSE-GREEN; see 009 and 018              |
| 008  | Full-interoperability documentation     | CLAIMS UNSUPPORTED; see 019               |

Corrective plans:

| Plan | Title                                                        | Priority | Effort | Depends on         | Status      |
| ---- | ------------------------------------------------------------ | -------- | ------ | ------------------ | ----------- |
| 009  | Make destructive oracles safe and non-skippable              | P0       | M      | —                  | DONE        |
| 010  | Compile formula lifecycle once; preflight and confine it     | P0       | L      | 009                | DONE        |
| 011  | Unify bottle/source finalization and truthful provenance     | P0       | L      | 010                | DONE        |
| 012  | Add closure-aware formula health and lifecycle-only repair   | P0       | L      | 010, 011           | DONE        |
| 013  | Make cask activation owned, transactional, and recoverable   | P0       | L      | 009                | IN PROGRESS |
| 014  | Execute replayable predecessor teardown on upgrade/reinstall | P0       | L      | 013                | IN PROGRESS |
| 015  | Implement mixed cask artifacts and platform-correct config   | P0       | M      | 013, 014           | IN PROGRESS |
| 016  | Coordinate cask mutations with Homebrew locks                | P1       | M      | 013                | IN PROGRESS |
| 017  | Build one semantically integrated stack                      | P0       | M      | 012, 014, 015, 016 | IN PROGRESS |
| 018  | Prove operational parity with real differential oracles      | P0       | L      | 017                | TODO        |
| 019  | Reconcile claims, review state, and final merge gate         | P0       | S      | 018                | TODO        |

Independent formula and cask work may proceed in parallel after plan 009.
Plan 017 is the hard join. Never infer combined correctness from two separate
green heads.

## Finding disposition

| Finding                                                       | Impact                                               | Effort | Risk   | Evidence                                                 | Owner plan |
| ------------------------------------------------------------- | ---------------------------------------------------- | ------ | ------ | -------------------------------------------------------- | ---------- |
| App activation target is backed up but never recreated        | P0: successful app install can end without the app   | S      | High   | `cask.rs:427-504`                                        | 013        |
| Existing app/binary/font targets lack an ownership claim      | Foreign user state can be destroyed                  | M      | High   | `cask.rs:1310-1328,5731-5802`                            | 013        |
| Mise duplicates moved artifacts instead of Homebrew topology  | Brew adoption/prune parity is false                  | L      | High   | `cask.rs:1275-1291,1505-1543,5599-5642`                  | 013        |
| Pending cask journal is never recovered                       | Irreversible actions may replay after crash          | L      | High   | `cask.rs:263-269,402-465,4141-4143`                      | 013        |
| Upgrade skips predecessor uninstall actions                   | Helper/pkg/process residue survives                  | L      | High   | `cask.rs:387-395,5280-5282`                              | 014        |
| Unsupported recorded teardown actions are accepted            | Mise can install state it cannot later remove        | L      | High   | `cask.rs:313-335,4413-4449,5405-5412`                    | 014        |
| `manpage` is silently non-installing                          | Ghostty mixed artifacts are incomplete               | M      | Medium | `cask.rs:2846-2849,4864-4910,5882`                       | 015        |
| Linux receipt config contains macOS directories               | Real brew removes/reads wrong Linux targets          | S      | Medium | `cask.rs:4539-4567`                                      | 015        |
| Mise lock does not contend with Homebrew CaskLock             | Concurrent brew/mise mutations race                  | M      | Medium | `cask.rs:5302-5311`                                      | 016        |
| Oracle loses `CI` under `env -i` and exits success            | Central parity proof is false-green                  | M      | High   | `e2e/run_test:74-126`; zero-second jobs                  | 009        |
| Generic `CI=true` authorizes destructive cleanup              | A naive harness fix can destroy host state           | M      | High   | mac/Linux oracle guards and cleanup                      | 009        |
| New formula lifecycle test is unwired and skippable           | Essential-mac regression has no canonical proof      | M      | High   | #11915 workflow/test/harness                             | 009, 018   |
| Whole closure is validated before mutation classification     | Current installed formula can block unrelated work   | S      | Medium | #11915 `brew/mod.rs:88-110`                              | 010        |
| Lifecycle raw JSON is validated and reparsed separately       | Unsupported details fail after mutation              | L      | High   | #11915 `lifecycle.rs:32-88,351-647`                      | 010        |
| Generic lifecycle `run` lacks Homebrew confinement            | Metadata command inherits mise authority             | L      | High   | #11915 `lifecycle.rs:451-463`                            | 010        |
| Source install skips lifecycle                                | Source formula can report installed incomplete       | M      | High   | #11915 `source.rs:156-177`                               | 011        |
| Source receipt requires a snapshot never written              | #11910 source builds fail at receipt generation      | S      | Low    | `source.rs:157-166`; `pour.rs:443-456`                   | 011        |
| Archive bottle is treated as a source build                   | False provenance and unnecessary compiler dependency | M      | Medium | `fetch.rs:49-59`; `pour.rs:350-474`                      | 011        |
| Lifecycle damage triggers full repour                         | Repair replaces valid keg and loses provenance       | L      | High   | #11915 `pour.rs:35-40,149-193`                           | 012        |
| Status checks only configured root, not dependency closure    | Root-only Kimi config can remain falsely healthy     | L      | Medium | `resources.rs:270-298`; `brew/mod.rs:240-255`            | 012        |
| Two heads conflict in formula state-machine files             | No mergeable or tested product exists                | M      | High   | merge-tree conflicts in `api.rs`, `pour.rs`, `source.rs` | 017        |
| Corpus uses one label per cask and six classes are unverified | 37 names do not prove 37 behaviors                   | M      | Medium | `007-corpus-results.md`                                  | 018        |
| Docs and PR bodies claim full parity                          | Users receive guarantees code does not meet          | S      | Low    | brew docs and plans 007–008                              | 019        |

## Essential-mac acceptance thread

Treat this as a confirmed production regression, not a speculative fixture.
`essential-mac` asks only for `brew:kimi-code`; Node, OpenSSL, and CA
certificates are transitive dependencies. Mise-created receipts and missing
`/opt/homebrew/etc/openssl@3` prove the dependency lifecycle was incomplete.

The corrective chain is:

1. Plan 010 makes lifecycle input fully preflighted and safely executable.
2. Plan 011 gives bottles and source builds the same ordered finalization.
3. Plan 012 makes root status closure-aware and adds non-destructive,
   provenance-backed lifecycle repair.
4. Plan 018 runs the exact root-only Kimi chain at canonical `/opt/homebrew`
   and requires default OpenSSL/Node trust with no CA override.

Declaring `ca-certificates`, `openssl@3`, or `node` explicitly in that
regression is forbidden: it would hide the production root-closure defect.

## Standing constraints

- Production mise never invokes the `brew` CLI. Real brew is an oracle only.
- Receipt fields and lifecycle provenance must be truthful. Unknown or
  unsupported semantics fail before mutation.
- `status` is strictly read-only. It may return exact `NeedsRepair` reasons.
- `apply` repairs only state whose ownership and source are proven. Otherwise
  it reports an exact reinstall requirement.
- Never overwrite user-modified persistent configuration.
- Version strings are opaque; delegate resolution to the backend.
- No new public CLI/config surface.
- No destructive test may run on ambiguous or non-disposable host state.
- No Clippy exclusions.

## Drift protocol

Every executor first compares relevant files with the audited heads:

```bash
rtk git diff --stat 05ccd7ab8..HEAD -- src/system/packages/brew e2e .github/workflows/test.yml docs/bootstrap/packages/brew.md
rtk git -C /private/tmp/mise-formula-lifecycle.QATPOu diff --stat b94b6b1c1..HEAD -- src/system/packages/brew e2e .github/workflows/test.yml
```

If paths moved, locate the current symbols with `rg`; preserve the invariant,
not stale line numbers. If behavior changed materially, update the affected
plan before implementation and record the new exact SHA.

## Findings considered but not promoted

- Production delegation to the `brew` CLI: rejected by the controlling
  design. It would replace native ownership rather than complete it.
- Internal-tap API entries without `tap_string`: current audited corpus has no
  reproducer. Keep fail-closed parsing and add a fixture only if authoritative
  metadata demonstrates the shape.
- Escaped `\\:` in internal API symbols: no current corpus example. Do not add
  speculative decoding; capture a pinned payload first.
- Alias metadata concern: pre-existing and not shown to cause either reported
  production failure. Keep outside this corrective stack unless an oracle
  produces a mismatch.
