# .parsetable File Format Specification

**Version**: 1.0
**Status**: DEPRECATED (Historical)
**Date**: 2025-11-20
**Implementation Status**: ✅ Production Ready for legacy format (Phases 1-3.2 Complete)
**Current Format**: `.parsetable` now uses postcard-based version 2 serialization in active implementations.
**Related**: PARSE_TABLE_SERIALIZATION_SPEC.md, GLR_V1_COMPLETION_CONTRACT.md

---

## 🎯 Purpose

This specification defines the `.parsetable` binary file format for distributing pre-generated GLR parse tables. This format enables:

1. **Fast Parser Loading**: Bypass expensive table generation at build time
2. **Deterministic Builds**: Ship consistent parse tables across environments
3. **Runtime Flexibility**: Load different grammars dynamically
4. **Size Optimization**: Compressed binary format for distribution

---

## 📋 File Format Structure

### Binary Layout

```
┌────────────────────────────────────────────┐
│ Magic Number (4 bytes): "RSPT"            │ 0x00
├────────────────────────────────────────────┤
│ Format Version (4 bytes): u32 LE          │ 0x04
├────────────────────────────────────────────┤
│ Grammar Hash (32 bytes): SHA256           │ 0x08
├────────────────────────────────────────────┤
│ Metadata Length (4 bytes): u32 LE         │ 0x28
├────────────────────────────────────────────┤
│ Metadata JSON (variable length)           │ 0x2C
├────────────────────────────────────────────┤
│ Table Data Length (4 bytes): u32 LE       │ 0x2C + metadata_len
├────────────────────────────────────────────┤
│ ParseTable (legacy bincode) (variable length) │ 0x30 + metadata_len
└────────────────────────────────────────────┘
```

### Field Definitions

#### 1. Magic Number (4 bytes, offset 0x00)
- **Value**: ASCII "RSPT" (Adze Parse Table)
- **Bytes**: `0x52 0x53 0x50 0x54`
- **Purpose**: File type identification

#### 2. Format Version (4 bytes, offset 0x04)
- **Type**: u32 little-endian
- **Current Version**: `0x00000001`
- **Purpose**: Forward compatibility and version detection

#### 3. Grammar Hash (32 bytes, offset 0x08)
- **Type**: SHA-256 hash
- **Purpose**: Verify table matches expected grammar
- **Computation**: SHA-256 of grammar definition source text
- **Use Case**: Detect grammar drift between build and runtime

#### 4. Metadata Length (4 bytes, offset 0x28)
- **Type**: u32 little-endian
- **Purpose**: Length of metadata JSON section

#### 5. Metadata JSON (variable length, offset 0x2C)
- **Type**: UTF-8 encoded JSON
- **Purpose**: Human-readable grammar information
- **Schema**: See Metadata Schema section below

#### 6. Table Data Length (4 bytes, offset 0x2C + metadata_len)
- **Type**: u32 little-endian
- **Purpose**: Length of legacy bincode-encoded ParseTable

#### 7. ParseTable Bincode (variable length, offset 0x30 + metadata_len)
- **Type**: Bincode-serialized ParseTable (legacy format only)
- **Format**: As defined in PARSE_TABLE_SERIALIZATION_SPEC.md
- **Note**: Includes its own version wrapper (FORMAT_VERSION)

---

## 📊 Metadata Schema

The metadata JSON follows this schema:

```json
{
  "schema_version": "1.0",
  "grammar": {
    "name": "string",           // Grammar name (e.g., "rust", "python")
    "version": "string",        // Grammar version (semver)
    "language": "string"        // Source language name
  },
  "generation": {
    "timestamp": "ISO8601",     // Generation time
    "tool_version": "string",   // tablegen version
    "rust_version": "string",   // Rust compiler version
    "host_triple": "string"     // Build host triple
  },
  "statistics": {
    "state_count": number,      // Number of parser states
    "symbol_count": number,     // Number of grammar symbols
    "rule_count": number,       // Number of production rules
    "conflict_count": number,   // Number of GLR conflicts
    "multi_action_cells": number // Cells with >1 action
  },
  "features": {
    "glr_enabled": boolean,     // GLR parsing supported
    "external_scanner": boolean,// External scanner present
    "incremental": boolean      // Incremental parsing support
  }
}
```

### Example Metadata

```json
{
  "schema_version": "1.0",
  "grammar": {
    "name": "python",
    "version": "3.12.0",
    "language": "Python"
  },
  "generation": {
    "timestamp": "2025-11-20T15:30:00Z",
    "tool_version": "0.8.0-dev",
    "rust_version": "1.89.0",
    "host_triple": "x86_64-unknown-linux-gnu"
  },
  "statistics": {
    "state_count": 273,
    "symbol_count": 57,
    "rule_count": 184,
    "conflict_count": 12,
    "multi_action_cells": 8
  },
  "features": {
    "glr_enabled": true,
    "external_scanner": true,
    "incremental": false
  }
}
```

---

## 🔒 Security Considerations

### Grammar Hash Verification

**Problem**: Prevent loading tables from incompatible grammars

**Solution**: Runtime verification flow:
```rust
let expected_hash = compute_grammar_hash(grammar_source);
let file_hash = parsetable_file.grammar_hash();

if expected_hash != file_hash {
    return Err(GrammarMismatchError {
        expected: expected_hash,
        actual: file_hash,
    });
}
```

### File Integrity

**Checksum**: Optional CRC32 footer for corruption detection
- **Location**: Last 4 bytes of file
- **Scope**: Covers all data from offset 0 to (file_size - 4)
- **Algorithm**: CRC32 (IEEE 802.3 polynomial)

### Version Compatibility

**Forward Compatibility Strategy**:
1. Reader checks `format_version` field
2. If version > reader's max supported version:
   - Return `UnsupportedVersionError`
3. If version ≤ reader's max supported version:
   - Read using version-specific decoder

**Backward Compatibility**:
- Format version bumps for breaking changes only
- Additive changes maintain version number
- Metadata schema extensible via optional fields

---

## 📦 File Naming Convention

### Recommended Pattern

```
<grammar_name>-<version>.parsetable
```

### Examples

```
python-3.12.0.parsetable
rust-1.89.0.parsetable
json-1.0.0.parsetable
```

### Platform-Specific Variants (Optional)

If tables have platform-specific optimizations:

```
<grammar_name>-<version>-<triple>.parsetable
```

Example:
```
python-3.12.0-x86_64-unknown-linux-gnu.parsetable
```

---

## 🚀 Usage Workflow

### 1. Generation (Build Time)

```rust
use adze_tablegen::generate_parsetable_file;

// In build.rs
fn main() {
    generate_parsetable_file(
        "grammar.rs",
        "target/python-3.12.0.parsetable"
    ).expect("table generation failed");
}
```

### 2. Loading (Runtime)

```rust
use adze::Parser;

let parser = Parser::from_parsetable_file(
    "python-3.12.0.parsetable"
)?;

parser.parse(source_code)?;
```

### 3. Verification (Runtime)

```rust
use adze::verify_parsetable;

verify_parsetable(
    "python-3.12.0.parsetable",
    grammar_source_hash
)?;
```

---

## 🎨 Design Rationale

### Why Not Pure JSON?

**Considered**: JSON-based table format
**Rejected**:
- Too large (10-50× larger than bincode)
- Slower to parse
- No native multi-action cell representation

### Why Not MessagePack/CBOR?

**Considered**: Alternative binary formats
**Rejected**:
- Bincode is Rust-native (zero-copy deserialization)
- Smaller dependency footprint
- Proven performance with Serde

### Why Separate Metadata?

**Decision**: Embed JSON metadata before bincode payload

**Rationale**:
- Human-inspectable metadata without deserializing entire table
- Tooling can read metadata without bincode dependency
- Graceful degradation if bincode version changes

### Why Grammar Hash?

**Decision**: Include SHA-256 hash of grammar source

**Rationale**:
- Detect silent grammar changes in distributed systems
- Prevent subtle bugs from version skew
- Enable cache invalidation strategies

---

## 📐 Size Benchmarks

### Expected File Sizes

| Grammar      | States | Symbols | .parsetable Size | Compression Ratio |
|--------------|--------|---------|------------------|-------------------|
| JSON         | 28     | 15      | ~8 KB            | 3:1 vs JSON       |
| Python 3.12  | 273    | 57      | ~120 KB          | 4:1 vs JSON       |
| Rust 1.89    | 847    | 142     | ~450 KB          | 5:1 vs JSON       |

### Metadata Overhead

- Fixed header: 44 bytes (magic + version + hash + lengths)
- Typical metadata: 300-500 bytes (JSON)
- **Total overhead**: < 1 KB per file

---

## 🧪 Validation Requirements

### File Format Validator

A standalone tool shall validate .parsetable files:

```bash
$ parsetable-validate python-3.12.0.parsetable

✓ Magic number valid: RSPT
✓ Format version: 1
✓ Grammar hash present
✓ Metadata valid JSON
✓ Metadata schema v1.0 compliant
✓ Table data deserializes successfully
✓ CRC32 checksum matches

File valid: python-3.12.0.parsetable
```

### Validation Checklist

- [ ] Magic number matches "RSPT"
- [ ] Format version supported
- [ ] Grammar hash is 32 bytes
- [ ] Metadata length matches actual metadata size
- [ ] Metadata is valid UTF-8 JSON
- [ ] Metadata conforms to schema
- [ ] Table data length matches actual table size
- [ ] ParseTable deserializes without error
- [ ] CRC32 checksum valid (if present)

---

## 🔄 Version History

### Version 1 (Current)

**Date**: 2025-11-20
**Changes**: Initial specification

**Features**:
- Magic number identification
- Version field for compatibility
- Grammar hash verification
- JSON metadata section
- Bincode ParseTable payload
- Optional CRC32 footer

---

## 📚 References

- [PARSE_TABLE_SERIALIZATION_SPEC.md](PARSE_TABLE_SERIALIZATION_SPEC.md)
- [GLR_V1_COMPLETION_CONTRACT.md](GLR_V1_COMPLETION_CONTRACT.md)
- [Bincode Format Documentation](https://github.com/bincode-org/bincode)
- [Tree-sitter TSLanguage ABI](https://tree-sitter.github.io/tree-sitter/)

---

## ✅ Implementation Checklist

- [x] Define file format constants (`MAGIC_NUMBER`, `FORMAT_VERSION`) ✅
  - Location: `tablegen/src/parsetable_writer.rs:34-41`
- [x] Implement `ParsetableWriter` struct ✅
  - Location: `tablegen/src/parsetable_writer.rs:143-289`
  - Features: Metadata generation, grammar hash, file writing
- [x] Implement `ParsetableReader` (via `Parser::load_glr_table_from_bytes()`) ✅
  - Location: `runtime2/src/parser.rs:305-429`
  - Features: Magic/version validation, metadata parsing, table deserialization
- [x] Add grammar hash computation ✅
  - Location: `tablegen/src/parsetable_writer.rs:221-237`
  - Algorithm: SHA-256 of grammar name and rules
- [x] Implement metadata JSON serialization ✅
  - Location: `tablegen/src/parsetable_writer.rs:63-141`
  - Schema: v1.0 with grammar, generation, statistics, and features
- [x] Write integration tests ✅
  - Generation tests: `tool/tests/test_parsetable_generation.rs` (3 tests)
  - Loading tests: `runtime2/tests/test_parsetable_loading.rs` (9 tests)
  - End-to-end tests: `runtime2/tests/test_end_to_end_parsetable.rs` (5 tests)
  - **Total: 17 tests, 15 passing, 2 deferred to Phase 3.3**
- [x] Document API in rustdoc ✅
  - `Parser::load_glr_table_from_bytes()`: Comprehensive API docs
  - `ParsetableWriter`: Module and struct documentation
  - `ParseTable::to_bytes()/from_bytes()`: Contract and examples
- [x] Add examples to documentation ✅
  - Location: `docs/GLR_PARSETABLE_QUICKSTART.md`
  - Coverage: Complete pipeline, error handling, advanced usage

### Deferred to Future Releases
- [ ] Add CRC32 checksum support (Optional - low priority)
- [ ] Create `parsetable-validate` CLI tool (Planned for v0.7.0)

---

**Implementation Complete**: All core functionality delivered. See [`docs/GLR_PARSETABLE_QUICKSTART.md`](../GLR_PARSETABLE_QUICKSTART.md) for usage guide.
