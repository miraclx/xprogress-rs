//! Newtypes for progress quantities, so raw integers never cross the API boundary.

/// A count of work units: bytes, items, steps. The currency of progress; never a bare `u64`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Units(u64);

impl Units {
    /// Wrap a raw count.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Add two counts, saturating at [`u64::MAX`] so progress arithmetic never wraps.
    pub(crate) const fn saturating_add(self, other: Units) -> Units {
        Units(self.0.saturating_add(other.0))
    }
}

impl From<u64> for Units {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

/// A percentage clamped to `0.0..=100.0` at construction, so an out-of-range percent is
/// unrepresentable.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct Percent(f64);

impl Percent {
    /// The ratio `done / capacity` as a percentage. A zero capacity is treated as complete (100%),
    /// since a task with no work to do is already finished.
    pub fn of(done: Units, capacity: Units) -> Self {
        if capacity.get() == 0 {
            return Self(100.0);
        }
        let ratio = done.get() as f64 / capacity.get() as f64;
        Self((ratio * 100.0).clamp(0.0, 100.0))
    }

    /// The percentage value, in `0.0..=100.0`.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// A snapshot of one progress: confirmed, buffered-ahead, and total capacity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Progress {
    done: Units,
    lead: Units,
    capacity: Units,
}

impl Progress {
    pub(crate) const fn new(done: Units, lead: Units, capacity: Units) -> Self {
        Self {
            done,
            lead,
            capacity,
        }
    }

    /// Confirmed progress.
    pub const fn done(self) -> Units {
        self.done
    }

    /// Buffered-ahead progress, always at least [`Progress::done`].
    pub const fn lead(self) -> Units {
        self.lead
    }

    /// Total capacity.
    pub const fn capacity(self) -> Units {
        self.capacity
    }

    /// Confirmed completion as a percentage.
    pub fn percent(self) -> Percent {
        Percent::of(self.done, self.capacity)
    }

    /// Whether confirmed progress has reached capacity.
    pub const fn is_complete(self) -> bool {
        self.done.get() >= self.capacity.get()
    }
}
