use std::fmt;
use std::io::Read;
use std::path::Path;

use crate::Error;

/// A file format the app can read or write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Bmp,
    Tiff,
    Gif,
    Heic,
    Psd,
    Ai,
    Svg,
    Pdf,
    Docx,
    Pptx,
    Xlsx,
    Markdown,
}

/// A group of formats that share a representation of content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatFamily {
    Raster,
    Layered,
    Vector,
    Document,
}

impl Format {
    pub fn family(self) -> FormatFamily {
        use Format::*;
        match self {
            Jpeg | Png | WebP | Bmp | Tiff | Gif | Heic => FormatFamily::Raster,
            Psd => FormatFamily::Layered,
            Ai | Svg => FormatFamily::Vector,
            Pdf | Docx | Pptx | Xlsx | Markdown => FormatFamily::Document,
        }
    }

    /// File extension used for Output files in this format.
    pub fn extension(self) -> &'static str {
        use Format::*;
        match self {
            Jpeg => "jpg",
            Png => "png",
            WebP => "webp",
            Bmp => "bmp",
            Tiff => "tiff",
            Gif => "gif",
            Heic => "heic",
            Psd => "psd",
            Ai => "ai",
            Svg => "svg",
            Pdf => "pdf",
            Docx => "docx",
            Pptx => "pptx",
            Xlsx => "xlsx",
            Markdown => "md",
        }
    }

    fn from_extension(ext: &str) -> Option<Format> {
        use Format::*;
        Some(match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" | "jpe" | "jfif" => Jpeg,
            "png" => Png,
            "webp" => WebP,
            "bmp" | "dib" => Bmp,
            "tif" | "tiff" => Tiff,
            "gif" => Gif,
            "heic" | "heif" | "hif" => Heic,
            "psd" => Psd,
            "ai" => Ai,
            "svg" => Svg,
            "pdf" => Pdf,
            "docx" => Docx,
            "pptx" => Pptx,
            "xlsx" => Xlsx,
            "md" | "markdown" => Markdown,
            _ => return None,
        })
    }

}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Format::*;
        f.write_str(match self {
            Jpeg => "JPG",
            Png => "PNG",
            WebP => "WebP",
            Bmp => "BMP",
            Tiff => "TIFF",
            Gif => "GIF",
            Heic => "HEIC",
            Psd => "PSD",
            Ai => "AI",
            Svg => "SVG",
            Pdf => "PDF",
            Docx => "Word",
            Pptx => "PowerPoint",
            Xlsx => "Excel",
            Markdown => "Markdown",
        })
    }
}

/// Works out a Source file's format from its first bytes, falling back to its extension
/// for formats that have no reliable signature (Markdown) or share one (ZIP-based Office files).
pub fn detect(path: &Path) -> Result<Format, Error> {
    let mut head = [0u8; 4096];
    let n = std::fs::File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .map_err(|e| Error::io(path, e))?;
    let head = &head[..n];
    let by_ext = path.extension().and_then(|e| e.to_str()).and_then(Format::from_extension);

    let sniffed = if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(Format::Jpeg)
    } else if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(Format::Png)
    } else if head.starts_with(b"GIF8") {
        Some(Format::Gif)
    } else if head.starts_with(b"BM") && by_ext == Some(Format::Bmp) {
        Some(Format::Bmp)
    } else if head.starts_with(b"II*\0") || head.starts_with(b"MM\0*") {
        Some(Format::Tiff)
    } else if head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Some(Format::WebP)
    } else if head.starts_with(b"8BPS") {
        Some(Format::Psd)
    } else if head.len() >= 12 && &head[4..8] == b"ftyp" && is_heif_brand(&head[8..12]) {
        Some(Format::Heic)
    } else if head.starts_with(b"%PDF") {
        // Modern Illustrator files are PDF inside; keep the user's intent from the extension.
        Some(if by_ext == Some(Format::Ai) { Format::Ai } else { Format::Pdf })
    } else if head.starts_with(b"%!PS") && by_ext == Some(Format::Ai) {
        Some(Format::Ai) // legacy PostScript .ai: detected so it can fail with a helpful message
    } else if head.starts_with(b"PK\x03\x04") {
        by_ext.filter(|f| matches!(f, Format::Docx | Format::Pptx | Format::Xlsx))
    } else if looks_like_svg(head) {
        Some(Format::Svg)
    } else {
        None
    };

    match sniffed.or(by_ext.filter(|f| matches!(f, Format::Markdown | Format::Svg))) {
        Some(f) => Ok(f),
        None => Err(Error::UnknownFormat),
    }
}

fn is_heif_brand(brand: &[u8]) -> bool {
    matches!(brand, b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx" | b"mif1" | b"msf1")
}

fn looks_like_svg(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(head);
    let t = text.trim_start_matches('\u{feff}').trim_start();
    (t.starts_with("<?xml") || t.starts_with("<svg") || t.starts_with("<!--")) && text.contains("<svg")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect_bytes(name: &str, bytes: &[u8]) -> Result<Format, Error> {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(name);
        std::fs::write(&p, bytes).unwrap();
        detect(&p)
    }

    #[test]
    fn detects_by_signature_even_with_wrong_extension() {
        assert_eq!(detect_bytes("photo.png", &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]).unwrap(), Format::Jpeg);
        assert_eq!(detect_bytes("x.bin", b"%PDF-1.7\n").unwrap(), Format::Pdf);
        assert_eq!(detect_bytes("x", b"8BPS\0\x01").unwrap(), Format::Psd);
        assert_eq!(detect_bytes("x", b"RIFF\0\0\0\0WEBPVP8 ").unwrap(), Format::WebP);
        assert_eq!(detect_bytes("x", b"\0\0\0\x18ftypheic\0\0").unwrap(), Format::Heic);
    }

    #[test]
    fn pdf_named_ai_is_ai_and_postscript_ai_is_ai() {
        assert_eq!(detect_bytes("logo.ai", b"%PDF-1.5\n").unwrap(), Format::Ai);
        assert_eq!(detect_bytes("old.ai", b"%!PS-Adobe-3.0\n").unwrap(), Format::Ai);
        assert!(detect_bytes("old.eps", b"%!PS-Adobe-3.0\n").is_err());
    }

    #[test]
    fn zip_needs_office_extension_and_markdown_needs_extension() {
        assert_eq!(detect_bytes("a.docx", b"PK\x03\x04rest").unwrap(), Format::Docx);
        assert!(detect_bytes("a.zip", b"PK\x03\x04rest").is_err());
        assert_eq!(detect_bytes("notes.md", b"# hi").unwrap(), Format::Markdown);
        assert_eq!(detect_bytes("a", b"<?xml version=\"1.0\"?><svg></svg>").unwrap(), Format::Svg);
    }
}
