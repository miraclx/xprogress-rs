//! A segmented single-line progress bar.

use core::fmt::Write as _;
use std::io;

use anstyle::Color;
use unicode_width::UnicodeWidthChar as _;

use crate::segment::Segment;
use crate::style::Style;
use crate::target::DrawTarget;
use crate::units::{Progress, Units};
use crate::width::Width;

/// Fallback width when the terminal size is unknown (piped or non-interactive output).
const FALLBACK_WIDTH: u16 = 80;

/// A single line of progress divided into independent [`Segment`]s, rendered side by side.
///
/// Build one from the segment sizes, advance segments as work happens, and [`Bar::draw`] to redraw it
/// in place. By default it draws to stderr at the terminal width and is silent when output is not a
/// terminal, so the common case is short:
///
/// ```no_run
/// # fn main() -> std::io::Result<()> {
/// use xprogress::Bar;
///
/// let mut bar = Bar::new([40u64, 30, 20]); // one segment per chunk, sizes in whatever unit you count
/// bar.draw()?;
/// bar.advance(0, 4096u64); // 4096 more units done on chunk 0
/// bar.draw()?;
/// bar.finish()?;
/// # Ok(()) }
/// ```
///
/// To place the bar inside another display (for example an indicatif `MultiProgress`), use
/// [`Bar::render`], which returns the line as a plain string and touches no terminal.
#[derive(Clone, Debug)]
pub struct Bar {
    segments: Vec<Segment>,
    style: Style,
    target: DrawTarget,
    width: Width,
}

impl Bar {
    /// Build a bar from its segment sizes. Draws to the terminal at full width by default.
    #[must_use]
    pub fn new(sizes: impl IntoIterator<Item = impl Into<Units>>) -> Self {
        Self {
            segments: sizes.into_iter().map(|s| Segment::new(s.into())).collect(),
            style: Style::default(),
            target: DrawTarget::from_env(),
            width: Width::Full,
        }
    }

    /// Use a specific drawing style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Color each segment in order, so adjacent segments are distinguishable by hue. Extra colors are
    /// ignored; segments beyond the colors keep the style default.
    #[must_use]
    pub fn with_segment_colors(
        mut self,
        colors: impl IntoIterator<Item = impl Into<Color>>,
    ) -> Self {
        for (segment, color) in self.segments.iter_mut().zip(colors) {
            segment.set_color(color);
        }
        self
    }

    /// Draw at a specific width instead of the full terminal.
    #[must_use]
    pub fn with_width(mut self, width: Width) -> Self {
        self.width = width;
        self
    }

    /// Draw to a specific target instead of the default stderr terminal.
    #[must_use]
    pub fn with_target(mut self, target: DrawTarget) -> Self {
        self.target = target;
        self
    }

    /// Advance segment `index`'s confirmed progress by `delta`. A no-op if `index` is out of range.
    pub fn advance(&mut self, index: usize, delta: impl Into<Units>) {
        if let Some(segment) = self.segments.get_mut(index) {
            segment.advance(delta.into());
        }
    }

    /// Move segment `index`'s buffered-ahead marker to `at`. A no-op if `index` is out of range.
    pub fn lead_to(&mut self, index: usize, at: impl Into<Units>) {
        if let Some(segment) = self.segments.get_mut(index) {
            segment.lead_to(at.into());
        }
    }

    /// Advance segment `index`'s buffered-ahead marker by `delta`, staying at or ahead of its confirmed
    /// progress. A no-op if `index` is out of range. Pair this with [`Bar::advance`] to shade bytes
    /// received (lead) apart from bytes confirmed (done): advance the lead as bytes arrive, advance the
    /// done as they are written.
    pub fn advance_lead(&mut self, index: usize, delta: impl Into<Units>) {
        if let Some(segment) = self.segments.get_mut(index) {
            segment.advance_lead(delta.into());
        }
    }

    /// Borrow a segment by position, for finer control than [`Bar::advance`].
    pub fn segment(&mut self, index: usize) -> Option<&mut Segment> {
        self.segments.get_mut(index)
    }

    /// The segments, in order.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Whether every segment has reached its capacity.
    pub fn is_complete(&self) -> bool {
        self.segments.iter().all(|segment| segment.is_complete())
    }

    /// The aggregate progress across all segments, summing done, lead, and capacity.
    pub fn aggregate(&self) -> Progress {
        let mut done = Units::new(0);
        let mut lead = Units::new(0);
        let mut capacity = Units::new(0);
        for segment in &self.segments {
            done = done.saturating_add(segment.done());
            lead = lead.saturating_add(segment.lead());
            capacity = capacity.saturating_add(segment.capacity());
        }
        Progress::new(done, lead, capacity)
    }

    /// Redraw the bar in place. A no-op when the target is not a terminal.
    pub fn draw(&mut self) -> io::Result<()> {
        let line = self.rendered();
        self.target.draw(&line)
    }

    /// Finish drawing, leaving the completed bar on its own line. A no-op when not a terminal.
    pub fn finish(&mut self) -> io::Result<()> {
        let line = self.rendered();
        self.target.finish(&line)
    }

    /// Render the whole bar's [`Bar::aggregate`] as a single two-tone bar of exactly `width` cells: one
    /// summary bar whose done and lead are the sums across every segment. Pair it with [`Bar::render`]
    /// to draw an aggregate header line above the per-segment line, as a chunked download does.
    #[must_use]
    pub fn render_aggregate(&self, width: u16) -> String {
        let progress = self.aggregate();
        let mut segment = Segment::new(progress.capacity());
        segment.set_done(progress.done());
        segment.lead_to(progress.lead());
        let mut out = String::new();
        render_segment(&segment, width, &self.style, &mut out);
        out
    }

    /// Render at the bar's own resolved width.
    fn rendered(&self) -> String {
        self.render(self.width.resolve(self.target.columns(), FALLBACK_WIDTH))
    }

    /// Render the bar as a string of exactly `width` display cells, segments side by side joined by the
    /// style separator. Pure: touches no terminal, so it composes inside other progress displays.
    #[must_use]
    pub fn render(&self, width: u16) -> String {
        if self.segments.is_empty() {
            // All empty, but still through push_run so blank_color applies as it does in a real segment.
            let mut out = String::new();
            push_run(
                &mut out,
                self.style.blank_color(),
                self.style.blank(),
                width,
            );
            return out;
        }
        let separator = self.style.separator();
        let gaps = (self.segments.len() - 1) as u16;
        let separators_width = separator.map_or(0, char_width).saturating_mul(gaps);
        let inner = width.saturating_sub(separators_width);
        let cells = allocate(&self.segments, inner);

        let mut out = String::new();
        for (index, (segment, span)) in self.segments.iter().zip(cells).enumerate() {
            if index > 0
                && let Some(glyph) = separator
            {
                out.push(glyph);
            }
            render_segment(segment, span, &self.style, &mut out);
        }
        out
    }
}

/// Divide `width` cells across the segments in proportion to capacity, using the largest-remainder
/// method so the returned spans always sum to exactly `width`.
fn allocate(segments: &[Segment], width: u16) -> Vec<u16> {
    let count = segments.len();
    if width == 0 {
        return vec![0; count];
    }
    let total: u128 = segments
        .iter()
        .map(|s| u128::from(s.capacity().get()))
        .sum();
    if total == 0 {
        return even_split(width, count);
    }

    let width = u128::from(width);
    let mut spans = Vec::with_capacity(count);
    let mut remainders = Vec::with_capacity(count);
    let mut assigned: u128 = 0;
    for segment in segments {
        let share = u128::from(segment.capacity().get()) * width;
        spans.push((share / total) as u16);
        remainders.push(share % total);
        assigned += share / total;
    }

    // Hand the leftover cells to the largest remainders, breaking ties toward earlier segments.
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by(|&a, &b| remainders[b].cmp(&remainders[a]).then(a.cmp(&b)));
    let mut leftover = (width - assigned) as u16;
    for &index in &order {
        if leftover == 0 {
            break;
        }
        spans[index] += 1;
        leftover -= 1;
    }
    spans
}

/// Split `width` evenly across `count` slots, giving the remainder to the earliest slots.
fn even_split(width: u16, count: usize) -> Vec<u16> {
    let count_u16 = count as u16;
    let mut spans = vec![width / count_u16; count];
    let mut extra = width % count_u16;
    for span in &mut spans {
        if extra == 0 {
            break;
        }
        *span += 1;
        extra -= 1;
    }
    spans
}

/// Append one segment's `span` cells to `out`: the filled region (done, capped by the header glyph,
/// then buffered-ahead) optionally colored, then the empty region. Color codes are zero display width,
/// so the segment is still exactly `span` cells wide.
fn render_segment(segment: &Segment, span: u16, style: &Style, out: &mut String) {
    if span == 0 {
        return;
    }
    let capacity = segment.capacity().get();
    let (done_cells, lead_cells) = if capacity == 0 {
        (span, span)
    } else {
        let done = scale(segment.done().get(), capacity, span);
        let lead = scale(segment.lead().get(), capacity, span).max(done);
        (done, lead)
    };

    // A per-segment color overrides the style's fill color; the lead falls back to the fill color when
    // it has no color of its own, so a glyph-distinguished bar stays single-hue as before.
    let done_color = segment.color().or(style.color());
    let lead_color = style.lead_color().or(done_color);

    // The header caps the leading edge of the whole filled region (done then buffered-ahead), so it
    // marks how far progress has reached. It only belongs while there is still something after it: a
    // segment filled edge to edge has no frontier, so it takes the plain glyph instead of the header.
    let head = style
        .header()
        .filter(|_| lead_cells > 0 && lead_cells < span);
    let body = if head.is_some() {
        lead_cells - 1
    } else {
        lead_cells
    };
    let done_body = done_cells.min(body);
    let lead_body = body - done_body;

    push_run(out, done_color, style.filler(), done_body);
    push_run(out, lead_color, style.leader(), lead_body);
    if let Some(header) = head {
        // The head sits on the lead frontier, so it takes the lead color; with no buffered-ahead cells
        // it caps the done region instead and takes the done color.
        let head_color = if lead_cells > done_cells {
            lead_color
        } else {
            done_color
        };
        push_run(out, head_color, header, 1);
    }
    push_run(out, style.blank_color(), style.blank(), span - lead_cells);
}

/// Append `count` copies of `glyph` to `out`, wrapped in `color`'s SGR codes when set. Color codes are
/// zero display width, so the run stays exactly `count` cells wide.
fn push_run(out: &mut String, color: Option<Color>, glyph: char, count: u16) {
    if count == 0 {
        return;
    }
    let style = color.map(|color| anstyle::Style::new().fg_color(Some(color)));
    if let Some(style) = style {
        let _ = write!(out, "{}", style.render());
    }
    for _ in 0..count {
        out.push(glyph);
    }
    if let Some(style) = style {
        let _ = write!(out, "{}", style.render_reset());
    }
}

/// Round `value / capacity * span` to the nearest cell, clamped to `0..=span`.
fn scale(value: u64, capacity: u64, span: u16) -> u16 {
    let value = u128::from(value.min(capacity));
    let capacity = u128::from(capacity);
    let span = u128::from(span);
    let cells = (value * span + capacity / 2) / capacity;
    cells.min(span) as u16
}

/// The display width of a glyph in terminal cells (zero for a control character).
fn char_width(glyph: char) -> u16 {
    glyph.width().unwrap_or(0) as u16
}
