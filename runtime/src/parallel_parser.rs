// Parallel parser for rust-sitter
// Uses rayon for data-parallel parsing of large files

use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::incremental_v3::{Subtree, SubtreePool, Tree};
use crate::parser_v3::{ParseNode, Parser};
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{Grammar, SymbolId};

/// Parallel parser configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Minimum file size (bytes) to enable parallel parsing
    pub min_file_size: usize,
    /// Target chunk size for splitting
    pub chunk_size: usize,
    /// Number of worker threads (0 = use rayon default)
    pub num_threads: usize,
    /// Enable subtree caching
    pub enable_caching: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            min_file_size: 100_000, // 100KB
            chunk_size: 50_000,     // 50KB chunks
            num_threads: 0,         // Use all available cores
            enable_caching: true,
        }
    }
}

/// Parallel parser for large files
pub struct ParallelParser {
    grammar: Arc<Grammar>,
    parse_table: Arc<ParseTable>,
    config: ParallelConfig,
    subtree_cache: Arc<Mutex<SubtreeCache>>,
}

/// Cache for reusable subtrees
struct SubtreeCache {
    cache: HashMap<u64, Arc<Subtree>>,
    pool: SubtreePool,
}

impl SubtreeCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            pool: SubtreePool::new(),
        }
    }

    fn get(&self, hash: u64) -> Option<Arc<Subtree>> {
        self.cache.get(&hash).cloned()
    }

    fn insert(&mut self, hash: u64, subtree: Arc<Subtree>) {
        self.cache.insert(hash, subtree);
    }
}

/// Chunk of input for parallel processing
#[derive(Debug)]
struct ParseChunk {
    start: usize,
    end: usize,
    content: Vec<u8>,
    /// Boundary type for chunk merging
    boundary: ChunkBoundary,
}

#[derive(Debug, Clone)]
enum ChunkBoundary {
    /// Clean boundary at statement/block level
    Clean,
    /// Boundary in the middle of a construct
    Dirty {
        /// Lookahead tokens for context
        lookahead: Vec<u8>,
        /// Lookbehind tokens for context  
        lookbehind: Vec<u8>,
    },
}

/// Result of parsing a chunk
#[derive(Debug)]
struct ChunkResult {
    chunk_id: usize,
    subtrees: Vec<Subtree>,
    /// Tokens that couldn't be fully parsed
    incomplete_tokens: Vec<IncompleteToken>,
    parse_time_ms: f64,
}

#[derive(Debug)]
struct IncompleteToken {
    start: usize,
    partial_content: Vec<u8>,
    expected_symbols: Vec<SymbolId>,
}

impl ParallelParser {
    pub fn new(grammar: Grammar, parse_table: ParseTable, config: ParallelConfig) -> Self {
        // Configure rayon thread pool if specified
        if config.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.num_threads)
                .build_global()
                .ok();
        }

        Self {
            grammar: Arc::new(grammar),
            parse_table: Arc::new(parse_table),
            config,
            subtree_cache: Arc::new(Mutex::new(SubtreeCache::new())),
        }
    }

    /// Parse input in parallel
    pub fn parse(&self, input: &str) -> Result<ParseNode> {
        let bytes = input.as_bytes();

        // For small files, use single-threaded parser
        if bytes.len() < self.config.min_file_size {
            let mut parser = Parser::new((*self.grammar).clone(), (*self.parse_table).clone());
            return parser.parse(input);
        }

        // Split input into chunks
        let chunks = self.split_into_chunks(bytes);

        // Parse chunks in parallel
        let chunk_results: Vec<ChunkResult> = chunks
            .into_par_iter()
            .enumerate()
            .map(|(id, chunk)| self.parse_chunk(id, chunk))
            .collect();

        // Merge results
        self.merge_chunk_results(chunk_results, bytes)
    }

    /// Split input into chunks for parallel processing
    fn split_into_chunks(&self, input: &[u8]) -> Vec<ParseChunk> {
        let mut chunks = Vec::new();
        let chunk_size = self.config.chunk_size;

        let mut start = 0;
        while start < input.len() {
            let mut end = (start + chunk_size).min(input.len());

            // Try to find a clean boundary
            let boundary = if end < input.len() {
                self.find_chunk_boundary(input, start, &mut end)
            } else {
                ChunkBoundary::Clean
            };

            chunks.push(ParseChunk {
                start,
                end,
                content: input[start..end].to_vec(),
                boundary,
            });

            start = end;
        }

        chunks
    }

    /// Find a good boundary for chunk splitting
    fn find_chunk_boundary(&self, input: &[u8], start: usize, end: &mut usize) -> ChunkBoundary {
        // Look for clean boundaries (newlines, semicolons, braces)
        let search_start = end.saturating_sub(1000); // Look back up to 1KB

        // Search backwards for clean break points
        for i in (search_start..*end).rev() {
            match input[i] {
                b'\n' => {
                    // Check if this is a statement boundary
                    if self.is_statement_boundary(input, i) {
                        *end = i + 1;
                        return ChunkBoundary::Clean;
                    }
                }
                b';' | b'}' => {
                    // Good boundary points
                    *end = i + 1;
                    return ChunkBoundary::Clean;
                }
                _ => {}
            }
        }

        // No clean boundary found, create dirty boundary with context
        let lookahead_start = *end;
        let lookahead_end = (*end + 100).min(input.len());
        let lookbehind_start = end.saturating_sub(100);

        ChunkBoundary::Dirty {
            lookahead: input[lookahead_start..lookahead_end].to_vec(),
            lookbehind: input[lookbehind_start..*end].to_vec(),
        }
    }

    /// Check if a position is at a statement boundary
    fn is_statement_boundary(&self, input: &[u8], pos: usize) -> bool {
        // Simple heuristic: check indentation
        if pos + 1 >= input.len() {
            return true;
        }

        // Skip whitespace after newline
        let mut i = pos + 1;
        while i < input.len() && (input[i] == b' ' || input[i] == b'\t') {
            i += 1;
        }

        // Check if we're at the start of a keyword or identifier
        if i < input.len() {
            match input[i] {
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => true,
                _ => false,
            }
        } else {
            true
        }
    }

    /// Parse a single chunk
    fn parse_chunk(&self, chunk_id: usize, chunk: ParseChunk) -> ChunkResult {
        use std::time::Instant;
        let start_time = Instant::now();

        // Create parser for this chunk
        let mut parser = Parser::new((*self.grammar).clone(), (*self.parse_table).clone());

        // Handle dirty boundaries by including context
        let parse_input = match &chunk.boundary {
            ChunkBoundary::Clean => chunk.content.clone(),
            ChunkBoundary::Dirty {
                lookbehind,
                lookahead,
            } => {
                // Include context for proper parsing
                let mut combined =
                    Vec::with_capacity(lookbehind.len() + chunk.content.len() + lookahead.len());
                combined.extend_from_slice(lookbehind);
                combined.extend_from_slice(&chunk.content);
                combined.extend_from_slice(lookahead);
                combined
            }
        };

        // Parse the chunk
        let input_str = String::from_utf8_lossy(&parse_input);
        let subtrees = match parser.parse(&input_str) {
            Ok(tree) => {
                // Convert to subtrees
                self.extract_subtrees(tree, chunk.start)
            }
            Err(_) => {
                // Partial parse - extract what we can
                Vec::new()
            }
        };

        let parse_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        ChunkResult {
            chunk_id,
            subtrees,
            incomplete_tokens: Vec::new(), // TODO: Track incomplete tokens
            parse_time_ms,
        }
    }

    /// Extract reusable subtrees from parse result
    fn extract_subtrees(&self, tree: ParseNode, offset: usize) -> Vec<Subtree> {
        let mut subtrees = Vec::new();

        // Convert ParseNode to Subtree format
        let subtree = Subtree {
            symbol: tree.symbol,
            start_byte: tree.start_byte + offset,
            end_byte: tree.end_byte + offset,
            children: tree
                .children
                .into_iter()
                .map(|child| self.extract_subtrees(child, offset))
                .flatten()
                .collect(),
        };

        // Cache if enabled
        if self.config.enable_caching {
            let hash = self.hash_subtree(&subtree);
            let mut cache = self.subtree_cache.lock().unwrap();
            cache.insert(hash, Arc::new(subtree.clone()));
        }

        subtrees.push(subtree);
        subtrees
    }

    /// Hash a subtree for caching
    fn hash_subtree(&self, subtree: &Subtree) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        subtree.symbol.hash(&mut hasher);
        subtree.start_byte.hash(&mut hasher);
        subtree.end_byte.hash(&mut hasher);
        hasher.finish()
    }

    /// Merge chunk results into final tree
    fn merge_chunk_results(
        &self,
        mut results: Vec<ChunkResult>,
        input: &[u8],
    ) -> Result<ParseNode> {
        // Sort by chunk ID to maintain order
        results.sort_by_key(|r| r.chunk_id);

        // Collect all subtrees
        let mut all_subtrees = Vec::new();
        for result in results {
            all_subtrees.extend(result.subtrees);
        }

        // Build final tree
        self.build_tree_from_subtrees(all_subtrees, input)
    }

    /// Build parse tree from subtrees
    fn build_tree_from_subtrees(&self, subtrees: Vec<Subtree>, input: &[u8]) -> Result<ParseNode> {
        let root_symbol = self.grammar.start_symbol().unwrap_or(SymbolId(0));

        // Convert subtrees to parse nodes, flattening any roots that match the start symbol
        let mut children: Vec<ParseNode> = subtrees
            .into_iter()
            .flat_map(|st| {
                let node = self.subtree_to_node(st);
                if node.symbol == root_symbol {
                    node.children
                } else {
                    vec![node]
                }
            })
            .collect();

        // Ensure children are ordered by their byte positions
        children.sort_by_key(|c| c.start_byte);

        Ok(ParseNode {
            symbol: root_symbol,
            children,
            start_byte: 0,
            end_byte: input.len(),
            field_name: None,
        })
    }

    /// Convert subtree to parse node, attaching field names using grammar metadata
    fn subtree_to_node(&self, subtree: Subtree) -> ParseNode {
        let mut children: Vec<ParseNode> = subtree
            .children
            .into_iter()
            .map(|st| self.subtree_to_node(st))
            .collect();

        // Attach field names based on grammar rule metadata
        if let Some(rule) = self.match_rule(subtree.symbol, &children) {
            for &(field_id, index) in &rule.fields {
                if let Some(child) = children.get_mut(index) {
                    if let Some(name) = self.grammar.fields.get(&field_id) {
                        child.field_name = Some(name.clone());
                    }
                }
            }
        }

        ParseNode {
            symbol: subtree.symbol,
            children,
            start_byte: subtree.start_byte,
            end_byte: subtree.end_byte,
            field_name: None,
        }
    }

    /// Find the grammar rule matching a subtree's children
    fn match_rule(&self, symbol: SymbolId, children: &[ParseNode]) -> Option<&rust_sitter_ir::Rule> {
        use rust_sitter_ir::Symbol;

        let rules = self.grammar.rules.get(&symbol)?;
        rules.iter().find(|rule| {
            if rule.rhs.len() != children.len() {
                return false;
            }
            rule.rhs.iter().zip(children.iter()).all(|(sym, child)| match sym {
                Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id == &child.symbol,
                _ => false,
            })
        })
    }
}

/// Parallel parsing statistics
#[derive(Debug, Default)]
pub struct ParallelStats {
    pub total_chunks: usize,
    pub clean_boundaries: usize,
    pub dirty_boundaries: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub total_parse_time_ms: f64,
    pub merge_time_ms: f64,
}

impl ParallelParser {
    /// Parse with statistics collection
    pub fn parse_with_stats(&self, input: &str) -> Result<(ParseNode, ParallelStats)> {
        let mut stats = ParallelStats::default();

        // TODO: Implement stats collection
        let tree = self.parse(input)?;

        Ok((tree, stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
    use rust_sitter_ir::{FieldId, ProductionId, Rule, Symbol, Token, TokenPattern};

    // Build a simple grammar with field metadata for testing
    fn create_test_grammar() -> (Grammar, ParseTable) {
        let mut grammar = Grammar::new("test".to_string());

        const SYM_NUMBER: SymbolId = SymbolId(1);
        const SYM_WS: SymbolId = SymbolId(2);
        const SYM_PAIR: SymbolId = SymbolId(3);
        const SYM_ROOT: SymbolId = SymbolId(4);

        grammar.tokens.insert(
            SYM_NUMBER,
            Token {
                name: "number".to_string(),
                pattern: TokenPattern::Regex("\\d+".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SYM_WS,
            Token {
                name: "_ws".to_string(),
                pattern: TokenPattern::Regex("\\s+".to_string()),
                fragile: false,
            },
        );
        grammar.extras.push(SYM_WS);

        grammar.rule_names.insert(SYM_PAIR, "pair".to_string());
        grammar.rule_names.insert(SYM_ROOT, "root".to_string());

        grammar.fields.insert(FieldId(0), "left".to_string());
        grammar.fields.insert(FieldId(1), "right".to_string());

        let pair_rule = Rule {
            lhs: SYM_PAIR,
            rhs: vec![Symbol::Terminal(SYM_NUMBER), Symbol::Terminal(SYM_NUMBER)],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
            production_id: ProductionId(0),
        };

        let root_rule_rec = Rule {
            lhs: SYM_ROOT,
            rhs: vec![Symbol::NonTerminal(SYM_PAIR), Symbol::NonTerminal(SYM_ROOT)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        };

        let root_rule_base = Rule {
            lhs: SYM_ROOT,
            rhs: vec![Symbol::NonTerminal(SYM_PAIR)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        };

        grammar.rules.insert(SYM_PAIR, vec![pair_rule]);
        grammar.rules.insert(SYM_ROOT, vec![root_rule_rec, root_rule_base]);

        let ff = FirstFollowSets::compute(&grammar);
        let table = build_lr1_automaton(&grammar, &ff).expect("table");
        (grammar, table)
    }

    #[test]
    fn test_chunk_splitting() {
        let (grammar, table) = create_test_grammar();
        let config = ParallelConfig {
            min_file_size: 10,
            chunk_size: 20,
            ..Default::default()
        };
        let parser = ParallelParser::new(grammar, table, config);

        let input = b"line1\nline2\nline3\nline4\nline5";
        let chunks = parser.split_into_chunks(input);

        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.end > chunk.start);
            assert_eq!(&input[chunk.start..chunk.end], &chunk.content[..]);
        }
    }

    #[test]
    fn test_boundary_detection() {
        let (grammar, table) = create_test_grammar();
        let parser = ParallelParser::new(grammar, table, Default::default());

        // Test clean boundaries
        assert!(parser.is_statement_boundary(b"}\nfunction", 1));
        assert!(parser.is_statement_boundary(b";\nlet x", 1));

        // Test dirty boundaries
        assert!(!parser.is_statement_boundary(b"hello\n    world", 5));
    }

    #[test]
    fn test_subtree_merging_with_fields() {
        let (grammar, table) = create_test_grammar();
        let parser = ParallelParser::new(grammar, table, Default::default());

        const SYM_NUMBER: SymbolId = SymbolId(1);
        const SYM_PAIR: SymbolId = SymbolId(3);
        const SYM_ROOT: SymbolId = SymbolId(4);

        let num1 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 0, end_byte: 1 };
        let num2 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 2, end_byte: 3 };
        let pair1 = Subtree { symbol: SYM_PAIR, children: vec![num1, num2], start_byte: 0, end_byte: 3 };
        let root1 = Subtree { symbol: SYM_ROOT, children: vec![pair1], start_byte: 0, end_byte: 3 };

        let num3 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 4, end_byte: 5 };
        let num4 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 6, end_byte: 7 };
        let pair2 = Subtree { symbol: SYM_PAIR, children: vec![num3, num4], start_byte: 4, end_byte: 7 };
        let root2 = Subtree { symbol: SYM_ROOT, children: vec![pair2], start_byte: 4, end_byte: 7 };

        // Intentionally pass subtrees out of order to test ordering
        let tree = parser
            .build_tree_from_subtrees(vec![root2, root1], b"1 2 3 4")
            .unwrap();

        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].start_byte, 0);
        assert_eq!(tree.children[1].start_byte, 4);

        // Check field names on pair children
        let first_pair = &tree.children[0];
        assert_eq!(
            first_pair.children[0].field_name.as_deref(),
            Some("left")
        );
        assert_eq!(
            first_pair.children[1].field_name.as_deref(),
            Some("right")
        );
    }

    #[test]
    fn test_merge_chunk_results_ordering() {
        let (grammar, table) = create_test_grammar();
        let parser = ParallelParser::new(grammar, table, Default::default());

        const SYM_NUMBER: SymbolId = SymbolId(1);
        const SYM_PAIR: SymbolId = SymbolId(3);
        const SYM_ROOT: SymbolId = SymbolId(4);

        let num1 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 0, end_byte: 1 };
        let num2 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 2, end_byte: 3 };
        let pair1 = Subtree { symbol: SYM_PAIR, children: vec![num1, num2], start_byte: 0, end_byte: 3 };
        let root1 = Subtree { symbol: SYM_ROOT, children: vec![pair1], start_byte: 0, end_byte: 3 };

        let num3 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 4, end_byte: 5 };
        let num4 = Subtree { symbol: SYM_NUMBER, children: vec![], start_byte: 6, end_byte: 7 };
        let pair2 = Subtree { symbol: SYM_PAIR, children: vec![num3, num4], start_byte: 4, end_byte: 7 };
        let root2 = Subtree { symbol: SYM_ROOT, children: vec![pair2], start_byte: 4, end_byte: 7 };

        let result = parser
            .merge_chunk_results(
                vec![
                    ChunkResult { chunk_id: 1, subtrees: vec![root2], incomplete_tokens: vec![], parse_time_ms: 0.0 },
                    ChunkResult { chunk_id: 0, subtrees: vec![root1], incomplete_tokens: vec![], parse_time_ms: 0.0 },
                ],
                b"1 2 3 4",
            )
            .unwrap();

        assert_eq!(result.children[0].start_byte, 0);
        assert_eq!(result.children[1].start_byte, 4);
    }
}

/// Benchmark utilities
#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use std::time::Instant;

    pub struct ParallelBenchmark {
        pub file_size: usize,
        pub single_thread_ms: f64,
        pub parallel_ms: f64,
        pub speedup: f64,
        pub num_chunks: usize,
    }

    pub fn benchmark_parallel_parsing(
        grammar: Grammar,
        table: ParseTable,
        input: &str,
    ) -> ParallelBenchmark {
        // Single-threaded baseline
        let start = Instant::now();
        let mut parser = Parser::new(grammar.clone(), table.clone());
        let _ = parser.parse(input);
        let single_thread_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Parallel parsing
        let config = ParallelConfig::default();
        let parallel_parser = ParallelParser::new(grammar, table, config);

        let start = Instant::now();
        let (_, stats) = parallel_parser.parse_with_stats(input).unwrap();
        let parallel_ms = start.elapsed().as_secs_f64() * 1000.0;

        ParallelBenchmark {
            file_size: input.len(),
            single_thread_ms,
            parallel_ms,
            speedup: single_thread_ms / parallel_ms,
            num_chunks: stats.total_chunks,
        }
    }
}
