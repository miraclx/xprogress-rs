# xprogress-rs decisions

A Rust rewrite of the NodeJS `xprogress`. This log captures the design calls; it is the living record
for the rewrite. Not part of theia; separate effort.

## Scope: the one real gap, nothing more

The distinguishing, ecosystem-unserved feature of the original is **segmented single-line progress**:
several independent progresses composited onto one terminal line. That is what this crate builds. We
deliberately drop everything indicatif already does well:

- the `stringd` **template mini-language** (`:{var}` strings). Replaced by typed styles. Kills the
  silent-typo class of bug and matches "make illegal states unrepresentable".
- **`append` / line-stacking** (one bar per line). That is indicatif `MultiProgress`'s job.
- the **stream-generator DSL**. Replaced by the `stream` feature's `AsyncRead`/`AsyncWrite` segment
  adapters, not a generator.

`pulsate` (an indeterminate bouncing bar) is NOT dropped: indicatif has no such thing, so it ships here
as `Pulse`.

## Name

Kept `xprogress` (dir `xprogress-rs`, crate `xprogress`), mirroring the `xbytes-rs` convention, so the
npm-to-crates lineage is obvious. `prism` / `spectra` were considered for a fresh identity; `spectra`
is already taken by an unrelated repo, and keeping the name preserves lineage. Open to a rename before
first publish if a fresh identity is preferred.

## API shape

- **Read like the use case.** `Bar::new(sizes)` takes a plain iterable of segment sizes (any
  `Into<Units>`, so `u64`/`usize` just work). The bar owns its terminal target and width, so a download
  client writes `bar.advance(chunk, bytes); bar.draw()?` with no ceremony. `Segment` still carries the
  behaviour (a `(bar, index)` accessor and per-segment methods); the bar-level `advance(index, delta)`
  is a deliberate convenience for the common indexed-chunk case. An empty bar renders blank;
  construction is infallible.
- **Newtype the scalars.** `Units` (a count) and `Percent` (clamped `0..=100`) are the currency
  internally; the ergonomic boundary takes `impl Into<Units>` so callers pass plain numbers.
- **`done`/`lead` invariant.** `0 <= done <= lead <= capacity`, enforced by every mutator, so the
  buffered-ahead marker can never fall behind confirmed progress or exceed capacity.

## Rendering

- **Pure and synchronous core.** `Bar::render(width) -> String` produces an exact-width line and never
  touches a terminal. A single owner draws; there is no shared-stdout singleton or cursor-math (the
  original's biggest source of corruption). A terminal draw target comes as a separate layer.
- **Largest-remainder allocation** over integers, so segment spans always sum to the exact width; a
  zero-capacity segment is treated as complete; an all-zero bar splits evenly.
- **`unicode-width`** measures separator display width so cell counts stay correct; the one
  indisputable dependency.

## Implemented since the first cut

- Terminal `DrawTarget` (Terminal / Hidden / Dump; non-interactive is a variant, not a runtime error).
- Async `AsyncRead`/`AsyncWrite` segment adapters behind the `stream` feature.
- `Pulse`, the indeterminate bouncing bar.
- Color (per segment, overriding a bar-wide default) and a header glyph; color codes are zero display
  width, so cells still count exactly.

## Still later

- A thin, feature-gated `indicatif` adapter so composing a segmented bar into a `MultiProgress` is a
  one-liner (glue only; it never reimplements stacking or formatting).

## Housekeeping

- Held to the current house standard: edition 2024, `rustfmt.toml` (StdExternalCrate + Module), lint
  gate wired from the first commit (`[lints]` + `clippy.toml` with `allow-unwrap-in-tests`), tests in
  `<module>_tests.rs`, `///` on every public item, no `unwrap`/`expect` in non-test code, no em dashes.
- Dual MIT/Apache, canonical license files, holder Miraculous Owonubi.
