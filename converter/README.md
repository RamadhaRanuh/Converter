# Converter (Rust)

A fast, self-contained desktop converter for images, designs and documents. Drop files or folders onto the
window, pick a format, convert. Everything runs offline on your machine, and files are converted in parallel
across all CPU cores.

It replaces the React/Express app in `frontend/` and `backend/`, which remains in the repository as a reference.

## What it converts

| From | To |
|---|---|
| JPG, PNG, WebP, BMP, TIFF, GIF | any of those, PSD (flattened), SVG (traced), PDF |
| HEIC (iPhone photos) | JPG, PNG, WebP, BMP, TIFF, GIF, PDF |
| PSD | JPG, PNG, WebP, BMP, TIFF, GIF, PDF (reads the stored composite image) |
| AI (Illustrator, PDF-compatible) | JPG, PNG, WebP, BMP, TIFF, GIF, PDF |
| SVG | JPG, PNG, WebP, BMP, TIFF, GIF, optimized SVG, PDF |
| PDF | one PNG or JPG per page, one PDF per page (split) |
| Word, PowerPoint, Excel | PDF |
| Markdown | PDF, HTML |

**Combine** several images and PDFs into a single PDF.

Output files are written next to each source file (or into a chosen folder) and **never overwrite** an existing
file: a clash becomes `name (1).ext`. A file that fails shows a plain-language reason and never stops the rest of
the batch.

Not supported, by design: ODT, AVIF, HEIC output, AI output, legacy (PostScript) AI files, PSD layer extraction,
audio and video.

## Layout

```
converter/
  crates/converter-core   all format knowledge: detect, the conversion matrix, engines, batches
  crates/converter-cli    `converter-cli` command-line tool
  crates/converter-gui    `Converter` desktop window (egui/eframe, wgpu)
  crates/converter-bench  speed benchmark against the targets
```

## Build

Needs Rust (stable) and, on Windows, the MSVC build tools.

```sh
cd converter
cargo run --release -p converter-gui          # the desktop app
cargo run --release -p converter-cli -- --help
cargo test --workspace
```

### HEIC support (optional feature)

HEIC decoding uses libheif and libde265 (LGPL), linked as DLLs next to the executable. Build them once with
[vcpkg](https://github.com/microsoft/vcpkg), **without** the default `hevc` feature (which pulls in the GPL x265
encoder):

```sh
vcpkg install "libheif[core]:x64-windows"
set VCPKG_ROOT=C:\path\to\vcpkg
set VCPKGRS_DYNAMIC=1
cargo build --release -p converter-gui --features heic
```

At runtime `heif.dll` and `libde265.dll` must sit next to the executable (or on `PATH`).

## Command line

```sh
converter-cli photos/ --to webp -q 80           # a whole folder, in parallel
converter-cli report.pdf --to png-pages --dpi 200
converter-cli scan1.jpg scan2.jpg notes.pdf --combine
converter-cli drop/ --targets                   # what can these files become?
```

## Speed

Measured with `converter-bench` on the reference machine (i9-14900HX, 32 threads, Windows 11) against the old
Node/sharp app, on the seeded benchmark corpus (median of 5 runs, quality 80):

| Case | Node app | Rust | Target |
|---|---|---|---|
| 64 MB PSD → PNG | 15.8 s | 0.21 s | ≤ 3 s |
| 48 MP TIFF → WebP | 8.8 s | 2.48 s | ≤ 6 s |
| 300 × 12 MP JPG → WebP | 52 s (177 s one at a time, as the web UI did) | 16.7 s | ≤ 20 s |
| 50-page PDF → PNG per page, 150 DPI | not supported | 1.66 s | (recorded) |

Peak memory during the 300-photo batch is about 110 MB per core (3.4 GB on 32 threads): each file in flight holds
its decoded pixels and the WebP encoder's buffers.

Run it yourself: `cargo run --release -p converter-bench -- <corpus-dir>`.

## Licences

The app is MIT OR Apache-2.0. Bundled engines are permissively licensed (image-rs, libwebp, zune, resvg, vtracer,
oxvg, svg2pdf, hayro, lopdf, Typst via office2pdf, comrak, egui). The optional HEIC decoder (libheif, libde265) is
LGPL and ships as separate DLLs so it can be replaced.
