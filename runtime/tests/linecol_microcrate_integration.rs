use adze::linecol::LineCol as RuntimeLineCol;
use adze_linecol_core::LineCol as CoreLineCol;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let input = b"one\r\ntwo\nthree\rfour";

    for position in 0..=input.len() {
        let runtime = RuntimeLineCol::at_position(input, position);
        let core = CoreLineCol::at_position(input, position);
        assert_eq!(runtime, core, "position={position}");
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreLineCol) -> CoreLineCol {
        value
    }

    let runtime_value = RuntimeLineCol::new();
    let returned = accepts_core_type(runtime_value);
    assert_eq!(returned.line, 0);
    assert_eq!(returned.line_start, 0);
}
