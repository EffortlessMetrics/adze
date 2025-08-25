/// Low-overhead telemetry for GLR parser performance monitoring
///
/// This module provides atomic counters for tracking fork, merge, and reduce
/// operations during GLR parsing. The telemetry is feature-gated to ensure
/// zero runtime cost when disabled.
#[cfg(feature = "glr_telemetry")]
use std::sync::atomic::{AtomicU64, Ordering};

/// Telemetry counters for GLR parser operations
#[cfg(feature = "glr_telemetry")]
#[derive(Default)]
pub struct Telemetry {
    /// Number of times the parser forked to explore multiple paths
    pub forks: AtomicU64,
    /// Number of times the parser merged compatible stacks
    pub merges: AtomicU64,
    /// Number of reduce operations performed
    pub reduces: AtomicU64,
    /// Number of shift operations performed
    pub shifts: AtomicU64,
    /// Maximum number of active stacks at any point
    pub max_stacks: AtomicU64,
    /// Total number of stacks created (including merged/dropped)
    pub total_stacks: AtomicU64,
}

#[cfg(feature = "glr_telemetry")]
impl Telemetry {
    /// Create new telemetry instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment fork counter
    #[inline(always)]
    pub fn inc_fork(&self) {
        self.forks.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment merge counter
    #[inline(always)]
    pub fn inc_merge(&self) {
        self.merges.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment reduce counter
    #[inline(always)]
    pub fn inc_reduce(&self) {
        self.reduces.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment shift counter
    #[inline(always)]
    pub fn inc_shift(&self) {
        self.shifts.fetch_add(1, Ordering::Relaxed);
    }

    /// Update maximum active stacks if current count is higher
    #[inline(always)]
    pub fn update_max_stacks(&self, current: u64) {
        let mut max = self.max_stacks.load(Ordering::Relaxed);
        while current > max {
            match self.max_stacks.compare_exchange_weak(
                max,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => max = x,
            }
        }
    }

    /// Increment total stacks counter
    #[inline(always)]
    pub fn inc_total_stacks(&self) {
        self.total_stacks.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current statistics
    pub fn stats(&self) -> TelemetryStats {
        TelemetryStats {
            forks: self.forks.load(Ordering::Relaxed),
            merges: self.merges.load(Ordering::Relaxed),
            reduces: self.reduces.load(Ordering::Relaxed),
            shifts: self.shifts.load(Ordering::Relaxed),
            max_stacks: self.max_stacks.load(Ordering::Relaxed),
            total_stacks: self.total_stacks.load(Ordering::Relaxed),
        }
    }

    /// Reset all counters to zero
    pub fn reset(&self) {
        self.forks.store(0, Ordering::Relaxed);
        self.merges.store(0, Ordering::Relaxed);
        self.reduces.store(0, Ordering::Relaxed);
        self.shifts.store(0, Ordering::Relaxed);
        self.max_stacks.store(0, Ordering::Relaxed);
        self.total_stacks.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of telemetry statistics
#[cfg(feature = "glr_telemetry")]
#[derive(Debug, Clone, Copy)]
pub struct TelemetryStats {
    pub forks: u64,
    pub merges: u64,
    pub reduces: u64,
    pub shifts: u64,
    pub max_stacks: u64,
    pub total_stacks: u64,
}

#[cfg(feature = "glr_telemetry")]
impl std::fmt::Display for TelemetryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GLR Stats: {} forks, {} merges, {} reduces, {} shifts, max {} stacks (total {})",
            self.forks, self.merges, self.reduces, self.shifts, self.max_stacks, self.total_stacks
        )
    }
}

// No-op implementation when telemetry is disabled
#[cfg(not(feature = "glr_telemetry"))]
#[derive(Default)]
pub struct Telemetry;

#[cfg(not(feature = "glr_telemetry"))]
impl Telemetry {
    #[inline(always)]
    pub fn new() -> Self {
        Self
    }
    #[inline(always)]
    pub fn inc_fork(&self) {}
    #[inline(always)]
    pub fn inc_merge(&self) {}
    #[inline(always)]
    pub fn inc_reduce(&self) {}
    #[inline(always)]
    pub fn inc_shift(&self) {}
    #[inline(always)]
    pub fn update_max_stacks(&self, _current: u64) {}
    #[inline(always)]
    pub fn inc_total_stacks(&self) {}

    pub fn stats(&self) -> TelemetryStats {
        TelemetryStats
    }

    pub fn reset(&self) {}
}

#[cfg(not(feature = "glr_telemetry"))]
#[derive(Debug, Clone, Copy)]
pub struct TelemetryStats;

#[cfg(not(feature = "glr_telemetry"))]
impl std::fmt::Display for TelemetryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GLR Stats: telemetry disabled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "glr_telemetry")]
    fn test_telemetry_counters() {
        let telemetry = Telemetry::new();

        telemetry.inc_fork();
        telemetry.inc_fork();
        telemetry.inc_merge();
        telemetry.inc_reduce();
        telemetry.inc_reduce();
        telemetry.inc_reduce();
        telemetry.inc_shift();

        let stats = telemetry.stats();
        assert_eq!(stats.forks, 2);
        assert_eq!(stats.merges, 1);
        assert_eq!(stats.reduces, 3);
        assert_eq!(stats.shifts, 1);

        telemetry.reset();
        let stats = telemetry.stats();
        assert_eq!(stats.forks, 0);
        assert_eq!(stats.merges, 0);
    }

    #[test]
    #[cfg(feature = "glr_telemetry")]
    fn test_max_stacks_tracking() {
        let telemetry = Telemetry::new();

        telemetry.update_max_stacks(5);
        assert_eq!(telemetry.stats().max_stacks, 5);

        telemetry.update_max_stacks(3);
        assert_eq!(telemetry.stats().max_stacks, 5); // Should not decrease

        telemetry.update_max_stacks(10);
        assert_eq!(telemetry.stats().max_stacks, 10);
    }

    #[test]
    fn test_no_op_when_disabled() {
        // This test works regardless of feature flag
        let telemetry = Telemetry::new();
        telemetry.inc_fork();
        telemetry.inc_merge();
        telemetry.inc_reduce();
        // Just ensure it doesn't crash
    }
}
