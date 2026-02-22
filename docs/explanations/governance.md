# Adze Governance Model: BDD-Driven Evolution

Adze uses a unique **Behavior-Driven Development (BDD)** governance model to ensure the stability and correctness of the parser runtime as it evolves. This model treats grammar specifications as formal contracts that the runtime must satisfy.

## The Core Problem

Parser runtimes are notoriously difficult to change. A small optimization or "fix" in the lexer or state machine logic can subtly break existing grammars in ways that aren't immediately obvious. In many projects, this leads to:
- Regressions that only appear in production.
- Fear of refactoring core components.
- Inconsistent behavior between different backends (e.g., C vs. pure-Rust).

## The BDD Solution

Adze solves this by embedding Gherkin-style feature specifications directly into the codebase. These specifications define the **expected behavior** of the parser in terms of high-level scenarios.

### 1. Feature Specifications

We define parser features in `.feature` files (using the `adze-bdd-*` crates). For example:

```gherkin
Feature: LR(1) Error Recovery
  Scenario: Recovering from a missing semicolon
    Given a grammar with rule "statement -> expression ';'"
    And the input "42"
    When I parse the input
    Then the result should contain an error at line 1, column 3
    And the resulting AST should be "Statement(Number(42))"
```

### 2. Implementation Contracts

Each major component in Adze (the table generator, the GLR engine, the incremental re-user) has a corresponding **Contract Trait**. These traits formalize the interface between the governance layer and the implementation.

### 3. Automated Verification

During CI, the governance layer runs these feature specifications against EVERY supported backend.
- If the pure-Rust backend behaves differently than the reference Tree-sitter behavior for a given scenario, the build fails.
- This ensures **functional parity** across all implementation paths.

## Benefits of BDD Governance

### Confidence in Refactoring
When we rewritten the core lexer logic to support the pure-Rust backend, we had 100% confidence because the existing governance scenarios verified that all edge cases (like multi-byte UTF-8 handling) remained correct.

### Living Documentation
The feature files serve as the "ground truth" for how the parser is supposed to behave. Unlike comments, these specifications are guaranteed to be accurate because they are executed as tests.

### Traceability
Every bug fix in Adze should be accompanied by a new BDD scenario. This ensures that the bug never regresses and provides a clear record of why specific logic exists in the runtime.

## How to Contribute to Governance

If you find a case where Adze behaves incorrectly:
1. Identify the relevant feature area in `crates/bdd-*`.
2. Add a new `.feature` file or scenario that reproduces the issue.
3. Run the governance tests: `cargo test -p adze-bdd-governance-core`.
4. Fix the bug until the scenario passes.

By following this process, you help ensure that Adze remains the most reliable and safest parser generator for the Rust ecosystem.
