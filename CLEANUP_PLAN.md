# Documentation Cleanup Plan

## Problem: Documentation Clutter

There are 27 old status/roadmap/summary files creating confusion:

**Old Files to Archive/Remove** (27 files):
1. BETA_FIXES_SUMMARY.md - v0.5.0-beta (obsolete)
2. BETA_RELEASE_SUMMARY.md - v0.5.0-beta (obsolete)
3. PHASE_7_SUMMARY.md - Old phase tracking (obsolete)
4. PHASE_8_SUMMARY.md - Old phase tracking (obsolete)
5. RELEASE_STATUS_v0.5.0-beta.md - v0.5.0-beta (obsolete)
6. API_FREEZE_SUMMARY.md - Old freeze status (obsolete)
7. FIXES_SUMMARY.md - Duplicate summary (obsolete)
8. PURE_RUST_SUMMARY.md - Redundant (replace with PROJECT_STATE)
9. PURE_RUST_IMPLEMENTATION_ROADMAP.md - Redundant (replaced by IMPLEMENTATION_PLAN)
10. PURE_RUST_BACKEND_ROADMAP.md - Redundant (replaced by IMPLEMENTATION_PLAN)
11. IMPLEMENTATION_ROADMAP.md - Replaced by IMPLEMENTATION_PLAN.md
12. IMPLEMENTATION_STATUS.md - Replaced by PROJECT_STATE_v0.8.0-dev.md
13. IMPLEMENTATION_SUMMARY.md - Replaced by PROJECT_STATE_v0.8.0-dev.md
14. PROJECT_COMPLETION_SUMMARY.md - Misleading (not complete)
15. RELEASE_SUMMARY.md - Old summary (redundant)
16. QUERY_IMPLEMENTATION_SUMMARY.md - Old tracking (replaced by IMPLEMENTATION_PLAN)
17. GLR_STATUS.md - Old status (replaced by PROJECT_STATE)
18. ROADMAP.md - Multiple roadmaps (consolidate)
19. ROADMAP_2025.md - Multiple roadmaps (consolidate)
20. ROADMAP-0.8.0.md - Multiple roadmaps (consolidate)
21. ROADMAP_TO_FULL_COMPATIBILITY.md - Redundant (replaced by IMPLEMENTATION_PLAN)
22. glr-core-parity-status.md - Old tracking (outdated)
23. DOCUMENTATION_UPDATE_SUMMARY_v0.6.0.md - v0.6.0 (obsolete)
24. CRITICAL_ISSUES_SUMMARY.md - Good reference, but superseded by IMPLEMENTATION_PLAN
25. PROJECT_STATUS.md - Superseded by PROJECT_STATE_v0.8.0-dev.md
26. RELEASE_CHECKLIST_v0.5.0-beta.md (if exists) - v0.5.0-beta (obsolete)

**Solution: Create Documentation Index**

Archive or consolidate these into a single source of truth:
- PRIMARY: PROJECT_STATE_v0.8.0-dev.md (current state)
- PRIMARY: IMPLEMENTATION_PLAN.md (development roadmap)
- REFERENCE: CRITICAL_ISSUES_SUMMARY.md (detailed issues)
- REFERENCE: KNOWN_LIMITATIONS.md (technical gaps)
- REFERENCE: CLAUDE.md (developer guidelines)

## Recommended Cleanup Steps

### Step 1: Create Archive Directory (optional)
```bash
mkdir -p docs/archive
mv BETA_*.md docs/archive/
mv PHASE_*.md docs/archive/
mv PURE_RUST_*.md docs/archive/
mv ROADMAP*.md docs/archive/
mv RELEASE_*.md docs/archive/
mv *_STATUS.md docs/archive/
mv glr-core-parity-status.md docs/archive/
```

### Step 2: Keep These Files (Primary Documentation)
1. **PROJECT_STATE_v0.8.0-dev.md** ✅ - Current state (NEW)
2. **IMPLEMENTATION_PLAN.md** ✅ - Development roadmap (NEW)
3. **DOCUMENTATION_UPDATE_SUMMARY.md** ✅ - What was updated (NEW)
4. **KNOWN_LIMITATIONS.md** ✅ - Technical gaps (UPDATED)
5. **CRITICAL_ISSUES_SUMMARY.md** ✅ - Issue detail (REFERENCE)
6. **CLAUDE.md** ✅ - Developer guidelines (UPDATE recommended)
7. **README.md** ✅ - Project overview (UPDATED)

### Step 3: Create Documentation Index File
**File**: `DOCUMENTATION.md` or add to README

Contents:
```markdown
# Documentation Guide

## Quick Navigation

### For Users Evaluating the Project
1. **[README.md](./README.md)** - Project overview and status
2. **[PROJECT_STATE_v0.8.0-dev.md](./PROJECT_STATE_v0.8.0-dev.md)** - Honest capability assessment
   - What works ✅
   - Critical gaps ⚠️
   - Timeline to production

### For Contributors & Developers
1. **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** - 5-phase development roadmap
   - Phase 1: Transform Functions (3-4 weeks)
   - Phase 2: Real Benchmarks (2 weeks)
   - Phase 3: External Scanners (4-6 weeks)
   - Phase 4: Grammar Testing (2-3 weeks)
   - Phase 5: Production Release (2-3 weeks)

2. **[CLAUDE.md](./CLAUDE.md)** - Developer workflow and architecture
3. **[KNOWN_LIMITATIONS.md](./KNOWN_LIMITATIONS.md)** - Technical gaps and roadmap

### For Detailed Technical Information
1. **[CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md)** - Root cause analysis of issues
2. **[book/src/](./book/src/)** - Complete API documentation and guides

## Version History

**Current**: v0.8.0-dev (November 2025)
- Status: Early Development
- Tests: 379/385 passing (98.4%)
- Timeline to v1.0.0: March 2026

For historical information about v0.5.0-beta, v0.6.0, and earlier phases, see:
[docs/archive/](./docs/archive/) (old documentation)
```

### Step 4: Update CLAUDE.md

Add section highlighting current priorities:

```markdown
### Current Development Priorities (v0.8.0-dev)

**CRITICAL (Blocks Everything)**:
1. Transform Function Execution (3-4 weeks)
   - File: runtime/src/parser_v4.rs
   - Issue: Lexer can't execute transforms, falls back to broken parsing
   - Impact: 6 python-simple tests failing, 95%+ of grammars blocked
   - See: IMPLEMENTATION_PLAN.md Phase 1

**HIGH (Restore Credibility)**:
2. Real Performance Benchmarks (2 weeks)
   - Current benchmarks measure character iteration, not parsing
   - See: IMPLEMENTATION_PLAN.md Phase 2

**HIGH (Unlock More Grammars)**:
3. External Scanners (4-6 weeks)
   - Python indentation, C++ raw strings, Ruby heredocs
   - See: IMPLEMENTATION_PLAN.md Phase 3

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for complete roadmap.
```

## Files to Create (Optional but Helpful)

### `NEXT_STEPS.md` - For New Contributors
Quick start guide for first-time contributors:

```markdown
# Next Steps: Getting Started with Development

## Before You Start
1. Read [README.md](./README.md) for project overview
2. Read [PROJECT_STATE_v0.8.0-dev.md](./PROJECT_STATE_v0.8.0-dev.md) for current status
3. Review [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for roadmap

## Available Work

### High Priority (Phase 1 - CRITICAL)
**Transform Function Execution** (3-4 weeks, ~5 tasks)
- Implement TSLexState type conversion trait
- Generate transform function execution code
- Integrate with lexer state management
- Test with python-simple grammar
- Document implementation details

→ Issue: See IMPLEMENTATION_PLAN.md Phase 1

### Medium Priority (Phase 2)
**Real Performance Benchmarks** (2 weeks, ~4 tasks)
- Create benchmark infrastructure
- Implement baseline comparisons
- Run honest performance analysis
- Remove false claims from docs

→ Issue: See IMPLEMENTATION_PLAN.md Phase 2

### Lower Priority (Phase 3+)
**External Scanners** (4-6 weeks)
**Grammar Testing** (2-3 weeks)
**Production Release** (2-3 weeks)

## Development Environment

```bash
# Clone and setup
git clone https://github.com/EffortlessMetrics/rust-sitter.git
cd rust-sitter

# Run tests (use capped concurrency)
cargo t2                    # 2-thread test run
cargo test-ultra-safe       # 1-thread (strictest)

# Build documentation
cargo doc --open

# Run benchmarks
cargo bench
```

## Code Style & Standards

See [CLAUDE.md](./CLAUDE.md) for:
- MSRV: Rust 1.89.0
- Edition: Rust 2024
- Testing: Use nextest with concurrency caps
- Formatting: cargo fmt + cargo clippy

## Getting Help

1. **Architecture questions**: See book/src/development/architecture.md
2. **Testing questions**: See CLAUDE.md testing section
3. **Design discussions**: Check GitHub issues
4. **Technical deep-dives**: See CRITICAL_ISSUES_SUMMARY.md
```

## Summary of Cleanup

### What to Remove/Archive (27 files)
- All BETA_* files (v0.5.0-beta obsolete)
- All PHASE_* files (old tracking)
- All PURE_RUST_*_ROADMAP files (superseded)
- All ROADMAP*.md files except ROADMAP.md (consolidate)
- IMPLEMENTATION_ROADMAP.md (replaced by IMPLEMENTATION_PLAN.md)
- IMPLEMENTATION_STATUS.md (replaced by PROJECT_STATE)
- All old *_STATUS.md files (superseded)
- PROJECT_COMPLETION_SUMMARY.md (misleading)

### What to Keep (7 primary docs)
1. PROJECT_STATE_v0.8.0-dev.md ✅ NEW
2. IMPLEMENTATION_PLAN.md ✅ NEW
3. DOCUMENTATION_UPDATE_SUMMARY.md ✅ NEW
4. README.md ✅ UPDATED
5. CLAUDE.md ✅ UPDATE recommended
6. KNOWN_LIMITATIONS.md ✅ UPDATED
7. CRITICAL_ISSUES_SUMMARY.md ✅ REFERENCE

### What to Create (2 optional docs)
1. DOCUMENTATION.md - Index/navigation guide
2. NEXT_STEPS.md - Contributor onboarding

## Expected Impact

**Before Cleanup**:
- 27+ conflicting/outdated files
- User confusion about which doc to read
- Misleading status information
- No clear entry point for new contributors

**After Cleanup**:
- Clear hierarchy of documentation
- Single source of truth per topic
- No contradictory information
- Easy onboarding for contributors
- Professional project appearance

## Time Estimate
- Archiving old files: 5-10 minutes
- Creating DOCUMENTATION.md: 10-15 minutes
- Updating CLAUDE.md: 10-15 minutes
- Creating NEXT_STEPS.md: 20-30 minutes
- **Total: 45-70 minutes**

## Recommendation

**Essential** (do immediately):
- ✅ Archive 27 old files
- ✅ Create DOCUMENTATION.md (navigation index)

**Recommended** (do soon):
- ✅ Update CLAUDE.md with priorities
- ✅ Create NEXT_STEPS.md for contributors

**Nice to have** (can defer):
- Remove references to archived docs from other files
- Create GitHub PR template referencing the roadmap
