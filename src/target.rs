//! Where a bar draws: an interactive terminal, a hidden no-op, or a frame dump.

use std::io::{self, IsTerminal as _, Write as _};

/// Where a bar is drawn. Non-interactive output is a first-class state, not a runtime error: drawing
/// to [`DrawTarget::Hidden`] does nothing and drawing to [`DrawTarget::Dump`] prints each frame on its
/// own line, so piping, redirecting, logging, and headless capture all just work.
#[derive(Clone, Copy, Debug)]
pub enum DrawTarget {
    /// An interactive terminal, redrawn in place on stderr.
    Terminal,
    /// A non-interactive sink that draws nothing.
    Hidden,
    /// Non-interactive: print each frame on its own line to stdout, for logs, CI, or headless capture.
    Dump,
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

    /// A target that prints each frame on its own line to stdout.
    #[must_use]
    pub fn dump() -> Self {
        Self::Dump
    }

    /// [`DrawTarget::Dump`] when the `XPROGRESS_DUMP` environment variable is set (for logs and
    /// headless capture), otherwise [`DrawTarget::stderr`]. This is the default a [`Bar`](crate::Bar)
    /// and [`Pulse`](crate::Pulse) use.
    #[must_use]
    pub fn from_env() -> Self {
        if std::env::var_os("XPROGRESS_DUMP").is_some() {
            Self::Dump
        } else {
            Self::stderr()
        }
    }

    /// Whether this target redraws in place.
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Terminal)
    }

    /// The terminal width in columns, if this is an interactive terminal.
    pub fn columns(&self) -> Option<u16> {
        match self {
            Self::Terminal => terminal_size::terminal_size().map(|(width, _)| width.0),
            Self::Hidden | Self::Dump => None,
        }
    }

    /// Show `line` as the current frame: redraw in place on a terminal, print a new line when dumping,
    /// nothing when hidden.
    pub fn draw(&mut self, line: &str) -> io::Result<()> {
        match self {
            Self::Terminal => {
                let mut stderr = io::stderr().lock();
                write!(stderr, "\r\x1b[K{line}")?;
                stderr.flush()
            }
            Self::Hidden => Ok(()),
            Self::Dump => {
                let mut stdout = io::stdout().lock();
                writeln!(stdout, "{line}")?;
                stdout.flush()
            }
        }
    }

    /// Finish drawing, leaving `tail` on its own line. Nothing when hidden.
    pub fn finish(&mut self, tail: &str) -> io::Result<()> {
        match self {
            Self::Terminal => {
                let mut stderr = io::stderr().lock();
                writeln!(stderr, "{tail}")?;
                stderr.flush()
            }
            Self::Hidden => Ok(()),
            Self::Dump => {
                let mut stdout = io::stdout().lock();
                writeln!(stdout, "{tail}")?;
                stdout.flush()
            }
        }
    }
}
