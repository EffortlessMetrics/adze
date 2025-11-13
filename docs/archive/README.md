# Documentation Archive

This directory contains historical documentation files from previous versions and development phases of rust-sitter. These files are preserved for reference but are **no longer actively maintained**.

## Why Files Are Archived

- **Version-specific documentation**: Files related to v0.5.0-beta, v0.6.0, and earlier versions
- **Superseded planning documents**: Multiple roadmap files and status documents that have been consolidated into current plans
- **Historical tracking**: Phase summaries, release notes, and status documents from earlier development

## Current Documentation (Use These Instead)

For current development information, please refer to:

### For Users Evaluating the Project
- **[README.md](../README.md)** - Project overview with honest status assessment
- **[PROJECT_STATE_v0.8.0-dev.md](../PROJECT_STATE_v0.8.0-dev.md)** - Comprehensive capability assessment and gap analysis
  - What actually works ✅
  - Critical gaps ⚠️
  - Test results (379/385 passing)
  - Recommendations for users
  - Timeline to v1.0.0

### For Contributors & Developers
- **[IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md)** - 5-phase roadmap to fix critical gaps
  - Phase 1: Transform Functions (3-4 weeks)
  - Phase 2: Real Benchmarks (2 weeks)
  - Phase 3: External Scanners (4-6 weeks)
  - Phase 4: Grammar Testing (2-3 weeks)
  - Phase 5: Production Release (2-3 weeks)
- **[CLAUDE.md](../CLAUDE.md)** - Developer guidelines and architecture overview
- **[KNOWN_LIMITATIONS.md](../KNOWN_LIMITATIONS.md)** - Technical gaps and feature compatibility matrix
- **[CRITICAL_ISSUES_SUMMARY.md](../CRITICAL_ISSUES_SUMMARY.md)** - Detailed analysis of known issues

### For Technical Deep Dives
- **[PURE_RUST_IMPLEMENTATION.md](../PURE_RUST_IMPLEMENTATION.md)** - Pure-Rust architecture and implementation status
- **[book/src/](../book/src/)** - Complete API documentation and guides

### For Project Navigation
- **[DOCUMENTATION.md](../DOCUMENTATION.md)** - Documentation index and quick navigation guide

## Contents of This Archive (35 files)

### Version-Specific Documentation
- `BETA_FIXES_SUMMARY.md` - v0.5.0-beta fixes
- `BETA_RELEASE_SUMMARY.md` - v0.5.0-beta release notes
- `DOCUMENTATION_UPDATE_SUMMARY_v0.6.0.md` - v0.6.0 documentation updates
- `RELEASE_CHECKLIST_v0.5.0-beta.md` - v0.5.0-beta release checklist
- `RELEASE_NOTES_v0.5.0-beta.md` - v0.5.0-beta release notes
- `RELEASE_NOTES_v0.6.1-beta.md` - v0.6.1-beta release notes
- `RELEASE_STATUS_v0.5.0-beta.md` - v0.5.0-beta status
- `RELEASE_v0.6.0-beta.1.md` - v0.6.0-beta.1 release notes
- `RELEASE_v0.6.0.md` - v0.6.0 release notes
- `QUICKSTART_BETA.md` - v0.5.0-beta quickstart

### Obsolete Planning Documents
- `API_FREEZE_SUMMARY.md` - Old API stability planning
- `FIXES_SUMMARY.md` - Old fixes tracking
- `GLR_STATUS.md` - Superseded by PROJECT_STATE_v0.8.0-dev.md
- `IMPLEMENTATION_ROADMAP.md` - Replaced by IMPLEMENTATION_PLAN.md
- `IMPLEMENTATION_STATUS.md` - Replaced by PROJECT_STATE_v0.8.0-dev.md
- `IMPLEMENTATION_SUMMARY.md` - Replaced by PROJECT_STATE_v0.8.0-dev.md
- `PHASE_7_SUMMARY.md` - Old phase tracking
- `PHASE_8_SUMMARY.md` - Old phase tracking
- `PROJECT_COMPLETION_SUMMARY.md` - Misleading completion status
- `PROJECT_STATUS.md` - Superseded by PROJECT_STATE_v0.8.0-dev.md
- `PURE_RUST_BACKEND_ROADMAP.md` - Replaced by IMPLEMENTATION_PLAN.md
- `PURE_RUST_IMPLEMENTATION_ROADMAP.md` - Replaced by IMPLEMENTATION_PLAN.md
- `PURE_RUST_SUMMARY.md` - Covered by PURE_RUST_IMPLEMENTATION.md
- `QUERY_IMPLEMENTATION_SUMMARY.md` - Old tracking
- `RELEASE_CHECKLIST.md` - Generic release checklist
- `RELEASE_NOTES.md` - Generic release notes
- `RELEASE_SUMMARY.md` - Old summary

### Multiple Roadmap Consolidations
- `ROADMAP.md` - Consolidated into IMPLEMENTATION_PLAN.md
- `ROADMAP-0.8.0.md` - Consolidated into IMPLEMENTATION_PLAN.md
- `ROADMAP_2025.md` - Consolidated into IMPLEMENTATION_PLAN.md
- `ROADMAP_TO_FULL_COMPATIBILITY.md` - Replaced by IMPLEMENTATION_PLAN.md

### Old Tracking Files
- `glr-core-parity-status.md` - Superseded by current test results
- `INTERNAL_RELEASE_CHECKLIST.md` - Old release process
- `CRITICAL_FIXES_COMPLETED.md` - Historical fixes
- `TRACKING_ISSUES.md` - Old issue tracking

## How to Use This Archive

### For Historical Context
If you need to understand decisions made in earlier versions (v0.5.0-beta through v0.6.0), files here provide that historical context.

### For Migration Information
If upgrading from an older version, the version-specific release notes may be helpful:
- Upgrading from v0.5.0-beta → v0.8.0-dev: See release notes in order
- Upgrading from v0.6.0+ → v0.8.0-dev: See DOCUMENTATION_UPDATE_SUMMARY_v0.6.0.md

### For Abandoned Features
Some files document features or approaches that were abandoned. Reference these if:
- You're curious about historical design decisions
- You're investigating "why didn't they do X?"
- You need to understand prior art before proposing changes

## Current Status

**Archive Created**: November 13, 2025
**Project Status**: rust-sitter v0.8.0-dev (Early Development)
**Consolidation**: 35 files consolidated from active documentation

## Migration Path

If you find references to archived files in the main documentation or code:
1. Check if current docs cover the same topic
2. If not, report an issue on GitHub
3. The archived file can help provide historical context while we update docs

---

For the latest updates, see [DOCUMENTATION.md](../DOCUMENTATION.md) for a navigation guide to all current documentation.
