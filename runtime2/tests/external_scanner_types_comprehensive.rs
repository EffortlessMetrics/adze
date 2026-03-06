//! Comprehensive tests for runtime2 external scanner types.

use adze_runtime::external_scanner::ScanResult;

// ── ScanResult construction ──

#[test]
fn scan_result_new() {
    let r = ScanResult {
        token_type: 0,
        bytes_consumed: 0,
    };
    assert_eq!(r.token_type, 0);
    assert_eq!(r.bytes_consumed, 0);
}

#[test]
fn scan_result_nonzero() {
    let r = ScanResult {
        token_type: 5,
        bytes_consumed: 10,
    };
    assert_eq!(r.token_type, 5);
    assert_eq!(r.bytes_consumed, 10);
}

#[test]
fn scan_result_max_token() {
    let r = ScanResult {
        token_type: u32::MAX,
        bytes_consumed: 0,
    };
    assert_eq!(r.token_type, u32::MAX);
}

#[test]
fn scan_result_max_bytes() {
    let r = ScanResult {
        token_type: 0,
        bytes_consumed: usize::MAX,
    };
    assert_eq!(r.bytes_consumed, usize::MAX);
}

#[test]
fn scan_result_debug() {
    let r = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    let s = format!("{:?}", r);
    assert!(s.contains("1"));
    assert!(s.contains("5"));
}

#[test]
fn scan_result_clone() {
    let r = ScanResult {
        token_type: 3,
        bytes_consumed: 7,
    };
    let c = r;
    assert_eq!(c.token_type, 3);
    assert_eq!(c.bytes_consumed, 7);
}

#[test]
fn scan_result_copy() {
    let r = ScanResult {
        token_type: 2,
        bytes_consumed: 4,
    };
    let c = r;
    assert_eq!(r.token_type, c.token_type); // r still usable if Copy
    assert_eq!(r.bytes_consumed, c.bytes_consumed);
}

// ── ScanResult eq ──

#[test]
fn scan_result_eq() {
    let a = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    let b = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    assert_eq!(a, b);
}

#[test]
fn scan_result_ne_token() {
    let a = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    let b = ScanResult {
        token_type: 2,
        bytes_consumed: 5,
    };
    assert_ne!(a, b);
}

#[test]
fn scan_result_ne_bytes() {
    let a = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    let b = ScanResult {
        token_type: 1,
        bytes_consumed: 6,
    };
    assert_ne!(a, b);
}

// ── ScanResult patterns ──

#[test]
fn scan_result_no_match() {
    let r = ScanResult {
        token_type: 0,
        bytes_consumed: 0,
    };
    assert_eq!(r.bytes_consumed, 0);
}

#[test]
fn scan_result_single_byte() {
    let r = ScanResult {
        token_type: 1,
        bytes_consumed: 1,
    };
    assert_eq!(r.bytes_consumed, 1);
}

#[test]
fn scan_result_multi_byte() {
    let r = ScanResult {
        token_type: 2,
        bytes_consumed: 100,
    };
    assert_eq!(r.bytes_consumed, 100);
}

// ── Trait checks ──

#[test]
fn scan_result_is_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<ScanResult>();
}

#[test]
fn scan_result_is_clone() {
    fn check<T: Clone>() {}
    check::<ScanResult>();
}

#[test]
fn scan_result_is_partialeq() {
    fn check<T: PartialEq>() {}
    check::<ScanResult>();
}

// ── Size checks ──

#[test]
fn scan_result_not_zero_size() {
    assert!(std::mem::size_of::<ScanResult>() > 0);
}

#[test]
fn scan_result_reasonable_size() {
    // Should be about 16 bytes (u32 + usize with padding)
    assert!(std::mem::size_of::<ScanResult>() <= 24);
}

// ── Multiple results ──

#[test]
fn scan_result_vec() {
    let results: Vec<ScanResult> = (0..10)
        .map(|i| ScanResult {
            token_type: i as u32,
            bytes_consumed: i * 2,
        })
        .collect();
    assert_eq!(results.len(), 10);
    assert_eq!(results[5].token_type, 5);
    assert_eq!(results[5].bytes_consumed, 10);
}

// ── Destructure ──

#[test]
fn scan_result_destructure() {
    let ScanResult {
        token_type,
        bytes_consumed,
    } = ScanResult {
        token_type: 3,
        bytes_consumed: 7,
    };
    assert_eq!(token_type, 3);
    assert_eq!(bytes_consumed, 7);
}
