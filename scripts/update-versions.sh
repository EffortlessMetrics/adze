#!/bin/bash
# Script to update all crate versions

OLD_VERSION="0.5.0-beta"
VERSION="1.0.0"

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