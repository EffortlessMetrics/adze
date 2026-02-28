use adze::field_tree::{
    ParsedNode as RuntimeParsedNode, Point as RuntimePoint, TSLanguage as RuntimeTSLanguage,
};
use adze_field_tree_core::{
    ParsedNode as CoreParsedNode, Point as CorePoint, TSLanguage as CoreTSLanguage,
};

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_point(point: CorePoint) -> CorePoint {
        point
    }

    let runtime_point = RuntimePoint::new(3, 9);
    let returned = accepts_core_point(runtime_point);
    assert_eq!(returned.row, 3);
    assert_eq!(returned.column, 9);
}

#[test]
fn runtime_reexport_exposes_same_node_and_language_types() {
    let language = CoreTSLanguage {
        field_names: vec!["left", "right"],
        symbol_names: vec!["ERROR", "expr"],
        production_field_map: vec![vec![Some(0), Some(1)]],
    };

    let core = CoreParsedNode {
        symbol: 1,
        children: Vec::new(),
        start_byte: 0,
        end_byte: 0,
        start_point: CorePoint::new(0, 0),
        end_point: CorePoint::new(0, 0),
        is_extra: false,
        is_error: false,
        is_missing: false,
        is_named: true,
        language: None,
    };

    let runtime: RuntimeParsedNode = core;
    assert_eq!(runtime.kind(&language), "expr");

    let runtime_lang: RuntimeTSLanguage = language;
    assert_eq!(runtime_lang.field_name(1), Some("right"));
}
