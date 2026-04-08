#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STRICT_PUBLISH_SURFACE="${STRICT_PUBLISH_SURFACE:-0}"
RELEASE_CRATE_FILE="${RELEASE_CRATE_FILE:-${SCRIPT_DIR}/release-crates.txt}"
RELEASE_SURFACE_MODE="${RELEASE_SURFACE_MODE:-fixed}"

mapfile -t ALLOWED_CRATES < <(RELEASE_SURFACE_MODE="$RELEASE_SURFACE_MODE" \
  RELEASE_CRATE_FILE="$RELEASE_CRATE_FILE" "${SCRIPT_DIR}/release-surface.sh")
if [[ ${#ALLOWED_CRATES[@]} -eq 0 ]]; then
  echo "::error::Release surface is empty (mode: ${RELEASE_SURFACE_MODE})." >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "::error::python3 is required for release-surface validation." >&2
  exit 1
fi
METADATA_FILE="$(mktemp)"
trap 'rm -f "$METADATA_FILE"' EXIT
if ! cargo metadata --no-deps --format-version 1 >"$METADATA_FILE"; then
  echo "::error::Unable to load cargo metadata JSON for release-surface validation." >&2
  exit 1
fi

declare -A ALLOWLIST=()
declare -A ALLOWLIST_INDEX=()
has_failure=0
STRICT_PUBLISH_SURFACE="${STRICT_PUBLISH_SURFACE,,}"

for idx in "${!ALLOWED_CRATES[@]}"; do
  crate="${ALLOWED_CRATES[$idx]}"
  if [[ -n "${ALLOWLIST[$crate]+x}" ]]; then
    echo "::error::Duplicate crate in allowlist: ${crate}" >&2
    has_failure=1
    continue
  fi
  ALLOWLIST["$crate"]=1
  ALLOWLIST_INDEX["$crate"]="$idx"
done
declare -A SEEN_ALLOWED=()

extra_publishable=()

while IFS=$'\t' read -r crate manifest_path publishable; do
  if [[ "$publishable" == "false" ]]; then
    if [[ "$RELEASE_SURFACE_MODE" == "fixed" && -n "${ALLOWLIST[$crate]+x}" ]]; then
      echo "::error::Allowlisted crate '$crate' is not publishable (publish = false)." >&2
      has_failure=1
    fi
    continue
  fi

  if [[ -z "${ALLOWLIST[$crate]+x}" ]]; then
    if [[ "$RELEASE_SURFACE_MODE" == "fixed" && ("$STRICT_PUBLISH_SURFACE" == "1" || "$STRICT_PUBLISH_SURFACE" == "true" || "$STRICT_PUBLISH_SURFACE" == "yes" || "$STRICT_PUBLISH_SURFACE" == "on") ]]; then
      echo "::error::Unexpected publishable crate: $crate ($manifest_path)" >&2
      has_failure=1
      continue
    elif [[ "$RELEASE_SURFACE_MODE" == "fixed" ]]; then
      extra_publishable+=("$crate")
    else
      continue
    fi
  fi

  SEEN_ALLOWED["$crate"]=1
done < <(python3 - "$METADATA_FILE" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as fh:
    metadata = json.load(fh)

for pkg in metadata["packages"]:
    publish = pkg.get("publish")
    is_publishable = (
        publish is None
        or publish is True
        or (isinstance(publish, list) and ("crates.io" in publish or "crates-io" in publish))
    )
    print(f"{pkg['name']}\t{pkg['manifest_path']}\t{'true' if is_publishable else 'false'}")
PY
)

# Validate allowlist order against workspace dependency graph for normal/build dependencies.
if [[ "$RELEASE_SURFACE_MODE" == "fixed" ]]; then
  while IFS=$'\t' read -r crate dep; do
    [[ -z "${ALLOWLIST[$crate]+x}" ]] && continue
    [[ -z "${ALLOWLIST[$dep]+x}" ]] && continue
    [[ "$crate" == "$dep" ]] && continue

    crate_idx="${ALLOWLIST_INDEX[$crate]}"
    dep_idx="${ALLOWLIST_INDEX[$dep]}"
    if (( dep_idx >= crate_idx )); then
      echo "::error::Allowlist order violation: '$dep' must be published before '$crate'" >&2
      has_failure=1
    fi
  done < <(python3 - "$METADATA_FILE" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as fh:
    metadata = json.load(fh)

for pkg in metadata["packages"]:
    crate = pkg["name"]
    for dep in pkg.get("dependencies", []):
        kind = dep.get("kind")
        if kind == "dev" or dep.get("optional", False):
            continue
        print(f"{crate}\t{dep['name']}")
PY
)
fi

for crate in "${ALLOWED_CRATES[@]}"; do
  if [[ -z "${SEEN_ALLOWED[$crate]+x}" ]]; then
    echo "::error::Allowlisted crate '$crate' is not marked as publishable." >&2
    has_failure=1
  fi
done

if [[ "$RELEASE_SURFACE_MODE" == "fixed" && $has_failure == 0 ]] && (( ${#extra_publishable[@]} > 0 )); then
  extra_count="${#extra_publishable[@]}"
  if (( extra_count <= 12 )); then
    echo "::warning::Extra publishable crates are not in ${RELEASE_CRATE_FILE}: ${extra_publishable[*]}" >&2
  else
    shown=("${extra_publishable[@]:0:12}")
    remaining=$(( extra_count - 12 ))
    echo "::warning::Extra publishable crates are not in ${RELEASE_CRATE_FILE}: ${shown[*]}" >&2
    echo "::warning::... and ${remaining} more publishable crates" >&2
  fi
fi

if (( has_failure != 0 )); then
  echo "::error::Publish-surface validation failed." >&2
  exit 1
fi

echo "Publish-surface validation passed for mode=${RELEASE_SURFACE_MODE}:" \
  "${ALLOWED_CRATES[*]}"
