# Current mise/Homebrew evidence

Date: 2026-08-22

This document records the live evidence gathered after the four source audits.
It is intentionally separate from the archived reports because several current
facts simplify their proposed replacement sequence.

## Required user outcome

From this repository, both a fresh Mac and an existing Homebrew-managed Mac
must be able to run:

```bash
mise bootstrap --yes
```

Repeated runs must converge without:

- rejecting a healthy Homebrew-owned cask such as `1password-cli`;
- transferring, rewriting, upgrading, pruning, or uninstalling foreign cask
  ownership;
- reporting an incomplete `ca-certificates`/`openssl@3` installation as
  healthy;
- requiring TLS environment overrides to make poured OpenSSL or its consumers
  work;
- depending on either existing oversized draft PR.

## Live essential-mac reproduction

The base profile declares `brew-cask:1password-cli` and 21 other casks in
`mise.toml`. Installed mise is `2026.8.10 macos-arm64`.

`mise bootstrap packages status --json` reports `1password-cli` as missing even
though all of the following native state exists:

- `/opt/homebrew/Caskroom/1password-cli/.metadata/INSTALL_RECEIPT.json`;
- version directory `/opt/homebrew/Caskroom/1password-cli/2.39.0`;
- `/opt/homebrew/bin/op` linked into that version directory;
- Homebrew reports `1password-cli 2.39.0` installed.

The receipt identifies `homebrew/cask`, version `2.39.0`, architecture
`arm64`, and native uninstall artifacts for `op` and its generated
completions. Current mise status ignores those facts because it reads only
mise's private `.mise-cask.toml` receipt.

The exact apply failure is therefore structural: current-main cask install
checks for `.metadata` and raises `Homebrew owns this cask` before installed
state is classified.

### Current ownership distribution

The live Caskroom contains native Homebrew metadata for almost every cask in
the base profile, including `1password-cli`, `1password`, `cleanshot`,
`little-snitch`, both JetBrains fonts, `bartender`, `sublime-text`, `sketch`,
`google-chrome`, `notion`, `grammarly-desktop`, `speechify-voice-ai`,
`superwhisper`, `vlc`, `yaak`, `cloudflare-warp`, `tableplus`, `zoom`,
`orbstack`, and `surge`.

`ghostty` is mise-owned and has `.mise-cask.toml`. Consequently, a temporary
downstream bridge cannot blindly run Homebrew for every cask or blindly leave
every cask in mise. It must classify and preserve existing ownership.

Moving only `1password-cli` is also insufficient: it would merely expose the
next Homebrew-owned declaration as the next hard failure.

## Current-main cask boundary

Current upstream main already contains most surrounding behavior needed by a
small successor:

- cask adoption options and receipt handling are merged;
- platform-unavailable state is classified inside the cask manager before
  installed-state detection;
- prune already skips any token directory containing native `.metadata`;
- status is required to be side-effect-free;
- current mise-owned install, upgrade, and prune behavior already has focused
  tests.

The successor therefore needs no formula code, generic `PackageState` change,
driver manager-name special case, doctor change, install-hint change, or broad
receipt serializer.

Correct private state model:

```text
Absent
MiseOwned(version)
HomebrewOwned(version)
Conflict(reason)
```

Producer origin must remain attached to installed state. Erasing it to a bare
version would allow ordinary upgrade or prune to mutate native Homebrew state.

## Formula/OpenSSL reproduction and root cause

Current mise already resolves and pours the complete formula dependency
closure and writes a Homebrew-compatible `INSTALL_RECEIPT.json`. Those are not
new capabilities required by the fix.

Current formula health is effectively:

```text
keg directory exists AND opt record points to that keg
```

It excludes `.bottle/etc`, `.bottle/var`, post-install operations, and their
shared-state postconditions. A formula can therefore look installed while its
runtime is incomplete.

Live required state demonstrates the missing lifecycle:

- `ca-certificates` contains `libexec/post-install`, which builds
  `/opt/homebrew/etc/ca-certificates/cert.pem` from system and Mozilla trust;
- `openssl@3` requires `/opt/homebrew/etc/openssl@3/openssl.cnf` from bottle
  shared defaults;
- `openssl@3` post-install links
  `/opt/homebrew/etc/openssl@3/cert.pem` to the CA bundle;
- the OpenSSL binary's compiled `OPENSSLDIR` is
  `/opt/homebrew/etc/openssl@3`.

The architectural invariant must become:

```text
healthy = verified artifact identity
       + healthy dependency closure
       + supported lifecycle effects
       + native receipt
       + expected links/topology
```

A missing lifecycle effect is repairable state, not absence and not a reason
to repour a healthy keg.

## Public API evidence that removes old scope

Current mise main already fetches formula metadata from
`https://formulae.brew.sh/api`. The current public `brew info --json=v2`
records expose structured `post_install_steps`.

The essential-mac base formula closure currently uses these operation shapes:

### `ca-certificates`

```json
{
  "type": "run",
  "command": { "base": "libexec", "path": "post-install" },
  "args": ["{{pkgshare}}/cacert.pem", "{{pkgetc}}/cert.pem"]
}
```

### `openssl@3`

```json
{
  "type": "symlink",
  "source": { "path": "{{etc}}/ca-certificates/cert.pem" },
  "target": { "path": "{{pkgetc}}/cert.pem" },
  "force": true
}
```

### Node

Node uses `mkdir_p`, guarded recursive `remove`, recursive `copy`, and forced
`symlink` operations, including declared source globs. This matters because
`brew:agent-browser` depends on Homebrew Node, which in turn depends on
`openssl@3` and `ca-certificates`.

The formula fix must therefore support the bounded structured operation set
used by the whole essential-mac closure. Supporting only an OpenSSL symlink or
one pinned `ca-certificates` revision would preserve the same bug class for
Node or the next formula revision.

### Trust conclusion

No new internal Homebrew JWS client is needed for this successor. Current mise
already trusts the public formula API for bottle URL and SHA-256. Lifecycle
metadata from the same record can be accepted within that existing boundary
when:

- the selected bottle is checksum verified;
- executable helpers must reside inside that verified bottle;
- every path and operation is compiled and bounded before mutation;
- unknown operation types fail before mutation;
- opaque Ruby `post_install` remains unsupported and fails before mutation.

This removes the old branch's internal signed-index parser, permanent cache,
OCI identity retry, trust-anchor rotation, and strict whole-record schema from
the required scope. It also removes their associated rolling-update outage and
stale-`Fresh` defects.

## Maintainer and PR evidence

PR #11910 is still an open draft at head `a1df6b7`; PR #11915 is still an open
draft at head `97640cc`. Both are mechanically mergeable and green, but neither
has maintainer approval.

jdx's decisive comment on #11910:

> I don't understand what this is for but a 40k line pull request is way
> outside the realm of something I'd be willing to review

This rejects both comprehension and review surface. More explanation or more
commits on the same branches cannot address it.

The formula-first replacement order in the archived plan was needed only to
hide #11915 from #11910's old stacked GitHub diff. A new cask branch from
current main has no technical or presentation dependency on formula lifecycle.

Current mise contribution guidance requires direction agreement for non-obvious
work, green CI, and addressed automated review before maintainer review. The
formula metadata boundary should therefore be agreed first. The cask fix is
small and obvious enough to prepare immediately from current main.

## CI conclusion

The old 13-cell brew matrix and multiple exact-head build pipelines are not
part of either behavior fix. Each replacement should use normal project gates,
focused unit tests, and one test added to an existing macOS e2e path. Large
Homebrew differential corpora belong in scheduled/manual validation or a later
CI-only proposal.
