#!/usr/bin/env bash

# Differential Homebrew state oracle. Values normalized here are inherently
# install-time/machine-specific; key presence remains mandatory so structural
# divergence cannot be hidden.

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
      .built_on = "<NORMALIZED>" |
      .source.path = "<NORMALIZED>" |
      .source.tap_git_head = "<NORMALIZED>" |
      if has("source_modified_time") then .source_modified_time = "<NORMALIZED>" else . end
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
  local root=$1 output=$2
  local scratch
  scratch=$(mktemp -d)
  : >"$output"
  while IFS= read -r path; do
    local relative normalized mode digest target
    relative=${path#"$root"/}
    relative=$(printf '%s' "$relative" | sed -E 's#/[0-9]{14}\.[0-9]{3}/#/<TIMESTAMP>/#g')
    mode=$(brew_oracle_mode "$path")
    if [[ -L $path ]]; then
      target=$(readlink "$path")
      printf 'l %s %s -> %s\n' "$mode" "$relative" "$target" >>"$output"
    elif [[ -d $path ]]; then
      printf 'd %s %s\n' "$mode" "$relative" >>"$output"
    elif [[ -f $path ]]; then
      if [[ $path == *.json ]]; then
        normalized="$scratch/normalized.json"
        brew_oracle_normalize_json "$path" "$normalized"
        digest=$(shasum -a 256 "$normalized" | awk '{print $1}')
      else
        digest=$(shasum -a 256 "$path" | awk '{print $1}')
      fi
      printf 'f %s %s %s\n' "$mode" "$relative" "$digest" >>"$output"
    fi
  done < <(find "$root" -mindepth 1 -print | LC_ALL=C sort)
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
