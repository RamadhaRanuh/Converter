use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::{Error, Options};

/// Reserves a fresh Output file path for `src` with extension `ext`, next to the Source file
/// (or in `opts.out_dir`). `suffix` distinguishes multi-output Conversions, e.g. `-p3`.
/// Never overwrites: on a clash it tries `<stem> (1).<ext>`, `<stem> (2).<ext>`, ...
/// The file is created empty to claim the name, so parallel Conversions can't collide.
pub(crate) fn reserve(src: &Path, ext: &str, suffix: &str, opts: &Options) -> Result<(PathBuf, File), Error> {
    let dir = match &opts.out_dir {
        Some(d) => {
            std::fs::create_dir_all(d).map_err(|e| Error::io(d, e))?;
            d.clone()
        }
        None => src.parent().map(Path::to_path_buf).unwrap_or_default(),
    };
    let stem = src.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "output".into());
    for n in 0.. {
        let name = match n {
            0 => format!("{stem}{suffix}.{ext}"),
            n => format!("{stem}{suffix} ({n}).{ext}"),
        };
        let path = dir.join(name);
        if path == src {
            continue; // same-format re-encode: never replace the Source file
        }
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(f) => return Ok((path, f)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(Error::io(&path, e)),
        }
    }
    unreachable!()
}

/// Deletes reserved Output files after a failed Conversion so no empty or partial files are left behind.
pub(crate) fn discard(paths: &[PathBuf]) {
    for p in paths {
        let _ = std::fs::remove_file(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_overwrites_and_numbers_clashes() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("photo.jpg");
        std::fs::write(&src, b"x").unwrap();
        let opts = Options::default();
        let (a, _) = reserve(&src, "png", "", &opts).unwrap();
        let (b, _) = reserve(&src, "png", "", &opts).unwrap();
        let (c, _) = reserve(&src, "jpg", "", &opts).unwrap();
        assert_eq!(a.file_name().unwrap(), "photo.png");
        assert_eq!(b.file_name().unwrap(), "photo (1).png");
        assert_eq!(c.file_name().unwrap(), "photo (1).jpg", "must not replace the Source file");
        assert_eq!(std::fs::read(&src).unwrap(), b"x");
    }

    #[test]
    fn out_dir_is_created_and_used() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.pdf");
        let opts = Options { out_dir: Some(dir.path().join("out/nested")), ..Default::default() };
        let (p, _) = reserve(&src, "png", "-p1", &opts).unwrap();
        assert_eq!(p, dir.path().join("out/nested/a-p1.png"));
    }
}
