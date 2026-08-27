//! The glyphs and color a [`Bar`](crate::Bar) draws with.

/// An ANSI foreground color for the filled portion of a bar.
#[derive(Clone, Copy, Debug)]
pub enum Color {
    /// ANSI red.
    Red,
    /// ANSI green.
    Green,
    /// ANSI yellow.
    Yellow,
    /// ANSI blue.
    Blue,
    /// ANSI magenta.
    Magenta,
    /// ANSI cyan.
    Cyan,
    /// ANSI white.
    White,
    /// Bright (high-intensity) black, usually rendered as gray.
    BrightBlack,
    /// Bright red.
    BrightRed,
    /// Bright green.
    BrightGreen,
    /// Bright yellow.
    BrightYellow,
    /// Bright blue.
    BrightBlue,
    /// Bright magenta.
    BrightMagenta,
    /// Bright cyan.
    BrightCyan,
    /// Bright white.
    BrightWhite,
}

impl Color {
    /// The SGR parameter for this color, as a static string so emitting it needs no allocation.
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::BrightBlack => "90",
            Color::BrightRed => "91",
            Color::BrightGreen => "92",
            Color::BrightYellow => "93",
            Color::BrightBlue => "94",
            Color::BrightMagenta => "95",
            Color::BrightCyan => "96",
            Color::BrightWhite => "97",
        }
    }
}

/// The glyphs and optional color a bar renders with. Construct with [`Style::default`] and adjust with
/// the `with_*` builders. The filled and buffered-ahead cells take the color; empty cells stay plain.
#[derive(Clone, Debug)]
pub struct Style {
    filler: char,
    leader: char,
    blank: char,
    header: Option<char>,
    separator: Option<char>,
    color: Option<Color>,
    lead_color: Option<Color>,
    blank_color: Option<Color>,
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

    /// Set a glyph to cap the leading edge of the filled region (the bar's "head"), like `>` in
    /// `===>`. Off by default.
    #[must_use]
    pub fn with_header(mut self, glyph: char) -> Self {
        self.header = Some(glyph);
        self
    }

    /// Color the confirmed (done) cells. Buffered-ahead cells take the same color unless
    /// [`Style::with_lead_color`] sets its own, and empty cells stay uncolored unless
    /// [`Style::with_blank_color`] sets one.
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Color the buffered-ahead (lead) cells apart from the confirmed cells, so a two-tone bar can
    /// distinguish done from buffered by hue alone (as pip and rich do) rather than by glyph. Falls back
    /// to [`Style::with_color`] when unset.
    #[must_use]
    pub fn with_lead_color(mut self, color: Color) -> Self {
        self.lead_color = Some(color);
        self
    }

    /// Color the empty cells, for a dim track behind the fill (as pip and rich draw). Uncolored by
    /// default.
    #[must_use]
    pub fn with_blank_color(mut self, color: Color) -> Self {
        self.blank_color = Some(color);
        self
    }

    /// Set the glyph drawn between adjacent segments. Its display width is reserved out of the bar
    /// width so the rendered line stays exact.
    #[must_use]
    pub fn with_separator(mut self, glyph: char) -> Self {
        self.separator = Some(glyph);
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

    /// The leading-edge cap glyph, if set.
    pub const fn header(&self) -> Option<char> {
        self.header
    }

    /// The fill color, if set.
    pub const fn color(&self) -> Option<Color> {
        self.color
    }

    /// The buffered-ahead color, if set apart from the fill color.
    pub const fn lead_color(&self) -> Option<Color> {
        self.lead_color
    }

    /// The empty-cell color, if set.
    pub const fn blank_color(&self) -> Option<Color> {
        self.blank_color
    }

    /// The separator glyph drawn between segments, if any.
    pub const fn separator(&self) -> Option<char> {
        self.separator
    }
}

impl Default for Style {
    /// Solid fill, a lighter lead, a dash for empty, no header, no separator, no color.
    fn default() -> Self {
        Self {
            filler: '█',
            leader: '▒',
            blank: '─',
            header: None,
            separator: None,
            color: None,
            lead_color: None,
            blank_color: None,
        }
    }
}
