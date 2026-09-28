//! SVG: render, trace, optimize, and SVG → PDF.
use std::io::Write;
use std::path::Path;

use image::DynamicImage;

use crate::{Error, Options};

pub(crate) fn render(_src: &Path, _opts: &Options) -> Result<DynamicImage, Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn trace(_img: &DynamicImage, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn optimize(_src: &Path, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn svg_to_pdf(_src: &Path, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}
