//! Word, PowerPoint and Excel → PDF.
use std::io::Write;
use std::path::Path;

use crate::{Error, Format};

pub(crate) fn to_pdf(_src: &Path, _format: Format, _out: &mut dyn Write) -> Result<(), Error> {
    Err(Error::Unsupported("not yet supported".into()))
}
