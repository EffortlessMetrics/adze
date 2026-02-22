# Adze Documentation

> **Status:** Documentation is being actively refreshed for Adze 0.8.0-dev.

## Core Guides

- [**Getting Started**](./GETTING_STARTED.md) - The best place to start. Build your first parser in 5 minutes.
- [**Grammar Examples**](./GRAMMAR_EXAMPLES.md) - Patterns for common language constructs (JSON, Math, etc).
- [**API Documentation**](../API_DOCUMENTATION.md) - Reference for the `adze` crate and macros.
- [**Known Limitations**](./KNOWN_LIMITATIONS.md) - Current status of experimental features and grammar compatibility.

## For Contributors

- [**Developer Guide**](./DEVELOPER_GUIDE.md) - How to build, test, and contribute to the Adze project.
- [**Testing Framework**](./TESTING_FRAMEWORK.md) - Overview of our testing strategy (unit, snapshot, golden tests).
- [**Roadmap**](../ROADMAP.md) - Our goals for 0.8.0, 0.9.0, and 1.0.

## Advanced & Experimental

- [**GLR Architecture**](./explanations/glr-incremental-architecture.md) - Deep dive into our Generalized LR parsing engine.
- [**Performance Guide**](./PERFORMANCE_GUIDE.md) - Tips for optimizing your grammar and runtime performance.
- [**LSP Generator**](./LSP_GENERATOR.md) - Experimental tool to generate Language Servers from grammars.
- [**Incremental Parsing**](./incremental-parsing.md) - How Adze handles partial reparses after text edits.
- [**Language Support**](./LANGUAGE_SUPPORT.md) - Status of built-in grammars (Python, JS, Go).

---

## Status Tracking

- [**Now / Next / Later**](./status/NOW_NEXT_LATER.md) - Current execution plan.
- [**Friction Log**](./status/FRICTION_LOG.md) - Tracking and burning down developer pain points.
- [**Known Red**](./status/KNOWN_RED.md) - Intentional exclusions from the supported CI lane.
