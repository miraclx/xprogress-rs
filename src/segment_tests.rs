use crate::segment::Segment;
use crate::units::Units;

#[test]
fn new_segment_has_no_progress() {
    let segment = Segment::new(Units::new(100));
    assert_eq!(segment.done(), Units::new(0));
    assert_eq!(segment.lead(), Units::new(0));
    assert_eq!(segment.capacity(), Units::new(100));
}

#[test]
fn advance_accumulates_and_clamps_to_capacity() {
    let mut segment = Segment::new(Units::new(100));
    segment.advance(Units::new(30));
    segment.advance(Units::new(90));
    assert_eq!(segment.done(), Units::new(100));
    assert!(segment.is_complete());
}

#[test]
fn advance_pulls_lead_up_to_done() {
    let mut segment = Segment::new(Units::new(100));
    segment.advance(Units::new(40));
    assert_eq!(segment.lead(), Units::new(40));
}

#[test]
fn lead_to_clamps_between_done_and_capacity() {
    let mut segment = Segment::new(Units::new(100));
    segment.set_done(Units::new(30));
    segment.lead_to(Units::new(10));
    assert_eq!(
        segment.lead(),
        Units::new(30),
        "below done clamps up to done"
    );
    segment.lead_to(Units::new(80));
    assert_eq!(segment.lead(), Units::new(80));
    segment.lead_to(Units::new(200));
    assert_eq!(
        segment.lead(),
        Units::new(100),
        "above capacity clamps down"
    );
}

#[test]
fn set_done_beyond_capacity_clamps() {
    let mut segment = Segment::new(Units::new(50));
    segment.set_done(Units::new(999));
    assert_eq!(segment.done(), Units::new(50));
}

#[test]
fn zero_capacity_segment_is_complete() {
    assert!(Segment::new(Units::new(0)).is_complete());
}
