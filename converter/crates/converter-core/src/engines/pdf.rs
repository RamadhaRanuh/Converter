//! PDF: render pages, split, merge, and images → PDF. Also renders modern (PDF-based) Illustrator files.
use std::io::Write;
use std::path::{Path, PathBuf};

use image::DynamicImage;

use crate::{Error, Format, Options};

pub(crate) enum Part {
    Pdf(Vec<u8>),
    Image(DynamicImage),
}

pub(crate) fn render_pages(_src: &Path, _img: Format, _opts: &Options) -> Result<Vec<PathBuf>, Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn render_first_page(_src: &Path, _opts: &Options) -> Result<DynamicImage, Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn split(_src: &Path, _opts: &Options) -> Result<Vec<PathBuf>, Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn ai_to_pdf(_src: &Path, _opts: &Options) -> Result<PathBuf, Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn images_to_pdf(_imgs: &[DynamicImage], _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn combine(_parts: Vec<Part>, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}
