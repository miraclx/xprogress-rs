use crate::segment::Segment;
use crate::stream::{Mark, Meter};
use crate::units::Units;

#[test]
fn meter_tracks_done_and_lead_independently() {
    let meter = Meter::new(Units::new(100));
    meter.advance(Mark::Lead, Units::new(70));
    meter.advance(Mark::Done, Units::new(40));
    let snapshot = meter.snapshot();
    assert_eq!(snapshot.done(), Units::new(40));
    assert_eq!(snapshot.lead(), Units::new(70));
}

#[test]
fn snapshot_clamps_to_capacity_and_keeps_lead_at_least_done() {
    let meter = Meter::new(Units::new(50));
    meter.set(Mark::Done, Units::new(80));
    meter.set(Mark::Lead, Units::new(10));
    let snapshot = meter.snapshot();
    assert_eq!(snapshot.done(), Units::new(50), "done clamps to capacity");
    assert_eq!(snapshot.lead(), Units::new(50), "lead rises to done");
}

#[test]
fn apply_to_updates_a_segment() {
    let meter = Meter::new(Units::new(100));
    meter.advance(Mark::Done, Units::new(25));
    meter.advance(Mark::Lead, Units::new(60));
    let mut segment = Segment::new(Units::new(100));
    meter.apply_to(&mut segment);
    assert_eq!(segment.done(), Units::new(25));
    assert_eq!(segment.lead(), Units::new(60));
}

#[test]
fn a_clone_shares_the_same_counters() {
    let meter = Meter::new(Units::new(100));
    let other = Meter::clone(&meter);
    other.advance(Mark::Done, Units::new(30));
    assert_eq!(meter.snapshot().done(), Units::new(30));
}
