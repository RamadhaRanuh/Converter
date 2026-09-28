//! PDF: render pages (hayro), split and merge (lopdf), and images → PDF.
//! Also renders modern Illustrator files, which are PDF inside.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flate2::Compression;
use flate2::write::ZlibEncoder;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::{TRANSPARENT, WHITE};
use hayro::{RenderCache, RenderSettings, render};
use image::{DynamicImage, RgbaImage};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream, dictionary};
use rayon::prelude::*;

use crate::engines::raster;
use crate::{Error, Format, Options, output, write_output};

/// One input to a Combine.
pub(crate) enum Part {
    Pdf(Vec<u8>),
    Image(DynamicImage),
}

/// Parses a PDF. Takes an `Arc` so parallel workers share one copy of the file's bytes.
fn parse(data: Arc<Vec<u8>>) -> Result<Pdf, Error> {
    Pdf::new(data).map_err(|e| match format!("{e:?}") {
        s if s.contains("Encrypt") || s.contains("Password") => Error::Unsupported("This PDF is password-protected.".into()),
        s => Error::decode(format!("PDF: {s}")),
    })
}

fn render_page(pdf: &Pdf, index: usize, dpi: u16, transparent: bool) -> Result<DynamicImage, Error> {
    let page = pdf.pages().get(index).ok_or_else(|| Error::decode("page out of range"))?;
    let scale = dpi as f32 / 72.0;
    let (w, h) = page.render_dimensions();
    if w * scale > u16::MAX as f32 || h * scale > u16::MAX as f32 {
        return Err(Error::Unsupported(format!("Page {} is too large to render at {dpi} DPI; lower the DPI in Options.", index + 1)));
    }
    let settings =
        RenderSettings { x_scale: scale, y_scale: scale, bg_color: if transparent { TRANSPARENT } else { WHITE }, ..Default::default() };
    let pixmap = render(page, &RenderCache::new(), &InterpreterSettings::default(), &settings);
    let (pw, ph) = (pixmap.width() as u32, pixmap.height() as u32);
    let px: Vec<u8> = pixmap.take_unpremultiplied().into_iter().flat_map(|p| [p.r, p.g, p.b, p.a]).collect();
    let img = RgbaImage::from_raw(pw, ph, px).ok_or_else(|| Error::decode("renderer returned a bad buffer"))?;
    Ok(if transparent { DynamicImage::ImageRgba8(img) } else { DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(img).to_rgb8()) })
}

/// PDF → one image per page. Pages render in parallel: each thread parses its own copy of the document,
/// because hayro's render cache can't be shared across threads.
pub(crate) fn render_pages(src: &Path, format: Format, opts: &Options) -> Result<Vec<PathBuf>, Error> {
    let data = Arc::new(std::fs::read(src).map_err(|e| Error::io(src, e))?);
    let n = parse(data.clone())?.pages().len();
    if n == 0 {
        return Err(Error::decode("the PDF has no pages"));
    }
    let digits = n.to_string().len();
    let threads = rayon::current_num_threads().max(1);
    let chunk = n.div_ceil(threads);
    let results: Vec<Result<Vec<PathBuf>, Error>> = (0..n)
        .step_by(chunk)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|start| {
            let pdf = parse(data.clone())?;
            (start..(start + chunk).min(n))
                .map(|i| {
                    let img = render_page(&pdf, i, opts.dpi, false)?;
                    let suffix = format!("-p{:0digits$}", i + 1);
                    write_output(src, format.extension(), &suffix, opts, |w| raster::encode(&img, format, opts, w))
                })
                .collect()
        })
        .collect();
    collect_all(results)
}

/// Gathers per-chunk results; on any failure, deletes what was already written so no partial set is left.
fn collect_all(results: Vec<Result<Vec<PathBuf>, Error>>) -> Result<Vec<PathBuf>, Error> {
    let mut ok = Vec::new();
    let mut first_err = None;
    for r in results {
        match r {
            Ok(v) => ok.extend(v),
            Err(e) => first_err = first_err.or(Some(e)),
        }
    }
    match first_err {
        None => {
            ok.sort();
            Ok(ok)
        }
        Some(e) => {
            output::discard(&ok);
            Err(e)
        }
    }
}

fn check_modern_ai(data: &[u8]) -> Result<(), Error> {
    if data.starts_with(b"%PDF") { Ok(()) } else { Err(Error::AiLegacy) }
}

/// Renders the first page (the artboard) of a modern Illustrator file, keeping transparency.
pub(crate) fn render_first_page(src: &Path, opts: &Options) -> Result<DynamicImage, Error> {
    let data = std::fs::read(src).map_err(|e| Error::io(src, e))?;
    check_modern_ai(&data)?;
    let pdf = parse(Arc::new(data))?;
    if pdf.pages().is_empty() {
        return Err(Error::AiLegacy);
    }
    render_page(&pdf, 0, opts.dpi, true)
}

/// A modern Illustrator file already is a PDF; the Output file is a verified copy.
pub(crate) fn ai_to_pdf(src: &Path, opts: &Options) -> Result<PathBuf, Error> {
    let data = std::fs::read(src).map_err(|e| Error::io(src, e))?;
    check_modern_ai(&data)?;
    let data = Arc::new(data);
    parse(data.clone())?;
    write_output(src, "pdf", "", opts, |w| w.write_all(&data).map_err(|e| Error::encode("PDF", e)))
}

fn load(data: &[u8]) -> Result<Document, Error> {
    let doc = Document::load_mem(data).map_err(|e| Error::decode(format!("PDF: {e}")))?;
    if doc.is_encrypted() {
        return Err(Error::Unsupported("This PDF is password-protected.".into()));
    }
    Ok(doc)
}

fn save(doc: &mut Document, out: &mut dyn Write) -> Result<(), Error> {
    doc.compress();
    doc.save_to(&mut std::io::BufWriter::new(out)).map_err(|e| Error::encode("PDF", e))
}

/// PDF → one single-page PDF per page.
pub(crate) fn split(src: &Path, opts: &Options) -> Result<Vec<PathBuf>, Error> {
    let data = std::fs::read(src).map_err(|e| Error::io(src, e))?;
    let doc = load(&data)?;
    let pages: Vec<u32> = doc.get_pages().into_keys().collect();
    let digits = pages.len().to_string().len();
    let results: Vec<Result<Vec<PathBuf>, Error>> = pages
        .par_iter()
        .map(|&keep| {
            let mut one = doc.clone();
            let others: Vec<u32> = pages.iter().copied().filter(|&p| p != keep).collect();
            one.delete_pages(&others);
            one.prune_objects();
            let suffix = format!("-p{:0digits$}", keep);
            write_output(src, "pdf", &suffix, opts, |w| save(&mut one, w)).map(|p| vec![p])
        })
        .collect();
    collect_all(results)
}

/// Copies inheritable page attributes down from ancestor Pages nodes, so a page stays correct
/// when it is moved under a different parent.
fn push_down_inherited(doc: &mut Document, page: ObjectId) {
    const KEYS: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];
    let mut found: Vec<(&[u8], Object)> = Vec::new();
    let Ok(dict) = doc.get_dictionary(page) else { return };
    let mut missing: Vec<&[u8]> = KEYS.iter().copied().filter(|k| !dict.has(k)).collect();
    let mut parent = dict.get(b"Parent").and_then(Object::as_reference).ok();
    let mut guard = 0;
    while let (Some(pid), false) = (parent, missing.is_empty()) {
        guard += 1;
        let Ok(pd) = doc.get_dictionary(pid) else { break };
        if guard > 64 {
            break;
        }
        missing.retain(|k| match pd.get(k) {
            Ok(v) => {
                found.push((k, v.clone()));
                false
            }
            Err(_) => true,
        });
        parent = pd.get(b"Parent").and_then(Object::as_reference).ok();
    }
    if let Ok(d) = doc.get_dictionary_mut(page) {
        for (k, v) in found {
            d.set(k.to_vec(), v);
        }
    }
}

/// Appends every page of `doc` to `out`, returning the new page ids in order.
fn absorb(out: &mut Document, mut doc: Document) -> Vec<ObjectId> {
    let page_ids: Vec<ObjectId> = doc.get_pages().into_values().collect();
    for &p in &page_ids {
        push_down_inherited(&mut doc, p);
    }
    doc.renumber_objects_with(out.max_id + 1);
    let page_ids: Vec<ObjectId> = doc.get_pages().into_values().collect();
    out.max_id = doc.max_id;
    for (id, obj) in doc.objects {
        match obj.type_name().unwrap_or(b"") {
            b"Catalog" | b"Pages" | b"Outlines" | b"Outline" => {}
            _ => {
                out.objects.insert(id, obj);
            }
        }
    }
    page_ids
}

/// Builds a PDF from images (each on its own A4 page, fitted and centred) and existing PDFs, in order.
pub(crate) fn combine(parts: Vec<Part>, quality: u8, out: &mut dyn Write) -> Result<(), Error> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let mut kids = Vec::new();
    for part in parts {
        match part {
            Part::Pdf(bytes) => kids.extend(absorb(&mut doc, load(&bytes)?)),
            Part::Image(img) => kids.push(add_image_page(&mut doc, &img, quality)?),
        }
    }
    for &k in &kids {
        if let Ok(d) = doc.get_dictionary_mut(k) {
            d.set("Parent", pages_id);
        }
    }
    let count = kids.len() as i64;
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
            "Count" => count,
        }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    save(&mut doc, out)
}

/// Images → PDF, one A4 page per image.
pub(crate) fn images_to_pdf(imgs: &[DynamicImage], quality: u8, out: &mut dyn Write) -> Result<(), Error> {
    combine(imgs.iter().cloned().map(Part::Image).collect(), quality, out)
}

fn zlib(data: &[u8]) -> Vec<u8> {
    let mut z = ZlibEncoder::new(Vec::new(), Compression::fast());
    z.write_all(data).expect("in-memory write");
    z.finish().expect("in-memory write")
}

/// Adds one A4 page (portrait or landscape to match the image) showing `img` fitted and centred.
/// Opaque images are embedded as JPEG; images with transparency as lossless Flate with a soft mask.
fn add_image_page(doc: &mut Document, img: &DynamicImage, quality: u8) -> Result<ObjectId, Error> {
    let (w, h) = (img.width(), img.height());
    let (base, mut xobj) = (
        dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => w as i64, "Height" => h as i64, "BitsPerComponent" => 8, "ColorSpace" => "DeviceRGB" },
        Vec::new(),
    );
    let mut dict: Dictionary = base;
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        let rgb: Vec<u8> = rgba.pixels().flat_map(|p| [p[0], p[1], p[2]]).collect();
        let alpha: Vec<u8> = rgba.pixels().map(|p| p[3]).collect();
        let smask = doc.add_object(Stream::new(
            dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => w as i64, "Height" => h as i64, "BitsPerComponent" => 8, "ColorSpace" => "DeviceGray", "Filter" => "FlateDecode" },
            zlib(&alpha),
        ));
        dict.set("Filter", "FlateDecode");
        dict.set("SMask", smask);
        xobj = zlib(&rgb);
    } else {
        raster::encode(img, Format::Jpeg, &Options { quality, ..Default::default() }, &mut xobj)?;
        dict.set("Filter", "DCTDecode");
    }
    let image_id = doc.add_object(Stream::new(dict, xobj).with_compression(false));

    let (pw, ph) = if w > h { (842.0, 595.0) } else { (595.0, 842.0) };
    let s = f64::min(pw / w as f64, ph / h as f64);
    let (dw, dh) = (w as f64 * s, h as f64 * s);
    let (x, y) = ((pw - dw) / 2.0, (ph - dh) / 2.0);
    let content = format!("q {dw:.3} 0 0 {dh:.3} {x:.3} {y:.3} cm /Im0 Do Q");
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
    Ok(doc.add_object(dictionary! {
        "Type" => "Page",
        "MediaBox" => vec![0.into(), 0.into(), Object::Real(pw as f32), Object::Real(ph as f32)],
        "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => image_id } },
        "Contents" => content_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::raster::tests::{mean_diff, sample};
    use crate::{Target, convert};

    fn pdf_from_images(dir: &Path, name: &str, imgs: &[DynamicImage]) -> PathBuf {
        let p = dir.join(name);
        images_to_pdf(imgs, 90, &mut std::fs::File::create(&p).unwrap()).unwrap();
        p
    }

    fn page_count(p: &Path) -> usize {
        Document::load(p).unwrap().get_pages().len()
    }

    #[test]
    fn images_to_pdf_and_render_back() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(300, 200, false);
        let pdf = pdf_from_images(dir.path(), "a.pdf", &[src.clone(), sample(100, 300, true)]);
        assert_eq!(page_count(&pdf), 2);
        let outs = convert(&pdf, Target::PdfPages(Format::Png), &Options { dpi: 72, ..Default::default() }).unwrap();
        assert_eq!(outs.len(), 2);
        assert!(outs[0].ends_with("a-p1.png") && outs[1].ends_with("a-p2.png"));
        let page1 = image::open(&outs[0]).unwrap();
        assert_eq!((page1.width(), page1.height()), (842, 595), "landscape A4 at 72 DPI");
        // The image fills the page width; its centre pixel should match the source's centre closely.
        let c = page1.to_rgb8().get_pixel(421, 297).0;
        let s = src.to_rgb8().get_pixel(150, 100).0;
        assert!(c.iter().zip(s).all(|(a, b)| a.abs_diff(b) < 12), "{c:?} vs {s:?}");
    }

    #[test]
    fn split_then_combine_round_trips_page_count() {
        let dir = tempfile::tempdir().unwrap();
        let pdf = pdf_from_images(dir.path(), "doc.pdf", &[sample(50, 50, false), sample(60, 40, false), sample(40, 60, false)]);
        let parts = convert(&pdf, Target::PdfSplit, &Options::default()).unwrap();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| page_count(p) == 1));
        let merged = crate::combine(&[parts[2].clone(), parts[0].clone(), pdf.clone()], &Options::default()).unwrap();
        assert_eq!(page_count(&merged), 5);
        // Every page of the merged file must still render.
        let pages = convert(&merged, Target::PdfPages(Format::Jpeg), &Options { dpi: 30, ..Default::default() }).unwrap();
        assert_eq!(pages.len(), 5);
    }

    #[test]
    fn modern_ai_renders_and_legacy_ai_fails_helpfully() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(80, 80, false);
        let pdf = pdf_from_images(dir.path(), "x.pdf", &[src]);
        let ai = dir.path().join("logo.ai");
        std::fs::copy(&pdf, &ai).unwrap();
        let png = convert(&ai, Target::Image(Format::Png), &Options { dpi: 72, ..Default::default() }).unwrap();
        assert_eq!(image::open(&png[0]).unwrap().width(), 595);
        assert_eq!(page_count(&convert(&ai, Target::Pdf, &Options::default()).unwrap()[0]), 1);

        let legacy = dir.path().join("old.ai");
        std::fs::write(&legacy, b"%!PS-Adobe-3.0\n%%Creator: Adobe Illustrator(TM) 8.0\n").unwrap();
        assert!(matches!(convert(&legacy, Target::Image(Format::Png), &Options::default()), Err(Error::AiLegacy)));
    }

    #[test]
    fn raster_to_pdf_keeps_pixels() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(595, 842, false);
        let png = dir.path().join("s.png");
        src.save(&png).unwrap();
        let pdf = convert(&png, Target::Pdf, &Options { quality: 95, ..Default::default() }).unwrap();
        let back = convert(&pdf[0], Target::PdfPages(Format::Png), &Options { dpi: 72, ..Default::default() }).unwrap();
        let d = mean_diff(&src, &image::open(&back[0]).unwrap());
        assert!(d < 4.0, "mean diff {d}");
    }
}
