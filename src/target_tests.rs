use crate::target::DrawTarget;

#[test]
fn hidden_target_never_draws() {
    let mut target = DrawTarget::hidden();
    assert!(!target.is_interactive());
    assert_eq!(target.columns(), None);
    target.draw("anything").unwrap();
    target.finish("done").unwrap();
}

#[test]
fn draw_block_reports_its_line_count() {
    let mut target = DrawTarget::hidden();
    assert_eq!(target.draw_block("one line", 0).unwrap(), 1);
    assert_eq!(target.draw_block("first\nsecond", 1).unwrap(), 2);
    assert_eq!(
        target.draw_block("", 2).unwrap(),
        1,
        "an empty block still counts as one line"
    );
    target.finish_block("first\nsecond", 2).unwrap();
}
