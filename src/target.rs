//! Where a [`Bar`](crate::Bar) draws: an interactive terminal, or a hidden no-op sink.

use std::io::{self, IsTerminal as _, Write as _};

/// Where a bar is drawn. Non-interactive output is a first-class [`DrawTarget::Hidden`] variant, not a
/// runtime error: drawing to it is a no-op, so piping or redirecting a program's output just works and
/// the progress state stays queryable.
pub enum DrawTarget {
    /// An interactive terminal, redrawn in place on stderr.
    Terminal,
    /// A non-interactive sink that draws nothing.
    Hidden,
}

impl DrawTarget {
    /// Draw to stderr when it is a terminal, otherwise be [`DrawTarget::Hidden`], so progress never
    /// corrupts piped or redirected output. Stderr keeps stdout clean for a program's real output.
    #[must_use]
    pub fn stderr() -> Self {
        if io::stderr().is_terminal() {
            Self::Terminal
        } else {
            Self::Hidden
        }
    }

    /// A target that never draws.
    #[must_use]
    pub fn hidden() -> Self {
        Self::Hidden
    }

    /// Whether this target draws anything.
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Terminal)
    }

    /// The terminal width in columns, if this is an interactive terminal.
    pub fn columns(&self) -> Option<u16> {
        match self {
            Self::Terminal => terminal_size::terminal_size().map(|(width, _)| width.0),
            Self::Hidden => None,
        }
    }

    /// Redraw `line` in place: return to column zero, clear to the end of the line, write, and flush.
    /// A no-op on a hidden target.
    pub fn draw(&mut self, line: &str) -> io::Result<()> {
        match self {
            Self::Terminal => {
                let mut stderr = io::stderr().lock();
                write!(stderr, "\r\x1b[K{line}")?;
                stderr.flush()
            }
            Self::Hidden => Ok(()),
        }
    }

    /// Finish drawing: write `tail` and a newline so the final bar stays on screen. A no-op on hidden.
    pub fn finish(&mut self, tail: &str) -> io::Result<()> {
        match self {
            Self::Terminal => {
                let mut stderr = io::stderr().lock();
                writeln!(stderr, "{tail}")?;
                stderr.flush()
            }
            Self::Hidden => Ok(()),
        }
    }
}
