#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/clippy-collect.sh
# Produces clippy-report/<pkg>-default.txt and clippy-report/<pkg>-c2rust.txt
# and prints a summary at the end.

command -v jq >/dev/null 2>&1 || {
  echo "Error: jq is required. Install: apt-get install jq | brew install jq"
  exit 2
}

OUTDIR="clippy-report"
mkdir -p "$OUTDIR"

# workspace package names (works when members refer to package id strings)
PKGS=$(cargo metadata --format-version 1 2>/dev/null \
  | jq -r '.workspace_members as $members | .packages[] | select(.id as $id | $members[] | contains($id)) | .name')

# quarantine: skip blank and comment lines, trim whitespace, join with commas
Q=$(grep -v '^#' .clippy-quarantine 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' | tr '\n' ',' | sed 's/,$//' || true)

# packages that actually define tree-sitter-c2rust feature (comma list)
C2RUST_PKGS=$(cargo metadata --format-version 1 2>/dev/null \
  | jq -r '.packages[] | select(.features | has("tree-sitter-c2rust")) | .name' | tr '\n' ',' | sed 's/,$//' || true)

# helpers
is_quarantined() {
  local p="$1"
  case ",$Q," in *,"$p",*) return 0;; *) return 1;; esac
}

has_c2rust() {
  local p="$1"
  case ",$C2RUST_PKGS," in *,"$p",*) return 0;; *) return 1;; esac
}

failures_default=()
failures_c2rust=()

echo "Packages to consider:"
for p in $PKGS; do
  if is_quarantined "$p"; then
    echo "  skip (quarantined): $p"
    continue
  fi
  echo "  will check (default): $p"
done

# Run default mode per package
echo
echo "=== Running default-feature clippy per-package (no-deps) ==="
for p in $PKGS; do
  if is_quarantined "$p"; then
    continue
  fi

  out="$OUTDIR/${p//\//-}-default.txt"
  printf "\n--- %s: default -> %s\n" "$p" "$out"
  set +e
  cargo clippy -p "$p" --all-targets --no-deps -- -D warnings >"$out" 2>&1
  rc=$?
  set -e
  if [ $rc -ne 0 ]; then
    failures_default+=("$p")
    printf "FAILED: %s (exit %d) — log: %s\n" "$p" "$rc" "$out"
  else
    printf "OK: %s\n" "$p"
    # optionally delete successful logs to save space:
    # rm -f "$out"
  fi
done

# Run c2rust mode only for packages that define the feature
echo
echo "=== Running tree-sitter-c2rust feature clippy (no-deps) ==="
for p in $PKGS; do
  if is_quarantined "$p"; then
    continue
  fi
  if ! has_c2rust "$p"; then
    echo "  skip (no c2rust feature): $p"
    continue
  fi

  out="$OUTDIR/${p//\//-}-c2rust.txt"
  printf "\n--- %s: c2rust -> %s\n" "$p" "$out"
  set +e
  cargo clippy -p "$p" --all-targets --no-default-features --features tree-sitter-c2rust --no-deps -- -D warnings >"$out" 2>&1
  rc=$?
  set -e
  if [ $rc -ne 0 ]; then
    failures_c2rust+=("$p")
    printf "FAILED: %s (exit %d) — log: %s\n" "$p" "$rc" "$out"
  else
    printf "OK: %s\n" "$p"
  fi
done

# Summary
echo
echo "=== Summary ==="
echo "Reports saved under: $OUTDIR/"
if [ ${#failures_default[@]} -eq 0 ] && [ ${#failures_c2rust[@]} -eq 0 ]; then
  echo "All non-quarantined packages passed Clippy (both modes where applicable). 🎉"
else
  if [ ${#failures_default[@]} -gt 0 ]; then
    echo "Default-feature failures (${#failures_default[@]}):"
    for p in "${failures_default[@]}"; do echo "  - $p -> $OUTDIR/${p//\//-}-default.txt"; done
  else
    echo "No default-feature failures."
  fi
  if [ ${#failures_c2rust[@]} -gt 0 ]; then
    echo "c2rust-mode failures (${#failures_c2rust[@]}):"
    for p in "${failures_c2rust[@]}"; do echo "  - $p -> $OUTDIR/${p//\//-}-c2rust.txt"; done
  else
    echo "No c2rust-mode failures."
  fi
fi

# Exit non-zero if anything failed (so CI can mark failure) — comment out locally if you prefer 0.
if [ ${#failures_default[@]} -gt 0 ] || [ ${#failures_c2rust[@]} -gt 0 ]; then
  echo "One or more packages failed Clippy. See logs."
  exit 2
fi

echo "Done."