#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_CRATE_FILE="${RELEASE_CRATE_FILE:-${SCRIPT_DIR}/release-crates.txt}"
RELEASE_SURFACE_MODE="${RELEASE_SURFACE_MODE:-fixed}"
RELEASE_CRATE_SYNC="${RELEASE_CRATE_SYNC:-0}"

usage() {
  cat <<'EOF'
Usage: release-surface.sh [--mode fixed|auto] [--sync]

Modes:
  fixed   Read crates from RELEASE_CRATE_FILE (default: scripts/release-crates.txt)
  auto    Recompute publishable crates from workspace metadata and topo-sort them

Options:
  --sync  In auto mode, write the computed order back to RELEASE_CRATE_FILE.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      if [[ $# -lt 2 ]]; then
        echo "::error::--mode requires an argument (fixed|auto)." >&2
        exit 1
      fi
      RELEASE_SURFACE_MODE="$2"
      shift 2
      ;;
    --sync)
      RELEASE_CRATE_SYNC=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "::error::Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

case "${RELEASE_SURFACE_MODE,,}" in
  fixed)
    RELEASE_SURFACE_MODE="fixed"
    ;;
  auto)
    RELEASE_SURFACE_MODE="auto"
    ;;
  *)
    echo "::error::Invalid RELEASE_SURFACE_MODE '${RELEASE_SURFACE_MODE}'. Expected fixed|auto." >&2
    exit 1
    ;;
esac

emit_fixed_crates() {
  if [[ ! -f "$RELEASE_CRATE_FILE" ]]; then
    echo "::error::Missing allowlist file: ${RELEASE_CRATE_FILE}" >&2
    exit 1
  fi

  mapfile -t crates < <(awk 'NF && $1 !~ /^#/ {print $1}' "$RELEASE_CRATE_FILE")
  if [[ ${#crates[@]} -eq 0 ]]; then
    echo "::error::Allowlist is empty: ${RELEASE_CRATE_FILE}" >&2
    exit 1
  fi

  declare -A seen=()
  for crate in "${crates[@]}"; do
    if [[ -n "${seen[$crate]+x}" ]]; then
      echo "::error::Duplicate crate in allowlist: ${crate}" >&2
      exit 1
    fi
    seen["$crate"]=1
    printf '%s\n' "$crate"
  done
}

emit_auto_crates() {
  local metadata
  metadata="$(cargo metadata --no-deps --format-version 1)"
  if ! jq -e '.packages | length > 0' <<<"$metadata" >/dev/null 2>&1; then
    echo "::error::Failed to read cargo metadata for auto surface calculation." >&2
    exit 1
  fi

  mapfile -t publishable_crates < <(
    jq -r '
      .packages[]
      | select((.publish == null) or
               (.publish == true) or
               ((.publish | type == "array") and (.publish | index("crates.io") != null)))
      | .name' <<< "$metadata" | sort -u
  )
  if [[ ${#publishable_crates[@]} -eq 0 ]]; then
    echo "::error::No publishable crates found in workspace metadata." >&2
    exit 1
  fi

  declare -A is_publishable=()
  for crate in "${publishable_crates[@]}"; do
    is_publishable["$crate"]=1
  done

  declare -A indegree=()
  declare -A dependents=()
  declare -A seen_edge=()
  for crate in "${publishable_crates[@]}"; do
    indegree["$crate"]=0
  done

  while IFS=$'\t' read -r crate dep; do
    [[ -z "$crate" || -z "$dep" ]] && continue
    [[ -z "${is_publishable[$crate]+x}" ]] && continue
    [[ -z "${is_publishable[$dep]+x}" ]] && continue
    [[ "$crate" == "$dep" ]] && continue

    edge="${dep}::${crate}"
    if [[ -n "${seen_edge[$edge]+x}" ]]; then
      continue
    fi
    seen_edge["$edge"]=1
    (( indegree["$crate"] += 1 ))

    if [[ -z "${dependents[$dep]+x}" ]]; then
      dependents["$dep"]="$crate"
    else
      dependents["$dep"]="${dependents[$dep]}$'\n'$crate"
    fi
  done < <(
    jq -r '
      .packages[] as $pkg
      | $pkg.name as $crate
      | $pkg.dependencies[]?
      | select((.kind // ["normal"]) | index("dev") | not)
      | select((.optional // false) | not)
      | "\($crate)\t\(.name)"' <<< "$metadata"
  )

  ordered_crates=()
  declare -A seen=()
  while true; do
    progress=0
    for crate in "${publishable_crates[@]}"; do
      [[ -n "${seen[$crate]+x}" ]] && continue
      if (( indegree["$crate"] == 0 )); then
        seen["$crate"]=1
        ordered_crates+=("$crate")
        progress=1

        if [[ -n "${dependents[$crate]+x}" ]]; then
          while IFS= read -r dependent; do
            [[ -z "$dependent" ]] && continue
            (( indegree["$dependent"] -= 1 ))
          done <<< "${dependents[$crate]}"
        fi
      fi
    done

    if (( progress == 0 )); then
      break
    fi
  done

  if [[ ${#ordered_crates[@]} -ne ${#publishable_crates[@]} ]]; then
    unresolved=()
    for crate in "${publishable_crates[@]}"; do
      [[ -n "${seen[$crate]+x}" ]] && continue
      unresolved+=("$crate")
    done
    echo "::error::Could not order publishable crates in a dependency-safe way." >&2
    echo "::error::Unordered crates: ${unresolved[*]}" >&2
    exit 1
  fi

  for crate in "${ordered_crates[@]}"; do
    printf '%s\n' "$crate"
  done
}

if [[ "$RELEASE_SURFACE_MODE" == "fixed" ]]; then
  emit_fixed_crates
else
  mapfile -t CRATES_TO_PUBLISH < <(emit_auto_crates)
fi

if [[ "${RELEASE_CRATE_SYNC}" == "1" || "${RELEASE_CRATE_SYNC,,}" == "true" ]]; then
  if [[ "$RELEASE_SURFACE_MODE" != "auto" ]]; then
    echo "::warning::RELEASE_CRATE_SYNC is only used in auto mode; ignoring." >&2
  else
    if [[ "${RELEASE_CRATE_FILE}" != "" ]]; then
      {
        echo "# Auto-generated publish order from workspace metadata (all publishable crates)."
        printf '%s\n' "${CRATES_TO_PUBLISH[@]}"
      } > "$RELEASE_CRATE_FILE"
    fi
  fi
fi
