//! Comprehensive thread safety and concurrency tests for the adze runtime crate.
//!
//! This test suite verifies:
//! - Send/Sync trait bounds for core types
//! - Concurrent creation and access patterns
//! - Thread-safe sharing via Arc/Mutex
//! - Channel-based communication with runtime types
//! - No data races with proper synchronization
//! - Thread-local storage compatibility
//! - Clone and Debug thread safety
//!
//! All tests use compile-time trait assertions and runtime concurrency validation.

use adze::Spanned;
use adze::error_recovery::{ErrorNode, ErrorRecoveryConfig, ErrorRecoveryState, RecoveryStrategy};
use adze::pure_parser::Point;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

// ============================================================================
// TESTS 1-6: Send/Sync Trait Bounds (Compile-Time Checks)
// ============================================================================

/// Test 1: Verify Spanned<T> is Send for any Send T
#[test]
fn test_spanned_is_send() {
    assert_send::<Spanned<i32>>();
    assert_send::<Spanned<String>>();
    assert_send::<Spanned<Vec<u8>>>();
}

/// Test 2: Verify Spanned<T> is Sync for any Sync T
#[test]
fn test_spanned_is_sync() {
    assert_sync::<Spanned<i32>>();
    assert_sync::<Spanned<String>>();
    assert_sync::<Spanned<Vec<u8>>>();
}

/// Test 3: Verify Point is Send + Sync
#[test]
fn test_point_is_send_sync() {
    assert_send::<Point>();
    assert_sync::<Point>();
}

/// Test 4: Verify ErrorRecoveryState is Send
#[test]
fn test_error_recovery_state_is_send() {
    assert_send::<ErrorRecoveryState>();
}

/// Test 5: Verify ErrorRecoveryConfig is Send + Sync
#[test]
fn test_error_recovery_config_is_send_sync() {
    assert_send::<ErrorRecoveryConfig>();
    assert_sync::<ErrorRecoveryConfig>();
}

/// Test 6: Verify RecoveryStrategy is Send + Sync
#[test]
fn test_recovery_strategy_is_send_sync() {
    assert_send::<RecoveryStrategy>();
    assert_sync::<RecoveryStrategy>();
}

// ============================================================================
// TESTS 7-10: Concurrent Creation and Access
// ============================================================================

/// Test 7: Concurrent Spanned<T> creation from multiple threads
#[test]
fn test_concurrent_spanned_creation() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                let spanned = Spanned {
                    value: format!("value_{}", i),
                    span: (i * 10, (i + 1) * 10),
                };
                assert_eq!(spanned.span.1 - spanned.span.0, 10);
                spanned
            })
        })
        .collect();

    for h in handles {
        let spanned = h.join().expect("thread should complete");
        assert!(!spanned.value.is_empty());
    }
}

/// Test 8: Concurrent Point creation from multiple threads
#[test]
fn test_concurrent_point_creation() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                let point = Point {
                    row: i as u32,
                    column: (i * 2) as u32,
                };
                assert_eq!(point.row, i as u32);
                point
            })
        })
        .collect();

    for h in handles {
        let point = h.join().expect("thread should complete");
        assert!(point.row <= 3);
    }
}

/// Test 9: Read Spanned values from multiple threads simultaneously
#[test]
fn test_concurrent_spanned_reads() {
    let shared_spanned = Arc::new(Spanned {
        value: vec![1, 2, 3, 4, 5],
        span: (0, 100),
    });

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&shared_spanned);
            thread::spawn(move || {
                assert_eq!(s.value.len(), 5);
                assert_eq!(s.span.0, 0);
                assert_eq!(s.span.1, 100);
                s.value.iter().sum::<i32>()
            })
        })
        .collect();

    let mut total = 0;
    for h in handles {
        total += h.join().expect("thread should complete");
    }
    assert_eq!(total, 60); // 4 threads × (1+2+3+4+5)
}

/// Test 10: Read Point values from multiple threads simultaneously
#[test]
fn test_concurrent_point_reads() {
    let shared_point = Arc::new(Point {
        row: 42,
        column: 13,
    });

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let p = Arc::clone(&shared_point);
            thread::spawn(move || {
                assert_eq!(p.row, 42);
                assert_eq!(p.column, 13);
                (p.row, p.column)
            })
        })
        .collect();

    for h in handles {
        let (row, col) = h.join().expect("thread should complete");
        assert_eq!(row, 42);
        assert_eq!(col, 13);
    }
}

// ============================================================================
// TESTS 11-14: Arc-based Sharing and Channel Communication
// ============================================================================

/// Test 11: Arc<Spanned<T>> shared across threads with mutation protection
#[test]
fn test_arc_spanned_shared_across_threads() {
    let spanned = Arc::new(Spanned {
        value: 42,
        span: (10, 20),
    });

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let s = Arc::clone(&spanned);
            thread::spawn(move || {
                // All threads see the same value
                assert_eq!(s.value, 42);
                assert_eq!(s.span.1 - s.span.0, 10);
                i
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread should complete");
    }
}

/// Test 12: Spanned<T> sent through MPSC channel
#[test]
fn test_spanned_in_mpsc_channel() {
    let (tx, rx) = mpsc::channel();

    let sender_handle = thread::spawn(move || {
        for i in 0..4 {
            let spanned = Spanned {
                value: i * 100,
                span: (i, i + 1),
            };
            tx.send(spanned).expect("send should succeed");
        }
    });

    let mut count = 0;
    while let Ok(spanned) = rx.recv() {
        #[allow(clippy::absurd_extreme_comparisons, unused_comparisons)]
        {
            assert!(spanned.value >= 0);
        }
        assert!(spanned.span.1 > spanned.span.0);
        count += 1;
    }

    sender_handle.join().expect("sender should complete");
    assert_eq!(count, 4);
}

/// Test 13: Point sent through MPSC channel
#[test]
fn test_point_in_mpsc_channel() {
    let (tx, rx) = mpsc::channel();

    let sender_handle = thread::spawn(move || {
        for i in 0..4 {
            let point = Point {
                row: i as u32,
                column: (i * 2) as u32,
            };
            tx.send(point).expect("send should succeed");
        }
    });

    let mut points = Vec::new();
    while let Ok(point) = rx.recv() {
        points.push(point);
    }

    sender_handle.join().expect("sender should complete");
    assert_eq!(points.len(), 4);
    assert_eq!(points[0].row, 0);
    assert_eq!(points[3].row, 3);
}

/// Test 14: ErrorRecoveryConfig sent through MPSC channel
#[test]
fn test_error_recovery_config_in_channel() {
    let (tx, rx) = mpsc::channel();

    let sender_handle = thread::spawn(move || {
        let config = ErrorRecoveryConfig::default();
        tx.send(config).expect("send should succeed");
    });

    let config = rx.recv().expect("receive should succeed");
    assert_eq!(config.max_panic_skip, 50);

    sender_handle.join().expect("sender should complete");
}

// ============================================================================
// TESTS 15-18: Shared State with Mutex Protection
// ============================================================================

/// Test 15: Parallel error recording with thread-safe state
#[test]
fn test_parallel_error_recording() {
    let error_count = Arc::new(Mutex::new(0usize));

    let handles: Vec<_> = (0..4)
        .map(|_i| {
            let count = Arc::clone(&error_count);
            thread::spawn(move || {
                for _ in 0..3 {
                    let mut guard = count.lock().expect("lock should succeed");
                    *guard += 1;
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread should complete");
    }

    let final_count = *error_count.lock().expect("lock should succeed");
    assert_eq!(final_count, 12); // 4 threads × 3 increments
}

/// Test 16: Multiple reader threads for shared ErrorRecoveryConfig
#[test]
fn test_multiple_readers_error_config() {
    let config = Arc::new(ErrorRecoveryConfig::default());

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let c = Arc::clone(&config);
            thread::spawn(move || {
                assert_eq!(c.max_panic_skip, 50);
                assert_eq!(c.max_token_deletions, 3);
                assert_eq!(c.max_token_insertions, 2);
                true
            })
        })
        .collect();

    for h in handles {
        assert!(h.join().expect("thread should complete"));
    }
}

/// Test 17: Spanned<T> in thread-local storage
#[test]
fn test_spanned_in_thread_local() {
    thread_local! {
        static SPANNED: Spanned<i32> = const { Spanned {
            value: 99,
            span: (0, 10),
        } };
    }

    let handles: Vec<_> = (0..3)
        .map(|_| {
            thread::spawn(|| {
                SPANNED.with(|s| {
                    assert_eq!(s.value, 99);
                    s.span.1 - s.span.0
                })
            })
        })
        .collect();

    for h in handles {
        let span_size = h.join().expect("thread should complete");
        assert_eq!(span_size, 10);
    }
}

/// Test 18: Point comparison from different threads
#[test]
fn test_point_comparison_across_threads() {
    let point1 = Arc::new(Point { row: 5, column: 10 });
    let point2 = Arc::new(Point { row: 5, column: 10 });

    let h1 = {
        let p = Arc::clone(&point1);
        thread::spawn(move || *p)
    };

    let h2 = {
        let p = Arc::clone(&point2);
        thread::spawn(move || *p)
    };

    let p1 = h1.join().expect("thread 1 should complete");
    let p2 = h2.join().expect("thread 2 should complete");

    assert_eq!(p1, p2);
}

// ============================================================================
// TESTS 19-22: Clone and Debug Thread Safety
// ============================================================================

/// Test 19: Concurrent clone operations on Spanned
#[test]
fn test_concurrent_spanned_clones() {
    let spanned = Arc::new(Spanned {
        value: "test_value".to_string(),
        span: (5, 15),
    });

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&spanned);
            thread::spawn(move || {
                let cloned = (*s).clone();
                assert_eq!(cloned.value, "test_value");
                assert_eq!(cloned.span, (5, 15));
                cloned
            })
        })
        .collect();

    for h in handles {
        let cloned = h.join().expect("thread should complete");
        assert_eq!(cloned.value, "test_value");
    }
}

/// Test 20: Thread-safe iteration over error nodes
#[test]
fn test_concurrent_error_node_iteration() {
    let error_nodes = Arc::new(vec![
        ErrorNode {
            start_byte: 0,
            end_byte: 5,
            start_position: (0, 0),
            end_position: (0, 5),
            expected: vec![1, 2, 3],
            actual: Some(4),
            recovery: RecoveryStrategy::PanicMode,
            skipped_tokens: vec![],
        },
        ErrorNode {
            start_byte: 5,
            end_byte: 10,
            start_position: (0, 5),
            end_position: (0, 10),
            expected: vec![5, 6],
            actual: Some(7),
            recovery: RecoveryStrategy::TokenDeletion,
            skipped_tokens: vec![],
        },
    ]);

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let nodes = Arc::clone(&error_nodes);
            thread::spawn(move || {
                let mut count = 0;
                for node in nodes.iter() {
                    assert!(node.end_byte > node.start_byte);
                    count += 1;
                }
                count
            })
        })
        .collect();

    for h in handles {
        let count = h.join().expect("thread should complete");
        assert_eq!(count, 2);
    }
}

/// Test 21: Verify Clone trait is thread-safe for all types
#[test]
fn test_clone_thread_safety() {
    let original_spanned = Spanned {
        value: 42,
        span: (0, 100),
    };

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let s = original_spanned.clone();
            thread::spawn(move || {
                let cloned = s.clone();
                assert_eq!(cloned.value, 42);
                assert_eq!(cloned.span, (0, 100));
                i
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread should complete");
    }
}

/// Test 22: Verify Debug trait is thread-safe
#[test]
fn test_debug_thread_safety() {
    let spanned = Arc::new(Spanned {
        value: "debug_test".to_string(),
        span: (10, 20),
    });

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&spanned);
            thread::spawn(move || {
                let debug_str = format!("{:?}", &*s);
                assert!(!debug_str.is_empty());
                debug_str.contains("debug_test")
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("thread should complete"))
        .collect();

    assert!(results.iter().all(|&b| b));
}

// ============================================================================
// TESTS 23-25: Advanced Concurrency Patterns
// ============================================================================

/// Test 23: Shared read-only Spanned across threads with no mutation
#[test]
fn test_shared_readonly_spanned() {
    let shared = Arc::new(Spanned {
        value: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        span: (0, 50),
    });

    let handles: Vec<_> = (0..4)
        .map(|_i| {
            let s = Arc::clone(&shared);
            thread::spawn(move || {
                // Multiple simultaneous reads, no synchronization needed
                let sum: i32 = s.value.iter().sum();
                let len = s.value.len();
                let span_size = s.span.1 - s.span.0;
                (sum, len, span_size)
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("thread should complete"))
        .collect();

    for (sum, len, span_size) in results {
        assert_eq!(sum, 55);
        assert_eq!(len, 10);
        assert_eq!(span_size, 50);
    }
}

/// Test 24: ErrorRecoveryState independent per thread
#[test]
fn test_error_recovery_state_per_thread() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                let config = ErrorRecoveryConfig::default();
                let _state = ErrorRecoveryState::new(config);

                // Each thread has its own independent state
                // (State is Send, allowing movement across thread boundary)
                i
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread should complete");
    }
}

/// Test 25: Verify no data races with concurrent access patterns
#[test]
fn test_no_data_races_concurrent_access() {
    let spanned = Arc::new(Spanned {
        value: Arc::new(Mutex::new(vec![1, 2, 3])),
        span: (0, 30),
    });

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let s = Arc::clone(&spanned);
            thread::spawn(move || {
                // Thread safely reads shared data through Mutex
                let guard = s.value.lock().expect("lock should succeed");
                let sum: i32 = guard.iter().sum();

                // Also access the span (no synchronization needed for Copy types)
                let span_size = s.span.1 - s.span.0;

                (sum, span_size, i)
            })
        })
        .collect();

    let mut valid_reads = 0;
    for h in handles {
        let (sum, span_size, _) = h.join().expect("thread should complete");
        assert_eq!(sum, 6); // 1 + 2 + 3
        assert_eq!(span_size, 30);
        valid_reads += 1;
    }

    assert_eq!(valid_reads, 4);
}

// ============================================================================
// BONUS TESTS: Additional Comprehensive Coverage
// ============================================================================

/// Bonus Test 1: Spanned deref through Arc is thread-safe
#[test]
fn test_spanned_deref_thread_safe() {
    let spanned = Arc::new(Spanned {
        value: 777,
        span: (0, 50),
    });

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&spanned);
            thread::spawn(move || {
                // Access value field
                let val = s.value;
                assert_eq!(val, 777);
                val
            })
        })
        .collect();

    for h in handles {
        assert_eq!(h.join().expect("thread should complete"), 777);
    }
}

/// Bonus Test 2: Point Copy semantics thread-safe
#[test]
fn test_point_copy_semantics() {
    let point = Point {
        row: 10,
        column: 20,
    };

    let handles: Vec<_> = (0..4)
        .map(|_| {
            // Point is Copy, so it's implicitly cloned
            thread::spawn(move || {
                assert_eq!(point.row, 10);
                assert_eq!(point.column, 20);
                point
            })
        })
        .collect();

    for h in handles {
        let p = h.join().expect("thread should complete");
        assert_eq!(p.row, 10);
    }
}

/// Bonus Test 3: Spanned equality across threads
#[test]
fn test_spanned_equality_across_threads() {
    let spanned1 = Arc::new(Spanned {
        value: 42,
        span: (0, 10),
    });
    let spanned2 = Arc::new(Spanned {
        value: 42,
        span: (0, 10),
    });

    let h1 = {
        let s = Arc::clone(&spanned1);
        thread::spawn(move || (*s).clone())
    };

    let h2 = {
        let s = Arc::clone(&spanned2);
        thread::spawn(move || (*s).clone())
    };

    let s1 = h1.join().expect("thread 1 should complete");
    let s2 = h2.join().expect("thread 2 should complete");

    assert_eq!(s1.value, s2.value);
    assert_eq!(s1.span, s2.span);
}

/// Bonus Test 4: ErrorRecoveryConfig clone thread safety
#[test]
fn test_error_recovery_config_clone_thread_safe() {
    let config = Arc::new(ErrorRecoveryConfig::default());

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let c = Arc::clone(&config);
            thread::spawn(move || {
                let cloned = (*c).clone();
                assert_eq!(cloned.max_panic_skip, 50);
                cloned
            })
        })
        .collect();

    for h in handles {
        let config = h.join().expect("thread should complete");
        assert_eq!(config.max_token_deletions, 3);
    }
}

/// Bonus Test 5: Concurrent Arc strong count (indirect thread safety test)
#[test]
fn test_arc_strong_count_concurrent() {
    let spanned = Arc::new(Spanned {
        value: "arc_test".to_string(),
        span: (0, 20),
    });

    let initial_count = Arc::strong_count(&spanned);
    assert_eq!(initial_count, 1);

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let s = Arc::clone(&spanned);
            thread::spawn(move || Arc::strong_count(&s))
        })
        .collect();

    // All handles should see at least 2 references (original + own clone)
    for h in handles {
        let count = h.join().expect("thread should complete");
        assert!(
            count >= 2,
            "strong count should be at least 2, got {}",
            count
        );
    }

    // After joins, should be back to 1
    let final_count = Arc::strong_count(&spanned);
    assert_eq!(final_count, 1);
}
