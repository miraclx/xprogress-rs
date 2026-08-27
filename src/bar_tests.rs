use anstyle::AnsiColor;

use crate::bar::Bar;
use crate::style::Style;
use crate::units::Units;

fn strip_ansi(line: &str) -> String {
    let mut out = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for esc in chars.by_ref() {
                if esc == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn cells(line: &str) -> usize {
    line.chars().count()
}

fn count(line: &str, glyph: char) -> usize {
    line.chars().filter(|&c| c == glyph).count()
}

#[test]
fn render_is_exactly_the_requested_width() {
    let bar = Bar::new([120u64, 80]);
    assert_eq!(cells(&bar.render(40)), 40);
    assert_eq!(cells(&bar.render(1)), 1);
    assert_eq!(cells(&bar.render(0)), 0);
}

#[test]
fn a_half_done_segment_is_half_filled() {
    let mut bar = Bar::new([100u64]);
    bar.advance(0, 50u64);
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 5);
    assert_eq!(count(&line, '─'), 5);
}

#[test]
fn lead_shades_between_done_and_lead() {
    let mut bar = Bar::new([100u64]);
    bar.advance(0, 30u64);
    bar.lead_to(0, 70u64);
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 3, "done");
    assert_eq!(count(&line, '▒'), 4, "buffered ahead");
    assert_eq!(count(&line, '─'), 3, "empty");
}

#[test]
fn unstarted_segments_render_all_blank() {
    let line = Bar::new([50u64, 50]).render(10);
    assert_eq!(cells(&line), 10);
    assert_eq!(count(&line, '─'), 10);
}

#[test]
fn allocation_is_proportional_to_capacity() {
    let mut bar = Bar::new([300u64, 100]);
    bar.advance(0, 300u64);
    bar.advance(1, 100u64);
    assert_eq!(
        count(&bar.render(40), '█'),
        40,
        "3:1 caps fully done fill all 40 cells"
    );
}

#[test]
fn advance_on_an_out_of_range_segment_is_a_no_op() {
    let mut bar = Bar::new([100u64]);
    bar.advance(9, 50u64);
    assert!(!bar.segments()[0].is_complete());
}

#[test]
fn separator_is_reserved_from_the_width() {
    let bar = Bar::new([50u64, 50]).with_style(Style::default().with_separator('|'));
    let line = bar.render(11);
    assert_eq!(cells(&line), 11);
    assert_eq!(count(&line, '|'), 1);
    assert_eq!(count(&line, '─'), 10);
}

#[test]
fn a_bar_with_no_segments_renders_blank() {
    let bar = Bar::new(core::iter::empty::<u64>());
    assert_eq!(cells(&bar.render(6)), 6);
    assert_eq!(count(&bar.render(6), '─'), 6);
}

#[test]
fn aggregate_sums_all_segments() {
    let mut bar = Bar::new([100u64, 100]);
    bar.advance(0, 40u64);
    bar.advance(1, 60u64);
    let aggregate = bar.aggregate();
    assert_eq!(aggregate.done(), Units::new(100));
    assert_eq!(aggregate.capacity(), Units::new(200));
    assert_eq!(aggregate.percent().get(), 50.0);
}

#[test]
fn lead_shades_the_buffered_region_between_done_and_empty() {
    let mut bar = Bar::new([100u64]);
    bar.advance(0, 30u64); // 30% confirmed
    bar.advance_lead(0, 30u64); // buffered ahead to 60%
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 3, "three cells done");
    assert_eq!(count(&line, '▒'), 3, "three cells buffered-ahead");
    assert_eq!(count(&line, '─'), 4, "four cells empty");
}

#[test]
fn render_aggregate_sums_done_and_lead_across_segments() {
    let mut bar = Bar::new([100u64, 100]);
    bar.advance(0, 50u64); // segment 0: 50 done
    bar.advance_lead(1, 50u64); // segment 1: 50 buffered ahead, 0 done
    // Aggregate: done 50, lead 100, capacity 200.
    let line = bar.render_aggregate(20);
    assert_eq!(cells(&line), 20, "exactly the requested width");
    assert_eq!(count(&line, '█'), 5, "done is 50/200");
    assert_eq!(count(&line, '▒'), 5, "lead adds another 50/200");
    assert_eq!(count(&line, '─'), 10, "the remaining half is empty");
}

#[test]
fn color_adds_ansi_but_keeps_the_display_width() {
    let mut bar = Bar::new([100u64]).with_style(Style::default().with_color(AnsiColor::Cyan));
    bar.advance(0, 50u64);
    let line = bar.render(10);
    assert!(line.contains("\x1b[36m"), "cyan code present");
    assert!(line.contains("\x1b[0m"), "reset present");
    assert_eq!(cells(&strip_ansi(&line)), 10, "still exactly ten cells");
}

#[test]
fn header_caps_the_filled_region() {
    let mut bar = Bar::new([100u64]).with_style(Style::default().with_header('>'));
    bar.advance(0, 50u64);
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 4, "four solid");
    assert_eq!(count(&line, '>'), 1, "one header tip");
    assert_eq!(count(&line, '─'), 5, "five empty");
}

#[test]
fn the_header_caps_the_lead_frontier_not_the_done_frontier() {
    // With buffered-ahead progress past done, the header marks the leading edge of the whole filled
    // region (done then lead), not the done edge, so it sits at the lead frontier.
    let mut bar = Bar::new([100u64]).with_style(Style::default().with_header('>'));
    bar.advance(0, 30u64); // 3 cells done
    bar.lead_to(0, 60u64); // buffered ahead to 6 cells
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 3, "three cells confirmed");
    assert_eq!(count(&line, '▒'), 2, "buffered-ahead body up to the cap");
    assert_eq!(count(&line, '>'), 1, "the header caps the lead frontier");
    assert_eq!(count(&line, '─'), 4, "four cells empty past the frontier");
}

#[test]
fn a_full_segment_has_no_header_cap() {
    let mut bar = Bar::new([100u64]).with_style(Style::default().with_header('>'));
    bar.advance(0, 100u64);
    let line = bar.render(10);
    assert_eq!(count(&line, '█'), 10, "filled edge to edge");
    assert_eq!(
        count(&line, '>'),
        0,
        "no cap once there is no frontier left"
    );
}

#[test]
fn lead_color_tints_the_buffered_region_apart_from_done() {
    // Color-only two-tone (as pip and rich draw): done and lead share a glyph but differ by hue, so the
    // bar shows both a bright done color and a distinct lead color at once.
    let mut bar = Bar::new([100u64]).with_style(
        Style::default()
            .with_filler('━')
            .with_leader('━')
            .with_color(AnsiColor::BrightCyan)
            .with_lead_color(AnsiColor::Cyan),
    );
    bar.advance(0, 30u64); // done
    bar.lead_to(0, 60u64); // buffered ahead
    let line = bar.render(10);
    assert!(line.contains("\x1b[96m"), "bright cyan for done cells");
    assert!(line.contains("\x1b[36m"), "plain cyan for the lead cells");
    assert_eq!(count(&strip_ansi(&line), '━'), 6, "six filled cells total");
    assert_eq!(cells(&strip_ansi(&line)), 10, "still exactly ten cells");
}

#[test]
fn blank_color_tints_the_empty_track() {
    let mut bar =
        Bar::new([100u64]).with_style(Style::default().with_blank_color(AnsiColor::BrightBlack));
    bar.advance(0, 40u64);
    let line = bar.render(10);
    assert!(line.contains("\x1b[90m"), "gray track behind the fill");
    assert_eq!(cells(&strip_ansi(&line)), 10, "still exactly ten cells");
}

#[test]
fn a_segment_color_overrides_the_bar_color() {
    let mut bar = Bar::new([50u64, 50]).with_style(Style::default().with_color(AnsiColor::Cyan));
    bar.segment(0).unwrap().set_color(AnsiColor::Magenta);
    bar.advance(0, 50u64);
    bar.advance(1, 50u64);
    let line = bar.render(10);
    assert!(
        line.contains("\x1b[35m"),
        "segment 0 uses its magenta override"
    );
    assert!(
        line.contains("\x1b[36m"),
        "segment 1 uses the cyan bar default"
    );
}
