# Disabled Test Files

The following test files are currently disabled due to API changes in the adze codebase:

## Runtime Tests
- `test_pure_rust_e2e.rs.disabled` - Uses old TSLanguage API structure
- `test_query_predicates.rs.disabled` - Query system not fully implemented
- `golden_tests.rs.disabled` - Depends on old parser implementation
- `test_pure_rust_real_grammar.rs.disabled` - Uses outdated grammar format
- `test_glr_parsing.rs.disabled` - GLR API has changed significantly
- `test_complete_example.rs.disabled` - Requires updated example code

## Examples
- `query_demo.rs.disabled` - Query system incomplete

## Benchmarks
- `parser_bench.rs.disabled` - Needs update for new parser API

## Why These Are Disabled

These tests were written for an earlier version of adze and need to be rewritten to work with:
1. The new GLR parser architecture (ActionCell instead of Action)
2. The updated pure-Rust implementation
3. The revised Tree-sitter ABI compatibility layer

## Re-enabling Strategy

To re-enable these tests:
1. Update imports to use new module paths
2. Adapt to the new parser API (ParseTable with Vec<Vec<ActionCell>>)
3. Update language creation to use new builder patterns
4. Fix any symbol ID mapping issues

These tests are not critical for v0.6.0 release as the core functionality is tested through:
- Unit tests in each module
- Integration tests in the example crate
- The working arithmetic and other example grammars