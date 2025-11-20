/// Performance baseline management and comparison
///
/// This module implements baseline capture and comparison for Criterion benchmarks,
/// replacing placeholder logic with real JSON parsing.
///
/// ## Usage
///
/// Save baseline:
/// ```bash
/// cargo bench --bench glr_performance_real
/// cargo xtask bench --save-baseline v0.8.0
/// ```
///
/// Compare against baseline:
/// ```bash
/// cargo bench --bench glr_performance_real
/// cargo xtask compare-baseline v0.8.0 --threshold 5
/// ```
///
/// ## Implementation
///
/// - Parses Criterion's `target/criterion/**/base/estimates.json` files
/// - Extracts mean and std dev in nanoseconds
/// - Saves to `baselines/<version>.json`
/// - Compares current results against saved baseline
/// - Detects regressions above configurable threshold
///
/// Related: docs/specs/BASELINE_MANAGEMENT_SPEC.md

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use xshell::Shell;

/// Performance baseline data
#[derive(Debug, Serialize, Deserialize)]
pub struct Baseline {
    /// Version identifier (e.g., "v0.8.0")
    pub version: String,
    /// Timestamp when baseline was captured
    pub date: String,
    /// Platform information
    pub platform: String,
    /// Benchmark results (name → result)
    pub benchmarks: HashMap<String, BenchmarkResult>,
}

/// Individual benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Mean time in nanoseconds
    pub mean_ns: f64,
    /// Standard deviation in nanoseconds
    pub stddev_ns: f64,
    /// Number of samples
    pub samples: usize,
}

/// Criterion estimates.json structure (subset we care about)
#[derive(Debug, Deserialize)]
struct CriterionEstimates {
    mean: Estimate,
    std_dev: Estimate,
}

#[derive(Debug, Deserialize)]
struct Estimate {
    point_estimate: f64,
    #[allow(dead_code)]
    standard_error: f64,
    #[allow(dead_code)]
    confidence_interval: ConfidenceInterval,
}

#[derive(Debug, Deserialize)]
struct ConfidenceInterval {
    #[allow(dead_code)]
    confidence_level: f64,
    #[allow(dead_code)]
    lower_bound: f64,
    #[allow(dead_code)]
    upper_bound: f64,
}

/// Criterion benchmark.json structure (for sample count)
#[derive(Debug, Deserialize)]
struct CriterionBenchmark {
    #[allow(dead_code)]
    group_id: String,
    #[allow(dead_code)]
    function_id: Option<String>,
    #[allow(dead_code)]
    value_str: Option<String>,
    #[allow(dead_code)]
    throughput: Option<Vec<serde_json::Value>>,
    #[allow(dead_code)]
    full_id: String,
    #[allow(dead_code)]
    directory_name: Option<String>,
    #[allow(dead_code)]
    title: String,
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

    // Discover all benchmarks in target/criterion/
    let criterion_dir = PathBuf::from("target/criterion");
    if !criterion_dir.exists() {
        bail!("Criterion output directory not found: {}\nRun benchmarks first: cargo bench", criterion_dir.display());
    }

    let benchmarks = discover_benchmarks(&criterion_dir)
        .context("Failed to discover benchmarks")?;

    if benchmarks.is_empty() {
        bail!("No benchmarks found in {}\nRun benchmarks first: cargo bench", criterion_dir.display());
    }

    println!("Discovered {} benchmarks", benchmarks.len());

    // Create baseline
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
    println!("\nSample results:");
    for (name, result) in baseline.benchmarks.iter().take(3) {
        println!("  {}: {:.2} µs ± {:.2} µs",
            name,
            result.mean_ns / 1000.0,
            result.stddev_ns / 1000.0
        );
    }

    Ok(())
}

/// Discover all benchmarks in Criterion output directory
///
/// Walks target/criterion/ recursively looking for base/estimates.json files.
/// Extracts benchmark names from directory structure.
fn discover_benchmarks(criterion_dir: &Path) -> Result<HashMap<String, BenchmarkResult>> {
    let mut benchmarks = HashMap::new();

    for entry in WalkDir::new(criterion_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Look for base/estimates.json files
        if path.ends_with("base/estimates.json") {
            match parse_benchmark_from_path(path, criterion_dir) {
                Ok((name, result)) => {
                    benchmarks.insert(name, result);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(benchmarks)
}

/// Parse benchmark name and result from estimates.json path
fn parse_benchmark_from_path(
    estimates_path: &Path,
    criterion_dir: &Path,
) -> Result<(String, BenchmarkResult)> {
    // Extract benchmark name from path
    let name = extract_benchmark_name(estimates_path, criterion_dir)?;

    // Parse estimates.json
    let result = parse_criterion_estimates(estimates_path)?;

    Ok((name, result))
}

/// Extract benchmark name from estimates.json file path
///
/// Example transformations:
/// - target/criterion/fixture_loading_python_small/base/estimates.json
///   → fixture_loading_python_small
/// - target/criterion/real_parsing/parse_arithmetic/python_small/base/estimates.json
///   → real_parsing/parse_arithmetic/python_small
fn extract_benchmark_name(estimates_path: &Path, criterion_dir: &Path) -> Result<String> {
    let relative = estimates_path
        .strip_prefix(criterion_dir)
        .context("Path is not under criterion directory")?;

    // Collect path components, excluding "base" and "estimates.json"
    let components: Vec<_> = relative
        .components()
        .filter_map(|c| {
            let s = c.as_os_str().to_string_lossy();
            if s != "base" && s != "estimates.json" {
                Some(s.into_owned())
            } else {
                None
            }
        })
        .collect();

    if components.is_empty() {
        bail!("Could not extract benchmark name from path: {}", estimates_path.display());
    }

    Ok(components.join("/"))
}

/// Parse Criterion estimates.json file
fn parse_criterion_estimates(path: &Path) -> Result<BenchmarkResult> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let estimates: CriterionEstimates = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    // Try to get sample count from benchmark.json (sibling file)
    let benchmark_json_path = path.parent()
        .and_then(|p| Some(p.join("benchmark.json")));

    let samples = if let Some(ref bj_path) = benchmark_json_path {
        if bj_path.exists() {
            // For now, default to 100 (Criterion default)
            // Could parse benchmark.json for actual count if needed
            100
        } else {
            100
        }
    } else {
        100
    };

    Ok(BenchmarkResult {
        mean_ns: estimates.mean.point_estimate,
        stddev_ns: estimates.std_dev.point_estimate,
        samples,
    })
}

/// Compare current benchmark results against a baseline
pub fn compare_baseline(
    _sh: &Shell,
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
        bail!(
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

    // Discover current benchmarks
    let criterion_dir = PathBuf::from("target/criterion");
    if !criterion_dir.exists() {
        bail!("Criterion output directory not found: {}\nRun benchmarks first: cargo bench", criterion_dir.display());
    }

    println!("\nDiscovering current benchmark results...");
    let current_benchmarks = discover_benchmarks(&criterion_dir)
        .context("Failed to discover current benchmarks")?;

    if current_benchmarks.is_empty() {
        bail!("No current benchmarks found. Run: cargo bench");
    }

    println!("Found {} current benchmarks", current_benchmarks.len());

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
                reg.baseline.mean_ns / 1000.0,
                reg.current.mean_ns / 1000.0,
                reg.change_percent,
                threshold_percent
            );
        }
        bail!("{} benchmark(s) regressed beyond threshold", regressions.len());
    }

    println!("\n✅ All benchmarks within performance threshold!");
    Ok(())
}

/// Compare baseline and current results
fn compare_results(
    baseline: &HashMap<String, BenchmarkResult>,
    current: &HashMap<String, BenchmarkResult>,
) -> Vec<Comparison> {
    let mut comparisons = Vec::new();

    for (name, baseline_result) in baseline {
        if let Some(current_result) = current.get(name) {
            let change_percent = ((current_result.mean_ns - baseline_result.mean_ns)
                / baseline_result.mean_ns)
                * 100.0;

            comparisons.push(Comparison {
                name: name.clone(),
                baseline: baseline_result.clone(),
                current: current_result.clone(),
                change_percent,
                is_regression: current_result.mean_ns > baseline_result.mean_ns,
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
    println!("{:<50} {:>12} {:>12} {:>10}", "Benchmark", "Baseline", "Current", "Change");
    println!("{}", "=".repeat(90));

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
            "{} {:<48} {:>10.2}µs {:>10.2}µs {:>9.1}%",
            symbol,
            truncate(&comp.name, 48),
            comp.baseline.mean_ns / 1000.0,
            comp.current.mean_ns / 1000.0,
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

    println!("\n{}", "=".repeat(90));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_benchmark_name_flat() {
        let criterion_dir = Path::new("target/criterion");
        let path = Path::new("target/criterion/fixture_loading_python_small/base/estimates.json");

        let name = extract_benchmark_name(path, criterion_dir).unwrap();
        assert_eq!(name, "fixture_loading_python_small");
    }

    #[test]
    fn test_extract_benchmark_name_grouped() {
        let criterion_dir = Path::new("target/criterion");
        let path = Path::new("target/criterion/real_parsing/parse_arithmetic/python_small/base/estimates.json");

        let name = extract_benchmark_name(path, criterion_dir).unwrap();
        assert_eq!(name, "real_parsing/parse_arithmetic/python_small");
    }

    #[test]
    fn test_parse_criterion_estimates() {
        use std::io::Write;

        let json = r#"{
            "mean": {
                "point_estimate": 6268.862773980828,
                "standard_error": 44.195293747354924,
                "confidence_interval": {
                    "confidence_level": 0.95,
                    "lower_bound": 6195.969135585125,
                    "upper_bound": 6366.260892037904
                }
            },
            "std_dev": {
                "point_estimate": 442.79896892051346,
                "standard_error": 142.07254888249162,
                "confidence_interval": {
                    "confidence_level": 0.95,
                    "lower_bound": 152.28083463396484,
                    "upper_bound": 680.3554002947283
                }
            }
        }"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_estimates.json");
        let mut file = std::fs::File::create(&temp_file).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let result = parse_criterion_estimates(&temp_file).unwrap();
        assert!((result.mean_ns - 6268.862773980828).abs() < 0.001);
        assert!((result.stddev_ns - 442.79896892051346).abs() < 0.001);
        assert_eq!(result.samples, 100);

        std::fs::remove_file(&temp_file).ok();
    }
}
