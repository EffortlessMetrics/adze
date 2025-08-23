#!/usr/bin/env python3
"""
Check for half-commented debug blocks where eprintln!/println! is commented
but the closing ); is not. Only checks blocks that start with // eprintln!( or // println!(
"""
import sys, re, pathlib

OPEN  = re.compile(r'^\s*//\s*(eprintln|println)!\s*\(')
COMMENTED_CLOSE = re.compile(r'^\s*//\s*\);\s*$')

def check_file(path: pathlib.Path) -> list[tuple[int,str]]:
    violations = []
    lines = path.read_text(encoding='utf-8').splitlines()
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Check if this line starts a commented debug print block
        if OPEN.match(line):
            # Skip one-liners like: // eprintln!("msg");
            if line.rstrip().endswith(');'):
                i += 1
                continue
                
            # Multi-line commented block found
            block_start_line = i + 1  # Line numbers are 1-based
            i += 1
            
            # Look for the closing of this block
            found_closing = False
            while i < len(lines):
                line = lines[i]
                
                # If we hit a non-commented line, the block ended without proper closing
                if not line.lstrip().startswith('//'):
                    violations.append((block_start_line, "unterminated commented debug block (missing '// );')"))
                    break
                    
                # Check if this is the commented closing
                if COMMENTED_CLOSE.match(line):
                    found_closing = True
                    i += 1
                    break
                    
                i += 1
            
            # If we hit EOF without finding a closing
            if i >= len(lines) and not found_closing:
                violations.append((block_start_line, "unterminated commented debug block at EOF (missing '// );')"))
        else:
            i += 1
    
    return violations

def main(files: list[str]) -> int:
    bad = []
    for f in files:
        p = pathlib.Path(f)
        if p.suffix == '.rs' and p.exists():
            v = check_file(p)
            if v:
                bad.append((p, v))
    if bad:
        for p, v in bad:
            for (ln, msg) in v:
                print(f"{p}:{ln}: {msg}")
        return 1
    return 0

if __name__ == '__main__':
    if len(sys.argv) == 1:
        files = [str(p) for p in pathlib.Path('runtime/src').rglob('*.rs')]
    else:
        files = sys.argv[1:]
    sys.exit(main(files))