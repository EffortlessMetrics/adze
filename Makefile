.PHONY: lint lint-fast test build clean

# Quick lint commands
lint:
	cargo xtask lint

lint-fast:
	cargo xtask lint --fast --changed-only

# Build commands
build:
	cargo build

release:
	cargo build --release

# Test commands
test:
	cargo test

test-all:
	cargo test --workspace --all-features

# Test with debug assertions enabled
test-asserts:
	RUSTFLAGS='-C debug-assertions' cargo test --workspace

# Test GLR core with debug assertions
test-glr-asserts:
	RUSTFLAGS='-C debug-assertions' cargo test -p adze-glr-core

# Run `just` with tmpdir workaround for permission errors
just-%:
	@source scripts/just-ensure-tmpdir.sh && just $*

# Clean
clean:
	cargo clean

# Help
help:
	@echo "Available targets:"
	@echo "  lint            - Run full lint suite"
	@echo "  lint-fast       - Run fast lint on changed files only"
	@echo "  build           - Build debug"
	@echo "  release         - Build release"
	@echo "  test            - Run tests"
	@echo "  test-all        - Run all workspace tests with all features"
	@echo "  test-asserts    - Run all tests with debug assertions enabled"
	@echo "  test-glr-asserts - Run GLR core tests with debug assertions"
	@echo "  clean           - Clean build artifacts"