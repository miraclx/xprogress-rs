# xprogress

Segmented single-line progress bars for Rust. Composite several independent progresses onto **one**
terminal line, each in its own segment, with confirmed-versus-buffered (`lead`) shading.

Where [indicatif] and [linya] draw one bar per line, `xprogress` divides a *single* line into
segments, one per concurrent task. That is the natural display for parallel range-downloads of one
file, or any "many small things at once" glance.

> Early. The rendering core, a terminal draw target, and async `AsyncRead`/`AsyncWrite` segment
> adapters (the `stream` feature) are here and tested. This is a Rust rewrite of the NodeJS
> [`xprogress`](https://github.com/miraclx/xprogress), rebuilt type-first.

## Demo

A download client: one file in parallel chunks, a segment per chunk, colored and animating in place:

![download](media/download.gif)

An indeterminate pulse, for work with no known total:

![pulsate](media/pulsate.gif)

## Usage

```rust
use xprogress::Bar;

// A bar with two segments (e.g. two parts of one download), sized by their byte counts.
let mut bar = Bar::new([120u64, 80]);

// Advance confirmed progress on a segment; mark bytes buffered-ahead separately.
bar.advance(0, 48);
bar.lead_to(0, 72);

// Render to a plain string of an exact width. The bar never touches the terminal itself.
let line = bar.render(40);
assert_eq!(line.chars().count(), 40);
println!("[{line}]");
```

## Multi-bar (stack with indicatif)

`xprogress` deliberately does not stack bars vertically. When you want that — several bars, each on its
own line — reach for [indicatif]'s `MultiProgress` and let an `xprogress` segmented line ride one of its
rows. The bridge is just `Bar::render` returning a `String`, so neither side owns the other:

```rust
use indicatif::{MultiProgress, ProgressBar};
use xprogress::Bar;

let multi = MultiProgress::new();
let overall = multi.add(ProgressBar::new(total));       // an ordinary indicatif line
let segmented = multi.add(ProgressBar::new_spinner());  // hosts the xprogress line

let mut bar = Bar::new(chunk_sizes);                    // one segment per chunk
// each tick, as work happens:
bar.advance(chunk, n);
overall.set_position(bar.aggregate().done().get());
segmented.set_message(bar.render(42));                  // an xprogress line *inside* indicatif
```

Run it: `cargo run --example multibar`.

## Styling

A `Bar` draws with a `Style`. Presets: `Style::default()` (block), `Style::ascii()` (`#`/`=`/`-`, for
non-UTF-8 terminals), `Style::line()` (heavy color two-tone), `Style::braille()` (dotted). Build your own
with the `with_filler`/`with_color`/`with_separator` builders. `cargo run --example styles` shows them all
animating together.

## Examples

Run them in a terminal to watch them animate. They draw to stderr and stay silent when output is not
a TTY, so they are safe to pipe.

```sh
cargo run --example download                   # a download client: parallel chunks, one segment each
cargo run --example pulsate                    # an indeterminate bar sweeping back and forth
cargo run --example stream --features stream   # segments driven by live async byte streams
cargo run --example styles                     # every built-in Style preset, filling in step
cargo run --example installer                  # a package manager: per-package segmented bars (via indicatif)
cargo run --example multibar                   # an xprogress line stacked inside an indicatif multi-bar
```

To see the frames without a terminal (logs, CI, or a quick look), dump each frame on its own line:

```sh
XPROGRESS_DUMP=1 cargo run --example download
```

## Model

- **`Bar`** owns one line and is the only thing that draws. Segments are data; they never render
  themselves, so there is no shared-stdout race.
- **`Segment`** tracks `done` (confirmed) and `lead` (buffered-ahead) against a fixed `capacity`,
  preserving `0 <= done <= lead <= capacity` by construction.
- **Width** is divided across segments in proportion to capacity by the largest-remainder method, so
  a rendered line is always exactly the requested number of cells.
- **Rendering is separate from drawing:** `Bar::render(width)` is pure, for composing into other
  displays (e.g. an indicatif line); `Bar::draw`/`finish` redraw in place on a terminal and stay silent
  off a TTY.

## What it is not

It does not reimplement [indicatif]: no template mini-language, no one-bar-per-line stacking, no
spinner. Reach for indicatif when you want a bar per line; reach for `xprogress` for the segmented
single line it does not offer.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.

[indicatif]: https://docs.rs/indicatif
[linya]: https://docs.rs/linya
