#!/usr/bin/env bash
# check-publish-readiness.sh — Verify all publishable crates are ready for crates.io publication
# Usage: ./scripts/check-publish-readiness.sh
# Checks for required fields, valid semver, README, license, no publish=false, 
# cargo package success, and file size limits
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# All publishable crates in dependency order
# Format: "directory:package-name"
CRATES=(
  "ir:adze-ir"
  "glr-core:adze-glr-core"
  "tablegen:adze-tablegen"
  "common:adze-common"
  "macro:adze-macro"
  "runtime:adze"
  "runtime2:adze-runtime"
  "tool:adze-tool"
)

REQUIRED_FIELDS=(name version description license repository edition)
MAX_FILE_SIZE=$((10 * 1024 * 1024))  # 10MB in bytes

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'  # No Color

errors=0
warnings=0

# Helper functions
pass()  { echo -e "  ${GREEN}✓${NC} $1"; }
fail()  { echo -e "  ${RED}✗${NC} $1"; ((errors++)); }
warn()  { echo -e "  ${YELLOW}!${NC} $1"; ((warnings++)); }
info()  { echo -e "  ${CYAN}ℹ${NC} $1"; }

echo "═══════════════════════════════════════════════════════════════"
echo " Publish Readiness Check - All Supported Crates"
echo "═══════════════════════════════════════════════════════════════"
echo

# ─── Check 1: License files ──────────────────────────────────────
echo -e "${CYAN}[1/7] License files${NC}"
for lic in LICENSE-APACHE LICENSE-MIT; do
  if [[ -f "$lic" ]]; then
    pass "$lic exists"
  else
    fail "$lic missing at repo root"
  fi
done
echo

# ─── Check 2: MSRV consistency ──────────────────────────────────
echo -e "${CYAN}[2/7] MSRV consistency${NC}"
WORKSPACE_MSRV=$(grep 'rust-version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
TOOLCHAIN_CHANNEL=$(grep 'channel' rust-toolchain.toml | sed 's/.*"\(.*\)".*/\1/')

if [[ "$WORKSPACE_MSRV" == "$TOOLCHAIN_CHANNEL" ]]; then
  pass "Workspace MSRV ($WORKSPACE_MSRV) matches rust-toolchain.toml ($TOOLCHAIN_CHANNEL)"
else
  fail "MSRV mismatch: workspace=$WORKSPACE_MSRV toolchain=$TOOLCHAIN_CHANNEL"
fi
echo

# ─── Check 3: Per-crate metadata ────────────────────────────────
echo -e "${CYAN}[3/7] Cargo.toml required fields${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  toml="$dir/Cargo.toml"
  
  if [[ ! -f "$toml" ]]; then
    fail "$pkg: $toml not found"
    continue
  fi
  
  echo -e "  ${CYAN}── $pkg ($dir/)${NC}"

  # Check required fields
  for field in "${REQUIRED_FIELDS[@]}"; do
    # Check for direct field or workspace inheritance
    if grep -qE "^${field}\s*=" "$toml" || grep -qE "^${field}\.workspace\s*=" "$toml"; then
      pass "$field present"
    else
      fail "$field MISSING in $toml"
    fi
  done
  
  # Validate semver version
  version=$(grep -oP '^version\s*=\s*"\K[^"]+' "$toml" || echo "")
  if [[ -z "$version" ]]; then
    # Try workspace inheritance
    version=$(grep -oP '^version\.workspace\s*=\s*true' "$toml" > /dev/null && grep -oP '^version\s*=\s*"\K[^"]+' Cargo.toml | head -1 || echo "")
  fi
  
  if [[ -n "$version" ]]; then
    # Basic semver check: major.minor.patch with optional pre-release/build
    if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?(\+[a-zA-Z0-9]+)?$ ]]; then
      pass "version '$version' is valid semver"
    else
      warn "version '$version' may not be valid semver (expected X.Y.Z[-pre][+build])"
    fi
  else
    fail "version not found or not parseable"
  fi
done
echo

# ─── Check 4: README availability ──────────────────────────────
echo -e "${CYAN}[4/7] README availability${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  toml="$dir/Cargo.toml"

  if [[ ! -f "$toml" ]]; then
    warn "$pkg: $toml not found (skipping README check)"
    continue
  fi

  # Get readme field from Cargo.toml
  readme_field=$(grep -oP 'readme\s*=\s*"\K[^"]+' "$toml" 2>/dev/null || echo "")
  if [[ -z "$readme_field" ]]; then
    readme_field="README.md"
  fi

  # Resolve path relative to crate dir
  if [[ "$readme_field" == /* ]]; then
    # Absolute path (unlikely but handle it)
    readme_path="$readme_field"
  elif [[ "$readme_field" == ../* ]]; then
    # Path starting with ../
    readme_path="$dir/$readme_field"
  else
    # Relative to crate dir
    readme_path="$dir/$readme_field"
  fi

  # Resolve to actual path
  if [[ -f "$readme_path" ]]; then
    pass "$pkg README found ($readme_field)"
  else
    # Check for README without extension in crate dir
    if [[ -f "$dir/README" ]]; then
      pass "$pkg README found (README without extension)"
    else
      fail "$pkg README not found at $readme_field (looked in: $readme_path)"
    fi
  fi
done
echo

# ─── Check 5: publish flag ──────────────────────────────────────
echo -e "${CYAN}[5/7] publish flag (should not be false)${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"
  toml="$dir/Cargo.toml"

  if [[ ! -f "$toml" ]]; then
    continue
  fi

  # Check if publish = false is set
  if grep -qE '^\s*publish\s*=\s*false' "$toml"; then
    fail "$pkg: publish = false is set (must be true or absent for public crates)"
  elif grep -qE '^\s*publish\s*=\s*true' "$toml"; then
    pass "$pkg: publish = true is set"
  else
    pass "$pkg: publish flag not set (defaults to true)"
  fi
done
echo

# ─── Check 6: cargo package --list (dry-run) ────────────────────
echo -e "${CYAN}[6/7] cargo package --list (dry-run packaging)${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"

  if [[ ! -f "$dir/Cargo.toml" ]]; then
    warn "$pkg: Cargo.toml not found (skipping package check)"
    continue
  fi

  echo -e "  ${CYAN}── $pkg${NC}"
  if output=$(cargo package --list -p "$pkg" --no-verify --allow-dirty 2>&1); then
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

# ─── Check 7: File size limits ──────────────────────────────────
echo -e "${CYAN}[7/7] File size limits (crates.io max 10MB)${NC}"
for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  pkg="${entry##*:}"

  if [[ ! -f "$dir/Cargo.toml" ]]; then
    continue
  fi

  # Get list of files that would be packaged
  if files_output=$(cargo package --list -p "$pkg" --no-verify --allow-dirty 2>&1); then
    # Extract file paths and check sizes
    found_oversized=false
    while IFS= read -r file; do
      file_path="$dir/$file"
      if [[ -f "$file_path" ]]; then
        size=$(stat -f%z "$file_path" 2>/dev/null || stat -c%s "$file_path" 2>/dev/null || echo 0)
        if [[ $size -gt $MAX_FILE_SIZE ]]; then
          fail "$pkg: oversized file $file ($(numfmt --to=iec $size 2>/dev/null || echo "$size bytes") > 10MB)"
          found_oversized=true
        fi
      fi
    done < <(echo "$files_output")
    
    if [[ "$found_oversized" == false ]]; then
      pass "$pkg: all packaged files are within size limits"
    fi
  else
    warn "$pkg: could not list package contents (skipping size check)"
  fi
done
echo

# ─── Summary ─────────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════════════"
echo "Summary:"
echo "  Errors:   $errors"
echo "  Warnings: $warnings"
echo "═══════════════════════────────────────────────────────────────"

if [[ $errors -eq 0 && $warnings -eq 0 ]]; then
  echo -e "${GREEN}✓ All checks passed! Crates are ready for publication.${NC}"
  exit 0
elif [[ $errors -eq 0 ]]; then
  echo -e "${YELLOW}⚠ Passed with $warnings warning(s)${NC}"
  exit 0
else
  echo -e "${RED}✗ $errors error(s), $warnings warning(s)${NC}"
  echo "Please fix errors before publishing."
  exit 1
fi
