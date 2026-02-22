# Now / Next / Later

**Last updated:** 2026-02-21

This is the rolling plan. Keep it short. If something is real, it should have:
- a finish line
- a link (issue/PR)
- a reason it matters

For paper cuts and recurring pain: `docs/status/FRICTION_LOG.md`.

---

## Now

### Docs that match reality
- [ ] README is the canonical entry point (short, correct, linked)
- [ ] Roadmap is outcomes (durable)
- [ ] Status docs live in `docs/status/` and are referenced everywhere

### Default dev loop stays cheap
- [ ] `just ci-supported` remains the supported gate and stays bounded
- [ ] Non-required CI workflows are clearly optional (nightly/manual/canary), not PR-blocking by accident

### Friction loop is real
- [ ] Every recurring "how do I..." becomes a Friction Log entry + issue link
- [ ] When fixed: mark resolved + link PR + add guardrail (docs/script/error message)

---

## Next

### Publishable baseline
- [ ] Decide publish set (what is publishable vs internal)
- [ ] Clean `cargo package` for publishable crates
- [ ] Tighten feature-flag story: stable vs experimental

### Tooling that reduces ambiguity
- [ ] CLI: validation + inspection commands (high-signal subset)
- [ ] Golden tests: expand coverage with explicit maintenance rules

---

## Later

- Incremental parsing maturity (beyond conservative fallback)
- Query predicate completion + cookbook
- Playground and LSP generator become genuinely useful for daily work
