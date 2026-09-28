//! Raster decode and encode: JPG, PNG, WebP, BMP, TIFF, GIF.
use std::io::{BufWriter, Write};
use std::path::Path;

use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, RgbImage};

use crate::{Error, Format, Options};

fn image_format(f: Format) -> Option<ImageFormat> {
    Some(match f {
        Format::Jpeg => ImageFormat::Jpeg,
        Format::Png => ImageFormat::Png,
        Format::WebP => ImageFormat::WebP,
        Format::Bmp => ImageFormat::Bmp,
        Format::Tiff => ImageFormat::Tiff,
        Format::Gif => ImageFormat::Gif,
        _ => return None,
    })
}

/// Decodes a raster Source file, applying its EXIF orientation so the Output file looks upright.
pub(crate) fn decode(path: &Path, format: Format) -> Result<DynamicImage, Error> {
    let mut reader = ImageReader::open(path).map_err(|e| Error::io(path, e))?;
    reader.set_format(image_format(format).ok_or(Error::UnknownFormat)?);
    let mut decoder = reader.into_decoder().map_err(Error::decode)?;
    let orientation = decoder.orientation().ok();
    let mut img = DynamicImage::from_decoder(decoder).map_err(Error::decode)?;
    if let Some(o) = orientation {
        img.apply_orientation(o);
    }
    Ok(img)
}

/// Flattens transparency onto white, for formats without alpha (JPG) or with 1-bit alpha only.
pub(crate) fn flatten_on_white(img: &DynamicImage) -> RgbImage {
    if !img.color().has_alpha() {
        return img.to_rgb8();
    }
    let rgba = img.to_rgba8();
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let p = rgba.get_pixel(x, y).0;
        let a = p[3] as u32;
        let blend = |c: u8| ((c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
        image::Rgb([blend(p[0]), blend(p[1]), blend(p[2])])
    })
}

/// Encodes `img` as `format` into `out`. `quality` (1-100) applies to JPG and WebP; WebP at 100 is lossless.
pub(crate) fn encode(img: &DynamicImage, format: Format, opts: &Options, out: impl Write) -> Result<(), Error> {
    let mut w = BufWriter::new(out);
    let err = |e: image::ImageError| Error::encode(format, e);
    match format {
        Format::Jpeg => {
            let rgb = flatten_on_white(img);
            JpegEncoder::new_with_quality(&mut w, opts.quality.clamp(1, 100)).encode_image(&rgb).map_err(err)?;
        }
        Format::Png => {
            let enc = PngEncoder::new_with_quality(&mut w, CompressionType::Fast, FilterType::Adaptive);
            write_with(img, enc).map_err(err)?;
        }
        Format::WebP => {
            // Borrow 8-bit pixels where possible: a 12 MP photo copy is 36 MB per Conversion in flight.
            let (w8, h8) = (img.width(), img.height());
            let converted: DynamicImage;
            let src = match img {
                DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgba8(_) => img,
                _ if img.color().has_alpha() => {
                    converted = DynamicImage::ImageRgba8(img.to_rgba8());
                    &converted
                }
                _ => {
                    converted = DynamicImage::ImageRgb8(img.to_rgb8());
                    &converted
                }
            };
            let enc = match src {
                DynamicImage::ImageRgba8(p) => webp::Encoder::from_rgba(p.as_raw(), w8, h8),
                DynamicImage::ImageRgb8(p) => webp::Encoder::from_rgb(p.as_raw(), w8, h8),
                _ => unreachable!("converted above"),
            };
            let mem = webp_encode(enc, opts)?;
            w.write_all(&mem).map_err(|e| Error::encode(format, e))?;
        }
        Format::Gif => {
            let rgba = img.to_rgba8();
            GifEncoder::new_with_speed(&mut w, 10).encode_frame(image::Frame::new(rgba)).map_err(err)?;
        }
        Format::Bmp | Format::Tiff => {
            // Both writers need `Seek`; encode to memory first.
            let mut buf = std::io::Cursor::new(Vec::new());
            let img = if format == Format::Bmp && img.color().has_alpha() {
                DynamicImage::ImageRgba8(img.to_rgba8())
            } else {
                normalize_depth(img)
            };
            img.write_to(&mut buf, image_format(format).unwrap()).map_err(err)?;
            w.write_all(buf.get_ref()).map_err(|e| Error::encode(format, e))?;
        }
        other => return Err(Error::encode(other, "not a raster output format")),
    }
    w.flush().map_err(|e| Error::encode(format, e))
}

fn webp_encode(enc: webp::Encoder<'_>, opts: &Options) -> Result<Vec<u8>, Error> {
    let mem = if opts.quality >= 100 { enc.encode_lossless() } else { enc.encode(opts.quality.clamp(1, 100) as f32) };
    Ok(mem.to_vec())
}

fn write_with(img: &DynamicImage, enc: impl image::ImageEncoder) -> image::ImageResult<()> {
    let img = normalize_depth(img);
    enc.write_image(img.as_bytes(), img.width(), img.height(), img.color().into())
}

/// Float images can't be written by most encoders; convert them to 16-bit.
fn normalize_depth(img: &DynamicImage) -> DynamicImage {
    match img {
        DynamicImage::ImageRgb32F(_) => DynamicImage::ImageRgb16(img.to_rgb16()),
        DynamicImage::ImageRgba32F(_) => DynamicImage::ImageRgba16(img.to_rgba16()),
        other => other.clone(),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A photo-like test image: smooth gradients plus some structure.
    pub(crate) fn sample(w: u32, h: u32, alpha: bool) -> DynamicImage {
        let img = image::RgbaImage::from_fn(w, h, |x, y| {
            let a = if alpha { ((x * 255) / w.max(1)) as u8 } else { 255 };
            image::Rgba([(x * 255 / w) as u8, (y * 255 / h) as u8, (((x + y) / 4) % 256) as u8, a])
        });
        if alpha { DynamicImage::ImageRgba8(img) } else { DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(img).to_rgb8()) }
    }

    /// Mean absolute difference per RGB channel, 0-255.
    pub(crate) fn mean_diff(a: &DynamicImage, b: &DynamicImage) -> f64 {
        assert_eq!((a.width(), a.height()), (b.width(), b.height()));
        let (a, b) = (flatten_on_white(a), flatten_on_white(b));
        let sum: u64 = a.as_raw().iter().zip(b.as_raw()).map(|(x, y)| x.abs_diff(*y) as u64).sum();
        sum as f64 / a.as_raw().len() as f64
    }

    #[test]
    fn round_trips_every_raster_format() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(96, 64, false);
        let opts = Options { quality: 90, ..Default::default() };
        for f in [Format::Jpeg, Format::Png, Format::WebP, Format::Bmp, Format::Tiff, Format::Gif] {
            let p = dir.path().join(format!("t.{}", f.extension()));
            encode(&src, f, &opts, std::fs::File::create(&p).unwrap()).unwrap();
            let back = decode(&p, f).unwrap();
            let d = mean_diff(&src, &back);
            let bar = match f {
                Format::Gif => 12.0,
                Format::Jpeg | Format::WebP => 4.0,
                _ => 0.01,
            };
            assert!(d <= bar, "{f}: mean diff {d:.2} > {bar}");
        }
    }

    #[test]
    fn transparency_survives_png_and_webp_and_flattens_for_jpg() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(32, 32, true);
        for f in [Format::Png, Format::WebP] {
            let p = dir.path().join(format!("a.{}", f.extension()));
            encode(&src, f, &Options::default(), std::fs::File::create(&p).unwrap()).unwrap();
            assert!(decode(&p, f).unwrap().color().has_alpha(), "{f} lost alpha");
        }
        let p = dir.path().join("a.jpg");
        encode(&src, Format::Jpeg, &Options::default(), std::fs::File::create(&p).unwrap()).unwrap();
        let back = decode(&p, Format::Jpeg).unwrap().to_rgb8();
        assert!(back.get_pixel(0, 16).0.iter().all(|&c| c > 240), "transparent pixel should become white");
    }

    #[test]
    fn webp_quality_100_is_lossless() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(40, 30, false);
        let p = dir.path().join("l.webp");
        encode(&src, Format::WebP, &Options { quality: 100, ..Default::default() }, std::fs::File::create(&p).unwrap()).unwrap();
        assert_eq!(mean_diff(&src, &decode(&p, Format::WebP).unwrap()), 0.0);
    }
}
