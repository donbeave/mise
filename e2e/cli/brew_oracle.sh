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
      .creationInfo | type == "object" and has("created") and has("creators")
    ' "$input" >/dev/null
    jq -S '
      .creationInfo.created = "<NORMALIZED>" |
      .creationInfo.creators = ["<NORMALIZED>"]
    ' "$input" >"$output"
  else
    jq -S . "$input" >"$output"
  fi
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
      relative=$(printf '%s/%s' "$label" "$relative" | sed -E 's#/[0-9]{14}\.[0-9]{3}/#/<TIMESTAMP>/#g')
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
