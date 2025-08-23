#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then DRY_RUN=true; fi

tag="v0.6.1-beta"
msg="Release ${tag}: Algorithmically correct GLR parser"

echo "=== Release helper ==="
echo "Tag: ${tag}"
echo "Dry run: ${DRY_RUN}"
echo

if $DRY_RUN; then
  echo "DRY RUN: git tag -a ${tag} -m \"${msg}\""
  echo "DRY RUN: git push origin ${tag}"
else
  git tag -a "${tag}" -m "${msg}"
  git push origin "${tag}"
fi

publish() {
  crate=$1
  if $DRY_RUN; then
    echo "DRY RUN: cargo publish -p ${crate}"
  else
    echo "Publishing ${crate} ..."
    cargo publish -p "${crate}"
    echo "Sleeping 75s to allow crates.io indexing..."
    sleep 75
  fi
}

# Publish in safe order:
publish rust-sitter-glr-core
publish rust-sitter-ir
publish rust-sitter-common
publish rust-sitter
publish rust-sitter-macro
publish rust-sitter-tool
# optional:
# publish rust-sitter-example

echo "Done. If not dry-run: double-check crates.io and run smoke tests."