use std::fmt;

use crate::Format;

/// What the user wants a Source file turned into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// A single raster image: JPG, PNG, WebP, BMP, TIFF or GIF.
    Image(Format),
    /// A flattened single-layer Photoshop file.
    Psd,
    /// SVG: traced from a raster image, or optimized from an SVG.
    Svg,
    /// One PDF.
    Pdf,
    /// One image per PDF page (PNG or JPG).
    PdfPages(Format),
    /// One single-page PDF per page of the source PDF.
    PdfSplit,
    /// HTML (from Markdown).
    Html,
}

impl Target {
    /// Every Target the app knows, in the order the UI shows them.
    pub const ALL: &'static [Target] = &[
        Target::Image(Format::Jpeg),
        Target::Image(Format::Png),
        Target::Image(Format::WebP),
        Target::Image(Format::Bmp),
        Target::Image(Format::Tiff),
        Target::Image(Format::Gif),
        Target::Psd,
        Target::Svg,
        Target::Pdf,
        Target::PdfPages(Format::Png),
        Target::PdfPages(Format::Jpeg),
        Target::PdfSplit,
        Target::Html,
    ];

    /// Short label for buttons and the CLI's `--to` flag help.
    pub fn label(self) -> String {
        match self {
            Target::Image(f) => f.to_string(),
            Target::Psd => "PSD".into(),
            Target::Svg => "SVG".into(),
            Target::Pdf => "PDF".into(),
            Target::PdfPages(f) => format!("{f} (per page)"),
            Target::PdfSplit => "PDF (split pages)".into(),
            Target::Html => "HTML".into(),
        }
    }

    /// Parses the CLI's `--to` value: `jpg`, `png`, `webp`, `bmp`, `tiff`, `gif`, `psd`, `svg`, `pdf`,
    /// `png-pages`, `jpg-pages`, `pdf-split`, `html`.
    pub fn parse(s: &str) -> Option<Target> {
        Some(match s.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Target::Image(Format::Jpeg),
            "png" => Target::Image(Format::Png),
            "webp" => Target::Image(Format::WebP),
            "bmp" => Target::Image(Format::Bmp),
            "tif" | "tiff" => Target::Image(Format::Tiff),
            "gif" => Target::Image(Format::Gif),
            "psd" => Target::Psd,
            "svg" => Target::Svg,
            "pdf" => Target::Pdf,
            "png-pages" => Target::PdfPages(Format::Png),
            "jpg-pages" | "jpeg-pages" => Target::PdfPages(Format::Jpeg),
            "pdf-split" | "split" => Target::PdfSplit,
            "html" => Target::Html,
            _ => return None,
        })
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

const IMAGES: [Target; 6] = [
    Target::Image(Format::Jpeg),
    Target::Image(Format::Png),
    Target::Image(Format::WebP),
    Target::Image(Format::Bmp),
    Target::Image(Format::Tiff),
    Target::Image(Format::Gif),
];

macro_rules! with_images {
    ($($extra:expr),*) => {
        &[IMAGES[0], IMAGES[1], IMAGES[2], IMAGES[3], IMAGES[4], IMAGES[5], $($extra),*]
    };
}

/// The v1 conversion matrix: every Target a Source format can become.
pub fn targets_for(source: Format) -> &'static [Target] {
    use Format::*;
    match source {
        Jpeg | Png | WebP | Bmp | Tiff | Gif => with_images!(Target::Psd, Target::Svg, Target::Pdf),
        Heic => with_images!(Target::Pdf),
        Psd => with_images!(Target::Pdf),
        Ai => with_images!(Target::Pdf),
        Svg => with_images!(Target::Svg, Target::Pdf),
        Pdf => &[Target::PdfPages(Png), Target::PdfPages(Jpeg), Target::PdfSplit],
        Docx | Pptx | Xlsx => &[Target::Pdf],
        Markdown => &[Target::Pdf, Target::Html],
    }
}

/// Whether a Source format can take part in a Combine into one PDF.
pub fn can_combine(source: Format) -> bool {
    matches!(source, Format::Jpeg | Format::Png | Format::WebP | Format::Bmp | Format::Tiff | Format::Gif | Format::Heic | Format::Pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_target_parses_from_its_cli_name() {
        for name in ["jpg", "png", "webp", "bmp", "tiff", "gif", "psd", "svg", "pdf", "png-pages", "jpg-pages", "pdf-split", "html"] {
            assert!(Target::parse(name).is_some(), "{name}");
        }
        assert_eq!(Target::parse("avif"), None);
    }

    #[test]
    fn matrix_matches_the_v1_decisions() {
        assert!(targets_for(Format::Heic).contains(&Target::Image(Format::Jpeg)));
        assert!(!targets_for(Format::Heic).contains(&Target::Psd), "PSD out is only from raster");
        assert!(!targets_for(Format::Psd).contains(&Target::Psd));
        assert_eq!(targets_for(Format::Docx), &[Target::Pdf]);
        assert!(targets_for(Format::Svg).contains(&Target::Svg));
        assert!(Target::ALL.iter().all(|t| Format::Pdf
            != match t {
                Target::Image(f) => *f,
                _ => Format::Jpeg,
            }));
    }
}
