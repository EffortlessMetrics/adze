/// Test-only symbol allocator that starts at 1 to avoid EOF collision
#[cfg(test)]
pub mod test {
    use rust_sitter_ir::SymbolId;

    /// Symbol allocator for tests that guarantees no collision with EOF (SymbolId(0))
    pub struct SymbolAllocator(u16);

    impl SymbolAllocator {
        /// Create a new allocator starting at SymbolId(1)
        pub fn new() -> Self {
            Self(1)
        }

        /// Allocate the next symbol ID
        pub fn next_id(&mut self) -> SymbolId {
            let id = self.0;
            self.0 = self.0.checked_add(1).expect("Symbol ID overflow");
            SymbolId(id)
        }

        /// Allocate N consecutive symbol IDs
        pub fn next_n(&mut self, n: usize) -> Vec<SymbolId> {
            (0..n).map(|_| self.next_id()).collect()
        }

        /// Get current symbol without advancing
        pub fn current(&self) -> SymbolId {
            SymbolId(self.0)
        }
    }

    impl Default for SymbolAllocator {
        fn default() -> Self {
            Self::new()
        }
    }
}
