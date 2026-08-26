use crate::width::Width;

#[test]
fn fixed_is_independent_of_terminal() {
    assert_eq!(Width::Fixed(40).resolve(Some(120), 80), 40);
    assert_eq!(Width::Fixed(40).resolve(None, 80), 40);
}

#[test]
fn full_uses_columns_then_fallback() {
    assert_eq!(Width::Full.resolve(Some(120), 80), 120);
    assert_eq!(Width::Full.resolve(None, 80), 80);
}

#[test]
fn terminal_minus_subtracts_saturates_and_falls_back() {
    assert_eq!(Width::TerminalMinus(20).resolve(Some(120), 80), 100);
    assert_eq!(
        Width::TerminalMinus(200).resolve(Some(120), 80),
        0,
        "saturates"
    );
    assert_eq!(Width::TerminalMinus(20).resolve(None, 80), 80, "falls back");
}
