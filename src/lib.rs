//! xprogress: segmented single-line progress bars.
//!
//! A [`Bar`] is one line of terminal width divided into independent [`Segment`]s, each tracking its
//! own progress against a fixed capacity. Where most progress libraries put one bar per line, this
//! composites several concurrent progresses side by side onto a *single* line, which is the natural
//! display for parallel range-downloads of one file, or any "many small things at once" glance.
//!
//! Each segment distinguishes confirmed progress ([`Segment::done`]) from buffered-ahead progress
//! ([`Segment::lead`]), so a download can shade bytes written to disk apart from bytes received but
//! not yet written.
//!
//! The core is pure and synchronous: a [`Bar`] renders to a [`String`] of an exact cell width and
//! never touches a terminal, so a single owner controls drawing and there is no shared-stdout race.
//!
//! ```
//! use xprogress::Bar;
//!
//! let mut bar = Bar::new([120u64, 80]); // two chunks, sized in bytes
//! bar.advance(0, 60u64); // 60 bytes done on chunk 0
//! let line = bar.render(40);
//! assert_eq!(line.chars().count(), 40);
//! ```

mod bar;
mod pulse;
mod segment;
mod style;
mod target;
mod units;
mod width;

#[cfg(feature = "stream")]
pub mod stream;

#[cfg(test)]
mod bar_tests;
#[cfg(test)]
mod pulse_tests;
#[cfg(test)]
mod segment_tests;
#[cfg(all(test, feature = "stream"))]
mod stream_tests;
#[cfg(test)]
mod target_tests;
#[cfg(test)]
mod units_tests;
#[cfg(test)]
mod width_tests;

pub use crate::bar::Bar;
pub use crate::pulse::Pulse;
pub use crate::segment::Segment;
pub use crate::style::Style;
pub use crate::target::DrawTarget;
pub use crate::units::{Percent, Progress, Units};
pub use crate::width::Width;
