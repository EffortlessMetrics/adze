#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <new-version> [old-version]" >&2
  exit 1
fi

VERSION="$1"
OLD_VERSION="${2:-}"

VERSION="${VERSION#v}"
if [[ -z "$VERSION" ]]; then
  echo "New version cannot be empty." >&2
  exit 1
fi

if [[ $# -eq 2 && -z "$OLD_VERSION" ]]; then
  echo "Old version cannot be empty." >&2
  exit 1
fi

if [[ -z "$OLD_VERSION" ]]; then
  OLD_VERSION="$(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[] | select(.name == "adze") | .version' | head -n 1)"
else
  OLD_VERSION="${OLD_VERSION#v}"
fi

if [[ -z "$OLD_VERSION" ]]; then
  echo "Could not determine current version. Pass old version as second argument." >&2
  exit 1
fi

echo "Updating all crate versions from $OLD_VERSION to $VERSION"

# Update workspace members
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" runtime/Cargo.toml
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" macro/Cargo.toml
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" tool/Cargo.toml
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" common/Cargo.toml
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" ir/Cargo.toml
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" glr-core/Cargo.toml
sed -i "s/version = \"$OLD_VERSION\"/version = \"$VERSION\"/g" tablegen/Cargo.toml

# Update internal dependencies
sed -i "s/adze-macro = { version = \"$OLD_VERSION\"/adze-macro = { version = \"$VERSION\"/g" runtime/Cargo.toml
sed -i "s/adze-common = { version = \"$OLD_VERSION\"/adze-common = { version = \"$VERSION\"/g" macro/Cargo.toml
sed -i "s/adze-common = { version = \"$OLD_VERSION\"/adze-common = { version = \"$VERSION\"/g" tool/Cargo.toml
sed -i "s/adze-ir = { version = \"$OLD_VERSION\"/adze-ir = { version = \"$VERSION\"/g" glr-core/Cargo.toml
sed -i "s/adze-ir = { version = \"$OLD_VERSION\"/adze-ir = { version = \"$VERSION\"/g" tablegen/Cargo.toml
sed -i "s/adze-glr-core = { version = \"$OLD_VERSION\"/adze-glr-core = { version = \"$VERSION\"/g" tablegen/Cargo.toml
sed -i "s/adze = { version = \"$OLD_VERSION\"/adze = { version = \"$VERSION\"/g" example/Cargo.toml
sed -i "s/adze-tool = { version = \"$OLD_VERSION\"/adze-tool = { version = \"$VERSION\"/g" example/Cargo.toml

echo "Version update complete!"
echo
