#!/bin/bash
# Script to update all crate versions for beta release

VERSION="0.5.0-beta"

echo "Updating all crate versions to $VERSION"

# Update workspace members
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" runtime/Cargo.toml
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" macro/Cargo.toml
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" tool/Cargo.toml
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" common/Cargo.toml
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" ir/Cargo.toml
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" glr-core/Cargo.toml
sed -i "s/version = \"0.4.5\"/version = \"$VERSION\"/g" tablegen/Cargo.toml

# Update internal dependencies
sed -i "s/rust-sitter-macro = { version = \"0.4.5\"/rust-sitter-macro = { version = \"$VERSION\"/g" runtime/Cargo.toml
sed -i "s/rust-sitter-common = { version = \"0.4.5\"/rust-sitter-common = { version = \"$VERSION\"/g" macro/Cargo.toml
sed -i "s/rust-sitter-common = { version = \"0.4.5\"/rust-sitter-common = { version = \"$VERSION\"/g" tool/Cargo.toml
sed -i "s/rust-sitter = { version = \"0.4.5\"/rust-sitter = { version = \"$VERSION\"/g" example/Cargo.toml
sed -i "s/rust-sitter-tool = { version = \"0.4.5\"/rust-sitter-tool = { version = \"$VERSION\"/g" example/Cargo.toml

echo "Version update complete!"