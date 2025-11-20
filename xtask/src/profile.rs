//! Profiling commands for rust-sitter performance analysis
//!
//! This module provides CPU and memory profiling capabilities for benchmarking
//! rust-sitter's GLR parser performance.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use xshell::{cmd, Shell};

/// Profile type: CPU or Memory
pub enum ProfileType {
    Cpu,
    Memory,
}

/// Run CPU or memory profiling on a fixture
pub fn profile(
    sh: &Shell,
    profile_type: ProfileType,
    language: &str,
    fixture: &str,
    output_dir: Option<&str>,
) -> Result<()> {
    match profile_type {
        ProfileType::Cpu => profile_cpu(sh, language, fixture, output_dir),
        ProfileType::Memory => profile_memory(sh, language, fixture, output_dir),
    }
}

fn profile_cpu(
    _sh: &Shell,
    language: &str,
    fixture: &str,
    output_dir: Option<&str>,
) -> Result<()> {
    eprintln!("🔥 CPU Profiling with Flamegraph");
    eprintln!();

    // Check for cargo-flamegraph
    if !command_exists("cargo-flamegraph") {
        anyhow::bail!(
            "cargo-flamegraph not found. Install with: cargo install flamegraph"
        );
    }

    // Validate fixture exists
    let fixture_path = PathBuf::from("benches/fixtures").join(fixture);
    if !fixture_path.exists() {
        anyhow::bail!("Fixture not found: {}", fixture_path.display());
    }

    // Determine size from path
    let size = determine_size(fixture);

    // Set output directory
    let output_dir = output_dir.unwrap_or("docs/analysis");
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    let output_file = format!("{}/flamegraph-{}-{}.svg", output_dir, language, size);

    eprintln!("Language: {}", language);
    eprintln!("Fixture: {}", fixture_path.display());
    eprintln!("Size: {}", size);
    eprintln!("Output: {}", output_file);
    eprintln!();

    // TODO: Replace with actual parsing benchmark once GLR integration is complete
    eprintln!("⏳ Placeholder: GLR parsing integration pending");
    eprintln!();
    eprintln!("To integrate:");
    eprintln!("1. Import rust-sitter parser in benchmark");
    eprintln!("2. Load grammar for the specified language");
    eprintln!("3. Run flamegraph on actual parsing");
    eprintln!();
    eprintln!("Command that will be used:");
    eprintln!("  cargo flamegraph --bench glr-performance --output {} -- --bench", output_file);
    eprintln!();

    // Create placeholder flamegraph metadata file
    let metadata_file = format!("{}/flamegraph-{}-{}.json", output_dir, language, size);
    let metadata = serde_json::json!({
        "language": language,
        "fixture": fixture,
        "size": size,
        "status": "pending_integration",
        "command": format!("cargo flamegraph --bench glr-performance --output {} -- --bench", output_file),
        "note": "Actual profiling will be available after GLR parsing integration in Week 3 Day 2"
    });

    fs::write(&metadata_file, serde_json::to_string_pretty(&metadata)?)
        .with_context(|| format!("Failed to write metadata: {}", metadata_file))?;

    eprintln!("✅ Metadata saved: {}", metadata_file);
    eprintln!();
    eprintln!("Next: Integrate actual GLR parsing for real flamegraphs");

    Ok(())
}

fn profile_memory(
    _sh: &Shell,
    language: &str,
    fixture: &str,
    output_dir: Option<&str>,
) -> Result<()> {
    eprintln!("💾 Memory Profiling");
    eprintln!();

    // Check for profiling tools
    let has_heaptrack = command_exists("heaptrack");
    let has_valgrind = command_exists("valgrind");

    if !has_heaptrack && !has_valgrind {
        anyhow::bail!(
            "Neither heaptrack nor valgrind found. Install one of:\n  \
            - heaptrack (recommended): sudo apt-get install heaptrack\n  \
            - valgrind: sudo apt-get install valgrind"
        );
    }

    let tool = if has_heaptrack { "heaptrack" } else { "valgrind" };

    // Validate fixture exists
    let fixture_path = PathBuf::from("benches/fixtures").join(fixture);
    if !fixture_path.exists() {
        anyhow::bail!("Fixture not found: {}", fixture_path.display());
    }

    // Determine size from path
    let size = determine_size(fixture);

    // Set output directory
    let output_dir = output_dir.unwrap_or("docs/analysis");
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    let output_file = format!("{}/memory-{}-{}.txt", output_dir, language, size);

    eprintln!("Language: {}", language);
    eprintln!("Fixture: {}", fixture_path.display());
    eprintln!("Size: {}", size);
    eprintln!("Tool: {}", tool);
    eprintln!("Output: {}", output_file);
    eprintln!();

    // TODO: Replace with actual memory profiling once GLR integration is complete
    eprintln!("⏳ Placeholder: GLR parsing integration pending");
    eprintln!();
    eprintln!("To integrate:");
    eprintln!("1. Build release binary with debug symbols");
    eprintln!("2. Run {} on parsing benchmark", tool);
    eprintln!("3. Analyze memory allocation patterns");
    eprintln!();

    let command = if tool == "heaptrack" {
        format!("heaptrack --output {} cargo bench --bench glr-performance", output_file)
    } else {
        format!("valgrind --tool=massif --massif-out-file={} cargo bench --bench glr-performance", output_file)
    };

    eprintln!("Command that will be used:");
    eprintln!("  {}", command);
    eprintln!();

    // Create placeholder memory report
    let report = format!(
        r#"Memory Profiling Report
Language: {}
Fixture: {}
Size: {}
Tool: {}

Status: Pending GLR parsing integration (Week 3 Day 2)

This report will include:
- Peak memory usage
- Allocation hotspots (top 5 functions by allocation count)
- Object lifetimes (short-lived vs long-lived allocations)
- Memory usage ratio vs input size

Methodology:
{}

Generated: {}
"#,
        language,
        fixture,
        size,
        tool,
        command,
        chrono::Utc::now().to_rfc3339()
    );

    fs::write(&output_file, report)
        .with_context(|| format!("Failed to write report: {}", output_file))?;

    eprintln!("✅ Placeholder report created: {}", output_file);
    eprintln!();
    eprintln!("Next: Integrate actual GLR parsing for real memory profiling");

    Ok(())
}

fn determine_size(fixture: &str) -> &str {
    if fixture.contains("/small/") {
        "small"
    } else if fixture.contains("/medium/") {
        "medium"
    } else if fixture.contains("/large/") {
        "large"
    } else {
        "unknown"
    }
}

fn command_exists(program: &str) -> bool {
    which::which(program).is_ok()
}
