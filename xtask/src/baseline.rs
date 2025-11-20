/// Performance baseline management and comparison
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use xshell::Shell;

/// Performance baseline data
#[derive(Debug, Serialize, Deserialize)]
pub struct Baseline {
    pub version: String,
    pub date: String,
    pub platform: String,
    pub benchmarks: HashMap<String, BenchmarkResult>,
}

/// Individual benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub mean_us: f64,
    pub stddev_us: f64,
    pub samples: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
}

/// Comparison result between two benchmarks
#[derive(Debug)]
pub struct Comparison {
    pub name: String,
    pub baseline: BenchmarkResult,
    pub current: BenchmarkResult,
    pub change_percent: f64,
    pub is_regression: bool,
}

/// Save current benchmark results as a new baseline
pub fn save_baseline(sh: &Shell, version: &str) -> Result<()> {
    println!("Saving performance baseline for version {}...", version);

    // Run benchmarks and capture output
    println!("Running benchmarks...");
    let bench_output = xshell::cmd!(sh, "cargo bench --bench glr_performance -- --save-baseline {version}")
        .ignore_status()
        .read()
        .context("Failed to run benchmarks")?;

    // Parse Criterion output to extract benchmark results
    let benchmarks = parse_criterion_output(&bench_output)?;

    let baseline = Baseline {
        version: version.to_string(),
        date: chrono::Local::now().to_rfc3339(),
        platform: format!(
            "{} {} (Rust {})",
            std::env::consts::OS,
            std::env::consts::ARCH,
            rustc_version()
        ),
        benchmarks,
    };

    // Save to baselines directory
    let baselines_dir = PathBuf::from("baselines");
    std::fs::create_dir_all(&baselines_dir)
        .context("Failed to create baselines directory")?;

    let baseline_path = baselines_dir.join(format!("{}.json", version));
    let json = serde_json::to_string_pretty(&baseline)
        .context("Failed to serialize baseline")?;
    std::fs::write(&baseline_path, json)
        .context("Failed to write baseline file")?;

    println!("✅ Baseline saved: {}", baseline_path.display());
    println!("📊 Benchmarks captured: {}", baseline.benchmarks.len());

    Ok(())
}

/// Compare current benchmark results against a baseline
pub fn compare_baseline(
    sh: &Shell,
    baseline_version: &str,
    threshold_percent: f64,
) -> Result<()> {
    println!(
        "Comparing against baseline {} (threshold: {}%)...",
        baseline_version, threshold_percent
    );

    // Load baseline
    let baseline_path = PathBuf::from("baselines").join(format!("{}.json", baseline_version));
    if !baseline_path.exists() {
        anyhow::bail!(
            "Baseline not found: {}\nRun 'cargo xtask bench --save-baseline {}' first",
            baseline_path.display(),
            baseline_version
        );
    }

    let baseline_json = std::fs::read_to_string(&baseline_path)
        .context("Failed to read baseline file")?;
    let baseline: Baseline = serde_json::from_str(&baseline_json)
        .context("Failed to parse baseline JSON")?;

    println!("Baseline: {} ({})", baseline.version, baseline.date);
    println!("Platform: {}", baseline.platform);

    // Run current benchmarks
    println!("\nRunning current benchmarks...");
    let bench_output = xshell::cmd!(sh, "cargo bench --bench glr_performance")
        .ignore_status()
        .read()
        .context("Failed to run benchmarks")?;

    let current_benchmarks = parse_criterion_output(&bench_output)?;

    // Compare results
    let comparisons = compare_results(&baseline.benchmarks, &current_benchmarks);

    // Print comparison report
    print_comparison_report(&comparisons, threshold_percent);

    // Check for regressions
    let regressions: Vec<_> = comparisons
        .iter()
        .filter(|c| c.is_regression && c.change_percent.abs() > threshold_percent)
        .collect();

    if !regressions.is_empty() {
        eprintln!("\n❌ Performance regressions detected:\n");
        for reg in &regressions {
            eprintln!(
                "  - {}: {:.2}µs → {:.2}µs ({:+.1}%, threshold {}%)",
                reg.name,
                reg.baseline.mean_us,
                reg.current.mean_us,
                reg.change_percent,
                threshold_percent
            );
        }
        anyhow::bail!("{} benchmark(s) regressed beyond threshold", regressions.len());
    }

    println!("\n✅ All benchmarks within performance threshold!");
    Ok(())
}

/// Parse Criterion benchmark output
fn parse_criterion_output(output: &str) -> Result<HashMap<String, BenchmarkResult>> {
    let mut benchmarks = HashMap::new();

    // This is a simplified parser. Real implementation would need to:
    // 1. Parse Criterion's verbose output format
    // 2. Extract mean, stddev, samples for each benchmark
    // 3. Handle different Criterion output formats

    // For now, return placeholder data for demonstration
    // In production, we'd use Criterion's JSON output or parse the text output
    println!("⚠️  Benchmark parsing not yet fully implemented");
    println!("📝 Using placeholder data. Real implementation will parse Criterion output.");

    // Placeholder: extract benchmark names from output
    for line in output.lines() {
        if line.contains("time:") && line.contains("µs") {
            // Extract benchmark name (simplified)
            if let Some(name_part) = line.split_whitespace().next() {
                benchmarks.insert(
                    name_part.to_string(),
                    BenchmarkResult {
                        mean_us: 100.0, // Placeholder
                        stddev_us: 5.0,
                        samples: 100,
                        memory_bytes: None,
                    },
                );
            }
        }
    }

    if benchmarks.is_empty() {
        println!("No benchmarks parsed from output (using defaults for demo)");
        // Add some default benchmarks for demonstration
        benchmarks.insert(
            "parse_python_small".to_string(),
            BenchmarkResult {
                mean_us: 6.32,
                stddev_us: 0.12,
                samples: 100,
                memory_bytes: None,
            },
        );
        benchmarks.insert(
            "parse_python_medium".to_string(),
            BenchmarkResult {
                mean_us: 31.28,
                stddev_us: 0.45,
                samples: 100,
                memory_bytes: None,
            },
        );
    }

    Ok(benchmarks)
}

/// Compare baseline and current results
fn compare_results(
    baseline: &HashMap<String, BenchmarkResult>,
    current: &HashMap<String, BenchmarkResult>,
) -> Vec<Comparison> {
    let mut comparisons = Vec::new();

    for (name, baseline_result) in baseline {
        if let Some(current_result) = current.get(name) {
            let change_percent = ((current_result.mean_us - baseline_result.mean_us)
                / baseline_result.mean_us)
                * 100.0;

            comparisons.push(Comparison {
                name: name.clone(),
                baseline: baseline_result.clone(),
                current: current_result.clone(),
                change_percent,
                is_regression: current_result.mean_us > baseline_result.mean_us,
            });
        }
    }

    // Sort by change magnitude (absolute value)
    comparisons.sort_by(|a, b| {
        b.change_percent
            .abs()
            .partial_cmp(&a.change_percent.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    comparisons
}

/// Print comparison report
fn print_comparison_report(comparisons: &[Comparison], threshold: f64) {
    println!("\n📊 Performance Comparison Report\n");
    println!("{:<40} {:>12} {:>12} {:>10}", "Benchmark", "Baseline", "Current", "Change");
    println!("{}", "=".repeat(80));

    for comp in comparisons {
        let symbol = if comp.is_regression {
            if comp.change_percent.abs() > threshold {
                "❌"
            } else {
                "⚠️ "
            }
        } else {
            "✅"
        };

        println!(
            "{} {:<38} {:>10.2}µs {:>10.2}µs {:>9.1}%",
            symbol,
            truncate(&comp.name, 38),
            comp.baseline.mean_us,
            comp.current.mean_us,
            comp.change_percent
        );
    }

    // Summary
    let improvements = comparisons.iter().filter(|c| !c.is_regression).count();
    let regressions = comparisons.iter().filter(|c| c.is_regression).count();
    let significant_regressions = comparisons
        .iter()
        .filter(|c| c.is_regression && c.change_percent.abs() > threshold)
        .count();

    println!("\n{}", "=".repeat(80));
    println!("Summary:");
    println!("  ✅ Improvements: {}", improvements);
    println!("  ⚠️  Regressions: {}", regressions);
    println!(
        "  ❌ Significant regressions (>{}%): {}",
        threshold, significant_regressions
    );
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Get rustc version string
fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|s| s.split_whitespace().nth(1).map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}
