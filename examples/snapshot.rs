//! Print static frames of a segmented bar. Unlike `download`/`stream`, this draws nothing in place: it
//! just prints each frame with `println!`, so it works with no terminal (piped, logged, or in CI) and
//! shows the segmented multibar and its done-versus-lead shading as plain text.
//!
//!     cargo run --example snapshot

use xprogress::{Bar, Style, Units};

fn main() {
    // Three segments (say three ranges of one download), sized by capacity.
    let caps = [40u64, 35, 25];
    println!("segmented bar filling: solid is confirmed (done), light is buffered-ahead (lead)\n");
    for step in 0..=5u32 {
        let fraction = f64::from(step) / 5.0;
        let mut bar = fresh(&caps);
        fill(&mut bar, &caps, fraction);
        println!("  [{}] {:>3}%", bar.render(48), (fraction * 100.0) as u32);
    }

    // The same three segments, with a visible separator between them. Its width is reserved out of the
    // bar width, so the rendered line stays exactly 48 cells.
    println!("\nsame segments at 60%, with a separator between them:\n");
    let mut bar = fresh(&caps).with_style(Style::default().with_separator("│"));
    fill(&mut bar, &caps, 0.6);
    println!("  [{}]", bar.render(48));
}

fn fresh(caps: &[u64]) -> Bar {
    Bar::new(
        Units::new(caps[0]),
        caps[1..].iter().map(|&cap| Units::new(cap)),
    )
}

fn fill(bar: &mut Bar, caps: &[u64], fraction: f64) {
    for (index, segment) in bar.segments_mut().enumerate() {
        let cap = caps[index];
        let done = (cap as f64 * fraction) as u64;
        let lead = (done + cap / 6).min(cap);
        segment.set_done(Units::new(done));
        segment.lead_to(Units::new(lead));
    }
}
