use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use std::sync::Barrier;
use std::thread;

use tokio::io;
use tokio::io::{AsyncWrite as _, AsyncWriteExt as _};

use crate::segment::Segment;
use crate::stream::{Mark, Meter, SegmentReader, SegmentWriter};
use crate::units::Units;

#[test]
fn meter_tracks_done_and_lead_independently() {
    let meter = Meter::new(Units::new(100));
    meter.advance(Mark::Lead, Units::new(70));
    meter.advance(Mark::Done, Units::new(40));
    let snapshot = meter.snapshot();
    assert_eq!(snapshot.done(), Units::new(40));
    assert_eq!(snapshot.lead(), Units::new(70));
}

#[test]
fn advancing_either_marker_saturates_at_u64_max() {
    for mark in [Mark::Done, Mark::Lead] {
        for start in [0, u64::MAX - 1, u64::MAX] {
            for delta in [0, 1, 2, u64::MAX] {
                let meter = Meter::new(Units::new(u64::MAX));
                meter.set(mark, Units::new(start));
                meter.advance(mark, Units::new(delta));
                let snapshot = meter.snapshot();
                let expected = Units::new(start.saturating_add(delta));
                assert_eq!(snapshot.lead(), expected, "{mark:?}, {start}, {delta}");
                assert_eq!(
                    snapshot.done(),
                    match mark {
                        Mark::Done => expected,
                        Mark::Lead => Units::new(0),
                    }
                );
            }
        }
    }
}

#[test]
fn advance_only_snapshots_are_monotonic_and_normalized_at_zero_one_and_many_capacity() {
    for capacity in [0, 1, 97, u64::MAX] {
        let meter = Meter::new(Units::new(capacity));
        let mut previous = meter.snapshot();
        for delta in [0, 1, 7, u64::MAX - 8, 1, u64::MAX] {
            for mark in [Mark::Done, Mark::Lead] {
                meter.advance(mark, Units::new(delta));
                let snapshot = meter.snapshot();
                assert!(snapshot.done() >= previous.done());
                assert!(snapshot.lead() >= previous.lead());
                assert!(snapshot.done() <= snapshot.lead());
                assert!(snapshot.lead() <= snapshot.capacity());
                assert_eq!(snapshot.capacity(), Units::new(capacity));
                previous = snapshot;
            }
        }
        assert_eq!(previous.done(), Units::new(capacity));
        assert_eq!(previous.lead(), Units::new(capacity));
    }
}

#[test]
fn concurrent_advances_keep_updates_and_saturate_without_regressing_snapshots() {
    for start in [0, u64::MAX - 1_024] {
        let meter = Meter::new(Units::new(u64::MAX));
        meter.set(Mark::Done, Units::new(start));
        meter.set(Mark::Lead, Units::new(start));
        let barrier = Barrier::new(9);
        thread::scope(|scope| {
            for _ in 0..8 {
                let meter = &meter;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    for _ in 0..512 {
                        meter.advance(Mark::Done, Units::new(1));
                        meter.advance(Mark::Lead, Units::new(1));
                    }
                });
            }
            barrier.wait();
            let mut previous = meter.snapshot();
            for _ in 0..4_096 {
                let snapshot = meter.snapshot();
                assert!(snapshot.done() >= previous.done());
                assert!(snapshot.lead() >= previous.lead());
                assert!(snapshot.done() <= snapshot.lead());
                assert!(snapshot.lead() <= snapshot.capacity());
                previous = snapshot;
            }
        });
        let expected = Units::new(start.saturating_add(4_096));
        assert_eq!(meter.snapshot().done(), expected);
        assert_eq!(meter.snapshot().lead(), expected);
    }
}

#[test]
fn setting_may_move_markers_backwards_but_snapshots_stay_normalized() {
    let meter = Meter::new(Units::new(100));
    meter.set(Mark::Done, Units::new(80));
    meter.set(Mark::Lead, Units::new(90));
    meter.set(Mark::Done, Units::new(20));
    assert_eq!(meter.snapshot().done(), Units::new(20));
    assert_eq!(meter.snapshot().lead(), Units::new(90));
    meter.set(Mark::Lead, Units::new(10));
    assert_eq!(meter.snapshot().done(), Units::new(20));
    assert_eq!(meter.snapshot().lead(), Units::new(20));
}

#[test]
fn snapshot_clamps_to_capacity_and_keeps_lead_at_least_done() {
    let meter = Meter::new(Units::new(50));
    meter.set(Mark::Done, Units::new(80));
    meter.set(Mark::Lead, Units::new(10));
    let snapshot = meter.snapshot();
    assert_eq!(snapshot.done(), Units::new(50), "done clamps to capacity");
    assert_eq!(snapshot.lead(), Units::new(50), "lead rises to done");
}

#[test]
fn apply_to_updates_a_segment() {
    let meter = Meter::new(Units::new(100));
    meter.advance(Mark::Done, Units::new(25));
    meter.advance(Mark::Lead, Units::new(60));
    let mut segment = Segment::new(Units::new(100));
    meter.apply_to(&mut segment);
    assert_eq!(segment.done(), Units::new(25));
    assert_eq!(segment.lead(), Units::new(60));
}

#[test]
fn a_clone_shares_the_same_counters() {
    let meter = Meter::new(Units::new(100));
    let other = Meter::clone(&meter);
    other.advance(Mark::Done, Units::new(30));
    assert_eq!(meter.snapshot().done(), Units::new(30));
}

#[tokio::test]
async fn segment_reader_advances_the_meter_by_bytes_read() {
    let meter = Meter::new(Units::new(11));
    let source: &[u8] = b"hello world";
    let mut reader = SegmentReader::new(source, Meter::clone(&meter), Mark::Lead);
    let mut sink = Vec::new();
    let copied = tokio::io::copy(&mut reader, &mut sink).await.unwrap();

    assert_eq!(copied, 11, "every byte was read through the adapter");
    assert_eq!(sink, source, "the bytes pass through unchanged");
    assert_eq!(
        meter.snapshot().lead(),
        Units::new(11),
        "the meter's lead advanced by the bytes read"
    );
    assert_eq!(
        meter.snapshot().done(),
        Units::new(0),
        "the other marker is untouched"
    );
    // The wrapped reader is recoverable.
    let _inner: &[u8] = reader.into_inner();
}

#[tokio::test]
async fn segment_writer_advances_the_meter_by_bytes_written() {
    let meter = Meter::new(Units::new(5));
    let mut buffer = Vec::new();
    let mut writer = SegmentWriter::new(&mut buffer, Meter::clone(&meter), Mark::Done);
    writer.write_all(b"abcde").await.unwrap();
    writer.flush().await.unwrap();

    assert_eq!(
        meter.snapshot().done(),
        Units::new(5),
        "the meter's done advanced by the bytes written"
    );
    drop(writer);
    assert_eq!(buffer, b"abcde", "the bytes reached the underlying writer");
}

#[test]
fn writer_counts_only_accepted_bytes_before_flush_and_passes_through_shutdown_failure() {
    for mark in [Mark::Done, Mark::Lead] {
        for start in [0, u64::MAX] {
            for outcome in [
                WriteOutcome::Accept(0),
                WriteOutcome::Accept(1),
                WriteOutcome::Accept(3),
                WriteOutcome::Pending,
                WriteOutcome::Error,
            ] {
                let meter = Meter::new(Units::new(u64::MAX));
                meter.set(mark, Units::new(start));
                let probe = WriteProbe {
                    outcome,
                    flushes: 0,
                    shutdowns: 0,
                };
                let mut writer = SegmentWriter::new(probe, Meter::clone(&meter), mark);
                let mut cx = Context::from_waker(Waker::noop());
                let result = Pin::new(&mut writer).poll_write(&mut cx, b"abc");
                let accepted = match outcome {
                    WriteOutcome::Accept(count) => {
                        assert!(
                            matches!(result, Poll::Ready(Ok(n)) if n == count),
                            "{result:?}"
                        );
                        count as u64
                    }
                    WriteOutcome::Pending => {
                        assert!(result.is_pending(), "{result:?}");
                        0
                    }
                    WriteOutcome::Error => {
                        assert!(
                            matches!(result, Poll::Ready(Err(ref error)) if error.kind() == io::ErrorKind::BrokenPipe),
                            "{result:?}"
                        );
                        0
                    }
                };
                let snapshot = meter.snapshot();
                let expected = Units::new(start.saturating_add(accepted));
                assert_eq!(snapshot.lead(), expected);
                assert_eq!(
                    snapshot.done(),
                    match mark {
                        Mark::Done => expected,
                        Mark::Lead => Units::new(0),
                    }
                );
                assert!(Pin::new(&mut writer).poll_flush(&mut cx).is_pending());
                assert!(
                    matches!(Pin::new(&mut writer).poll_shutdown(&mut cx), Poll::Ready(Err(error)) if error.kind() == io::ErrorKind::BrokenPipe)
                );
                assert_eq!(meter.snapshot().done(), snapshot.done());
                assert_eq!(meter.snapshot().lead(), snapshot.lead());
                let probe = writer.into_inner();
                assert_eq!(probe.flushes, 1);
                assert_eq!(probe.shutdowns, 1);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum WriteOutcome {
    Accept(usize),
    Pending,
    Error,
}

struct WriteProbe {
    outcome: WriteOutcome,
    flushes: u16,
    shutdowns: u16,
}

impl io::AsyncWrite for WriteProbe {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.outcome {
            WriteOutcome::Accept(count) => Poll::Ready(Ok(count)),
            WriteOutcome::Pending => Poll::Pending,
            WriteOutcome::Error => Poll::Ready(Err(io::ErrorKind::BrokenPipe.into())),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.flushes += 1;
        Poll::Pending
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.shutdowns += 1;
        Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))
    }
}
