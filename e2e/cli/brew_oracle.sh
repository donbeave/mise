#!/usr/bin/env bash

# Differential Homebrew state oracle. Normalize only producer identity,
# wall-clock values, and producer-local source paths. Immutable package facts
# (including built_on, tap_git_head, and source_modified_time) remain exact.

brew_oracle_mode() {
  if [[ $(uname) == Darwin ]]; then
    stat -f '%Lp' "$1"
  else
    stat -c '%a' "$1"
  fi
}

brew_oracle_normalize_json() {
  local input=$1 output=$2
  if [[ $input == */INSTALL_RECEIPT.json ]]; then
    jq -e '
      has("homebrew_version") and has("time") and has("built_on") and
      (.source | type == "object" and has("path") and has("tap_git_head")) and
      ((has("used_options") | not) or has("source_modified_time"))
    ' "$input" >/dev/null
    jq -S '
      .homebrew_version = "<NORMALIZED>" |
      .time = "<NORMALIZED>" |
      .source.path = "<NORMALIZED>"
    ' "$input" >"$output"
  elif [[ $input == */sbom.spdx.json ]]; then
    jq -e '
      .creationInfo |
      type == "object" and has("created") and
      (.creators | type == "array" and length == 1 and
        (.[0] | type == "string" and
          startswith("Tool: https://github.com/Homebrew/brew@")))
    ' "$input" >/dev/null
    jq -S '
      .creationInfo.created = "<NORMALIZED>" |
      .creationInfo.creators[0] = "<NORMALIZED>"
    ' "$input" >"$output"
  else
    jq -S . "$input" >"$output"
  fi
}

brew_oracle_normalize_path() {
  # Homebrew assigns cask metadata a wall-clock installation directory.
  sed -E 's#(/\.metadata/[^/]+)/[0-9]{14}\.[0-9]{3}(/|$)#\1/<TIMESTAMP>\2#'
}

brew_oracle_snapshot() {
  local output=$1
  shift
  local scratch entries
  scratch=$(mktemp -d)
  entries="$scratch/entries"
  : >"$entries"
  local spec label root
  for spec in "$@"; do
    label=${spec%%=*}
    root=${spec#*=}
    [[ $label != "$spec" && -e $root ]] || return 1
    while IFS= read -r path; do
      local relative normalized mode digest target
      if [[ $path == "$root" ]]; then
        relative=.
      else
        relative=${path#"$root"/}
      fi
      relative=$(printf '%s/%s' "$label" "$relative" | brew_oracle_normalize_path)
      mode=$(brew_oracle_mode "$path")
      if [[ -L $path ]]; then
        target=$(readlink "$path")
        printf 'l %s %s -> %s\n' "$mode" "$relative" "$target" >>"$entries"
      elif [[ -d $path ]]; then
        printf 'd %s %s\n' "$mode" "$relative" >>"$entries"
      elif [[ -f $path ]]; then
        if [[ $path == *.json ]]; then
          normalized="$scratch/normalized.json"
          brew_oracle_normalize_json "$path" "$normalized"
          digest=$(shasum -a 256 "$normalized" | awk '{print $1}')
        else
          digest=$(shasum -a 256 "$path" | awk '{print $1}')
        fi
        printf 'f %s %s %s\n' "$mode" "$relative" "$digest" >>"$entries"
      fi
    done < <(find "$root" -print | LC_ALL=C sort)
  done
  LC_ALL=C sort "$entries" >"$output"
  rm -rf "$scratch"
}

brew_oracle_diff() {
  local brew_snapshot=$1 mise_snapshot=$2
  if ! diff -u "$brew_snapshot" "$mise_snapshot"; then
    printf 'first divergent snapshot line:\n' >&2
    diff -u "$brew_snapshot" "$mise_snapshot" | sed -n '/^[+-][^+-]/p' | head -1 >&2
    return 1
  fi
}

brew_oracle_record_api_fixture() {
  local kind=$1 token=$2 expected_version=$3 expected_sha=$4
  local result_dir=${MISE_BREW_ORACLE_RESULT_DIR:-}
  local version_filter url safe_token raw canonical actual_version actual_sha

  [[ $kind == formula || $kind == cask ]] || return 1
  [[ $token =~ ^[a-z0-9@+._-]+$ ]] || return 1
  [[ $expected_sha =~ ^[0-9a-f]{64}$ ]] || return 1
  [[ -d $result_dir && ! -L $result_dir ]] || return 1
  safe_token=${token//@/_at_}
  raw="$result_dir/api-$kind-$safe_token.raw.json"
  canonical="$result_dir/api-$kind-$safe_token.json"
  [[ ! -e $raw && ! -L $raw && ! -e $canonical && ! -L $canonical ]] || return 1
  url="https://formulae.brew.sh/api/$kind/$token.json"
  curl --retry 3 --retry-delay 1 --max-time 60 -fsSL "$url" -o "$raw"
  if [[ $kind == formula ]]; then
    version_filter=.versions.stable
  else
    version_filter=.version
  fi
  actual_version=$(jq -er "$version_filter" "$raw")
  [[ $actual_version == "$expected_version" ]] || {
    echo "brew oracle fixture drift: $kind:$token version $actual_version != $expected_version" >&2
    return 1
  }
  # `tap_git_head` changes for every unrelated tap commit. Pin the payload's
  # per-definition ruby_source_checksum and all operational fields instead.
  jq -S 'del(.analytics, .generated_date, .tap_git_head)' "$raw" >"$canonical"
  actual_sha=$(shasum -a 256 "$canonical" | awk '{print $1}')
  [[ $actual_sha == "$expected_sha" ]] || {
    echo "brew oracle fixture drift: $kind:$token digest $actual_sha != $expected_sha" >&2
    return 1
  }
  printf '%s:%s version=%s sha256=%s\n' \
    "$kind" "$token" "$actual_version" "$actual_sha" \
    >>"$result_dir/api-fixtures.txt"
}
