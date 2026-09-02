//! A gallery of the built-in [`Style`] presets, all filling in step so you can compare them *animating*
//! together rather than as static swatches. Each row is one preset drawn on the same progress (done
//! trailing the buffered-ahead lead, so the two-tone shows), redrawing in place.
//!
//!     cargo run --example styles
//!
//! To see the frames without a terminal (or to capture them), dump each frame on its own line:
//!
//!     XPROGRESS_DUMP=1 cargo run --example styles

use core::time::Duration;
use std::thread::sleep;

use xprogress::{AnsiColor, Bar, DrawTarget, Style};

/// The presets on show, each with a fixed-width label. `line` and `braille` are given the two-tone cyan
/// so the color-carried styles read as intended; `block` and `ascii` distinguish by glyph alone.
fn presets() -> Vec<(&'static str, Style)> {
    vec![
        ("block  ", Style::default()),
        ("ascii  ", Style::ascii()),
        (
            "line   ",
            Style::line()
                .with_color(AnsiColor::BrightCyan)
                .with_lead_color(AnsiColor::Cyan)
                .with_blank_color(AnsiColor::BrightBlack),
        ),
        (
            "braille",
            Style::braille()
                .with_color(AnsiColor::BrightCyan)
                .with_lead_color(AnsiColor::Cyan)
                .with_blank_color(AnsiColor::BrightBlack),
        ),
    ]
}

fn main() -> std::io::Result<()> {
    let presets = presets();
    let width = 40u16;
    let mut target = DrawTarget::from_env();
    let mut prev = 0usize;

    // Rebuild each row from the current fill every frame, so there is no per-bar delta bookkeeping: the
    // buffered-ahead lead runs ahead and the confirmed done trails it by a fixed gap.
    let block = |pct: u64| {
        let mut out = String::new();
        for (label, style) in &presets {
            let mut bar = Bar::new([100u64]).with_style(Style::clone(style));
            bar.advance(0, pct);
            bar.lead_to(0, pct);
            out.push_str(&format!("  {label}  {}  {pct:>3}%\n", bar.render(width)));
        }
        out.trim_end().to_owned()
    };

    for step in 0..=50u64 {
        let frame = block((step * 2).min(100));
        if let Ok(lines) = target.draw_block(&frame, prev) {
            prev = lines;
        }
        sleep(Duration::from_millis(60));
    }
    target.finish_block(&block(100), prev)
}
