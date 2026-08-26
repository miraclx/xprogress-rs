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
- **`pulsate`** (indeterminate bounce) and the **stream-generator DSL**. A future async adapter will
  cover "attach a bar to a byte stream" with plain `AsyncRead`/`AsyncWrite`, not a generator.

## Name

Kept `xprogress` (dir `xprogress-rs`, crate `xprogress`), mirroring the `xbytes-rs` convention, so the
npm-to-crates lineage is obvious. `prism` / `spectra` were considered for a fresh identity; `spectra`
is already taken by an unrelated repo, and keeping the name preserves lineage. Open to a rename before
first publish if a fresh identity is preferred.

## API shape

- **Bind behaviour to types.** Advancing is a method on `Segment`, not a `(bar, index, value)`
  free-for-all like the original. `Bar` owns rendering; `Segment` owns its own progress.
- **Non-empty by construction.** `Bar::new(first, rest)` takes the first capacity separately, so an
  empty bar cannot exist. No error type needed for construction.
- **Newtype every scalar.** `Units` (a count) and `Percent` (clamped `0..=100`) never cross the API
  as bare numbers.
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

## Later (not in the first cut)

- Terminal `DrawTarget` (a TTY vs a hidden no-op sink; non-interactive is a type, not a runtime error).
- Async `AsyncRead`/`AsyncWrite` segment adapters, behind a `stream` feature, shaped like iris's
  progress reader/writer, so the sync core stays usable in non-async CLIs.
- Optional color, as a wrapping concern over the plain-glyph core (kept out of cell-counting).

## Housekeeping

- Held to the current house standard: edition 2024, `rustfmt.toml` (StdExternalCrate + Module), lint
  gate wired from the first commit (`[lints]` + `clippy.toml` with `allow-unwrap-in-tests`), tests in
  `<module>_tests.rs`, `///` on every public item, no `unwrap`/`expect` in non-test code, no em dashes.
- Dual MIT/Apache, canonical license files, holder Miraculous Owonubi.
