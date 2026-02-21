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

# Check if tag already exists locally
if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "Tag ${tag} already exists locally. Aborting."
  exit 1
fi

# Check if tag already exists on origin
if git ls-remote --exit-code --tags origin "${tag}" >/dev/null 2>&1; then
  echo "Tag ${tag} already exists on origin. Aborting."
  exit 1
fi

if $DRY_RUN; then
  echo "DRY RUN: git tag -a ${tag} -m \"${msg}\""
  echo "DRY RUN: git push origin ${tag}"
else
  git tag -a "${tag}" -m "${msg}"
  git push origin "${tag}"
fi

# Allow SLEEP_SECS override
SLEEP_SECS="${SLEEP_SECS:-75}"

publish() {
  crate=$1
  if $DRY_RUN; then
    echo "DRY RUN: cargo publish -p ${crate}"
  else
    echo "Publishing ${crate} ..."
    cargo publish -p "${crate}"
    echo "Sleeping ${SLEEP_SECS}s to allow crates.io indexing..."
    sleep "${SLEEP_SECS}"
  fi
}

# Publish in safe order:
publish adze-glr-core
publish adze-ir
publish adze-common
publish adze
publish adze-macro
publish adze-tool
# optional:
# publish adze-example

echo "Done. If not dry-run: double-check crates.io and run smoke tests."