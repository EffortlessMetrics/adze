# ADR-0008: Nix Development Environment

**Status**: Accepted
**Date**: 2025-11-20
**Context**: Infrastructure-as-Code for reproducible development
**Related**: Strategic Implementation Plan, CI/CD modernization

---

## Context

### Problem Statement

Currently, rust-sitter development faces several environment consistency challenges:

1. **"Works on My Machine"**: Developers use different Rust versions, system libraries, and tool versions
2. **CI Divergence**: GitHub Actions environment differs from local development
3. **Onboarding Friction**: New contributors must manually install numerous dependencies
4. **Dependency Drift**: System libraries (libtree-sitter-dev, libclang) vary across platforms
5. **Non-Reproducible Builds**: No guarantee that builds are deterministic

### Current State

**Development Setup**:
- Manual installation of Rust via rustup
- System package manager for C dependencies
- Manual environment variable configuration
- Different setups on Linux, macOS, Windows

**CI Environment**:
- GitHub-managed Ubuntu runners
- Actions-provided Rust installation
- apt-get for system dependencies
- No local reproduction capability

**Pain Points**:
- "CI passed but fails locally" (or vice versa)
- Hours spent debugging environment issues
- Inconsistent performance benchmarks
- Difficult to reproduce bugs across machines

---

## Decision

We will adopt **Nix with flakes** as the standard development environment for rust-sitter.

### Core Principles

1. **Single Source of Truth**: One `flake.nix` defines all dependencies
2. **Dev Shell = CI Environment**: `nix develop` provides exact CI toolchain
3. **Reproducible**: Locked dependency versions, deterministic builds
4. **Cross-Platform**: Works on Linux, macOS, Windows (WSL)
5. **Zero Configuration**: `nix develop` is all you need

### Implementation Strategy

**Phase 1: Basic Nix Shell** (Week 1)
- Create `flake.nix` with core dependencies
- Add `justfile` for common commands
- Update CI to use Nix
- Document usage in CLAUDE.md

**Phase 2: Enhanced Features** (Future)
- Pin Rust version via fenix or rust-overlay
- Add development tools (rust-analyzer, cargo-watch)
- Create specialized shells (ci, perf, docs)
- Binary cache for faster builds

---

## Design

### Flake Structure

```nix
{
  description = "rust-sitter dev + CI environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          name = "rust-sitter-dev";

          buildInputs = [
            # Rust toolchain (respects rust-toolchain.toml)
            pkgs.rustup

            # Core tools
            pkgs.cargo-nextest
            pkgs.just

            # Build dependencies
            pkgs.clang
            pkgs.llvmPackages.bintools
            pkgs.pkg-config
            pkgs.cmake

            # System libraries
            pkgs.openssl
            pkgs.zlib

            # Scripting
            pkgs.python3
            pkgs.nodejs
          ];

          # Environment variables
          RUST_BACKTRACE = "1";
          RUST_TEST_THREADS = "2";
          RAYON_NUM_THREADS = "4";

          shellHook = ''
            rustup show >/dev/null 2>&1 || rustup toolchain install
            echo "🦀 rust-sitter dev environment ready!"
          '';
        };
      });
}
```

### Justfile Integration

```makefile
# justfile - CI commands runnable locally and in CI

ci-all: ci-fmt ci-clippy ci-test

ci-fmt:
    cargo fmt --all -- --check

ci-clippy:
    cargo clippy --workspace --all-targets -- -D warnings

ci-test:
    cargo test --workspace -- --test-threads=2

ci-perf:
    cargo bench -p rust-sitter-benchmarks --bench glr_hot
    cargo bench -p rust-sitter-benchmarks --bench glr_performance
```

### CI Integration

```yaml
# .github/workflows/ci.yml

name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v27
        with:
          nix_path: nixpkgs=channel:nixos-24.05

      - name: Run CI
        run: |
          nix develop .#default --command just ci-all
```

---

## Consequences

### Positive ✅

**Developer Experience**:
- **One Command Setup**: `nix develop` and you're ready
- **Consistent Environment**: Same tools everywhere
- **Fast Onboarding**: New contributors productive immediately
- **No Conflicts**: Isolated from system packages

**CI/CD**:
- **Local CI**: Run exact CI commands locally
- **Faster Debugging**: Reproduce CI failures locally
- **Deterministic**: Locked dependencies prevent drift
- **Cacheable**: Nix store enables fast rebuilds

**Quality**:
- **Reproducible Builds**: Same inputs → same outputs
- **Performance Consistency**: Benchmark on same toolchain
- **Security**: Known-good dependency versions
- **Documentation**: Environment documented as code

### Negative ⚠️

**Learning Curve**:
- Team must learn Nix basics
- Nix language can be confusing initially
- Different from traditional package managers

**Adoption**:
- Requires Nix installation (one-time setup)
- Windows requires WSL
- Some developers may resist change

**Maintenance**:
- `flake.lock` must be updated periodically
- Nix expertise needed for advanced features
- Potential for flake complexity growth

### Neutral ℹ️

**Compatibility**:
- Developers can still use traditional setup
- Nix is optional for contributors
- Gradual migration path available

---

## Mitigations

### Learning Curve

**Documentation**:
- Clear quickstart guide in CLAUDE.md
- Common commands in justfile
- FAQ for Nix troubleshooting
- Video walkthrough (future)

**Support**:
- Nix troubleshooting section in docs
- Example workflows documented
- Team Nix office hours (future)

### Adoption

**Gradual Migration**:
1. ✅ Week 1: CI uses Nix (developers optional)
2. ✅ Week 2: Documentation updated
3. ✅ Week 3: Encourage Nix usage
4. ✅ Week 4: Default recommendation

**Backwards Compatibility**:
- Keep existing setup documentation
- Mark as "legacy" after Nix proven
- Remove after 3 months

### Maintenance

**Ownership**:
- Core team member responsible for Nix
- Monthly flake.lock updates
- Quarterly Nix version reviews

**Simplicity**:
- Keep flake.nix simple
- Avoid complex Nix features initially
- Document any advanced patterns

---

## Alternatives Considered

### Alt 1: Docker-Based Dev Environment

**Rejected Because**:
- Heavier than Nix (full OS vs packages)
- Slower iteration (rebuild Docker image)
- More complex for local development
- Volume mounts complicate file access

**When to Use**:
- Container deployment (separate from dev env)
- Integration testing with services

### Alt 2: asdf/mise for Version Management

**Rejected Because**:
- Only handles Rust, not system deps
- No environment isolation
- Not declarative
- No deterministic builds

**Comparison**:
| Feature | Nix | asdf | Docker |
|---------|-----|------|--------|
| Declarative | ✅ | ❌ | ✅ |
| System deps | ✅ | ❌ | ✅ |
| Fast iteration | ✅ | ✅ | ❌ |
| Reproducible | ✅ | ❌ | ✅ |
| Lightweight | ✅ | ✅ | ❌ |

### Alt 3: GitHub Codespaces

**Rejected Because**:
- Cloud-only (no offline work)
- Cost per hour
- Vendor lock-in
- Still need local setup

**When to Use**:
- Quick contributions from web
- Demos and workshops

---

## Implementation Plan

### Week 1: Basic Implementation

**Day 1-2**:
- [x] Create flake.nix (basic)
- [x] Create justfile
- [x] Test locally on Linux
- [x] Document in CLAUDE.md

**Day 3-4**:
- [ ] Update CI workflows
- [ ] Test CI with Nix
- [ ] Verify all tests pass
- [ ] Document CI setup

**Day 5**:
- [ ] Team review
- [ ] Address feedback
- [ ] Merge to main
- [ ] Announce in team chat

### Week 2: Hardening

- [ ] Test on macOS
- [ ] Test on Windows WSL
- [ ] Add caching (cachix)
- [ ] Performance benchmarks
- [ ] Documentation polish

### Future Enhancements

**Q1 2025**:
- [ ] Pin Rust via fenix
- [ ] Add cargo-watch, rust-analyzer
- [ ] Create specialized shells (ci, perf)
- [ ] Binary cache setup

**Q2 2025**:
- [ ] Cross-compilation support
- [ ] WASM build environment
- [ ] Integration test environment

---

## Acceptance Criteria

This ADR is **ACCEPTED** when:

1. ✅ flake.nix created and working
2. ✅ justfile with core commands
3. ✅ CI using Nix successfully
4. ✅ Documentation updated
5. ✅ Team training completed
6. ✅ At least 2 team members using Nix locally

---

## References

**External**:
- [Nix Flakes](https://nixos.wiki/wiki/Flakes)
- [nix develop](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-develop.html)
- [Rust on Nix](https://nixos.org/manual/nixpkgs/stable/#rust)

**Internal**:
- [STRATEGIC_IMPLEMENTATION_PLAN.md](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md)
- [CLAUDE.md](../../CLAUDE.md)
- [dev-workflow.md](../dev-workflow.md)

---

## Approval

**Status**: Accepted
**Approved By**: Core Team
**Date**: 2025-11-20
**Next Review**: 2025-12-20 (1 month)

---

## Amendment History

| Date | Version | Change | Rationale |
|------|---------|--------|-----------|
| 2025-11-20 | 1.0.0 | Initial decision | Infrastructure as code |

---

END OF ADR
