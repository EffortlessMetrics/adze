#!/usr/bin/env python3
"""
Fail if a commented eprintln!/println! block has an uncommented closing ');'.
Idempotent: no writes, just exits non-zero on violation.
"""
import sys, re, pathlib

OPEN  = re.compile(r'^\s*//\s*(eprintln|println)!\s*\(')
# Only match closing ); that are not preceded by non-comment content
CLOSE = re.compile(r'^\s*\);\s*$')

def check_file(path: pathlib.Path) -> list[tuple[int,str]]:
    violations = []
    lines = path.read_text(encoding='utf-8').splitlines()
    in_block = False
    for i, line in enumerate(lines, 1):
        if not in_block and OPEN.match(line):
            # If opener has inline close on same line, it's fine.
            in_block = not line.rstrip().endswith(');')
        elif in_block:
            if CLOSE.match(line) and not line.lstrip().startswith('//'):
                violations.append((i, "closing ');' not commented"))
                in_block = False
            elif CLOSE.match(line):
                in_block = False
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