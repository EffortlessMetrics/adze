#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 3 ]]; then
    echo "Usage: $0 <manifest-path> <baseline-subdir> [baseline-ref]" >&2
    exit 2
fi

MANIFEST_PATH=$1
BASELINE_SUBDIR=$2
BASELINE_REF=${3:-v0.8.0-dev.api-freeze-1}

REPO_ROOT=$(git rev-parse --show-toplevel)
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT
SEMVER_LOG="$TMPDIR/semver.log"

mkdir -p "$TMPDIR/baseline"
# Cargo needs the workspace root and the referenced member/path crates to
# resolve `workspace.dependencies` when the archived manifests point at sibling
# packages. Extract the full tracked repo snapshot so the baseline is
# self-contained, then rewrite crate names below.
git -C "$REPO_ROOT" archive "$BASELINE_REF" | tar -x -C "$TMPDIR/baseline"

python3 - "$TMPDIR/baseline" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])

string_replacements = [
    ("rust-sitter-common", "adze-common"),
    ("rust-sitter-macro", "adze-macro"),
    ("rust-sitter-ir", "adze-ir"),
    ("rust-sitter-glr-core", "adze-glr-core"),
    ("rust-sitter-tablegen", "adze-tablegen"),
    ("rust-sitter-tool", "adze-tool"),
    ("rust-sitter-runtime", "adze-runtime"),
    ("rust-sitter", "adze"),
]
identifier_replacements = [
    ("rust_sitter_common", "adze_common"),
    ("rust_sitter_macro", "adze_macro"),
    ("rust_sitter_ir", "adze_ir"),
    ("rust_sitter_glr_core", "adze_glr_core"),
    ("rust_sitter_tablegen", "adze_tablegen"),
    ("rust_sitter_tool", "adze_tool"),
    ("rust_sitter_runtime", "adze_runtime"),
    ("rust_sitter", "adze"),
]

def strip_dev_only_manifest_sections(text: str) -> str:
    text = re.sub(
        r"\n\[dev-dependencies\][\s\S]*?(?=\n\[[^\]]+\]|\n\[\[[^\]]+\]\]|\Z)",
        "\n",
        text,
    )
    text = re.sub(
        r"\n\[\[(bench|example|test)\]\][\s\S]*?(?=\n\[[^\]]+\]|\n\[\[[^\]]+\]\]|\Z)",
        "\n",
        text,
    )
    return text

for path in root.rglob("*"):
    if not path.is_file() or path.suffix not in {".toml", ".rs"}:
        continue

    text = path.read_text()
    for old, new in string_replacements:
        text = text.replace(old, new)
    for old, new in identifier_replacements:
        text = text.replace(old, new)

    if path.name == "Cargo.toml":
        text = strip_dev_only_manifest_sections(text)

    path.write_text(text)
PY

if CARGO_BUILD_RUSTFLAGS='' RUSTFLAGS='' cargo semver-checks check-release \
    --manifest-path "$MANIFEST_PATH" \
    --baseline-root "$TMPDIR/baseline/$BASELINE_SUBDIR" \
    >"$SEMVER_LOG" 2>&1; then
    cat "$SEMVER_LOG"
    exit 0
else
    SEMVER_STATUS=$?
fi

cat "$SEMVER_LOG"

if grep -q "no crates with library targets selected, nothing to semver-check" "$SEMVER_LOG"; then
    echo "Skipping semver check for unsupported target in $MANIFEST_PATH"
    exit 0
fi

exit "$SEMVER_STATUS"
