//! The glyphs a [`Bar`](crate::Bar) draws with.

/// The characters a bar renders: confirmed fill, buffered-ahead lead, empty cell, and the separator
/// drawn between segments. Construct with [`Style::default`] and adjust with the `with_*` builders.
#[derive(Clone, Debug)]
pub struct Style {
    filler: char,
    leader: char,
    blank: char,
    separator: String,
}

impl Style {
    /// Set the glyph for confirmed (done) cells.
    #[must_use]
    pub fn with_filler(mut self, glyph: char) -> Self {
        self.filler = glyph;
        self
    }

    /// Set the glyph for buffered-ahead (lead) cells, drawn between done and lead.
    #[must_use]
    pub fn with_leader(mut self, glyph: char) -> Self {
        self.leader = glyph;
        self
    }

    /// Set the glyph for empty cells.
    #[must_use]
    pub fn with_blank(mut self, glyph: char) -> Self {
        self.blank = glyph;
        self
    }

    /// Set the separator drawn between adjacent segments. Its display width is reserved out of the
    /// bar width so the rendered line stays exact.
    #[must_use]
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// The confirmed-fill glyph.
    pub const fn filler(&self) -> char {
        self.filler
    }

    /// The buffered-ahead glyph.
    pub const fn leader(&self) -> char {
        self.leader
    }

    /// The empty-cell glyph.
    pub const fn blank(&self) -> char {
        self.blank
    }

    /// The separator drawn between segments.
    pub fn separator(&self) -> &str {
        &self.separator
    }
}

impl Default for Style {
    /// Solid fill, a lighter lead, a dash for empty, and no separator.
    fn default() -> Self {
        Self {
            filler: '█',
            leader: '▒',
            blank: '─',
            separator: String::new(),
        }
    }
}
