//! A package manager installing a set of dependencies, the mixed.js use case. indicatif owns the
//! stacked lines; each package is an xprogress segmented bar over its variable-sized chunks. When a
//! package finishes, its line settles to "installed", and the next one appears below it. Run it in a
//! terminal:
//!
//!     cargo run --example installer

use core::time::Duration;
use std::thread::sleep;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use xprogress::Bar;

#[derive(Clone, Copy)]
struct Package {
    name: &'static str,
    chunks: &'static [u64],
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let packages = [
        Package {
            name: "react",
            chunks: &[40, 35, 30, 20],
        },
        Package {
            name: "scheduler",
            chunks: &[20, 30, 30, 20],
        },
        Package {
            name: "loose-envify",
            chunks: &[40, 35, 25],
        },
        Package {
            name: "webpack",
            chunks: &[10, 25, 30, 20, 15],
        },
    ];

    let multi = MultiProgress::new();
    for package in packages {
        let line = multi.add(ProgressBar::new_spinner());
        line.set_style(ProgressStyle::with_template("  {prefix:<14} {msg}")?);
        line.set_prefix(package.name);

        // Download this package's chunks in parallel, advancing each segment as its bytes arrive.
        let mut bar = Bar::new(package.chunks.iter().copied());
        while !bar.is_complete() {
            for (chunk, &size) in package.chunks.iter().enumerate() {
                bar.advance(chunk, size / 8 + 1);
            }
            line.set_message(format!("[{}]", bar.render(28)));
            sleep(Duration::from_millis(60));
        }
        line.finish_with_message("installed");
    }

    println!("all packages installed");
    Ok(())
}
