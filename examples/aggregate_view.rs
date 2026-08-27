//! A chunked-download view: an aggregate header bar over a per-chunk bar, both two-tone (done vs
//! buffered-ahead lead). This is exactly what a parallel downloader wants, and all xprogress provides
//! is two pure render calls; the caller owns the readout text (it knows the speed and ETA).
//!
//! Run: `cargo run --example aggregate_view`

use xprogress::{AnsiColor, Bar, Style};

fn main() {
    // One segment per chunk, sized in bytes.
    let sizes = [520_000u64, 500_000, 500_000, 500_000, 500_000];
    let mut bar = Bar::new(sizes).with_style(
        Style::default()
            .with_color(AnsiColor::Cyan)
            .with_separator('┆'),
    );

    // As a download runs, each chunk receives bytes (lead) ahead of what the reassembler has written
    // and hashed (done). Feed both: advance_lead on receipt, advance on write.
    let received = [520_000u64, 480_000, 300_000, 260_000, 120_000];
    let written = [520_000u64, 300_000, 180_000, 120_000, 40_000];
    for index in 0..sizes.len() {
        bar.advance_lead(index, received[index]);
        bar.advance(index, written[index]);
    }

    let width = 56;
    let aggregate = bar.render_aggregate(width);
    let chunks = bar.render(width);

    // The caller composes the borders and the readout it alone knows (percent/speed/eta, sizes).
    println!("  ┏ {aggregate} ┓  47%  1.2 MiB/s  0:03");
    println!("  ┗ {chunks} ┛  1.13 MiB / 2.41 MiB");
}
