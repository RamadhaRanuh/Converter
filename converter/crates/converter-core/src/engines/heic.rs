//! HEIC input (iPhone photos).
use std::path::Path;

use image::DynamicImage;

use crate::Error;

pub(crate) fn decode(_path: &Path) -> Result<DynamicImage, Error> {
    Err(Error::Unsupported("HEIC support isn't built into this version.".into()))
}
