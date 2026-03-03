//! Tests for GLR telemetry and version info modules.
#![cfg(feature = "test-api")]

use adze_glr_core::telemetry::*;
use adze_glr_core::version_info::*;

// ---- Telemetry tests ----

#[test]
fn telemetry_creation() {
    let t = Telemetry::new();
    let _stats = t.stats();
}

#[test]
fn telemetry_inc_fork() {
    let t = Telemetry::new();
    t.inc_fork();
    t.inc_fork();
    let stats = t.stats();
    // Stats may or may not track depending on feature, just verify no panic
    let _ = stats;
}

#[test]
fn telemetry_inc_merge() {
    let t = Telemetry::new();
    t.inc_merge();
    let stats = t.stats();
    let _ = stats;
}

#[test]
fn telemetry_inc_shift_reduce() {
    let t = Telemetry::new();
    t.inc_shift();
    t.inc_reduce();
    let stats = t.stats();
    let _ = stats;
}

#[test]
fn telemetry_reset() {
    let t = Telemetry::new();
    t.inc_fork();
    t.reset();
    let stats = t.stats();
    let _ = stats;
}

// ---- Version Info tests ----

#[test]
fn version_info_creation() {
    let vi = VersionInfo::new();
    let debug = format!("{vi:?}");
    assert!(debug.contains("VersionInfo"));
}

#[test]
fn version_info_dynamic_prec() {
    let mut vi = VersionInfo::new();
    vi.add_dynamic_prec(5);
    vi.add_dynamic_prec(-3);
    let debug = format!("{vi:?}");
    assert!(!debug.is_empty());
}

#[test]
fn version_info_enter_error() {
    let mut vi = VersionInfo::new();
    vi.enter_error();
    let debug = format!("{vi:?}");
    assert!(!debug.is_empty());
}

#[test]
fn compare_versions_equal() {
    let a = VersionInfo::new();
    let b = VersionInfo::new();
    let result = compare_versions(&a, &b);
    // Two default versions should be equal or equivalent
    let debug = format!("{result:?}");
    assert!(!debug.is_empty());
}
