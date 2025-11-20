# Migrating to Nix Development Environment

**Target Audience**: Existing rust-sitter contributors with traditional setup
**Time to Complete**: 15-30 minutes
**Prerequisites**: Existing rust-sitter development environment

---

## Why Migrate?

**Current Pain Points** with traditional setup:
- ❌ "Works on my machine" - CI environment differs from local
- ❌ Manual dependency management - multiple steps to install tools
- ❌ Platform inconsistencies - different behavior on Linux vs macOS vs Windows
- ❌ Difficult onboarding - 30-60 minute setup for new contributors

**Benefits of Nix**:
- ✅ **CI Parity**: Local environment = CI environment (100% match)
- ✅ **Fast Setup**: 5-10 minute setup, fully automated
- ✅ **Reproducible**: Same results every time, everywhere
- ✅ **Isolated**: No conflicts with system packages

---

## Migration Strategies

Choose the strategy that fits your situation:

### Strategy 1: Side-by-Side (Recommended)

**Best For**: You want to try Nix while keeping your existing setup

**Approach**: Install Nix alongside your current tooling, use it selectively

**Pros**:
- No risk - existing setup unchanged
- Easy to compare Nix vs traditional
- Can switch back anytime

**Cons**:
- Slightly more disk space used
- Need to choose which environment to use

**Time**: 15 minutes

---

### Strategy 2: Clean Switch

**Best For**: You're confident in Nix and want to clean up your system

**Approach**: Uninstall traditional tools, go all-in on Nix

**Pros**:
- Clean system - no duplicate tools
- Forces adoption of best practices
- Simplest mental model

**Cons**:
- Can't easily go back
- Need to uninstall existing tools

**Time**: 30 minutes

---

### Strategy 3: Gradual Migration

**Best For**: Teams transitioning over time

**Approach**: Use Nix for CI, traditional locally during transition period

**Pros**:
- Smooth team transition
- CI benefits immediately
- Individuals migrate at own pace

**Cons**:
- Mixed environments during transition
- Documentation overhead

**Time**: Ongoing over 2-4 weeks

---

## Migration Path: Side-by-Side (Recommended)

### Step 1: Backup Your Current Setup

Before making any changes:

```bash
# Document current versions
rustc --version > ~/rust-sitter-traditional-versions.txt
cargo --version >> ~/rust-sitter-traditional-versions.txt
just --version >> ~/rust-sitter-traditional-versions.txt
echo "---" >> ~/rust-sitter-traditional-versions.txt
cargo test --workspace 2>&1 | grep "test result" >> ~/rust-sitter-traditional-versions.txt

# Save for comparison later
cat ~/rust-sitter-traditional-versions.txt
```

### Step 2: Install Nix (Without Removing Traditional Tools)

```bash
# Install Nix (daemon mode - most isolated)
sh <(curl -L https://nixos.org/nix/install) --daemon

# Enable flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Restart shell
exec $SHELL

# Verify Nix installed
nix --version
```

**Important**: This does NOT remove your existing Rust installation!

### Step 3: Test Nix Environment

```bash
cd /path/to/rust-sitter

# Enter Nix shell
nix develop

# You should see: "🦀 rust-sitter development environment ready!"

# Check versions in Nix shell
rustc --version
cargo --version
just --version

# Run tests in Nix environment
just ci-all

# Exit Nix shell
exit
```

### Step 4: Compare Results

```bash
# Outside Nix shell (traditional)
cargo test --workspace

# Inside Nix shell
nix develop --command cargo test --workspace

# Compare: both should pass all tests
```

### Step 5: Daily Workflow Decision

**Option A: Use Nix for CI-like work**
```bash
# For PR work (needs to match CI)
nix develop
just ci-all
exit

# For experimental work (use traditional)
cargo build
cargo test
```

**Option B: Use Nix exclusively**
```bash
# Always enter Nix shell
nix develop

# All work in Nix shell
just ci-all
cargo build
# ... etc
```

**Option C: Use direnv (automatic shell switching)**
```bash
# Install direnv
nix-env -i direnv

# Configure project
cd rust-sitter
echo "use flake" > .envrc
direnv allow

# Now Nix environment loads automatically when entering directory!
cd rust-sitter  # Shell automatically enters Nix environment
cd ..           # Shell automatically exits Nix environment
```

---

## Migration Path: Clean Switch

**Warning**: Only proceed if you're comfortable uninstalling your traditional tooling.

### Step 1: Document Current State

```bash
# Save current versions
rustc --version > ~/rust-sitter-before-nix.txt
cargo --version >> ~/rust-sitter-before-nix.txt
which rustc >> ~/rust-sitter-before-nix.txt
which cargo >> ~/rust-sitter-before-nix.txt

# Test current setup
cd /path/to/rust-sitter
cargo test --workspace 2>&1 | grep "test result" >> ~/rust-sitter-before-nix.txt
```

### Step 2: Uninstall Traditional Tools (Optional)

**Rust (via rustup)**:
```bash
# List installed toolchains
rustup toolchain list

# Uninstall rustup (this removes all Rust installations)
rustup self uninstall
```

**Homebrew tools (macOS)**:
```bash
# Only if you installed these system-wide
brew uninstall just
brew uninstall cargo-nextest
brew uninstall cargo-insta
```

**System packages (Linux)**:
```bash
# Ubuntu/Debian
sudo apt remove cargo rustc

# Arch
sudo pacman -R rust
```

### Step 3: Install Nix

```bash
# Install Nix
sh <(curl -L https://nixos.org/nix/install) --daemon

# Enable flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Restart shell
exec $SHELL
```

### Step 4: Verify Nix Setup

```bash
cd /path/to/rust-sitter

# Enter Nix shell
nix develop

# Verify tools available
rustc --version
cargo --version
just --version

# Run full CI suite
just ci-all
```

### Step 5: Update Shell Configuration

**Remove old tool configurations**:

```bash
# Edit ~/.bashrc, ~/.zshrc, or ~/.profile
# Remove lines like:
#   source "$HOME/.cargo/env"
#   export PATH="$HOME/.cargo/bin:$PATH"

# Restart shell
exec $SHELL
```

**Add Nix configuration** (optional):

```bash
# Add to ~/.bashrc or ~/.zshrc for automatic Nix shell
alias rs-dev='cd /path/to/rust-sitter && nix develop'
```

---

## Migration Path: Gradual (Team)

### Phase 1: CI Uses Nix (Week 1)

**Goal**: Get CI benefits immediately without requiring developer changes

**Actions**:
1. CI workflows updated to use Nix (already done ✅)
2. Developers continue using traditional setup
3. Document Nix as "optional" in CLAUDE.md

**Success Criteria**: All CI jobs passing with Nix

---

### Phase 2: Documentation and Training (Week 2)

**Goal**: Prepare team for migration

**Actions**:
1. Create Nix quickstart guide (this doc ✅)
2. Create troubleshooting guide (NIX_TROUBLESHOOTING.md ✅)
3. Team training session (1 hour)
4. Designate "Nix champion" for support

**Success Criteria**: All team members understand Nix benefits and process

---

### Phase 3: Early Adopters (Week 3)

**Goal**: 20-30% of team migrates

**Actions**:
1. Identify volunteers for early adoption
2. Provide hands-on support during migration
3. Collect feedback and address issues
4. Update documentation based on feedback

**Success Criteria**: 2-3 team members using Nix successfully

---

### Phase 4: General Migration (Week 4)

**Goal**: 80%+ of team using Nix

**Actions**:
1. Recommend Nix as default in documentation
2. Provide migration support for all developers
3. Address platform-specific issues (Windows WSL, macOS)
4. Mark traditional setup as "legacy"

**Success Criteria**: Majority of team using Nix daily

---

### Phase 5: Cleanup (Week 5-6)

**Goal**: Consolidate on Nix

**Actions**:
1. Remove traditional setup from primary docs
2. Move to "legacy" section
3. Deprecation timeline announced (3-6 months)
4. Final stragglers supported

**Success Criteria**: 95%+ using Nix, traditional docs legacy

---

## Troubleshooting Migration Issues

### Issue: "I can't enter Nix shell because I already have Cargo running"

**Solution**: Exit all Cargo processes first:

```bash
# Find and kill Cargo processes
pkill cargo

# Or just exit your traditional shell
exit

# Start fresh
cd rust-sitter
nix develop
```

---

### Issue: "Nix and traditional Rust conflict"

**Solution**: They don't actually conflict - they're isolated. But to verify:

```bash
# Check which Rust you're using
which rustc
# Inside Nix shell: /nix/store/...
# Outside Nix shell: ~/.cargo/bin/rustc or /usr/bin/rustc

# To be sure, always check:
echo $IN_NIX_SHELL
# Inside Nix: "impure"
# Outside Nix: (empty)
```

---

### Issue: "My IDE still uses the old Rust"

**Solution**: See [NIX_TROUBLESHOOTING.md](./NIX_TROUBLESHOOTING.md#ide-integration-issues) for IDE integration.

Short version:
```bash
# Use direnv for automatic IDE integration
nix-env -i direnv
echo "use flake" > .envrc
direnv allow

# Or launch IDE from Nix shell
nix develop
code .  # VS Code
```

---

### Issue: "Tests pass locally (traditional) but fail in CI (Nix)"

**Cause**: Environment mismatch - the exact problem Nix solves!

**Solution**: Reproduce CI locally:

```bash
# Run in Nix shell (matches CI)
nix develop --command just ci-all

# Compare with traditional
exit  # Exit Nix shell
cargo test --workspace

# Fix the issue so both pass, or migrate to Nix
```

---

### Issue: "I want to go back to traditional setup"

**Solution**: Nix doesn't prevent this!

```bash
# Exit Nix shell
exit

# Use traditional tools as before
cargo build
cargo test

# Nix is only active inside "nix develop"
```

To fully uninstall Nix:
```bash
sudo rm -rf /nix /etc/nix /etc/profile.d/nix.sh ~/.nix-*
```

---

## Comparison: Before and After

### Before (Traditional Setup)

**Setup Process**:
```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install system dependencies
# Linux:
sudo apt-get install build-essential pkg-config libssl-dev cmake clang

# macOS:
brew install cmake pkg-config openssl

# 3. Install Rust tools
cargo install just cargo-nextest cargo-insta

# 4. Configure environment
export RUST_BACKTRACE=1
export RUST_TEST_THREADS=2
# ... etc

# 5. Test setup
cargo test --workspace
```

**Time**: 30-60 minutes
**Maintenance**: Manual updates for each tool
**Reproducibility**: No guarantees

---

### After (Nix Setup)

**Setup Process**:
```bash
# 1. Install Nix
sh <(curl -L https://nixos.org/nix/install) --daemon

# 2. Enable flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# 3. Enter dev shell
nix develop

# Done!
```

**Time**: 5-10 minutes
**Maintenance**: `nix flake update` (one command)
**Reproducibility**: 100% guaranteed

---

## Workflow Comparison

| Task | Traditional | Nix |
|------|-------------|-----|
| **Setup new machine** | 30-60 min | 5-10 min |
| **Update Rust** | `rustup update` | `nix flake update` |
| **Update tools** | `cargo install --force ...` | `nix flake update` |
| **Run tests** | `cargo test` | `just ci-test` |
| **Run CI locally** | ❌ Not possible | ✅ `just ci-all` |
| **Fix "works locally but not CI"** | ❌ Manual debugging | ✅ Impossible |
| **Onboard new contributor** | 30-60 min | 5 min |
| **Switch Rust versions** | `rustup default ...` | Edit rust-toolchain.toml |
| **Isolate from system** | ❌ No isolation | ✅ Full isolation |

---

## FAQ

### Can I keep my traditional setup while using Nix?

**Yes!** Nix only activates inside `nix develop`. Outside that shell, your traditional tools work normally.

---

### Will Nix slow down my builds?

**No.** After initial setup, Nix has zero runtime overhead. Builds run at native speed.

---

### What if I don't like Nix?

You can stop using it anytime by simply not entering `nix develop`. Your traditional setup continues working.

---

### Do I need to learn Nix language?

**No.** For development, you only need 2 commands:
- `nix develop` (enter shell)
- `exit` (leave shell)

Modifying `flake.nix` is optional and infrequent.

---

### What about CI?

CI uses Nix exclusively (already done ✅). This ensures local `nix develop` matches CI exactly.

---

### Can I gradually migrate?

**Yes!** See [Gradual Migration](#migration-path-gradual-team) path above.

---

## Next Steps

### Immediate Actions

1. **Try Nix**: Follow [Side-by-Side Migration](#migration-path-side-by-side-recommended)
2. **Compare**: Run tests in both traditional and Nix environments
3. **Decide**: Choose your migration strategy
4. **Get Support**: See [NIX_TROUBLESHOOTING.md](./NIX_TROUBLESHOOTING.md)

### Long-Term

1. **Adopt direnv**: Automatic shell switching
2. **Update workflow**: Use `just ci-*` commands
3. **Share experience**: Help others migrate
4. **Stay updated**: `nix flake update` monthly

---

## Migration Checklist

Use this checklist to track your migration:

### Pre-Migration
- [ ] Backup current environment (`rustc --version`, etc.)
- [ ] Run tests with traditional setup (baseline)
- [ ] Read [Nix Quickstart Guide](./NIX_QUICKSTART.md)
- [ ] Choose migration strategy

### Installation
- [ ] Install Nix
- [ ] Enable flakes
- [ ] Verify `nix --version` works
- [ ] Enter `nix develop` successfully

### Verification
- [ ] Run `just ci-all` in Nix shell
- [ ] Compare results with traditional setup
- [ ] Verify all tests pass
- [ ] Check performance (should be same)

### Integration
- [ ] Configure IDE (direnv or manual launch)
- [ ] Update shell aliases (optional)
- [ ] Test full workflow (edit → test → commit)
- [ ] Verify CI matches local (open test PR)

### Cleanup (Optional)
- [ ] Uninstall traditional tools (if clean switch)
- [ ] Remove old shell configuration
- [ ] Document experience / feedback

### Complete
- [ ] Using Nix daily
- [ ] No issues for 1 week
- [ ] Team member trained (if applicable)

---

## Support

**Questions?** See:
- [Nix Quickstart Guide](./NIX_QUICKSTART.md) - Setup instructions
- [Nix Troubleshooting](./NIX_TROUBLESHOOTING.md) - Common issues
- [ADR-0008](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md) - Design rationale

**Need Help?**
- [GitHub Discussions](https://github.com/EffortlessMetrics/rust-sitter/discussions)
- [GitHub Issues](https://github.com/EffortlessMetrics/rust-sitter/issues) (tag: `nix`)

---

**Guide Version**: 1.0.0
**Last Updated**: 2025-11-20
**Maintained By**: rust-sitter core team

---

END OF MIGRATION GUIDE
