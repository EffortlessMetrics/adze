// Lexicographic symbol comparison for Tree-sitter conflict resolution
//! Lexicographic symbol comparison as a tie-breaker for conflict resolution.

// This implements the final tie-breaker when all other comparisons are equal

use crate::CompareResult;
use adze_ir::SymbolId;

/// Compare two parse trees by their root symbols lexicographically
/// This is Tree-sitter's final tie-breaker for conflict resolution
pub fn compare_symbols(left_symbol: SymbolId, right_symbol: SymbolId) -> CompareResult {
    match left_symbol.0.cmp(&right_symbol.0) {
        std::cmp::Ordering::Less => CompareResult::TakeLeft,
        std::cmp::Ordering::Greater => CompareResult::TakeRight,
        std::cmp::Ordering::Equal => CompareResult::Tie,
    }
}

/// Extended comparison that includes symbol comparison as final tie-breaker
pub fn compare_versions_with_symbols(
    left_version: &crate::VersionInfo,
    right_version: &crate::VersionInfo,
    left_symbol: SymbolId,
    right_symbol: SymbolId,
) -> CompareResult {
    // First, use the standard version comparison
    let version_result = crate::compare_versions(left_version, right_version);

    // If versions are tied, use symbol comparison
    match version_result {
        CompareResult::Tie => compare_symbols(left_symbol, right_symbol),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_comparison() {
        // Lower symbol ID wins
        assert_eq!(
            compare_symbols(SymbolId(10), SymbolId(20)),
            CompareResult::TakeLeft
        );

        assert_eq!(
            compare_symbols(SymbolId(30), SymbolId(15)),
            CompareResult::TakeRight
        );

        assert_eq!(
            compare_symbols(SymbolId(42), SymbolId(42)),
            CompareResult::Tie
        );
    }

    #[test]
    fn test_full_comparison_with_symbols() {
        let v1 = crate::VersionInfo::new();
        let v2 = crate::VersionInfo::new();

        // When versions are equal, symbols are the tie-breaker
        assert_eq!(
            compare_versions_with_symbols(&v1, &v2, SymbolId(1), SymbolId(2)),
            CompareResult::TakeLeft
        );

        // When versions differ, symbols are ignored
        let mut v3 = crate::VersionInfo::new();
        v3.add_dynamic_prec(5);

        assert_eq!(
            compare_versions_with_symbols(&v3, &v1, SymbolId(100), SymbolId(1)),
            CompareResult::TakeLeft // v3 wins due to higher dynamic precedence
        );
    }
}
