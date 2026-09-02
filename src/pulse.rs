//! An indeterminate "pulsing" bar for work with no known total.

use std::io;

use crate::style::Style;
use crate::target::DrawTarget;
use crate::width::Width;

/// Fallback width when the terminal size is unknown.
const FALLBACK_WIDTH: u16 = 80;

/// An indeterminate progress bar: a fixed-length block that sweeps across the width and back, for work
/// whose total is unknown. Unlike a spinner it fills bar-shaped space; unlike a [`Bar`](crate::Bar) it
/// has no capacity. Advance a tick counter and [`Pulse::draw`] each frame to animate it. Rendering is
/// pure ([`Pulse::render`]): the same `(width, phase)` always yields the same line.
#[derive(Clone, Debug)]
pub struct Pulse {
    block: u16,
    style: Style,
    target: DrawTarget,
    width: Width,
}

impl Pulse {
    /// A pulse whose moving block is `block` cells long. Draws to the terminal at full width by default.
    #[must_use]
    pub fn new(block: u16) -> Self {
        Self {
            block,
            style: Style::default(),
            target: DrawTarget::from_env(),
            width: Width::Full,
        }
    }

    /// Use a specific drawing style. The block uses the filler glyph; the rest uses blank.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Draw at a specific width instead of the full terminal.
    #[must_use]
    pub fn with_width(mut self, width: Width) -> Self {
        self.width = width;
        self
    }

    /// Draw to a specific target instead of the default.
    #[must_use]
    pub fn with_target(mut self, target: DrawTarget) -> Self {
        self.target = target;
        self
    }

    /// Redraw the pulse in place for tick `phase`. A no-op when the target is not a terminal.
    pub fn draw(&mut self, phase: u64) -> io::Result<()> {
        let line = self.render(
            self.width.resolve(self.target.columns(), FALLBACK_WIDTH),
            phase,
        );
        self.target.draw(&line)
    }

    /// Finish, leaving `line` on its own line. A no-op when not a terminal.
    pub fn finish(&mut self, line: &str) -> io::Result<()> {
        self.target.finish(line)
    }

    /// Render at `width` cells for tick `phase`. The block's left edge sweeps from the left edge to the
    /// right and back, one cell per phase step, so the line is always exactly `width` cells.
    #[must_use]
    pub fn render(&self, width: u16, phase: u64) -> String {
        let block = self.block.min(width);
        let position = bounce(phase, width - block);
        let mut out = String::new();
        for _ in 0..position {
            out.push(self.style.blank());
        }
        for _ in 0..block {
            out.push(self.style.filler());
        }
        for _ in (position + block)..width {
            out.push(self.style.blank());
        }
        out
    }
}

/// A triangle wave: map `phase` to a position in `0..=travel`, reflecting at each end so the block
/// bounces rather than jumping back to the start.
fn bounce(phase: u64, travel: u16) -> u16 {
    if travel == 0 {
        return 0;
    }
    let span = u64::from(travel);
    let point = phase % (span * 2);
    let position = if point <= span {
        point
    } else {
        span * 2 - point
    };
    position as u16
}
