//! HEIC input (iPhone photos) via libheif + libde265, behind the `heic` cargo feature.
//! There is no HEIC output: the only HEVC encoder (x265) is GPL.
use std::path::Path;

use image::DynamicImage;

use crate::Error;

#[cfg(feature = "heic")]
pub(crate) fn decode(path: &Path) -> Result<DynamicImage, Error> {
    use image::{RgbImage, RgbaImage};
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

    let data = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    let ctx = HeifContext::read_from_bytes(&data).map_err(|e| Error::decode(format!("HEIC: {e}")))?;
    let handle = ctx.primary_image_handle().map_err(|e| Error::decode(format!("HEIC: {e}")))?;
    let alpha = handle.has_alpha_channel();
    let chroma = if alpha { RgbChroma::Rgba } else { RgbChroma::Rgb };
    // libheif applies the file's rotation and mirroring (irot/imir) while decoding.
    let img = LibHeif::new().decode(&handle, ColorSpace::Rgb(chroma), None).map_err(|e| Error::decode(format!("HEIC: {e}")))?;
    let plane = img.planes().interleaved.ok_or_else(|| Error::decode("HEIC: no interleaved plane"))?;
    let (w, h, stride) = (plane.width, plane.height, plane.stride);
    let bpp = if alpha { 4 } else { 3 };
    let row = w as usize * bpp;
    let mut px = Vec::with_capacity(row * h as usize);
    for y in 0..h as usize {
        px.extend_from_slice(&plane.data[y * stride..y * stride + row]);
    }
    Ok(if alpha {
        DynamicImage::ImageRgba8(RgbaImage::from_raw(w, h, px).expect("sized"))
    } else {
        DynamicImage::ImageRgb8(RgbImage::from_raw(w, h, px).expect("sized"))
    })
}

#[cfg(not(feature = "heic"))]
pub(crate) fn decode(_path: &Path) -> Result<DynamicImage, Error> {
    Err(Error::Unsupported("HEIC support isn't included in this build.".into()))
}

#[cfg(all(test, feature = "heic"))]
mod tests {
    use crate::{Format, Options, Target, convert};

    /// Uses `CONVERTER_HEIC_FIXTURE`, or the benchmark corpus's `example.heic` (libheif's sample), when present.
    /// HEIC can't be generated without a GPL encoder, so the fixture isn't committed.
    #[test]
    fn heic_decodes_to_jpg() {
        let fixture = std::env::var_os("CONVERTER_HEIC_FIXTURE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.wayfinder/bench/corpus/example.heic"));
        if !fixture.exists() {
            eprintln!("skipping: no HEIC fixture at {}", fixture.display());
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("photo.heic");
        std::fs::copy(&fixture, &src).unwrap();
        assert_eq!(crate::detect(&src).unwrap(), Format::Heic);
        let out = convert(&src, Target::Image(Format::Jpeg), &Options::default()).unwrap();
        let img = image::open(&out[0]).unwrap();
        assert!(img.width() > 0 && img.height() > 0);
    }
}
