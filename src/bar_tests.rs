use crate::bar::Bar;
use crate::style::{Color, Style};
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
    let bar = Bar::new([50u64, 50]).with_style(Style::default().with_separator("|"));
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
fn color_adds_ansi_but_keeps_the_display_width() {
    let mut bar = Bar::new([100u64]).with_style(Style::default().with_color(Color::Cyan));
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
