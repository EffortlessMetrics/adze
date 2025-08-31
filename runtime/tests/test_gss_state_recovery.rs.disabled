#[cfg(feature = "incremental_glr")]
#[cfg(test)]
mod gss_state_recovery_tests {
    use rust_sitter::glr_incremental::{GSSSnapshot, GSSStateMap, SUBTREE_REUSE_COUNT};
    use rust_sitter_ir::StateId;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_gss_snapshot_creation() {
        let snapshot = GSSSnapshot {
            token_position: 10,
            byte_position: 50,
            state: StateId(42),
            state_stack: vec![StateId(0), StateId(1), StateId(42)],
            partial_tree: None,
        };

        assert_eq!(snapshot.token_position, 10);
        assert_eq!(snapshot.byte_position, 50);
        assert_eq!(snapshot.state, StateId(42));
        assert_eq!(snapshot.state_stack.len(), 3);
    }

    #[test]
    fn test_gss_state_map_resume_point() {
        let mut state_map = GSSStateMap::new();

        // Add snapshots at various positions
        state_map.add_snapshot(GSSSnapshot {
            token_position: 0,
            byte_position: 0,
            state: StateId(0),
            state_stack: vec![StateId(0)],
            partial_tree: None,
        });

        state_map.add_snapshot(GSSSnapshot {
            token_position: 10,
            byte_position: 100,
            state: StateId(5),
            state_stack: vec![StateId(0), StateId(2), StateId(5)],
            partial_tree: None,
        });

        state_map.add_snapshot(GSSSnapshot {
            token_position: 20,
            byte_position: 200,
            state: StateId(10),
            state_stack: vec![StateId(0), StateId(2), StateId(5), StateId(10)],
            partial_tree: None,
        });

        // Test finding resume points
        let resume_point = state_map.find_resume_point(150);
        assert!(resume_point.is_some());
        assert_eq!(resume_point.unwrap().byte_position, 100);

        let resume_point = state_map.find_resume_point(250);
        assert!(resume_point.is_some());
        assert_eq!(resume_point.unwrap().byte_position, 200);

        // Test invalidation
        state_map.invalidate_after(150);
        let resume_point = state_map.find_resume_point(250);
        assert!(resume_point.is_none());
    }

    #[test]
    fn test_subtree_reuse_counter() {
        // Reset the counter
        SUBTREE_REUSE_COUNT.store(0, Ordering::SeqCst);

        // Simulate reuse
        SUBTREE_REUSE_COUNT.fetch_add(1, Ordering::SeqCst);
        SUBTREE_REUSE_COUNT.fetch_add(1, Ordering::SeqCst);

        assert_eq!(SUBTREE_REUSE_COUNT.load(Ordering::SeqCst), 2);
    }
}
