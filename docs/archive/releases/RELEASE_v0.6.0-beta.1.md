# Release Process for v0.6.0-beta.1

## ✅ Pre-Release Checklist (COMPLETED)

- [x] All tests passing
- [x] Code cleanup (clippy warnings addressed)
- [x] Dry-run verification script created
- [x] Dependencies verified in correct order

## 📦 Crate Publication Order

The crates **MUST** be published in this exact order due to dependencies:

1. **adze-common** (no external deps)
2. **adze-ir** (no external deps)
3. **adze-glr-core** (depends on: ir)
4. **adze-tablegen** (depends on: ir, glr-core)
5. **adze-macro** (depends on: common)
6. **adze-tool** (depends on: common, ir, glr-core, tablegen)
7. **adze** (runtime - depends on: macro, ir, glr-core, tablegen)

## 🚀 Actual Release Commands

### Step 1: Login to crates.io
```bash
cargo login [YOUR_API_TOKEN]
```

### Step 2: Sequential Publishing

**IMPORTANT**: Wait 60 seconds between each publish for crates.io index to update!

```bash
# Publish adze-common
cd common && cargo publish --allow-dirty && cd ..
sleep 60

# Publish adze-ir  
cd ir && cargo publish --allow-dirty && cd ..
sleep 60

# Publish adze-glr-core
cd glr-core && cargo publish --allow-dirty && cd ..
sleep 60

# Publish adze-tablegen
cd tablegen && cargo publish --allow-dirty && cd ..
sleep 60

# Publish adze-macro
cd macro && cargo publish --allow-dirty && cd ..
sleep 60

# Publish adze-tool
cd tool && cargo publish --allow-dirty && cd ..
sleep 60

# Publish adze (runtime)
cd runtime && cargo publish --allow-dirty && cd ..
```

### Step 3: Git Tag & Push
```bash
git tag -a v0.6.0-beta.1 -m "Release v0.6.0-beta.1: Production-Ready GLR Parser"
git push origin v0.6.0-beta.1
```

### Step 4: GitHub Release

Create release at: https://github.com/EffortlessMetrics/adze/releases/new

**Title**: v0.6.0-beta.1: Production-Ready GLR Parser

**Description**:
```markdown
## 🎯 Major Achievement: GLR Parser Implementation

This release transforms adze from a simple LR parser to a true GLR (Generalized LR) parser capable of handling ambiguous grammars.

### ✨ Key Features

- **Full GLR Support**: Multi-action cells allow runtime forking on shift/reduce and reduce/reduce conflicts
- **Python Grammar Success**: Successfully compiles 273 symbols with 57 fields and full external scanner support
- **Pure-Rust Implementation**: WASM-compatible parser generation
- **Incremental Parsing**: Foundation for GLR incremental parsing (coming in v0.7.0)

### 🔧 Technical Highlights

- Action table architecture restructured to `Vec<Vec<Vec<Action>>>` (ActionCell model)
- Each state/symbol pair can now have multiple valid actions
- Parser dynamically forks on conflicts, exploring all valid paths
- Comprehensive error recovery strategies

### 📦 Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
adze = "0.6.0-beta.1"
```

### 🚧 Beta Notice

This is a beta release. While the GLR implementation is complete and tested, performance optimizations and incremental parsing improvements are planned for v0.7.0.

### 📖 Documentation

See the updated [README](https://github.com/EffortlessMetrics/adze) for usage examples and migration guide.
```

### Step 5: Announce

- [ ] Discord/Slack announcement
- [ ] Twitter/X post
- [ ] Blog post (if applicable)
- [ ] Reddit r/rust (if significant enough)

## 🎯 Post-Release

After successful release:

1. Update README.md with new version badge
2. Update documentation site
3. Monitor for early adopter feedback
4. Begin v0.7.0 development (performance optimization sprint)

## ⚠️ Rollback Plan

If critical issues are discovered:

1. Yank affected versions: `cargo yank --vers 0.6.0-beta.1`
2. Fix issues
3. Release v0.6.0-beta.2
4. Communicate clearly with users about the issue