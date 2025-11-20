/// CPU and memory profiling for rust-sitter
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

/// Profile type selection
#[derive(Clone, Copy, Debug)]
pub enum ProfileType {
    Cpu,
    Memory,
}

/// Grammar to profile
#[derive(Clone, Copy, Debug)]
pub enum ProfileGrammar {
    Python,
    Javascript,
    Arithmetic,
}

impl ProfileGrammar {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Javascript => "javascript",
            Self::Arithmetic => "arithmetic",
        }
    }
}

/// Fixture size
#[derive(Clone, Copy, Debug)]
pub enum FixtureSize {
    Small,   // 100 LOC
    Medium,  // 1-2k LOC
    Large,   // 5-10k LOC
}

impl FixtureSize {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

/// Run profiling for a specific grammar and fixture
pub fn profile(
    sh: &Shell,
    profile_type: ProfileType,
    grammar: ProfileGrammar,
    size: FixtureSize,
    json_output: bool,
) -> Result<()> {
    println!(
        "Profiling {:?} for {} grammar with {} fixture...",
        profile_type,
        grammar.name(),
        size.name()
    );

    match profile_type {
        ProfileType::Cpu => profile_cpu(sh, grammar, size, json_output),
        ProfileType::Memory => profile_memory(sh, grammar, size, json_output),
    }
}

/// CPU profiling with flamegraph
fn profile_cpu(
    sh: &Shell,
    grammar: ProfileGrammar,
    size: FixtureSize,
    json_output: bool,
) -> Result<()> {
    // Ensure target directory exists
    let target_dir = PathBuf::from("target/profile");
    std::fs::create_dir_all(&target_dir)
        .context("Failed to create target/profile directory")?;

    // Check if cargo-flamegraph is installed
    let flamegraph_installed = cmd!(sh, "cargo flamegraph --version")
        .ignore_stdout()
        .ignore_stderr()
        .run()
        .is_ok();

    if !flamegraph_installed {
        println!("Installing cargo-flamegraph...");
        cmd!(sh, "cargo install flamegraph")
            .run()
            .context("Failed to install flamegraph")?;
    }

    // Construct benchmark name
    let bench_name = format!("parse_{}_{}_{}", grammar.name(), size.name(), "bench");

    // Output file path
    let output_file = target_dir.join(format!(
        "flamegraph_{}_{}.svg",
        grammar.name(),
        size.name()
    ));

    println!("Generating flamegraph...");
    println!("Output: {}", output_file.display());

    // Run flamegraph on the benchmark
    // Note: This is a simplified version. In practice, you'd want to:
    // 1. Build a custom profiling binary that runs just the parse operation
    // 2. Use `flamegraph` or `perf` directly on that binary
    cmd!(
        sh,
        "cargo flamegraph --bench glr_performance --output {output_file} -- --bench {bench_name}"
    )
    .run()
    .context("Failed to generate flamegraph")?;

    println!("✅ Flamegraph generated: {}", output_file.display());

    if json_output {
        // Extract metrics from profiling run
        // This is a placeholder - real implementation would parse perf data
        let metrics = ProfilingMetrics {
            grammar: grammar.name().to_string(),
            fixture_size: size.name().to_string(),
            total_time_us: 0.0, // Would be extracted from perf data
            samples_count: 0,
            top_functions: vec![],
        };

        let json_path = target_dir.join(format!(
            "profile_metrics_{}_{}.json",
            grammar.name(),
            size.name()
        ));
        let json = serde_json::to_string_pretty(&metrics)?;
        std::fs::write(&json_path, json).context("Failed to write JSON metrics")?;

        println!("📊 JSON metrics: {}", json_path.display());
    }

    Ok(())
}

/// Memory profiling with heaptrack or valgrind
fn profile_memory(
    sh: &Shell,
    grammar: ProfileGrammar,
    size: FixtureSize,
    json_output: bool,
) -> Result<()> {
    // Ensure target directory exists
    let target_dir = PathBuf::from("target/profile");
    std::fs::create_dir_all(&target_dir)
        .context("Failed to create target/profile directory")?;

    // Check if heaptrack is available
    let heaptrack_available = cmd!(sh, "heaptrack --version")
        .ignore_stdout()
        .ignore_stderr()
        .run()
        .is_ok();

    if !heaptrack_available {
        println!("⚠️  heaptrack not found. Falling back to valgrind massif.");
        return profile_memory_valgrind(sh, grammar, size, json_output, &target_dir);
    }

    println!("Using heaptrack for memory profiling...");

    // Build the benchmark in release mode first
    println!("Building benchmarks...");
    cmd!(sh, "cargo build --release --benches").run()?;

    // Construct benchmark binary path
    let bench_binary = PathBuf::from("target/release/deps")
        .join("glr_performance")
        .with_extension(std::env::consts::EXE_EXTENSION);

    if !bench_binary.exists() {
        // Try to find it with glob
        let pattern = format!("target/release/deps/glr_performance-*{}", if cfg!(windows) { ".exe" } else { "" });
        let entries: Vec<_> = glob::glob(&pattern)?.collect();
        if entries.is_empty() {
            anyhow::bail!("Benchmark binary not found at {}", bench_binary.display());
        }
    }

    // Output file path
    let output_file = target_dir.join(format!(
        "heaptrack_{}_{}.txt",
        grammar.name(),
        size.name()
    ));

    println!("Running heaptrack...");

    // Note: This is simplified. Real implementation would:
    // 1. Run heaptrack on the benchmark binary
    // 2. Parse heaptrack output
    // 3. Generate report
    cmd!(sh, "echo 'Heaptrack profiling placeholder'")
        .run()
        .context("Heaptrack profiling not yet fully implemented")?;

    println!("⚠️  Full heaptrack integration coming soon!");
    println!("📝 For now, run manually:");
    println!("   heaptrack target/release/deps/glr_performance-*");

    if json_output {
        // Placeholder for JSON metrics
        let metrics = MemoryMetrics {
            grammar: grammar.name().to_string(),
            fixture_size: size.name().to_string(),
            peak_memory_bytes: 0,
            total_allocations: 0,
            top_allocation_sites: vec![],
        };

        let json_path = target_dir.join(format!(
            "memory_metrics_{}_{}.json",
            grammar.name(),
            size.name()
        ));
        let json = serde_json::to_string_pretty(&metrics)?;
        std::fs::write(&json_path, json).context("Failed to write JSON metrics")?;

        println!("📊 JSON metrics (placeholder): {}", json_path.display());
    }

    Ok(())
}

/// Fallback memory profiling with valgrind massif
fn profile_memory_valgrind(
    sh: &Shell,
    grammar: ProfileGrammar,
    size: FixtureSize,
    json_output: bool,
    target_dir: &Path,
) -> Result<()> {
    // Check if valgrind is available
    let valgrind_available = cmd!(sh, "valgrind --version")
        .ignore_stdout()
        .ignore_stderr()
        .run()
        .is_ok();

    if !valgrind_available {
        anyhow::bail!("Neither heaptrack nor valgrind available. Please install one of them.");
    }

    println!("Using valgrind massif for memory profiling...");
    println!("⚠️  Valgrind integration coming soon!");
    println!("📝 For now, run manually:");
    println!("   valgrind --tool=massif --massif-out-file=massif.out \\");
    println!("     target/release/deps/glr_performance-*");

    Ok(())
}

/// Profiling metrics for JSON export
#[derive(serde::Serialize)]
struct ProfilingMetrics {
    grammar: String,
    fixture_size: String,
    total_time_us: f64,
    samples_count: usize,
    top_functions: Vec<FunctionMetrics>,
}

#[derive(serde::Serialize)]
struct FunctionMetrics {
    name: String,
    time_percent: f64,
    time_microseconds: f64,
    call_count: u64,
}

/// Memory profiling metrics
#[derive(serde::Serialize)]
struct MemoryMetrics {
    grammar: String,
    fixture_size: String,
    peak_memory_bytes: u64,
    total_allocations: u64,
    top_allocation_sites: Vec<AllocationSite>,
}

#[derive(serde::Serialize)]
struct AllocationSite {
    function: String,
    bytes: u64,
    count: u64,
}
