# AGENTS.md

`xprogress`: segmented single-line progress bars. Several independent progresses composited onto one
terminal line, each in its own segment, with confirmed-versus-buffered shading. Lib only, edition 2024,
toolchain pinned to 1.97.1.

- `STYLE.md` is the contract, synced byte-identical from `craft/`. Do not edit the copy; edit the
  canonical and run `just sync`.
- `miraclx/REVIEW.md` is how this code gets reviewed. `notes/craft-backlog.md` is what is open.

## The trap that already bit once

This crate is a **sibling dependency**, not an application. `libxget-rs` builds against the working copy
here through a gitignored path override, while its `Cargo.lock` pins a remote rev. Those two bodies of
code drift. When `DrawTarget::interject` was cut here, xget's bar stopped compiling locally while its CI
stayed green on the old pin. Eleven days.

**Before removing or renaming any `pub` item, grep the callers in `libxget-rs` and `freyr-rs`.**
In-repo zero callers is not evidence. That is the rule this incident wrote (`REVIEW.md`, the caller sweep).

`unreachable_pub` is not enabled. Check a new `pub` is reachable before adding it.

## Gates

```
cargo test --workspace --all-targets && cargo test --doc
cargo test --workspace --all-targets --features stream && cargo test --doc --features stream
cargo clippy --workspace --all-targets -- -D warnings            # default and --features stream
cargo +nightly fmt --check
cargo sort --workspace --check
```

`--features stream` is a second build, not an afterthought: `src/stream.rs` only compiles under it.
CI runs both. `alloc_instead_of_core = "warn"` is set here and nowhere else in the family; `xbytes-rs`
deliberately omits even `wildcard_imports`.

Unlike `xbytes-rs`, `Cargo.lock` is **not** tracked here, and `#![warn(missing_docs)]` is **not** set
anywhere in `src/`. A new `pub` item without docs builds clean. That is current state, not intent.

## Layout

No `mod.rs`. Flat `<module>.rs` with sibling `<module>_tests.rs`, no inline test modules.
There is no `style_tests.rs`: glyph and separator validation are covered through the bar regressions.
Run both default and stream matrices rather than relying on a fixed test count in this document.

`src/stream.rs` is the one `#[cfg]`-gated module. Its tests need `--features stream` to exist at all.

## The drawing contract

One enum, five draw methods, one writer. `DrawTarget::{Terminal, Hidden, Dump}`. `draw` for one line,
`draw_block` for a block, `finish_block` to commit one, `finish` for a tail line. `emit` is private and
is the single terminal write behind all five.

`draw_block(block, previous) -> usize` is a feedback loop, not a fire-and-forget call. `previous` is how
many lines the last call drew (`0` first); the return is how many this call drew; the caller threads it
forward and keeps it. `DrawTarget` is `Clone + Copy` and stateless by design, so the bookkeeping lives
with the caller. Do not "simplify" it into a stateful handle.

The Terminal escape protocol (`target.rs`, `emit`): cursor up `previous - 1` lines only when
`previous > 1`, then carriage return plus erase-to-end-of-screen, then the body, then a newline only on
commit. The erase is what makes a shrinking block leave no remnant. Touch nothing here without running
the examples against a real terminal, because the only tests run against `Hidden` (`target_tests.rs`,
two tests), and `Terminal` and `Dump` have no coverage at all.

`rows()` has no in-crate/example/README caller, but Freyr's ledger uses it to bound terminal height.
Keep it and improve target coverage; local non-use is not a reason to remove a shared public method.

`XPROGRESS_DUMP=1` turns any `from_env()` target into `Dump`: every frame on its own stdout line. That is
the logs-and-CI path. Setting the variable to the empty string still counts as set.

## The render contract

Widths are **terminal cells**, measured with `unicode-width`, never chars. A rendered line is always
exactly the requested number of cells; the tests assert this.

Cell glyphs (`filler`, `leader`, `blank`, `header`) are rejected unless they are exactly one cell wide.
The **separator may be wide**: positive-width glyphs are measured, not assumed to be one cell.
Invalid control, line/paragraph and zero-width separators leave the previous setting unchanged.
Render retains all separators only when their full cell cost fits the requested width, including exact
fit. Otherwise it omits all separators and allocates the full span to the segments. Width zero is empty.

`Layout` (`bar.rs`) is private. It splits one span into done body, lead body, one optional head cell, and
empty, and a debug assertion pins the tiling. The head exists only while a frontier exists: a segment
filled edge to edge takes the plain glyph. The color resolution is done-colour from the segment
overriding the style, lead falling back to done, head on the lead frontier (`bar.rs`, `Layout::resolve`).
Per-segment colors are ignored by `render_aggregate`, which builds a throwaway segment.

`allocate` is largest-remainder by descending remainder, ties to the earlier segment. `even_split`
explicitly handles zero and keeps counts as `usize`; narrowing a segment count to terminal-cell width
made 65,536 zero-capacity segments panic. Boundary regressions pin that class and the allocation ties.

## The `stream` module

`Meter` is `Arc` over two `Relaxed` atomics. `advance` uses saturating atomic updates; `set` may move
an absolute value backwards. `snapshot` clamps to capacity and raises lead to done. It is a normalized
observation from independent loads, not a coherent two-marker transaction or a publication barrier.

`SegmentReader` advances on bytes read; `SegmentWriter` counts accepted successful writes, not flush,
durability, hashing or expected-integrity completion. Flush and shutdown pass through untouched.

`Progress::new` and `Units::saturating_add` are `pub(crate)`. Downstream cannot build a `Progress` or sum
`Units`; it gets them from `Bar::aggregate`, `Segment::progress`, `Meter::snapshot`. That ceiling is the
invariant, not an oversight, but it is a ceiling all the same.

Watch the asymmetry: `Bar::finish()` takes nothing, `Pulse::finish` takes the line. Do not "align" one to
the other without checking both callers.

## Things you will get wrong if nobody tells you

- **`u64` is the only count.** `Units(u64)`. Signed `From` impls saturate negatives to zero; `usize`
  truncates on 128-bit platforms. The family rule about explicitly-sized integers holds; this is the one
  place it is load-bearing.
- **A zero-capacity segment is complete.** `is_complete` is `done >= capacity`. An empty `Bar` is
  vacuously complete. Out-of-range `advance` is a silent no-op.
- **`Percent` of nothing is 100.** A task with no work is finished, not undefined. Overfull clamps.
- **`Pulse` does not validate glyphs.** It trusts `Style` to have rejected them already. Render raw glyphs
  past it and the exact-width guarantee is gone.
- **`from_env` is the default** for `Bar` and `Pulse`. Examples draw to stderr and stay silent off a TTY
  for this reason; never add a print to an example.

## Features

`stream` gates the async adapters and pulls `tokio` with `default-features = false`. Every other build in
the repo, including CI, runs both matrices. `indicatif` is a dev-dependency only, and only for the
`multibar` example that shows the deliberate seam: stacking is indicatif's job, one exact line is this
crate's.

`rust-version = "1.85"` in `Cargo.toml` but the toolchain is pinned to `1.97.1`. Known inconsistency, same
as the siblings.

## Docs

`README.md` lists six of the ten examples. `aggregate`, `buffered`, `glyphs` and `playback` are missing
from the list; two of those are the most substantial demos in the tree. `media/` holds six GIFs but only
three tapes (`download`, `installer`, `pulsate`); `buffered`, `multibar` and `styles` have no tape and are
not reproducible. The missing tapes are mechanical: same template, different height and sleep.

`vhs` is the tool and `media/README.md` is the invocation. Record, then judge the frames yourself before
committing one.
