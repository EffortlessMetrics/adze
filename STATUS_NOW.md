# Status Now - Maintainer Overview

**Last Updated**: 2025-11-20
**Version**: v0.6.1-beta
**Next Release**: v0.7.0 (Target: March 2026)

---

## 🎯 Current Focus

**This Week**: .parsetable Pipeline Completion & Documentation (GLR v1 Phase 4)
- [x] Repository URL migration (hydro-project → EffortlessMetrics)
- [x] Messaging alignment (production-ready → strong beta)
- [x] GLR runtime architectural issue documented
- [x] TDD/BDD/Schema infrastructure complete
- [x] **GLR Step 1**: Feature flag architecture (glr feature) ✅
- [x] **GLR Step 2**: Parser backend selection API ✅
- [x] **GLR Step 3**: Parser routing infrastructure in __private::parse() ✅
- [x] **parser_v4 Integration**: Extraction integration complete ✅
- [x] **.parsetable Pipeline**: Complete (Phases 1-3.2) ✅ NEW!
  - [x] Phase 1: ParseTable serialization (bincode + versioning)
  - [x] Phase 2: .parsetable file format (writer + spec)
  - [x] Phase 3.1: Parser::load_glr_table_from_bytes() API
  - [x] Phase 3.2: End-to-end integration tests (30/32 passing)
  - [x] Phase 4: Documentation & API docs complete
- [ ] **GLR Step 4**: Grammar metadata generation (deferred - optional optimization)
- [ ] **GLR Step 5**: Implement BDD scenario tests (NEXT)
- [ ] **GLR Step 6**: Re-enable arithmetic tests

**Blockers for v0.7.0**:
1. **GLR Runtime Wiring** - ⚠️ PARTIAL (Infrastructure complete, table loading blocked)
   - ✅ GLR tables generate correctly (tablegen)
   - ✅ Runtime routing infrastructure in place
   - ✅ parser_v4 extraction integration complete
   - ❌ **NEW BLOCKER**: parser_v4 table loading/decoder incompatibility
   - See: [PARSER_V4_TABLE_LOADING_BLOCKER.md](./docs/plans/PARSER_V4_TABLE_LOADING_BLOCKER.md)
   - ⏳ Pending: Decoder fix, then BDD scenario tests
2. **Incremental Parsing** - Designed but not implemented
3. **Query System** - Partial implementation, needs completion

---

## 📊 What Works Today

### ✅ Stable & Production-Ready
- **Macro-based grammar generation**: 13/13 tests passing
- **Type-safe AST generation**: Compile-time validation works
- **Pure-Rust compilation**: Zero C dependencies
- **WASM support**: First-class support
- **Build system**: `build.rs` integration stable
- **Precedence & associativity**: Works in table generation (not yet in runtime)
- **.parsetable Pipeline**: ✨ NEW! Production-ready binary format for GLR tables
  - **ParseTable serialization**: Bincode-based with version wrapper (Format v1)
  - **File generation**: Automatic .parsetable generation in build.rs
  - **Runtime loading**: `Parser::load_glr_table_from_bytes()` API
  - **Test coverage**: 30/32 tests passing (94%), 2 deferred to Phase 3.3
  - **Documentation**: Comprehensive spec, quickstart guide, and API docs
  - **Use case**: Fast builds, deterministic deployment, runtime grammar loading

### ⚠️ Experimental / Partial
- **GLR runtime**: Fully wired with extraction integration (`parser_v4.rs`), available via `glr` feature
- **External scanners**: Design complete, limited testing
- **Query system**: Basic parsing works, predicates partial
- **Incremental parsing**: Designed, not implemented
- **Error recovery**: Basic support, needs enhancement

### ❌ Not Yet Implemented
- **CLI tools**: Planned for v0.7.0
- **Performance optimization**: Baseline being established
- **Editor plugins**: Planned for v1.0
- **Comprehensive docs**: In progress

---

## 📈 Test Status

### Passing Suites
- Macro generation: 13/13 ✅
- Integration tests: 6/6 ✅
- GLR fork/merge: 30/30 ✅
- Basic error recovery: passing ✅

### Tests with `#[ignore]`
- **Incremental parsing**: ~8 tests (feature not implemented)
- **Query predicates**: ~5 tests (partial implementation)
- **External scanners**: ~3 tests (limited coverage)
- **Python grammars**: ~4 tests (GLR runtime wiring needed)

**Policy**: All tests either pass or are explicitly `#[ignore]` with documentation. No `.rs.disabled` files allowed.

---

## 🏗️ Architecture Status

### What Changed Recently
1. **GLR Precedence/Associativity** (Nov 2025)
   - Fixed in `glr-core/src/lib.rs:344`
   - Tables now correctly encode rule associativity
   - Runtime integration pending

2. **Documentation Reorganization** (Nov 2025)
   - Clear hierarchy: README → QUICK_START → GETTING_STARTED
   - Navigation via NAVIGATION.md
   - Task tracking via GAPS.md + IMPLEMENTATION_PLAN.md

3. **Test Connectivity** (Jan 2025)
   - CI job enforces non-zero test counts
   - Pre-commit hooks prevent `.rs.disabled` files
   - All previously disabled tests re-enabled

### Current Architectural Issues
1. **GLR Runtime Integration** (Priority: MEDIUM - Infrastructure Complete)
   - See: [ARCHITECTURE_ISSUE_GLR_PARSER.md](./ARCHITECTURE_ISSUE_GLR_PARSER.md)
   - Status: ✅ parser_v4 extraction integration complete
   - Remaining: BDD scenario tests and default feature enablement
   - Impact: Associativity works with `glr` feature flag

---

## 🎯 Next 3 Milestones

### 1. v0.6.2-beta (Immediate)
- Complete documentation polish
- Establish performance baseline
- Fix quick-win bugs
- **Timeline**: 1-2 weeks

### 2. v0.7.0 (March 2026)
- Wire GLR runtime as default
- Implement incremental parsing
- Complete query system
- CLI tools (parse, test)
- **Blockers**: GLR wiring, incremental design → implementation

### 3. v1.0 (Q4 2026)
- API stability guarantees
- Production-grade everything
- Editor plugins
- 50+ grammars
- **Dependencies**: v0.7.0 stable, community adoption

---

## 🔧 For Contributors

**Want to Help?**
1. **Quick wins** (1-4 hours): See [GAPS.md](./GAPS.md) section "Good First Issues"
2. **Re-enable ignored tests**: Use `.github/ISSUE_TEMPLATE/enable_test.md`
3. **Core features**: GLR wiring, incremental parsing (8+ hours)

**This Week's Priorities**:
- GLR runtime integration planning
- Performance baseline establishment
- Documentation consistency

---

## 📚 Documentation Map

**For Maintainers**:
- This file (STATUS_NOW.md) - Quick status overview
- [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md) - Detailed status
- [ARCHITECTURE_ISSUE_GLR_PARSER.md](./ARCHITECTURE_ISSUE_GLR_PARSER.md) - Critical issue

**For Planning**:
- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - 8-week v0.7.0 plan
- [GAPS.md](./GAPS.md) - 43 tasks with estimates
- [ROADMAP.md](./ROADMAP.md) - Long-term vision

**For Users**:
- [README.md](./README.md) - Project overview
- [QUICK_START.md](./QUICK_START.md) - 5-minute tutorial
- [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) - Full guide

---

## 🚨 Known Issues

1. **GLR runtime not wired** - Affects associativity in pure-Rust mode
2. **Performance not benchmarked** - Baseline being established
3. **Some ignored tests** - Documented, tied to unimplemented features
4. **Limited production usage** - Early beta, feedback welcome

---

**Questions?** Check [FAQ.md](./FAQ.md) or [GitHub Issues](https://github.com/EffortlessMetrics/rust-sitter/issues)
