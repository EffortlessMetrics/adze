// TODO: Update these tests for the new GSS API
// The GSS API has changed significantly with the GLR implementation
// These tests need to be rewritten to work with the current ArenaGSS interface

/*
#[cfg(test)]
mod tests {
    use adze_glr_core::gss_arena::{ArenaGSS, ArenaStackNode};
    use adze_ir::{StateId, SymbolId};
    use typed_arena::Arena;

    #[test]
    fn test_arena_gss_shared_prefix() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));

        // Push on the main stack
        let config1 = gss.push(StateId(1), Some(SymbolId(10)));
        assert_eq!(config1.len(), 1);

        // Fork the stack
        let config2 = gss.fork(0);
        let config2 = gss.push_to(config2[0], StateId(2), Some(SymbolId(20)));

        // Both stacks share the initial state
        assert_eq!(config1[0].depth, 1);

        // Check parent sharing
        assert!(config1[0].parent.is_some());

        // The two configurations share the same parent
        assert!(config1[0].shares_prefix_with(config2[0]));
    }

    #[test]
    fn test_arena_gss_fork_and_merge() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));

        // Build initial stack: 0 -> 1 -> 2
        let config = gss.push(StateId(1), Some(SymbolId(10)));
        let config = gss.push_to(config[0], StateId(2), Some(SymbolId(20)));

        // Fork at state 0
        let forked = gss.fork(0);

        // Build different path on forked stack
        // ...additional test implementation...
    }

    #[test]
    fn test_arena_gss_stress() {
        // Stress test with many forks and merges
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));

        // Create multiple parallel paths
        for i in 0..10 {
            // Test implementation...
        }
    }

    #[test]
    fn test_arena_gss_independent_stacks() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));

        // Create two independent stacks
        let config1 = gss.push(StateId(1), Some(SymbolId(10)));
        let config2 = gss.push(StateId(2), Some(SymbolId(20)));
        let config3 = gss.fork(0);

        // They should have different states
        // Each should be independent

        assert_ne!(config1[0].state, config2[0].state);
    }

    #[test]
    fn test_arena_gss_sharing_detection() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));

        // Build a common prefix
        let path1 = gss.push(StateId(1), Some(SymbolId(10)));
        let path1 = gss.push_to(path1[0], StateId(2), Some(SymbolId(20)));

        // Fork and diverge
        let path2 = gss.fork(0);
        let path2 = gss.push_to(path2[0], StateId(3), Some(SymbolId(30)));
        let path2 = gss.push_to(path2[0], StateId(2), Some(SymbolId(20)));

        // Both paths end at state 2 but took different routes
        // Test implementation...
    }

    #[test]
    fn test_arena_gss_stats() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));

        // Create some operations
        let config1 = gss.push(StateId(1), Some(SymbolId(10)));
        let _config2 = gss.push_to(config1[0], StateId(2), Some(SymbolId(20)));
        let _fork = gss.fork(0);

        // Check statistics
        let stats = gss.get_stats();
        assert!(stats.total_nodes > 0);
        assert!(stats.shared_nodes >= 0);
        assert!(stats.fork_count > 0);

        println!("GSS Statistics:");
        println!("  Total nodes: {}", stats.total_nodes);
        println!("  Shared nodes: {}", stats.shared_nodes);
        println!("  Fork count: {}", stats.fork_count);
        println!("  Merge count: {}", stats.merge_count);
    }
}
*/
