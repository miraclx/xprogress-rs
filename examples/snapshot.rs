//! Print static frames of a segmented bar. Unlike `download`/`stream`, this draws nothing in place: it
//! just prints each frame with `println!`, so it works with no terminal (piped, logged, or in CI) and
//! shows the segmented multibar and its done-versus-lead shading as plain text.
//!
//!     cargo run --example snapshot

use xprogress::{Bar, Units};

fn main() {
    // Three segments (say three ranges of one download), sized by capacity.
    let caps = [40u64, 35, 25];
    println!("segmented bar filling: solid is confirmed (done), light is buffered-ahead (lead)\n");
    for step in 0..=5u32 {
        let fraction = f64::from(step) / 5.0;
        let mut bar = Bar::new(
            Units::new(caps[0]),
            caps[1..].iter().map(|&cap| Units::new(cap)),
        );
        for (index, segment) in bar.segments_mut().enumerate() {
            let cap = caps[index];
            let done = (cap as f64 * fraction) as u64;
            let lead = (done + cap / 6).min(cap);
            segment.set_done(Units::new(done));
            segment.lead_to(Units::new(lead));
        }
        println!("  [{}] {:>3}%", bar.render(48), (fraction * 100.0) as u32);
    }
}
