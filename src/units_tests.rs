use crate::units::{Percent, Units};

#[test]
fn percent_of_zero_capacity_is_complete() {
    assert_eq!(Percent::of(Units::new(0), Units::new(0)).get(), 100.0);
}

#[test]
fn percent_of_half() {
    assert_eq!(Percent::of(Units::new(50), Units::new(100)).get(), 50.0);
}

#[test]
fn percent_clamps_an_overfull_value() {
    assert_eq!(Percent::of(Units::new(150), Units::new(100)).get(), 100.0);
}

#[test]
fn saturating_add_does_not_wrap() {
    assert_eq!(
        Units::new(u64::MAX).saturating_add(Units::new(1)),
        Units::new(u64::MAX)
    );
}
