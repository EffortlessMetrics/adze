# PR #11 Fixes: CI Stabilization and Process Management

## Problem Analysis

PR #11 experienced two critical failures:
1. **Rust job timeout** (30 minutes) during `cargo build && cargo test` for `adze-python`
2. **Node runner crash** with `Error: spawn ps EAGAIN` on Node v24.1.0

### Root Causes
1. **Cold, parallel Rust build**: High parallelism → many `rustc` processes → memory/process pressure → timeout
2. **Process management fragility**: Node orchestration using `ps` commands that failed under fork pressure 
3. **Agent duplication**: Multiple `pr-cleanup-reviewer` invocations causing process table exhaustion

## Implemented Fixes

### 1. CI Rust Build Optimization ✅

**File**: `.github/workflows/ci.yml`

Added dedicated `adze-python` job with:
```yaml
adze-python:
  timeout-minutes: 45
  env:
    CARGO_BUILD_JOBS: 2              # cap parallelism
    CARGO_INCREMENTAL: "1"           # incremental builds
    RUSTFLAGS: "-C debuginfo=0 -C codegen-units=8"
  steps:
    - uses: Swatinem/rust-cache@v2   # aggressive caching
    - name: Build adze-python
      run: cargo build -p adze-python --locked --jobs 2
    - name: Compile tests (no run)   # separate compile/run phases
      run: cargo test -p adze-python --locked --no-run --jobs 2
    - name: Run tests
      run: cargo test -p adze-python -- --test-threads 1 --nocapture
```

**Benefits**:
- Caps `rustc` fan-out (2 jobs max)
- Caches build artifacts across runs
- Separates compilation from execution for better resource management
- Captures timing data for analysis

### 2. Process Management Hardening ✅

**Files**: 
- `scripts/process-utils.js` - Node.js process utilities
- `scripts/safe-run.sh` - Shell script wrapper

**Key Features**:

#### Eliminates `ps` Dependency
- Uses **process groups** (`setsid`) instead of `ps` for process management
- Kills entire process tree with `kill -TERM $PGID` (negative PID)
- No external process enumeration required

#### EAGAIN Handling
```bash
run_with_retry() {
    for attempt in $(seq 1 $max_retries); do
        if run_with_pgid "$@"; then return 0; fi
        
        local exit_code=$?
        if [[ $exit_code -eq 1 && $attempt -lt $max_retries ]]; then
            echo "Command failed (possibly EAGAIN), retrying in ${retry_delay}s..."
            sleep $retry_delay
            retry_delay=$((retry_delay * 2))  # exponential backoff
            continue
        fi
        return $exit_code
    done
}
```

#### Process Group Management
```bash
run_with_pgid() {
    setsid "$cmd" "${args[@]}" &
    local child_pid=$!
    CHILD_PGID=-$child_pid  # Process group ID (negative)
    
    # Reliable cleanup on timeout/signal
    trap "kill -TERM ${CHILD_PGID} 2>/dev/null || true" EXIT INT TERM
}
```

### 3. Agent Orchestration Debouncing ✅

**Global Locking System**:
```bash
with_lock() {
    local lock_name="$1"
    local lock_file="${LOCK_DIR}/${lock_name}.lock"
    
    # Acquire exclusive lock with stale detection
    if (set -C; echo "$$:$(date):$*" > "$lock_file") 2>/dev/null; then
        run_with_retry "$@"
        rm -f "$lock_file"
    else
        # Wait or detect stale locks (>5min old)
        wait_for_lock_or_cleanup_stale
    fi
}
```

**Usage**:
```bash
# Prevent duplicate agent runs
./scripts/safe-run.sh agent pr-cleanup-reviewer
./scripts/safe-run.sh run-with-lock rust-build cargo build --workspace
```

### 4. Enhanced S-expression Testing ✅

**File**: `runtime/tests/test_serialization_roundtrip.rs`

Added comprehensive testing coverage:

#### Roundtrip Identity Tests
- **Basic structures**: atoms, lists, nested structures
- **Property-based**: 100+ random structures tested for serialize/deserialize identity
- **Edge cases**: empty structures, error nodes, missing nodes

#### Unicode & Canonicalization
- **Non-BMP characters**: Emoji, mathematical symbols
- **Combining marks**: Proper normalization handling
- **RTL text**: Arabic, Hebrew script support
- **Escape sequences**: `\"`, `\\`, `\n`, `\t`, `\r`

#### Performance & Stability
- **Deep structures**: 1000-level nesting without stack overflow
- **Wide structures**: 10,000 children without quadratic performance
- **Large trees**: 1000-node serialization within time bounds

#### Sample Test:
```rust
#[test]
fn test_unicode_edge_cases() {
    let unicode_cases = vec![
        ("🚀", "🚀"),                    // Emoji
        ("𝔘𝔫𝔦𝔠𝔬𝔡𝔢", "𝔘𝔫𝔦𝔠𝔬𝔡𝔢"), // Mathematical
        ("e\u{0301}", "é"),             // Combining marks
        ("שלום", "שלום"),                // Hebrew RTL
        ("مرحبا", "مرحبا"),              // Arabic RTL
    ];

    for (input, expected) in unicode_cases {
        let quoted_input = format!("\"{}\"", input);
        let parsed = parse_sexpr(&quoted_input).unwrap();
        assert_roundtrip_fidelity(&parsed, expected);
    }
}
```

### 5. Enhanced Serialization API ✅

**File**: `runtime/src/serialization.rs`

Added missing functionality:

#### S-expression Parser
```rust
pub fn parse_sexpr(input: &str) -> Result<SExpr, String>
pub enum SExpr { Atom(String), List(Vec<SExpr>) }
```

#### Tree Statistics
```rust
pub struct TreeStatistics {
    pub total_nodes: usize,
    pub named_nodes: usize,
    pub error_nodes: usize,
    pub max_depth: usize,
    pub node_types: HashMap<String, usize>,
}
```

## Usage & Integration

### For CI/Automation
```bash
# Use safe process management
./scripts/safe-run.sh run cargo test -p adze-python
./scripts/safe-run.sh run-with-lock build-lock cargo build --workspace

# Cleanup stale locks periodically  
./scripts/safe-run.sh cleanup-locks
```

### For Claude Agent Orchestration
```bash
# Debounced agent execution
./scripts/safe-run.sh agent pr-cleanup-reviewer
./scripts/process-utils.js agent pr-initial-reviewer
```

### Environment Variables
```bash
TIMEOUT_SEC=1800        # Command timeout (default: 30min)
RUST_TEST_THREADS=1     # Test thread limit
CARGO_BUILD_JOBS=2      # Build parallelism cap
```

## Verification Checklist

- ✅ CI uses `rust-cache`, `--jobs 2`, and staged build process
- ✅ Process management no longer shells out to `ps`
- ✅ EAGAIN handling with exponential backoff retry
- ✅ Agent triggers debounced with global locking
- ✅ S-expression roundtrip tests cover Unicode, performance, edge cases
- ✅ All scripts are executable and have proper error handling

## Expected Outcomes

1. **CI Stability**: Rust build timeout eliminated via caching + parallelism limits
2. **Process Reliability**: EAGAIN errors eliminated via process group management  
3. **Agent Reliability**: Duplicate invocations prevented via global locking
4. **Test Coverage**: S-expression serialization comprehensively validated

These fixes address the core infrastructure issues preventing PR #11 from landing successfully while maintaining all existing functionality.