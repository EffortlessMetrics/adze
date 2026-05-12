use adze_ir::FieldId;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::ffi::{CStr, c_char};

use crate::pure_parser::TSLanguage;

pub(super) fn decode_field_names(lang: &TSLanguage) -> IndexMap<FieldId, String> {
    let mut field_names_map = IndexMap::new();
    if lang.field_names.is_null() || lang.field_count == 0 {
        return field_names_map;
    }

    let field_count = lang.field_count as usize;
    // SAFETY: `lang.field_names` is non-null and `field_count > 0` (branch guard).
    // TSLanguage contract guarantees the array has `field_count` elements.
    let field_name_ptrs = unsafe { std::slice::from_raw_parts(lang.field_names, field_count) };

    for (i, &name_ptr) in field_name_ptrs.iter().enumerate() {
        if name_ptr.is_null() {
            continue;
        }

        // SAFETY: `name_ptr` is non-null (branch guard) and points to a
        // null-terminated C string per TSLanguage contract.
        let name = match unsafe { CStr::from_ptr(name_ptr as *const c_char) }.to_str() {
            Ok(valid_str) => valid_str.to_owned(),
            Err(_) => format!("field_invalid_{}", i),
        };
        field_names_map.insert(FieldId(i as u16), name);
    }

    field_names_map
}

pub(super) fn decode_rule_fields(lang: &TSLanguage, rule_index: usize) -> Vec<(FieldId, usize)> {
    if lang.field_count == 0 || lang.field_map_slices.is_null() || lang.field_map_entries.is_null()
    {
        return Vec::new();
    }

    let production_count = lang.production_id_count as usize;
    let slice_count = production_count.saturating_mul(2);
    if slice_count == 0 || slice_count > usize::MAX / 4 {
        return Vec::new();
    }

    // SAFETY: `lang.field_map_slices` is non-null (branch guard) and
    // `slice_count = production_count * 2`. TSLanguage contract guarantees
    // the field_map_slices array covers all productions.
    let slices = unsafe { std::slice::from_raw_parts(lang.field_map_slices, slice_count) };
    let slice_idx = rule_index.saturating_mul(2);
    if slice_idx + 1 >= slices.len() {
        return Vec::new();
    }

    let start = slices[slice_idx] as usize;
    let len = slices[slice_idx + 1] as usize;
    read_field_entries(lang, start, len, 10_000)
}

pub(super) fn decode_fields_by_rule(lang: &TSLanguage) -> HashMap<u16, Vec<(FieldId, usize)>> {
    let mut fields_by_rule = HashMap::new();
    if lang.field_map_slices.is_null()
        || lang.field_map_entries.is_null()
        || lang.production_count == 0
    {
        return fields_by_rule;
    }

    let production_count = lang.production_count as usize;
    let slice_array_size = production_count.saturating_mul(2);
    if slice_array_size == 0 || slice_array_size > usize::MAX / 4 {
        return fields_by_rule;
    }

    // SAFETY: `lang.field_map_slices` is non-null (branch guard) and
    // `slice_array_size = production_count * 2`. Bounded by `usize::MAX / 4` check.
    let slices_array =
        unsafe { std::slice::from_raw_parts(lang.field_map_slices, slice_array_size) };

    for pid in 0..production_count {
        let slice_base = pid.saturating_mul(2);
        if slice_base + 1 >= slices_array.len() {
            continue;
        }

        let start = slices_array[slice_base] as usize;
        let len = slices_array[slice_base + 1] as usize;
        let entries = read_field_entries(lang, start, len, 1_000);
        if !entries.is_empty() {
            fields_by_rule.insert(pid as u16, entries);
        }
    }

    fields_by_rule
}

fn read_field_entries(
    lang: &TSLanguage,
    start: usize,
    len: usize,
    max_len: usize,
) -> Vec<(FieldId, usize)> {
    if len == 0 || len > max_len {
        return Vec::new();
    }

    let entry_count = start.saturating_add(len).saturating_mul(2);
    if entry_count > usize::MAX / 4 || start > entry_count {
        return Vec::new();
    }

    // TODO(safety): `entry_count` is derived from slice data which may not
    // match the true allocation size of `lang.field_map_entries`.
    let entries = unsafe { std::slice::from_raw_parts(lang.field_map_entries, entry_count) };
    let mut out = Vec::new();

    for j in 0..len {
        let entry_base = (start + j).saturating_mul(2);
        if entry_base + 1 < entries.len() {
            let low = entries[entry_base];
            let high = entries[entry_base + 1];
            let packed = ((high as u32) << 16) | (low as u32);
            let field_id = (packed & 0xFFFF) as u16;
            let child_index = ((packed >> 16) & 0xFF) as usize;
            out.push((FieldId(field_id), child_index));
        }
    }

    out
}
