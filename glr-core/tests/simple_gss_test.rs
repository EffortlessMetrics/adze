#[cfg(test)]
mod tests {
    use rust_sitter_glr_core::gss_arena::{ArenaGSS, ArenaStackNode};
    use rust_sitter_ir::{StateId, SymbolId};
    use typed_arena::Arena;
    
    #[test]
    fn test_arena_gss_basic_operations() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        
        // Initial state
        assert_eq!(gss.active_heads.len(), 1);
        
        // Push a new state
        gss.push(0, StateId(1), Some(SymbolId(10)));
        
        // Fork creates a new head
        let fork_idx = gss.fork_head(0);
        assert_eq!(gss.active_heads.len(), 2);
        assert_eq!(fork_idx, 1);
        
        // Push different states to each fork
        gss.push(0, StateId(2), Some(SymbolId(20)));
        gss.push(1, StateId(3), Some(SymbolId(30)));
        
        // Verify they have different states
        assert_eq!(gss.active_heads[0].state, StateId(2));
        assert_eq!(gss.active_heads[1].state, StateId(3));
        
        // But they share the same parent chain
        let parent0 = gss.active_heads[0].parent;
        let parent1 = gss.active_heads[1].parent;
        
        assert!(parent0.is_some());
        assert!(parent1.is_some());
        
        // Both should have state 1 as parent
        assert_eq!(parent0.unwrap().state, StateId(1));
        assert_eq!(parent1.unwrap().state, StateId(1));
        
        // Check statistics
        assert!(gss.stats.total_nodes_created > 0);
        assert_eq!(gss.stats.total_forks, 1);
        assert_eq!(gss.stats.max_active_heads, 2);
        
        println!("✓ Arena GSS basic operations work correctly");
        println!("  Total nodes created: {}", gss.stats.total_nodes_created);
        println!("  Total forks: {}", gss.stats.total_forks);
        println!("  Arena bytes: {}", gss.stats.arena_bytes_allocated);
    }
}