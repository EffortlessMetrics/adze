# Adze Documentation

This directory contains documentation for Adze.

## Getting Started

- [Getting Started Guide](./GETTING_STARTED.md) - Comprehensive guide to building parsers with macro-based grammars
- [Installation and Quick Start](../README.md#installation) - Get up and running quickly
- [Developer Workflow](./dev-workflow.md) - Linting, testing, and development commands
- [Migration to v0.5](./migration-to-v0.5.md) - Upgrading from earlier versions

## Core Guides

### Grammar Development
- [Grammar Examples](./GRAMMAR_EXAMPLES.md) - Example grammars and patterns
- [Usage Examples](./USAGE_EXAMPLES.md) - Practical usage examples
- [Empty Production Rules](./empty-production-rules.md) - Handling empty rules
- [Empty Rules Quick Reference](./empty-rules-quick-reference.md) - Quick reference for empty rules
- [Optimizer Usage](./optimizer-usage.md) - Grammar optimization guide
- [Precedence Troubleshooting](./precedence-troubleshooting.md) - Debugging precedence issues

### Testing
- [Testing Framework](./TESTING_FRAMEWORK.md) - Comprehensive testing guide
- [Test Strategy](./TEST_STRATEGY.md) - Testing strategies and best practices

### Performance
- [Performance Guide](./PERFORMANCE_GUIDE.md) - Optimization and benchmarking
- [Performance Improvements](./PERFORMANCE_IMPROVEMENTS.md) - Performance enhancement techniques
- [Performance](./PERFORMANCE.md) - General performance information

### Language and Tools
- [Language Support](./LANGUAGE_SUPPORT.md) - Supported language grammars
- [LSP Generator](./LSP_GENERATOR.md) - Generate language servers
- [Playground](./PLAYGROUND.md) - Interactive grammar development (planned)
- [Custom Hover](./how-to-custom-hover.md) - Customizing LSP hover support

## Advanced Topics

### GLR Parsing
- [GLR Internals](./glr_internals.md) - GLR parser implementation details
- [GLR Visualization Guide](./glr-visualization-guide.md) - Visualizing GLR parse trees
- [Goto Indexing Invariants](./goto-indexing-invariants.md) - GLR goto table invariants

### Incremental Parsing
- [Incremental Parsing](./incremental-parsing.md) - Incremental parsing guide

### Query System
- [Predicate Evaluation](./predicate-evaluation.md) - Query predicate evaluation

## Technical Specifications

- [Tree-sitter Table Format Spec](./ts_spec.md) - Tree-sitter compatibility layer
- [Known Limitations](./KNOWN_LIMITATIONS.md) - Current limitations and workarounds

## Development Resources

- [Developer Guide](./DEVELOPER_GUIDE.md) - Developer documentation
- [PR Template](./PR_TEMPLATE.md) - Pull request template

### Cookbooks
- [C++ Templates Cookbook](./cookbook_cpp_templates.md) - Parsing C++ templates

## Historical Documentation

See [archive/](./archive/) for historical documentation, status reports, and planning documents archived in February 2026.

## External Resources

- [Main README](../README.md) - Project overview
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Roadmap](../ROADMAP.md) - Project roadmap and future plans
- [API Documentation](../API_DOCUMENTATION.md) - API reference
