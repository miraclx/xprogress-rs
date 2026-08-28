//! The chunked-download view, animated: an aggregate header bar over a per-chunk bar, both two-tone
//! (confirmed vs buffered-ahead). This is what a parallel downloader wants, and xprogress does the two
//! things that are annoying by hand: `aggregate()` sums the segments into the overall progress (so the
//! header percent needs no manual math), and the per-chunk bar splits one line across the chunks itself.
//! The caller owns only the borders and the readout text it alone knows (speed, ETA). Run it in a
//! terminal to watch it animate:
//!
//!     cargo run --example aggregate
//!
//! To see the frames without a terminal, dump them:
//!
//!     XPROGRESS_DUMP=1 cargo run --example aggregate

use core::time::Duration;
use std::thread::sleep;

use xprogress::{AnsiColor, Bar, DrawTarget, Style};

fn main() -> std::io::Result<()> {
    const KB: u64 = 1000;
    // One segment per chunk, sized in bytes.
    let chunks = [520 * KB, 500 * KB, 500 * KB, 500 * KB, 500 * KB];

    // One heavy line, the regions told apart by hue: bright for written-and-hashed, dimmer for received
    // ahead, gray for the rest, with a light separator between chunks.
    let style = Style::default()
        .with_filler('━')
        .with_leader('━')
        .with_blank('━')
        .with_header('╸')
        .with_separator('┆')
        .with_color(AnsiColor::BrightCyan)
        .with_lead_color(AnsiColor::Cyan)
        .with_blank_color(AnsiColor::BrightBlack);
    let mut bar = Bar::new(chunks).with_style(style);

    // As a download runs, each chunk receives bytes (lead) ahead of what the reassembler has written and
    // hashed (done). Feed both, at different per-chunk rates: advance_lead on receipt, advance on write.
    let receive = [34 * KB, 30 * KB, 26 * KB, 22 * KB, 18 * KB];
    let write = [30 * KB, 24 * KB, 20 * KB, 16 * KB, 12 * KB];

    let mut target = DrawTarget::from_env();
    let width = 52;
    let mut previous = 0;
    loop {
        let block = frame(&bar, width);
        if bar.is_complete() {
            return target.finish_block(&block, previous);
        }
        previous = target.draw_block(&block, previous)?;
        for (index, (&received, &written)) in receive.iter().zip(&write).enumerate() {
            bar.advance_lead(index, received);
            bar.advance(index, written);
        }
        sleep(Duration::from_millis(90));
    }
}

/// Compose the two-line block: the aggregate header over the per-chunk bar, with the borders and the
/// readout the caller owns. The header percent and the byte totals come from `aggregate()`, so there is
/// no manual overall math.
fn frame(bar: &Bar, width: u16) -> String {
    let overall = bar.aggregate();
    let pct = overall.percent().get() as u32;
    let done = overall.done().get() / 1000;
    let total = overall.capacity().get() / 1000;
    format!(
        "  ┏ {} ┓  {pct:>3}%\n  ┗ {} ┛  {done}/{total} KB",
        bar.render_aggregate(width),
        bar.render(width),
    )
}
