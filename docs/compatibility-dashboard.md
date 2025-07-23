# Grammar Compatibility Dashboard

This dashboard tracks the compatibility status of popular Tree-sitter grammars with rust-sitter v0.5.0-beta.

Last updated: 2025-01-23

## Status Legend

- ✅ **Working** - Grammar fully supported and tested
- ⚠️ **Partial** - Basic features work, some advanced features missing
- ❌ **Blocked** - Grammar requires features not yet implemented
- 🔄 **In Progress** - Active development to support this grammar

## Grammar Status

### Fully Supported Grammars ✅

| Grammar | Version | Tests | Notes |
|---------|---------|-------|-------|
| JSON | latest | ✅ Pass | Full support including NODE_TYPES.json |
| TOML | latest | ✅ Pass | All features supported |
| INI | latest | ✅ Pass | Simple grammar, works perfectly |
| CSV | latest | ✅ Pass | Basic parsing supported |

### Blocked Grammars ❌

| Grammar | Blocking Features | Priority | Target Release |
|---------|------------------|----------|----------------|
| JavaScript | `prec`, `prec.left`, `word`, externals | High | v0.6.0 |
| TypeScript | `prec`, `prec.left`, `word`, externals, supertypes | High | v0.6.0 |
| Python | `prec`, `word`, externals (indentation) | High | v0.7.0 |
| Rust | `prec`, `prec.left`, `word`, externals | Medium | v0.6.0 |
| C | `prec`, `prec.left`, `word` | Medium | v0.6.0 |
| C++ | `prec`, `prec.left`, `word`, externals | Medium | v0.7.0 |
| Go | `prec`, `word` | Medium | v0.6.0 |
| Ruby | `prec`, `word`, externals (heredoc) | Medium | v0.7.0 |
| Java | `prec`, `prec.left`, `word` | Low | v0.6.0 |
| C# | `prec`, `word`, externals | Low | v0.7.0 |
| Swift | `prec`, `word`, externals | Low | v0.7.0 |
| Kotlin | `prec`, `word` | Low | v0.6.0 |

### Configuration Languages

| Grammar | Status | Notes |
|---------|--------|-------|
| YAML | ❌ Blocked | Requires externals for indentation |
| XML | ⚠️ Partial | Basic parsing works, DTD support missing |
| HTML | ❌ Blocked | Requires externals for script/style |
| Markdown | ❌ Blocked | Requires externals for indentation |

### Query Languages

| Grammar | Status | Notes |
|---------|--------|-------|
| SQL | ❌ Blocked | Requires precedence for operators |
| GraphQL | ⚠️ Partial | Basic queries work |
| SPARQL | ❌ Blocked | Complex precedence rules |

## Feature Implementation Progress

| Feature | Status | Target | Impact |
|---------|--------|--------|--------|
| Precedence (`prec`) | 🔄 In Progress | v0.6.0 | Unblocks 15+ grammars |
| Left associativity (`prec.left`) | 🔄 In Progress | v0.6.0 | Required for operators |
| Right associativity (`prec.right`) | 🔄 In Progress | v0.6.0 | Required for operators |
| Word tokens | 📋 Planned | v0.6.0 | Unblocks keyword handling |
| External scanners | 📋 Planned | v0.7.0 | Unblocks Python, Ruby |
| Conflicts | 📋 Planned | v0.6.0 | Better ambiguity handling |
| Supertypes | 📋 Planned | v0.6.0 | Type hierarchy in AST |

## Testing Your Grammar

To test if your grammar works with rust-sitter:

```bash
# Clone your grammar
git clone https://github.com/tree-sitter/tree-sitter-yourlang

# Create a test project
cargo new --lib test-yourlang
cd test-yourlang

# Add rust-sitter dependency
echo 'rust-sitter = "0.5.0-beta"' >> Cargo.toml
echo 'rust-sitter-tool = "0.5.0-beta"' >> Cargo.toml

# Copy grammar.js
cp ../tree-sitter-yourlang/grammar.js .

# Try to build
cargo build
```

## Contributing

Help us improve grammar support! If you find a grammar that works but isn't listed, or if you've implemented support for new features, please:

1. [Open an issue](https://github.com/hydro-project/rust-sitter/issues) with your findings
2. Submit a PR updating this dashboard
3. Add test cases for the grammar

## Roadmap

- **v0.5.0-beta** (Current) - Core features, simple grammars
- **v0.6.0** - Precedence, associativity, word tokens (~60% grammar support)
- **v0.7.0** - External scanners, full conflicts (~90% grammar support)
- **v0.8.0** - Query language, incremental parsing
- **v1.0.0** - Full Tree-sitter compatibility