//! A segmented single-line progress bar.

use unicode_width::UnicodeWidthStr as _;

use crate::segment::Segment;
use crate::style::Style;
use crate::units::{Progress, Units};

/// A single line of progress divided into independent [`Segment`]s, rendered side by side.
///
/// A bar always has at least one segment: [`Bar::new`] takes the first capacity separately so an
/// empty bar cannot be constructed. Each segment receives a share of the width proportional to its
/// capacity. [`Bar::render`] returns a plain string of an exact cell width; the bar never writes to a
/// terminal itself, so a caller owns when and where drawing happens.
#[derive(Clone, Debug)]
pub struct Bar {
    /// Always non-empty, guaranteed by every constructor.
    segments: Vec<Segment>,
    style: Style,
}

impl Bar {
    /// Build a bar from the first segment capacity plus any further capacities. Taking the first
    /// separately makes an empty bar unrepresentable.
    #[must_use]
    pub fn new(first: Units, rest: impl IntoIterator<Item = Units>) -> Self {
        let mut segments = vec![Segment::new(first)];
        segments.extend(rest.into_iter().map(Segment::new));
        Self {
            segments,
            style: Style::default(),
        }
    }

    /// Build a single-segment bar.
    #[must_use]
    pub fn single(capacity: Units) -> Self {
        Self {
            segments: vec![Segment::new(capacity)],
            style: Style::default(),
        }
    }

    /// Replace the drawing style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The segments, in order.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Mutably borrow every segment, to advance them.
    pub fn segments_mut(&mut self) -> impl Iterator<Item = &mut Segment> {
        self.segments.iter_mut()
    }

    /// Mutably borrow one segment by position, if it exists.
    pub fn segment(&mut self, index: usize) -> Option<&mut Segment> {
        self.segments.get_mut(index)
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

    /// Render the bar as a string of exactly `width` display cells (with the default single-width
    /// glyphs), segments composited side by side and joined by the style separator.
    ///
    /// The separators' display width is reserved first; the remaining width is divided across
    /// segments in proportion to capacity, with rounding slack handed to the segments with the
    /// largest remainders so the line is always exactly `width` cells.
    #[must_use]
    pub fn render(&self, width: u16) -> String {
        let separator = self.style.separator();
        let gaps = self.segments.len().saturating_sub(1) as u16;
        let separators_width = display_width(separator).saturating_mul(gaps);
        let inner = width.saturating_sub(separators_width);
        let cells = allocate(&self.segments, inner);

        let mut out = String::new();
        for (index, (segment, span)) in self.segments.iter().zip(cells).enumerate() {
            if index > 0 {
                out.push_str(separator);
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

/// Append one segment's `span` cells to `out`: filler up to done, leader up to lead, then blanks.
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
    for _ in 0..done_cells {
        out.push(style.filler());
    }
    for _ in done_cells..lead_cells {
        out.push(style.leader());
    }
    for _ in lead_cells..span {
        out.push(style.blank());
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

/// The display width of a string in terminal cells, saturating at [`u16::MAX`].
fn display_width(text: &str) -> u16 {
    text.width().min(usize::from(u16::MAX)) as u16
}
