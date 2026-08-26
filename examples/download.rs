//! A download client pulling one file in parallel chunks, which is what segmented progress is built
//! for: one segment per chunk, advanced as its bytes arrive. Run it in a terminal to watch it animate:
//!
//!     cargo run --example download
//!
//! To see the frames without a terminal, dump them:
//!
//!     XPROGRESS_DUMP=1 cargo run --example download

use core::time::Duration;
use std::thread::sleep;

use xprogress::Bar;

fn main() -> std::io::Result<()> {
    const MIB: u64 = 1024 * 1024;
    let chunks = [40 * MIB, 30 * MIB, 20 * MIB, 10 * MIB];

    let mut bar = Bar::new(chunks);
    bar.draw()?;

    // Bytes arrive per chunk at different rates until every chunk is whole.
    let rates = [7 * MIB, 5 * MIB, 4 * MIB, 2 * MIB];
    while !bar.is_complete() {
        for (chunk, &rate) in rates.iter().enumerate() {
            bar.advance(chunk, rate);
        }
        bar.draw()?;
        sleep(Duration::from_millis(80));
    }
    bar.finish()
}
