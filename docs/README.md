# Rust Sitter Documentation

This directory contains comprehensive documentation for Rust Sitter.

## Getting Started

- [Installation & Quick Start](../README.md#installation) - Get up and running quickly
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

### Language & Tools
- [Language Support](./LANGUAGE_SUPPORT.md) - Supported language grammars
- [LSP Generator](./LSP_GENERATOR.md) - Generate language servers
- [Playground](./PLAYGROUND.md) - Interactive grammar development
- [Custom Hover](./how-to-custom-hover.md) - Customizing LSP hover support

## Advanced Topics

### GLR Parsing
- [GLR Internals](./glr_internals.md) - GLR parser implementation details
- [GLR Visualization Guide](./glr-visualization-guide.md) - Visualizing GLR parse trees
- [GLR Guardrails](./GLR_GUARDRAILS.md) - GLR safety mechanisms
- [Goto Indexing Invariants](./goto-indexing-invariants.md) - GLR goto table invariants

### Incremental Parsing
- [Incremental Parsing](./incremental-parsing.md) - Incremental parsing guide

### Query System
- [Predicate Evaluation](./predicate-evaluation.md) - Query predicate evaluation

## Technical Specifications

- [Tree-sitter Table Format Spec](./ts_spec.md) - Tree-sitter compatibility layer
- [Compatibility Dashboard](./compatibility-dashboard.md) - Compatibility tracking
- [Known Limitations](./KNOWN_LIMITATIONS.md) - Current limitations and workarounds

## Development Resources

### Process & Workflows
- [Developer Guide](./DEVELOPER_GUIDE.md) - Developer documentation
- [PR Template](./PR_TEMPLATE.md) - Pull request template
- [PR Hardening](./PR_HARDENING.md) - PR quality guidelines
- [PR Description](./PR_DESCRIPTION.md) - Writing effective PR descriptions
- [Merge Checklist](./MERGE_CHECKLIST.md) - Pre-merge checklist
- [Merge Gate Checklist](./MERGE_GATE_CHECKLIST.md) - Merge gate requirements

### Infrastructure
- [Git Hooks Optimization](./git-hooks-optimization.md) - Optimizing git hooks
- [Hook Hardening Summary](./hook-hardening-summary.md) - Git hook improvements
- [Stabilization Summary](./stabilization-summary.md) - Project stabilization efforts

### Cookbooks
- [C++ Templates Cookbook](./cookbook_cpp_templates.md) - Parsing C++ templates

## Implementation Details

See [implementation/](./implementation/) for detailed implementation documentation:

- [GLR Status](./implementation/GLR_STATUS.md) - GLR implementation status
- [GLR Incremental Design](./implementation/GLR_INCREMENTAL_DESIGN.md) - Incremental GLR design
- [Implementation Status](./implementation/IMPLEMENTATION_STATUS.md) - Overall implementation status
- [Implementation Roadmap](./implementation/IMPLEMENTATION_ROADMAP.md) - Implementation roadmap
- [Pure Rust Implementation](./implementation/PURE_RUST_IMPLEMENTATION.md) - Pure Rust backend

## Roadmaps & Planning

See [roadmaps/](./roadmaps/) for project roadmaps:

- [Roadmap](./roadmaps/ROADMAP.md) - Main project roadmap
- [Roadmap 2025](./roadmaps/ROADMAP_2025.md) - 2025 roadmap
- [Roadmap 0.8.0](./roadmaps/ROADMAP-0.8.0.md) - Version 0.8.0 roadmap
- [Roadmap to Full Compatibility](./roadmaps/ROADMAP_TO_FULL_COMPATIBILITY.md) - Compatibility goals
- [Incremental GLR Roadmap](./INCR_GLR_ROADMAP.md) - Incremental GLR roadmap

## Release Information

See [releases/](./releases/) for release documentation:

- [Release Notes](./releases/RELEASE_NOTES.md) - Latest release notes
- [Release Checklist](./releases/RELEASE_CHECKLIST.md) - Release process checklist
- [Changelog](./releases/CHANGELOG.md) - Project changelog

## Historical Documentation

See [archive/](./archive/) for historical documentation and status reports.

## External Resources

- [Main README](../README.md) - Project overview
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Project Status](../PROJECT_STATUS.md) - Current project status
- [API Documentation](../API_DOCUMENTATION.md) - API reference
- [Migration Guide](../MIGRATION_GUIDE.md) - Migration from Tree-sitter
