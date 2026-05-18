//! Query execution and overlap handling for syntax highlighting.

use super::types::Highlight;
use crate::parser_v4::ParseNode;
use crate::query::{Query, QueryCursor};
use std::collections::HashMap;

/// Highlighter that uses Tree-sitter queries.
pub struct Highlighter {
    /// The highlight query.
    query: Query,
    /// Mapping from capture indices to highlight names.
    highlight_map: HashMap<u32, String>,
}

impl Highlighter {
    /// Create a new highlighter from a query.
    pub fn new(query: Query) -> Self {
        let highlight_map = capture_highlight_map(&query);

        Self {
            query,
            highlight_map,
        }
    }

    /// Highlight a parse tree.
    pub fn highlight(&self, root: &ParseNode) -> Vec<Highlight> {
        let mut highlights = self.collect_highlights(root);
        highlights.sort_by_key(|h| (h.start_byte, h.end_byte));
        remove_overlaps(&mut highlights);
        highlights
    }

    fn collect_highlights(&self, root: &ParseNode) -> Vec<Highlight> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.collect_matches(&self.query, root);
        let mut highlights = Vec::new();

        for query_match in matches {
            for capture in query_match.captures {
                if let Some(highlight_name) = self.highlight_map.get(&capture.index) {
                    highlights.push(Highlight::new(
                        capture.node.start_byte,
                        capture.node.end_byte,
                        highlight_name.clone(),
                    ));
                }
            }
        }

        highlights
    }
}

fn capture_highlight_map(query: &Query) -> HashMap<u32, String> {
    query
        .capture_names
        .iter()
        .map(|(name, &index)| (index, name.clone()))
        .collect()
}

/// Remove overlapping highlights, keeping the more specific ones.
fn remove_overlaps(highlights: &mut Vec<Highlight>) {
    if highlights.is_empty() {
        return;
    }

    let mut result = Vec::new();
    let mut current = highlights[0].clone();

    for highlight in highlights.iter().skip(1) {
        if highlight.start_byte >= current.end_byte {
            result.push(current);
            current = highlight.clone();
        } else if highlight.end_byte <= current.end_byte {
            push_contained_highlight(&mut result, &mut current, highlight, highlights);
        } else {
            push_prefix_if_present(&mut result, &current, highlight.start_byte);
            current = highlight.clone();
        }
    }

    if current.start_byte < current.end_byte {
        result.push(current);
    }

    *highlights = result;
}

fn push_contained_highlight(
    result: &mut Vec<Highlight>,
    current: &mut Highlight,
    highlight: &Highlight,
    highlights: &[Highlight],
) {
    result.push(highlight.clone());
    push_prefix_if_present(result, current, highlight.start_byte);

    if highlight.end_byte < current.end_byte {
        current.start_byte = highlight.end_byte;
    } else {
        *current = highlights[highlights.len() - 1].clone();
    }
}

fn push_prefix_if_present(result: &mut Vec<Highlight>, current: &Highlight, split_byte: usize) {
    if split_byte > current.start_byte {
        result.push(current.slice(current.start_byte, split_byte));
    }
}
