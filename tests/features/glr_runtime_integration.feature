# GLR Runtime Integration - BDD Scenarios
# Purpose: Define acceptance criteria for wiring parser_v4.rs as default GLR runtime
# Related: ARCHITECTURE_ISSUE_GLR_PARSER.md
# Target: v0.7.0

Feature: GLR Runtime Integration
  As a rust-sitter user
  I want correct precedence and associativity in pure-Rust mode
  So that my grammars parse expressions according to their rules

  Background:
    Given rust-sitter is configured with pure-rust feature
    And the GLR runtime (parser_v4.rs) is wired as the default parser

  # ==============================================================================
  # Scenario Group 1: Left Associativity
  # ==============================================================================

  Scenario: Left-associative multiplication
    Given a grammar with left-associative multiplication at precedence 2
    """rust
    #[rust_sitter::prec_left(2)]
    Mul(Box<Expr>, #[leaf(text = "*")] (), Box<Expr>)
    """
    When I parse "1 * 2 * 3"
    Then the result should be a left-associative tree
    And the structure should be ((1 * 2) * 3)
    And not (1 * (2 * 3))

  Scenario: Left-associative addition
    Given a grammar with left-associative addition at precedence 1
    """rust
    #[rust_sitter::prec_left(1)]
    Add(Box<Expr>, #[leaf(text = "+")] (), Box<Expr>)
    """
    When I parse "5 + 10 + 15"
    Then the result should be ((5 + 10) + 15)

  Scenario: Chain of left-associative operations
    Given a grammar with left-associative subtraction
    When I parse "100 - 20 - 10 - 5"
    Then the result should be (((100 - 20) - 10) - 5)
    And the final value should be 65

  # ==============================================================================
  # Scenario Group 2: Right Associativity
  # ==============================================================================

  Scenario: Right-associative exponentiation
    Given a grammar with right-associative power operation
    """rust
    #[rust_sitter::prec_right(3)]
    Power(Box<Expr>, #[leaf(text = "^")] (), Box<Expr>)
    """
    When I parse "2 ^ 3 ^ 4"
    Then the result should be (2 ^ (3 ^ 4))
    And not ((2 ^ 3) ^ 4)

  Scenario: Right-associative assignment
    Given a grammar with right-associative assignment
    When I parse "a = b = c = 5"
    Then the result should be (a = (b = (c = 5)))

  # ==============================================================================
  # Scenario Group 3: Mixed Precedence
  # ==============================================================================

  Scenario: Addition and multiplication with correct precedence
    Given a grammar with:
      | Operator | Precedence | Associativity |
      | +        | 1          | left          |
      | *        | 2          | left          |
    When I parse "1 + 2 * 3"
    Then the multiplication should bind tighter
    And the result should be (1 + (2 * 3))
    And the final value should be 7

  Scenario: Three levels of precedence
    Given a grammar with:
      | Operator | Precedence | Associativity |
      | or       | 1          | left          |
      | and      | 2          | left          |
      | ==       | 3          | left          |
    When I parse "a or b and c == d"
    Then the result should be (a or (b and (c == d)))

  Scenario: Parentheses override precedence
    Given a grammar with parentheses and operators
    When I parse "(1 + 2) * 3"
    Then the addition should be evaluated first
    And the result should be ((1 + 2) * 3)
    And the final value should be 9

  # ==============================================================================
  # Scenario Group 4: GLR Fork/Merge Behavior
  # ==============================================================================

  Scenario: Ambiguous grammar with multiple valid parses
    Given an intentionally ambiguous grammar
    """rust
    #[rust_sitter::grammar("ambig")]
    pub enum Expr {
        Binop(Box<Expr>, Op, Box<Expr>),  // No precedence
        Num(i32),
    }
    """
    When I parse "1 + 2 * 3"
    Then the GLR parser should explore both interpretations:
      | Parse Tree 1      | Parse Tree 2      |
      | (1 + 2) * 3       | 1 + (2 * 3)       |
    And both parse forests should be preserved
    And the user can select the desired interpretation

  Scenario: Conflict resolution via precedence
    Given a grammar with shift/reduce conflict
    And precedence annotations resolve the conflict
    When parsing encounters the conflict state
    Then only the higher-precedence action should be taken
    And no forking should occur
    And the parse should complete deterministically

  # ==============================================================================
  # Scenario Group 5: Runtime Feature Selection
  # ==============================================================================

  Scenario: Feature flag enables GLR runtime
    Given Cargo.toml contains `features = ["pure-rust", "glr"]`
    When I compile the grammar
    Then __private::parse() should use parser_v4::Parser
    And not pure_parser::Parser

  Scenario: Default feature uses tree-sitter C runtime
    Given Cargo.toml contains only `rust-sitter = "0.7"`
    When I compile the grammar
    Then the tree-sitter C backend should be used
    And GLR behavior should be correct via C runtime

  Scenario: Pure-rust without GLR uses simple LR
    Given Cargo.toml contains `features = ["pure-rust"]`
    And not the "glr" feature
    When I compile a grammar without conflicts
    Then pure_parser::Parser should be used
    And parsing should work for LR-compatible grammars

  # ==============================================================================
  # Scenario Group 6: Error Cases
  # ==============================================================================

  Scenario: Grammar with conflicts but no resolution
    Given a grammar with unresolved shift/reduce conflicts
    And no precedence annotations
    When I compile the grammar in GLR mode
    Then the compiler should warn about conflicts
    But should still generate a parser
    And the GLR runtime should handle conflicts via forking

  Scenario: Invalid precedence levels
    Given a grammar with precedence level 1000
    When I compile the grammar
    Then the compiler should warn about unusually high precedence
    But should accept it and encode correctly

  Scenario: Empty input
    Given any grammar
    When I parse ""
    Then the parser should handle empty input gracefully
    And return appropriate error or empty result

  # ==============================================================================
  # Scenario Group 7: Regression Tests
  # ==============================================================================

  Scenario: Python grammar state 0 bug (Issue #GLR-1)
    Given the Python grammar with empty module rule
    """python
    module: REPEAT(_statement)
    """
    When I parse a Python file starting with "def"
    Then state 0 should have both shift and reduce actions
    And the GLR runtime should fork correctly
    And the parse should succeed

  Scenario: Arithmetic associativity regression
    Given the arithmetic example grammar
    When I parse "1 * 2 * 3"
    Then this should NOT regress to right-associative
    And should maintain left-associative behavior
    And tests in example/src/arithmetic.rs should pass

  # ==============================================================================
  # Acceptance Criteria for v0.7.0 Release
  # ==============================================================================

  # ✅ All tests in runtime/tests/test_action_decoding.rs pass
  # ✅ All tests in example/src/arithmetic.rs pass (no #[ignore])
  # ✅ Python grammar (273 symbols) parses correctly
  # ✅ Feature flags work: default, pure-rust, pure-rust+glr
  # ✅ No regressions in existing passing tests
  # ✅ GLR fork/merge logic handles ambiguous grammars
  # ✅ Precedence and associativity work correctly in all modes
  # ✅ ARCHITECTURE_ISSUE_GLR_PARSER.md marked as resolved
