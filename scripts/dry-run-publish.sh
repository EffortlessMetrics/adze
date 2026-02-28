#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_CRATE_FILE="${RELEASE_CRATE_FILE:-${SCRIPT_DIR}/release-crates.txt}"
RELEASE_SURFACE_MODE="${RELEASE_SURFACE_MODE:-fixed}"
STRICT_PUBLISH_SURFACE="${STRICT_PUBLISH_SURFACE:-0}"

echo "=== Package validation for adze workspace ==="
echo "Release surface mode: ${RELEASE_SURFACE_MODE}"
echo "Release crate file: ${RELEASE_CRATE_FILE}"
echo "Strict publish surface: ${STRICT_PUBLISH_SURFACE}"
echo ""

mapfile -t CRATES < <(RELEASE_SURFACE_MODE="$RELEASE_SURFACE_MODE" \
  RELEASE_CRATE_FILE="$RELEASE_CRATE_FILE" "${SCRIPT_DIR}/release-surface.sh")
if [[ ${#CRATES[@]} -eq 0 ]]; then
  echo "Release crate allowlist is empty: ${RELEASE_CRATE_FILE}" >&2
  exit 1
fi

RELEASE_SURFACE_MODE="$RELEASE_SURFACE_MODE" \
RELEASE_CRATE_FILE="$RELEASE_CRATE_FILE" \
STRICT_PUBLISH_SURFACE="$STRICT_PUBLISH_SURFACE" \
"${SCRIPT_DIR}/validate-release-surface.sh"
echo

for crate in "${CRATES[@]}"; do
    echo ">>> Validating $crate..."
    if cargo package -p "$crate" --no-verify 2>&1; then
        echo "  $crate OK"
    else
        echo "  $crate FAILED"
        exit 1
    fi
    echo ""
done

echo "=== All crate manifests valid ==="
