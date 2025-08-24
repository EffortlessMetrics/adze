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

# Clean
clean:
	cargo clean

# Help
help:
	@echo "Available targets:"
	@echo "  lint       - Run full lint suite"
	@echo "  lint-fast  - Run fast lint on changed files only"
	@echo "  build      - Build debug"
	@echo "  release    - Build release"
	@echo "  test       - Run tests"
	@echo "  test-all   - Run all workspace tests with all features"
	@echo "  clean      - Clean build artifacts"