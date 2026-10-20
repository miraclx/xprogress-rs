use anstyle::AnsiColor;
use unicode_width::UnicodeWidthStr as _;

use crate::bar::Bar;
use crate::style::Style;
use crate::units::Units;

#[test]
fn a_wide_glyph_filler_is_ignored_so_the_width_stays_exact() {
    // A wide emoji cannot be a cell: it would overflow the bar's exact width, so with_filler ignores it
    // and the default glyph stands. The rendered width holds in display columns.
    let mut bar = Bar::new([100u64]).with_style(Style::default().with_filler('🚀'));
    bar.advance(0, 100u64);
    let line = strip_ansi(&bar.render(10));
    assert_eq!(
        line.chars().count(),
        10,
        "width holds; the wide glyph was rejected"
    );
    assert!(!line.contains('🚀'), "the wide glyph never entered the bar");
}

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
    line.width()
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
fn separators_are_omitted_together_only_when_they_cannot_fit() {
    for (separator, expected) in [
        (
            '|',
            ["", "█", "||", "█||", "█|▒|", "█|▒|─", "██|▒|─", "██|▒▒|─"],
        ),
        (
            '界',
            ["", "█", "█▒", "█▒─", "界界", "█界界", "█界▒界", "█界▒界─"],
        ),
        (
            '\u{17d8}',
            [
                "",
                "█",
                "█▒",
                "█▒─",
                "██▒─",
                "██▒▒─",
                "\u{17d8}\u{17d8}",
                "█\u{17d8}\u{17d8}",
            ],
        ),
    ] {
        let mut bar = Bar::new([1u64; 3]).with_style(Style::default().with_separator(separator));
        bar.advance(0, 1u64);
        bar.advance_lead(1, 1u64);
        for (width, expected) in expected.into_iter().enumerate() {
            let line = bar.render(width as u16);
            assert_eq!(line, expected, "separator {separator:?}, width {width}");
            assert_eq!(cells(&line), width);
        }
    }
}

#[test]
fn separator_omission_preserves_weighted_allocation_and_earlier_ties() {
    let mut bar = Bar::new([1u64, 3, 0]).with_style(Style::default().with_separator('界'));
    bar.advance(0, 1u64);
    bar.advance_lead(1, 3u64);
    assert_eq!(bar.render(1), "▒", "the largest remainder gets the cell");
    assert_eq!(
        bar.render(2),
        "█▒",
        "equal remainders favor the earlier segment"
    );
    assert_eq!(bar.render(3), "█▒▒", "zero capacity gets no weighted share");
}

#[test]
fn control_line_break_and_zero_width_separators_leave_the_previous_style_unchanged() {
    let invalid = ('\0'..='\u{1f}')
        .chain('\u{7f}'..='\u{9f}')
        .chain(['\u{2028}', '\u{2029}', '\u{301}', '\u{200d}', '\u{fe0f}']);
    for glyph in invalid {
        assert_eq!(
            Style::default().with_separator(glyph).separator(),
            None,
            "{glyph:?}"
        );
        let style = Style::default().with_separator('界').with_separator(glyph);
        assert_eq!(style.separator(), Some('界'), "{glyph:?}");
        let line = Bar::new([1u64; 3]).with_style(style).render(7);
        assert_eq!(line, "─界─界─", "{glyph:?}");
    }
}

#[test]
fn empty_single_and_many_segments_keep_exact_cell_widths_with_separators_and_colors() {
    for sizes in [
        Vec::<u64>::new(),
        vec![0],
        vec![7],
        vec![0; 5],
        vec![1, 3, 0, 2, 7],
    ] {
        for separator in ['|', '界', '\n', '\u{301}'] {
            let mut bar = Bar::new(sizes.iter().copied()).with_style(
                Style::default()
                    .with_separator(separator)
                    .with_color(AnsiColor::Cyan)
                    .with_lead_color(AnsiColor::Yellow)
                    .with_blank_color(AnsiColor::BrightBlack)
                    .with_header('>'),
            );
            for (index, capacity) in sizes.iter().copied().enumerate() {
                bar.advance(index, capacity / 3);
                bar.lead_to(index, capacity.saturating_mul(2) / 3);
            }
            for width in (0..=32).chain([u16::MAX]) {
                let line = strip_ansi(&bar.render(width));
                assert_eq!(
                    cells(&line),
                    usize::from(width),
                    "{sizes:?}, {separator:?}, {width}"
                );
                assert!(!line.chars().any(char::is_control), "{line:?}");
                if sizes.len() <= 1 {
                    assert!(!line.contains(separator));
                }
                let aggregate = strip_ansi(&bar.render_aggregate(width));
                assert_eq!(cells(&aggregate), usize::from(width));
            }
        }
    }
}

#[test]
fn zero_capacity_splits_give_remainder_cells_to_earlier_segments() {
    let bar = Bar::new([0u64; 3]).with_segment_colors([
        AnsiColor::Magenta,
        AnsiColor::Cyan,
        AnsiColor::Yellow,
    ]);
    assert_eq!(bar.render(1), "\x1b[35m█\x1b[0m");
    assert_eq!(bar.render(2), "\x1b[35m█\x1b[0m\x1b[36m█\x1b[0m");
    assert_eq!(
        bar.render(4),
        "\x1b[35m██\x1b[0m\x1b[36m█\x1b[0m\x1b[33m█\x1b[0m"
    );
}

#[test]
fn zero_capacity_segment_count_does_not_narrow_at_65_536() {
    let bar = Bar::new(core::iter::repeat_n(0u64, 65_536));
    assert_eq!(bar.render(1), "█");
}

#[test]
fn segment_counts_at_and_above_u16_max_keep_exact_widths() {
    for count in [65_535, 65_536, 65_537] {
        for separator in [None, Some('|'), Some('界')] {
            let style = separator.map_or_else(Style::default, |glyph| {
                Style::default().with_separator(glyph)
            });
            let bar = Bar::new(core::iter::repeat_n(0u64, count)).with_style(style);
            for width in [0, 1, 2, 3, u16::MAX] {
                let line = bar.render(width);
                assert_eq!(
                    cells(&line),
                    usize::from(width),
                    "{count}, {separator:?}, {width}"
                );
                if width == 1 {
                    assert_eq!(line, "█", "zero-capacity work is complete");
                }
            }
        }
    }
}

#[test]
fn high_nonzero_segment_counts_do_not_truncate_separator_budgets_or_weighted_shares() {
    let mut bar = Bar::new(core::iter::repeat_n(1u64, 65_537))
        .with_style(Style::default().with_separator('|'));
    bar.advance(0, 1u64);
    bar.advance_lead(1, 1u64);
    assert_eq!(bar.render(0), "");
    assert_eq!(bar.render(1), "█");
    assert_eq!(bar.render(2), "█▒");
    assert_eq!(cells(&bar.render(u16::MAX)), usize::from(u16::MAX));
}

#[test]
fn ascii_style_uses_plain_glyphs() {
    let mut bar = Bar::new([100u64]).with_style(Style::ascii());
    bar.advance(0, 50u64);
    let line = bar.render(10);
    assert_eq!(cells(&line), 10);
    assert!(
        count(&line, '#') >= 4,
        "confirmed cells use '#', got {line:?}"
    );
    assert!(count(&line, '-') >= 4, "empty cells use '-', got {line:?}");
    assert!(
        line.is_ascii(),
        "the whole ascii bar stays ascii, got {line:?}"
    );
}

#[test]
fn line_and_braille_presets_render_their_glyphs() {
    let mut line = Bar::new([100u64]).with_style(Style::line());
    line.advance(0, 50u64);
    let rendered = line.render(10);
    assert!(
        rendered.contains('━'),
        "the line preset uses the heavy glyph, got {rendered:?}"
    );

    let mut braille = Bar::new([100u64]).with_style(Style::braille());
    braille.advance(0, 50u64);
    let rendered = braille.render(10);
    assert!(
        rendered.contains('⣿'),
        "braille done cells are full dots, got {rendered:?}"
    );
    assert!(
        rendered.contains('⣀'),
        "braille empty track is dotted, got {rendered:?}"
    );
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
