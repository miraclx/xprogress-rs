//! A package manager installing a dependency TREE concurrently, the mixed.js use case. All top-level
//! packages start at once; as each begins it discovers its deps and inserts their bars right below it,
//! and those download concurrently too, discovering their own deps in turn. indicatif owns the growing
//! stack of lines (add / insert_after, and a finished line settles to "installed"); xprogress renders
//! each line as a segmented bar over that package's variable-sized chunks. Run it in a terminal:
//!
//!     cargo run --example installer

use core::time::Duration;
use std::thread;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use xprogress::{Bar, Color, Style};

struct Package {
    name: &'static str,
    chunks: &'static [u64],
    deps: &'static [Package],
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let tree = [
        Package {
            name: "react",
            chunks: &[40, 35, 30, 20],
            deps: &[
                Package {
                    name: "scheduler",
                    chunks: &[20, 30, 30, 20],
                    deps: &[],
                },
                Package {
                    name: "loose-envify",
                    chunks: &[40, 35, 25],
                    deps: &[],
                },
            ],
        },
        Package {
            name: "webpack",
            chunks: &[10, 25, 30, 20, 15],
            deps: &[Package {
                name: "acorn",
                chunks: &[25, 35, 25, 15],
                deps: &[],
            }],
        },
    ];

    let multi = MultiProgress::new();
    let style = ProgressStyle::with_template("  {prefix:<14} {msg}")?;

    // Start every top-level package at once; the tree grows from there.
    thread::scope(|scope| {
        for package in &tree {
            let line = multi.add(ProgressBar::new_spinner());
            line.set_style(ProgressStyle::clone(&style));
            install(scope, &multi, &style, line, package);
        }
    });

    println!("all packages installed");
    Ok(())
}

/// Install `package` on its own thread, rendering into `line`. Once it starts, its deps are discovered
/// and inserted right below it, each installed concurrently in turn.
fn install<'scope, 'env>(
    scope: &'scope thread::Scope<'scope, 'env>,
    multi: &'env MultiProgress,
    style: &'env ProgressStyle,
    line: ProgressBar,
    package: &'env Package,
) {
    scope.spawn(move || {
        line.set_prefix(package.name);
        let mut bar = Bar::new(package.chunks.iter().copied()).with_style(
            Style::default()
                .with_color(Color::Yellow)
                .with_filler('▰')
                .with_blank('▱')
                .with_separator('·'),
        );

        // A moment to resolve, then the deps are known: insert each one right below this line and
        // install it concurrently.
        thread::sleep(Duration::from_millis(200));
        let mut anchor = ProgressBar::clone(&line);
        for dep in package.deps {
            let dep_line = multi.insert_after(&anchor, ProgressBar::new_spinner());
            dep_line.set_style(ProgressStyle::clone(style));
            anchor = ProgressBar::clone(&dep_line);
            install(scope, multi, style, dep_line, dep);
        }

        // Download this package's chunks, advancing each segment as its bytes arrive.
        while !bar.is_complete() {
            for (chunk, &size) in package.chunks.iter().enumerate() {
                bar.advance(chunk, size / 25 + 1);
            }
            line.set_message(format!("[{}]", bar.render(28)));
            thread::sleep(Duration::from_millis(60));
        }
        line.finish_with_message("installed");
    });
}
