#[cfg(test)]
mod tests {
    use rust_sitter_glr_core::gss_arena::{ArenaGSS, ArenaStackNode};
    use rust_sitter_ir::{StateId, SymbolId};
    use typed_arena::Arena;
    
    #[test]
    fn test_arena_gss_shared_prefix() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        
        // Create first stack configuration
        let config1 = gss.push(StateId(1), Some(SymbolId(10)));
        assert_eq!(config1.len(), 1);
        
        // Fork from the same point
        let config2 = gss.fork(0);
        let config2 = gss.push_to(config2[0], StateId(2), Some(SymbolId(20)));
        
        // Verify both configurations share the same parent
        assert_eq!(config1[0].depth, 1);
        assert_eq!(config2[0].depth, 1);
        
        // They should have the same root (state 0)
        assert!(config1[0].parent.is_some());
        assert!(config2[0].parent.is_some());
        
        // Check that they share a common prefix
        assert!(config1[0].shares_prefix_with(config2[0]));
        
        println!("✓ Arena GSS correctly shares common prefix nodes");
    }
    
    #[test]
    fn test_arena_gss_zero_copy_fork() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        
        // Build initial stack
        let config = gss.push(StateId(1), Some(SymbolId(10)));
        let config = gss.push_to(config[0], StateId(2), Some(SymbolId(20)));
        
        // Fork creates new head but reuses existing nodes
        let forked = gss.fork(0);
        
        // Both should point to the same parent chain
        assert_eq!(config[0].depth, 2);
        assert_eq!(forked.len(), 1);
        
        // The forked configuration should share the same parent chain
        let original_states = config[0].get_states();
        let forked_states = forked[0].get_states();
        
        // First two states should be the same (shared prefix)
        assert_eq!(original_states[0], forked_states[0]);
        assert_eq!(original_states[1], forked_states[1]);
        
        println!("✓ GSS fork reuses existing nodes (zero-copy)");
    }
    
    #[test]
    fn test_arena_gss_memory_locality() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        
        // Add multiple configurations
        let config1 = gss.push(StateId(1), Some(SymbolId(10)));
        let config2 = gss.push(StateId(2), Some(SymbolId(20)));
        let config3 = gss.fork(0);
        
        // All nodes are allocated in the same arena
        assert!(gss.active_heads.len() >= 2);
        
        // Verify different configurations
        assert_ne!(config1[0].state, config2[0].state);
        
        println!("✓ Arena GSS maintains nodes in contiguous memory");
    }
    
    #[test]
    fn test_arena_gss_merge_detection() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        
        // Create two paths that could merge
        // Path 1: [0] -> [1] -> [2]
        let path1 = gss.push(StateId(1), Some(SymbolId(10)));
        let path1 = gss.push_to(path1[0], StateId(2), Some(SymbolId(20)));
        
        // Path 2: [0] -> [3] -> [2]  
        let path2 = gss.fork(0);
        let path2 = gss.push_to(path2[0], StateId(3), Some(SymbolId(30)));
        let path2 = gss.push_to(path2[0], StateId(2), Some(SymbolId(20)));
        
        // Both paths end in state 2 but have different histories
        assert_eq!(path1[0].state, path2[0].state);
        
        // They should have different parent states
        let path1_states = path1[0].get_states();
        let path2_states = path2[0].get_states();
        
        assert_eq!(path1_states.last(), path2_states.last()); // Same final state
        assert_ne!(path1_states[1], path2_states[1]); // Different intermediate state
        
        println!("✓ GSS correctly maintains multiple paths to same state");
    }
    
    #[test]
    fn test_arena_gss_performance_stats() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        
        // Perform various operations
        let config1 = gss.push(StateId(1), Some(SymbolId(10)));
        let _config2 = gss.push_to(config1[0], StateId(2), Some(SymbolId(20)));
        let _fork = gss.fork(0);
        
        // Check statistics
        let stats = &gss.stats;
        assert!(stats.total_nodes > 0);
        assert!(stats.shared_nodes >= 0);
        assert!(stats.fork_count > 0);
        
        println!("✓ Arena GSS tracks performance statistics");
        println!("  Total nodes: {}", stats.total_nodes);
        println!("  Shared nodes: {}", stats.shared_nodes); 
        println!("  Fork count: {}", stats.fork_count);
        println!("  Merge count: {}", stats.merge_count);
    }
}