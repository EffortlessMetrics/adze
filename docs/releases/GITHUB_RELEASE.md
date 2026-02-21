# adze v0.6.1-beta

**Algorithmically correct GLR parser — six critical correctness fixes, 100% core GLR test pass.**

### ✅ Core fixes
- **Reduce → re-closure** (same lookahead) — cascaded reduces & accepts found.
- **Per-token accept aggregation** — no early short-circuit.
- **EOF recovery loop**: `close → check → (insert|pop)` — never delete at EOF.
- **ε loop guard** keyed on `(state, rule, end)`.
- **Nonterminal goto** semantics restored.
- **Query & forest:** wrapper-squash and capture dedup `(symbol,start,end)`.
- **Fork/merge stability:** pointer-equality safe dedup (optionally gated).

### 🧪 Test results (core)
- Fork/Merge: **30/30** ✅  
- Integration (queries): **5/5** ✅  
- Error Recovery: **5/5** ✅  
- GLR Parsing: **6/6** ✅  
- Regression Guards: **5/5** ✅

### ⚠️ Known limits
- Query predicates & advanced APIs WIP
- Incremental-GLR heuristics & equivalence suite WIP  
- CLI runtime loader and external scanner linking docs pending
- Safe-dedup heuristics pending perf tuning

### Upgrade
```toml
[dependencies]
adze = "0.6.1-beta"
adze-tool = "0.6.1-beta" # optional
```

---

**Full Changelog**: https://github.com/EffortlessMetrics/adze/compare/v0.6.0...v0.6.1-beta