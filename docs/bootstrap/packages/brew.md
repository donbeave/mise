# brew

Homebrew formulae and casks — **without requiring Homebrew to be installed**.

```toml
[bootstrap.packages]
"brew:postgresql@17" = "latest"
"brew:ffmpeg" = "latest"
"brew:imagemagick" = "latest"
"brew-cask:firefox" = "latest"
```

mise installs [homebrew/core](https://formulae.brew.sh) formulae directly
into the canonical Homebrew prefix — `/opt/homebrew` on arm64 macOS,
`/home/linuxbrew/.linuxbrew` on Linux. It fetches metadata from the
formulae.brew.sh API, resolves the runtime dependency closure, downloads
prebuilt bottles from ghcr.io (verifying sha256 checksums), and performs the
Homebrew-compatible relocation, code-signing, linking, shared-state, and typed
post-install work described below. Unknown lifecycle operations fail before
the formula is changed. Formulae without a usable bottle are built from source,
also without Homebrew (see [Source formulae](#source-formulae)). mise never
shells out to `brew` for homebrew/core formulae.

Third-party taps are supported directly when the tap publishes Homebrew API
metadata (`api/formula/<name>.json` or `api/cask/<token>.json`). Use the same
fully-qualified name you would pass to Homebrew:

```toml
[bootstrap.packages]
"brew:railwaycat/emacsmacport/emacs-mac" = "latest"
"brew-cask:owner/tap/app" = "latest"
```

For taps whose GitHub URL cannot be inferred, add a tap source. This mirrors
`[plugins]`: the key is the tap name and the value is the GitHub git URL.

```toml
[bootstrap.brew.taps]
"acme/tools" = "https://github.com/acme/homebrew-tools.git"

[bootstrap.packages]
"brew:acme/tools/widget" = "latest"
"brew-cask:acme/tools/widget-app" = "latest"
```

`mise bootstrap packages brew tap` and `mise bootstrap packages brew untap`
manage `[bootstrap.brew.taps]` in `mise.toml`; they do not mutate a Homebrew
installation. Non-GitHub taps are not currently supported because mise needs
direct raw access to the generated API metadata.

```sh
mise bootstrap packages brew tap railwaycat/emacsmacport
mise bootstrap packages brew tap acme/tools https://github.com/acme/homebrew-tools.git
mise bootstrap packages brew untap acme/tools
```

## Casks

Casks use the `brew-cask:` manager. mise fetches cask metadata directly from
the Homebrew cask API (or from tap API metadata), downloads the artifact,
verifies its sha256 when the cask provides one, extracts the archive, and
installs app bundles into `/Applications` while recording the version under
`<prefix>/Caskroom`.

```toml
[bootstrap.packages]
"brew-cask:firefox" = "latest"
"brew-cask:homebrew/cask/visual-studio-code" = "latest"
```

On Linux, initial cask support is limited to font-only casks without lifecycle
hooks or structured `preflight_steps` or `postflight_steps` — concepts from
Homebrew's cask DSL, documented in the
[Homebrew Cask Cookbook](https://docs.brew.sh/Cask-Cookbook). Fonts are
installed into `$XDG_DATA_HOME/fonts`, which defaults to `~/.local/share/fonts`:

```toml
[bootstrap.packages]
"brew-cask:font-heavy-data-nerd-font" = "latest"
```

Other Linux casks are reported as unavailable and skipped when they come from
`[bootstrap.packages]`, allowing macOS and Linux to share a package list. An
explicit request such as `mise bootstrap packages apply brew-cask:firefox`
still fails with a clear unsupported-platform error. You can also mark macOS
casks explicitly with `{ os = "macos" }`. This boundary will expand as mise
gains portable implementations for more cask artifact types.

`brew-cask` currently supports app-bundle casks (`app` artifacts), binary and
generated command-wrapper casks (`binary` and `command_wrapper` artifacts),
fonts, manpages, and shell completions
(`bash_completion`, `fish_completion`, `zsh_completion`, and
`generate_completions_from_executable`) from dmg and common archive formats.
Binary artifacts and generated wrappers are staged in the Caskroom and linked
into the Homebrew prefix, usually under `<prefix>/bin`. Mixed receipts may
contain several supported artifact mechanisms; for example, an app can also
install manpages and completions. Every old teardown and new activation target
is ownership-checked and journaled before metadata is committed.

Package installers (`pkg`), `pkgutil`/BOM teardown, structured lifecycle `run`
steps, services, and unknown lifecycle or artifact types currently fail before
mutation. Other typed preflight/postflight and uninstall operations are accepted
only when the complete recorded plan can be represented and preflighted. mise
does not delegate unsupported behavior to Homebrew and does not infer teardown
from the current catalog in place of the installed receipt.

Cask installs write Homebrew-shaped Caskroom metadata, receipts, configuration,
and installed-cask snapshots for the supported artifact subset. A healthy cask
installed by Homebrew satisfies the same mise declaration when its recorded
artifacts and teardown vocabulary are supported; otherwise mise reports the
unsupported state without mutation.

Legacy casks installed by older mise versions are validated by read-only
status, then converted by apply when their recorded version, package receipts,
and payload fingerprints prove the installed state. When that history cannot
be proven, status reports
`brew-cask:<token>: legacy mise install cannot be converted (<reason>); reinstall with either 'brew install --cask <token>' or mise apply after uninstalling`.
It never invents missing ownership facts.

This exists because shared-library packages — postgres, ffmpeg, imagemagick,
php — fundamentally can't be served by mise's per-project backends like
`aqua:` or `github:`: their bottles are built against fixed install paths and
a shared dependency tree. Installing them at Homebrew's canonical prefix is
what makes them work.

## Supported platforms

| Platform                    | Prefix                       |
| --------------------------- | ---------------------------- |
| macOS arm64 (Apple Silicon) | `/opt/homebrew`              |
| Linux x86_64                | `/home/linuxbrew/.linuxbrew` |
| Linux arm64                 | `/home/linuxbrew/.linuxbrew` |

Intel macs are not supported — the `brew` manager reports itself unavailable
there. On Linux, formulae without a bottle for your architecture (arm64
Linux bottles exist for most but not all of homebrew/core) are built from
source instead.

## The prefix

If the prefix doesn't exist, mise creates it with the standard layout — the
only time the brew manager uses sudo, mirroring what Homebrew's own installer
does (`mkdir` + `chown` to your user). After that, installs are plain file
operations as your user; nothing runs as root.

Only the canonical prefixes above are supported. `MISE_SYSTEM_BREW_PREFIX`
exists solely for isolated tests and is not a custom-prefix feature.

## Coexistence with a real Homebrew

Homebrew does not need to be installed for either engine to work. When present,
both tools intentionally share the canonical prefix. Interoperability is
limited to the formula lifecycle operations and cask artifact/teardown
mechanisms documented here; it is not a claim that mise implements the full
Homebrew Ruby DSL.

For that supported subset, mise writes native receipts, SBOMs, formula
snapshots, links, shared lifecycle state, and cask metadata. A package already
installed by Homebrew is a no-op only when read-only status proves its complete
operational topology healthy. Unknown provenance, lifecycle, ownership, or
teardown behavior fails closed.

For non-keg-only formulae, mise maintains Homebrew's
`<prefix>/var/homebrew/linked/<name>` record alongside the `opt` record. For a
configured formula, if either record is missing, `mise bootstrap packages
apply` restores it without repouring the keg or replacing its public links.
Older mise installs are recognised as linked only when their existing public
links match the keg's layout. Status traverses installed receipt
`runtime_dependencies` offline, so a configured root reports damaged transitive
state. Apply repairs only lifecycle effects whose ownership and inputs are
provable; ambiguous or interrupted state requires reinstall instead of replay.

Bottle defaults under `.bottle/etc` and `.bottle/var` are installed with
persistent-file semantics. Existing user-modified files are preserved during
upgrade and repair. Typed post-install state is recorded by phase so missing
idempotent effects can be repaired without replacing a healthy keg; an action
with unknown prior outcome is never blindly replayed.

mise reads the Homebrew prefix directly, whether formulae were poured by mise
or by a real Homebrew. It never overwrites files in the prefix that it didn't
create — link conflicts fail with a list of the offending files rather than
clobbering them.

## Importing and pruning

`mise bootstrap packages import --manager brew` snapshots installed Homebrew
formulae into `[bootstrap.packages]`, similar in spirit to
[`brew bundle dump`](https://docs.brew.sh/Brew-Bundle-and-Brewfile). It reads
the active `opt` links in the Homebrew prefix and writes entries like:

```toml
[bootstrap.packages]
"brew:ffmpeg" = "latest"
"brew:postgresql@17" = "latest"
```

By default, import records only formulae whose active keg receipt says they
were installed on request. Pass `--all` to include dependency formulae too.
Tapped formulae are written with fully-qualified names, and mise adds inferred
`[bootstrap.brew.taps]` entries when it can derive the conventional GitHub tap
URL:

```toml
[bootstrap.brew.taps]
"acme/tools" = "https://github.com/acme/homebrew-tools.git"

[bootstrap.packages]
"brew:acme/tools/widget" = "latest"
```

`mise bootstrap packages prune --manager brew` treats the current config and
trusted, loadable tracked configs as the source of truth. It removes linked
Homebrew formulae that are not in the resolved dependency closure of those
configured `brew:` entries, including formulae installed by a real Homebrew.

Prune removes the active keg, its `opt` and linked-keg records, and prefix
symlinks pointing into that keg. Use `--dry-run` to preview and `--yes` to skip
the confirmation prompt.

This command is mise's declarative cleanup for bootstrap packages, similar to
[`brew bundle cleanup`](https://docs.brew.sh/Manpage). It is not upstream
`brew prune`, which Homebrew removed in favor of cleanup commands.

`mise bootstrap packages prune --manager brew-cask` applies the same merged
config model to casks installed by either engine. It consumes the installed
version's recorded artifacts, removes the same payload and Caskroom metadata
as `brew uninstall`, and refuses unreadable or unclassified state before any
mutation. `--dry-run` previews the plan and `--yes` skips confirmation. Plain
prune never executes `zap` directives.

## How pouring works

For each formula in the dependency closure (dependencies first):

1. **Fetch** the bottle for your platform from ghcr.io and verify its sha256
   against the API metadata.
2. **Extract** into a temporary directory inside the Cellar (incomplete
   pours are never visible as installed packages).
3. **Relocate**: bottles embed placeholder paths like `@@HOMEBREW_PREFIX@@`.
   mise rewrites them to real paths — plain replacement in text files and in
   the shebang preamble of binary-backed executables such as zipapps (leaving
   their payload untouched), and in-place and load-command rewriting in Mach-O
   binaries (growing load commands into header padding when needed, exactly
   like brew's ruby-macho does). On Linux, the ELF
   interpreter and rpath are patched the way brew's PatchELF gem does it:
   strings that no longer fit are moved into a new segment appended to the
   binary, and the interpreter is pointed at `<prefix>/lib/ld.so` (a symlink
   mise maintains to the system's dynamic loader, or to a brewed glibc when
   one is installed).
4. **Re-sign** (macOS): any modified binary is ad-hoc re-signed with
   `codesign` — required on arm64, where the kernel kills binaries whose
   signature doesn't match.
5. **Receipt**: a brew-compatible `INSTALL_RECEIPT.json` is written.
6. **Link**: `<prefix>/opt/<name>` is created and the keg's `bin`, `lib`,
   `include`, `share`, etc. are symlinked into the prefix. The Homebrew
   linked-keg record is created for non-keg-only formulae.
   [keg-only](https://docs.brew.sh/FAQ#what-does-keg-only-mean) formulae get
   the `opt` link but are not linked into the prefix, same as brew.
7. **Shared state**: `.bottle/etc` and `.bottle/var` defaults are installed
   without overwriting user changes.
8. **Post-install**: the preflighted typed lifecycle plan runs in a restricted,
   deterministic environment. mise records completion only after health checks
   pass.

## Source formulae

A few formulae have no bottle at all (source-only formulae), and some have
bottles for other platforms but not yours. mise builds those from source —
still without Homebrew:

1. **Ruby** — a formula is Ruby code, so mise provisions a mise-managed
   ruby through its normal tool machinery (precompiled, fast; respects your
   configured ruby if you have one).
2. **Formula** — the formula's `.rb` is downloaded from homebrew/core,
   pinned to the exact commit the API metadata was generated from and
   verified against the API's sha256 for it.
3. **Source** — the stable source archive is downloaded and verified
   against the API's sha256.
4. **Build deps** — the formula's build dependencies (cmake, pkgconf, ...)
   are added to the install closure and poured as regular bottles first.
5. **Build** — mise evaluates the formula with its own Formula-DSL shim and
   runs `def install` against the canonical prefix, with `PATH`,
   `PKG_CONFIG_PATH`, and compiler flags pointing at the dependency kegs.
   The keg gets the same brew-compatible receipt as a poured bottle, with
   `poured_from_bottle: false` — exactly how brew marks its own source
   builds. The exact formula source is stored under `.brew`, then the source
   keg enters the same link, shared-state, typed post-install, and health
   finalizer as a bottle.

The shim implements the commonly-used subset of the formula DSL
(configure/cmake/meson-style builds, resources, patches, the standard path
and environment helpers). Formulae that use parts of the DSL the shim
doesn't cover — language-specific helpers like `virtualenv_install_with_resources`,
VCS downloads, and similar — fail with a clear `formula uses ...` error
rather than miscompiling silently.

Source builds need a working toolchain (Xcode Command Line Tools on macOS,
gcc/make on Linux), exactly as they would under plain Homebrew.

## Upgrades

`mise bootstrap packages upgrade` re-resolves the configured formulae against the
formulae.brew.sh API and pours any whose current version differs from the
linked keg — the new keg replaces the old one and the links are repointed,
the same dance `brew upgrade` does. Since bottles only exist for a formula's
current version, "upgrade" and "install the current bottle" are the same
operation.

Apply ensures presence; it does not implicitly upgrade an already installed
package. Upgrade is explicit and follows Homebrew semantics. In particular,
`auto_updates` casks are skipped rather than receiving brew's greedy upgrade
behavior.

## Limitations

- **Cask artifact coverage is intentionally narrow.** On macOS, `brew-cask`
  supports app bundles, binary artifacts, command wrappers, fonts, manpages,
  and declared or generated shell completions from dmg and common archive
  formats. Pkg/BOM teardown, structured lifecycle `run`, services, and unknown
  artifact or teardown mechanisms fail before mutation. On Linux, it supports
  font-only casks without lifecycle hooks or structured `preflight_steps` or
  `postflight_steps`; targets use XDG/Linux Homebrew paths, never macOS cask
  paths.
- **`brew services` is not implemented.**
- **Cask import is not implemented.** Cask prune reads the installed receipt and
  refuses unknown uninstall directives; it never substitutes today's catalog
  definition for historical lifecycle facts.
- **Source builds cover the common formula shapes.** mise's formula shim
  implements the widely-used subset of the DSL (see
  [Source formulae](#source-formulae)); formulae that reach beyond it fail
  with a clear error naming the unsupported feature.
- **Formula post-install coverage is typed, not arbitrary Ruby.** Supported
  plans use preflighted `mkdir_p`, `remove`, `copy`, `symlink`, and confined
  `run` operations plus `.bottle/etc` and `.bottle/var` installation. Opaque
  Ruby and unknown step/path/guard forms fail before the mutation set changes.
- **Use canonical formula names.** `postgresql@17` is a formula name, not a
  mise version pin — the API's current stable version decides what gets
  installed. Aliases (`postgres`) install correctly but `mise bootstrap packages status`
  can't track them; mise warns and tells you the canonical name.
- `PATH` is up to you: `<prefix>/bin` must be on `PATH` to use linked
  binaries, just like with Homebrew itself.
