# Documentation Guide

Welcome to rust-sitter! This guide will help you find the right documentation based on your needs.

## Quick Navigation

### 🚀 For Users Evaluating the Project

Start here if you're considering rust-sitter for a project:

1. **[README.md](./README.md)** - Project overview with status
   - What is rust-sitter?
   - Current capabilities
   - What works and what doesn't
   - Early development status

2. **[PROJECT_STATE_v0.8.0-dev.md](./PROJECT_STATE_v0.8.0-dev.md)** - Honest capability assessment
   - What actually works ✅
   - Critical gaps ⚠️
   - Test results summary (379/385 passing = 98.4%)
   - Detailed recommendations
   - Timeline to v1.0.0 (March 2026)

3. **[KNOWN_LIMITATIONS.md](./KNOWN_LIMITATIONS.md)** - Feature compatibility matrix
   - Supported features
   - Known limitations
   - Grammar compatibility status
   - Roadmap

**Decision Time**:
- ✅ Good for: Research, architecture evaluation, simple grammars, learning
- ❌ Not ready for: Production language servers, complex real-world grammars, performance-critical apps

### 👨‍💻 For Contributors & Developers

Start here if you want to contribute or understand the codebase:

1. **[CLAUDE.md](./CLAUDE.md)** - Developer guidelines
   - MSRV: Rust 1.89.0
   - Architecture overview
   - Common development commands
   - Testing guidelines
   - Concurrency caps implementation
   - Known issues and achievements

2. **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** - Development roadmap
   - **Phase 1** (3-4 weeks): Transform Function Execution ⚡ **CRITICAL**
     - Fix custom lexer type conversion
     - Enable most real-world grammars
     - File: `runtime/src/parser_v4.rs`
   - **Phase 2** (2 weeks): Real Performance Benchmarks
   - **Phase 3** (4-6 weeks): External Scanner Support
   - **Phase 4** (2-3 weeks): Comprehensive Testing (50+ grammars)
   - **Phase 5** (2-3 weeks): Production Release
   - Total timeline: 13-19 weeks to v1.0.0

3. **[NEXT_STEPS.md](./NEXT_STEPS.md)** - Getting started with development
   - Before you start (required reading)
   - Available work opportunities
   - Development environment setup
   - Code style and standards
   - Getting help

4. **[CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md)** - Detailed issue analysis
   - Root cause analysis for each issue
   - Specific code locations
   - Recommended fixes
   - Impact assessments

### 📚 For Technical Deep Dives

### Architecture & Implementation Details
- **[PURE_RUST_IMPLEMENTATION.md](./PURE_RUST_IMPLEMENTATION.md)** - Pure-Rust implementation
  - GLR parser architecture
  - Grammar IR design
  - Table generation algorithms
  - Runtime components

### API Documentation
- **[book/src/](./book/src/)** - Complete mdBook documentation
  - Installation and setup
  - Quick start guide
  - API reference
  - LSP integration guide
  - Advanced features
  - FAQ

### Testing & Quality
- **Test Results**: See CLAUDE.md for current test status
  - 379/385 tests passing (98.4%)
  - All failures in python-simple grammar (transform function gaps)
  - Comprehensive infrastructure tests passing

## Current Project Status

### v0.8.0-dev (November 2025)
- **Status**: Early Development
- **Test Coverage**: 379/385 passing (98.4%)
- **MSRV**: Rust 1.89.0 (Rust 2024 Edition)
- **Primary Implementation**: Pure-Rust (no C dependencies)
- **Blocked Features**: Transform functions, external scanners, real benchmarks

### Critical Gaps (Blocking Real Parsing)
| Feature | Status | Timeline | Impact |
|---------|--------|----------|--------|
| Transform Functions | ❌ Incomplete | 3-4 weeks | Blocks 95% of grammars |
| Real Performance Benchmarks | ❌ Unverified | 2 weeks | Claims unsubstantiated |
| External Scanners | ❌ Not implemented | 4-6 weeks | Blocks Python, C++, Ruby |

### Timeline to Production
- **v0.8.0-dev** (Now): Early Development, 379/385 tests passing
- **v0.9.0** (Feb 2026): Transform functions + real benchmarks, ~50% grammar support
- **v1.0.0** (Mar 2026): External scanners + production release, ~90% grammar support

## Documentation Structure

```
root/
├── DOCUMENTATION.md          ← You are here
├── README.md                 ← Project overview
├── PROJECT_STATE_v0.8.0-dev.md      ← Honest capability assessment
├── KNOWN_LIMITATIONS.md      ← Feature matrix & gaps
├── IMPLEMENTATION_PLAN.md    ← 5-phase development roadmap
├── NEXT_STEPS.md            ← Contributor onboarding
├── CRITICAL_ISSUES_SUMMARY.md ← Detailed issue analysis
├── CLAUDE.md                ← Developer guidelines
├── PURE_RUST_IMPLEMENTATION.md ← Implementation details
├── CLEANUP_PLAN.md          ← Documentation consolidation strategy
├── DOCUMENTATION_UPDATE_SUMMARY.md ← Recent updates
├── book/src/                ← Complete API documentation
│   ├── README.md
│   ├── getting-started/
│   ├── guide/
│   ├── development/
│   ├── advanced/
│   └── appendix/
└── docs/
    ├── archive/             ← Historical documentation
    │   ├── README.md
    │   └── (35+ old files)
```

## Frequently Asked Questions

### Q: Is rust-sitter production-ready?
**A**: Not yet. It's early development (v0.8.0-dev) with 98.4% test coverage but critical gaps:
- ❌ Transform functions incomplete
- ❌ Performance benchmarks unverified
- ❌ External scanners not implemented

See [PROJECT_STATE_v0.8.0-dev.md](./PROJECT_STATE_v0.8.0-dev.md) for details.

### Q: What works right now?
**A**:
- ✅ GLR parser architecture (production-grade design)
- ✅ Simple grammars (arithmetic, JSON)
- ✅ LR(1) automation generation
- ✅ Error recovery framework

See [KNOWN_LIMITATIONS.md](./KNOWN_LIMITATIONS.md#-supported-features) for complete list.

### Q: When will it be production-ready?
**A**: Realistic timeline is **March 2026** (v1.0.0):
- Phase 1 (transform functions): 3-4 weeks
- Phase 2 (real benchmarks): 2 weeks
- Phase 3 (external scanners): 4-6 weeks
- Phase 4 (testing): 2-3 weeks
- Phase 5 (release): 2-3 weeks

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for detailed plan.

### Q: What should I work on if contributing?
**A**: Highest priority is **Phase 1: Transform Functions** (3-4 weeks):
- Blocks 95%+ of real-world grammars
- Specific file: `runtime/src/parser_v4.rs`
- See [NEXT_STEPS.md](./NEXT_STEPS.md) for detailed tasks

### Q: Which grammars are supported?
**A**:
- ✅ JSON, TOML, INI, Arithmetic
- 🟡 C, Go, Java (need testing)
- ❌ Python, JavaScript, TypeScript, Rust, C++, Ruby (need external scanners)

See [KNOWN_LIMITATIONS.md#-grammar-compatibility-status](./KNOWN_LIMITATIONS.md#-grammar-compatibility-status) for matrix.

### Q: Where can I report issues?
**A**: Check [CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md) for known issues. For new issues:
1. Verify it's not in CRITICAL_ISSUES_SUMMARY.md
2. Open a GitHub issue with:
   - Which grammar/feature is affected
   - What you expected to happen
   - What actually happened
   - Minimal reproduction case

### Q: Where's the old documentation?
**A**: Historical documentation from v0.5.0-beta through v0.6.0 is archived in [docs/archive/](./docs/archive/README.md).

## Recent Changes (November 2025)

### Documentation Updates
- ✅ Updated all version references to 0.8.0-dev
- ✅ Updated MSRV to 1.89 (Rust 2024)
- ✅ Created honest capability assessment (PROJECT_STATE_v0.8.0-dev.md)
- ✅ Created detailed implementation plan (IMPLEMENTATION_PLAN.md)
- ✅ Consolidated 35 old files to archive

### What's Different from Earlier Versions
| Aspect | v0.5-v0.6 | v0.8.0-dev |
|--------|----------|-----------|
| Status | Beta (outdated) | Early Development (honest) |
| MSRV | 1.70 | 1.89 (Rust 2024) |
| Primary Backend | Unclear | Pure-Rust (clear) |
| Performance Claims | Unverified (815 MB/sec) | Honest (unverified benchmarks removed) |
| Transform Functions | Claimed working | Documented as incomplete |
| External Scanners | Claimed working | Documented as not implemented |
| Timeline | Unclear | Clear (March 2026 for v1.0.0) |

## Getting Help

### For Architecture Questions
See [CLAUDE.md](./CLAUDE.md) architecture section and [PURE_RUST_IMPLEMENTATION.md](./PURE_RUST_IMPLEMENTATION.md)

### For Testing & Development
See [CLAUDE.md](./CLAUDE.md) testing section and [NEXT_STEPS.md](./NEXT_STEPS.md)

### For Implementation Guidance
See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for your phase, and [CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md) for detailed analysis

### For Design Discussions
Check GitHub issues and discussions, or open a new issue to discuss approach

### For Deep Technical Details
See [book/src/development/](./book/src/development/) section

---

## Document Maintenance

This documentation is maintained by the rust-sitter team. Last updated: **November 13, 2025**

For outdated information or broken links:
1. Check if the file might be in [docs/archive/](./docs/archive/)
2. Open a GitHub issue with "docs:" prefix
3. Reference the specific file and section

---

**Ready to get started?** Choose your path:
- 👤 **Evaluating**: Start with [README.md](./README.md) → [PROJECT_STATE_v0.8.0-dev.md](./PROJECT_STATE_v0.8.0-dev.md)
- 👨‍💻 **Contributing**: Start with [CLAUDE.md](./CLAUDE.md) → [NEXT_STEPS.md](./NEXT_STEPS.md)
- 🏗️ **Building**: See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)
