# 🎯 rust-sitter v0.6.1-beta

## Algorithmically Correct GLR Parser

This beta release delivers critical correctness fixes for the GLR parser, achieving **100% pass rates** on all core test suites. The parser now correctly handles ambiguous grammars with true GLR semantics.

## ✅ What's Fixed

### Core Algorithm Corrections
- 🔄 **Phase-2 Re-closure**: Reductions now properly re-saturate with the same lookahead, revealing cascaded reduces and accepts
- 📦 **Accept Aggregation**: All valid parses are collected per token with no early returns
- 🔚 **EOF Recovery**: Implements the correct `close → check → (insert|pop)` pattern with no deletion at EOF
- ♾️ **Epsilon Loop Prevention**: Position-aware `RedStamp(state, rule, end)` prevents infinite loops
- ➡️ **Nonterminal Goto**: Fixed critical bug using action table instead of goto table for nonterminals

### Query & Forest Improvements
- 🎯 **Query Correctness**: Squashes unary wrapper nodes with identical spans to prevent double-counting
- 🔗 **Capture Deduplication**: Query matches deduplicated by `(symbol, start, end)` tuple
- 🌳 **Safe Stack Dedup**: Only removes exact pointer duplicates, preserving all ambiguous derivations

### Test Infrastructure
- 🏗️ **Proper Parse Tables**: Replaced hand-crafted tables with LR(1) automaton builder
- 📏 **Ambiguity Understanding**: Tests respect that LR(1) ambiguity typically surfaces at length ≥3
- 🛡️ **Regression Guards**: Added 5 guard tests that will fail if fixes are removed

## 📊 Test Results

```
Test Suite          | Pass Rate | Status
--------------------|-----------|--------
Fork/Merge          | 30/30     | ✅ Perfect
Integration         | 5/5       | ✅ Perfect  
Error Recovery      | 5/5       | ✅ Perfect
GLR Parsing         | 6/6       | ✅ Perfect
Regression Guards   | 5/5       | ✅ Perfect
```

## 🚀 What This Enables

- **Complex Language Support**: Correctly parse C++, Rust, Python, and other ambiguous grammars
- **Better IDE Experience**: Improved error recovery and incremental parsing stability
- **Research Applications**: Foundation for grammar inference and language analysis tools
- **WASM Compatibility**: Pure-Rust implementation enables browser-based parsing

## ⚠️ Known Limitations (Beta)

- Performance optimization pending (safe dedup heuristics need tuning)
- Query predicates and advanced APIs still in development
- External scanner FFI integration needs final touches
- CLI runtime loading and corpus runner not yet implemented

## 🔧 For Maintainers

See [GLR_GUARDRAILS.md](docs/GLR_GUARDRAILS.md) for:
- Regression prevention checklist
- Performance monitoring points
- "If this breaks" troubleshooting guide
- Code review red flags

## 📦 Installation

```toml
[dependencies]
rust-sitter = "0.6.1-beta"
```

## 🙏 Acknowledgments

This release represents a major milestone in creating a correct, safe, pure-Rust GLR parser while maintaining Tree-sitter compatibility. Thank you to all contributors and testers who helped identify and fix these critical issues.