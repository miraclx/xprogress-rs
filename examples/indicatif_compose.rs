//! Compose indicatif with xprogress. indicatif owns the terminal and draws a normal total-bytes bar
//! (bytes, rate, eta - all formatted by indicatif), while an xprogress segmented line rides one of its
//! lines as a message. This is how you get the full xprogress-js experience: indicatif for everything
//! it already does well, xprogress for the one thing it does not (several progresses on a single line).
//!
//! The bridge is that `Bar::render` returns a plain `String`, so neither side owns the other: xprogress
//! produces the segmented content, indicatif places it. Run it in a terminal:
//!
//!     cargo run --example indicatif_compose

use core::time::Duration;
use std::thread::sleep;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use xprogress::{Bar, Units};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    const MIB: u64 = 1024 * 1024;
    // One file fetched as three parallel ranges. This is the case segmented single-line progress is
    // built for: one file, one line, a segment per range, with buffered-ahead (lead) shading.
    let chunks = [40 * MIB, 35 * MIB, 25 * MIB];
    let total: u64 = chunks.iter().sum();

    let multi = MultiProgress::new();

    // A normal indicatif bar: total bytes, rate, and eta, formatted by indicatif.
    let overall = multi.add(ProgressBar::new(total));
    overall.set_style(ProgressStyle::with_template(
        "{bytes:>10} / {total_bytes:<10}  {bytes_per_sec:>12}  eta {eta}",
    )?);

    // The xprogress segmented line, carried as an indicatif message line. indicatif coordinates the
    // terminal; xprogress supplies the segmented content via render().
    let segmented = multi.add(ProgressBar::new_spinner());
    segmented.set_style(ProgressStyle::with_template("     chunks  [{msg}]")?);

    let mut bar = Bar::new(
        Units::new(chunks[0]),
        chunks[1..].iter().map(|&chunk| Units::new(chunk)),
    );
    let width = 42u16;

    let received_per_tick = [7 * MIB, 6 * MIB, 4 * MIB];
    let flushed_per_tick = [5 * MIB, 4 * MIB, 3 * MIB];
    let mut received = [0u64; 3];
    let mut flushed = [0u64; 3];

    loop {
        let mut complete = true;
        for index in 0..chunks.len() {
            received[index] = (received[index] + received_per_tick[index]).min(chunks[index]);
            flushed[index] = (flushed[index] + flushed_per_tick[index]).min(received[index]);
            if flushed[index] < chunks[index] {
                complete = false;
            }
            if let Some(segment) = bar.segment(index) {
                segment.set_done(Units::new(flushed[index]));
                segment.lead_to(Units::new(received[index]));
            }
        }
        overall.set_position(flushed.iter().sum());
        segmented.set_message(bar.render(width));
        if complete {
            break;
        }
        sleep(Duration::from_millis(90));
    }

    overall.finish();
    segmented.finish_with_message(bar.render(width));
    Ok(())
}
