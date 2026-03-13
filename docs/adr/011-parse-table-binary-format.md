# ADR-011: Parse Table Binary Format (Postcard)

## Status

Accepted

## Context

GLR parse tables can be large and expensive to generate:
- **Python grammar**: 273 symbols, 57 fields, 184 rules
- **Generation time**: Can take seconds during build
- **Memory usage**: Significant during table construction

The project needed a way to:
1. **Distribute pre-generated tables**: Avoid build-time generation
2. **Load tables quickly**: Minimize parser startup time
3. **Ensure determinism**: Same grammar produces identical binary output
4. **Support versioning**: Handle format evolution gracefully

### Alternatives Considered

1. **Bincode**: Standard Rust binary serialization
   - Pros: Fast, well-supported, zero-copy options
   - Cons: Not stable across Rust versions, architecture-dependent

2. **JSON**: Human-readable text format
   - Pros: Debuggable, language-agnostic
   - Cons: Large file size, slow parsing

3. **Postcard**: Compact, stable binary format
   - Pros: Small output, stable across versions, `no_std` compatible
   - Cons: Less widely used than bincode

4. **Custom binary format**: Hand-optimized layout
   - Pros: Maximum control and optimization
   - Cons: High maintenance burden, reinventing the wheel

## Decision

We adopted **Postcard** as the serialization format for `.parsetable` files, with a custom file wrapper for metadata and validation.

### File Format Structure

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
│ ParseTable Postcard (variable length)     │ 0x30 + metadata_len
└────────────────────────────────────────────┘
```

### Key Design Choices

#### Magic Number
- **Value**: ASCII "RSPT" (Adze Parse Table)
- **Purpose**: Quick file type identification
- **Bytes**: `0x52 0x53 0x50 0x54`

#### Grammar Hash (SHA-256)
- **Purpose**: Verify table matches expected grammar
- **Computation**: SHA-256 of grammar definition source
- **Use case**: Detect grammar drift between build and runtime

#### Metadata JSON
Human-readable grammar information:
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
    "rust_version": "1.89.0"
  },
  "statistics": {
    "state_count": 273,
    "symbol_count": 57,
    "rule_count": 184,
    "conflict_count": 12
  },
  "features": {
    "glr_enabled": true,
    "external_scanner": true,
    "incremental": false
  }
}
```

#### Postcard for Table Data
- **Compact**: Varint encoding for small integers
- **Stable**: Same output across Rust versions
- **`no_std` compatible**: Works in embedded contexts
- **Fast**: Zero-copy deserialization options

### File Naming Convention

```
<grammar_name>-<version>.parsetable
```

Examples:
- `python-3.12.0.parsetable`
- `javascript-es2024.parsetable`
- `go-1.22.parsetable`

### Loading Workflow

```rust
fn load_parsetable(path: &Path) -> Result<ParseTable, LoadError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    
    // 1. Verify magic number
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"RSPT" {
        return Err(LoadError::InvalidMagic);
    }
    
    // 2. Check format version
    let version = reader.read_u32::<LittleEndian>()?;
    if version > MAX_SUPPORTED_VERSION {
        return Err(LoadError::UnsupportedVersion(version));
    }
    
    // 3. Read grammar hash (for verification)
    let mut hash = [0u8; 32];
    reader.read_exact(&mut hash)?;
    
    // 4. Skip metadata (or parse for diagnostics)
    let metadata_len = reader.read_u32::<LittleEndian>()?;
    reader.seek(SeekFrom::Current(metadata_len as i64))?;
    
    // 5. Read and deserialize table data
    let table_len = reader.read_u32::<LittleEndian>()?;
    let mut table_data = vec![0u8; table_len as usize];
    reader.read_exact(&mut table_data)?;
    
    // 6. Deserialize with Postcard
    let table: ParseTable = postcard::from_bytes(&table_data)?;
    
    Ok(table)
}
```

## Consequences

### Positive

- **Fast parser loading**: Bypass expensive table generation at build time
- **Deterministic builds**: Same grammar produces identical binary
- **Runtime flexibility**: Load different grammars dynamically
- **Size optimization**: Postcard produces compact output
- **Version safety**: Format version enables compatibility checking
- **Grammar verification**: SHA-256 hash detects mismatches
- **Human-readable metadata**: JSON section aids debugging
- **`no_std` support**: Postcard works in embedded contexts

### Negative

- **Binary format**: Not human-readable (partially addressed by metadata)
- **Version management**: Need to handle format evolution
- **Additional dependency**: Postcard crate required
- **Build complexity**: Two-step process (generate then serialize)

### Neutral

- **Bincode alternative**: Could migrate if stability becomes issue
- **Compression optional**: Could add gzip for further size reduction
- **CRC32 optional**: File integrity check available but not required

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md)
- Reference: [docs/archive/specs/PARSETABLE_FILE_FORMAT_SPEC.md](../archive/specs/PARSETABLE_FILE_FORMAT_SPEC.md)
- Reference: [docs/archive/specs/PARSE_TABLE_SERIALIZATION_SPEC.md](../archive/specs/PARSE_TABLE_SERIALIZATION_SPEC.md)
