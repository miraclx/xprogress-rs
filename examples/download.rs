//! A simulated multi-file download: three files as segments of one line, each showing bytes flushed
//! to disk (done) trailing bytes received into buffer (lead).
//!
//! Run it in a terminal to watch it animate:
//!
//!     cargo run --example download

use core::time::Duration;
use std::thread::sleep;

use xprogress::{Bar, DrawTarget, Units, Width};

fn main() -> std::io::Result<()> {
    const MIB: u64 = 1024 * 1024;
    let files = [
        ("kernel.iso", 120 * MIB),
        ("rootfs.tar", 80 * MIB),
        ("initrd.img", 20 * MIB),
    ];
    // Download (lead) outruns disk flush (done), at different rates per file.
    let download = [14 * MIB, 9 * MIB, 6 * MIB];
    let flush = [9 * MIB, 6 * MIB, 4 * MIB];

    let mut bar = Bar::new(
        Units::new(files[0].1),
        files[1..].iter().map(|file| Units::new(file.1)),
    );
    let mut target = DrawTarget::stderr();
    let width = Width::TerminalMinus(2).resolve(target.columns(), 60);

    let mut received = [0u64; 3];
    let mut flushed = [0u64; 3];
    loop {
        let mut complete = true;
        for index in 0..files.len() {
            received[index] = (received[index] + download[index]).min(files[index].1);
            flushed[index] = (flushed[index] + flush[index]).min(received[index]);
            if flushed[index] < files[index].1 {
                complete = false;
            }
            if let Some(segment) = bar.segment(index) {
                segment.set_done(Units::new(flushed[index]));
                segment.lead_to(Units::new(received[index]));
            }
        }
        target.draw(&format!("[{}]", bar.render(width)))?;
        if complete {
            break;
        }
        sleep(Duration::from_millis(80));
    }
    target.finish(&format!("[{}]  done", bar.render(width)))
}
