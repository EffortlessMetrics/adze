#!/usr/bin/env bash
set -euo pipefail

# publish-order.sh — Analyze workspace dependency graph and output
# the correct crates.io publish order (leaves first).
#
# Usage:
#   scripts/publish-order.sh              # Show publish order
#   scripts/publish-order.sh --dry-run    # Also run cargo publish --dry-run
#   scripts/publish-order.sh --validate   # Validate metadata only

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

DRY_RUN=0
VALIDATE_ONLY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)   DRY_RUN=1; shift ;;
    --validate)  VALIDATE_ONLY=1; shift ;;
    -h|--help)
      cat <<'EOF'
Usage: publish-order.sh [OPTIONS]

Analyze workspace dependency graph and output correct crates.io publish order.

Options:
  --dry-run     Run cargo publish --dry-run for each crate in order
  --validate    Only validate metadata, don't show order
  -h, --help    Show this help
EOF
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! command -v jq &>/dev/null; then
  echo "error: jq is required but not found. Install with: apt install jq" >&2
  exit 1
fi

cd "$ROOT_DIR"

METADATA="$(cargo metadata --no-deps --format-version 1 2>/dev/null)"

# Extract publishable crates (publish == null, true, or contains "crates.io")
# along with their metadata and workspace dependencies.
CRATE_INFO="$(jq -r '
  # Build set of all workspace package names
  ([.packages[].name] | sort | unique) as $ws_names |

  .packages[] |
  # Filter to publishable crates
  select(
    (.publish == null) or
    (.publish == true) or
    ((.publish | type == "array") and (.publish | length > 0) and (.publish | index("crates.io") != null))
  ) |
  {
    name: .name,
    version: .version,
    description: (.description // ""),
    license: (.license // ""),
    repository: (.repository // ""),
    # Collect non-dev workspace dependencies
    ws_deps: [
      .dependencies[]? |
      select(.path != null) |
      select((.kind // "normal") != "dev") |
      .name
    ] | unique
  } |
  # Format as TSV for bash consumption
  "\(.name)\t\(.version)\t\(.description)\t\(.license)\t\(.repository)\t\(.ws_deps | join(","))"
' <<< "$METADATA")"

if [[ -z "$CRATE_INFO" ]]; then
  echo "error: no publishable crates found in workspace." >&2
  exit 1
fi

# Parse into associative arrays
declare -A VERSIONS=()
declare -A DESCRIPTIONS=()
declare -A LICENSES=()
declare -A REPOSITORIES=()
declare -A WS_DEPS=()
PUBLISHABLE=()

while IFS=$'\t' read -r name version desc lic repo deps; do
  [[ -z "$name" ]] && continue
  PUBLISHABLE+=("$name")
  VERSIONS["$name"]="$version"
  DESCRIPTIONS["$name"]="$desc"
  LICENSES["$name"]="$lic"
  REPOSITORIES["$name"]="$repo"
  WS_DEPS["$name"]="$deps"
done <<< "$CRATE_INFO"

# Build set of publishable names for filtering deps
declare -A IS_PUBLISHABLE=()
for c in "${PUBLISHABLE[@]}"; do
  IS_PUBLISHABLE["$c"]=1
done

# Filter ws_deps to only include other publishable crates
declare -A FILTERED_DEPS=()
for c in "${PUBLISHABLE[@]}"; do
  filtered=()
  if [[ -n "${WS_DEPS[$c]}" ]]; then
    IFS=',' read -ra raw_deps <<< "${WS_DEPS[$c]}"
    for d in "${raw_deps[@]}"; do
      [[ -n "${IS_PUBLISHABLE[$d]+x}" ]] && filtered+=("$d")
    done
  fi
  FILTERED_DEPS["$c"]="$(IFS=','; echo "${filtered[*]}")"
done

# --- Metadata validation ---
BLOCKERS=()
validate_metadata() {
  local name="$1"
  local issues=()
  [[ -z "${DESCRIPTIONS[$name]}" ]] && issues+=("missing description")
  [[ -z "${LICENSES[$name]}" ]]     && issues+=("missing license")
  [[ -z "${REPOSITORIES[$name]}" ]] && issues+=("missing repository")
  if [[ ${#issues[@]} -gt 0 ]]; then
    BLOCKERS+=("$name: $(IFS=', '; echo "${issues[*]}")")
  fi
}

for c in "${PUBLISHABLE[@]}"; do
  validate_metadata "$c"
done

if [[ "$VALIDATE_ONLY" -eq 1 ]]; then
  echo "=== Metadata validation for ${#PUBLISHABLE[@]} publishable crates ==="
  echo ""
  if [[ ${#BLOCKERS[@]} -gt 0 ]]; then
    echo "BLOCKING ISSUES:"
    for b in "${BLOCKERS[@]}"; do
      echo "  ✗ $b"
    done
    exit 1
  else
    echo "✓ All ${#PUBLISHABLE[@]} crates have required metadata."
    exit 0
  fi
fi

# --- Topological sort (Kahn's algorithm) ---
declare -A INDEGREE=()
declare -A DEPENDENTS=()
for c in "${PUBLISHABLE[@]}"; do
  INDEGREE["$c"]=0
done

for c in "${PUBLISHABLE[@]}"; do
  [[ -z "${FILTERED_DEPS[$c]}" ]] && continue
  IFS=',' read -ra deps <<< "${FILTERED_DEPS[$c]}"
  for d in "${deps[@]}"; do
    [[ -z "$d" ]] && continue
    INDEGREE["$c"]=$(( INDEGREE["$c"] + 1 ))
    if [[ -z "${DEPENDENTS[$d]+x}" ]]; then
      DEPENDENTS["$d"]="$c"
    else
      DEPENDENTS["$d"]="${DEPENDENTS[$d]}"$'\n'"$c"
    fi
  done
done

ORDERED=()
declare -A VISITED=()
while true; do
  progress=0
  for c in "${PUBLISHABLE[@]}"; do
    [[ -n "${VISITED[$c]+x}" ]] && continue
    if (( INDEGREE["$c"] == 0 )); then
      VISITED["$c"]=1
      ORDERED+=("$c")
      progress=1
      if [[ -n "${DEPENDENTS[$c]+x}" ]]; then
        while IFS= read -r dep; do
          [[ -z "$dep" ]] && continue
          INDEGREE["$dep"]=$(( INDEGREE["$dep"] - 1 ))
        done <<< "${DEPENDENTS[$c]}"
      fi
    fi
  done
  [[ "$progress" -eq 0 ]] && break
done

if [[ ${#ORDERED[@]} -ne ${#PUBLISHABLE[@]} ]]; then
  echo "error: circular dependency detected. Unresolved crates:" >&2
  for c in "${PUBLISHABLE[@]}"; do
    [[ -z "${VISITED[$c]+x}" ]] && echo "  - $c" >&2
  done
  exit 1
fi

# --- Output ---
echo "=== crates.io publish order (${#ORDERED[@]} crates) ==="
echo ""

for i in "${!ORDERED[@]}"; do
  c="${ORDERED[$i]}"
  n=$(( i + 1 ))
  dep_str="${FILTERED_DEPS[$c]}"
  if [[ -z "$dep_str" ]]; then
    dep_label="no workspace deps"
  else
    dep_label="depends on: ${dep_str//,/, }"
  fi
  printf '%2d. %s v%s (%s)\n' "$n" "$c" "${VERSIONS[$c]}" "$dep_label"
done

# --- Blocking issues ---
if [[ ${#BLOCKERS[@]} -gt 0 ]]; then
  echo ""
  echo "BLOCKING ISSUES:"
  for b in "${BLOCKERS[@]}"; do
    echo "  ✗ $b"
  done
fi

# --- Dry run ---
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo ""
  echo "=== Running cargo publish --dry-run ==="
  echo ""
  failures=0
  for c in "${ORDERED[@]}"; do
    echo ">>> [$c] cargo publish --dry-run ..."
    if cargo publish -p "$c" --dry-run 2>&1; then
      echo "  ✓ $c OK"
    else
      echo "  ✗ $c FAILED"
      failures=$(( failures + 1 ))
    fi
    echo ""
  done

  if [[ "$failures" -gt 0 ]]; then
    echo "FAIL: $failures crate(s) failed dry-run publish."
    exit 1
  fi
  echo "=== All dry-run publishes succeeded ==="
fi
