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

A download client (parallel chunks, one segment each), a package-manager installer (a segmented bar
per package, composed with indicatif), and an indeterminate pulse:

![download](media/download.gif)

![installer](media/installer.gif)

![pulsate](media/pulsate.gif)

## Usage

```rust
use xprogress::{Bar, Units};

// A bar with two segments (e.g. two parts of one download), sized by capacity.
let mut bar = Bar::new(Units::new(120), [Units::new(80)]);

// Advance confirmed progress; mark bytes buffered-ahead separately.
bar.segment(0).unwrap().advance(Units::new(48));
bar.segment(0).unwrap().lead_to(Units::new(72));

// Render to a plain string of an exact width. The bar never touches the terminal itself.
let line = bar.render(40);
assert_eq!(line.chars().count(), 40);
println!("[{line}]");
```

## Examples

Run them in a terminal to watch them animate. They draw to stderr and stay silent when output is not
a TTY, so they are safe to pipe.

```sh
cargo run --example download                   # a download client: parallel chunks, one segment each
cargo run --example pulsate                    # an indeterminate bar sweeping back and forth
cargo run --example stream --features stream   # segments driven by live async byte streams
cargo run --example installer                  # a package manager: per-package segmented bars (via indicatif)
cargo run --example indicatif_compose          # an xprogress line inside an indicatif MultiProgress
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
- **Non-empty by construction:** `Bar::new` takes the first segment separately, so an empty bar is
  unrepresentable.

## What it is not

It does not reimplement [indicatif]: no template mini-language, no one-bar-per-line stacking, no
spinner. Reach for indicatif when you want a bar per line; reach for `xprogress` for the segmented
single line it does not offer.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.

[indicatif]: https://docs.rs/indicatif
[linya]: https://docs.rs/linya
