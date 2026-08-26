//! How wide a [`Bar`](crate::Bar) should render, resolved against the terminal.

/// A width policy, resolved to a concrete cell count against the terminal's column count. An enum, so
/// every legal policy is a distinct case rather than an overloaded number.
#[derive(Clone, Copy, Debug)]
pub enum Width {
    /// An exact number of cells, independent of the terminal.
    Fixed(u16),
    /// The full terminal width.
    Full,
    /// The full terminal width less a fixed margin, leaving room for surrounding text.
    TerminalMinus(u16),
}

impl Width {
    /// Resolve to a concrete cell count. `columns` is the terminal width when known; a terminal that
    /// is not interactive reports `None`, and terminal-relative policies then use `fallback`.
    #[must_use]
    pub fn resolve(self, columns: Option<u16>, fallback: u16) -> u16 {
        match self {
            Width::Fixed(cells) => cells,
            Width::Full => columns.unwrap_or(fallback),
            Width::TerminalMinus(margin) => {
                columns.map_or(fallback, |cols| cols.saturating_sub(margin))
            }
        }
    }
}
