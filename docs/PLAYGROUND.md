# Adze Playground

Interactive web-based environment for grammar development and testing.

## Overview

The Adze Playground provides a browser-based IDE for developing, testing, and sharing grammars. Features include:

- **Live Parsing**: See parse trees update as you type
- **Grammar Editor**: Syntax highlighting for grammar definitions
- **Visual Debugging**: Step through parsing with visual feedback
- **Performance Metrics**: Real-time parsing statistics
- **Sharing**: Share grammars and examples via URLs
- **Export**: Generate parser code directly from the browser

## Quick Start

### Online Playground
Visit [play.adze.dev](https://play.adze.dev) to start immediately.

### Local Playground
```bash
# Install and run locally
cargo install adze-playground
adze-playground

# Open browser at http://localhost:8080
```

### Embedded Playground
```bash
# Add to your project
adze playground --grammar src/grammar.rs

# Custom port
adze playground --port 3000

# Watch mode (auto-reload)
adze playground --watch
```

## User Interface

### Layout
```
┌─────────────────────────────────────────────────────────┐
│  File  Edit  View  Tools  Help                          │
├─────────────────┬───────────────────┬───────────────────┤
│                 │                   │                   │
│  Grammar Editor │   Code Editor     │   Parse Tree      │
│                 │                   │                   │
│  #[grammar]     │  fn main() {      │  source_file      │
│  mod grammar {  │    let x = 42;    │    function_def   │
│    ...          │  }                │      name: main   │
│                 │                   │      body: ...    │
├─────────────────┴───────────────────┴───────────────────┤
│  Errors (0)  Warnings (0)  Performance: 1.2ms           │
└─────────────────────────────────────────────────────────┘
```

### Panels

#### 1. Grammar Editor
- Rust syntax highlighting
- Auto-completion for attributes
- Error squiggles with quick fixes
- Folding and formatting

#### 2. Code Editor
- Test your grammar on real code
- Multiple file support
- Syntax highlighting (generated)
- Error markers

#### 3. Parse Tree View
- Interactive tree exploration
- Node highlighting
- Field names and values
- Source position mapping

#### 4. Output Panel
- Error messages with links
- Performance metrics
- Debug output
- Export results

## Features

### 1. Live Development

#### Auto-Compilation
```rust
// Changes compile automatically
#[adze::grammar("my_lang")]
mod grammar {
    #[adze::language]
    pub struct Program {
        statements: Vec<Statement>,
    }
    // Parse tree updates on save
}
```

#### Hot Reload
- Grammar changes apply instantly
- Parse tree updates in real-time
- Error feedback immediate
- No manual compilation needed

### 2. Visual Debugging

#### Step-Through Parsing
```
Controls: [▶️ Play] [⏸️ Pause] [⏭️ Step] [⏮️ Back]

Current State: 15
Stack: [0, 5, 12, 15]
Lookahead: IDENTIFIER "main"
Action: Shift 23

[Visual state machine diagram]
```

#### Parse Tree Visualization
- Tree view with expand/collapse
- Graph view with connections
- Highlight corresponding source
- Show invisible nodes option

#### Error Visualization
```
Error at line 3, column 5:
Expected '}' but found 'fn'

fn main() {
    let x = 42
    fn nested() {}  ← Error here
}

Suggested fixes:
1. Add missing semicolon
2. Add closing brace
```

### 3. Performance Analysis

#### Real-Time Metrics
```
Parsing Statistics:
├─ Parse Time: 1.2ms
├─ Tokens: 156
├─ Nodes: 89
├─ Reused Nodes: 45 (50.6%)
├─ Memory: 12.5 KB
└─ Throughput: 130K tokens/sec
```

#### Flame Graph
Interactive flame graph showing time spent in each rule.

#### Hot Path Analysis
Identifies slow grammar rules and suggests optimizations.

### 4. Testing Tools

#### Test Corpus
```yaml
# Define test cases in YAML
tests:
  - name: "Basic function"
    input: |
      fn main() {
        println!("Hello");
      }
    expect: success
    
  - name: "Syntax error"
    input: "fn main("
    expect: error
    contains: "Expected ')'"
```

#### Fuzzing
```javascript
// In-browser fuzzing
playground.fuzz({
  iterations: 1000,
  maxDepth: 50,
  timeout: 5000,
  onError: (input, error) => {
    console.log(`Found crash: ${input}`);
  }
});
```

#### Coverage Report
Visual coverage overlay showing which grammar rules are tested.

### 5. Sharing & Export

#### Share via URL
```
https://play.adze.dev/#grammar=...&code=...
```

#### Export Options
- Download Rust parser code
- Export to GitHub Gist
- Generate npm package
- Create VS Code extension

#### Embedding
```html
<iframe 
  src="https://play.adze.dev/embed?grammar=..." 
  width="100%" 
  height="600">
</iframe>
```

## Advanced Features

### 1. Grammar Templates

#### Quick Start Templates
- Expression parser
- C-style language
- Lisp-style language
- Configuration language
- Markdown parser

#### Import from URL
```
https://play.adze.dev/?import=github:tree-sitter/tree-sitter-rust
```

### 2. Collaborative Editing

#### Real-Time Collaboration
- Share session link
- See other cursors
- Chat integration
- Voice/video calls

#### Code Review Mode
- Add comments to grammar
- Suggest changes
- Track versions
- Merge conflicts

### 3. AI Assistant

#### Grammar Generation
```
AI: "Generate a grammar for a simple calculator language"
→ Generates complete grammar with tests
```

#### Error Fixing
```
AI: "Why is this grammar ambiguous?"
→ Explains issue and suggests fixes
```

#### Optimization Suggestions
```
AI: "How can I make this grammar faster?"
→ Analyzes and suggests optimizations
```

### 4. Integration

#### GitHub Integration
- Load grammars from repos
- Create PRs from playground
- Run playground in GitHub Codespaces

#### CI/CD Integration
```yaml
# .github/workflows/playground.yml
- uses: adze/playground-action@v1
  with:
    grammar: src/grammar.rs
    tests: tests/corpus
```

## Configuration

### Playground Config
```toml
# playground.toml
[server]
port = 8080
host = "0.0.0.0"

[features]
ai_assistant = true
collaboration = true
export_code = true

[limits]
max_input_size = "1MB"
parse_timeout = "5s"
fuzz_iterations = 10000

[theme]
editor = "monokai"
ui = "dark"
```

### Customization

#### Custom Themes
```javascript
playground.addTheme({
  name: "my-theme",
  colors: {
    background: "#1e1e1e",
    foreground: "#cccccc",
    // ...
  }
});
```

#### Custom Tools
```javascript
playground.addTool({
  name: "AST Differ",
  icon: "🔍",
  panel: "bottom",
  component: AstDifferComponent
});
```

## API Reference

### JavaScript API
```javascript
// Initialize playground
const playground = new RustSitterPlayground({
  container: "#playground",
  grammar: myGrammarCode,
  code: "fn main() {}",
});

// Subscribe to events
playground.on("parse", (tree) => {
  console.log("Parsed:", tree);
});

playground.on("error", (errors) => {
  console.error("Parse errors:", errors);
});

// Control playground
playground.setGrammar(newGrammar);
playground.setCode(newCode);
playground.parse();

// Get results
const tree = playground.getParseTree();
const errors = playground.getErrors();
const metrics = playground.getMetrics();
```

### REST API
```http
# Parse code with grammar
POST /api/parse
Content-Type: application/json

{
  "grammar": "...",
  "code": "...",
  "options": {
    "timeout": 5000,
    "includeMetrics": true
  }
}

# Response
{
  "success": true,
  "tree": { ... },
  "metrics": {
    "parseTime": 1.2,
    "tokens": 156
  }
}
```

## Deployment

### Self-Hosted

#### Docker
```dockerfile
FROM rust:1.70
RUN cargo install adze-playground
EXPOSE 8080
CMD ["adze-playground", "--host", "0.0.0.0"]
```

#### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: adze-playground
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: playground
        image: rustsitter/playground:latest
        ports:
        - containerPort: 8080
```

### Cloud Providers

#### Vercel
```json
{
  "functions": {
    "api/parse.js": {
      "maxDuration": 10
    }
  }
}
```

#### Cloudflare Workers
```javascript
export default {
  async fetch(request, env) {
    return handlePlaygroundRequest(request);
  }
}
```

## Security

### Sandboxing
- Grammar compilation in WASM sandbox
- Resource limits enforced
- No filesystem access
- Network requests blocked

### Rate Limiting
```toml
[security]
rate_limit = "100/hour"
max_grammar_size = "100KB"
max_parse_time = "5s"
```

## Troubleshooting

### Common Issues

1. **Grammar won't compile**
   - Check error panel for details
   - Verify attribute syntax
   - Ensure all types are defined

2. **Parse tree not updating**
   - Check for compilation errors
   - Try manual refresh (Ctrl+R)
   - Clear browser cache

3. **Performance issues**
   - Reduce input size
   - Simplify grammar rules
   - Check for left recursion

### Debug Mode
```
?debug=true
```
Enables verbose logging and additional tools.

## Examples Gallery

Browse example grammars:
- [Simple Calculator](https://play.adze.dev/?example=calc)
- [JSON Parser](https://play.adze.dev/?example=json)
- [Mini Python](https://play.adze.dev/?example=python)
- [Config Language](https://play.adze.dev/?example=config)
- [Markdown](https://play.adze.dev/?example=markdown)

## Resources

- [Playground Tutorial](https://docs.adze.dev/playground/tutorial)
- [Video Walkthrough](https://youtube.com/@rustsitter)
- [Example Grammars](https://github.com/adze/playground-examples)
- [Report Issues](https://github.com/adze/playground/issues)