/// Benchmark runner with enhanced features
use anyhow::{Context, Result};
use xshell::{Shell, cmd};

const RELEASE_PARSER_BENCHMARKS: &[&str] = &["glr_performance_real"];

/// Run benchmarks with optional baseline saving
pub fn run_benchmarks(
    sh: &Shell,
    save_baseline: bool,
    baseline_name: Option<String>,
) -> Result<()> {
    println!(
        "Running adze performance benchmarks: {}",
        RELEASE_PARSER_BENCHMARKS.join(", ")
    );

    run_released_parser_benchmarks(sh)?;

    if save_baseline {
        let version = baseline_name.unwrap_or_else(|| {
            // Auto-detect version from Cargo.toml
            detect_version().unwrap_or_else(|_| "latest".to_string())
        });

        println!("Will save baseline as: {}", version);

        // Use the baseline module to save results
        // After benchmarks complete, save baseline
        crate::baseline::save_baseline(sh, &version)?;
    } else {
        println!("Benchmarks completed. Baseline not saved.");
    }

    println!("✅ Benchmarks complete!");
    Ok(())
}

fn run_released_parser_benchmarks(sh: &Shell) -> Result<()> {
    for bench in RELEASE_PARSER_BENCHMARKS {
        cmd!(sh, "cargo bench -p adze-benchmarks --bench {bench}")
            .run()
            .context(format!("Failed to run benchmark {bench}"))?;
    }
    Ok(())
}

/// Detect version from Cargo.toml
fn detect_version() -> Result<String> {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;

    for line in cargo_toml.lines() {
        if line.trim().starts_with("version") {
            // Extract version string: version = "0.8.0-dev"
            if let Some(version_str) = line.split('"').nth(1) {
                return Ok(version_str.to_string());
            }
        }
    }

    anyhow::bail!("Could not detect version from Cargo.toml")
}
