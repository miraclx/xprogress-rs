//! The one thing indicatif and linya cannot show: a segment tracks **two** frontiers at once — `done`
//! (confirmed) and `lead` (received but not yet confirmed) — as a two-tone shade. In a download, `lead`
//! is bytes off the wire and `done` is bytes verified, so the shaded gap between them is the buffer
//! flying ahead of verification. This example makes that gap the whole point: the lead surges, and the
//! confirmed `done` chases it, with a legend so the two tones are named.
//!
//!     cargo run --example buffered

use core::time::Duration;
use std::thread::sleep;

use xprogress::{AnsiColor, Bar, DrawTarget, Style};

fn main() -> std::io::Result<()> {
    // Solid bright for confirmed, a dim shade for received-but-unconfirmed, a plain track for the rest.
    let style = Style::default()
        .with_filler('█')
        .with_leader('▒')
        .with_blank('─')
        .with_color(AnsiColor::Green)
        .with_lead_color(AnsiColor::BrightBlack);

    let legend = "  \x1b[32m█\x1b[0m verified   \x1b[90m▒\x1b[0m received, not yet verified   ─ not yet received";
    let mut target = DrawTarget::from_env();
    let mut prev = 0usize;

    // Each frame the lead runs ahead; done trails it by a fixed gap, so the ▒ buffered window is always
    // on show and visibly travels down the bar as verification catches up.
    let frame = |lead: u64| {
        let done = lead.saturating_sub(24);
        let mut bar = Bar::new([100u64]).with_style(style.clone());
        bar.advance(0, done);
        bar.lead_to(0, lead);
        format!(
            "  {}   {done:>3} verified · {lead:>3} received\n{legend}",
            bar.render(50)
        )
    };

    for step in 0..=52u64 {
        let block = frame((step * 2).min(100));
        prev = target.draw_block(&block, prev)?;
        sleep(Duration::from_millis(70));
    }
    target.finish_block(&frame(100), prev)
}
