use adze_linecol_core::LineCol;

#[test]
fn contract_line_starts_remain_stable_across_crlf_boundaries() {
    let input = b"a\r\nb";

    let before_cr = LineCol::at_position(input, 1);
    let after_cr_before_lf = LineCol::at_position(input, 2);
    let after_lf = LineCol::at_position(input, 3);

    assert_eq!((before_cr.line, before_cr.line_start), (0, 0));
    assert_eq!(
        (after_cr_before_lf.line, after_cr_before_lf.line_start),
        (0, 0)
    );
    assert_eq!((after_lf.line, after_lf.line_start), (1, 3));
}

#[test]
fn contract_default_and_new_are_equivalent() {
    assert_eq!(LineCol::default(), LineCol::new());
}
