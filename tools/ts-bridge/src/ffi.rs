use std::os::raw::c_char;

#[repr(C)]
pub struct TSLanguage {
    _priv: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsbEntryHeader {
    pub action_index: u32,
    pub count: u8,
    pub reusable: bool,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TsbActionKind {
    Shift = 0,
    Reduce = 1,
    Accept = 2,
    Recover = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsbAction {
    pub kind: TsbActionKind,
    pub state: u16,
    pub lhs: u16,
    pub rhs_len: u16,
    pub dynamic_precedence: i16,
    pub production_id: u16,
    pub extra: bool,
    pub repetition: bool,
}

impl Default for TsbAction {
    fn default() -> Self {
        TsbAction {
            kind: TsbActionKind::Shift,
            state: 0,
            lhs: 0,
            rhs_len: 0,
            dynamic_precedence: 0,
            production_id: 0,
            extra: false,
            repetition: false,
        }
    }
}

/// Symbol metadata from Tree-sitter
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsbSymbolMetadata {
    pub visible: bool,
    pub named: bool,
}

unsafe extern "C" {
    pub fn tsb_language_version() -> u32;
    pub fn tsb_min_compatible_version() -> u32;

    pub fn tsb_counts(
        lang: *const TSLanguage,
        symc: *mut u32,
        stc: *mut u32,
        tokc: *mut u32,
        extc: *mut u32,
        lstc: *mut u32,
    );

    pub fn tsb_symbol_name(lang: *const TSLanguage, sym: u32) -> *const c_char;

    pub fn tsb_symbol_metadata(lang: *const TSLanguage, sym: u32) -> TsbSymbolMetadata;

    pub fn tsb_table_entry(
        lang: *const TSLanguage,
        state: u32,
        symbol: u32,
        hdr: *mut TsbEntryHeader,
    ) -> u32;

    pub fn tsb_unpack_actions(
        lang: *const TSLanguage,
        action_index: u32,
        count: u8,
        out: *mut TsbAction,
        cap: u32,
    ) -> u32;

    pub fn tsb_next_state(lang: *const TSLanguage, state: u32, nonterm: u32) -> u32;

    pub fn tsb_detect_start_symbol(lang: *const TSLanguage) -> u32;
}

pub struct SafeLang(pub *const TSLanguage);

unsafe impl Send for SafeLang {}
unsafe impl Sync for SafeLang {}

impl SafeLang {
    pub fn assert_abi() {
        unsafe {
            let v = tsb_language_version();
            let min = tsb_min_compatible_version();
            assert!(
                v == 15 && v >= min,
                "Tree-sitter ABI drift: v={} min={}",
                v,
                min
            );
        }
    }

    pub fn counts(&self) -> (u32, u32, u32, u32, u32) {
        let mut a = 0;
        let mut b = 0;
        let mut c = 0;
        let mut d = 0;
        let mut e = 0;
        unsafe {
            tsb_counts(self.0, &mut a, &mut b, &mut c, &mut d, &mut e);
        }
        (a, b, c, d, e)
    }

    pub fn symbol_name(&self, sym: u32) -> String {
        unsafe {
            let p = tsb_symbol_name(self.0, sym);
            let s = std::ffi::CStr::from_ptr(p);
            s.to_string_lossy().into_owned()
        }
    }

    pub fn symbol_metadata(&self, sym: u32) -> TsbSymbolMetadata {
        unsafe { tsb_symbol_metadata(self.0, sym) }
    }

    pub fn entry(&self, state: u32, symbol: u32) -> Option<(TsbEntryHeader, u32)> {
        let mut hdr = TsbEntryHeader {
            action_index: 0,
            count: 0,
            reusable: false,
        };
        let idx = unsafe { tsb_table_entry(self.0, state, symbol, &mut hdr) };
        if hdr.count > 0 {
            Some((hdr, idx))
        } else {
            None
        }
    }

    pub fn unpack(&self, idx: u32, cnt: u8, out: &mut [TsbAction]) -> usize {
        unsafe { tsb_unpack_actions(self.0, idx, cnt, out.as_mut_ptr(), out.len() as u32) as usize }
    }

    pub fn next_state(&self, state: u32, nonterm: u32) -> u32 {
        unsafe { tsb_next_state(self.0, state, nonterm) }
    }

    pub fn detect_start_symbol(&self) -> u32 {
        unsafe { tsb_detect_start_symbol(self.0) }
    }
}

/// Assert that the Tree-sitter ABI is compatible
pub fn assert_abi_compatible() {
    let v = unsafe { tsb_language_version() };
    let min = unsafe { tsb_min_compatible_version() };
    assert!(
        v == 15 && v >= min,
        "Tree-sitter ABI drift: got {}, expected 15+",
        v
    );
}
