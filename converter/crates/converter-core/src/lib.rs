//! Converter core: everything that knows about formats. No UI, no printing.
//!
//! A **Conversion** turns one Source file into one or more Output files in a Target format;
//! a **Combine** turns many Source files into one PDF; a **Batch** runs many Conversions in parallel.
//! See `CONTEXT.md` at the repository root for the glossary.

mod batch;
mod engines;
mod error;
mod format;
mod output;
mod target;

use std::io::Write;
use std::path::{Path, PathBuf};

use image::DynamicImage;

pub use batch::{Event, Plan, PlannedFile, Summary, expand_sources, plan, run_batch};
pub use error::Error;
pub use format::{Format, FormatFamily, detect};
pub use target::{Target, can_combine, targets_for};

/// Knobs the user can change. Everything has a sensible default, so most users never open Options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// 1-100, for lossy outputs (JPG, WebP). 100 means lossless WebP.
    pub quality: u8,
    /// Resolution for rendering pages or vectors to images (PDF, AI, SVG, and Office → PDF → image).
    pub dpi: u16,
    /// Where Output files go. `None` means next to each Source file.
    pub out_dir: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Options { quality: 85, dpi: 150, out_dir: None }
    }
}

/// Runs one Conversion and returns the Output files it wrote.
pub fn convert(src: &Path, target: Target, opts: &Options) -> Result<Vec<PathBuf>, Error> {
    let format = detect(src)?;
    if !targets_for(format).contains(&target) {
        return Err(Error::NotPossible { from: format, to: target });
    }
    use Format::*;
    match (format, target) {
        (Pdf, Target::PdfPages(img)) => engines::pdf::render_pages(src, img, opts),
        (Pdf, Target::PdfSplit) => engines::pdf::split(src, opts),
        (Ai, Target::Pdf) => engines::pdf::ai_to_pdf(src, opts).map(|p| vec![p]),
        (Svg, Target::Svg) => write_output(src, "svg", "", opts, |w| engines::vector::optimize(src, w)).map(|p| vec![p]),
        (Svg, Target::Pdf) => write_output(src, "pdf", "", opts, |w| engines::vector::svg_to_pdf(src, w)).map(|p| vec![p]),
        (Docx | Pptx | Xlsx, Target::Pdf) => write_output(src, "pdf", "", opts, |w| engines::office::to_pdf(src, format, w)).map(|p| vec![p]),
        (Markdown, Target::Html) => write_output(src, "html", "", opts, |w| engines::markdown::to_html(src, w)).map(|p| vec![p]),
        (Markdown, Target::Pdf) => write_output(src, "pdf", "", opts, |w| engines::markdown::to_pdf(src, w)).map(|p| vec![p]),
        // Everything else starts from a single decoded image.
        (_, target) => {
            let img = decode_image(src, format, opts)?;
            let out = match target {
                Target::Image(f) => write_output(src, f.extension(), "", opts, |w| engines::raster::encode(&img, f, opts, w))?,
                Target::Psd => write_output(src, "psd", "", opts, |w| engines::psd::encode(&img, w))?,
                Target::Svg => write_output(src, "svg", "", opts, |w| engines::vector::trace(&img, w))?,
                Target::Pdf => write_output(src, "pdf", "", opts, |w| engines::pdf::images_to_pdf(std::slice::from_ref(&img), opts.quality, w))?,
                _ => return Err(Error::NotPossible { from: format, to: target }),
            };
            Ok(vec![out])
        }
    }
}

/// Runs one Combine: images and PDFs, in the given order, into a single PDF named after the first Source file.
pub fn combine(srcs: &[PathBuf], opts: &Options) -> Result<PathBuf, Error> {
    let first = srcs.first().ok_or_else(|| Error::Unsupported("Nothing to combine.".into()))?;
    let mut parts = Vec::with_capacity(srcs.len());
    for s in srcs {
        let f = detect(s)?;
        if !can_combine(f) {
            return Err(Error::CombineNotPossible(f));
        }
        parts.push(if f == Format::Pdf {
            engines::pdf::Part::Pdf(std::fs::read(s).map_err(|e| Error::io(s, e))?)
        } else {
            engines::pdf::Part::Image(decode_image(s, f, opts)?)
        });
    }
    write_output(first, "pdf", "-combined", opts, |w| engines::pdf::combine(parts, opts.quality, w))
}

/// Decodes any Source file that has a single-image form into pixels.
fn decode_image(src: &Path, format: Format, opts: &Options) -> Result<DynamicImage, Error> {
    use Format::*;
    match format {
        Jpeg | Png | WebP | Bmp | Tiff | Gif => engines::raster::decode(src, format),
        Heic => engines::heic::decode(src),
        Psd => engines::psd::decode(src),
        Ai => engines::pdf::render_first_page(src, opts),
        Svg => engines::vector::render(src, opts),
        other => Err(Error::Unsupported(format!("{other} files have no single-image form."))),
    }
}

/// Reserves an Output file, runs `write` into it, and removes it again if writing fails.
pub(crate) fn write_output(
    src: &Path,
    ext: &str,
    suffix: &str,
    opts: &Options,
    write: impl FnOnce(&mut dyn Write) -> Result<(), Error>,
) -> Result<PathBuf, Error> {
    let (path, mut file) = output::reserve(src, ext, suffix, opts)?;
    let result = write(&mut file).and_then(|_| file.flush().map_err(|e| Error::io(&path, e)));
    drop(file);
    match result {
        Ok(()) => Ok(path),
        Err(e) => {
            output::discard(std::slice::from_ref(&path));
            Err(e)
        }
    }
}
