# GLR .parsetable Quickstart Guide

**Version**: 1.0
**Status**: ACTIVE
**Date**: 2025-11-20
**Related**: PARSETABLE_FILE_FORMAT_SPEC.md, GLR_V1_COMPLETION_CONTRACT.md

---

## 🎯 Overview

This guide shows you how to use the `.parsetable` file format to distribute pre-generated GLR parse tables. The .parsetable pipeline enables:

- **Fast builds**: Skip expensive table generation during compilation
- **Deterministic deployment**: Ship consistent parse tables across environments
- **Runtime flexibility**: Load different grammars dynamically without recompiling
- **Efficient distribution**: Compact binary format optimized for size

**Implementation Status**: ✅ **Production Ready** (Phases 1-3.2 complete)

---

## 📋 Prerequisites

### Feature Flags

Enable these features in your `Cargo.toml`:

```toml
[dependencies]
adze-runtime = { version = "0.1", features = ["pure-rust", "serialization"] }

[build-dependencies]
adze-tool = { version = "0.8.0-dev", features = ["serialization"] }
adze-tablegen = { version = "0.8.0-dev", features = ["serialization"] }
```

### System Requirements

- **Rust**: 1.92.0 or later (Rust 2024 Edition)
- **Disk Space**: ~100-500 KB per grammar (depends on grammar size)
- **Memory**: Minimal overhead (~1-2 MB for typical grammars)

---

## 🚀 Quick Start: Three-Step Pipeline

### Step 1: Generate .parsetable File (Build Time)

In your `build.rs`, enable .parsetable generation:

```rust
// build.rs
use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

fn main() {
    let options = BuildOptions {
        out_dir: std::env::var("OUT_DIR").unwrap(),
        emit_artifacts: true,  // ← Enable .parsetable generation
        compress_tables: true,
    };

    let grammar_json = std::fs::read_to_string("grammar.json").unwrap();

    build_parser_from_json(grammar_json, options)
        .expect("Parser build failed");
}
```

**Output**: `$OUT_DIR/grammar_<name>/<name>.parsetable`

### Step 2: Load .parsetable File (Runtime)

In your application code:

```rust
use adze_runtime::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read .parsetable file
    let parsetable_bytes = std::fs::read("path/to/grammar.parsetable")?;

    // Create parser and load table
    let mut parser = Parser::new();
    parser.load_glr_table_from_bytes(&parsetable_bytes)?;

    // Configure symbol metadata and token patterns
    parser.set_symbol_metadata(metadata)?;
    parser.set_token_patterns(patterns)?;

    // Parse input
    let input = b"your source code here";
    let tree = parser.parse(input, None)?;

    println!("Parse succeeded! Root: {}", tree.root_node().kind());
    Ok(())
}
```

### Step 3: Parse Input

```rust
// Reuse parser for multiple inputs
for input in inputs {
    let tree = parser.parse(input, None)?;
    process_tree(&tree);
}
```

---

## 📚 Complete Example: Arithmetic Grammar

### 1. Define Grammar (grammar.json)

```json
{
  "name": "arithmetic",
  "rules": {
    "expr": {
      "type": "SYMBOL",
      "name": "number"
    },
    "number": {
      "type": "PATTERN",
      "value": "[0-9]+"
    }
  }
}
```

### 2. Build Script (build.rs)

```rust
use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

fn main() {
    let options = BuildOptions {
        out_dir: "target/generated".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    let grammar = r#"{
        "name": "arithmetic",
        "rules": {
            "expr": {"type": "SYMBOL", "name": "number"},
            "number": {"type": "PATTERN", "value": "[0-9]+"}
        }
    }"#;

    build_parser_from_json(grammar.to_string(), options)
        .expect("Build failed");

    println!("Generated: target/generated/grammar_arithmetic/arithmetic.parsetable");
}
```

### 3. Parser Setup (src/main.rs)

```rust
use adze_runtime::{
    Parser,
    language::SymbolMetadata,
    tokenizer::{TokenPattern, Matcher},
};
use adze_ir::SymbolId;

fn create_arithmetic_parser() -> Result<Parser, Box<dyn std::error::Error>> {
    // Load .parsetable file
    let bytes = std::fs::read("target/generated/grammar_arithmetic/arithmetic.parsetable")?;

    let mut parser = Parser::new();
    parser.load_glr_table_from_bytes(&bytes)?;

    // Define symbol metadata
    let metadata = vec![
        SymbolMetadata { is_terminal: true, is_visible: false, is_supertype: false }, // EOF
        SymbolMetadata { is_terminal: true, is_visible: true, is_supertype: false },  // number
        SymbolMetadata { is_terminal: false, is_visible: true, is_supertype: false }, // expr
    ];
    parser.set_symbol_metadata(metadata)?;

    // Define token patterns
    let patterns = vec![
        TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
            is_keyword: false,
        },
    ];
    parser.set_token_patterns(patterns)?;

    Ok(parser)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = create_arithmetic_parser()?;

    // Parse input
    let input = b"42";
    let tree = parser.parse(input, None)?;

    // Inspect tree
    let root = tree.root_node();
    assert_eq!(root.kind(), "expr");
    assert_eq!(root.child_count(), 1);

    let number = root.child(0).unwrap();
    assert_eq!(number.kind(), "number");

    println!("✓ Parsed '42' successfully!");
    Ok(())
}
```

---

## 🔍 File Format Details

### .parsetable Binary Structure

```
┌────────────────────────────────┐
│ "RSPT" (4 bytes)              │ Magic number
├────────────────────────────────┤
│ Version: 2 (u32 LE)           │ Format version
├────────────────────────────────┤
│ Grammar Hash (32 bytes)       │ SHA-256 hash
├────────────────────────────────┤
│ Metadata Length (u32 LE)      │ JSON size
├────────────────────────────────┤
│ Metadata JSON (variable)      │ Human-readable info
├────────────────────────────────┤
│ Table Length (u32 LE)         │ Postcard size
├────────────────────────────────┤
│ ParseTable (postcard)         │ Serialized table
└────────────────────────────────┘
```

### Metadata Example

```json
{
  "schema_version": "1.0",
  "grammar": {
    "name": "arithmetic",
    "version": "1.0.0",
    "language": "arithmetic"
  },
  "generation": {
    "timestamp": "2025-11-20T15:30:00Z",
    "tool_version": "0.8.0-dev",
    "rust_version": "1.92.0",
    "host_triple": "x86_64-unknown-linux-gnu"
  },
  "statistics": {
    "state_count": 3,
    "symbol_count": 3,
    "rule_count": 1,
    "conflict_count": 0,
    "multi_action_cells": 0
  },
  "features": {
    "glr_enabled": true,
    "external_scanner": false,
    "incremental": false
  }
}
```

---

## 🛠️ Advanced Usage

### Custom Symbol Metadata

```rust
// Define custom metadata for complex grammars
let metadata = vec![
    SymbolMetadata {
        is_terminal: true,  // EOF symbol
        is_visible: false,  // Don't show in tree
        is_supertype: false
    },
    SymbolMetadata {
        is_terminal: true,  // Keyword "if"
        is_visible: true,   // Show in tree
        is_supertype: false
    },
    SymbolMetadata {
        is_terminal: false, // Non-terminal "statement"
        is_visible: true,
        is_supertype: true  // Is a supertype (union)
    },
];
parser.set_symbol_metadata(metadata)?;
```

### Regex Token Patterns

```rust
use regex::Regex;

let patterns = vec![
    TokenPattern {
        symbol_id: SymbolId(1),
        matcher: Matcher::Regex(Regex::new(r"\d+").unwrap()),
        is_keyword: false,
    },
    TokenPattern {
        symbol_id: SymbolId(2),
        matcher: Matcher::Regex(Regex::new(r"[a-zA-Z_]\w*").unwrap()),
        is_keyword: false,
    },
    TokenPattern {
        symbol_id: SymbolId(3),
        matcher: Matcher::Regex(Regex::new(r"if|then|else").unwrap()),
        is_keyword: true, // Mark as keyword
    },
];
parser.set_token_patterns(patterns)?;
```

### Error Handling

```rust
match parser.load_glr_table_from_bytes(&bytes) {
    Ok(_) => println!("Table loaded successfully"),
    Err(e) => match e.kind() {
        ParseErrorKind::InvalidFormat => {
            eprintln!("Invalid .parsetable file: {}", e);
        }
        ParseErrorKind::UnsupportedVersion => {
            eprintln!("Unsupported format version: {}", e);
        }
        ParseErrorKind::DeserializationError => {
            eprintln!("Failed to deserialize table: {}", e);
        }
        _ => eprintln!("Unknown error: {}", e),
    }
}
```

---

## ✅ Testing Your Implementation

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsetable_loading() {
        let bytes = std::fs::read("path/to/test.parsetable")
            .expect("Failed to read test file");

        let mut parser = Parser::new();
        parser.load_glr_table_from_bytes(&bytes)
            .expect("Loading should succeed");

        assert!(parser.is_glr_mode());
    }

    #[test]
    fn test_parsing_with_loaded_table() {
        let mut parser = create_arithmetic_parser()
            .expect("Parser setup failed");

        let input = b"123";
        let tree = parser.parse(input, None)
            .expect("Parsing should succeed");

        assert_eq!(tree.root_node().kind(), "expr");
    }
}
```

### Integration Test Example

```rust
// tests/integration_test.rs
use adze_runtime::Parser;

#[test]
fn test_end_to_end_pipeline() {
    // Step 1: Generate .parsetable (would run in build.rs)
    // (Assumed already generated)

    // Step 2: Load table
    let bytes = include_bytes!("../target/generated/grammar_arithmetic/arithmetic.parsetable");
    let mut parser = Parser::new();
    parser.load_glr_table_from_bytes(bytes).unwrap();

    // Step 3: Configure parser
    // (set_symbol_metadata, set_token_patterns)

    // Step 4: Parse
    let tree = parser.parse(b"42", None).unwrap();
    assert_eq!(tree.root_node().kind(), "expr");
}
```

---

## 🐛 Troubleshooting

### Error: "Invalid .parsetable file: too short"

**Cause**: File is truncated or corrupted

**Solution**: Regenerate .parsetable file with build script

### Error: "bad magic number"

**Cause**: File is not a valid .parsetable file

**Solution**: Verify file path and regeneration

### Error: "Unsupported format version"

**Cause**: .parsetable version mismatch

**Solution**: Upgrade adze-runtime or regenerate table

### Error: "Failed to deserialize ParseTable"

**Cause**: Corrupted postcard data or version mismatch

**Solution**:
1. Check adze-glr-core version compatibility
2. Regenerate .parsetable with matching tool version
3. Verify file integrity (not truncated)

### Error: "Syntax error: unexpected token"

**Cause**: Token patterns or symbol metadata misconfigured

**Solution**:
1. Verify SymbolId indices match grammar definition
2. Check regex patterns are correct
3. Ensure all terminals have corresponding TokenPattern entries

---

## 📊 Performance Characteristics

### File Sizes

| Grammar Size | States | .parsetable Size | Load Time |
|-------------|--------|------------------|-----------|
| Small (arithmetic) | 3-5 | 2-8 KB | < 1 ms |
| Medium (JSON) | 20-40 | 20-50 KB | < 5 ms |
| Large (Python) | 200-300 | 100-200 KB | < 20 ms |

### Memory Usage

- **ParseTable**: ~10-50 KB per grammar (leaked, 'static lifetime)
- **Parser State**: ~1-2 KB per parser instance
- **Parse Tree**: ~100-500 bytes per node (depends on input)

### Build Time Savings

Using .parsetable reduces build time by:
- **First build**: No change (table generation required)
- **Incremental builds**: 50-90% faster (skip table generation)
- **Clean builds**: 30-60% faster (cached .parsetable reused)

---

## 🔗 Related Documentation

- [PARSETABLE_FILE_FORMAT_SPEC.md](specs/PARSETABLE_FILE_FORMAT_SPEC.md) - **Historical** binary format specification
- [PARSE_TABLE_SERIALIZATION_SPEC.md](specs/PARSE_TABLE_SERIALIZATION_SPEC.md) - **Historical** ParseTable serialization details
- [GLR_V1_COMPLETION_CONTRACT.md](specs/GLR_V1_COMPLETION_CONTRACT.md) - Completion contract and acceptance criteria
- [GETTING_STARTED.md](GETTING_STARTED.md) - General adze usage guide

---

## 📦 Distribution Best Practices

### Packaging .parsetable Files

1. **Include in crate artifacts**:
   ```toml
   [package]
   include = ["src/**/*", "grammars/**/*.parsetable"]
   ```

2. **Lazy loading pattern**:
   ```rust
   use std::sync::OnceLock;

   static PARSER: OnceLock<Parser> = OnceLock::new();

   fn get_parser() -> &'static Parser {
       PARSER.get_or_init(|| {
           let bytes = include_bytes!("../grammars/lang.parsetable");
           let mut parser = Parser::new();
           parser.load_glr_table_from_bytes(bytes).unwrap();
           // configure parser...
           parser
       })
   }
   ```

3. **Version pinning**:
   - Pin adze-runtime version in Cargo.toml
   - Regenerate .parsetable on version upgrades
   - Include format version in filename: `lang-v1.0.0.parsetable`

### CI/CD Integration

```yaml
# .github/workflows/build.yml
- name: Generate parse tables
  run: cargo build --release

- name: Upload artifacts
  uses: actions/upload-artifact@v3
  with:
    name: parsetables
    path: target/generated/**/*.parsetable

- name: Validate parse tables
  run: |
    for f in target/generated/**/*.parsetable; do
      cargo run --bin validate-parsetable -- "$f"
    done
```

---

## 🎓 Next Steps

1. **Explore GLR Conflicts**: Learn how GLR handles ambiguous grammars → [GLR_USER_GUIDE.md](guides/GLR_USER_GUIDE.md)
2. **Optimize Performance**: Profile your parser → [PERFORMANCE_GUIDE.md](PERFORMANCE_GUIDE.md)
3. **Write Custom Grammars**: Grammar authoring guide → [GRAMMAR_EXAMPLES.md](GRAMMAR_EXAMPLES.md)
4. **Contribute**: Improve .parsetable tooling → [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md)

---

**Questions?** Open an issue at https://github.com/EffortlessMetrics/adze/issues

**Feedback?** We'd love to hear about your experience with .parsetable!

---

**Version**: 1.0
**Last Updated**: 2025-11-20
**Maintainer**: adze core team
