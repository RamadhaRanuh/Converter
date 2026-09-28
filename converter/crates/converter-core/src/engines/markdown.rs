//! Markdown → HTML and PDF.
use std::io::Write;
use std::path::Path;

use crate::Error;

pub(crate) fn to_html(_src: &Path, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}

pub(crate) fn to_pdf(_src: &Path, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}
