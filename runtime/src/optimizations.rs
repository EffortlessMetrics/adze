// Performance optimizations for pure-Rust parser
use std::sync::atomic::{AtomicU64, Ordering};

/// Performance statistics collector
pub struct PerfStats {
    pub parse_calls: AtomicU64,
    pub total_parse_time_ns: AtomicU64,
    pub total_bytes_parsed: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

impl PerfStats {
    pub fn new() -> Self {
        Self {
            parse_calls: AtomicU64::new(0),
            total_parse_time_ns: AtomicU64::new(0),
            total_bytes_parsed: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    pub fn record_parse(&self, bytes: u64, time_ns: u64) {
        self.parse_calls.fetch_add(1, Ordering::Relaxed);
        self.total_parse_time_ns
            .fetch_add(time_ns, Ordering::Relaxed);
        self.total_bytes_parsed.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn report(&self) -> String {
        let calls = self.parse_calls.load(Ordering::Relaxed);
        let time_ns = self.total_parse_time_ns.load(Ordering::Relaxed);
        let bytes = self.total_bytes_parsed.load(Ordering::Relaxed);
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);

        let avg_time_us = if calls > 0 {
            (time_ns / calls) / 1000
        } else {
            0
        };
        let throughput_mb_s = if time_ns > 0 {
            (bytes as f64 / (1024.0 * 1024.0)) / (time_ns as f64 / 1_000_000_000.0)
        } else {
            0.0
        };
        let cache_hit_rate = if hits + misses > 0 {
            (hits as f64 / (hits + misses) as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "Parse Performance Stats:\n\
             - Total parses: {}\n\
             - Average parse time: {} μs\n\
             - Throughput: {:.2} MB/s\n\
             - Cache hit rate: {:.1}%",
            calls, avg_time_us, throughput_mb_s, cache_hit_rate
        )
    }
}

/// SIMD-accelerated utilities
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub mod simd {
    use std::arch::x86_64::*;
    // SIMD utilities for x86_64

    /// Find the next newline character using SIMD
    #[target_feature(enable = "sse2")]
    pub unsafe fn find_newline_simd_impl(data: &[u8]) -> Option<usize> {
        unsafe {
            let newline = _mm_set1_epi8(b'\n' as i8);
            let len = data.len();
            let mut i = 0;

            // Process 16 bytes at a time with SSE2
            while i + 16 <= len {
                let chunk = _mm_loadu_si128(data.as_ptr().add(i) as *const __m128i);
                let cmp = _mm_cmpeq_epi8(chunk, newline);
                let mask = _mm_movemask_epi8(cmp);

                if mask != 0 {
                    return Some(i + mask.trailing_zeros() as usize);
                }

                i += 16;
            }

            // Handle remaining bytes
            while i < len {
                if data[i] == b'\n' {
                    return Some(i);
                }
                i += 1;
            }

            None
        }
    }

    /// Safe wrapper for find_newline
    pub fn find_newline_simd(data: &[u8]) -> Option<usize> {
        if is_x86_feature_detected!("sse2") {
            unsafe { find_newline_simd_impl(data) }
        } else {
            data.iter().position(|&b| b == b'\n')
        }
    }

    /// Count whitespace characters using SIMD
    #[target_feature(enable = "sse2")]
    pub unsafe fn count_whitespace_simd_impl(data: &[u8]) -> usize {
        unsafe {
            let space = _mm_set1_epi8(b' ' as i8);
            let tab = _mm_set1_epi8(b'\t' as i8);
            let newline = _mm_set1_epi8(b'\n' as i8);
            let cr = _mm_set1_epi8(b'\r' as i8);

            let mut count = 0;
            let mut i = 0;

            // Process 16 bytes at a time
            while i + 16 <= data.len() {
                let chunk = _mm_loadu_si128(data.as_ptr().add(i) as *const __m128i);

                let is_space = _mm_cmpeq_epi8(chunk, space);
                let is_tab = _mm_cmpeq_epi8(chunk, tab);
                let is_newline = _mm_cmpeq_epi8(chunk, newline);
                let is_cr = _mm_cmpeq_epi8(chunk, cr);

                let is_whitespace = _mm_or_si128(
                    _mm_or_si128(is_space, is_tab),
                    _mm_or_si128(is_newline, is_cr),
                );

                let mask = _mm_movemask_epi8(is_whitespace);
                count += mask.count_ones() as usize;

                i += 16;
            }

            // Handle remaining bytes
            count += data[i..]
                .iter()
                .filter(|&&b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
                .count();

            count
        }
    }

    /// Safe wrapper for count_whitespace
    pub fn count_whitespace_simd(data: &[u8]) -> usize {
        if is_x86_feature_detected!("sse2") {
            unsafe { count_whitespace_simd_impl(data) }
        } else {
            data.iter()
                .filter(|&&b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
                .count()
        }
    }

    /// Fast string comparison using SIMD
    #[target_feature(enable = "sse2")]
    pub unsafe fn compare_bytes_simd_impl(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        unsafe {
            let mut i = 0;

            // Compare 16 bytes at a time
            while i + 16 <= a.len() {
                let chunk_a = _mm_loadu_si128(a.as_ptr().add(i) as *const __m128i);
                let chunk_b = _mm_loadu_si128(b.as_ptr().add(i) as *const __m128i);

                let cmp = _mm_cmpeq_epi8(chunk_a, chunk_b);
                let mask = _mm_movemask_epi8(cmp);

                if mask != 0xFFFF {
                    return false;
                }

                i += 16;
            }

            // Compare remaining bytes
            a[i..] == b[i..]
        }
    }

    /// Safe wrapper for compare_bytes
    pub fn compare_bytes_simd(a: &[u8], b: &[u8]) -> bool {
        if is_x86_feature_detected!("sse2") {
            unsafe { compare_bytes_simd_impl(a, b) }
        } else {
            a == b
        }
    }
}

/// Memory pool for reducing allocations
pub struct MemoryPool<T> {
    pool: Vec<Box<T>>,
    in_use: Vec<bool>,
}

impl<T: Default> MemoryPool<T> {
    pub fn new(initial_size: usize) -> Self {
        let pool = (0..initial_size).map(|_| Box::new(T::default())).collect();
        let in_use = vec![false; initial_size];

        Self { pool, in_use }
    }

    pub fn acquire(&mut self) -> Option<&mut T> {
        for (i, used) in self.in_use.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return Some(&mut *self.pool[i]);
            }
        }

        // Grow the pool if needed
        self.pool.push(Box::new(T::default()));
        self.in_use.push(true);
        self.pool.last_mut().map(|b| &mut **b)
    }

    pub fn release(&mut self, item: *const T) {
        for (i, boxed) in self.pool.iter().enumerate() {
            if std::ptr::eq(&**boxed as *const T, item) {
                self.in_use[i] = false;
                return;
            }
        }
    }

    pub fn clear(&mut self) {
        self.in_use.fill(false);
    }
}

/// Cache for parsed subtrees
pub struct SubtreeCache {
    cache: ahash::AHashMap<u64, crate::pure_parser::ParsedNode>,
    max_size: usize,
}

impl SubtreeCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: ahash::AHashMap::with_capacity(max_size),
            max_size,
        }
    }

    pub fn get(&self, key: u64) -> Option<&crate::pure_parser::ParsedNode> {
        self.cache.get(&key)
    }

    pub fn insert(&mut self, key: u64, node: crate::pure_parser::ParsedNode) {
        if self.cache.len() >= self.max_size {
            // Simple eviction: remove a random entry
            if let Some(&k) = self.cache.keys().next() {
                self.cache.remove(&k);
            }
        }
        self.cache.insert(key, node);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Compute a fast hash for cache keys
pub fn compute_cache_key(source: &[u8], start: usize, end: usize) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = ahash::AHasher::default();

    start.hash(&mut hasher);
    end.hash(&mut hasher);
    source[start..end.min(source.len())].hash(&mut hasher);

    hasher.finish()
}

/// Optimize parser for specific workloads
pub fn optimize_for_workload(parser: &mut crate::pure_parser::Parser, workload: Workload) {
    match workload {
        Workload::SmallFiles => {
            // Optimize for many small files
            parser.set_timeout_micros(1000); // 1ms timeout
        }
        Workload::LargeFiles => {
            // Optimize for large files
            parser.set_timeout_micros(0); // No timeout
        }
        Workload::Interactive => {
            // Optimize for interactive use (low latency)
            parser.set_timeout_micros(16000); // 16ms (one frame)
        }
        Workload::Batch => {
            // Optimize for batch processing (high throughput)
            parser.set_timeout_micros(0); // No timeout
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Workload {
    SmallFiles,
    LargeFiles,
    Interactive,
    Batch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_stats() {
        let stats = PerfStats::new();
        stats.record_parse(1000, 5000);
        stats.record_cache_hit();
        stats.record_cache_miss();

        let report = stats.report();
        assert!(report.contains("Parse Performance Stats"));
    }

    #[cfg(feature = "simd")]
    #[test]
    fn test_simd_newline() {
        let data = b"hello\nworld\n";
        assert_eq!(simd::find_newline_simd(data), Some(5));
    }

    #[test]
    fn test_memory_pool() {
        let mut pool: MemoryPool<Vec<u8>> = MemoryPool::new(2);

        let item1 = pool.acquire().unwrap();
        item1.push(1);

        let item2 = pool.acquire().unwrap();
        item2.push(2);

        // Pool should grow
        let item3 = pool.acquire().unwrap();
        item3.push(3);

        assert_eq!(pool.pool.len(), 3);
    }
}
