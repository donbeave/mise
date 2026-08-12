# Plan 007 corpus results — evidence invalidated

Recorded 2026-08-12; invalidated by deep audit 2026-08-13.

## Verdict

The previous PASS claims are withdrawn. `e2e/run_test` launched scripts through
`env -i` without forwarding `CI`; both guarded oracle bodies returned success
before executing fixtures. These zero-second jobs are links to false-green
evidence, not parity proof:

- [macOS job 93970283233](https://github.com/jdx/mise/actions/runs/31549912509/job/93970283233)
- [Linux job 93980178865](https://github.com/jdx/mise/actions/runs/31552852719/job/93980178865)

All 37 cask names were present in the old table. That proves name accounting
only. It does not prove equivalence-class coverage.

## Classification defect

The old table assigned mostly one label per cask. Current casks combine multiple
artifact/lifecycle mechanisms. For example, Ghostty has an app, two manpages,
and three completions; mise currently skips manpages. Hidden Bar therefore
cannot prove Ghostty equivalence. Other corpus members require binary, pkg,
postflight, quit, signal, script, launchctl, privileged helper, auto-update, or
versioned-token behavior in combinations.

Six of the eight old classes were explicitly NOT VERIFIED. The two PASS rows are
now INVALIDATED because their job bodies did not run.

## Replacement evidence

Plan 018 must replace this file with a pinned cask-by-mechanism matrix. Multiple
mechanisms per cask are required. Each supported mechanism and important
combination needs a real disposable differential representative in both safe
ownership directions. Unsafe/interactive or unimplemented behavior must be
recorded UNSUPPORTED and fail closed before mutation.

Required replacement fields:

| Field | Requirement |
| --- | --- |
| Cask/token | All original 37 names |
| Metadata identity | API/tap revision and payload digest |
| Mechanisms | Boolean/details per artifact and lifecycle action |
| Support | SUPPORTED or UNSUPPORTED with exact reason |
| Representative | One or more tests covering combinations |
| Evidence | Combined mise SHA, Homebrew SHA/version, job URL, completion marker, fixture count |
| Result | Real structural/runtime comparison; no unit-only PASS |

Until that replacement exists, corpus equivalence status is **NOT VERIFIED**.
