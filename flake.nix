{
  description = "rust-sitter dev + CI environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ ];
        };

        # Use rustup to respect rust-toolchain.toml
        rustToolchain = pkgs.rustup;
      in {
        devShells.default = pkgs.mkShell {
          name = "rust-sitter-dev";

          buildInputs = [
            # Rust toolchain (respects rust-toolchain.toml)
            rustToolchain

            # Core Rust tooling
            pkgs.cargo-nextest
            pkgs.cargo-insta
            pkgs.just

            # Quality automation (Policy-as-Code)
            pkgs.pre-commit

            # C / system deps for tree-sitter-c2rust, cc, etc.
            pkgs.clang
            pkgs.llvmPackages.bintools
            pkgs.pkg-config
            pkgs.cmake
            pkgs.gnumake
            pkgs.openssl
            pkgs.zlib

            # Scripting / generators
            pkgs.python3
            pkgs.nodejs

            # CI helpers
            pkgs.git
            pkgs.bashInteractive
          ];

          # Environment variables for reproducible builds and testing
          RUST_BACKTRACE = "1";
          RUST_TEST_THREADS = "2";  # Concurrency cap for stable tests
          RAYON_NUM_THREADS = "4";  # Rayon thread pool limit
          TOKIO_WORKER_THREADS = "2";  # Tokio async runtime limit
          TOKIO_BLOCKING_THREADS = "8";  # Tokio blocking pool limit
          CARGO_BUILD_JOBS = "4";  # Cargo parallel build limit

          # Shell hook for setup
          shellHook = ''
            # Ensure Rust toolchain is installed
            if [ -f rust-toolchain.toml ] || [ -f rust-toolchain ]; then
              rustup show >/dev/null 2>&1 || rustup toolchain install
            fi

            # Install pre-commit hooks automatically (Policy-as-Code)
            if [ -f .pre-commit-config.yaml ]; then
              if pre-commit install --install-hooks >/dev/null 2>&1; then
                echo "✅ Pre-commit hooks installed"
              else
                echo "⚠️  Pre-commit hooks installation failed (non-fatal)"
              fi
              # Also install commit-msg hook for message validation
              pre-commit install --hook-type commit-msg >/dev/null 2>&1 || true
            fi

            # Show environment info
            echo "🦀 rust-sitter development environment ready!"
            echo ""
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "Test threads: $RUST_TEST_THREADS"
            echo ""
            echo "Common commands:"
            echo "  just ci-all       - Run full CI suite locally"
            echo "  just ci-test      - Run tests only"
            echo "  just ci-perf      - Run performance benchmarks"
            echo "  just help         - Show all available commands"
            echo ""
            echo "Quality automation:"
            echo "  Pre-commit hooks: Installed (run on git commit)"
            echo "  Manual run:       pre-commit run --all-files"
            echo "  Bypass (WIP):     git commit --no-verify"
            echo ""
          '';
        };

        # Optional: Separate CI shell with minimal dependencies
        devShells.ci = pkgs.mkShell {
          name = "rust-sitter-ci";

          buildInputs = [
            rustToolchain
            pkgs.just
            pkgs.clang
            pkgs.pkg-config
            pkgs.cmake
          ];

          RUST_BACKTRACE = "1";
          RUST_TEST_THREADS = "2";
          RAYON_NUM_THREADS = "4";
        };

        # Optional: Performance benchmarking shell
        devShells.perf = pkgs.mkShell {
          name = "rust-sitter-perf";

          buildInputs = [
            rustToolchain
            pkgs.just
            pkgs.clang
            pkgs.pkg-config
            pkgs.flamegraph
            pkgs.heaptrack
            pkgs.valgrind
          ];

          RUST_BACKTRACE = "1";
          # Performance shell uses more threads
          RUST_TEST_THREADS = "8";
          RAYON_NUM_THREADS = "8";
        };
      });
}
