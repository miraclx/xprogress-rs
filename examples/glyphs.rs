//! A side-by-side gallery of bar glyph sets, every candidate drawn on the *same* progress so you can
//! compare weight and two-tone at a glance and pick one. Each set is shown twice: as a single aggregate
//! bar (done, buffered-ahead, empty, and the head cap all visible at once) and as a four-chunk bar (so
//! the separator and per-chunk look show too).
//!
//! Two ways to draw a two-tone bar (done vs buffered-ahead):
//!   - by glyph: done and lead use different glyphs (default, A, C). One color.
//!   - by color: done and lead share a glyph but differ by hue (B). This is what pip and rich do; it
//!     needs `with_lead_color` (and usually `with_blank_color` for the dim track).
//!
//!     cargo run --example glyphs

use xprogress::{AnsiColor, Bar, Style};

/// A labelled candidate: how it draws, and a one-line note on what it is.
struct Candidate {
    label: &'static str,
    note: &'static str,
    style: Style,
}

fn candidates() -> Vec<Candidate> {
    vec![
        Candidate {
            label: "default",
            note: "█ ▒ ─   the library default, for reference",
            style: Style::default(),
        },
        Candidate {
            label: "current",
            note: "━ ┅ ╸   what xget draws today (solid vs triple-dash: the weight clash)",
            style: Style::default()
                .with_filler('━')
                .with_leader('┅')
                .with_blank('┅')
                .with_header('╸')
                .with_separator('┆')
                .with_color(AnsiColor::BrightCyan),
        },
        Candidate {
            label: "A light",
            note: "─ ┄ ╴   thin light family, consistent weight, drop-in",
            style: Style::default()
                .with_filler('─')
                .with_leader('┄')
                .with_blank('┄')
                .with_header('╴')
                .with_separator('┆')
                .with_color(AnsiColor::BrightCyan),
        },
        Candidate {
            label: "B pip",
            note: "━ ━ ╸   color-only two-tone (pip/rich): done bright, lead dim, gray track",
            style: Style::default()
                .with_filler('━')
                .with_leader('━')
                .with_blank('━')
                .with_header('╸')
                .with_separator('┃')
                .with_color(AnsiColor::BrightCyan)
                .with_lead_color(AnsiColor::Cyan)
                .with_blank_color(AnsiColor::BrightBlack),
        },
        Candidate {
            label: "C heavy",
            note: "━ ╍ ╾   heavy, double-dash lead (dash weight closer to solid), drop-in",
            style: Style::default()
                .with_filler('━')
                .with_leader('╍')
                .with_blank('╌')
                .with_header('╾')
                .with_separator('┋')
                .with_color(AnsiColor::BrightCyan),
        },
    ]
}

fn main() {
    let width = 44u16;

    println!("\nsame data every row: 45% done, buffered ahead to 70%.\n");
    for c in candidates() {
        // One aggregate bar: done, buffered-ahead, head, and empty all on show.
        let mut bar = Bar::new([100u64]).with_style(c.style.clone());
        bar.advance(0, 45u64);
        bar.lead_to(0, 70u64);
        println!("  {:<9}{}", c.label, bar.render(width));
        println!("  {:<9}{}\n", "", dim(c.note));
    }

    // A realistic in-flight scatter: early chunks confirmed, later ones still buffering ahead.
    let chunks = [30u64, 30, 20, 20];
    let fills = [(30u64, 30u64), (20, 27), (5, 15), (0, 4)];
    println!("as four chunks (shows the separator and per-chunk shading):\n");
    for c in candidates() {
        let mut bar = Bar::new(chunks).with_style(c.style.clone());
        for (index, &(done, lead)) in fills.iter().enumerate() {
            bar.advance(index, done);
            bar.lead_to(index, lead);
        }
        println!("  {:<9}{}", c.label, bar.render(width));
    }
    println!();
}

/// Wrap `text` in a dim SGR so the note reads as secondary.
fn dim(text: &str) -> String {
    format!("\x1b[2m{text}\x1b[0m")
}
