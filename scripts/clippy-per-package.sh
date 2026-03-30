#!/usr/bin/env bash
set -euo pipefail

mode="${1:-default}"  # default | c2rust

# Check for jq availability
command -v jq >/dev/null 2>&1 || {
  echo "Error: jq is required for clippy checks"
  echo "Install it via: apt-get install jq (Linux) | brew install jq (macOS) | choco install jq (Windows)"
  exit 2
}

# Build quarantine list (normalize whitespace, skip comments and blank lines)
Q=""
if [ -f .clippy-quarantine ]; then
  Q=$(grep -v '^#' .clippy-quarantine 2>/dev/null \
      | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
      | grep -v '^$' \
      | tr -d '\r' \
      | tr '\n' ',' | sed 's/,$//')
fi

# Get workspace packages - use package names from packages array filtered by workspace membership
PKGS=$(cargo metadata --format-version 1 | jq -r '
  .workspace_members as $members |
  .packages[] |
  select(.id as $id | $members[] | contains($id)) |
  .name' | tr -d '\r')

# Get packages that define tree-sitter-c2rust feature
C2RUST_PKGS=""
if [ "$mode" = "c2rust" ]; then
  C2RUST_PKGS=$(cargo metadata --format-version 1 \
    | jq -r '.packages[] | select(.features["tree-sitter-c2rust"]) | .name' \
    | tr -d '\r' \
    | tr '\n' ',' \
    | sed 's/,$//')
fi

# Helper function for error reporting
fail_clippy() {
  local pkg="$1"
  local features="$2"
  local cmd="$3"
  echo "✖ Clippy failed for package: $pkg ($features)"
  echo "  To reproduce: $cmd"
  exit 1
}

# Track if we ran any checks
ran_checks=false

# Run clippy on each package
for p in $PKGS; do
  # Skip quarantined packages
  case ",$Q," in 
    *,"$p",*) 
      echo "  skip (quarantined): $p"
      continue
      ;;
  esac

  if [ "$mode" = "default" ]; then
    echo "  clippy (default): $p"
    cmd=(cargo clippy -q -p "$p" --all-targets --no-deps -- -D warnings)
    "${cmd[@]}" || fail_clippy "$p" "default features" "$(printf '%q ' "${cmd[@]}")"
    ran_checks=true
  elif [ "$mode" = "c2rust" ]; then
    # Only run c2rust checks on packages that define the feature
    case ",$C2RUST_PKGS," in 
      *,"$p",*) 
        echo "  clippy (c2rust): $p"
        cmd=(cargo clippy -q -p "$p" --all-targets --no-default-features --features tree-sitter-c2rust --no-deps -- -D warnings)
        "${cmd[@]}" || fail_clippy "$p" "tree-sitter-c2rust" "$(printf '%q ' "${cmd[@]}")"
        ran_checks=true
        ;;
      *)
        echo "  skip (no c2rust feature): $p"
        ;;
    esac
  fi
done

if [ "$ran_checks" = true ]; then
  echo "✓ clippy-per-package ($mode) OK"
else
  echo "⚠ No packages were checked (all quarantined or skipped)"
fi
