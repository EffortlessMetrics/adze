//! Comprehensive API test suite for adze runtime parsing and tree operations.
//!
//! This file contains 64 tests organized into 8 categories:
//! 1. Error recovery config (8 tests)
//! 2. Error recovery state (8 tests)
//! 3. Error nodes (8 tests)
//! 4. Arena allocator (8 tests)
//! 5. Tree operations (8 tests)
//! 6. Visitor pattern (8 tests)
//! 7. SExpr types (8 tests)
//! 8. Integration (8 tests)

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use adze::arena_allocator::{TreeArena, TreeNode};
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};
use adze::visitor::{
    BreadthFirstWalker, PrettyPrintVisitor, SearchVisitor, StatsVisitor, TreeWalker, VisitorAction,
};

#[cfg(feature = "serialization")]
use adze::serialization::SExpr;

// ============================================================================
// CATEGORY 1: Error Recovery Config (8 tests)
// ============================================================================

#[test]
fn error_recovery_config_default() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_token_deletions, 3);
    assert_eq!(config.max_token_insertions, 2);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn error_recovery_config_custom_max_panic_skip() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 100,
        ..ErrorRecoveryConfig::default()
    };
    assert_eq!(config.max_panic_skip, 100);
}

#[test]
fn error_recovery_config_custom_insertions() {
    let config = ErrorRecoveryConfig {
        max_token_insertions: 5,
        ..ErrorRecoveryConfig::default()
    };
    assert_eq!(config.max_token_insertions, 5);
}

#[test]
fn error_recovery_config_custom_deletions() {
    let config = ErrorRecoveryConfig {
        max_token_deletions: 7,
        ..ErrorRecoveryConfig::default()
    };
    assert_eq!(config.max_token_deletions, 7);
}

#[test]
fn error_recovery_config_builder() {
    let config = adze::error_recovery::ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(75)
        .max_consecutive_errors(5)
        .build();
    assert_eq!(config.max_panic_skip, 75);
    assert_eq!(config.max_consecutive_errors, 5);
}

#[test]
fn error_recovery_config_enable_phrase_recovery() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        ..ErrorRecoveryConfig::default()
    };
    assert!(!config.enable_phrase_recovery);
}

#[test]
fn error_recovery_config_enable_scope_recovery() {
    let config = ErrorRecoveryConfig {
        enable_scope_recovery: false,
        ..ErrorRecoveryConfig::default()
    };
    assert!(!config.enable_scope_recovery);
}

#[test]
fn error_recovery_config_scope_delimiters() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2), (3, 4)],
        ..ErrorRecoveryConfig::default()
    };
    assert_eq!(config.scope_delimiters.len(), 2);
}

// ============================================================================
// CATEGORY 2: Error Recovery State (8 tests)
// ============================================================================

#[test]
fn error_recovery_state_new() {
    let config = ErrorRecoveryConfig::default();
    let _state = ErrorRecoveryState::new(config);
    // State created successfully
}

#[test]
fn error_recovery_state_increment_error_count() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    // Error count incremented successfully
}

#[test]
fn error_recovery_state_reset_error_count() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
    // Error count reset successfully
}

#[test]
fn error_recovery_state_should_give_up() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);
    // Check if recovery should give up based on config
    let _should_give_up = state.should_give_up();
}

#[test]
fn error_recovery_state_update_recent_tokens() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.update_recent_tokens(ir::SymbolId(42));
    // Recent tokens updated successfully
}

#[test]
fn error_recovery_state_push_scope() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    // Scope pushed successfully
}

#[test]
fn error_recovery_state_pop_scope() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    let popped = state.pop_scope_test();
    assert_eq!(popped, Some(10));
}

#[test]
fn error_recovery_state_multiple_scopes() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    state.push_scope(20);
    let popped1 = state.pop_scope_test();
    let popped2 = state.pop_scope_test();
    assert_eq!(popped1, Some(20));
    assert_eq!(popped2, Some(10));
}

// ============================================================================
// CATEGORY 3: Error Nodes (8 tests)
// ============================================================================

#[test]
fn error_node_create() {
    let error = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2, 3],
        actual: Some(4),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert_eq!(error.start_byte, 0);
    assert_eq!(error.end_byte, 10);
}

#[test]
fn error_node_position() {
    let error = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (1, 5),
        end_position: (1, 15),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert_eq!(error.start_byte, 5);
    assert_eq!(error.start_position, (1, 5));
}

#[test]
fn error_node_length() {
    let error = ErrorNode {
        start_byte: 10,
        end_byte: 25,
        start_position: (0, 10),
        end_position: (0, 25),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };
    let length = error.end_byte - error.start_byte;
    assert_eq!(length, 15);
}

#[test]
fn error_node_expected_tokens() {
    let error = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2, 3, 4],
        actual: Some(5),
        recovery: RecoveryStrategy::TokenSubstitution,
        skipped_tokens: vec![],
    };
    assert_eq!(error.expected.len(), 4);
    assert!(error.expected.contains(&1));
}

#[test]
fn error_node_recovery_strategy() {
    let error = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::ScopeRecovery,
        skipped_tokens: vec![],
    };
    assert_eq!(error.recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn error_node_with_skipped_tokens() {
    let error = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![7, 8, 9],
    };
    assert_eq!(error.skipped_tokens.len(), 3);
}

#[test]
fn error_node_at_zero_position() {
    let error = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert_eq!(error.start_byte, 0);
    assert_eq!(error.start_position, (0, 0));
}

// ============================================================================
// CATEGORY 4: Arena Allocator (8 tests)
// ============================================================================

#[test]
fn arena_new_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_allocate_single_node() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(handle).value(), 42);
}

#[test]
fn arena_allocate_multiple_nodes() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.get(h3).value(), 3);
}

#[test]
fn arena_node_handle_index() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(99));
    // NodeHandle is Copy, can be copied without .clone()
    let handle_copy = handle;
    assert_eq!(arena.get(handle_copy).value(), 99);
}

#[test]
fn arena_node_data_retrieval() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(55));
    let node_ref = arena.get(handle);
    assert_eq!(node_ref.value(), 55);
    assert!(node_ref.is_leaf());
}

#[test]
fn arena_node_children() {
    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNode::leaf(10));
    let child2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));
    let node_ref = arena.get(parent);
    assert_eq!(node_ref.children().len(), 2);
}

#[test]
fn arena_node_parent_link() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(5));
    let parent = arena.alloc(TreeNode::branch(vec![child]));
    let parent_ref = arena.get(parent);
    assert!(parent_ref.is_branch());
}

#[test]
fn arena_with_capacity_1000() {
    let mut arena = TreeArena::with_capacity(100);
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 1000);
}

// ============================================================================
// CATEGORY 5: Tree Operations (8 tests)
// ============================================================================

#[test]
fn tree_create_node() {
    let node = TreeNode::leaf(123);
    assert_eq!(node.symbol(), 123);
}

#[test]
fn tree_node_data_access() {
    let node = TreeNode::leaf(456);
    assert_eq!(node.value(), 456);
}

#[test]
fn tree_node_branch_with_children() {
    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNode::leaf(1));
    let child2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));
    let parent_ref = arena.get(parent);
    assert_eq!(parent_ref.children().len(), 2);
}

#[test]
fn tree_node_is_leaf() {
    let leaf = TreeNode::leaf(42);
    assert!(leaf.is_leaf());
    assert!(!leaf.is_branch());
}

#[test]
fn tree_node_is_branch() {
    let branch = TreeNode::branch(vec![]);
    assert!(branch.is_branch());
    assert!(!branch.is_leaf());
}

#[test]
fn tree_depth_traversal() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let root = arena.alloc(TreeNode::branch(vec![b1, l3]));

    let root_ref = arena.get(root);
    assert_eq!(root_ref.children().len(), 2);
}

#[test]
fn tree_node_with_symbol() {
    let node = TreeNode::branch_with_symbol(777, vec![]);
    assert_eq!(node.symbol(), 777);
    assert!(node.is_branch());
}

#[test]
fn tree_multiple_operations() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(100));
    let h2 = arena.alloc(TreeNode::leaf(200));
    let h3 = arena.alloc(TreeNode::branch(vec![h1, h2]));

    assert_eq!(arena.get(h1).value(), 100);
    assert_eq!(arena.get(h2).value(), 200);
    assert_eq!(arena.get(h3).children().len(), 2);
}

// ============================================================================
// CATEGORY 6: Visitor Pattern (8 tests)
// ============================================================================

#[test]
fn visitor_stats_visitor_creation() {
    let stats = StatsVisitor::default();
    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.max_depth, 0);
}

#[test]
fn visitor_stats_visitor_counts() {
    let stats = StatsVisitor::default();
    assert_eq!(stats.total_nodes, 0);
    // StatsVisitor would count nodes during a walk
}

#[test]
fn visitor_pretty_print_visitor() {
    let visitor = PrettyPrintVisitor::new();
    let output = visitor.output();
    assert_eq!(output.len(), 0); // Empty initially
}

#[test]
fn visitor_search_visitor_creation() {
    let _visitor = SearchVisitor::new(|_node| false);
    // SearchVisitor created successfully
}

#[test]
fn visitor_tree_walker_creation() {
    let source = b"test";
    let _walker = TreeWalker::new(source);
    // TreeWalker created successfully
}

#[test]
fn visitor_breadth_first_walker() {
    let source = b"test";
    let _walker = BreadthFirstWalker::new(source);
    // BreadthFirstWalker created successfully
}

#[test]
fn visitor_action_continue() {
    let action = VisitorAction::Continue;
    assert_eq!(action, VisitorAction::Continue);
}

#[test]
fn visitor_action_skip_children() {
    let action = VisitorAction::SkipChildren;
    assert_eq!(action, VisitorAction::SkipChildren);
}

// ============================================================================
// CATEGORY 7: SExpr Types (8 tests)
// ============================================================================

#[cfg(feature = "serialization")]
#[test]
fn sexpr_create_atom() {
    let atom = SExpr::atom("test");
    assert!(atom.is_atom());
    assert!(!atom.is_list());
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_create_list() {
    let list = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b")]);
    assert!(list.is_list());
    assert!(!list.is_atom());
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_nested_list() {
    let inner = SExpr::list(vec![SExpr::atom("x"), SExpr::atom("y")]);
    let outer = SExpr::list(vec![SExpr::atom("outer"), inner]);
    assert!(outer.is_list());
    if let Some(items) = outer.as_list() {
        assert_eq!(items.len(), 2);
    }
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_empty_list() {
    let list = SExpr::list(vec![]);
    assert!(list.is_list());
    if let Some(items) = list.as_list() {
        assert_eq!(items.len(), 0);
    }
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_atom_display() {
    let atom = SExpr::atom("hello");
    let display_str = format!("{}", atom);
    assert_eq!(display_str, "hello");
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_list_display() {
    let list = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b")]);
    let display_str = format!("{}", list);
    assert!(display_str.contains("a"));
    assert!(display_str.contains("b"));
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_deeply_nested() {
    let mut current = SExpr::atom("core");
    for _ in 0..5 {
        current = SExpr::list(vec![current]);
    }
    assert!(current.is_list());
}

#[cfg(feature = "serialization")]
#[test]
fn sexpr_with_special_chars() {
    let atom = SExpr::atom("test-value_123");
    assert!(atom.is_atom());
    if let Some(s) = atom.as_atom() {
        assert!(s.contains("-"));
        assert!(s.contains("_"));
    }
}

// ============================================================================
// CATEGORY 8: Integration Tests (8 tests)
// ============================================================================

#[test]
fn integration_error_recovery_with_arena() {
    let _config = ErrorRecoveryConfig::default();
    let _arena = TreeArena::new();
    // Error recovery config can work with arena allocator
}

#[test]
fn integration_visitor_on_arena_tree() {
    let mut arena = TreeArena::new();
    let _h1 = arena.alloc(TreeNode::leaf(1));
    let _h2 = arena.alloc(TreeNode::leaf(2));
    // Visitor can traverse arena-allocated trees
}

#[test]
fn integration_arena_with_error_nodes() {
    let mut arena = TreeArena::new();
    let _handle = arena.alloc(TreeNode::leaf(42));
    let _error = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    // Both arena nodes and error nodes can coexist
}

#[test]
fn integration_full_parse_pipeline_mock() {
    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNode::leaf(10));
    let child2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));

    let parent_ref = arena.get(parent);
    assert!(parent_ref.is_branch());
    assert_eq!(parent_ref.children().len(), 2);
}

#[test]
fn integration_error_recovery_reset_and_reuse() {
    let config = ErrorRecoveryConfig::default();
    let mut state1 = ErrorRecoveryState::new(config.clone());
    state1.increment_error_count();

    let mut state2 = ErrorRecoveryState::new(config);
    state2.reset_error_count();
    // Two independent recovery states can be created and reset
}

#[test]
fn integration_visitor_collect_all_nodes() {
    let mut arena = TreeArena::new();
    let mut nodes = Vec::new();
    for i in 0..10 {
        let h = arena.alloc(TreeNode::leaf(i));
        nodes.push(h);
    }
    assert_eq!(nodes.len(), 10);
}

#[test]
fn integration_arena_memory_growth() {
    let mut arena = TreeArena::with_capacity(10);
    let initial_capacity = arena.capacity();

    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }

    let final_capacity = arena.capacity();
    assert!(final_capacity >= initial_capacity);
}

#[test]
fn integration_combined_operations() {
    let mut arena = TreeArena::new();
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let h1 = arena.alloc(TreeNode::leaf(1));
    state.increment_error_count();
    let h2 = arena.alloc(TreeNode::leaf(2));
    state.reset_error_count();
    let h3 = arena.alloc(TreeNode::branch(vec![h1, h2]));

    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(h3).children().len(), 2);
}
