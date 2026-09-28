//! Word, PowerPoint and Excel → PDF with office2pdf (pure Rust, Typst layout).
use std::io::Write;
use std::path::Path;

use office2pdf::config::{ConvertOptions, Format as OfficeFormat};

use crate::{Error, Format};

pub(crate) fn to_pdf(src: &Path, format: Format, out: &mut dyn Write) -> Result<(), Error> {
    let kind = match format {
        Format::Docx => OfficeFormat::Docx,
        Format::Pptx => OfficeFormat::Pptx,
        Format::Xlsx => OfficeFormat::Xlsx,
        other => return Err(Error::Unsupported(format!("{other} isn't an Office format."))),
    };
    let data = std::fs::read(src).map_err(|e| Error::io(src, e))?;
    let result = office2pdf::convert_bytes(&data, kind, &ConvertOptions::default()).map_err(|e| Error::decode(format!("{format}: {e}")))?;
    out.write_all(&result.pdf).map_err(|e| Error::encode("PDF", e))
}

#[cfg(test)]
mod tests {
    use crate::{Format, Options, Target, convert};

    /// A real (Word-compatible) DOCX with one paragraph, built with docx-rs.
    pub(crate) fn tiny_docx(path: &std::path::Path, text: &str) {
        let f = std::fs::File::create(path).unwrap();
        docx_rs::Docx::new()
            .add_paragraph(docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text(text).size(48)))
            .build()
            .pack(f)
            .unwrap();
    }

    #[test]
    fn docx_becomes_a_pdf_with_the_text_drawn() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("letter.docx");
        tiny_docx(&p, "Hello from the converter test suite");
        let pdf = convert(&p, Target::Pdf, &Options::default()).unwrap();
        let pages = convert(&pdf[0], Target::PdfPages(Format::Png), &Options { dpi: 50, ..Default::default() }).unwrap();
        assert_eq!(pages.len(), 1);
        let img = image::open(&pages[0]).unwrap().to_luma8();
        assert!(img.pixels().any(|p| p.0[0] < 100), "some dark text pixels should be drawn");
    }
}
