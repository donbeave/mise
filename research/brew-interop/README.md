# Homebrew formula/cask interop — donbeave fork record

Living branch: [`feat/brew-native-interop`](https://github.com/donbeave/mise/tree/feat/brew-native-interop)  
Head at persist time: `a1df6b7fd` (includes `jdx/mise#11910` and ancestor `jdx/mise#11915` / `97640cc04`).

## Why this fork exists

On 2026-08-21 jdx declined to review the stacked brew PRs:

> I don't understand what this is for but a 40k line pull request is way outside the realm of something I'd be willing to review

Source: <https://github.com/jdx/mise/pull/11910#issuecomment-5364856639>

On 2026-08-23 both PRs were closed **unmerged**:

| PR | Result |
|---|---|
| [jdx/mise#11910](https://github.com/jdx/mise/pull/11910) | CLOSED, not merged |
| [jdx/mise#11915](https://github.com/jdx/mise/pull/11915) | CLOSED, not merged |
| [jdx/mise#11164](https://github.com/jdx/mise/pull/11164) | MERGED (keep) |
| [jdx/mise#11215](https://github.com/jdx/mise/pull/11215) | MERGED (keep) |

Work continues here, not against `jdx/mise`.

## What the analysis concluded

The oversized PRs are evidence and prototypes, not merge candidates. Two independent user bugs remain on upstream `main`:

1. Formula pour can look healthy while required post-install / shared lifecycle state (OpenSSL/CA) is missing.
2. A healthy Homebrew-owned cask (e.g. `1password-cli`) is rejected as a conflict before installed-state detection.

Replacements, if ever proposed upstream again, must be small current-main successors — not these 40k-line stacks.

## Documents in this tree

| File | Role |
|---|---|
| [BREW-INTEROP-PROBLEM-REFERENCE.md](BREW-INTEROP-PROBLEM-REFERENCE.md) | Canonical problem statement after jdx's refusal |
| [CURRENT-STATE-EVIDENCE.md](CURRENT-STATE-EVIDENCE.md) | Live essential-mac / current-main observations |
| [DELIVERY-PLAN.md](DELIVERY-PLAN.md) | Focused-successor delivery plan (historical; upstream PRs now closed) |
| [ROADMAP.md](ROADMAP.md) | Roadmap item that produced the audits |
| [audits/PR-REVIEW-SUMMARY.md](audits/PR-REVIEW-SUMMARY.md) | Combined review of #11910 / #11915 vs jdx's comment |
| [audits/PR-11910-BREW-CASK-INTEROP-AUDIT.md](audits/PR-11910-BREW-CASK-INTEROP-AUDIT.md) | File-level #11910 audit |
| [audits/PR-11915-FORMULA-LIFECYCLE-AUDIT.md](audits/PR-11915-FORMULA-LIFECYCLE-AUDIT.md) | File-level #11915 audit |
| [audits/BREW-PR-REPLACEMENT-PLAN.md](audits/BREW-PR-REPLACEMENT-PLAN.md) | Earlier replacement plan (superseded ordering) |
| [engine-parity/](engine-parity/) | Current-main `brew:` / `brew-cask:` vs Homebrew correspondence |

Copies also live in [donbeave/essential-mac](https://github.com/donbeave/essential-mac) under `docs/mise/` and `research/mise-homebrew-bootstrap-engine-parity/`.
