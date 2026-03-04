#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for ExternalScanner trait and related types.

use adze_runtime::external_scanner::{ExternalScanner, ScanResult};

// ---------------------------------------------------------------------------
// Test helpers: custom scanner implementations
// ---------------------------------------------------------------------------

/// A minimal no-op scanner that never matches.
struct NullScanner;

impl ExternalScanner for NullScanner {
    fn init(&mut self) {}
    fn scan(&mut self, _valid_symbols: &[bool], _input: &[u8]) -> Option<ScanResult> {
        None
    }
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
    fn deserialize(&mut self, _data: &[u8]) {}
}

/// Scanner that recognises a single keyword ("fn") when token 0 is valid.
struct KeywordScanner {
    call_count: u32,
}

impl KeywordScanner {
    fn new() -> Self {
        Self { call_count: 0 }
    }
}

impl ExternalScanner for KeywordScanner {
    fn init(&mut self) {
        self.call_count = 0;
    }

    fn scan(&mut self, valid_symbols: &[bool], input: &[u8]) -> Option<ScanResult> {
        self.call_count += 1;
        if valid_symbols.first().copied().unwrap_or(false) && input.starts_with(b"fn") {
            Some(ScanResult {
                token_type: 0,
                bytes_consumed: 2,
            })
        } else {
            None
        }
    }

    fn serialize(&self) -> Vec<u8> {
        self.call_count.to_le_bytes().to_vec()
    }

    fn deserialize(&mut self, data: &[u8]) {
        if data.len() >= 4 {
            self.call_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        }
    }
}

/// Scanner that tracks indentation (mirrors the crate-internal IndentationScanner).
struct IndentScanner {
    indent_stack: Vec<u32>,
}

impl IndentScanner {
    fn new() -> Self {
        Self {
            indent_stack: vec![0],
        }
    }
}

impl ExternalScanner for IndentScanner {
    fn init(&mut self) {
        self.indent_stack.clear();
        self.indent_stack.push(0);
    }

    fn scan(&mut self, _valid_symbols: &[bool], input: &[u8]) -> Option<ScanResult> {
        let indent = input.iter().take_while(|&&b| b == b' ').count() as u32;
        let top = *self.indent_stack.last()?;

        if indent > top {
            self.indent_stack.push(indent);
            Some(ScanResult {
                token_type: 0, // INDENT
                bytes_consumed: 0,
            })
        } else if indent < top {
            while self.indent_stack.len() > 1 && indent < *self.indent_stack.last().unwrap_or(&0) {
                self.indent_stack.pop();
            }
            Some(ScanResult {
                token_type: 1, // DEDENT
                bytes_consumed: 0,
            })
        } else {
            None
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(self.indent_stack.len() as u32).to_le_bytes());
        for &indent in &self.indent_stack {
            data.extend_from_slice(&indent.to_le_bytes());
        }
        data
    }

    fn deserialize(&mut self, data: &[u8]) {
        if data.len() >= 4 {
            let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            self.indent_stack.clear();
            for i in 0..len {
                let offset = 4 + i * 4;
                if offset + 4 <= data.len() {
                    let val = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    self.indent_stack.push(val);
                }
            }
        }
    }
}

/// Multi-token scanner that recognises several string literals.
struct MultiTokenScanner;

impl ExternalScanner for MultiTokenScanner {
    fn init(&mut self) {}

    fn scan(&mut self, valid_symbols: &[bool], input: &[u8]) -> Option<ScanResult> {
        let candidates: &[(&[u8], u32)] = &[(b"let", 0), (b"mut", 1), (b"if", 2), (b"else", 3)];
        for &(kw, id) in candidates {
            let idx = id as usize;
            if idx < valid_symbols.len() && valid_symbols[idx] && input.starts_with(kw) {
                return Some(ScanResult {
                    token_type: id,
                    bytes_consumed: kw.len(),
                });
            }
        }
        None
    }

    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
    fn deserialize(&mut self, _data: &[u8]) {}
}

// ========================== ScanResult tests ===============================

#[test]
fn scan_result_construction() {
    let r = ScanResult {
        token_type: 42,
        bytes_consumed: 10,
    };
    assert_eq!(r.token_type, 42);
    assert_eq!(r.bytes_consumed, 10);
}

#[test]
fn scan_result_clone_copy() {
    let r = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    let r2 = r; // Copy
    let r3 = r;
    assert_eq!(r, r2);
    assert_eq!(r, r3);
}

#[test]
fn scan_result_equality() {
    let a = ScanResult {
        token_type: 0,
        bytes_consumed: 0,
    };
    let b = ScanResult {
        token_type: 0,
        bytes_consumed: 0,
    };
    let c = ScanResult {
        token_type: 1,
        bytes_consumed: 0,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn scan_result_debug() {
    let r = ScanResult {
        token_type: 7,
        bytes_consumed: 3,
    };
    let dbg = format!("{r:?}");
    assert!(dbg.contains("token_type"));
    assert!(dbg.contains("bytes_consumed"));
}

#[test]
fn scan_result_zero_bytes() {
    let r = ScanResult {
        token_type: 0,
        bytes_consumed: 0,
    };
    assert_eq!(r.bytes_consumed, 0);
}

#[test]
fn scan_result_large_token_type() {
    let r = ScanResult {
        token_type: u32::MAX,
        bytes_consumed: usize::MAX,
    };
    assert_eq!(r.token_type, u32::MAX);
    assert_eq!(r.bytes_consumed, usize::MAX);
}

// ========================== NullScanner tests ==============================

#[test]
fn null_scanner_returns_none() {
    let mut s = NullScanner;
    s.init();
    assert!(s.scan(&[true, true], b"hello").is_none());
}

#[test]
fn null_scanner_empty_input() {
    let mut s = NullScanner;
    s.init();
    assert!(s.scan(&[true], b"").is_none());
}

#[test]
fn null_scanner_no_valid_symbols() {
    let mut s = NullScanner;
    s.init();
    assert!(s.scan(&[], b"hello").is_none());
}

#[test]
fn null_scanner_serialize_empty() {
    let s = NullScanner;
    assert!(s.serialize().is_empty());
}

#[test]
fn null_scanner_deserialize_empty() {
    let mut s = NullScanner;
    s.deserialize(&[]); // should not panic
}

// ========================== KeywordScanner tests ===========================

#[test]
fn keyword_scanner_match() {
    let mut s = KeywordScanner::new();
    s.init();
    let result = s.scan(&[true], b"fn main()");
    assert_eq!(
        result,
        Some(ScanResult {
            token_type: 0,
            bytes_consumed: 2,
        })
    );
}

#[test]
fn keyword_scanner_no_match() {
    let mut s = KeywordScanner::new();
    s.init();
    assert!(s.scan(&[true], b"let x = 1").is_none());
}

#[test]
fn keyword_scanner_invalid_symbol_slot() {
    let mut s = KeywordScanner::new();
    s.init();
    // Token 0 is not valid → should not match even though input starts with "fn".
    assert!(s.scan(&[false], b"fn main()").is_none());
}

#[test]
fn keyword_scanner_empty_valid_symbols() {
    let mut s = KeywordScanner::new();
    s.init();
    assert!(s.scan(&[], b"fn").is_none());
}

#[test]
fn keyword_scanner_empty_input() {
    let mut s = KeywordScanner::new();
    s.init();
    assert!(s.scan(&[true], b"").is_none());
}

#[test]
fn keyword_scanner_serialize_roundtrip() {
    let mut s = KeywordScanner::new();
    s.init();
    // Perform some scans to change state
    s.scan(&[true], b"fn");
    s.scan(&[true], b"fn");
    s.scan(&[true], b"xyz");
    assert_eq!(s.call_count, 3);

    let data = s.serialize();
    let mut s2 = KeywordScanner::new();
    s2.deserialize(&data);
    assert_eq!(s2.call_count, 3);
}

#[test]
fn keyword_scanner_init_resets_state() {
    let mut s = KeywordScanner::new();
    s.scan(&[true], b"fn");
    assert_eq!(s.call_count, 1);
    s.init();
    assert_eq!(s.call_count, 0);
}

#[test]
fn keyword_scanner_deserialize_short_data() {
    let mut s = KeywordScanner::new();
    s.call_count = 99;
    s.deserialize(&[1, 2]); // too short — state should be unchanged
    assert_eq!(s.call_count, 99);
}

// ========================== IndentScanner tests ============================

#[test]
fn indent_scanner_indent() {
    let mut s = IndentScanner::new();
    s.init();
    let r = s.scan(&[true, true], b"    code");
    assert_eq!(
        r,
        Some(ScanResult {
            token_type: 0,
            bytes_consumed: 0,
        })
    );
}

#[test]
fn indent_scanner_dedent() {
    let mut s = IndentScanner::new();
    s.init();
    // First indent
    s.scan(&[true, true], b"    code");
    // Then dedent
    let r = s.scan(&[true, true], b"code");
    assert_eq!(
        r,
        Some(ScanResult {
            token_type: 1,
            bytes_consumed: 0,
        })
    );
}

#[test]
fn indent_scanner_same_level_returns_none() {
    let mut s = IndentScanner::new();
    s.init();
    assert!(s.scan(&[true, true], b"code").is_none()); // level 0 → 0
}

#[test]
fn indent_scanner_multi_indent_dedent() {
    let mut s = IndentScanner::new();
    s.init();
    // level 0 → 4
    assert!(s.scan(&[true, true], b"    a").is_some());
    // level 4 → 8
    assert!(s.scan(&[true, true], b"        a").is_some());
    // level 8 → 0 (dedent)
    let r = s.scan(&[true, true], b"a");
    assert_eq!(r.unwrap().token_type, 1); // DEDENT
}

#[test]
fn indent_scanner_serialize_roundtrip() {
    let mut s = IndentScanner::new();
    s.init();
    s.scan(&[true, true], b"    a"); // push 4
    s.scan(&[true, true], b"        a"); // push 8

    let data = s.serialize();
    let mut s2 = IndentScanner::new();
    s2.deserialize(&data);

    // s2 should behave identically: dedent from 8 → 0
    let r = s2.scan(&[true, true], b"a");
    assert_eq!(r.unwrap().token_type, 1);
}

#[test]
fn indent_scanner_empty_input() {
    let mut s = IndentScanner::new();
    s.init();
    // indent=0, top=0 → same level → None
    assert!(s.scan(&[true, true], b"").is_none());
}

#[test]
fn indent_scanner_deserialize_empty_data() {
    let mut s = IndentScanner::new();
    s.init();
    s.scan(&[true, true], b"    a"); // push 4
    s.deserialize(&[]); // too short — stack unchanged
    assert!(s.scan(&[true, true], b"a").is_some()); // still has indent
}

#[test]
fn indent_scanner_init_after_use() {
    let mut s = IndentScanner::new();
    s.scan(&[true, true], b"    a"); // push 4
    s.init();
    // After init the stack is [0], so level-0 input yields None.
    assert!(s.scan(&[true, true], b"a").is_none());
}

// ========================== MultiTokenScanner tests ========================

#[test]
fn multi_token_scanner_first_keyword() {
    let mut s = MultiTokenScanner;
    s.init();
    let r = s.scan(&[true, true, true, true], b"let x");
    assert_eq!(
        r,
        Some(ScanResult {
            token_type: 0,
            bytes_consumed: 3,
        })
    );
}

#[test]
fn multi_token_scanner_second_keyword() {
    let mut s = MultiTokenScanner;
    s.init();
    let r = s.scan(&[true, true, true, true], b"mut y");
    assert_eq!(
        r,
        Some(ScanResult {
            token_type: 1,
            bytes_consumed: 3,
        })
    );
}

#[test]
fn multi_token_scanner_respects_valid_symbols() {
    let mut s = MultiTokenScanner;
    s.init();
    // "let" would match token 0, but it's invalid.
    let r = s.scan(&[false, false, true, true], b"let x");
    assert!(r.is_none());
}

#[test]
fn multi_token_scanner_no_match() {
    let mut s = MultiTokenScanner;
    s.init();
    assert!(s.scan(&[true, true, true, true], b"return").is_none());
}

// ========================== Trait object & Send+Sync tests =================

#[test]
fn scanner_as_trait_object() {
    let mut scanners: Vec<Box<dyn ExternalScanner>> = vec![
        Box::new(NullScanner),
        Box::new(KeywordScanner::new()),
        Box::new(IndentScanner::new()),
        Box::new(MultiTokenScanner),
    ];
    for s in &mut scanners {
        s.init();
        let _ = s.scan(&[true], b"fn");
    }
}

#[test]
fn scanner_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NullScanner>();
    assert_send_sync::<KeywordScanner>();
    assert_send_sync::<IndentScanner>();
    assert_send_sync::<MultiTokenScanner>();
}

// ========================== TSExternalScanner / VTable tests ================

#[cfg(feature = "external_scanners")]
mod ffi_tests {
    use adze_runtime::external_scanner::{TSExternalScanner, TSExternalScannerVTable};
    use std::mem;

    #[test]
    fn ts_external_scanner_is_repr_c_sized() {
        // TSExternalScanner must have a stable, non-zero size for FFI.
        let size = mem::size_of::<TSExternalScanner>();
        assert!(size > 0, "TSExternalScanner should have non-zero size");
    }

    #[test]
    fn ts_external_scanner_vtable_is_repr_c_sized() {
        let size = mem::size_of::<TSExternalScannerVTable>();
        assert!(
            size > 0,
            "TSExternalScannerVTable should have non-zero size"
        );
        // A vtable with 5 function pointers should be 5 * pointer size.
        assert_eq!(size, 5 * mem::size_of::<usize>());
    }
}
