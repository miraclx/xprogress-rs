use crate::target::DrawTarget;

#[test]
fn hidden_target_never_draws() {
    let mut target = DrawTarget::hidden();
    assert!(!target.is_interactive());
    assert_eq!(target.columns(), None);
    target.draw("anything").unwrap();
    target.finish("done").unwrap();
}
