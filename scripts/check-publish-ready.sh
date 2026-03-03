#!/usr/bin/env bash
# check-publish-ready.sh — Verify crates are ready for crates.io publication
# Usage: ./scripts/check-publish-ready.sh [--fix]
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Publishable crates in dependency order
CRATES=(
  "ir:adze-ir"
  "glr-core:adze-glr-core"
  "tablegen:adze-tablegen"
  "common:adze-common"
  "macro:adze-macro"
  "runtime:adze"
  "tool:adze-tool"
)

REQUIRED_FIELDS=(name version license description repository homepage)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

errors=0
warnings=0

pass()  { echo -e "  ${GREEN}✓${NC} $1"; }
fail()  { echo -e "  ${RED}✗${NC} $1"; ((errors++)); }
warn()  { echo -e "  ${YELLOW}!${NC} $1"; ((warnings++)); }
info()  { echo -e "  ${CYAN}ℹ${NC} $1"; }

echo "═══════════════════════════════════════════════"
echo " Adze Publish Readiness Check"
echo "═══════════════════════════════════════════════"
echo

# ─── 1. License files ────────────────────────────
echo -e "${CYAN}[1/7] License files${NC}"
for lic in LICENSE-APACHE LICENSE-MIT; do
  if [[ -f "$lic" ]]; then
    pass "$lic exists"
  else
    fail "$lic missing at repo root"
  fi
done
echo

# ─── 2. MSRV consistency ─────────────────────────
echo -e "${CYAN}[2/7] MSRV consistency${NC}"
WORKSPACE_MSRV=$(grep 'rust-version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
TOOLCHAIN_CHANNEL=$(grep 'channel' rust-toolchain.toml | sed 's/.*"\(.*\)".*/\1/')

if [[ "$WORKSPACE_MSRV" == "$TOOLCHAIN_CHANNEL" ]]; then
  pass "Workspace MSRV ($WORKSPACE_MSRV) matches rust-toolchain.toml ($TOOLCHAIN_CHANNEL)"
else
  fail "MSRV mismatch: workspace=$WORKSPACE_MSRV toolchain=$TOOLCHAIN_CHANNEL"
fi
echo

# ─── 3. Per-crate checks ─────────────────────────
echo -e "${CYAN}[3/7] Cargo.toml required fields${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  toml="$dir/Cargo.toml"
  echo -e "  ${CYAN}── $pkg ($toml)${NC}"

  for field in "${REQUIRED_FIELDS[@]}"; do
    # Check for direct field or workspace inheritance
    if grep -qE "^${field}\s*=" "$toml" || grep -qE "^${field}\.workspace\s*=" "$toml"; then
      pass "$field present"
    else
      fail "$field MISSING in $toml"
    fi
  done
done
echo

# ─── 4. README availability ──────────────────────
echo -e "${CYAN}[4/7] README availability${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  toml="$dir/Cargo.toml"

  readme_field=$(grep -oP 'readme\s*=\s*"\K[^"]+' "$toml" 2>/dev/null || echo "")
  if [[ -z "$readme_field" ]]; then
    readme_field="README.md"
  fi

  # Resolve relative to crate dir
  if [[ "$readme_field" == ../* ]]; then
    readme_path="$dir/$readme_field"
  else
    readme_path="$dir/$readme_field"
  fi

  # Normalize path
  readme_resolved=$(realpath -m "$readme_path" 2>/dev/null || echo "$readme_path")

  if [[ -f "$readme_resolved" ]]; then
    pass "$pkg README found ($readme_field)"
  else
    fail "$pkg README not found at $readme_field (resolved: $readme_resolved)"
  fi
done
echo

# ─── 5. Path dependency leak check ───────────────
echo -e "${CYAN}[5/7] Path dependency leak check (deps must have version)${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  toml="$dir/Cargo.toml"

  # Find lines with path = that are not [lib] path or [[bin]] path or [[bench]] path or [[example]] path
  # Only check [dependencies] and [dev-dependencies] sections
  leak_found=false
  while IFS= read -r line; do
    # Skip lib/bin/bench/example path declarations
    if echo "$line" | grep -qE '^\s*path\s*=\s*"src/'; then
      continue
    fi
    # Check if line has path= but no version=
    if echo "$line" | grep -qE 'path\s*=' && ! echo "$line" | grep -qE 'version\s*='; then
      # This is an inline table without version - flag it
      dep_name=$(echo "$line" | sed 's/\s*=.*//')
      fail "$pkg: dependency '$dep_name' has path but no version in $toml"
      leak_found=true
    fi
  done < <(grep -E '^\s*\w.*path\s*=' "$toml" | grep -v '^\[' | grep -v 'path\s*=\s*"src/')

  if [[ "$leak_found" == false ]]; then
    pass "$pkg: all path dependencies have versions"
  fi
done
echo

# ─── 6. TODO/FIXME/HACK in public API ────────────
echo -e "${CYAN}[6/7] TODO/FIXME/HACK in public API (lib.rs)${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  librs="$dir/src/lib.rs"

  if [[ ! -f "$librs" ]]; then
    warn "$pkg: no lib.rs found"
    continue
  fi

  count=$(grep -cE '\bTODO\b|\bFIXME\b|\bHACK\b' "$librs" 2>/dev/null || true)
  count="${count:-0}"
  if [[ "$count" -gt 0 ]]; then
    warn "$pkg: $count TODO/FIXME/HACK markers in lib.rs"
    grep -nE '\bTODO\b|\bFIXME\b|\bHACK\b' "$librs" | head -5 | while read -r line; do
      info "  $line"
    done
  else
    pass "$pkg: no TODO/FIXME/HACK in lib.rs"
  fi
done
echo

# ─── 7. cargo package --list ─────────────────────
echo -e "${CYAN}[7/7] cargo package --list (dry-run packaging)${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"

  echo -e "  ${CYAN}── $pkg${NC}"
  if output=$(cargo package --list -p "$pkg" --no-verify 2>&1); then
    file_count=$(echo "$output" | wc -l)
    pass "package --list succeeded ($file_count files)"
  else
    fail "package --list failed for $pkg"
    echo "$output" | head -10 | while read -r line; do
      info "  $line"
    done
  fi
done
echo

# ─── Summary ─────────────────────────────────────
echo "═══════════════════════════════════════════════"
if [[ $errors -eq 0 && $warnings -eq 0 ]]; then
  echo -e "${GREEN}All checks passed!${NC}"
elif [[ $errors -eq 0 ]]; then
  echo -e "${YELLOW}Passed with $warnings warning(s)${NC}"
else
  echo -e "${RED}$errors error(s), $warnings warning(s)${NC}"
fi
echo "═══════════════════════════════════════════════"

exit $errors
