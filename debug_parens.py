#!/usr/bin/env python3
"""Debug script to understand the parentheses parsing issue"""

def parse_parens(s):
    """Simple parentheses parser to understand the expected behavior"""
    stack = []
    for i, ch in enumerate(s):
        if ch == '(':
            stack.append(('LPAREN', i))
        elif ch == ')':
            if not stack:
                print(f"Error: unmatched ) at position {i}")
                return None
            stack.pop()
        elif ch.isdigit():
            print(f"Number {ch} at position {i}, stack depth: {len(stack)}")
    
    if stack:
        print(f"Error: {len(stack)} unclosed parentheses")
        return None
    
    return True

# Test cases
test_cases = [
    "1",
    "(1)",
    "((1))",
    "(((1)))",
    "((((1))))",
]

for test in test_cases:
    print(f"\nParsing: {test}")
    result = parse_parens(test)
    if result:
        print("✓ Valid")