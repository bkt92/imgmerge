# imgmerge

A fast Rust CLI utility to combine multiple images into one by stacking them
**horizontally**, **vertically**, or in a **custom grid** (e.g. 3×2).
Images can be freely reordered before placement.

## Features

- **Three layouts** — horizontal row, vertical column, or any N×M grid
- **Reordering** — `--order 2,0,1` picks which image goes where
- **Custom cell size** — `--cell-width` / `--cell-height` force every slot to
  the same dimensions; omit to use the largest source image automatically
- **Gap control** — `--gap N` adds N pixels between cells (no outer border)
- **Background fill** — `--bg rrggbb` or `--bg rrggbbaa` for transparent gaps
- **Any format** — reads/writes PNG, JPEG, BMP, TIFF, WebP, GIF, and more
  (format is inferred from the output file extension)

## Build

```bash
cargo build --release
# Binary: ./target/release/imgmerge
```

## Usage

### Horizontal

```bash
# Stack three images side-by-side with an 8-pixel gap
imgmerge horizontal a.png b.png c.png -o out.png --gap 8

# Force a uniform 400×300 cell size
imgmerge horizontal *.jpg -o out.jpg --cell-width 400 --cell-height 300
```

### Vertical

```bash
imgmerge vertical a.png b.png c.png -o out.png --gap 4
```

### Grid

```bash
# 3-column grid — rows computed automatically from image count
imgmerge grid img*.png -o grid.png --cols 3

# Explicit 3×2 grid with 10px gaps and a black background
imgmerge grid img*.png -o grid.png --cols 3 --rows 2 --gap 10 --bg 000000

# 2×2 grid with fixed 512×512 cells
imgmerge grid a.png b.png c.png d.png -o grid.png --cols 2 --cell-width 512 --cell-height 512
```

### Reordering

`--order` accepts comma-separated **0-based** indices into the input file list.

```bash
# Input order: a.png(0) b.png(1) c.png(2) d.png(3) e.png(4) f.png(5)
# Display as 3×2 with images reordered: 5,4,3,2,1,0
imgmerge grid a.png b.png c.png d.png e.png f.png \
  -o out.png --cols 3 --order 5,4,3,2,1,0

# Duplicate an image into multiple slots
imgmerge horizontal logo.png photo.png logo.png -o banner.png
# …or using --order:
imgmerge horizontal logo.png photo.png -o banner.png --order 0,1,0
```

> **Note:** If you supply fewer indices than grid slots, the remaining slots
> stay filled with the background color. If you supply more images than slots
> (cols × rows), the excess images are silently ignored.

## All flags

| Flag | Default | Description |
|------|---------|-------------|
| `-o / --output` | *required* | Output file path |
| `--gap` | `0` | Pixel gap between cells |
| `--cell-width` | auto | Force cell width (pixels) |
| `--cell-height` | auto | Force cell height (pixels) |
| `--order` | none | Comma-separated reorder indices |
| `--bg` | `ffffff` | Background hex color (RGB or RGBA) |
| `--cols` | *required (grid)* | Number of grid columns |
| `--rows` | auto | Number of grid rows |

## Dependencies

| Crate | Use |
|-------|-----|
| [`image`](https://crates.io/crates/image) | Image I/O and pixel operations |
| [`clap`](https://crates.io/crates/clap) | CLI argument parsing |
| [`anyhow`](https://crates.io/crates/anyhow) | Ergonomic error handling |
