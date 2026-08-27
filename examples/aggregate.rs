//! The per-chunk segments and the whole-download bar, together. `Bar::aggregate()` sums the segments
//! into the overall progress, and the bar splits the line across segments itself, so a caller never
//! hand-computes the overall percentage or the per-segment widths. Each segment gets its own color, so
//! you can tell them apart. Prints frames, so it works with no terminal.
//!
//!     cargo run --example aggregate

use xprogress::{Bar, Color, Style};

fn main() {
    let chunks = [40u64, 30, 20, 10];
    let palette = [Color::Cyan, Color::Green, Color::Yellow, Color::Magenta];
    println!("per-chunk segments (each its own color), and the aggregate the library computes:\n");
    for step in 1..=4u64 {
        let mut bar = Bar::new(chunks).with_segment_colors(palette);
        for (index, &cap) in chunks.iter().enumerate() {
            let done = (cap * step + index as u64 * 3) / 4;
            bar.advance(index, done.min(cap));
        }

        // The whole-download progress, computed from the segments; no manual math.
        let overall = bar.aggregate();
        let mut whole = Bar::new([overall.capacity().get()])
            .with_style(Style::default().with_color(Color::Cyan));
        whole.advance(0, overall.done().get());

        println!("  segments  [{}]", bar.render(40));
        println!(
            "  whole     [{}] {:>3}%\n",
            whole.render(40),
            overall.percent().get() as u32
        );
    }
}
