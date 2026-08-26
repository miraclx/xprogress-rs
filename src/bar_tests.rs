use crate::bar::Bar;
use crate::style::Style;
use crate::units::Units;

fn cells(line: &str) -> usize {
    line.chars().count()
}

fn count(line: &str, glyph: char) -> usize {
    line.chars().filter(|&c| c == glyph).count()
}

#[test]
fn render_is_exactly_the_requested_width() {
    let bar = Bar::new(Units::new(120), [Units::new(80)]);
    assert_eq!(cells(&bar.render(40)), 40);
    assert_eq!(cells(&bar.render(1)), 1);
    assert_eq!(cells(&bar.render(0)), 0);
}

#[test]
fn single_segment_half_done_is_half_filled() {
    let mut bar = Bar::single(Units::new(100));
    bar.segments_mut().for_each(|s| s.advance(Units::new(50)));
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 5);
    assert_eq!(count(&line, '─'), 5);
}

#[test]
fn lead_shades_between_done_and_lead() {
    let mut bar = Bar::single(Units::new(100));
    bar.segments_mut().for_each(|s| {
        s.set_done(Units::new(30));
        s.lead_to(Units::new(70));
    });
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 3, "done");
    assert_eq!(count(&line, '▒'), 4, "buffered ahead");
    assert_eq!(count(&line, '─'), 3, "empty");
}

#[test]
fn empty_segments_render_all_blank() {
    let bar = Bar::new(Units::new(50), [Units::new(50)]);
    let line = bar.render(10);
    assert_eq!(cells(&line), 10);
    assert_eq!(count(&line, '─'), 10);
}

#[test]
fn allocation_is_proportional_to_capacity() {
    let mut bar = Bar::new(Units::new(300), [Units::new(100)]);
    bar.segments_mut().for_each(|s| s.set_done(s.capacity()));
    let line = bar.render(40);
    assert_eq!(
        count(&line, '█'),
        40,
        "3:1 caps fully done fill all 40 cells"
    );
}

#[test]
fn separator_is_reserved_from_the_width() {
    let bar =
        Bar::new(Units::new(50), [Units::new(50)]).with_style(Style::default().with_separator("|"));
    let line = bar.render(11);
    assert_eq!(cells(&line), 11);
    assert_eq!(count(&line, '|'), 1);
    assert_eq!(count(&line, '─'), 10);
}

#[test]
fn zero_capacity_segment_renders_complete() {
    let line = Bar::single(Units::new(0)).render(8);
    assert_eq!(count(&line, '█'), 8);
}

#[test]
fn aggregate_sums_all_segments() {
    let mut bar = Bar::new(Units::new(100), [Units::new(100)]);
    bar.segment(0).unwrap().set_done(Units::new(40));
    bar.segment(1).unwrap().set_done(Units::new(60));
    let aggregate = bar.aggregate();
    assert_eq!(aggregate.done(), Units::new(100));
    assert_eq!(aggregate.capacity(), Units::new(200));
    assert_eq!(aggregate.percent().get(), 50.0);
}
