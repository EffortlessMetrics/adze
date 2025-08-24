#!/usr/bin/env bash
#
# Print a unique, newline-separated list of Cargo package names affected by the
# currently staged files. We consider .rs, build.rs and Cargo.toml changes.
# If none of those are staged, exit quietly.
set -euo pipefail

# Gather staged candidates (zero-delimited for safety)
changed=()
while IFS= read -r -d '' file; do
  changed+=("$file")
done < <(git diff --cached --name-only -z --diff-filter=ACMR \
  -- '*.rs' 'build.rs' '*/build.rs' 'Cargo.toml' '*/Cargo.toml' 2>/dev/null || true)
if [ "${#changed[@]}" -eq 0 ]; then exit 0; fi

# cargo metadata + jq are required for precise mapping.
if ! command -v jq >/dev/null 2>&1; then
  echo "jq not installed; cannot compute affected crates" >&2
  exit 2
fi

# Build a (name, dir) table from cargo metadata, using TAB as delimiter.
rows=()
while IFS= read -r row; do
  rows+=("$row")
done < <(cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[] | [.name, (.manifest_path | sub("/Cargo.toml$"; ""))] | @tsv')

names=() dirs=()
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
# Use space-delimited string instead of associative array for portability
touched_list=""
for file in "${changed[@]}"; do
  # Handle deleted/renamed entries defensively
  [ -e "$file" ] || continue
  abs="$(abspath "$file")"
  best_idx=-1
  best_len=0
  i=0
  for d in "${dirs[@]}"; do
    case "$abs" in
      "$d"/*)
        l="${#d}"
        if [ "$l" -gt "$best_len" ]; then
          best_len="$l"
          best_idx="$i"
        fi
        ;;
    esac
    i=$((i + 1))
  done
  if [ "$best_idx" -ge 0 ]; then
    pkg_name="${names[$best_idx]}"
    # Check if already in list to avoid duplicates
    case " $touched_list " in
      *" $pkg_name "*) ;;
      *) touched_list="$touched_list $pkg_name" ;;
    esac
  fi
done

# Emit unique list, sorted for stable output
if [ -n "$touched_list" ]; then
  echo "$touched_list" | tr ' ' '\n' | grep -v '^$' | sort -u
fi