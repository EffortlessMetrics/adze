# Implementation Plan

## Critical Strategic Priorities (Based on Research)

**Key Finding**: Tree-sitter is fundamentally a GLR parser generator, not LR(1). The GLR fork/merge logic and compile-time conflict resolution are core to its functionality and compatibility.

## MVP Roadmap (0.6)

Focus on GLR state machine fidelity, conflict resolution logic, and bit-for-bit table compression compatibility. Performance target: 4-8x faster than current FFI-based Rust bindings (not faster than specialized compiler frontends like rustc).

## Phase 0: Research & Macro Hardening (Week 1)

**CRITICAL**: Fix rust-sitter macro system debuggability before proceeding with GLR implementation.

- [x] 0.1 Fix RUST_SITTER_EMIT_ARTIFACTS debugging (BLOCKING)
  - Investigate and resolve rust-sitter Issue #63: "RUST_SITTER_EMIT_ARTIFACTS=true causes the build to fail"
  - Restore reliable grammar.js and IR artifact emission for debugging
  - Create golden-test pipeline comparing C output vs Rust IR vs round-tripped output
  - Build IR inspection tooling for development workflow
  - _Requirements: Research Priority #2_

- [x] 0.2 Harden macro system for IDE compatibility
  - Implement error-recovering parsing strategies for incomplete TokenStreams
  - Build "partial IR" error reporting that emits diagnostic errors with as much IR as possible
  - Test macro resilience with syntactically incorrect input in IDE scenarios
  - Ensure rust-analyzer can provide features even with broken grammars
  - _Requirements: Research Priority #2_

- [x] 0.3 Set up GLR-aware project structure
  - Create workspace structure with ir/, glr-core/, tablegen/, scanner-bridge/ crates
  - Define core types supporting multiple actions per (state, lookahead): SymbolId, RuleId, StateId, FieldId
  - Set up CI matrix with MSRV 1.78, stable, beta, nightly builds
  - Add Miri and address-sanitizer jobs for UB detection from Week 1
  - _Requirements: 12.1, 11.3_

## Phase 1: GLR-Aware IR and Conflict Resolution (Weeks 2-3)

**CRITICAL**: Implement GLR state machine fidelity and conflict resolution logic as core functionality.

- [ ] 1.1 Implement GLR-aware Grammar IR structure
  - Define Grammar, Rule, Token, Precedence structs using IndexMap for deterministic ordering
  - Add support for dynamic precedence (PREC_DYNAMIC), fragile tokens, and alias sequences
  - Implement field allocation with lexicographic ordering and validation
  - Add production_ids and alias_map for complex grammar features
  - Model IR for multiple actions per (state, lookahead) pair to support GLR
  - _Requirements: 2.1, 2.2, 2.6, Research Priority #1_

- [ ] 1.2 Port Tree-sitter's exact conflict resolution logic
  - Implement C's exact logic for rule comparison and conflict pruning
  - Handle subtle interactions of explicit/implicit precedence
  - Parse and preserve all macro annotations faithfully into IR
  - Add IR invariants/testing to catch single-token divergences from C output
  - Implement TSFragile/TSForcedReduce semantics for lexical vs parse conflicts
  - _Requirements: 1.4, Research Priority #2_

- [ ] 1.3 Create grammar extraction with emit_ir!() macro
  - Implement rust-sitter::emit_ir!() macro to generate const GRAMMAR_JSON
  - Build Grammar::from_json() parser with comprehensive validation
  - Extract IR from sample grammars and serialize back to JSON identical to tree-sitter generate --json
  - Create round-trip testing: C output vs Rust IR vs round-tripped output
  - _Requirements: 2.1, 13.5_

## Phase 2: GLR State Machine and Parse Table Generation (Weeks 4-6)

- [ ] 2.1 Implement GLR state machine construction
  - Build FIRST/FOLLOW computation using FixedBitSet for efficient set operations
  - Implement GLR item set collection with support for multiple actions per state
  - Create closure and goto operations with deterministic ordering
  - Add support for fork/merge points in the state machine
  - _Requirements: 1.1, 1.3, Research Priority #1_

- [ ] 2.2 Generate GLR-compatible parse tables
  - Create ParseTable supporting multiple actions per (state, lookahead) pair
  - Implement conflict detection that preserves ambiguity for GLR resolution
  - Apply precedence/associativity rules to prune conflicts at generation time
  - Add table validation against Tree-sitter CLI output with bit-for-bit comparison
  - _Requirements: 1.1, 1.3, 3.1, Research Priority #1_

- [ ] 2.3 Implement Tree-sitter's table compression exactly
  - Replicate "small table" factoring (ts_small_parse_table, ts_small_parse_table_map)
  - Implement indexed/offset layout and one-dimensional array compression
  - Serialize in-memory IR as const Rust arrays bit-for-bit identical to parser.c output
  - Build test suite that round-trips C's output and Rust output for identical grammars
  - _Requirements: 1.7, 3.1, Research Priority #3_

## Phase 3: Table Generation and Static Language (Weeks 5-6)

- [ ] 3. Implement table compression matching Tree-sitter
  - Build row-based compression with default actions
  - Implement "small table" optimization for <32k states
  - Add pointer table compression for large state machines
  - Create const-friendly decompression for runtime
  - _Requirements: 3.1, 3.3, 7.2_

- [ ] 3.1 Generate static Language objects
  - Create StaticLanguageGenerator with TokenStream output
  - Generate static arrays: parse tables, symbol metadata, field names
  - Build Language constructor with proper FFI compatibility
  - Add thread-safety assertions and OnceLock wrapping
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 3.2 Implement symbol and metadata generation
  - Generate symbol_names array with deterministic ordering
  - Create symbol_metadata with visible/named/supertype flags
  - Build field_names array with lexicographic sorting
  - Generate NODE_TYPES JSON with full compatibility
  - _Requirements: 3.2, 3.3, 6.3_

- [ ] 3.3 Add Language validation and compatibility testing
  - Test Parser::set_language() integration
  - Validate parse tree equivalence with C implementation
  - Add query compatibility verification
  - Create performance benchmarks against C parser
  - _Requirements: 6.1, 6.3, 7.1_

## Phase 4: External Scanner Integration (Week 7)

- [ ] 4. Build external scanner FFI bridge
  - Define ExternalScanner trait with type-safe interface
  - Generate extern "C" functions for Tree-sitter integration
  - Implement state serialization/deserialization
  - Add safety assertions and error handling
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 4.1 Create scanner integration utilities
  - Build generate_scanner_bridge!() macro
  - Implement scanner state persistence for incremental parsing
  - Add scanner error propagation through Tree-sitter
  - Create scanner testing utilities and fixtures
  - _Requirements: 4.1, 4.4, 4.5_

- [ ] 4.2 Test scanner integration with real examples
  - Port existing Racket here-string scanner
  - Test heredoc handling with complex delimiters
  - Validate incremental parsing with scanner state
  - Add fuzzing for scanner edge cases
  - _Requirements: 4.1, 4.4, 10.2_

## Phase 5: Build System Integration (Week 8)

- [ ] 5. Refactor build.rs integration
  - Remove all cc and tree-sitter-generate dependencies
  - Implement pure-Rust generation pipeline
  - Add incremental build support with rule-level caching
  - Create proper rerun-if-changed directives
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 5.1 Add build configuration and feature management
  - Implement BuildConfig with comprehensive options
  - Add feature flag handling for optional components
  - Create build-time validation and error reporting
  - Add cache statistics and debugging output
  - _Requirements: 5.1, 5.4, 5.5_

- [ ] 5.2 Test cross-platform build compatibility
  - Validate Linux, macOS, Windows builds
  - Test WebAssembly compilation without C toolchain
  - Add no-std compatibility for embedded targets
  - Create CI matrix for all supported platforms
  - _Requirements: 5.3, 9.4, 9.5_

## Phase 6: Advanced Features and Optimization (Week 9)

- [ ] 6. Implement performance optimizations
  - Add parallel table generation with rayon
  - Implement memory-efficient data structures
  - Create table compression with size optimization
  - Add runtime performance tuning
  - _Requirements: 7.1, 7.2, 7.3_

- [ ] 6.1 Add developer experience improvements
  - Implement colorized error messages with ariadne
  - Create conflict resolution suggestions
  - Build grammar visualization tools
  - Add debugging and profiling utilities
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 6.2 Implement ABI 15 compliance and compatibility
  - Implement #[repr(C)] Language struct with exact field layout matching C
  - Expose all ABI 15 metadata functions (language name, version, supertypes, reserved words)
  - Load and process tree-sitter.json for metadata embedding
  - Add ABI compliance testing against multiple Tree-sitter versions
  - _Requirements: 13.1, 13.2, 13.3, Research Priority #5_

## Phase 7: Testing and Quality Assurance (Week 10)

- [ ] 7. Comprehensive testing and validation
  - Run full corpus tests with 100% pass rate
  - Execute fuzzing campaigns for robustness
  - Validate incremental parsing correctness
  - Test memory usage and leak detection
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 7.1 Performance benchmarking and optimization
  - Compare parsing speed with C implementation
  - Measure memory usage across different grammar sizes
  - Test WebAssembly performance and bundle size
  - Create performance regression detection
  - _Requirements: 7.1, 7.2, 7.3_

- [ ] 7.2 Cross-platform and ecosystem integration testing
  - Test editor integration (VS Code, Neovim, Helix)
  - Validate LSP server performance improvements
  - Test syntax highlighting query compatibility
  - Verify tree-sitter CLI tool integration
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

## Phase 8: Documentation and Release Preparation (Week 11)

- [ ] 8. Create comprehensive documentation
  - Write migration guide from C to pure-Rust
  - Document all APIs with examples
  - Create grammar development tutorial
  - Build troubleshooting and FAQ sections
  - _Requirements: 8.4, 6.4_

- [ ] 8.1 Prepare release infrastructure
  - Set up automated releases with GitHub Actions
  - Create changelog generation
  - Add version compatibility matrix
  - Prepare crates.io publication
  - _Requirements: 6.4_

- [ ] 8.2 Community preparation and contribution guidelines
  - Create contributor documentation
  - Set up issue templates and PR guidelines
  - Build grammar compatibility test suite
  - Prepare community grammar migration tools
  - _Requirements: 8.4_

## Phase 9: Beta Release and Feedback (Week 12)

- [ ] 9. Beta release and community testing
  - Publish rust-sitter 0.5.0-beta
  - Gather community feedback on API and performance
  - Address critical issues and edge cases
  - Refine documentation based on user experience
  - _Requirements: 6.4_

- [ ] 9.1 Ecosystem integration validation
  - Test with major Tree-sitter grammars (Rust, TypeScript, Python)
  - Validate editor plugin compatibility
  - Test language server integration
  - Measure real-world performance improvements
  - _Requirements: 9.1, 9.2, 9.3_

- [ ] 9.2 Prepare stable release
  - Address all beta feedback
  - Finalize API stability guarantees
  - Complete security audit of unsafe code
  - Prepare 1.0.0 release announcement
  - _Requirements: 6.1, 6.2, 6.3_

## Success Criteria

Each phase must meet these criteria before proceeding:

**Phase 0**: RUST_SITTER_EMIT_ARTIFACTS works reliably and macro system handles incomplete input gracefully
**Phase 1**: Grammar IR can parse and validate all major Tree-sitter grammars with GLR support
**Phase 2**: GLR parse tables match Tree-sitter CLI output bit-for-bit for test grammars
**Phase 3**: Generated Language objects work with tree_sitter::Parser and pass ABI 15 compliance
**Phase 4**: External scanners integrate seamlessly with complex lexical rules
**Phase 5**: Build system works without C toolchain on all platforms
**Phase 6**: Performance achieves 4-8x improvement over FFI-based Rust bindings
**Phase 7**: 100% corpus compatibility and robust fuzzing results
**Phase 8**: Complete documentation and migration guides
**Phase 9**: Successful community beta testing and feedback integration

## Risk Mitigation

**Technical Risks**:
- GLR algorithm complexity and fork/merge logic → Extensive unit testing with known ambiguous grammars and golden file comparison
- Table compression bugs → Bit-for-bit golden file tests against Tree-sitter CLI output
- Performance regressions → Continuous benchmarking with failure gates targeting 4-8x FFI improvement
- Macro system fragility → Early Phase 0 focus on debugging and error recovery

**Project Risks**:
- Scope creep → Strict phase boundaries with success criteria
- Community adoption → Early beta release and feedback integration
- Maintenance burden → Comprehensive test suite and documentation

**Timeline Risks**:
- Algorithm implementation delays → Focus on correctness over optimization initially
- Integration complexity → Phased approach with working increments
- Testing bottlenecks → Parallel development of test infrastructure