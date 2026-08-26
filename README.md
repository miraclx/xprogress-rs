# xprogress

Segmented single-line progress bars for Rust. Composite several independent progresses onto **one**
terminal line, each in its own segment, with confirmed-versus-buffered (`lead`) shading.

Where [indicatif] and [linya] draw one bar per line, `xprogress` divides a *single* line into
segments, one per concurrent task. That is the natural display for parallel range-downloads of one
file, or any "many small things at once" glance.

> Early. The pure rendering core is here and tested; a terminal draw target and async
> `AsyncRead`/`AsyncWrite` segment adapters are coming. This is a Rust rewrite of the NodeJS
> [`xprogress`](https://github.com/miraclx/xprogress), rebuilt type-first.

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
