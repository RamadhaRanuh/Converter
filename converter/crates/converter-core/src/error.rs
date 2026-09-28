use std::path::Path;

use crate::{Format, Target};

/// Why a Conversion or Combine failed. `Display` is written for the person using the app:
/// what went wrong and, where there is one, how to fix it.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("This file type isn't supported.")]
    UnknownFormat,
    #[error("A {from} file can't be converted to {to}.")]
    NotPossible { from: Format, to: Target },
    #[error("Couldn't read or write {path}: {source}")]
    Io { path: String, source: std::io::Error },
    #[error("The file looks damaged or uses a feature that isn't supported ({0}).")]
    Decode(String),
    #[error("Couldn't write the {format} file ({reason}).")]
    Encode { format: String, reason: String },
    #[error("This PSD has no stored composite image. Re-save it in Photoshop with \"Maximize compatibility\" turned on.")]
    PsdNoComposite,
    #[error("This is a legacy Illustrator file (saved without PDF content). Re-save it in Illustrator with \"Create PDF Compatible File\" turned on.")]
    AiLegacy,
    #[error("{0}")]
    Unsupported(String),
    #[error("Only images and PDFs can be combined into one PDF.")]
    CombineNotPossible(Format),
    #[error("The converter hit an internal error on this file ({0}).")]
    Panicked(String),
    #[error("Cancelled.")]
    Cancelled,
}

impl Error {
    pub(crate) fn io(path: &Path, source: std::io::Error) -> Self {
        Error::Io { path: path.display().to_string(), source }
    }

    pub(crate) fn decode(e: impl std::fmt::Display) -> Self {
        Error::Decode(e.to_string())
    }

    pub(crate) fn encode(format: impl std::fmt::Display, e: impl std::fmt::Display) -> Self {
        Error::Encode { format: format.to_string(), reason: e.to_string() }
    }
}
