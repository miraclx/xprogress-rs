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

    /// The terminal height in rows, if this is an interactive terminal. Mirrors [`DrawTarget::columns`]
    /// for the vertical axis: `Some` on a live terminal, `None` when there is no terminal to probe.
    pub fn rows(&self) -> Option<u16> {
        match self {
            Self::Terminal => terminal_size::terminal_size().map(|(_, height)| height.0),
            Self::Hidden | Self::Dump => None,
        }
    }

    /// Show `line` as the current frame: redraw in place on a terminal, print a new line when dumping,
    /// nothing when hidden.
    pub fn draw(&mut self, line: &str) -> io::Result<()> {
        self.emit(line, 1, false)
    }

    /// Redraw a possibly multi-line `block` in place. `previous` is how many lines the last call drew
    /// (`0` for the first), and the return value is how many this call drew, to pass back next time. On
    /// a terminal it moves up to the top of the previous block and clears to the end of the screen, so a
    /// block that grows or shrinks between frames still redraws cleanly. Dumping and hidden targets
    /// ignore `previous` and behave like [`DrawTarget::draw`].
    pub fn draw_block(&mut self, block: &str, previous: usize) -> io::Result<usize> {
        self.emit(block, previous, false)?;
        Ok(block.lines().count().max(1))
    }

    /// Finish a multi-line block: redraw it in place like [`DrawTarget::draw_block`], then commit it
    /// with a trailing newline so the completed block stays on screen.
    pub fn finish_block(&mut self, block: &str, previous: usize) -> io::Result<()> {
        self.emit(block, previous, true)
    }

    /// Finish drawing, leaving `tail` on its own line. Nothing when hidden.
    pub fn finish(&mut self, tail: &str) -> io::Result<()> {
        self.emit(tail, 1, true)
    }

    /// The one terminal write behind every draw method: move to the top of the `previous`-line block,
    /// clear to the end of the screen (so a block that shrank leaves no remnant, and a narrowed terminal
    /// no stale wrap), write `body`, and commit it with a trailing newline when `commit`. Hidden writes
    /// nothing; Dump prints `body` on its own line, ignoring `previous` and `commit`.
    fn emit(&mut self, body: &str, previous: usize, commit: bool) -> io::Result<()> {
        match self {
            Self::Terminal => {
                let mut stderr = io::stderr().lock();
                if previous > 1 {
                    write!(stderr, "\x1b[{}A", previous - 1)?;
                }
                write!(stderr, "\r\x1b[J{body}")?;
                if commit {
                    writeln!(stderr)?;
                }
                stderr.flush()
            }
            Self::Hidden => Ok(()),
            Self::Dump => {
                let mut stdout = io::stdout().lock();
                writeln!(stdout, "{body}")?;
                stdout.flush()
            }
        }
    }
}
