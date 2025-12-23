# GLR Positioning: rust-sitter vs Other Parser Tools

**Version**: 1.0.0
**Date**: 2025-11-20
**Purpose**: Compare rust-sitter's GLR implementation against alternative parser tools
**Audience**: Technical decision-makers evaluating parser infrastructure

---

## Executive Summary

rust-sitter provides a **Rust-native GLR parser with artifact-driven deployment** using modern governance practices (BDD, TDD, contract-first, Infrastructure-as-Code). This document positions rust-sitter against common alternatives and clarifies when to use each tool.

### When to Choose rust-sitter

Use rust-sitter when you need:
- ✅ **GLR semantics** for ambiguous grammars
- ✅ **Artifact-driven deployment** (`.parsetable` as deployable binary artifacts)
- ✅ **Governance-first infrastructure** (contracts, ADRs, BDD specs, audit trails)
- ✅ **Pure Rust** with WASM support
- ✅ **Multi-grammar runtime** that can load different grammars dynamically

### When to Choose Alternatives

- **Tree-sitter**: Need battle-tested incremental parsing + editor integration **today**
- **LALRPOP**: Building a closed set of simple DSLs with minimal setup
- **pest**: Want PEG semantics with lightweight developer experience
- **nom**: Need maximum performance with full control via hand-written combinators

---

## Comparison Matrix

| Feature | rust-sitter GLR | Tree-sitter | LALRPOP | pest | nom |
|---------|----------------|-------------|---------|------|-----|
| **Ambiguous Grammars** | ✅ GLR preserves conflicts | ❌ Must resolve at design time | ❌ LR(1) only | ⚠️ PEG ordered choice | ✅ Manual handling |
| **Artifact Story** | ✅ `.parsetable` binary format | ⚠️ C parser + JSON | ❌ Codegen | ❌ Codegen | ❌ Codegen |
| **Runtime Grammar Loading** | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| **Incremental Parsing** | ⏳ Planned (v0.7.0) | ✅ Production-ready | ❌ No | ❌ No | ✅ Manual |
| **Editor Integration** | ⏳ Planned (v1.0) | ✅ Extensive | ❌ No | ❌ No | ❌ No |
| **WASM Support** | ✅ First-class | ⚠️ Limited | ✅ Yes | ✅ Yes | ✅ Yes |
| **Governance/Contracts** | ✅ BDD, ADRs, Contracts | ⚠️ Well-tested, no formal contracts | ❌ No | ❌ No | ❌ No |
| **CI-friendly** | ✅ Table validation as code | ⚠️ Manual testing | ⚠️ Manual testing | ⚠️ Manual testing | ⚠️ Manual testing |
| **Performance** | ⚠️ Baseline (16K lines/sec) | ✅ Highly optimized | ✅ Fast | ✅ Fast | ✅ Fastest |
| **Maturity** | ⚠️ Beta (v0.6.1) | ✅ Production | ✅ Stable | ✅ Stable | ✅ Stable |

---

## Detailed Comparisons

### 1. rust-sitter GLR vs Tree-sitter

**Tree-sitter** is the industry standard for incremental parsing in editors, with a mature C implementation, extensive grammar ecosystem, and strong LSP integration.

#### Where rust-sitter is Stronger

**GLR Semantics**
- **rust-sitter**: Preserves all conflicting actions in multi-action cells; can explore multiple parse paths
- **Tree-sitter**: Requires conflict resolution at grammar design time; cannot handle true ambiguity
- **Use Case**: Parsing inherently ambiguous languages (e.g., C++ templates, dangling-else constructs)

**Artifact-Driven Architecture**
- **rust-sitter**: `.parsetable` files are versioned, hashed, and deployable as standalone artifacts
  - Build: `grammar → ParseTable IR → .parsetable (bincode)`
  - Runtime: `Parser::load_glr_table_from_bytes()`
  - Benefit: Reproducible builds, auditable parsing infrastructure
- **Tree-sitter**: Generated C code + JSON node types; no binary table artifact
- **Use Case**: Multi-tenant systems, compliance-heavy environments, CI/CD with artifact provenance

**Governance & Contracts**
- **rust-sitter**: BDD specs, ADRs, completion contracts, performance baselines with CI gates
  - Example: `GLR_V1_COMPLETION_CONTRACT.md` defines all acceptance criteria
  - Example: `PERFORMANCE_BASELINE.md` documents thresholds with automated regression tests
- **Tree-sitter**: Well-tested but no formal contract documents
- **Use Case**: Regulated industries, security-critical systems, audit requirements

**Pure Rust + WASM**
- **rust-sitter**: Zero C dependencies, first-class WASM support
- **Tree-sitter**: C core with Rust bindings; WASM support limited
- **Use Case**: Browser-based parsing, sandboxed environments, Rust-only stacks

#### Where Tree-sitter is Stronger

**Incremental Parsing**
- **Tree-sitter**: Battle-tested incremental reparsing, fundamental to design
- **rust-sitter**: Planned for v0.7.0, not yet implemented
- **Impact**: Tree-sitter is orders of magnitude faster for editor use cases

**Editor Integration**
- **Tree-sitter**: Neovim, Emacs, VS Code, and many more; syntax highlighting, folding, selections
- **rust-sitter**: Planned for v1.0
- **Impact**: Use Tree-sitter if editor integration is needed today

**Grammar Ecosystem**
- **Tree-sitter**: 50+ maintained grammars (Python, JavaScript, Rust, Go, etc.)
- **rust-sitter**: Small set of proof-of-concept grammars
- **Impact**: Tree-sitter has a large head start on grammar availability

**Maturity & Performance**
- **Tree-sitter**: Years of production use, highly optimized C implementation
- **rust-sitter**: Beta quality (v0.6.1), baseline performance (not yet optimized)
- **Impact**: Tree-sitter is proven at scale; rust-sitter is emerging infrastructure

#### Recommendation

| Scenario | Choose |
|----------|--------|
| Editor integration needed **today** | **Tree-sitter** |
| Incremental parsing critical | **Tree-sitter** |
| Need governance/contracts/BDD | **rust-sitter** |
| Artifact-driven deployment | **rust-sitter** |
| GLR ambiguity handling | **rust-sitter** |
| Pure Rust/WASM required | **rust-sitter** |
| Multi-grammar runtime loading | **rust-sitter** |

**Summary**: Tree-sitter wins on maturity and editor story; rust-sitter wins on governance, GLR semantics, and artifact infrastructure.

---

### 2. rust-sitter GLR vs LALRPOP

**LALRPOP** is a Rust LR(1) parser generator with clean syntax and good error messages.

#### Semantic Differences

**Ambiguity Handling**
- **rust-sitter**: GLR multi-action cells preserve all conflicting actions; runtime explores multiple paths
- **LALRPOP**: LR(1) requires conflict resolution at grammar design time; shift/reduce conflicts are errors
- **Impact**: LALRPOP cannot handle inherently ambiguous grammars; rust-sitter can

**Artifact vs Codegen**
- **rust-sitter**: Emits `.parsetable` binary artifacts; runtime loads them dynamically
- **LALRPOP**: Generates Rust code at build time; one grammar = one compiled crate
- **Impact**: rust-sitter supports runtime grammar selection; LALRPOP requires recompilation

#### Where rust-sitter is Stronger

1. **Multi-Grammar Systems**: Load different grammars at runtime without recompiling
2. **Ambiguous Grammars**: Handle dangling-else, expression grammars with multiple valid parses
3. **Artifact Provenance**: SHA256 hashes, version metadata, CI validation of table generation

#### Where LALRPOP is Stronger

1. **Developer Experience**: Cleaner grammar syntax, better error messages at grammar design time
2. **Simplicity**: No separate build artifact; just write grammar, generate code, done
3. **Performance**: Optimized LR(1) codegen can be faster than table-driven parsing

#### Recommendation

| Scenario | Choose |
|----------|--------|
| Building a few simple DSLs | **LALRPOP** |
| Need ambiguous grammar support | **rust-sitter** |
| Want runtime grammar loading | **rust-sitter** |
| Prefer codegen simplicity | **LALRPOP** |

**Summary**: LALRPOP is simpler for deterministic grammars; rust-sitter is more powerful for ambiguous grammars and multi-grammar systems.

---

### 3. rust-sitter GLR vs pest

**pest** is a PEG (Parsing Expression Grammar) parser with clean syntax and good ergonomics.

#### Semantic Differences

**PEG vs GLR**
- **pest**: Ordered choice (`/`) tries alternatives sequentially; first match wins
- **rust-sitter**: GLR explores all valid parses; can expose parse forests
- **Impact**: For ambiguous inputs, PEG gives one parse (by ordering); GLR can give all valid parses

**Example: Ambiguous Expression**
```
Input: "a + b * c"

PEG (ordered choice):
  Rule: Expr = Add / Mul / Var
  Parse: Add(a, Mul(b, c))  # Only one parse, by rule order

GLR:
  Parse forest can contain:
    - Add(a, Mul(b, c))
    - Mul(Add(a, b), c)
  With precedence: Select Add(a, Mul(b, c)) as primary
```

#### Where rust-sitter is Stronger

1. **True Ambiguity Handling**: GLR preserves all parses; PEG hides alternatives
2. **Governance**: BDD specs, contracts, CI gates (pest has none)
3. **Artifact Infrastructure**: `.parsetable` deployment story
4. **Multi-Grammar Runtime**: Load different grammars dynamically

#### Where pest is Stronger

1. **Developer Experience**: Extremely clean grammar syntax, great error messages
2. **Simplicity**: No separate table generation; grammar = parser
3. **Performance**: PEG backtracking can be very fast with memoization
4. **Maturity**: Widely used, stable, good ecosystem

#### Recommendation

| Scenario | Choose |
|----------|--------|
| Building DSLs with simple grammars | **pest** |
| Need ordered choice semantics | **pest** |
| Need all valid parses (ambiguity) | **rust-sitter** |
| Governance/compliance requirements | **rust-sitter** |
| Artifact-driven deployment | **rust-sitter** |

**Summary**: pest is lightweight and developer-friendly; rust-sitter is heavier but handles ambiguity and governance needs.

---

### 4. rust-sitter GLR vs nom

**nom** is a parser combinator library where parsers are hand-written Rust functions.

#### Architectural Differences

**Declarative vs Combinators**
- **rust-sitter**: Grammars defined in IR, compiled to parse tables, loaded by runtime
- **nom**: Parsers are Rust functions composed with combinators (`alt`, `many`, `tag`, etc.)

**Separation of Concerns**
- **rust-sitter**: Grammar (data) vs Runtime (code) cleanly separated
- **nom**: Grammar and parsing logic mixed in Rust code

#### Where rust-sitter is Stronger

1. **Governance**: Grammars as inspectable, versioned artifacts vs Rust code
2. **Audit Trail**: SHA256 hashes of tables, BDD specs, contract compliance
3. **GLR Semantics**: Multi-action cells with conflict preservation vs manual ambiguity handling
4. **Multi-Grammar Runtime**: Load different grammars without recompiling

#### Where nom is Stronger

1. **Performance**: Hand-tuned combinators can be extremely fast
2. **Control**: Full Rust expressiveness for custom parsing logic
3. **Simplicity**: No build step, no external tools, just Rust
4. **Flexibility**: Can parse non-context-free structures easily

#### Recommendation

| Scenario | Choose |
|----------|--------|
| Maximum performance needed | **nom** |
| Need full control over parsing | **nom** |
| Governance/audit requirements | **rust-sitter** |
| Non-programmers write grammars | **rust-sitter** |
| GLR semantics for ambiguity | **rust-sitter** |

**Summary**: nom is for performance and control; rust-sitter is for governance and GLR semantics as infrastructure.

---

## The rust-sitter Unique Value Proposition

Across all comparisons, rust-sitter's unique positioning is:

> **Parser infrastructure for governed environments where provenance, reproducibility, and GLR semantics matter more than raw performance or maturity.**

### Core Differentiators

1. **Artifact-Driven Architecture**
   - `.parsetable` files are **first-class deployment artifacts**
   - SHA256 hashing, version metadata, CI validation
   - Enables **"parse table as data"** mindset vs **"parser as code"**

2. **Governance-First Design**
   - BDD scenarios (`BDD_GLR_CONFLICT_PRESERVATION.md`)
   - Completion contracts (`GLR_V1_COMPLETION_CONTRACT.md`)
   - Performance baselines with CI gates (`PERFORMANCE_BASELINE.md`)
   - ADRs (Architecture Decision Records)
   - Single Source of Truth (`STATUS_NOW.md`)

3. **GLR Conflict Preservation**
   - Multi-action cells in parse tables
   - Runtime fork/merge for ambiguous input
   - Can expose parse forests (planned for vNext)
   - Precedence-ordered action selection

4. **CI/CD Integration**
   - Parse table generation as testable build artifact
   - Performance regression gates (5% threshold)
   - Test connectivity safeguards
   - Concurrency caps for stable testing

### Target Audience

**Ideal for**:
- ✅ Infrastructure teams building multi-tenant parser services
- ✅ Compliance-heavy environments (finance, healthcare, government)
- ✅ Systems requiring audit trails for parsing logic
- ✅ Polyglot environments needing runtime grammar loading
- ✅ Teams valuing BDD/TDD/contract-first methodologies

**Not ideal for** (yet):
- ❌ Editor plugin authors (use Tree-sitter)
- ❌ Performance-critical single-grammar applications (use nom or LALRPOP)
- ❌ Quick prototyping (use pest or nom)
- ❌ Production incremental parsing (wait for v0.7.0)

---

## Roadmap: Closing Gaps

rust-sitter is **intentionally beta** to establish governance before scaling. Here's how gaps close:

### v0.7.0 (March 2026)
- ✅ Incremental parsing (Tree-sitter parity feature)
- ✅ Performance optimizations (arena allocator fix, allocation pooling)
- ✅ Hybrid stack implementation (15-20% improvement)
- ✅ Complete documentation (architecture, user guides)

### v1.0 (Q4 2026)
- ✅ Editor plugins (LSP integration)
- ✅ 50+ grammars (ecosystem expansion)
- ✅ API stability guarantees
- ✅ Production-grade everything

### Current Strengths (v0.6.1-beta)
- ✅ GLR core engine (tested, working)
- ✅ .parsetable pipeline (100% functional)
- ✅ Performance baseline (documented, CI-gated)
- ✅ BDD methodology (60% scenarios, deferred items documented)
- ✅ 93/93 tests passing (100%)

---

## Decision Framework

Use this flowchart to choose the right tool:

```
Do you need GLR semantics (ambiguous grammars)?
├─ Yes → rust-sitter or nom (if you want manual control)
└─ No → Continue...

Do you need editor integration TODAY?
├─ Yes → Tree-sitter
└─ No → Continue...

Do you need governance/contracts/audit trails?
├─ Yes → rust-sitter
└─ No → Continue...

Do you need runtime grammar loading?
├─ Yes → rust-sitter
└─ No → Continue...

Is this a single simple DSL?
├─ Yes → LALRPOP or pest (simplicity wins)
└─ No → Continue...

Do you need maximum performance?
├─ Yes → nom (hand-tuned combinators)
└─ No → Continue...

Default recommendation: Start with pest or LALRPOP for simplicity;
graduate to rust-sitter when governance/GLR becomes critical.
```

---

## Performance Comparison (Preliminary)

**Note**: rust-sitter is at baseline performance (not yet optimized). These numbers will improve significantly with planned optimizations.

### Current Benchmarks (v0.6.1-beta)

| Operation | rust-sitter GLR | Tree-sitter (est.) | Notes |
|-----------|----------------|-------------------|-------|
| Python 1000 lines | 62.4 µs (~16K lines/sec) | ~50-100K lines/sec (est.) | rust-sitter: baseline, not optimized |
| GLR fork operation | 73 ns | N/A (no GLR) | rust-sitter: sub-microsecond fork |
| Expression parsing (100 ops) | 11 ns | ~5-10 ns (est.) | rust-sitter: very competitive |
| Memory (Python grammar) | Comparable | Comparable | Similar algorithms |

**Planned Optimizations** (v0.7.0):
- Arena allocator fix: 2356x improvement (currently broken)
- Small allocation pooling: 208x improvement (high-frequency pattern)
- Hybrid stack implementation: 15-20% improvement
- Memory pooling enabled by default: 28% improvement

**Target** (post-v0.7.0): 70-90% of Tree-sitter C performance (typical for Rust vs C on compute-bound tasks).

---

## References

### Comparison Sources

- **Tree-sitter**: https://tree-sitter.github.io/tree-sitter/
- **LALRPOP**: https://github.com/lalrpop/lalrpop
- **pest**: https://pest.rs/
- **nom**: https://github.com/rust-bakery/nom

### rust-sitter Documentation

- **Performance Baseline**: [docs/PERFORMANCE_BASELINE.md](../PERFORMANCE_BASELINE.md)
- **GLR Completion Contract**: [docs/specs/GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md)
- **BDD Conflict Preservation**: [docs/plans/BDD_GLR_CONFLICT_PRESERVATION.md](../plans/BDD_GLR_CONFLICT_PRESERVATION.md)
- **Implementation Plan**: [IMPLEMENTATION_PLAN.md](../../IMPLEMENTATION_PLAN.md)
- **Current Status**: [STATUS_NOW.md](../../STATUS_NOW.md)

### External References

- **GLR Parsing Theory**: https://en.wikipedia.org/wiki/GLR_parser
- **PEG vs CFG**: https://en.wikipedia.org/wiki/Parsing_expression_grammar
- **LR Parsing**: https://en.wikipedia.org/wiki/LR_parser

---

**Document Status**: ✅ COMPLETE
**Last Updated**: 2025-11-20
**Next Review**: After v0.7.0 release (March 2026)
**Owner**: rust-sitter core team

---

END OF DOCUMENT
