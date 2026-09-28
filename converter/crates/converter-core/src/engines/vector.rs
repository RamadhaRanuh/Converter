//! SVG: render to pixels (resvg), trace pixels to SVG (vtracer), optimize SVG (oxvg), SVG → PDF (svg2pdf).
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use image::{DynamicImage, RgbaImage};
use resvg::{tiny_skia, usvg};

use crate::{Error, Options};

/// System fonts, loaded once: the slow part of opening an SVG that contains text.
fn fontdb() -> Arc<usvg::fontdb::Database> {
    static DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    })
    .clone()
}

fn pdf_fontdb() -> Arc<svg2pdf::usvg::fontdb::Database> {
    static DB: OnceLock<Arc<svg2pdf::usvg::fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = svg2pdf::usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    })
    .clone()
}

fn read(src: &Path) -> Result<Vec<u8>, Error> {
    std::fs::read(src).map_err(|e| Error::io(src, e))
}

/// Renders an SVG at `opts.dpi` (SVG user units are CSS pixels at 96 DPI), keeping transparency.
pub(crate) fn render(src: &Path, opts: &Options) -> Result<DynamicImage, Error> {
    let data = read(src)?;
    let uopts = usvg::Options { resources_dir: src.parent().map(Path::to_path_buf), fontdb: fontdb(), ..Default::default() };
    let tree = usvg::Tree::from_data(&data, &uopts).map_err(|e| Error::decode(format!("SVG: {e}")))?;
    let scale = opts.dpi as f32 / 96.0;
    let size = tree.size().to_int_size().scale_by(scale).ok_or_else(|| Error::decode("SVG has no size"))?;
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| Error::Unsupported("This SVG is too large to render at this DPI; lower the DPI in Options.".into()))?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let px: Vec<u8> = pixmap.pixels().iter().flat_map(|p| {
        let c = p.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect();
    Ok(DynamicImage::ImageRgba8(RgbaImage::from_raw(size.width(), size.height(), px).expect("pixmap size")))
}

/// Traces a raster image into colour SVG paths.
pub(crate) fn trace(img: &DynamicImage, out: &mut dyn Write) -> Result<(), Error> {
    let rgba = img.to_rgba8();
    let color = vtracer::ColorImage { width: rgba.width() as usize, height: rgba.height() as usize, pixels: rgba.into_raw() };
    let svg = vtracer::convert(color, vtracer::Config::default()).map_err(|e| Error::encode("SVG", e))?;
    out.write_all(svg.to_string().as_bytes()).map_err(|e| Error::encode("SVG", e))
}

/// Optimizes an SVG like SVGO's default preset. Falls back to an error (never a broken file) if parsing fails.
pub(crate) fn optimize(src: &Path, out: &mut dyn Write) -> Result<(), Error> {
    use oxvg_ast::parse::roxmltree::parse;
    use oxvg_ast::serialize::Node as _;
    use oxvg_ast::visitor::Info;

    let text = String::from_utf8(read(src)?).map_err(|_| Error::decode("SVG is not UTF-8 text"))?;
    let result = parse(&text, |dom, allocator| {
        oxvg_optimiser::Jobs::default().run(dom, &Info::new(allocator)).map_err(|e| e.to_string())?;
        dom.serialize().map_err(|e| e.to_string())
    })
    .map_err(|e| Error::decode(format!("SVG: {e}")))?
    .map_err(|e| Error::encode("SVG", e))?;
    // Only keep the optimized document if it still parses as SVG.
    usvg::Tree::from_str(&result, &usvg::Options::default()).map_err(|e| Error::encode("SVG", format!("optimizer produced invalid SVG: {e}")))?;
    out.write_all(result.as_bytes()).map_err(|e| Error::encode("SVG", e))
}

/// Converts an SVG to a vector PDF (text and shapes stay sharp), one page sized to the SVG.
pub(crate) fn svg_to_pdf(src: &Path, out: &mut dyn Write) -> Result<(), Error> {
    use svg2pdf::usvg as u;
    let data = read(src)?;
    let uopts = u::Options { resources_dir: src.parent().map(Path::to_path_buf), fontdb: pdf_fontdb(), ..Default::default() };
    let tree = u::Tree::from_data(&data, &uopts).map_err(|e| Error::decode(format!("SVG: {e}")))?;
    // SVG user units are CSS pixels (96 per inch), so a 96 px wide SVG becomes a 1 inch wide page.
    let page = svg2pdf::PageOptions { dpi: 96.0 };
    let pdf = svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), page)
        .map_err(|e| Error::encode("PDF", format!("{e:?}")))?;
    out.write_all(&pdf).map_err(|e| Error::encode("PDF", e))
}

#[cfg(test)]
mod tests {
    use crate::engines::raster::tests::sample;
    use crate::{Format, Options, Target, convert};

    const SVG: &str = r##"<?xml version="1.0"?>
<!-- a comment the optimizer should drop -->
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 100">
  <g><g><rect x="0" y="0" width="100" height="100" fill="#ff0000"/></g></g>
  <circle cx="150" cy="50" r="40" fill="#0000ff" fill-opacity="0.5"/>
</svg>"##;

    #[test]
    fn svg_renders_at_dpi_with_transparency() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.svg");
        std::fs::write(&p, SVG).unwrap();
        let out = convert(&p, Target::Image(Format::Png), &Options { dpi: 192, ..Default::default() }).unwrap();
        let img = image::open(&out[0]).unwrap().to_rgba8();
        assert_eq!(img.dimensions(), (400, 200), "96 DPI user units at 192 DPI double");
        assert_eq!(img.get_pixel(50, 100).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(399, 0).0[3], 0, "outside shapes stays transparent");
    }

    #[test]
    fn svg_optimizes_smaller_and_still_renders_the_same() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.svg");
        std::fs::write(&p, SVG).unwrap();
        let out = convert(&p, Target::Svg, &Options::default()).unwrap();
        let opt = std::fs::read_to_string(&out[0]).unwrap();
        assert!(opt.len() < SVG.len(), "{opt}");
        assert!(!opt.contains("comment"));
        let a = super::render(&p, &Options::default()).unwrap();
        let b = super::render(&out[0], &Options::default()).unwrap();
        assert!(crate::engines::raster::tests::mean_diff(&a, &b) < 0.5);
    }

    #[test]
    fn svg_to_pdf_is_one_renderable_page() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.svg");
        std::fs::write(&p, SVG).unwrap();
        let pdf = convert(&p, Target::Pdf, &Options::default()).unwrap();
        let pages = convert(&pdf[0], Target::PdfPages(Format::Png), &Options { dpi: 96, ..Default::default() }).unwrap();
        assert_eq!(pages.len(), 1);
        let img = image::open(&pages[0]).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (200, 100));
        let px = img.get_pixel(50, 50).0;
        assert!(px[0] > 240 && px[1] < 20 && px[2] < 20, "{px:?}");
    }

    #[test]
    fn raster_traces_to_valid_svg() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.png");
        sample(64, 48, false).save(&p).unwrap();
        let out = convert(&p, Target::Svg, &Options::default()).unwrap();
        let svg = std::fs::read_to_string(&out[0]).unwrap();
        assert!(svg.contains("<svg") && svg.contains("<path"));
        let back = super::render(&out[0], &Options { dpi: 96, ..Default::default() }).unwrap();
        assert_eq!((back.width(), back.height()), (64, 48));
    }
}
