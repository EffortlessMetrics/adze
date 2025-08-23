#!/usr/bin/env bash
#
# Print a unique, newline-separated list of Cargo package names affected by the
# currently staged files. We consider .rs, build.rs and Cargo.toml changes.
# If none of those are staged, exit quietly.
set -euo pipefail

# Gather staged candidates (zero-delimited for safety)
mapfile -d '' -t changed < <(git diff --cached --name-only -z --diff-filter=ACMR \
  -- '*.rs' '*/build.rs' '*/Cargo.toml' 'Cargo.toml' 2>/dev/null || true)
if [ "${#changed[@]}" -eq 0 ]; then exit 0; fi

# cargo metadata + jq are required for precise mapping.
if ! command -v jq >/dev/null 2>&1; then
  echo "jq not installed; cannot compute affected crates" >&2
  exit 2
fi

# Build a (name, dir) table from cargo metadata, using TAB as delimiter.
mapfile -t rows < <(cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[] | [.name, (.manifest_path | sub("/Cargo.toml$"; ""))] | @tsv')

declare -a names=() dirs=()
for row in "${rows[@]}"; do
  # Split on TAB - use $'\t' for explicit tab character
  IFS=$'\t' read -r name dir <<< "$row"
  names+=("$name")
  dirs+=("$dir")
done

# Portable absolute path (no realpath dependency)
abspath() {
  local p="$1"
  ( cd "$(dirname "$p")" >/dev/null 2>&1 && printf '%s/%s\n' "$(pwd -P)" "$(basename "$p")" )
}

# For each changed file, pick the longest matching package dir prefix.
declare -A touched=()
for file in "${changed[@]}"; do
  # Handle deleted/renamed entries defensively
  [ -e "$file" ] || continue
  abs="$(abspath "$file")"
  best_idx=-1
  best_len=0
  for i in "${!dirs[@]}"; do
    d="${dirs[$i]}"
    case "$abs" in
      "$d"/*)
        l="${#d}"
        if (( l > best_len )); then best_len="$l"; best_idx="$i"; fi
        ;;
    esac
  done
  if (( best_idx >= 0 )); then touched["${names[$best_idx]}"]=1; fi
done

# Emit unique list, sorted for stable output
for k in "${!touched[@]}"; do echo "$k"; done | sort -u