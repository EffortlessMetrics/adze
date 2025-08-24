#!/usr/bin/env python3
"""
Detect (and optionally fix) half-commented debug blocks such as:

// eprintln!(
//   "msg: {}",
//   x
# MISSING: // );
<real code>

- Only considers blocks that start with '// eprintln!(', '// println!(' or '// dbg!('.
- Ignores doc comments (/// and //!) that mention these macros.
- 'check' mode: exit non-zero on any unterminated block.
- '--fix' mode: inserts a commented ');' with correct indentation.
"""

from __future__ import annotations
import sys, re, pathlib, argparse, os, subprocess

def _repo_root() -> pathlib.Path:
    return pathlib.Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())

# Treat only *line comments*, not doc comments:
#   // eprintln!(...)  ✅
#   /// eprintln!(...) ❌ (docs)
#   //! eprintln!(...) ❌ (inner docs)
OPEN_EPRINTLN = re.compile(r'^\s*//(?!/|!)\s*eprintln!\s*\(')
OPEN_PRINTLN = re.compile(r'^\s*//(?!/|!)\s*println!\s*\(')
OPEN_DBG = re.compile(r'^\s*//(?!/|!)\s*dbg!\s*\(')
OPEN_PATTERNS = [OPEN_EPRINTLN, OPEN_PRINTLN, OPEN_DBG]

def is_open(line: str) -> bool:
    return any(p.match(line) for p in OPEN_PATTERNS)
COMMENTED_CLOSE = re.compile(r'^\s*//\s*\)\s*;\s*(?://.*)?')  # allow trailing comments

def _one_line_closed(line: str) -> bool:
    # e.g., // eprintln!("x {}", y); or with trailing comment
    return re.search(r'\);\s*(?://.*)?', line) is not None

def is_multiline_open(line: str) -> bool:
    return is_open(line) and not _one_line_closed(line)

def _emit_violation(path: pathlib.Path, ln: int, msg: str) -> None:
    if os.getenv("GITHUB_ACTIONS"):
        # Normalize to repo-relative POSIX path so annotations are clickable
        try:
            repo = subprocess.check_output(
                ["git", "rev-parse", "--show-toplevel"], text=True
            ).strip()
            rel = os.path.relpath(path, repo)
        except Exception:
            rel = str(path)
        rel = rel.replace(os.sep, "/")
        # https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-an-error-message
        print(f"::error file={rel},line={ln},title=Commented debug block::{msg}")
    else:
        print(f"{path}:{ln}: {msg}")

def _detect_newline(text: str) -> str:
    # Preserve original newline style (CRLF on Windows)
    return "\r\n" if "\r\n" in text else "\n"

def _read_worktree(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")

def _read_index(rel_posix: str) -> str:
    # Read staged blob (index) for exact pre-commit semantics.
    # Note: use repo-relative POSIX path: :path
    proc = subprocess.run(
        ["git", "show", f":{rel_posix}"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    return proc.stdout.decode("utf-8", "replace")

def find_violations(path: pathlib.Path, *, prefer_index: bool = False) -> list[tuple[int, str]]:
    text: str
    if prefer_index:
        try:
            # Convert to repo-relative POSIX for the index
            root = _repo_root()
            rel = os.path.relpath(path, root)
            rel_posix = rel.replace(os.sep, "/")
            text = _read_index(rel_posix)
        except Exception:
            # Fall back to worktree if the file isn't staged or any Git issue
            text = _read_worktree(path)
    else:
        text = _read_worktree(path)
    lines = text.splitlines()
    i = 0
    violations: list[tuple[int, str]] = []

    while i < len(lines):
        line = lines[i]
        if is_open(line):
            if _one_line_closed(line):
                i += 1
                continue

            # Multi-line commented block; support (rare) nested opens
            block_start = i + 1  # 1-based for messages
            depth = 1
            i += 1
            while i < len(lines):
                cur = lines[i]
                if not cur.lstrip().startswith("//"):
                    # left comment region without closing all opens
                    violations.append((block_start, "unterminated commented debug block (missing '// );')"))
                    break
                if is_open(cur) and not _one_line_closed(cur):
                    depth += 1
                elif COMMENTED_CLOSE.match(cur):
                    depth -= 1
                    if depth == 0:
                        i += 1
                        break
                i += 1
            else:
                # EOF while still open
                violations.append((block_start, "unterminated commented debug block at EOF (missing '// );')"))
        else:
            i += 1

    return violations

def fix_file(path: pathlib.Path) -> bool:
    text = path.read_text(encoding="utf-8", errors="replace")
    nl = _detect_newline(text)
    lines = text.splitlines()
    i = 0
    changed = False

    while i < len(lines):
        line = lines[i]
        if is_open(line) and not _one_line_closed(line):
            indent = re.match(r'^(\s*)', line).group(1) or ""
            depth = 1
            last_comment_idx = i
            i += 1
            while i < len(lines) and lines[i].lstrip().startswith("//"):
                last_comment_idx = i
                cur = lines[i]
                if is_multiline_open(cur):
                    depth += 1
                elif COMMENTED_CLOSE.match(cur):
                    depth -= 1
                    if depth == 0:
                        break
                i += 1

            if depth > 0:
                # Insert as many closers as needed to balance depth
                for _ in range(depth):
                    lines.insert(last_comment_idx + 1, f"{indent}// );")
                    last_comment_idx += 1
                changed = True
                i = last_comment_idx + 1
            else:
                i += 1
        else:
            i += 1

    if changed:
        path.write_text(nl.join(lines) + nl, encoding="utf-8")
    return changed

def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fix", action="store_true", help="Auto-insert missing '// );' after commented debug blocks")
    ap.add_argument("--since", metavar="REV", help="Only check files changed since REV (uses Git)")
    ap.add_argument("--changed-only", action="store_true", help="Only check staged .rs files in the index (uses Git)")
    ap.add_argument("files", nargs="*", help="Files to check (defaults to Git-tracked *.rs, else runtime/src/**/*.rs)")
    args = ap.parse_args(argv)

    files: list[pathlib.Path]
    if args.files:
        files = [pathlib.Path(f) for f in args.files if f.endswith(".rs")]
    else:
        # Prefer Git-backed discovery for speed and accuracy
        try:
            if args.changed_only:
                out = subprocess.check_output(
                    ["git", "diff", "--name-only", "--cached", "--diff-filter=ACMR", "--", "*.rs"], text=True
                )
            elif args.since:
                out = subprocess.check_output(
                    ["git", "diff", "--name-only", f"{args.since}...HEAD", "--", "*.rs"],
                    text=True,
                )
            else:
                out = subprocess.check_output(["git", "ls-files", "*.rs"], text=True)
            files = [pathlib.Path(p) for p in out.splitlines() if p.strip()]
        except Exception:
            files = [p for p in pathlib.Path("runtime/src").rglob("*.rs")]

    if args.fix:
        changed_any = False
        for p in files:
            if p.exists() and fix_file(p):  # fix applies to working tree only
                print(f"fixed: {p}")
                changed_any = True

        remaining = []
        for p in files:
            for ln, msg in find_violations(p):
                remaining.append((p, ln, msg))

        if remaining:
            print("❌ Still found unterminated blocks after --fix:")
            for p, ln, msg in remaining:
                _emit_violation(p, ln, msg)
            return 1

        if changed_any:
            print("✅ Auto-fixes applied, no remaining unterminated debug blocks.")
        else:
            print("✅ No fixes required.")
        return 0

    violations = []
    for p in files:
        if not p.exists():
            continue
        for ln, msg in find_violations(p, prefer_index=args.changed_only):
            violations.append((p, ln, msg))

    if violations:
        for p, ln, msg in violations:
            _emit_violation(p, ln, msg)
        return 1
    return 0

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))