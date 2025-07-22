use crate::compress::CompressedParseTable;

/// Validates that a generated Language struct meets Tree-sitter ABI requirements
pub struct LanguageValidator<'a> {
    language: &'a TSLanguage,
    tables: &'a CompressedParseTable,
}

/// Tree-sitter Language struct for ABI v15
#[repr(C)]
pub struct TSLanguage {
    pub version: u32,
    pub symbol_count: u32,
    pub alias_count: u32,
    pub token_count: u32,
    pub external_token_count: u32,
    pub state_count: u32,
    pub large_state_count: u32,
    pub production_id_count: u32,
    pub field_count: u32,
    pub max_alias_sequence_length: u16,
    pub parse_table: *const u16,
    pub small_parse_table: *const u16,
    pub small_parse_table_map: *const u32,
    pub parse_actions: *const TSParseActionEntry,
    pub symbol_names: *const *const i8,
    pub field_names: *const *const i8,
    pub field_map_slices: *const TSFieldMapSlice,
    pub field_map_entries: *const TSFieldMapEntry,
    pub symbol_metadata: *const TSSymbolMetadata,
    pub public_symbol_map: *const TSSymbol,
    pub alias_map: *const u16,
    pub alias_sequences: *const TSSymbol,
    pub lex_modes: *const TSLexMode,
    pub lex_fn: Option<unsafe extern "C" fn(*mut TSLexer, TSStateId) -> bool>,
    pub keyword_lex_fn: Option<unsafe extern "C" fn(*mut TSLexer, TSStateId) -> bool>,
    pub keyword_capture_token: TSSymbol,
    pub external_scanner_data: TSExternalScannerData,
    pub primary_state_ids: *const TSStateId,
}

#[repr(C)]
pub struct TSParseActionEntry {
    pub action: u32,
}

#[repr(C)]
pub struct TSFieldMapSlice {
    pub start: u16,
    pub length: u16,
}

#[repr(C)]
pub struct TSFieldMapEntry {
    pub field_id: u16,
    pub child_index: u8,
    pub inherited: bool,
}

#[repr(C)]
pub struct TSSymbolMetadata {
    pub visible: bool,
    pub named: bool,
}

#[repr(C)]
pub struct TSLexMode {
    pub lex_mode_id: u8,
}

#[repr(C)]
pub struct TSExternalScannerData {
    pub states: *const bool,
    pub symbol_map: *const TSSymbol,
    pub create: Option<unsafe extern "C" fn() -> *mut std::ffi::c_void>,
    pub destroy: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
    pub scan: Option<
        unsafe extern "C" fn(*mut std::ffi::c_void, *mut TSLexer, *const bool) -> bool,
    >,
    pub serialize: Option<
        unsafe extern "C" fn(*mut std::ffi::c_void, *mut u8) -> u32,
    >,
    pub deserialize: Option<
        unsafe extern "C" fn(*mut std::ffi::c_void, *const u8, u32),
    >,
}

#[repr(C)]
pub struct TSLexer {
    pub lookahead: i32,
    pub result_symbol: TSSymbol,
    pub advance: Option<unsafe extern "C" fn(*mut TSLexer, bool)>,
    pub mark_end: Option<unsafe extern "C" fn(*mut TSLexer)>,
    pub get_column: Option<unsafe extern "C" fn(*mut TSLexer) -> u32>,
    pub is_at_included_range_start: Option<unsafe extern "C" fn(*mut TSLexer) -> bool>,
    pub eof: Option<unsafe extern "C" fn(*mut TSLexer) -> bool>,
}

pub type TSSymbol = u16;
pub type TSStateId = u16;

/// Validation errors that can occur when checking Language structs
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Language version doesn't match expected ABI
    InvalidVersion { expected: u32, actual: u32 },
    
    /// Symbol count doesn't match tables
    SymbolCountMismatch { language: u32, tables: u32 },
    
    /// State count doesn't match tables
    StateCountMismatch { language: u32, tables: u32 },
    
    /// Null pointer where data was expected
    NullPointer(&'static str),
    
    /// Field names not in lexicographic order
    FieldNamesNotSorted,
    
    /// Invalid symbol metadata
    InvalidSymbolMetadata { symbol: TSSymbol, reason: String },
    
    /// Table dimensions don't match metadata
    TableDimensionMismatch { expected: usize, actual: usize },
    
    /// Production ID out of bounds
    InvalidProductionId { id: u32, max: u32 },
    
    /// Invalid field mapping
    InvalidFieldMapping { field_id: u16, max: u16 },
}

impl<'a> LanguageValidator<'a> {
    /// Creates a new validator for the given Language and tables
    pub fn new(language: &'a TSLanguage, tables: &'a CompressedParseTable) -> Self {
        Self { language, tables }
    }
    
    /// Performs comprehensive validation of the Language struct
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Check ABI version
        if self.language.version != 15 {
            errors.push(ValidationError::InvalidVersion {
                expected: 15,
                actual: self.language.version,
            });
        }
        
        // Validate counts match tables
        self.validate_counts(&mut errors);
        
        // Validate pointers are non-null where required
        self.validate_pointers(&mut errors);
        
        // Validate symbol metadata
        self.validate_symbol_metadata(&mut errors);
        
        // Validate field names ordering
        self.validate_field_names(&mut errors);
        
        // Validate table dimensions
        self.validate_table_dimensions(&mut errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn validate_counts(&self, errors: &mut Vec<ValidationError>) {
        // Check symbol count
        let table_symbol_count = self.tables.symbol_count();
        if self.language.symbol_count != table_symbol_count as u32 {
            errors.push(ValidationError::SymbolCountMismatch {
                language: self.language.symbol_count,
                tables: table_symbol_count as u32,
            });
        }
        
        // Check state count
        let table_state_count = self.tables.state_count();
        if self.language.state_count != table_state_count as u32 {
            errors.push(ValidationError::StateCountMismatch {
                language: self.language.state_count,
                tables: table_state_count as u32,
            });
        }
    }
    
    fn validate_pointers(&self, errors: &mut Vec<ValidationError>) {
        // Parse tables must be present
        if self.language.parse_table.is_null() && self.language.small_parse_table.is_null() {
            errors.push(ValidationError::NullPointer("parse_table or small_parse_table"));
        }
        
        // Symbol names must be present
        if self.language.symbol_names.is_null() {
            errors.push(ValidationError::NullPointer("symbol_names"));
        }
        
        // Symbol metadata must be present
        if self.language.symbol_metadata.is_null() {
            errors.push(ValidationError::NullPointer("symbol_metadata"));
        }
        
        // Field names must be present if field_count > 0
        if self.language.field_count > 0 && self.language.field_names.is_null() {
            errors.push(ValidationError::NullPointer("field_names"));
        }
    }
    
    fn validate_symbol_metadata(&self, errors: &mut Vec<ValidationError>) {
        if self.language.symbol_metadata.is_null() {
            return;
        }
        
        unsafe {
            let metadata_slice = std::slice::from_raw_parts(
                self.language.symbol_metadata,
                self.language.symbol_count as usize,
            );
            
            // First symbol should always be unnamed and invisible (EOF)
            if metadata_slice[0].visible || metadata_slice[0].named {
                errors.push(ValidationError::InvalidSymbolMetadata {
                    symbol: 0,
                    reason: "EOF symbol must be invisible and unnamed".to_string(),
                });
            }
        }
    }
    
    fn validate_field_names(&self, errors: &mut Vec<ValidationError>) {
        if self.language.field_count == 0 || self.language.field_names.is_null() {
            return;
        }
        
        unsafe {
            let field_names = std::slice::from_raw_parts(
                self.language.field_names,
                self.language.field_count as usize + 1, // +1 for empty string at start
            );
            
            // Check lexicographic ordering
            for i in 2..field_names.len() {
                let prev = std::ffi::CStr::from_ptr(field_names[i - 1]);
                let curr = std::ffi::CStr::from_ptr(field_names[i]);
                
                if prev >= curr {
                    errors.push(ValidationError::FieldNamesNotSorted);
                    break;
                }
            }
        }
    }
    
    fn validate_table_dimensions(&self, _errors: &mut Vec<ValidationError>) {
        // Validate based on whether we have small or large tables
        if !self.language.small_parse_table.is_null() {
            // Small table validation
            let _expected_entries = self.language.state_count as usize * self.language.symbol_count as usize;
            // Additional validation would require accessing the actual table data
        } else if !self.language.parse_table.is_null() {
            // Large table validation
            // Would need to check parse_actions array length matches compressed data
        }
    }
}

/// Creates a test Language struct for validation testing
#[cfg(test)]
pub fn create_test_language() -> TSLanguage {
    TSLanguage {
        version: 15,
        symbol_count: 10,
        alias_count: 0,
        token_count: 5,
        external_token_count: 0,
        state_count: 20,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: std::ptr::null(),
        small_parse_table: std::ptr::null(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner_data: TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_validation() {
        let mut language = create_test_language();
        language.version = 14; // Wrong version
        
        let tables = CompressedParseTable::new_for_testing(10, 20);
        let validator = LanguageValidator::new(&language, &tables);
        
        let result = validator.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidVersion { .. })));
    }
    
    #[test]
    fn test_null_pointer_validation() {
        let language = create_test_language();
        let tables = CompressedParseTable::new_for_testing(10, 20);
        let validator = LanguageValidator::new(&language, &tables);
        
        let result = validator.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::NullPointer(_))));
    }
}