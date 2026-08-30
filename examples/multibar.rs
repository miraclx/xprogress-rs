//! Compose indicatif with xprogress. indicatif owns the terminal and draws a normal total-bytes bar
//! (bytes, rate, eta, all formatted by indicatif), while an xprogress segmented line rides one of its
//! lines as a message. indicatif is a dev-dependency only; the bridge is `Bar::render` returning a
//! plain string, so neither side owns the other. Run it in a terminal:
//!
//!     cargo run --example indicatif_compose

use core::time::Duration;
use std::thread::sleep;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use xprogress::Bar;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    const MIB: u64 = 1024 * 1024;
    let chunks = [40 * MIB, 35 * MIB, 25 * MIB];
    let total: u64 = chunks.iter().sum();

    let multi = MultiProgress::new();

    let overall = multi.add(ProgressBar::new(total));
    overall.set_style(ProgressStyle::with_template(
        "{bytes:>10} / {total_bytes:<10} {bytes_per_sec:>12} eta {eta}",
    )?);

    let segmented = multi.add(ProgressBar::new_spinner());
    segmented.set_style(ProgressStyle::with_template("     chunks  [{msg}]")?);

    let mut bar = Bar::new(chunks);
    while !bar.is_complete() {
        for (chunk, &size) in chunks.iter().enumerate() {
            bar.advance(chunk, size / 20 + 1);
        }
        overall.set_position(bar.aggregate().done().get());
        segmented.set_message(bar.render(42));
        sleep(Duration::from_millis(80));
    }

    overall.finish();
    segmented.finish_with_message(bar.render(42));
    Ok(())
}
