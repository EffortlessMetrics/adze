//! Panic-family detector and allowlist checker.
//!
//! See `docs/policy/NO_PANIC_POLICY.md` for the policy.
//!
//! # Detection strategy
//!
//! Stage 1 uses regex-based scanning. This is intentionally pragmatic:
//! it produces fast, useful baselines and feeds the proposal flow. It
//! will miss some macros and accept a small number of false positives
//! inside string literals. Stage 3 (after baselines are populated and
//! reviewed) upgrades to a `syn`-based AST walk.
//!
//! # Identity
//!
//! ```text
//! identity = path + family + selector
//! ```
//!
//! Where `selector` is `kind + container + (callee | name | target)`.
//! `last_seen` is a drift hint and is never part of the matching key.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::{CheckOutcome, ensure_reports_dir, workspace_root};

/// Schema version this checker understands.
pub const SCHEMA_VERSION: &str = "0.3";

#[derive(Debug, Deserialize)]
struct AllowlistFile {
    #[serde(default)]
    schema_version: String,
    #[serde(default, rename = "allow")]
    allows: Vec<AllowEntry>,
}

#[expect(
    dead_code,
    reason = "Editor-facing fields (classification, owner, explanation) are deserialized for round-trip and report rendering, even when the matching code does not consume them directly."
)]
#[derive(Debug, Clone, Deserialize)]
struct AllowEntry {
    id: String,
    path: String,
    family: String,
    #[serde(default)]
    classification: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    explanation: Option<String>,
    #[serde(default)]
    expires: Option<String>,
    #[serde(default)]
    retired: Option<bool>,
    selector: Selector,
    #[serde(default)]
    last_seen: Option<LastSeen>,
}

#[expect(
    dead_code,
    reason = "receiver_fingerprint is captured for human review of the proposed allowlist; the matching key today uses kind + container + callee/name/target."
)]
#[derive(Debug, Clone, Deserialize)]
struct Selector {
    kind: String,
    container: String,
    #[serde(default)]
    callee: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    receiver_fingerprint: Option<String>,
    #[serde(default)]
    target_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LastSeen {
    line: usize,
    column: usize,
}

/// One detected panic-family call site.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub path: String,
    pub family: &'static str,
    pub selector_kind: &'static str,
    pub container: String,
    pub callee: Option<String>,
    pub name: Option<String>,
    pub receiver_fingerprint: Option<String>,
    pub target_fingerprint: Option<String>,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

impl Finding {
    /// Identity tuple — must match selector matching exactly.
    fn identity(&self) -> (String, &'static str, &'static str, String, String) {
        let detail = self
            .callee
            .clone()
            .or_else(|| self.name.clone())
            .or_else(|| self.target_fingerprint.clone())
            .unwrap_or_default();
        (
            self.path.clone(),
            self.family,
            self.selector_kind,
            self.container.clone(),
            detail,
        )
    }
}

/// Configuration knobs for [`run`].
#[derive(Debug, Default, Clone)]
pub struct RunOpts {
    /// If true, exit non-zero on any unallowlisted finding or stale
    /// allowlist entry. Default is advisory (always exit 0).
    pub strict: bool,
    /// If true, write `target/policy/reports/no-panic-proposed-allowlist.toml`
    /// in addition to the standard reports.
    pub propose: bool,
}

/// Run the panic-family checker. Returns the high-level outcome.
pub fn run(opts: RunOpts) -> Result<CheckOutcome> {
    let root = workspace_root()?;
    let reports_dir = ensure_reports_dir(&root)?;
    let allowlist_path = root.join("policy/no-panic-allowlist.toml");

    let allowlist = read_allowlist(&allowlist_path)?;
    let findings = scan(&root)?;

    let mut allow_index: BTreeMap<_, &AllowEntry> = BTreeMap::new();
    let mut allow_matched: BTreeSet<usize> = BTreeSet::new();
    for entry in &allowlist.allows {
        allow_index.insert(entry_identity(entry), entry);
    }

    let mut unallowlisted: Vec<&Finding> = Vec::new();
    let mut drift: Vec<DriftHit<'_>> = Vec::new();

    for f in &findings {
        if let Some(entry) = allow_index.get(&f.identity()) {
            // Matched — drift check.
            allow_matched.insert(entry_index(&allowlist.allows, entry));
            if let Some(seen) = &entry.last_seen
                && (seen.line != f.line || seen.column != f.column)
            {
                drift.push(DriftHit {
                    entry,
                    finding: f,
                    seen_line: seen.line,
                    seen_column: seen.column,
                });
            }
        } else {
            unallowlisted.push(f);
        }
    }

    let stale: Vec<&AllowEntry> = allowlist
        .allows
        .iter()
        .enumerate()
        .filter(|(idx, e)| !allow_matched.contains(idx) && !e.retired.unwrap_or(false))
        .map(|(_, e)| e)
        .collect();

    let expired: Vec<&AllowEntry> = allowlist
        .allows
        .iter()
        .filter(|e| e.expires.as_deref().is_some_and(is_expired))
        .collect();

    let report_md = reports_dir.join("no-panic.md");
    let report_json = reports_dir.join("no-panic.json");
    write_md_report(
        &report_md,
        &findings,
        &unallowlisted,
        &stale,
        &expired,
        &drift,
    )?;
    write_json_report(&report_json, &findings, &unallowlisted, &stale, &expired)?;

    if opts.propose {
        let proposed = reports_dir.join("no-panic-proposed-allowlist.toml");
        write_proposed_allowlist(&proposed, &findings)?;
        eprintln!(
            "[policy:no-panic] proposed allowlist written to {}",
            proposed.display()
        );
    }

    let total_problems = unallowlisted.len() + stale.len() + expired.len();
    let outcome = CheckOutcome {
        label: "no-panic",
        findings: total_problems,
        report_md,
    };
    outcome.print_summary();

    if opts.strict && total_problems > 0 {
        anyhow::bail!(
            "no-panic policy: {} unallowlisted, {} stale, {} expired",
            unallowlisted.len(),
            stale.len(),
            expired.len()
        );
    }
    Ok(outcome)
}

struct DriftHit<'a> {
    entry: &'a AllowEntry,
    finding: &'a Finding,
    seen_line: usize,
    seen_column: usize,
}

fn entry_index(all: &[AllowEntry], target: &AllowEntry) -> usize {
    // Pointer-identity comparison through ID + path + family.
    all.iter()
        .position(|e| e.id == target.id && e.path == target.path && e.family == target.family)
        .expect("entry must come from `all`")
}

fn entry_identity(e: &AllowEntry) -> (String, &'static str, &'static str, String, String) {
    let family = canonical_family(&e.family);
    let kind = canonical_kind(&e.selector.kind);
    let detail = e
        .selector
        .callee
        .clone()
        .or_else(|| e.selector.name.clone())
        .or_else(|| e.selector.target_fingerprint.clone())
        .unwrap_or_default();
    (
        e.path.clone(),
        family,
        kind,
        e.selector.container.clone(),
        detail,
    )
}

fn canonical_family(s: &str) -> &'static str {
    match s {
        "unwrap" => "unwrap",
        "expect" => "expect",
        "panic_macro" | "panic" => "panic_macro",
        "todo" => "todo",
        "unimplemented" => "unimplemented",
        "unreachable" => "unreachable",
        "indexing" => "indexing",
        "string_slice" => "string_slice",
        "get_unwrap" => "get_unwrap",
        "unchecked_time_subtraction" => "unchecked_time_subtraction",
        _ => "unknown",
    }
}

fn canonical_kind(s: &str) -> &'static str {
    match s {
        "method_call" => "method_call",
        "macro_invoke" => "macro_invoke",
        "index_expr" => "index_expr",
        _ => "unknown",
    }
}

fn read_allowlist(path: &Path) -> Result<AllowlistFile> {
    if !path.exists() {
        return Ok(AllowlistFile {
            schema_version: SCHEMA_VERSION.to_string(),
            allows: vec![],
        });
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let file: AllowlistFile =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if !file.schema_version.is_empty() && file.schema_version != SCHEMA_VERSION {
        anyhow::bail!(
            "{} has schema_version {} but checker expects {}",
            path.display(),
            file.schema_version,
            SCHEMA_VERSION
        );
    }
    Ok(file)
}

fn is_expired(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d < chrono::Local::now().date_naive())
        .unwrap_or(false)
}

// --- Scanning ---------------------------------------------------------------

fn scan(root: &Path) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_skipped(e.path()))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        scan_text(&rel, &text, &mut out);
    }
    out.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
    });
    Ok(out)
}

fn is_skipped(path: &Path) -> bool {
    let s = path.to_string_lossy();
    // Skip generated / vendored / non-source trees so we focus on first-party code.
    s.contains("/target/")
        || s.contains("/.git/")
        || s.contains("/book/book/")
        || s.ends_with("Cargo.lock")
}

/// Strip `/* ... */` block-comment contents while preserving byte offsets,
/// so line/column reporting remains accurate. Nested block comments are
/// supported (Rust permits them).
fn mask_block_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut depth: u32 = 0;
    let mut in_str = false;
    let mut escape = false;
    while i < bytes.len() {
        let c = bytes[i];
        if depth > 0 {
            // Inside block comment.
            if i + 1 < bytes.len() && c == b'/' && bytes[i + 1] == b'*' {
                depth += 1;
                out.push(b' ');
                out.push(b' ');
                i += 2;
                continue;
            }
            if i + 1 < bytes.len() && c == b'*' && bytes[i + 1] == b'/' {
                depth -= 1;
                out.push(b' ');
                out.push(b' ');
                i += 2;
                continue;
            }
            // Preserve newlines so line numbers don't shift.
            if c == b'\n' {
                out.push(b'\n');
            } else {
                out.push(b' ');
            }
            i += 1;
            continue;
        }
        if in_str {
            if escape {
                escape = false;
            } else if c == b'\\' {
                escape = true;
            } else if c == b'"' {
                in_str = false;
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && c == b'/' && bytes[i + 1] == b'*' {
            depth = 1;
            out.push(b' ');
            out.push(b' ');
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    // SAFETY: We only ever emit ASCII bytes from the original or replace
    // non-newline bytes inside masked regions with single ASCII spaces. The
    // bytes outside masked regions are copied verbatim, preserving any
    // multibyte UTF-8 sequences. The result is therefore valid UTF-8.
    String::from_utf8(out).unwrap_or_else(|_| text.to_string())
}

/// Detect panic-family call sites in `text` and append findings to `out`.
///
/// Public for unit testing.
pub fn scan_text(rel_path: &str, text: &str, out: &mut Vec<Finding>) {
    let text = &mask_block_comments(text);
    // Regex catalog. These are conservative — we err toward false positives
    // because the proposal flow lets owners turn them into receipts. See
    // `docs/policy/NO_PANIC_POLICY.md` for the policy this implements.
    static PATTERNS: &[(&str, &str, &str)] = &[
        // (family, selector_kind, regex)
        ("unwrap", "method_call", r"\.unwrap\s*\(\s*\)"),
        ("expect", "method_call", r"\.expect\s*\("),
        ("panic_macro", "macro_invoke", r"\bpanic\s*!\s*\("),
        ("todo", "macro_invoke", r"\btodo\s*!\s*\("),
        ("unimplemented", "macro_invoke", r"\bunimplemented\s*!\s*\("),
        ("unreachable", "macro_invoke", r"\bunreachable\s*!\s*\("),
        (
            "get_unwrap",
            "method_call",
            r"\.get\s*\([^)]*\)\s*\.unwrap\s*\(\s*\)",
        ),
    ];

    let compiled: Vec<(&'static str, &'static str, Regex)> = PATTERNS
        .iter()
        .map(|(f, k, p)| (*f, *k, Regex::new(p).expect("hard-coded pattern compiles")))
        .collect();

    let line_starts = compute_line_starts(text);

    for (family, kind, re) in &compiled {
        for m in re.find_iter(text) {
            let byte = m.start();
            if is_in_line_comment_or_string_or_attr(text, byte) {
                continue;
            }
            // get_unwrap is a superset of unwrap; we keep both detections,
            // but prefer get_unwrap when reporting (handled below).
            let (line, column) = byte_to_line_col(byte, &line_starts, text);
            let container = enclosing_fn_name(text, byte).unwrap_or_else(|| "<top-level>".into());
            let snippet = snippet_at(text, byte);
            let mut callee = None;
            let mut name = None;
            let mut receiver_fingerprint = None;
            match *kind {
                "method_call" => {
                    callee = Some(method_name(family));
                    receiver_fingerprint = Some(receiver_fingerprint_at(text, byte));
                }
                "macro_invoke" => {
                    name = Some((*family).to_string());
                }
                _ => {}
            }
            out.push(Finding {
                path: rel_path.to_string(),
                family,
                selector_kind: kind,
                container,
                callee,
                name,
                receiver_fingerprint,
                target_fingerprint: None,
                line,
                column,
                snippet,
            });
        }
    }

    // Deduplicate: when both `unwrap` and `get_unwrap` match the same byte,
    // keep only `get_unwrap`.
    let mut keep = vec![true; out.len()];
    for (i, a) in out.iter().enumerate() {
        if a.path != rel_path {
            continue;
        }
        if a.family == "unwrap" {
            for b in out.iter() {
                if b.family == "get_unwrap"
                    && b.path == a.path
                    && b.line == a.line
                    && b.column.saturating_sub(a.column) <= 16
                {
                    keep[i] = false;
                    break;
                }
            }
        }
    }
    let kept: Vec<Finding> = out
        .drain(..)
        .zip(keep)
        .filter_map(|(f, k)| k.then_some(f))
        .collect();
    out.extend(kept);
}

fn method_name(family: &str) -> String {
    match family {
        "unwrap" => "unwrap".into(),
        "expect" => "expect".into(),
        "get_unwrap" => "unwrap".into(),
        _ => family.to_string(),
    }
}

fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn byte_to_line_col(byte: usize, line_starts: &[usize], text: &str) -> (usize, usize) {
    let line_idx = match line_starts.binary_search(&byte) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let line_start = line_starts[line_idx];
    let col = text[line_start..byte].chars().count() + 1;
    (line_idx + 1, col)
}

fn snippet_at(text: &str, byte: usize) -> String {
    let line_start = text[..byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[byte..]
        .find('\n')
        .map(|i| byte + i)
        .unwrap_or(text.len());
    let snippet = &text[line_start..line_end];
    snippet.trim().chars().take(160).collect()
}

/// Walk back from `byte` to find the enclosing `fn name`. Best-effort —
/// good enough for selector identity but not for AST refactors.
fn enclosing_fn_name(text: &str, byte: usize) -> Option<String> {
    let prefix = &text[..byte];
    // Match `fn <name>` and impl method `fn <name>` patterns. We pick the
    // last occurrence before `byte`.
    let re = Regex::new(r"(?m)\bfn\s+([A-Za-z_][A-Za-z0-9_]*)").ok()?;
    re.captures_iter(prefix)
        .last()
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn receiver_fingerprint_at(text: &str, byte: usize) -> String {
    // Walk back from `byte` over the receiver expression, balancing parens.
    let bytes = text.as_bytes();
    if byte == 0 {
        return String::new();
    }
    let mut i = byte;
    let mut depth: i32 = 0;
    while i > 0 {
        let c = bytes[i - 1];
        if c == b')' || c == b']' {
            depth += 1;
        } else if c == b'(' || c == b'[' {
            if depth == 0 {
                break;
            }
            depth -= 1;
        } else if depth == 0 && (c == b';' || c == b'{' || c == b'\n' || c == b',' || c == b'=') {
            break;
        }
        i -= 1;
    }
    let raw = &text[i..byte];
    raw.trim().chars().take(80).collect()
}

fn is_in_line_comment_or_string_or_attr(text: &str, byte: usize) -> bool {
    // Line comment: scan back to last \n, see if `//` appears before byte.
    let line_start = text[..byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_so_far = &text[line_start..byte];
    if let Some(idx) = line_so_far.find("//") {
        // If `//` is not inside a string literal, the rest of the line is a comment.
        if !is_inside_string(line_so_far, idx) {
            return true;
        }
    }
    // Inside a string literal on this line?
    if is_inside_string(line_so_far, line_so_far.len()) {
        return true;
    }
    // Inside an attribute like #[expect(...)] or #[allow(...)]?
    // Crude check: the line trims to start with `#[`.
    let trimmed = line_so_far.trim_start();
    if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
        return true;
    }
    false
}

fn is_inside_string(slice: &str, byte: usize) -> bool {
    let mut in_str = false;
    let mut escape = false;
    let bytes = slice.as_bytes();
    let upto = byte.min(bytes.len());
    let mut i = 0;
    while i < upto {
        let c = bytes[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if c == b'\\' && in_str {
            escape = true;
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = !in_str;
        }
        i += 1;
    }
    in_str
}

// --- Reports ----------------------------------------------------------------

fn write_md_report(
    path: &Path,
    findings: &[Finding],
    unallowlisted: &[&Finding],
    stale: &[&AllowEntry],
    expired: &[&AllowEntry],
    drift: &[DriftHit<'_>],
) -> Result<()> {
    let mut s = String::new();
    s.push_str("# No-panic policy report\n\n");
    s.push_str(&format!(
        "- Scanned findings: {}\n- Unallowlisted: {}\n- Stale entries: {}\n- Expired entries: {}\n- Drift hits: {}\n\n",
        findings.len(),
        unallowlisted.len(),
        stale.len(),
        expired.len(),
        drift.len(),
    ));

    s.push_str("## Family breakdown\n\n");
    let mut by_family: BTreeMap<&str, usize> = BTreeMap::new();
    for f in findings {
        *by_family.entry(f.family).or_insert(0) += 1;
    }
    for (k, v) in &by_family {
        s.push_str(&format!("- `{}` × {}\n", k, v));
    }

    if !unallowlisted.is_empty() {
        s.push_str("\n## Unallowlisted findings (top 50)\n\n");
        for f in unallowlisted.iter().take(50) {
            s.push_str(&format!(
                "- `{}:{}:{}` `{}` in `{}` — `{}`\n",
                f.path, f.line, f.column, f.family, f.container, f.snippet
            ));
        }
        if unallowlisted.len() > 50 {
            s.push_str(&format!("\n…and {} more.\n", unallowlisted.len() - 50));
        }
    }

    if !stale.is_empty() {
        s.push_str("\n## Stale allowlist entries\n\n");
        for e in stale {
            s.push_str(&format!(
                "- `{}` (`{}`) in `{}` — selector matched nothing in tree.\n",
                e.id, e.family, e.path
            ));
        }
    }

    if !expired.is_empty() {
        s.push_str("\n## Expired allowlist entries\n\n");
        for e in expired {
            s.push_str(&format!(
                "- `{}` expired {} (`{}` in `{}`)\n",
                e.id,
                e.expires.as_deref().unwrap_or("?"),
                e.family,
                e.path
            ));
        }
    }

    if !drift.is_empty() {
        s.push_str("\n## Drift hits (advisory)\n\n");
        for d in drift {
            s.push_str(&format!(
                "- `{}` last_seen {}:{} but now at {}:{}\n",
                d.entry.id, d.seen_line, d.seen_column, d.finding.line, d.finding.column
            ));
        }
    }

    fs::write(path, s).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_json_report(
    path: &Path,
    findings: &[Finding],
    unallowlisted: &[&Finding],
    stale: &[&AllowEntry],
    expired: &[&AllowEntry],
) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "totals": {
            "findings": findings.len(),
            "unallowlisted": unallowlisted.len(),
            "stale": stale.len(),
            "expired": expired.len(),
        },
        "findings": findings,
        "stale": stale.iter().map(|e| serde_json::json!({
            "id": e.id, "path": e.path, "family": e.family,
        })).collect::<Vec<_>>(),
        "expired": expired.iter().map(|e| serde_json::json!({
            "id": e.id, "expires": e.expires,
        })).collect::<Vec<_>>(),
    });
    fs::write(path, serde_json::to_string_pretty(&payload)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_proposed_allowlist(path: &Path, findings: &[Finding]) -> Result<()> {
    let mut s = String::new();
    s.push_str("# Generated by `cargo xtask no-panic propose`. Review and copy.\n");
    s.push_str(&format!("schema_version = \"{}\"\n\n", SCHEMA_VERSION));
    let mut counter = 1u32;
    for f in findings {
        s.push_str(&format!("[[allow]]\nid = \"panic-{:04}\"\n", counter));
        s.push_str(&format!("path = \"{}\"\n", f.path));
        s.push_str(&format!("family = \"{}\"\n", f.family));
        s.push_str("classification = \"TODO\"\n");
        s.push_str("owner = \"TODO\"\n");
        s.push_str("explanation = \"TODO — explain why this is the right shape.\"\n");
        s.push_str("expires = \"2027-01-01\"\n\n");
        s.push_str("[allow.selector]\n");
        s.push_str(&format!("kind = \"{}\"\n", f.selector_kind));
        s.push_str(&format!("container = \"{}\"\n", f.container));
        if let Some(c) = &f.callee {
            s.push_str(&format!("callee = \"{}\"\n", escape_toml(c)));
        }
        if let Some(n) = &f.name {
            s.push_str(&format!("name = \"{}\"\n", escape_toml(n)));
        }
        if let Some(r) = &f.receiver_fingerprint {
            s.push_str(&format!("receiver_fingerprint = \"{}\"\n", escape_toml(r)));
        }
        s.push_str("\n[allow.last_seen]\n");
        s.push_str(&format!("line = {}\ncolumn = {}\n\n", f.line, f.column));
        counter += 1;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).ok();
    fs::write(path, s).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unwrap_in_method_position() {
        let mut out = Vec::new();
        scan_text(
            "x.rs",
            "fn run() -> i32 { let v = std::env::var(\"X\").unwrap(); v.len() as i32 }",
            &mut out,
        );
        assert!(out.iter().any(|f| f.family == "unwrap"), "{:?}", out);
        assert!(out.iter().all(|f| f.container == "run"));
    }

    #[test]
    fn ignores_unwrap_in_line_comment() {
        let mut out = Vec::new();
        scan_text(
            "x.rs",
            "fn run() { /* note: do not call .unwrap() */ // .unwrap()\n }",
            &mut out,
        );
        assert!(out.is_empty(), "{:?}", out);
    }

    #[test]
    fn detects_panic_macro() {
        let mut out = Vec::new();
        scan_text("x.rs", "fn boom() { panic!(\"nope\"); }", &mut out);
        assert!(out.iter().any(|f| f.family == "panic_macro"));
    }

    #[test]
    fn skips_attribute_lines() {
        let mut out = Vec::new();
        scan_text(
            "x.rs",
            "#[allow(clippy::unwrap_used)]\nfn ok() {}",
            &mut out,
        );
        assert!(out.is_empty());
    }
}
