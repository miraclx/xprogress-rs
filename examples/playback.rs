//! A media player's timeline, which is what confirmed-versus-buffered shading is built for: `done` is
//! the playhead (what you have watched) and `lead` is the buffer (what has downloaded ahead). The buffer
//! races ahead in bursts and now and then stalls; the playhead advances a steady second at a time but
//! never past the buffer. The shaded gap between the two is the buffered-but-unwatched span a video
//! scrubber shows. Run it in a terminal to watch it animate:
//!
//!     cargo run --example playback
//!
//! To see the frames without a terminal, dump them:
//!
//!     XPROGRESS_DUMP=1 cargo run --example playback

use core::time::Duration;
use std::thread::sleep;

use xprogress::{AnsiColor, Bar, Style};

fn main() -> std::io::Result<()> {
    // A four-minute clip, counted in seconds.
    const LENGTH: u64 = 240;

    // One heavy line, the regions told apart by hue: bright for watched, a dimmer cyan for buffered
    // ahead, a gray track for what has not downloaded yet.
    let style = Style::default()
        .with_filler('━')
        .with_leader('━')
        .with_blank('━')
        .with_color(AnsiColor::BrightCyan)
        .with_lead_color(AnsiColor::Cyan)
        .with_blank_color(AnsiColor::BrightBlack);
    let mut bar = Bar::new([LENGTH]).with_style(style);
    bar.draw()?;

    for tick in 0u64.. {
        let done = bar.segments()[0].done().get();
        if done >= LENGTH {
            break;
        }
        // The buffer fills ahead in bursts, stalling two ticks out of every seven; over time it stays
        // well ahead, but a stall lets the playhead close the gap.
        let fill: u64 = if tick % 7 < 2 { 0 } else { 4 };
        bar.advance_lead(0, fill);
        // The playhead advances a steady second, but only while there is buffer ahead of it: catching up
        // to the buffer is a rebuffering pause.
        if done < bar.segments()[0].lead().get() {
            bar.advance(0, 1u64);
        }
        bar.draw()?;
        sleep(Duration::from_millis(60));
    }
    bar.finish()
}
