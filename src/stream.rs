//! Drive segments from live byte streams (requires the `stream` feature).
//!
//! A [`Meter`] is a cheaply-clonable, thread-safe progress handle: a stream adapter advances it as
//! bytes flow, and a [`Bar`](crate::Bar) reads its snapshot to render. [`SegmentReader`] and
//! [`SegmentWriter`] wrap an async reader or writer and advance a meter by the bytes that pass
//! through, so attaching a bar to a transfer is just wrapping the stream.

use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};
use std::io;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::segment::Segment;
use crate::units::{Progress, Units};

/// Which marker a stream adapter advances. The caller assigns their meaning; adapters count bytes
/// read or accepted by a writer, not flush, hash, or expected-integrity completion.
#[derive(Clone, Copy, Debug)]
pub enum Mark {
    /// Progress the caller considers done. A [`SegmentWriter`] counts accepted writes, not flushed bytes.
    Done,
    /// Progress the caller considers buffered ahead of done.
    Lead,
}

/// A shared progress handle. Advancing it from a stream adapter and rendering a bar from its snapshot
/// are decoupled; cloning shares the same underlying counters, so one download's reader and writer can
/// feed a single meter's lead and done.
#[derive(Clone)]
pub struct Meter {
    inner: Arc<Inner>,
}

struct Inner {
    capacity: u64,
    done: AtomicU64,
    lead: AtomicU64,
}

impl Meter {
    /// A meter for a transfer of the given capacity, at zero progress.
    #[must_use]
    pub fn new(capacity: Units) -> Self {
        Self {
            inner: Arc::new(Inner {
                capacity: capacity.get(),
                done: AtomicU64::new(0),
                lead: AtomicU64::new(0),
            }),
        }
    }

    /// Advance a marker by `delta`, saturating at [`u64::MAX`]. Snapshots clamp it to capacity.
    pub fn advance(&self, mark: Mark, delta: Units) {
        // A relaxed atomic update preserves concurrent increments without synchronizing payload I/O.
        let _ = self
            .counter(mark)
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                Some(value.saturating_add(delta.get()))
            });
    }

    /// Set a marker to an absolute value, which may move it backwards. Snapshots clamp it to capacity.
    pub fn set(&self, mark: Mark, value: Units) {
        // The counter is an observation, not a publication barrier for work the caller performs.
        self.counter(mark).store(value.get(), Ordering::Relaxed);
    }

    /// The transfer capacity.
    #[must_use]
    pub fn capacity(&self) -> Units {
        Units::new(self.inner.capacity)
    }

    /// An observational snapshot, normalized to `0 <= done <= lead <= capacity`. Independent relaxed
    /// loads may observe markers from different instants; this is not a cross-marker transaction or
    /// synchronization with payload I/O. Normalization does not establish flush or integrity completion.
    #[must_use]
    pub fn snapshot(&self) -> Progress {
        let capacity = self.inner.capacity;
        let done = self.inner.done.load(Ordering::Relaxed).min(capacity);
        let lead = self
            .inner
            .lead
            .load(Ordering::Relaxed)
            .min(capacity)
            .max(done);
        Progress::new(Units::new(done), Units::new(lead), Units::new(capacity))
    }

    /// Apply this meter's current snapshot to a segment, ready to render.
    pub fn apply_to(&self, segment: &mut Segment) {
        let snapshot = self.snapshot();
        segment.set_done(snapshot.done());
        segment.lead_to(snapshot.lead());
    }

    fn counter(&self, mark: Mark) -> &AtomicU64 {
        match mark {
            Mark::Done => &self.inner.done,
            Mark::Lead => &self.inner.lead,
        }
    }
}

/// Wraps an [`AsyncRead`], advancing a [`Meter`] marker by the bytes read.
pub struct SegmentReader<R> {
    inner: R,
    meter: Meter,
    mark: Mark,
}

impl<R> SegmentReader<R> {
    /// Wrap `inner`, advancing `meter`'s `mark` as bytes are read.
    pub fn new(inner: R, meter: Meter, mark: Mark) -> Self {
        Self { inner, meter, mark }
    }

    /// Recover the wrapped reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for SegmentReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &result {
            let read = buf.filled().len() - before;
            if read > 0 {
                this.meter.advance(this.mark, Units::new(read as u64));
            }
        }
        result
    }
}

/// Wraps an [`AsyncWrite`], advancing a [`Meter`] marker by bytes accepted by successful writes.
///
/// Even with [`Mark::Done`], this does not report flush, durable storage, hashing, or expected-integrity
/// completion. Flush and shutdown are passed through without advancing the meter.
pub struct SegmentWriter<W> {
    inner: W,
    meter: Meter,
    mark: Mark,
}

impl<W> SegmentWriter<W> {
    /// Wrap `inner`, advancing `meter`'s `mark` as bytes are accepted by writes, before any flush.
    pub fn new(inner: W, meter: Meter, mark: Mark) -> Self {
        Self { inner, meter, mark }
    }

    /// Recover the wrapped writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for SegmentWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(written)) = &result
            && *written > 0
        {
            this.meter.advance(this.mark, Units::new(*written as u64));
        }
        result
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
