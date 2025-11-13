# Project State: v0.8.0-dev (November 2025)

## Executive Summary

Rust-Sitter v0.8.0-dev is in **active development**. While the pure-Rust implementation has a solid foundation with a production-grade GLR parser architecture, there are **critical gaps** between the documented feature set and actual implementation that users should be aware of before adopting.

**Status: Early Development with Advanced Architecture**

## What Actually Works ✅

### Infrastructure & Architecture
- **GLR Parser Design**: Production-grade architecture with multi-action cells, fork/merge logic, and forest management
- **Grammar Macro System**: Rust-based grammar definitions with compile-time code generation
- **Build System**: Complete workspace organization with proper module structure
- **Testing Infrastructure**: 379+ unit tests with comprehensive test coverage
- **Import Standardization**: Proper module organization (MSRV 1.89, Rust 2024 edition)

### Core Parsing Components
- **LR(1) Automaton Construction**: Fully implemented state machine generation
- **Parse Table Generation**: Complete table compression matching Tree-sitter format
- **NODE_TYPES.json Generation**: Proper metadata generation for all symbol types
- **FFI-Compatible Language Struct**: Proper ABI for integration with external tools

### Runtime Features
- **Error Recovery Framework**: Comprehensive error recovery strategies and state management
- **Visitor API**: Complete tree traversal and analysis interface
- **Basic Serialization**: Parse tree serialization infrastructure

## Known Limitations & Gaps ⚠️

### Critical Issues (Blocking Real Parsing)

1. **Transform Function Execution** ❌
   - **Issue**: Lexer type conversion not fully implemented
   - **Impact**: Grammars with number/string literals, identifiers fail to parse
   - **Status**: Infrastructure in place, execution incomplete
   - **Examples Affected**: Python-simple tests (6 tests failing)
   - **Workaround**: None - affects most real-world grammars

2. **Performance Benchmarks** ❌
   - **Issue**: Current benchmarks measure mocks, not real parsing
   - **Current Claims**: "815 MB/sec throughput", "100x faster than Tree-sitter"
   - **Actual Status**: Character iteration benchmarks, not parser benchmarks
   - **Impact**: Performance claims cannot be verified
   - **Action Needed**: Replace with real parsing benchmarks once lexer is fixed

### High-Priority Gaps

3. **External Scanner Runtime** ❌
   - **Issue**: External scanner declarations parsed but not executed
   - **Impact**: Cannot parse Python (indentation), C++ (raw strings), Ruby (heredocs)
   - **Current Support**: Python grammar defined but cannot run
   - **Affected Grammars**: ~20% of popular grammars require external scanners

4. **Full Incremental Parsing** ⚠️
   - **Status**: Infrastructure exists, but untested with real parsing
   - **Feature Flag**: `incremental` available but requires working lexer
   - **Impact**: Editor integration features may not function correctly

### Medium-Priority Gaps

5. **Query Language Support** ⚠️
   - **Status**: Experimental, feature-gated
   - **Stability**: Not recommended for production
   - **Feature Flag**: `queries` requires careful testing

6. **CLI Compatibility** ⚠️
   - **Current State**: Not fully compatible with tree-sitter CLI
   - **Status**: Only basic grammar generation works
   - **Dynamic Loading**: Not implemented

## Test Results Summary

### Overall Status
- **Total Tests**: 385
- **Passing**: 379 (98.4%)
- **Failing**: 6 (1.6%)
- **Ignored**: 1

### Failure Details
- **Failed Component**: `rust-sitter-python-simple` grammar tests
- **Affected Tests**: 6 string, number, identifier, and expression parsing tests
- **Root Cause**: Transform function execution gaps
- **Issue Reference**: Critical Issue #74

### Passing Components
✅ Arena allocator utilities
✅ Decoder and action tables
✅ Error recovery system
✅ External scanner framework
✅ Concurrency caps system
✅ GLR lexer basics
✅ GLR parser stack management
✅ Query parser
✅ Tree bridge interfaces

## Documentation Accuracy Assessment

### v0.8.0-dev Documentation Status

**Recently Updated (Nov 2025)**:
- ✅ MSRV correctly documented as 1.89
- ✅ Version numbers standardized to 0.8.0-dev
- ✅ Package names clarified (rust-sitter, rust-sitter-tool)
- ✅ Backend status marked (pure-rust as default)

**Issues Identified**:
- ⚠️ README.md claims "production-ready GLR" - needs disclaimer
- ⚠️ Performance claims unverified against real parsing
- ⚠️ Python grammar support claimed but not functional
- ⚠️ Quickstart examples use working arithmetic grammar (not representative)

## Recommended Actions for Users

### For Evaluation
```
✅ DO:
- Review the GLR parser architecture and design
- Run example grammars (arithmetic is reliable)
- Read CLAUDE.md for development guidelines
- Check test results to understand actual capabilities

❌ DON'T:
- Deploy to production without testing your grammar
- Rely on performance benchmarks as-is
- Assume transform functions work (they don't yet)
- Use with grammars requiring external scanners
```

### For Adoption
```
Only Recommended For:
- Research and architecture evaluation
- Grammars without transform functions
- Teams with ability to contribute fixes
- Experimental projects without production requirements

Not Recommended For:
- Production language servers
- Complex real-world language parsing
- Projects depending on external scanners
- Performance-critical applications (until benchmarks are real)
```

## Development Priorities (Next Steps)

### Critical (Blocks Real Parsing)
1. **Implement Transform Function Execution** (3-4 weeks)
   - Complete TSLexState type conversion
   - Execute transform functions instead of capturing them
   - Add proper error handling for transform failures
   - Test with python-simple grammar

2. **Implement Real Performance Benchmarks** (2 weeks)
   - Replace character counting with actual parsing
   - Benchmark against real Tree-sitter
   - Document actual performance characteristics

### High-Priority (Unlocks More Grammars)
3. **External Scanner Runtime Support** (4-6 weeks)
   - Implement Python indentation tracking
   - Implement C++ raw string handling
   - Add Ruby heredoc support

4. **Comprehensive Testing** (2-3 weeks)
   - Test 50+ popular grammars
   - Document compatibility matrix
   - Create grammar-specific test suites

### Medium-Priority (Polish & Features)
5. **CLI Tools** (3-4 weeks)
   - Full tree-sitter generate compatibility
   - Dynamic grammar loading
   - Integration testing

6. **Documentation** (2 weeks)
   - Honest capability matrix
   - Migration guide from Tree-sitter
   - API stability guarantees

## Version Comparison

| Feature | v0.5.0-beta | v0.6.0 (planned) | v0.8.0-dev | Status |
|---------|------------|-----------------|-----------|--------|
| MSRV | 1.70 | 1.70 | 1.89 | ✅ Updated |
| GLR Architecture | ✓ | ✓ | ✓ | ✅ Stable |
| Precedence | ✓ | ✓ | ✓ | ✅ Working |
| Transform Functions | ✓ | ✓ | ✓ | ❌ Incomplete |
| External Scanners | ✗ | ✗ | ✗ | ❌ Not Ready |
| Incremental Parsing | ✗ | ✗ | ✓ (feature-gated) | ⚠️ Untested |
| Query Language | ✗ | ✗ | ✓ (experimental) | ⚠️ Unstable |
| Real Benchmarks | N/A | N/A | ✗ | ❌ Missing |

## Honest Assessment for Stakeholders

### Strengths
1. **Solid Architecture**: GLR design is production-grade and well-engineered
2. **Good Test Coverage**: 379 passing tests demonstrate infrastructure quality
3. **Active Development**: Regular improvements and fixes
4. **Rust Best Practices**: Proper error handling, safety guarantees

### Weaknesses
1. **Gap Between Claims and Reality**: Documentation overstates capabilities
2. **Incomplete Core Features**: Transform functions and external scanners need work
3. **Unverified Benchmarks**: Performance claims based on mocks, not measurements
4. **Limited Grammar Support**: Only simple grammars work reliably

### Timeline to Production
- **Minimum**: 3-4 weeks (fix critical gaps, test thoroughly)
- **Realistic**: 2-3 months (external scanners, comprehensive testing)
- **Safe**: 4-6 months (full feature parity, performance tuning, documentation)

## Questions for Maintainers

1. What is the primary use case for v0.8.0-dev? (Research? Experimental? Production?)
2. What is the timeline for fixing critical issue #74 (transform functions)?
3. Should performance claims be removed from README until benchmarks are real?
4. Which grammars are certified to work correctly?
5. What is the upgrade path from v0.5/v0.6 to v0.8.0-dev?

## References

- **CRITICAL_ISSUES_SUMMARY.md** - Detailed issue breakdown
- **KNOWN_LIMITATIONS.md** - Feature compatibility matrix
- **Test Results** - 379/385 tests passing, 6 failures in python-simple
- **Recent Updates** - MSRV 1.89, v0.8.0-dev standardization

---

*Last Updated: November 13, 2025*
*Status: Active Development*
*Recommendation: Evaluate architecture, defer production adoption*
