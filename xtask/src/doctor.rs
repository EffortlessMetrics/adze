//! Local environment doctor for Adze development.
//!
//! Runs a battery of fast checks and reports pass/fail/skip for each.
//! Exit code 0 when all required checks pass, 1 otherwise.

use anyhow::Result;
use std::process::Command;

const MSRV: &str = "1.95.0";

struct CheckResult {
    name: &'static str,
    status: CheckStatus,
    detail: String,
}

#[derive(Debug, Eq, PartialEq)]
enum CheckStatus {
    Pass,
    Fail,
    Skip,
}

pub fn run() -> Result<()> {
    let results = vec![
        check_rustc_version(),
        check_cargo_available(),
        check_rustfmt_available(),
        check_clippy_available(),
        check_just_available(),
        check_workspace_metadata(),
        check_wasm32_target(),
    ];

    let any_fail = results
        .iter()
        .any(|r| matches!(r.status, CheckStatus::Fail));

    println!("\n--- adze doctor ---\n");
    for r in &results {
        let icon = match r.status {
            CheckStatus::Pass => "ok",
            CheckStatus::Fail => "FAIL",
            CheckStatus::Skip => "skip",
        };
        println!("  [{icon:>4}] {} {}", r.name, r.detail);
    }
    println!();

    if any_fail {
        anyhow::bail!("one or more required checks failed");
    }
    Ok(())
}

fn run_capture(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn check_rustc_version() -> CheckResult {
    let output = run_capture("rustc", &["--version"]);
    match output {
        Some(ver) => {
            let version_str = ver.strip_prefix("rustc ").unwrap_or(&ver);
            let ok = meets_msrv(version_str);
            CheckResult {
                name: "rustc",
                status: if ok {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Fail
                },
                detail: format!("({version_str}) {} {MSRV}", if ok { ">=" } else { "<" }),
            }
        }
        None => CheckResult {
            name: "rustc",
            status: CheckStatus::Fail,
            detail: String::from("not found"),
        },
    }
}

fn check_cargo_available() -> CheckResult {
    let output = run_capture("cargo", &["--version"]);
    match output {
        Some(ver) => CheckResult {
            name: "cargo",
            status: CheckStatus::Pass,
            detail: format!("({ver})"),
        },
        None => CheckResult {
            name: "cargo",
            status: CheckStatus::Fail,
            detail: String::from("not found"),
        },
    }
}

fn check_rustfmt_available() -> CheckResult {
    let output = run_capture("cargo", &["fmt", "--", "--version"]);
    match output {
        Some(_) => CheckResult {
            name: "rustfmt",
            status: CheckStatus::Pass,
            detail: String::from("available"),
        },
        None => CheckResult {
            name: "rustfmt",
            status: CheckStatus::Fail,
            detail: String::from("not found (install via rustup component add rustfmt)"),
        },
    }
}

fn check_clippy_available() -> CheckResult {
    let output = run_capture("cargo", &["clippy", "--version"]);
    match output {
        Some(ver) => CheckResult {
            name: "clippy",
            status: CheckStatus::Pass,
            detail: format!("({ver})"),
        },
        None => CheckResult {
            name: "clippy",
            status: CheckStatus::Fail,
            detail: String::from("not found (install via rustup component add clippy)"),
        },
    }
}

fn check_just_available() -> CheckResult {
    let output = run_capture("just", &["--version"]);
    match output {
        Some(ver) => CheckResult {
            name: "just",
            status: CheckStatus::Pass,
            detail: format!("({ver})"),
        },
        None => CheckResult {
            name: "just",
            status: CheckStatus::Fail,
            detail: String::from("not found (install via cargo install just)"),
        },
    }
}

fn check_workspace_metadata() -> CheckResult {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let count = serde_json::from_slice::<serde_json::Value>(&o.stdout)
                .ok()
                .and_then(|v| v["packages"].as_array().map(|a| a.len()))
                .unwrap_or(0);
            CheckResult {
                name: "workspace",
                status: CheckStatus::Pass,
                detail: format!("{count} packages resolve"),
            }
        }
        _ => CheckResult {
            name: "workspace",
            status: CheckStatus::Fail,
            detail: String::from("cargo metadata failed"),
        },
    }
}

fn check_wasm32_target() -> CheckResult {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let installed = String::from_utf8_lossy(&o.stdout);
            let has_wasm = installed.contains("wasm32-unknown-unknown");
            CheckResult {
                name: "wasm32 target",
                status: if has_wasm {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Skip
                },
                detail: if has_wasm {
                    String::from("installed")
                } else {
                    String::from(
                        "not installed (optional, rustup target add wasm32-unknown-unknown)",
                    )
                },
            }
        }
        _ => CheckResult {
            name: "wasm32 target",
            status: CheckStatus::Skip,
            detail: String::from("rustup not available"),
        },
    }
}

/// Minimal semver parser: returns (major, minor, patch) from "1.95.0" style strings.
/// Handles trailing metadata like "1.95.0 (hash date)" or "1.95.0-nightly".
fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.split(&[' ', '(', '-'][..]).next()?;
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

fn meets_msrv(version_str: &str) -> bool {
    let Some(version) = parse_semver(version_str) else {
        return false;
    };
    let msrv = parse_semver(MSRV).expect("MSRV constant should be valid semver");
    version >= msrv
}

#[cfg(test)]
mod tests {
    use super::{meets_msrv, parse_semver};

    #[test]
    fn parse_semver_accepts_stable_and_prerelease_rustc_versions() {
        assert_eq!(parse_semver("1.95.0"), Some((1, 95, 0)));
        assert_eq!(parse_semver("1.96.0-nightly"), Some((1, 96, 0)));
        assert_eq!(parse_semver("1.95.0 (abcdef 2026-01-01)"), Some((1, 95, 0)));
        assert_eq!(parse_semver("1.95"), Some((1, 95, 0)));
    }

    #[test]
    fn parse_semver_rejects_invalid_versions() {
        assert_eq!(parse_semver("rustc 1.95.0"), None);
        assert_eq!(parse_semver("not-a-version"), None);
        assert_eq!(parse_semver("1.x.0"), None);
    }

    #[test]
    fn meets_msrv_compares_against_workspace_msrv() {
        assert!(!meets_msrv("1.94.9"));
        assert!(meets_msrv("1.95.0"));
        assert!(meets_msrv("1.96.0-nightly"));
    }
}
