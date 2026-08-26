//! An indeterminate pulsing bar for work with no known total: a block that sweeps back and forth. Run
//! it in a terminal to watch it animate:
//!
//!     cargo run --example pulsate
//!
//! To see the frames without a terminal, dump them:
//!
//!     XPROGRESS_DUMP=1 cargo run --example pulsate

use core::time::Duration;
use std::thread::sleep;

use xprogress::Pulse;

fn main() -> std::io::Result<()> {
    let mut pulse = Pulse::new(12);
    // Indeterminate work: sweep while it runs. Here we stop after a fixed number of ticks.
    for phase in 0..70u64 {
        pulse.draw(phase)?;
        sleep(Duration::from_millis(60));
    }
    pulse.finish("done")
}
