//! Photoshop: read the stored composite image, write a flattened single-layer PSD.
use std::io::Write;
use std::path::Path;

use image::{DynamicImage, GrayImage, ImageBuffer, RgbImage, RgbaImage};
use zune_core::bytestream::ZCursor;
use zune_core::colorspace::ColorSpace;
use zune_core::result::DecodingResult;
use zune_psd::PSDDecoder;

use crate::Error;

/// Decodes the composite ("Maximize compatibility") image stored in a PSD.
pub(crate) fn decode(path: &Path) -> Result<DynamicImage, Error> {
    let data = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    let file_len = data.len();
    let mut dec = PSDDecoder::new(ZCursor::new(data));
    dec.decode_headers().map_err(|e| Error::decode(format!("{e:?}")))?;
    let (w, h) = dec.dimensions().ok_or_else(|| Error::decode("missing dimensions"))?;
    let cs = dec.colorspace().ok_or_else(|| Error::decode("missing colour mode"))?;
    let pixels = dec.decode().map_err(|e| Error::decode(format!("{e:?}")))?;
    let (w, h) = (w as u32, h as u32);
    let bad = || Error::decode("pixel data doesn't match the header");

    let img = match pixels {
        DecodingResult::U8(px) => match cs {
            ColorSpace::RGB => DynamicImage::ImageRgb8(RgbImage::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::RGBA => DynamicImage::ImageRgba8(RgbaImage::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::Luma => DynamicImage::ImageLuma8(GrayImage::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::LumaA => DynamicImage::ImageLumaA8(ImageBuffer::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::CMYK => DynamicImage::ImageRgb8(cmyk_to_rgb(w, h, &px).ok_or_else(bad)?),
            other => return Err(Error::decode(format!("unsupported PSD colour mode {other:?}"))),
        },
        DecodingResult::U16(px) => match cs {
            ColorSpace::RGB => DynamicImage::ImageRgb16(ImageBuffer::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::RGBA => DynamicImage::ImageRgba16(ImageBuffer::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::Luma => DynamicImage::ImageLuma16(ImageBuffer::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::LumaA => DynamicImage::ImageLumaA16(ImageBuffer::from_raw(w, h, px).ok_or_else(bad)?),
            ColorSpace::CMYK => {
                let px8: Vec<u8> = px.iter().map(|v| (v >> 8) as u8).collect();
                DynamicImage::ImageRgb8(cmyk_to_rgb(w, h, &px8).ok_or_else(bad)?)
            }
            other => return Err(Error::decode(format!("unsupported PSD colour mode {other:?}"))),
        },
        _ => return Err(Error::decode("unsupported PSD bit depth")),
    };

    // Photoshop leaves the composite blank when "Maximize compatibility" is off. A perfectly uniform
    // composite in a file far larger than a blank image would need is that case, not a real flat image.
    if file_len > 64 * 1024 && is_uniform(&img) {
        return Err(Error::PsdNoComposite);
    }
    Ok(img)
}

/// Naive CMYK → RGB (no ICC). Photoshop stores CMYK inverted: 255 means no ink.
fn cmyk_to_rgb(w: u32, h: u32, px: &[u8]) -> Option<RgbImage> {
    if px.len() != (w * h * 4) as usize {
        return None;
    }
    let rgb = px.chunks_exact(4).flat_map(|p| {
        let k = p[3] as u16;
        [0, 1, 2].map(|i| ((p[i] as u16 * k) / 255) as u8)
    });
    RgbImage::from_raw(w, h, rgb.collect())
}

fn is_uniform(img: &DynamicImage) -> bool {
    let b = img.as_bytes();
    let px = img.color().bytes_per_pixel() as usize;
    b.len() >= px && b.chunks_exact(px).all(|c| c == &b[..px])
}

/// Writes `img` as a flattened 8-bit RGB(A) PSD with RLE (PackBits) image data.
/// Photoshop opens it as a single "Background" image; alpha becomes an extra channel.
pub(crate) fn encode(img: &DynamicImage, out: &mut dyn Write) -> Result<(), Error> {
    let alpha = img.color().has_alpha();
    let (w, h) = (img.width(), img.height());
    if w > 30_000 || h > 30_000 {
        return Err(Error::encode("PSD", "PSD files are limited to 30,000 px per side"));
    }
    let channels: u16 = if alpha { 4 } else { 3 };
    let raw = if alpha {
        // Photoshop's composite convention: colour is stored matted against white.
        let mut px = img.to_rgba8().into_raw();
        for p in px.chunks_exact_mut(4) {
            let a = p[3] as u32;
            for c in &mut p[..3] {
                *c = ((*c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
            }
        }
        px
    } else {
        img.to_rgb8().into_raw()
    };
    let c = channels as usize;

    // Planar rows, each PackBits-compressed: all rows of R, then G, then B (then A).
    let mut counts: Vec<u16> = Vec::with_capacity(h as usize * c);
    let mut body = Vec::with_capacity(raw.len() / 2);
    let mut row = vec![0u8; w as usize];
    for ch in 0..c {
        for y in 0..h as usize {
            let start = y * w as usize * c;
            for x in 0..w as usize {
                row[x] = raw[start + x * c + ch];
            }
            let before = body.len();
            packbits(&row, &mut body);
            counts.push((body.len() - before) as u16);
        }
    }

    let mut buf = Vec::with_capacity(26 + 12 + 2 + counts.len() * 2 + body.len());
    buf.extend_from_slice(b"8BPS");
    buf.extend_from_slice(&1u16.to_be_bytes()); // version
    buf.extend_from_slice(&[0; 6]); // reserved
    buf.extend_from_slice(&channels.to_be_bytes());
    buf.extend_from_slice(&h.to_be_bytes());
    buf.extend_from_slice(&w.to_be_bytes());
    buf.extend_from_slice(&8u16.to_be_bytes()); // depth
    buf.extend_from_slice(&3u16.to_be_bytes()); // colour mode: RGB
    buf.extend_from_slice(&0u32.to_be_bytes()); // colour mode data
    buf.extend_from_slice(&0u32.to_be_bytes()); // image resources
    buf.extend_from_slice(&0u32.to_be_bytes()); // layer and mask info
    buf.extend_from_slice(&1u16.to_be_bytes()); // compression: RLE
    for n in &counts {
        buf.extend_from_slice(&n.to_be_bytes());
    }
    buf.extend_from_slice(&body);
    out.write_all(&buf).map_err(|e| Error::encode("PSD", e))
}

/// Apple PackBits, as used by PSD RLE. Rows are compressed independently.
fn packbits(src: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < src.len() {
        // Length of the run starting at i.
        let mut run = 1;
        while i + run < src.len() && run < 128 && src[i + run] == src[i] {
            run += 1;
        }
        if run >= 3 {
            out.push((257 - run) as u8);
            out.push(src[i]);
            i += run;
            continue;
        }
        // Literal: extend until a run of 3 begins or 128 bytes.
        let start = i;
        while i < src.len() && i - start < 128 {
            if i + 2 < src.len() && src[i] == src[i + 1] && src[i] == src[i + 2] {
                break;
            }
            i += 1;
        }
        out.push((i - start - 1) as u8);
        out.extend_from_slice(&src[start..i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::raster::tests::{mean_diff, sample};

    #[test]
    fn packbits_round_trips() {
        fn unpack(mut s: &[u8]) -> Vec<u8> {
            let mut o = vec![];
            while let Some((&n, rest)) = s.split_first() {
                if n < 128 {
                    o.extend_from_slice(&rest[..n as usize + 1]);
                    s = &rest[n as usize + 1..];
                } else if n > 128 {
                    o.extend(std::iter::repeat_n(rest[0], 257 - n as usize));
                    s = &rest[1..];
                } else {
                    s = rest;
                }
            }
            o
        }
        for input in [vec![], vec![7], vec![1, 1, 1, 1, 2, 3, 3, 4, 4, 4], (0..=255).collect(), vec![9; 1000]] {
            let mut enc = vec![];
            packbits(&input, &mut enc);
            assert_eq!(unpack(&enc), input);
        }
    }

    #[test]
    fn writes_psd_that_reads_back_identically() {
        let dir = tempfile::tempdir().unwrap();
        for alpha in [false, true] {
            let src = sample(123, 45, alpha);
            let p = dir.path().join(format!("t{alpha}.psd"));
            encode(&src, &mut std::fs::File::create(&p).unwrap()).unwrap();
            let back = decode(&p).unwrap();
            assert_eq!(back.color().has_alpha(), alpha);
            // Alpha goes through Photoshop's white matte and back, so allow rounding.
            let d = mean_diff(&src, &back);
            assert!(if alpha { d < 1.0 } else { d == 0.0 }, "alpha={alpha}: mean diff {d}");
        }
    }

    #[test]
    fn uniform_composite_in_large_file_means_no_composite() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("blank.psd");
        let mut f = std::fs::File::create(&p).unwrap();
        encode(&DynamicImage::ImageRgb8(RgbImage::from_pixel(64, 64, image::Rgb([255, 255, 255]))), &mut f).unwrap();
        f.write_all(&vec![0u8; 100 * 1024]).unwrap(); // stands in for layer data after the composite
        drop(f);
        assert!(matches!(decode(&p), Err(Error::PsdNoComposite)));
    }
}
