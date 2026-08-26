//! Segments driven by live async byte streams: each segment is advanced by a `SegmentReader` as it
//! drains a throttled source, and the bar renders from the shared meters. Run in a terminal:
//!
//!     cargo run --example stream --features stream
//!
//! To see the frames without a terminal:
//!
//!     XPROGRESS_DUMP=1 cargo run --example stream --features stream

use core::time::Duration;

use tokio::io::AsyncReadExt as _;
use xprogress::stream::{Mark, Meter, SegmentReader};
use xprogress::{Bar, Units};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let totals = [3_000_000u64, 2_000_000, 1_000_000];
    let meters: Vec<Meter> = totals.iter().map(|&t| Meter::new(Units::new(t))).collect();

    // One task per source, each draining a throttled reader through a SegmentReader that advances its
    // meter. In a real download these would be network sockets.
    for (index, &total) in totals.iter().enumerate() {
        let meter = Meter::clone(&meters[index]);
        tokio::spawn(async move {
            let source = tokio::io::repeat(1u8).take(total);
            let mut reader = SegmentReader::new(source, meter, Mark::Done);
            let mut buf = [0u8; 64 * 1024];
            while reader.read(&mut buf).await? != 0 {
                tokio::time::sleep(Duration::from_millis(15)).await;
            }
            std::io::Result::Ok(())
        });
    }

    let mut bar = Bar::new(totals);
    bar.draw()?;
    while !bar.is_complete() {
        for (index, meter) in meters.iter().enumerate() {
            if let Some(segment) = bar.segment(index) {
                meter.apply_to(segment);
            }
        }
        bar.draw()?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    bar.finish()
}
