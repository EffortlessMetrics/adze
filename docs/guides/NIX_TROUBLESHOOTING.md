# Nix Development Environment - Troubleshooting Guide

**Purpose**: Comprehensive troubleshooting for Nix-related issues in rust-sitter
**Audience**: Contributors experiencing Nix setup or runtime issues
**Reference**: [NIX_QUICKSTART.md](./NIX_QUICKSTART.md)

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Flake and Configuration Issues](#flake-and-configuration-issues)
3. [Build and Compilation Issues](#build-and-compilation-issues)
4. [Test Failures](#test-failures)
5. [Performance Issues](#performance-issues)
6. [Platform-Specific Issues](#platform-specific-issues)
7. [IDE Integration Issues](#ide-integration-issues)
8. [Cache and Storage Issues](#cache-and-storage-issues)
9. [Getting Help](#getting-help)

---

## Installation Issues

### "command not found: nix"

**Symptoms**:
```bash
$ nix develop
bash: nix: command not found
```

**Causes**:
- Nix not installed
- Nix not in PATH
- Shell not restarted after installation

**Solutions**:

1. **Verify Nix is installed**:
   ```bash
   ls -la /nix/var/nix/profiles/default/bin/nix
   ```
   If this file doesn't exist, Nix isn't installed.

2. **Reinstall Nix**:
   ```bash
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

3. **Manually source Nix**:
   ```bash
   # For single-user installation
   source ~/.nix-profile/etc/profile.d/nix.sh

   # For multi-user installation
   source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
   ```

4. **Restart shell**:
   ```bash
   exec $SHELL
   ```

5. **Check PATH**:
   ```bash
   echo $PATH | grep nix
   # Should show something like: /nix/var/nix/profiles/default/bin
   ```

---

### "error: experimental features 'nix-command' and 'flakes' are not enabled"

**Symptoms**:
```bash
$ nix develop
error: experimental feature 'nix-command' is disabled; ...
```

**Cause**: Flakes not enabled in Nix configuration

**Solution**:

1. **Enable flakes**:
   ```bash
   mkdir -p ~/.config/nix
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

2. **Verify configuration**:
   ```bash
   cat ~/.config/nix/nix.conf
   # Should contain: experimental-features = nix-command flakes
   ```

3. **Alternative: Use environment variable**:
   ```bash
   # Temporary (current session only)
   export NIX_CONFIG="experimental-features = nix-command flakes"

   # Permanent (add to ~/.bashrc or ~/.zshrc)
   echo 'export NIX_CONFIG="experimental-features = nix-command flakes"' >> ~/.bashrc
   ```

---

### "error: opening lock file '/nix/var/nix/db/big-lock': Permission denied"

**Symptoms**:
```bash
$ nix develop
error: opening lock file '/nix/var/nix/db/big-lock': Permission denied
```

**Cause**: Incorrect Nix installation mode or permissions issue

**Solutions**:

1. **Install with daemon mode** (recommended):
   ```bash
   # Uninstall current Nix
   sudo rm -rf /nix

   # Reinstall with daemon
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

2. **Fix permissions** (if daemon mode not possible):
   ```bash
   sudo chown -R $(whoami) /nix
   ```

3. **Use single-user mode**:
   ```bash
   sh <(curl -L https://nixos.org/nix/install) --no-daemon
   ```

---

## Flake and Configuration Issues

### "error: getting status of '/nix/store/...': No such file or directory"

**Symptoms**:
```bash
$ nix develop
error: getting status of '/nix/store/xyz-rust-1.89.0': No such file or directory
```

**Cause**: Nix store corruption or incomplete download

**Solutions**:

1. **Clear Nix cache and retry**:
   ```bash
   # Clear evaluation cache
   rm -rf ~/.cache/nix

   # Retry
   nix develop
   ```

2. **Garbage collect and rebuild**:
   ```bash
   # Remove unused store paths
   nix-collect-garbage -d

   # Retry
   nix develop
   ```

3. **Verify store integrity**:
   ```bash
   nix-store --verify --check-contents
   ```

4. **Force re-download**:
   ```bash
   # Update flake inputs
   nix flake update

   # Rebuild shell
   nix develop --recreate-lock-file
   ```

---

### "warning: Git tree '/path/to/rust-sitter' is dirty"

**Symptoms**:
```bash
$ nix develop
warning: Git tree '/home/user/rust-sitter' is dirty
```

**Cause**: Uncommitted changes in git repository

**Impact**: This is a **warning**, not an error. Nix will still work.

**Solutions**:

1. **Commit your changes** (recommended):
   ```bash
   git add .
   git commit -m "WIP: testing Nix shell"
   ```

2. **Ignore the warning** (if testing):
   The warning is informational and doesn't affect functionality.

3. **Use --impure** (not recommended for reproducibility):
   ```bash
   nix develop --impure
   ```

---

### "error: flake 'git+file:///path' does not provide attribute 'devShells.x86_64-linux.default'"

**Symptoms**:
```bash
$ nix develop
error: flake 'git+file://...' does not provide attribute 'devShells.x86_64-linux.default'
```

**Cause**: Invalid flake.nix or missing outputs

**Solutions**:

1. **Check flake syntax**:
   ```bash
   nix flake check
   ```

2. **Show available outputs**:
   ```bash
   nix flake show
   ```

3. **Verify flake.nix structure**:
   ```nix
   outputs = { self, nixpkgs, flake-utils, ... }:
     flake-utils.lib.eachDefaultSystem (system:
       let pkgs = import nixpkgs { inherit system; };
       in {
         devShells.default = pkgs.mkShell { ... };
       });
   ```

4. **Reset to known-good version**:
   ```bash
   git checkout main -- flake.nix
   nix develop
   ```

---

## Build and Compilation Issues

### "error: builder for '...' failed with exit code 101"

**Symptoms**:
```bash
$ nix develop --command cargo build
error: builder for '/nix/store/xyz-rust-sitter-0.6.1' failed with exit code 101
```

**Cause**: Compilation error in Rust code

**Solutions**:

1. **Check Rust compiler output**:
   ```bash
   nix develop
   cargo build 2>&1 | less
   # Look for actual compilation errors
   ```

2. **Clean build artifacts**:
   ```bash
   nix develop
   cargo clean
   cargo build
   ```

3. **Verify Rust version**:
   ```bash
   nix develop --command rustc --version
   # Should match rust-toolchain.toml (1.89+)
   ```

4. **Check for missing dependencies**:
   ```bash
   nix develop --command bash -c '
     pkg-config --list-all | grep -E "(tree-sitter|clang|llvm)"
   '
   ```

---

### "error: linking with `cc` failed: exit code: 1"

**Symptoms**:
```bash
$ cargo build
error: linking with `cc` failed: exit status: 1
ld: library not found for -lz
```

**Cause**: Missing system libraries in Nix shell

**Solutions**:

1. **Verify you're in Nix shell**:
   ```bash
   echo $IN_NIX_SHELL
   # Should output: impure
   ```

2. **Check library availability**:
   ```bash
   nix develop --command bash -c '
     find $NIX_STORE -name "libz.so" -o -name "libz.dylib" 2>/dev/null | head -5
   '
   ```

3. **Add missing library to flake.nix**:
   ```nix
   buildInputs = [
     # ... existing packages ...
     pkgs.zlib  # Add missing library
   ];
   ```

4. **Verify pkg-config paths**:
   ```bash
   nix develop --command bash -c 'echo $PKG_CONFIG_PATH'
   ```

---

### "error: rustup could not choose a version of rustc to run"

**Symptoms**:
```bash
$ cargo build
error: rustup could not choose a version of rustc to run
```

**Cause**: rust-toolchain.toml not respected or rustup not initialized

**Solutions**:

1. **Initialize rustup in shell**:
   ```bash
   nix develop --command bash -c '
     rustup show
   '
   ```

2. **Verify rust-toolchain.toml**:
   ```bash
   cat rust-toolchain.toml
   # Should specify channel = "1.89" or similar
   ```

3. **Manually install toolchain**:
   ```bash
   nix develop
   rustup toolchain install stable
   rustup default stable
   ```

---

## Test Failures

### "test failed: Too many open files"

**Symptoms**:
```bash
$ just ci-test
thread 'main' panicked at 'failed to create file: Os { code: 24, kind: Other, message: "Too many open files" }'
```

**Cause**: File descriptor limit too low

**Solutions**:

1. **Use safe mode**:
   ```bash
   just ci-test-safe
   ```

2. **Increase ulimit temporarily**:
   ```bash
   ulimit -n 4096
   cargo test --workspace
   ```

3. **Set ulimit permanently** (Linux):
   ```bash
   echo "* soft nofile 4096" | sudo tee -a /etc/security/limits.conf
   echo "* hard nofile 8192" | sudo tee -a /etc/security/limits.conf
   # Logout and login for changes to take effect
   ```

4. **Use ultra-safe mode**:
   ```bash
   RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 cargo test --workspace -- --test-threads=1
   ```

---

### "test failed: Cannot create thread"

**Symptoms**:
```bash
$ cargo test
thread 'main' panicked at 'failed to spawn thread: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }'
```

**Cause**: Thread limit exceeded or concurrency cap too high

**Solutions**:

1. **Verify concurrency settings**:
   ```bash
   nix develop --command bash -c '
     echo "RUST_TEST_THREADS: $RUST_TEST_THREADS"
     echo "RAYON_NUM_THREADS: $RAYON_NUM_THREADS"
   '
   # Should show: RUST_TEST_THREADS=2, RAYON_NUM_THREADS=4
   ```

2. **Use minimal concurrency**:
   ```bash
   just ci-test-safe
   ```

3. **Check system limits**:
   ```bash
   ulimit -u  # Max user processes
   ```

---

### "test timed out after 60 seconds"

**Symptoms**:
```bash
$ cargo test
test glr_core::tests::test_large_input ... FAILED (timeout)
```

**Cause**: Test genuinely slow or infinite loop

**Solutions**:

1. **Increase test timeout**:
   ```bash
   RUST_TEST_TIMEOUT=300 cargo test
   ```

2. **Run test in isolation**:
   ```bash
   cargo test --test test_glr_core test_large_input -- --nocapture
   ```

3. **Use nextest** (better timeout handling):
   ```bash
   cargo nextest run
   ```

4. **Check for infinite loops**:
   ```bash
   # Run with debug output
   RUST_LOG=trace cargo test test_large_input -- --nocapture
   ```

---

## Performance Issues

### "nix develop is very slow (>5 minutes)"

**Symptoms**: First `nix develop` takes a long time

**Cause**: Expected behavior - downloading and building dependencies

**Solutions**:

1. **Use binary cache** (cachix):
   ```bash
   # Install cachix
   nix-env -iA cachix -f https://cachix.org/api/v1/install

   # Use rust-sitter cache
   cachix use rust-sitter

   # Retry
   nix develop
   ```

2. **Check network speed**:
   ```bash
   # Test Nix binary cache
   time nix-shell -p hello
   ```

3. **Monitor progress**:
   ```bash
   nix develop --verbose
   ```

4. **Subsequent runs should be instant** (cached)

---

### "cargo build is slower in Nix shell"

**Symptoms**: Compilation slower than traditional setup

**Cause**: Possibly rebuilding dependencies or cache issues

**Solutions**:

1. **Verify no rebuilds**:
   ```bash
   cargo build --timings
   # Check generated report for unnecessary rebuilds
   ```

2. **Check disk I/O**:
   ```bash
   # Monitor during build
   iostat -x 1
   ```

3. **Use ramdisk for target** (Linux):
   ```bash
   # Create ramdisk
   sudo mkdir -p /mnt/ramdisk
   sudo mount -t tmpfs -o size=4G tmpfs /mnt/ramdisk

   # Build to ramdisk
   CARGO_TARGET_DIR=/mnt/ramdisk/target cargo build
   ```

4. **Comparison test**:
   ```bash
   # Time Nix build
   time nix develop --command cargo build --release

   # Time traditional build (exit Nix first)
   exit
   time cargo build --release
   ```

---

## Platform-Specific Issues

### macOS: "xcrun: error: invalid active developer path"

**Symptoms**:
```bash
$ nix develop
xcrun: error: invalid active developer path (/Library/Developer/CommandLineTools)
```

**Cause**: Xcode Command Line Tools not installed

**Solution**:
```bash
xcode-select --install
```

---

### macOS M1/M2: "error: cannot execute binary file"

**Symptoms**:
```bash
$ nix develop
error: cannot execute binary file: Exec format error
```

**Cause**: Rosetta 2 not installed (needed for x86_64 binaries)

**Solution**:
```bash
softwareupdate --install-rosetta
```

---

### Windows (WSL): "mount: /mnt/wslg: cannot mount none"

**Symptoms**: WSL display issues preventing Nix installation

**Solution**:

1. **Update WSL**:
   ```powershell
   wsl --update
   ```

2. **Use WSL2** (not WSL1):
   ```powershell
   wsl --set-default-version 2
   ```

3. **Restart WSL**:
   ```powershell
   wsl --shutdown
   wsl
   ```

---

### Linux: "error: unable to start build hook"

**Symptoms**:
```bash
$ nix develop
error: unable to start build hook
```

**Cause**: Nix daemon not running (multi-user installation)

**Solution**:

1. **Start Nix daemon**:
   ```bash
   sudo systemctl start nix-daemon
   sudo systemctl enable nix-daemon  # Start on boot
   ```

2. **Check daemon status**:
   ```bash
   sudo systemctl status nix-daemon
   ```

3. **Switch to single-user mode** (alternative):
   ```bash
   sudo rm -rf /nix
   sh <(curl -L https://nixos.org/nix/install) --no-daemon
   ```

---

## IDE Integration Issues

### VS Code: "rust-analyzer failed to load workspace"

**Symptoms**: rust-analyzer doesn't work in VS Code

**Cause**: rust-analyzer not using Nix environment

**Solutions**:

1. **Use direnv** (recommended):
   ```bash
   # Install direnv
   nix-env -i direnv

   # Create .envrc
   cd rust-sitter
   echo "use flake" > .envrc
   direnv allow

   # Install VS Code direnv extension
   code --install-extension mkhl.direnv
   ```

2. **Launch VS Code from Nix shell**:
   ```bash
   nix develop
   code .
   ```

3. **Configure rust-analyzer settings.json**:
   ```json
   {
     "rust-analyzer.server.extraEnv": {
       "NIX_PATH": "nixpkgs=/nix/var/nix/profiles/per-user/root/channels/nixos"
     }
   }
   ```

---

### IntelliJ / CLion: "Cargo project update failed"

**Symptoms**: IntelliJ Rust plugin can't find Cargo

**Cause**: IntelliJ not using Nix environment

**Solutions**:

1. **Launch from Nix shell**:
   ```bash
   nix develop
   idea .  # or clion .
   ```

2. **Configure Rust plugin**:
   - Settings → Languages & Frameworks → Rust
   - Set toolchain location to: `~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu`

3. **Use nix-idea plugin**:
   - Install "Nix Idea" plugin from marketplace
   - Configure to use flake.nix

---

## Cache and Storage Issues

### "error: cannot link '/nix/store/...': No space left on device"

**Symptoms**: Nix store full, can't download more packages

**Cause**: `/nix` partition full

**Solutions**:

1. **Check disk space**:
   ```bash
   df -h /nix
   ```

2. **Garbage collect**:
   ```bash
   # Remove old generations
   nix-collect-garbage

   # Aggressive: remove all generations except current
   nix-collect-garbage -d

   # Verify space freed
   df -h /nix
   ```

3. **Optimize store**:
   ```bash
   nix-store --optimize
   ```

4. **Increase partition size** (if needed)

---

### "warning: ignoring the client-specified setting 'experimental-features'"

**Symptoms**: Warning during `nix develop`

**Cause**: Nix daemon has different config than user

**Impact**: Warning only, doesn't affect functionality

**Solutions**:

1. **Ignore** (safe to do so)

2. **Configure daemon** (if it bothers you):
   ```bash
   sudo mkdir -p /etc/nix
   echo "experimental-features = nix-command flakes" | sudo tee /etc/nix/nix.conf
   sudo systemctl restart nix-daemon
   ```

---

## Getting Help

### Self-Diagnosis

Run the diagnostic script:

```bash
# Check all common issues
nix develop --command bash -c '
  echo "=== Environment Check ==="
  echo "Nix version: $(nix --version)"
  echo "Rust version: $(rustc --version)"
  echo "Cargo version: $(cargo --version)"
  echo "Just version: $(just --version)"
  echo ""
  echo "=== Environment Variables ==="
  echo "RUST_TEST_THREADS: $RUST_TEST_THREADS"
  echo "RAYON_NUM_THREADS: $RAYON_NUM_THREADS"
  echo "IN_NIX_SHELL: $IN_NIX_SHELL"
  echo ""
  echo "=== System Resources ==="
  echo "File descriptor limit: $(ulimit -n)"
  echo "Process limit: $(ulimit -u)"
  echo "Available disk space: $(df -h /nix | tail -1)"
  echo ""
  echo "=== Flake Check ==="
  nix flake check 2>&1 || echo "Flake check failed"
'
```

---

### Community Support

1. **GitHub Issues**: [rust-sitter/issues](https://github.com/EffortlessMetrics/rust-sitter/issues)
   - Include output of diagnostic script above
   - Tag with `nix` label

2. **GitHub Discussions**: [rust-sitter/discussions](https://github.com/EffortlessMetrics/rust-sitter/discussions)
   - Q&A for general Nix questions

3. **Nix Community**:
   - [NixOS Discourse](https://discourse.nixos.org/)
   - [Nix Matrix Channel](https://matrix.to/#/#nix:nixos.org)

---

### Reporting Bugs

When reporting Nix-related issues, include:

1. **Environment info**:
   ```bash
   nix --version
   uname -a
   echo $SHELL
   ```

2. **Flake info**:
   ```bash
   nix flake metadata
   cat flake.lock | head -20
   ```

3. **Diagnostic output** (from Self-Diagnosis section)

4. **Reproduction steps**:
   ```
   1. nix develop
   2. just ci-all
   3. Error: ...
   ```

5. **Expected vs Actual behavior**

---

## Appendix: Common Error Codes

| Exit Code | Meaning | Common Cause |
|-----------|---------|--------------|
| 1 | Generic error | Build failure, test failure |
| 2 | Misuse of shell builtin | Syntax error in script |
| 101 | Builder failed | Rust compilation error |
| 124 | Command timed out | Test timeout |
| 126 | Command not executable | Permission issue |
| 127 | Command not found | Binary not in PATH |

---

**Guide Version**: 1.0.0
**Last Updated**: 2025-11-20
**Maintained By**: rust-sitter core team

For setup instructions, see [Nix Quickstart Guide](./NIX_QUICKSTART.md).

---

END OF TROUBLESHOOTING GUIDE
