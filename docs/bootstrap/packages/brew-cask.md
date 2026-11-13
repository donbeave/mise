# brew-cask

mise-owned casks live under `<prefix>/Caskroom` with a `.mise-cask.toml`
receipt and symlinked binaries/completions (for example
`/opt/homebrew/bin/codex`). They stay mise-owned by default. See the
full cask reference in [brew](/bootstrap/packages/brew.html#casks).

## Troubleshooting

### Hand a mise-owned cask to Homebrew (`mise` -> `brew`)

When a `brew-cask:` entry is removed from config but its versioned Caskroom
directory remains (`.mise-cask.toml` present, no Homebrew `.metadata`),
a later `brew install --cask <token>` fails with
`already a Binary at <prefix>/bin/<name>` while `brew` still reports
`Not installed` and `brew doctor` reports no valid metadata.

Hand the token to Homebrew without deleting metadata mise does not own,
without overwriting links, and without running `brew` from mise:

```sh
mise bootstrap packages prune --manager brew-cask --dry-run
mise bootstrap packages prune --manager brew-cask
brew install --cask <token>
```

`prune --dry-run` lists the stale mise receipt as a candidate
(`.mise-cask.toml` yes, `.metadata` no, undeclared). `prune` removes only
that receipt's recorded links and version directory. `status` never rewrites
links: bin/completion conflicts are presence-only `readlink` checks under
`lock_caskroom()`. Homebrew-owned, interrupted, or shared targets stay
`forceable:false` and are skipped.
