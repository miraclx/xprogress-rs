//! A single independent progress inside a [`Bar`](crate::Bar).

use anstyle::Color;

use crate::units::{Progress, Units};

/// One independent progress value. Every mutator preserves the invariant
/// `0 <= done <= lead <= capacity`, so confirmed progress never exceeds buffered-ahead progress and
/// neither exceeds capacity.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    capacity: Units,
    done: Units,
    lead: Units,
    color: Option<Color>,
}

impl Segment {
    /// A fresh segment of the given capacity, with no progress.
    pub const fn new(capacity: Units) -> Self {
        Self {
            capacity,
            done: Units::new(0),
            lead: Units::new(0),
            color: None,
        }
    }

    /// The total work this segment represents.
    pub const fn capacity(self) -> Units {
        self.capacity
    }

    /// Confirmed progress.
    pub const fn done(self) -> Units {
        self.done
    }

    /// Buffered-ahead progress: at least [`Segment::done`], at most [`Segment::capacity`].
    pub const fn lead(self) -> Units {
        self.lead
    }

    /// Advance confirmed progress by `delta`, clamped to capacity, pulling `lead` up to stay ahead.
    pub fn advance(&mut self, delta: Units) {
        self.set_done(self.done.saturating_add(delta));
    }

    /// Set confirmed progress to `done`, clamped to `0..=capacity`, pulling `lead` up if needed.
    pub fn set_done(&mut self, done: Units) {
        self.done = done.min(self.capacity);
        self.lead = self.lead.max(self.done);
    }

    /// Advance the buffered-ahead marker by `delta`, clamped to capacity.
    pub fn advance_lead(&mut self, delta: Units) {
        self.lead_to(self.lead.saturating_add(delta));
    }

    /// Move the buffered-ahead marker to `at`, clamped to `done..=capacity`.
    pub fn lead_to(&mut self, at: Units) {
        self.lead = at.clamp(self.done, self.capacity);
    }

    /// Whether confirmed progress has reached capacity.
    pub const fn is_complete(self) -> bool {
        self.done.get() >= self.capacity.get()
    }

    /// A snapshot of this segment's progress.
    pub const fn progress(self) -> Progress {
        Progress::new(self.done, self.lead, self.capacity)
    }

    /// This segment's color override, if any. When set, it overrides the bar-wide style color.
    pub const fn color(self) -> Option<Color> {
        self.color
    }

    /// Set this segment's color, so adjacent segments on one line can be told apart by hue.
    pub fn set_color(&mut self, color: impl Into<Color>) {
        self.color = Some(color.into());
    }
}
