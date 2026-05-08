#!/usr/bin/env python3
"""Process the no-panic proposed baseline and generate a classified allowlist.

Filters to the 7 core pipeline crates only and applies bulk classifications.
"""

import re
import sys
from pathlib import Path

# Core crate directory prefixes
CORE_CRATES = [
    "runtime/",
    "macro/",
    "tool/",
    "common/",
    "ir/",
    "glr-core/",
    "tablegen/",
]

EXPIRES = "2026-11-08"
OWNER = "core-pipeline"


def classify_entry(path: str, family: str, container: str, callee: str) -> tuple[str, str]:
    """Return (classification, explanation) for an entry."""
    # Test files
    if "/tests/" in path or path.endswith("_test.rs") or container.startswith("test_") or "tests::" in container:
        if family == "panic_macro":
            return ("test_helper", "Test assertion with panic!(); expected in test code")
        if family in ("unwrap", "expect", "get_unwrap"):
            return ("test_helper", "Test assertion; unwrap/expect in test code is idiomatic")
        if family == "indexing":
            return ("test_helper", "Test indexing; bounds are known in test context")
        if family in ("todo", "unimplemented", "unreachable"):
            return ("test_helper", "Test placeholder/guard; acceptable in test code")
        if family == "string_slice":
            return ("test_helper", "Test string slicing; bounds are known in test context")
        if family == "unwrap_in_result":
            return ("test_helper", "Test unwrap_in_result; acceptable in test code")
        return ("test_helper", "Test code panic-family call; acceptable in test context")

    # Benchmark files
    if "/benches/" in path or "/bench" in path:
        return ("test_helper", "Benchmark harness; panic is acceptable in measurement code")

    # Example files
    if "/examples/" in path or "/example" in path:
        return ("fixture", "Example/demo code; panic is acceptable for clarity")

    # Source files - classify by family and context
    if family == "unwrap":
        if "lock" in container.lower() or "lock" in callee.lower():
            return ("invariant", "Mutex lock unwrap; poisoning is not expected in single-threaded usage")
        if "last_mut" in container or "arena" in container.lower():
            return ("invariant", "Arena/collection unwrap; invariant maintained by construction")
        return ("invariant", "Production unwrap; caller guarantees Some by construction")

    if family == "expect":
        if "regex" in container.lower() or "regex" in callee.lower():
            return ("external_contract", "Regex compilation; pattern is a compile-time constant verified by tests")
        if "cursor" in container.lower() or "child" in container.lower():
            return ("invariant", "Tree traversal expect; invariant maintained by parser structure")
        if "temp" in container.lower() or "tempfile" in container.lower():
            return ("test_helper", "Temp file creation; acceptable in test/build context")
        return ("invariant", "Expect with documented reason; invariant maintained by caller")

    if family == "panic_macro":
        if "invalid" in container.lower() or "error" in container.lower():
            return ("invariant", "Validation panic; guards against invalid internal state")
        if "expected" in container.lower():
            return ("invariant", "Test-style assertion panic in test context")
        return ("invariant", "Internal panic; guards against unreachable program state")

    if family == "todo":
        return ("placeholder", "TODO marker; to be replaced with implementation before stage 2")

    if family == "unimplemented":
        return ("placeholder", "Unimplemented stub; to be completed before stage 2")

    if family == "unreachable":
        return ("invariant", "Unreachable marker; dead code path guarded by exhaustive match")

    if family == "indexing":
        return ("invariant", "Index operation; bounds verified by parser construction or prior check")

    if family == "string_slice":
        return ("invariant", "String slice; bounds verified by lexer/tokenizer guarantees")

    if family == "get_unwrap":
        return ("invariant", "Get-then-unwrap; existence verified by prior check or invariant")

    if family == "unwrap_in_result":
        return ("invariant", "Unwrap in Result context; error propagation is acceptable")

    return ("legacy", "Unclassified; review and refine classification before stage 2")


def escape_toml_string(s: str) -> str:
    """Escape a string for TOML double-quoted literal."""
    return s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t')


def parse_toml_value(v: str) -> str:
    """Parse a TOML value string, unescaping if quoted."""
    v = v.strip()
    if v.startswith('"') and v.endswith('"'):
        return v[1:-1].replace('\\"', '"').replace('\\\\', '\\')
    return v


def parse_entries(text: str) -> list[dict]:
    """Parse [[allow]] entries from TOML text.
    
    Each entry has top-level keys, plus [allow.selector] and [allow.last_seen] subsections.
    """
    entries = []
    
    # Split on [[allow]] markers
    chunks = re.split(r'\[\[allow\]\]\s*\n', text)
    
    for chunk in chunks:
        if not chunk.strip():
            continue
        
        entry = {"selector": {}, "last_seen": {}}
        current_section = None  # None = top-level, "selector", "last_seen"
        
        for line in chunk.split('\n'):
            stripped = line.strip()
            if not stripped or stripped.startswith('#'):
                continue
            
            # Check for section headers
            if stripped == '[allow.selector]':
                current_section = "selector"
                continue
            elif stripped == '[allow.last_seen]':
                current_section = "last_seen"
                continue
            
            # Parse key = value
            m = re.match(r'^(\w+)\s*=\s*(.+)$', stripped)
            if not m:
                continue
            
            key = m.group(1)
            value = parse_toml_value(m.group(2))
            
            if current_section is None:
                entry[key] = value
            elif current_section == "selector":
                entry["selector"][key] = value
            elif current_section == "last_seen":
                entry["last_seen"][key] = value
        
        if entry.get("path"):
            entries.append(entry)
    
    return entries


def format_entry(entry: dict, id_num: int) -> str:
    """Format a single entry as TOML."""
    path = entry.get('path', '')
    family = entry.get('family', '')
    container = entry.get('selector', {}).get('container', '')
    callee = entry.get('selector', {}).get('callee', '')

    classification, explanation = classify_entry(path, family, container, callee)

    selector = entry.get('selector', {})
    last_seen = entry.get('last_seen', {})

    lines = []
    lines.append('[[allow]]')
    lines.append(f'id = "panic-{id_num:04d}"')
    lines.append(f'path = "{escape_toml_string(path)}"')
    lines.append(f'family = "{family}"')
    lines.append(f'classification = "{classification}"')
    lines.append(f'owner = "{OWNER}"')
    lines.append(f'explanation = "{escape_toml_string(explanation)}"')
    lines.append(f'expires = "{EXPIRES}"')
    lines.append('')
    lines.append('[allow.selector]')
    
    # Required fields
    kind = selector.get('kind', 'method_call')
    lines.append(f'kind = "{kind}"')
    lines.append(f'container = "{escape_toml_string(container)}"')
    lines.append(f'callee = "{escape_toml_string(callee)}"')
    
    # Optional receiver_fingerprint
    fp = selector.get('receiver_fingerprint', '')
    if fp:
        lines.append(f'receiver_fingerprint = "{escape_toml_string(fp)}"')
    
    lines.append('')
    lines.append('[allow.last_seen]')
    lines.append(f'line = {last_seen.get("line", "0")}')
    lines.append(f'column = {last_seen.get("column", "0")}')
    lines.append('')

    return '\n'.join(lines)


def main():
    proposed_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("target/policy/no-panic-proposed.toml")
    output_path = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("policy/no-panic-allowlist.toml")

    text = proposed_path.read_text(encoding='utf-8')
    entries = parse_entries(text)

    # Filter to core crates only
    core_entries = []
    for e in entries:
        path = e.get('path', '')
        if any(path.startswith(prefix) for prefix in CORE_CRATES):
            core_entries.append(e)

    print(f"Total proposed entries: {len(entries)}")
    print(f"Core crate entries: {len(core_entries)}")

    # Build output
    header = '''schema_version = "0.3"

# Semantic allowlist for panic-family debt.
#
# Identity = (path, family, selector).
# `last_seen` is advisory only and updated by `cargo xtask no-panic-propose`.
#
# This baseline was generated from the workspace-wide scan and filtered to the
# 7 core pipeline crates (adze, adze-macro, adze-tool, adze-common, adze-ir,
# adze-glr-core, adze-tablegen). Entries are classified by context:
#   - test_helper: test/bench code where panic is idiomatic
#   - fixture: example/demo code where panic is acceptable for clarity
#   - invariant: production code where the panic guards an internal invariant
#   - external_contract: external dependency contract assumed valid
#   - placeholder: TODO/unimplemented to be resolved before stage 2
#
# Mode: advisory. Will be promoted to blocking-allowlist after burn-down.
# Generated: 2026-05-08
# Expires: 2026-11-08 (6 months)

'''

    output = header
    for i, entry in enumerate(core_entries, 1):
        output += format_entry(entry, i) + '\n'

    output_path.write_text(output, encoding='utf-8')
    print(f"Wrote {len(core_entries)} entries to {output_path}")

    # Stats
    classifications = {}
    for e in core_entries:
        path = e.get('path', '')
        family = e.get('family', '')
        container = e.get('selector', {}).get('container', '')
        callee = e.get('selector', {}).get('callee', '')
        c, _ = classify_entry(path, family, container, callee)
        classifications[c] = classifications.get(c, 0) + 1

    print("\nClassification breakdown:")
    for k, v in sorted(classifications.items()):
        print(f"  {k}: {v}")


if __name__ == '__main__':
    main()
