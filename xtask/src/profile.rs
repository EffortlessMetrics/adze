/// CPU and memory profiling for adze
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use xshell::{Shell, cmd};

const PERF_BENCH_NAME: &str = "glr_performance_real";
const PERF_BENCH_FILTER_PREFIX: &str = "arithmetic_parsing/parse";

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
    Small,  // 100 LOC
    Medium, // 1-2k LOC
    Large,  // 5-10k LOC
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
    std::fs::create_dir_all(&target_dir).context("Failed to create target/profile directory")?;

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

    let bench_name = parser_benchmark_filter(size);
    let output_file = target_dir.join(format!("flamegraph_{}_{}.svg", grammar.name(), size.name()));

    println!("Generating flamegraph...");
    println!("Output: {}", output_file.display());

    let command = format!(
        "cargo flamegraph -p adze-benchmarks --bench {} --output {} -- --bench {}",
        PERF_BENCH_NAME,
        output_file.display(),
        bench_name
    );
    cmd!(
        sh,
        "cargo flamegraph -p adze-benchmarks --bench {PERF_BENCH_NAME} --output {output_file} -- --bench {bench_name}"
    )
    .run()
    .context("Failed to generate flamegraph")?;

    println!("✅ Flamegraph generated: {}", output_file.display());

    if json_output {
        write_profile_metadata(
            &target_dir,
            "cpu",
            grammar,
            size,
            &bench_name,
            &command,
            &output_file,
        )?;
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
    std::fs::create_dir_all(&target_dir).context("Failed to create target/profile directory")?;

    let bench_filter = parser_benchmark_filter(size);
    println!("Building benchmark binary...");
    cmd!(
        sh,
        "cargo build -p adze-benchmarks --release --bench {PERF_BENCH_NAME}"
    )
    .run()?;
    let bench_binary = resolve_bench_binary(PERF_BENCH_NAME)?;

    // Check if heaptrack is available
    let heaptrack_available = cmd!(sh, "heaptrack --version")
        .ignore_stdout()
        .ignore_stderr()
        .run()
        .is_ok();

    if !heaptrack_available {
        println!("⚠️  heaptrack not found. Falling back to valgrind massif.");
        return profile_memory_valgrind(
            sh,
            &bench_binary,
            grammar,
            size,
            bench_filter,
            &target_dir,
            json_output,
        );
    }

    println!("Using heaptrack for memory profiling...");

    let output_file = target_dir.join(format!("heaptrack_{}_{}.txt", grammar.name(), size.name()));
    let command = format!(
        "heaptrack --output {} {} -- --bench {}",
        output_file.display(),
        bench_binary.display(),
        bench_filter
    );
    println!("Running heaptrack...");
    cmd!(
        sh,
        "heaptrack --output {output_file} {bench_binary} -- --bench {bench_filter}"
    )
    .run()
    .context("Failed to run heaptrack profile")?;

    println!("⚠️  Heaptrack output written to {}", output_file.display());

    if json_output {
        write_profile_metadata(
            &target_dir,
            "memory-heaptrack",
            grammar,
            size,
            &bench_filter,
            &command,
            &output_file,
        )?;
    }

    Ok(())
}

/// Fallback memory profiling with valgrind massif
fn profile_memory_valgrind(
    sh: &Shell,
    bench_binary: &Path,
    grammar: ProfileGrammar,
    size: FixtureSize,
    bench_filter: String,
    target_dir: &Path,
    json_output: bool,
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
    let output_file = target_dir.join(format!("massif_{}_{}.out", grammar.name(), size.name()));
    let command = format!(
        "valgrind --tool=massif --massif-out-file {} {} -- --bench {}",
        output_file.display(),
        bench_binary.display(),
        bench_filter
    );
    cmd!(
        sh,
        "valgrind --tool=massif --massif-out-file={output_file} {bench_binary} -- --bench {bench_filter}"
    )
    .run()
    .context("Valgrind massif failed")?;

    if json_output {
        write_profile_metadata(
            target_dir,
            "memory-valgrind",
            grammar,
            size,
            &bench_filter,
            &command,
            &output_file,
        )?;
    }

    println!("💾 Massif output written to {}", output_file.display());
    Ok(())
}

fn parser_benchmark_filter(size: FixtureSize) -> String {
    format!("{}/{}", PERF_BENCH_FILTER_PREFIX, size.name())
}

fn resolve_bench_binary(bench_name: &str) -> Result<PathBuf> {
    let direct_binary = PathBuf::from("target/release/deps")
        .join(bench_name)
        .with_extension(std::env::consts::EXE_EXTENSION);

    if direct_binary.exists() {
        return Ok(direct_binary);
    }

    let pattern = format!(
        "target/release/deps/{}-*{}",
        bench_name,
        if cfg!(windows) { ".exe" } else { "" }
    );
    let mut iter = glob::glob(&pattern)?;
    iter.next()
        .and_then(|entry| entry.ok())
        .context("Benchmark binary not found. Run `cargo build -p adze-benchmarks --release --bench glr_performance_real` first.")
}

#[derive(serde::Serialize)]
struct ProfileMetadata {
    profile_type: String,
    grammar: String,
    fixture_size: String,
    benchmark: String,
    benchmark_filter: String,
    command: String,
    output: String,
}

fn write_profile_metadata(
    target_dir: &Path,
    profile_type: &str,
    grammar: ProfileGrammar,
    size: FixtureSize,
    filter: &str,
    command: &str,
    output_file: &Path,
) -> Result<()> {
    let metadata = ProfileMetadata {
        profile_type: profile_type.to_string(),
        grammar: grammar.name().to_string(),
        fixture_size: size.name().to_string(),
        benchmark: PERF_BENCH_NAME.to_string(),
        benchmark_filter: filter.to_string(),
        command: command.to_string(),
        output: output_file.display().to_string(),
    };

    let json_path = target_dir.join(format!(
        "{}_{}_{}.json",
        profile_type,
        grammar.name(),
        size.name()
    ));
    let json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(&json_path, json).context("Failed to write profile metadata")?;

    println!("📊 Profile metadata: {}", json_path.display());
    Ok(())
}
