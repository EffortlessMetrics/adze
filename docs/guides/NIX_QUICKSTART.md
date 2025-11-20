# Nix Development Environment - Quickstart Guide

**Target Audience**: Contributors who want to set up a reproducible development environment
**Time to Complete**: 5-10 minutes
**Prerequisites**: None (Nix installation included)

---

## Why Nix?

Rust-sitter uses Nix to provide a **reproducible development environment** that matches CI exactly:

✅ **One Command Setup**: `nix develop` and you're ready
✅ **Identical Environment**: Same Rust, libraries, and tools everywhere
✅ **No Conflicts**: Isolated from your system packages
✅ **Fast Onboarding**: New contributors productive immediately

**Key Benefit**: If `just ci-all` passes locally, it will pass in CI (and vice versa).

---

## Quick Setup (5 Minutes)

### Step 1: Install Nix

**Linux / macOS**:
```bash
# Install Nix (multi-user installation, recommended)
sh <(curl -L https://nixos.org/nix/install) --daemon

# Or single-user installation (simpler, but less isolated)
sh <(curl -L https://nixos.org/nix/install) --no-daemon
```

**Windows**:
```powershell
# Install WSL2 first (if not already installed)
wsl --install

# Then install Nix inside WSL
sh <(curl -L https://nixos.org/nix/install) --no-daemon
```

### Step 2: Enable Flakes

```bash
# Create Nix config directory
mkdir -p ~/.config/nix

# Enable experimental features
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Restart your shell to apply changes
exec $SHELL
```

### Step 3: Enter the Development Shell

```bash
# Clone rust-sitter (if not already done)
git clone https://github.com/EffortlessMetrics/rust-sitter.git
cd rust-sitter

# Enter the development environment
nix develop

# You should see: "🦀 rust-sitter development environment ready!"
```

That's it! You now have:
- Rust 1.89+ with the correct toolchain
- All system dependencies (clang, cmake, etc.)
- All Rust tools (cargo-nextest, cargo-insta, just)
- All environment variables configured correctly

---

## Running CI Locally

Once in the Nix shell, you can run the exact same commands as CI:

### Full CI Suite

```bash
# Run all CI checks (formatting, linting, tests, docs)
just ci-all
```

This runs:
1. `cargo fmt --all -- --check` (formatting)
2. `cargo clippy --workspace --all-targets -- -D warnings` (linting)
3. `cargo test --workspace -- --test-threads=2` (tests)
4. `cargo doc --no-deps --workspace` (documentation)

### Individual Checks

```bash
# Check formatting only
just ci-fmt

# Run clippy only
just ci-clippy

# Run tests only
just ci-test

# Check documentation builds
just ci-doc

# Run performance benchmarks
just ci-perf

# Run GLR-specific tests
just ci-glr
```

### Safe Testing

```bash
# Run tests with ultra-safe concurrency (single thread)
just ci-test-safe

# Run tests with custom thread count
RUST_TEST_THREADS=1 cargo test --workspace -- --test-threads=1
```

---

## Running Without Entering the Shell

You can run commands directly without entering the shell:

```bash
# Run CI suite
nix develop . --command just ci-all

# Run tests
nix develop . --command just ci-test

# Run any cargo command
nix develop . --command cargo build --release
```

---

## Performance Shell (Optional)

For performance profiling and benchmarking:

```bash
# Enter performance shell (includes flamegraph, heaptrack, valgrind)
nix develop .#perf

# Run benchmarks
just ci-perf

# Generate flamegraph
cargo flamegraph --bench glr_hot
```

---

## CI Shell (Optional)

Minimal shell for CI-like testing:

```bash
# Enter minimal CI shell
nix develop .#ci

# Run CI suite
just ci-all
```

---

## Troubleshooting

### Issue: "command not found: nix"

**Solution**: Restart your shell after installation:
```bash
exec $SHELL
```

If still not working, check your PATH:
```bash
echo $PATH | grep -q nix && echo "Nix in PATH" || echo "Nix NOT in PATH"

# Manually source Nix (if needed)
source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
```

### Issue: "experimental-features 'nix-command' and 'flakes' are not enabled"

**Solution**: Enable flakes in your Nix config:
```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### Issue: "error: opening lock file '/nix/var/nix/db/big-lock'"

**Solution**: This is a permissions issue. Either:

1. Install Nix with daemon mode (recommended):
   ```bash
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

2. Or fix permissions:
   ```bash
   sudo chown -R $(whoami) /nix
   ```

### Issue: "nix develop" is slow the first time

**Expected Behavior**: The first run downloads all dependencies and may take 5-10 minutes. Subsequent runs are instant due to caching.

**Speed up**: Use cachix (binary cache):
```bash
# Install cachix
nix-env -iA cachix -f https://cachix.org/api/v1/install

# Use rust-sitter cache
cachix use rust-sitter
```

### Issue: Tests fail with "Too many open files"

**Solution**: This is usually handled automatically by the Nix shell, but if it persists:

```bash
# Check current limits
ulimit -n

# Increase file descriptor limit (temporary)
ulimit -n 4096

# Or use ultra-safe mode
just ci-test-safe
```

### Issue: Build failures on macOS

**Common Causes**:
1. Rosetta 2 not installed (M1/M2 Macs):
   ```bash
   softwareupdate --install-rosetta
   ```

2. Xcode Command Line Tools missing:
   ```bash
   xcode-select --install
   ```

---

## Verification

To verify your Nix setup is working correctly:

```bash
# 1. Check Nix is installed
nix --version
# Should show: nix (Nix) 2.x.x

# 2. Check flake is valid
nix flake check
# Should show: no errors

# 3. Enter shell and check environment
nix develop .#default --command bash -c '
  echo "Rust: $(rustc --version)"
  echo "Cargo: $(cargo --version)"
  echo "Just: $(just --version)"
  echo "RUST_TEST_THREADS: $RUST_TEST_THREADS"
'

# 4. Run a simple test
nix develop .#default --command cargo test -p rust-sitter-ir --lib
```

If all four steps pass, your Nix environment is correctly configured! 🎉

---

## Next Steps

- **Read**: [Nix Troubleshooting Guide](./NIX_TROUBLESHOOTING.md) for common issues
- **Read**: [Migrating to Nix](./MIGRATING_TO_NIX.md) if you have an existing setup
- **Read**: [ADR-0008](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md) for design rationale
- **Contribute**: Run `just ci-all` before opening PRs to ensure CI will pass

---

## Comparison: Traditional vs Nix Setup

| Aspect | Traditional Setup | Nix Setup |
|--------|------------------|-----------|
| **Setup Time** | 30-60 minutes | 5-10 minutes |
| **Dependencies** | Manual installation | Automatic |
| **Environment Isolation** | No | Yes |
| **CI Parity** | ~80% match | 100% match |
| **Cross-Platform** | Platform-specific | Consistent |
| **Reproducibility** | No guarantees | Fully reproducible |
| **Updates** | Manual | `nix flake update` |

---

## FAQs

### Do I need to use Nix?

**No**, Nix is optional for contributors. You can still use the traditional setup documented in CLAUDE.md. However:
- CI uses Nix exclusively
- Using Nix guarantees your local results match CI
- Nix makes setup much faster and easier

### Can I use my existing Rust installation?

**Yes**, but it's not recommended. The Nix shell respects `rust-toolchain.toml`, so you'll automatically get the correct Rust version without conflicting with your system installation.

### Does Nix slow down builds?

**No**. After the initial setup, Nix has zero runtime overhead. Builds run at native speed.

### Can I use Nix with my IDE (VS Code, IntelliJ, etc.)?

**Yes**. Two options:

1. **Use direnv** (automatic):
   ```bash
   # Install direnv
   nix-env -i direnv

   # Allow direnv in project
   cd rust-sitter
   echo "use flake" > .envrc
   direnv allow
   ```

   Now your IDE automatically uses the Nix environment!

2. **Launch IDE from shell** (manual):
   ```bash
   nix develop
   code .  # VS Code
   # or
   idea .  # IntelliJ
   ```

### How do I update dependencies?

```bash
# Update flake.lock (updates all Nix dependencies)
nix flake update

# Update specific input
nix flake update nixpkgs

# Check what changed
git diff flake.lock
```

### Can I add custom tools to the shell?

**Yes**. Edit `flake.nix`:

```nix
buildInputs = [
  # ... existing packages ...
  pkgs.yourPackage
];
```

Then reload the shell:
```bash
exit  # Exit current shell
nix develop  # Re-enter with new packages
```

---

**Guide Version**: 1.0.0
**Last Updated**: 2025-11-20
**Maintained By**: rust-sitter core team

For issues or questions, see the [Troubleshooting Guide](./NIX_TROUBLESHOOTING.md).

---

END OF QUICKSTART GUIDE
