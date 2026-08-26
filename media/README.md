# media

GIFs for the README, generated headlessly with [vhs](https://github.com/charmbracelet/vhs) (no manual
screen recording). Each `.tape` is a script vhs runs in its own terminal and captures to a `.gif`.

## Generate

```sh
brew install vhs          # once (or: go install github.com/charmbracelet/vhs@latest)
vhs media/download.tape   # -> media/download.gif
vhs media/pulsate.tape    # -> media/pulsate.gif
vhs media/installer.tape  # -> media/installer.gif
```

Then reference them from the top-level `README.md`, e.g. `![download](media/download.gif)`.

## Note

The examples animate in a real terminal (vhs provides one). For a quick text look without vhs, dump
frames instead: `XPROGRESS_DUMP=1 cargo run --example download`.
