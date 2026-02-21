# Requirements Document

## Introduction

This document outlines the requirements for evolving adze into a complete, pure-Rust Tree-sitter language generator ecosystem. The goal is to eliminate all C dependencies from the Tree-sitter grammar development workflow while maintaining full compatibility with the existing Tree-sitter ecosystem. This will make adze THE go-to solution for Tree-sitter integration by providing a faster, safer, and more ergonomic development experience.

### Scope and Assumptions

This project focuses on creating a pure-Rust GLR (Generalized LR) parser generator that produces Tree-sitter-compatible Language objects. Tree-sitter's power comes from its GLR algorithm with compile-time conflict resolution, not simple LR(1) parsing. The following are explicitly out of scope for the initial release:
- Lossless syntax trees (CST) - focus remains on Tree-sitter's AST model
- Query runtime modifications - existing tree-sitter query system remains unchanged
- Alternative parsing algorithms (Earley, PEG, etc.)
- Custom error recovery strategies beyond Tree-sitter's standard approach

### Glossary

- **Language**: A Tree-sitter Language object containing parse tables and metadata
- **IR**: Intermediate Representation - structured grammar data extracted from Rust annotations
- **Action Table**: LR parser table mapping (state, symbol) → action (shift/reduce/accept/error)
- **Alias Sequence**: Tree-sitter feature for renaming nodes in parse trees
- **Production ID**: Unique identifier for grammar rules used in alias mapping
- **MSRV**: Minimum Supported Rust Version (1.78 for this project)

## Requirements

### Requirement 1: Pure-Rust GLR Parser Generator

**User Story:** As a language grammar developer, I want to generate Tree-sitter parsers entirely in Rust, so that I don't need a C compiler or any C dependencies in my build process.

#### Acceptance Criteria

1. WHEN a developer defines a grammar using adze macros THEN the system SHALL generate GLR parse tables entirely in Rust without invoking the C-based tree-sitter CLI
2. WHEN the generator processes a grammar THEN it SHALL produce static Rust constants containing all necessary parse tables (action table, goto table, symbol metadata) with support for multiple actions per (state, lookahead) pair
3. WHEN the generated parser encounters ambiguity THEN it SHALL implement GLR fork/merge logic identical to Tree-sitter's C implementation
4. IF a grammar contains conflicts THEN the system SHALL resolve them using precedence and associativity rules identical to the C implementation, including static and dynamic precedence as well as fragile tokens
5. WHEN building for WebAssembly THEN the system SHALL compile without requiring any C toolchain or external dependencies
6. WHEN generating tables THEN the system SHALL emit alias-sequence and production-id tables identical to C output
7. WHEN parse table compression is applied THEN the system SHALL replicate Tree-sitter's "small table" optimization with bit-for-bit compatibility

### Requirement 2: Complete Grammar IR System

**User Story:** As a adze maintainer, I want a comprehensive intermediate representation for grammars, so that the system can handle all Tree-sitter grammar features consistently.

#### Acceptance Criteria

1. WHEN a grammar is processed THEN the system SHALL extract all grammar rules, tokens, precedences, and conflicts into a structured IR
2. WHEN the IR is built THEN it SHALL support all Tree-sitter grammar features including external scanners, field names, and node supertypes
3. WHEN precedence rules are defined THEN the IR SHALL capture and preserve all precedence and associativity information
4. IF a grammar uses external tokens THEN the IR SHALL properly represent the external scanner interface requirements
5. WHEN conflicts are declared THEN the IR SHALL maintain the conflict resolution strategy for table generation
6. WHEN field names are processed THEN the IR SHALL maintain lexicographic ordering of field names to satisfy Tree-sitter ABI requirements

### Requirement 3: Static Language Generation

**User Story:** As a grammar user, I want the generated parser to be a static Rust object, so that I get zero-cost abstractions and optimal performance.

#### Acceptance Criteria

1. WHEN tables are generated THEN the system SHALL produce static Rust constants that can be embedded directly in the binary
2. WHEN a Language is requested THEN the system SHALL return a tree_sitter::Language constructed from static Rust data
3. WHEN the parser runs THEN it SHALL achieve performance within 5% of the C implementation for parsing speed
4. IF table compression is enabled THEN the system SHALL generate compressed tables while maintaining compatibility
5. WHEN multiple grammars are used THEN each SHALL have its own static Language with no runtime overhead

### Requirement 4: External Scanner Integration

**User Story:** As a grammar developer using external scanners, I want seamless integration with my Rust scanner implementation, so that I can handle complex lexical requirements without C code.

#### Acceptance Criteria

1. WHEN a grammar declares external tokens THEN the system SHALL generate appropriate FFI glue code for the scanner
2. WHEN a Rust scanner is provided THEN the system SHALL automatically wire it into the Language's external scanner interface
3. WHEN scanner state needs persistence THEN the system SHALL handle serialization/deserialization through safe Rust interfaces
4. IF incremental parsing is used THEN the scanner state SHALL be properly maintained across parse sessions
5. WHEN the scanner encounters errors THEN the system SHALL propagate them through the standard Tree-sitter error handling mechanism

### Requirement 5: Build System Integration

**User Story:** As a developer using adze, I want a simple build.rs integration, so that parser generation happens automatically during compilation.

#### Acceptance Criteria

1. WHEN a project includes adze THEN the build.rs SHALL automatically detect grammar changes and regenerate parsers
2. WHEN grammar files are modified THEN the build system SHALL trigger recompilation of only the affected components
3. WHEN building for different targets THEN the system SHALL generate appropriate code for each target without manual intervention
4. IF the build fails THEN the system SHALL provide clear error messages indicating the specific grammar issues
5. WHEN using cargo features THEN the system SHALL respect feature flags for optional components like table compression

### Requirement 6: Backward Compatibility

**User Story:** As an existing adze user, I want to upgrade to the pure-Rust implementation, so that I can benefit from improved performance without changing my code.

#### Acceptance Criteria

1. WHEN upgrading from the current adze THEN existing grammar definitions SHALL continue to work without modification
2. WHEN the generated parser is used THEN it SHALL produce identical parse trees to the C implementation
3. WHEN queries are run THEN they SHALL work identically with both C and Rust-generated parsers
4. IF a project needs the C fallback THEN it SHALL be available through a feature flag
5. WHEN migrating THEN the public API SHALL remain stable with only internal implementation changes

### Requirement 7: Performance and Memory Efficiency

**User Story:** As a performance-conscious developer, I want the pure-Rust implementation to be faster than the C version, so that I get better performance while gaining safety.

#### Acceptance Criteria

1. WHEN parsing typical source files THEN the Rust implementation SHALL be at least as fast as the C implementation
2. WHEN handling large files THEN memory usage SHALL not exceed 110% of the C implementation
3. WHEN using incremental parsing THEN performance SHALL degrade gracefully with edit distance
4. IF table compression is enabled THEN parse speed SHALL remain within 10% of uncompressed performance
5. WHEN running benchmarks THEN the system SHALL consistently outperform the C implementation on modern hardware

### Requirement 8: Developer Experience

**User Story:** As a grammar developer, I want excellent tooling and error messages, so that I can debug and iterate on grammars efficiently.

#### Acceptance Criteria

1. WHEN grammar compilation fails THEN the system SHALL provide precise error locations and helpful suggestions
2. WHEN conflicts occur THEN the system SHALL explain the conflict and suggest resolution strategies
3. WHEN debugging parsers THEN the system SHALL provide tools to inspect parse tables and trace parsing decisions
4. IF performance issues arise THEN the system SHALL offer profiling tools to identify bottlenecks
5. WHEN writing grammars THEN the system SHALL provide comprehensive documentation with examples

### Requirement 9: Ecosystem Integration

**User Story:** As a tool developer, I want the pure-Rust parsers to work seamlessly with existing Tree-sitter tooling, so that I can use them in editors, linters, and other tools.

#### Acceptance Criteria

1. WHEN editors load the parser THEN they SHALL work identically to C-based parsers
2. WHEN syntax highlighting is applied THEN queries SHALL produce identical results
3. WHEN LSP servers use the parser THEN performance SHALL be improved over C implementations
4. IF tree-sitter CLI tools are used THEN they SHALL work with Rust-generated parsers
5. WHEN bindings are created THEN they SHALL support all target languages that support C-based parsers

### Requirement 10: Testing and Quality Assurance

**User Story:** As a adze maintainer, I want comprehensive testing to ensure reliability, so that users can trust the pure-Rust implementation in production.

#### Acceptance Criteria

1. WHEN running corpus tests THEN 100% of existing Tree-sitter grammar test suites SHALL pass
2. WHEN fuzzing the parser THEN it SHALL handle malformed input gracefully without panics
3. WHEN testing incremental parsing THEN all edit scenarios SHALL produce correct results
4. IF memory leaks occur THEN they SHALL be detected and prevented by automated testing, and SHALL run miri or cargo-sanitize in CI to detect undefined behavior
5. WHEN releasing THEN the system SHALL pass comprehensive integration tests across all supported platforms

### Requirement 11: Security and Licensing

**User Story:** As a project maintainer, I want to ensure security and license compliance, so that the generated code is safe and legally compliant.

#### Acceptance Criteria

1. WHEN processing third-party grammars THEN the system SHALL verify grammar licenses are MIT/Apache compatible
2. WHEN generating code THEN the system SHALL escape identifier names to prevent Rust code injection
3. WHEN using unsafe code THEN it SHALL be limited to FFI boundaries only with comprehensive safety documentation
4. IF license incompatibilities are detected THEN the system SHALL fail compilation with clear error messages
5. WHEN building THEN the system SHALL generate a LICENSE-THIRD-PARTY file documenting all dependencies

### Requirement 12: Build System Requirements

**User Story:** As a developer, I want reliable and efficient builds, so that my development workflow is smooth and predictable.

#### Acceptance Criteria

1. WHEN the project is built THEN it SHALL compile with MSRV 1.78 and CI SHALL enforce it
2. WHEN c-backend feature is selected but cc toolchain is unavailable THEN the build SHALL fail fast with a clear error message
3. WHEN grammar files are unchanged THEN the generator SHALL use cached results so rebuilds complete in under 200ms
4. IF table compression is enabled THEN compressed binary size SHALL be ≤60% of uncompressed tables for grammars with ≥5k states
5. WHEN building for no_std targets THEN runtime crates SHALL be compatible with no_std + alloc

### Requirement 13: ABI Compatibility and Versioning

**User Story:** As an ecosystem participant, I want consistent ABI compatibility, so that parsers work reliably across different Tree-sitter versions.

#### Acceptance Criteria

1. WHEN generating Language objects THEN the version SHALL match upstream Tree-sitter minor release within 30 days of upstream bump
2. WHEN multiple Languages with external scanners coexist THEN they SHALL operate in the same process without global mutable state conflicts
3. WHEN generating metadata THEN symbol_is_heap_allocated flags SHALL be populated correctly for UTF-8 input and UTF-16 editor offset conversions
4. IF ABI changes occur upstream THEN the system SHALL detect and adapt to changes automatically
5. WHEN regenerating parsers THEN the tool SHALL be able to produce grammar.json and node-types.json for external consumers