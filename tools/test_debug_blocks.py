#!/usr/bin/env python3
"""
Self-test for the debug block validator.
Tests that it correctly identifies unterminated blocks and doesn't flag legitimate code.
"""

import tempfile
import pathlib
import sys
import os

# Add tools directory to path to import check_debug_blocks
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import check_debug_blocks


def write_test_file(content: str) -> pathlib.Path:
    """Write content to a temporary .rs file"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.rs', delete=False) as f:
        f.write(content)
        return pathlib.Path(f.name)


def test_valid_cases():
    """Test cases that should NOT be flagged"""
    
    valid_cases = [
        # One-liner
        """// eprintln!("debug msg");""",
        
        # Properly closed multi-line
        """// eprintln!(
//   "msg: {}",
//   x
// );""",
        
        # With trailing comment
        """// eprintln!(
//   "msg"
// ); // done debugging""",
        
        # Nested but balanced
        """// eprintln!(
//   "outer: {}", 
//   // println!(
//   //   "inner"
//   // );
//   value
// );""",
        
        # Not debug prints - should be ignored
        """bail!("error message");""",
        """panic!("unexpected state");""",
        """debug_assert!(condition, "failed");""",
        """// Some other comment
eprintln!("active debug");""",
    ]
    
    for i, content in enumerate(valid_cases):
        path = write_test_file(content)
        try:
            violations = check_debug_blocks.find_violations(path)
            if violations:
                print(f"❌ Valid case {i+1} incorrectly flagged:")
                print(f"   Content: {content[:50]}...")
                print(f"   Violations: {violations}")
                return False
        finally:
            path.unlink()
    
    return True


def test_invalid_cases():
    """Test cases that SHOULD be flagged"""
    
    invalid_cases = [
        # Missing closer
        ("""// eprintln!(
//   "msg: {}",
//   x
let y = 42;""", "unterminated commented debug block"),
        
        # EOF without closer
        ("""// eprintln!(
//   "msg"
//   no closer here""", "unterminated commented debug block at EOF"),
        
        # Nested with missing closer
        ("""// eprintln!(
//   "outer",
//   // println!(
//   //   "inner"
//   x
// missing one );""", "unterminated commented debug block"),
    ]
    
    for i, (content, expected_msg) in enumerate(invalid_cases):
        path = write_test_file(content)
        try:
            violations = check_debug_blocks.find_violations(path)
            if not violations:
                print(f"❌ Invalid case {i+1} not flagged:")
                print(f"   Content: {content[:50]}...")
                return False
            if expected_msg not in violations[0][1]:
                print(f"❌ Invalid case {i+1} wrong message:")
                print(f"   Expected: ...{expected_msg}...")
                print(f"   Got: {violations[0][1]}")
                return False
        finally:
            path.unlink()
    
    return True


def test_fix_mode():
    """Test that --fix correctly adds missing closers"""
    
    broken = """// eprintln!(
//   "needs fix"
let x = 1;"""
    
    expected_fixed = """// eprintln!(
//   "needs fix"
// );
let x = 1;"""
    
    path = write_test_file(broken)
    try:
        # Apply fix
        changed = check_debug_blocks.fix_file(path)
        if not changed:
            print("❌ Fix mode didn't detect broken block")
            return False
        
        # Read fixed content
        fixed = path.read_text()
        if fixed.strip() != expected_fixed.strip():
            print("❌ Fix mode produced wrong output:")
            print(f"   Expected:\n{expected_fixed}")
            print(f"   Got:\n{fixed}")
            return False
        
        # Verify no violations remain
        violations = check_debug_blocks.find_violations(path)
        if violations:
            print("❌ Fix mode left violations:")
            print(f"   {violations}")
            return False
            
    finally:
        path.unlink()
    
    return True


def main():
    print("Running debug block validator self-tests...")
    
    tests = [
        ("Valid cases", test_valid_cases),
        ("Invalid cases", test_invalid_cases),
        ("Fix mode", test_fix_mode),
    ]
    
    failed = False
    for name, test_fn in tests:
        print(f"\nTesting {name}...")
        if test_fn():
            print(f"✅ {name} passed")
        else:
            print(f"❌ {name} failed")
            failed = True
    
    if failed:
        print("\n❌ Some tests failed")
        return 1
    else:
        print("\n✅ All tests passed")
        return 0


if __name__ == "__main__":
    sys.exit(main())