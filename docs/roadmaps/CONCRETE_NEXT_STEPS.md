# Concrete Next Steps for Rust-Sitter Beta Launch

## Week 1: Beta Release Preparation

### Day 1-2: Release Engineering
- [ ] Tag v0.5.0-beta1 in git
- [ ] Update all Cargo.toml versions
- [ ] Run full test suite on all platforms
- [ ] Build and test WASM artifacts
- [ ] Publish to crates.io with beta tag

### Day 3-4: Dashboard Setup
```bash
# Create dashboard infrastructure
mkdir -p dashboard/{src,dist,data}
cargo xtask init-dashboard

# Set up GitHub Pages
git checkout -b gh-pages
# ... configure GitHub Pages in repo settings
```

### Day 5-7: Community Preparation
- [ ] Write beta announcement blog post
- [ ] Create GitHub issue templates for:
  - Grammar compatibility reports
  - Performance regressions
  - Feature requests
- [ ] Set up Discord/Zulip channel
- [ ] Prepare grammar migration guide with examples

## Week 2: Grammar.js Compatibility Spike

### Early Prototype Goals
Start with 20% coverage to identify edge cases:

```rust
// tool/src/grammar_js/parser.rs
pub struct GrammarJs {
    name: String,
    word: Option<String>,
    rules: HashMap<String, Rule>,
    extras: Vec<Rule>,
    conflicts: Vec<Vec<String>>,
    inline: Vec<String>,
    precedences: Vec<Precedence>,
}

// Parse simple grammar.js files
pub fn parse_grammar_js(content: &str) -> Result<GrammarJs> {
    // Start with regex-based parsing for MVP
    // Move to proper JS parser later
}
```

### Test Grammars (Start Simple)
1. **JSON** - No advanced features
2. **TOML** - Simple precedence
3. **Markdown** - External scanner
4. **JavaScript** - Complex precedence/conflicts

## Week 3: Automated Testing Infrastructure

### Corpus Test Runner
```yaml
# .github/workflows/corpus-test.yml
name: Grammar Corpus Tests

on:
  pull_request:
  schedule:
    - cron: '0 0 * * *' # Daily

jobs:
  test-corpus:
    strategy:
      matrix:
        grammar: [
          javascript, typescript, rust, python, go,
          ruby, java, c, cpp, csharp, swift, 
          haskell, elm, lua, php, bash
        ]
    steps:
      - name: Test ${{ matrix.grammar }}
        run: |
          cargo xtask test-grammar ${{ matrix.grammar }}
          
      - name: Update dashboard data
        if: github.ref == 'refs/heads/main'
        run: |
          cargo xtask update-dashboard-grammar \
            ${{ matrix.grammar }} \
            ${{ steps.test.outputs.result }}
```

### Performance Guard Rails
```rust
// benches/incremental_edits.rs
#[bench]
fn bench_typical_edit(b: &mut Bencher) {
    let mut parser = Parser::new();
    let tree = parser.parse(LARGE_FILE, None).unwrap();
    
    b.iter(|| {
        // Simulate typical edit: insert character
        let edit = Edit {
            start_byte: 1000,
            old_end_byte: 1000,
            new_end_byte: 1001,
            start_position: Point { row: 25, column: 10 },
            old_end_position: Point { row: 25, column: 10 },
            new_end_position: Point { row: 25, column: 11 },
        };
        
        tree.edit(&edit);
        parser.parse(EDITED_FILE, Some(&tree))
    });
}
```

## Week 4: Community Engagement

### Grammar Migration PRs
Template for automated PRs:

```markdown
# Add Rust-Sitter support to tree-sitter-{language}

This PR adds support for the pure-Rust Tree-sitter implementation alongside the existing C implementation.

## Changes
- Added `rust-sitter` feature flag to Cargo.toml
- Created `src/rust_grammar.rs` with grammar definition
- Updated CI to test both implementations
- Added migration guide in README

## Compatibility
- ✅ All tests pass with Rust implementation
- ✅ Performance within 5% of C version
- ✅ Identical parse trees for corpus

## How to test
```bash
cargo test --features rust-sitter
cargo bench --features bench-compare
```

Fixes #xxx (if applicable)
Part of rust-sitter/rust-sitter#1 (grammar compatibility tracking)
```

### Editor Plugin PoC (Neovim)
```lua
-- nvim-treesitter-rust/lua/treesitter-rust.lua
local M = {}

function M.setup()
  -- Check if rust-sitter binary exists
  local rust_sitter = vim.fn.exepath('rust-sitter-cli')
  if rust_sitter == '' then
    vim.notify('rust-sitter-cli not found', vim.log.levels.WARN)
    return
  end
  
  -- Override parser installation
  require'nvim-treesitter.install'.compilers = { rust_sitter }
end

return M
```

## Quick Wins Checklist

### This Week
1. **Fix known issues** from example grammars
2. **Set up badges** for README
3. **Create grammar template** repo
4. **Write "Getting Started in 5 minutes"** guide

### Next Week  
5. **Grammar.js parser spike** (even 20% helps)
6. **First compatibility bulletin** draft
7. **Recruit 2-3 contributors** for specific tasks
8. **Performance regression test** suite

### By End of Month
9. **5+ grammars** fully compatible
10. **Dashboard live** and auto-updating
11. **First external** grammar PR merged
12. **Query system design** doc ready

## Communication Plan

### Weekly Updates (Every Friday)
- GitHub Discussions post
- Discord announcement
- Twitter thread with metrics

### Monthly Bulletin (First Monday)
- Blog post with detailed progress
- Email to interested parties
- Reddit/HN post for major milestones

### Response SLA
- Critical bugs: < 24 hours
- Grammar issues: < 48 hours  
- Feature requests: < 1 week

## Success Indicators

### Week 1
- Beta published to crates.io
- Dashboard URL live
- 10+ GitHub issues filed

### Week 2
- First external grammar working
- 3+ community PRs submitted
- Performance benchmarks published

### Month 1
- 80% corpus pass rate achieved
- 5+ contributors active
- First editor plugin working

## Resources

- Grammar corpus: github.com/tree-sitter/tree-sitter/test/fixtures
- C implementation: github.com/tree-sitter/tree-sitter
- Discord: discord.gg/rust-lang #tree-sitter
- Zulip: rust-lang.zulipchat.com #tree-sitter