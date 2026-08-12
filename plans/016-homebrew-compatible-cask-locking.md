# Plan 016: Coordinate cask mutations with Homebrew-compatible locks

Status: TODO
Priority: P1
Effort: M
Planned against: #11910 `05ccd7ab8`
Depends on: 013

## Objective

Make mise and real Homebrew contend on the same per-token cask lock before
mutating shared Caskroom, metadata, or public targets. Define deterministic lock
ordering and prove multi-process exclusion.

## Current defect

Mise locks only `Caskroom/.mise.lock`. Homebrew 6.0.17 `CaskLock` locks the token
through its `HOMEBREW_LOCKS` protocol. The engines can concurrently install,
upgrade, or remove the same cask despite claiming shared ownership.

## Files in scope

- `src/system/packages/brew/cask.rs`
- a small internal Homebrew lock helper if appropriate
- multi-process focused tests

No global package-manager lock redesign.

## Implementation steps

1. Read pinned Homebrew 6.0.17 `cask_lock.rb`, generic lock implementation, and
   constant resolution. Record exact filename, directory, permissions, open and
   flock semantics, blocking policy, stale inode/race handling, and diagnostics.
2. Implement protocol compatibility without invoking `brew`. Use exact token
   canonicalization and prefix-derived lock directory. Never guess an unresolved
   `HOMEBREW_LOCKS` path.
3. Acquire the per-token Homebrew lock before reading mutable predecessor state
   for an operation, and hold through plan 013/014 transaction commit/recovery.
4. If retaining `Caskroom/.mise.lock`, define one global order: shared/global
   lock then sorted per-token locks, or remove the global lock if no invariant
   needs it. Every call site follows the same order.
5. Status remains read-only and lock-free unless a consistent snapshot truly
   requires a shared lock; it must never create lock directories.
6. Provide bounded wait/contention diagnostics including token and holder facts
   available without exposing unrelated process environment.

## Required tests

- Two mise processes on the same token cannot overlap mutation phases.
- A test process holding the exact Homebrew-compatible lock blocks/fails a mise
  mutation according to pinned semantics; release permits it.
- Different tokens can proceed concurrently when no global invariant requires
  serialization.
- Multiple-token acquisition in opposite request order cannot deadlock.
- Symlinked/foreign lock directory and permission failures stop before package
  mutation.
- Read-only status creates no lock file/directory.

## Verification

```bash
rtk cargo test --bin mise system::packages::brew::cask
rtk mise run lint
```

Run a disposable integration probe with real `brew` holding the same token lock;
never race real package changes on operator state.

## Done criteria

- Mise and Homebrew demonstrably contend on one per-token lock.
- Lock order is documented and mechanically shared by every mutating path.
- No deadlock or mutation-before-lock remains.

## Stop conditions

Do not infer the lock path from memory. Do not delete a lock file to break
contention. Do not hold the lock across unrelated network download if Homebrew's
protocol does not; match the pinned critical section deliberately.
