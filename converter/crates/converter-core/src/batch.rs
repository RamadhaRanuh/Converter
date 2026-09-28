//! Batches: many independent Conversions from one drop, run in parallel, one per core.
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use rayon::prelude::*;

use crate::{Error, Format, Options, Target, convert, detect, targets_for};

/// Progress of a Batch, sent from worker threads. `usize` is the file's index in the Batch.
#[derive(Debug)]
pub enum Event {
    Started(usize),
    Done(usize, Vec<PathBuf>),
    Failed(usize, Error),
    /// The file can't become the chosen Target format; it was left alone.
    Skipped(usize, String),
    Finished(Summary),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub converted: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cancelled: bool,
}

/// One dropped file, with its detected format (or why it couldn't be recognised).
#[derive(Debug)]
pub struct PlannedFile {
    pub path: PathBuf,
    pub format: Result<Format, Error>,
}

/// What the UI needs after a drop: the files and, for each offered Target format, how many files can become it.
#[derive(Debug, Default)]
pub struct Plan {
    pub files: Vec<PlannedFile>,
    /// Target formats at least one file can become, in display order, with the count of files that can.
    pub targets: Vec<(Target, usize)>,
}

/// Expands dropped paths: files stay as they are, folders are walked recursively (sorted, hidden entries skipped).
pub fn expand_sources(paths: &[PathBuf]) -> Vec<PathBuf> {
    fn walk(p: &Path, out: &mut Vec<PathBuf>) {
        if p.is_dir() {
            let Ok(rd) = std::fs::read_dir(p) else { return };
            let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
            entries.sort();
            for e in entries {
                if !e.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
                    walk(&e, out);
                }
            }
        } else if p.is_file() {
            out.push(p.to_path_buf());
        }
    }
    let mut out = Vec::new();
    for p in paths {
        walk(p, &mut out);
    }
    out
}

/// Detects every Source file and works out which Target formats to offer.
pub fn plan(paths: &[PathBuf]) -> Plan {
    let files: Vec<PlannedFile> = expand_sources(paths)
        .into_par_iter()
        .map(|path| PlannedFile { format: detect(&path), path })
        .collect();
    let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
    for f in &files {
        if let Ok(fmt) = f.format {
            for t in targets_for(fmt) {
                let order = Target::ALL.iter().position(|x| x == t).unwrap();
                *counts.entry(order).or_default() += 1;
            }
        }
    }
    let targets = counts.into_iter().map(|(i, n)| (Target::ALL[i], n)).collect();
    Plan { files, targets }
}

/// Converts every file to `target` in parallel, reporting through `events`. Files that can't become
/// `target` are skipped, failures never stop the Batch, and a panicking decoder only fails its own file.
/// Returns when every file is finished; the last event is always `Finished`.
pub fn run_batch(sources: Vec<PathBuf>, target: Target, opts: Options, events: Sender<Event>, cancel: Arc<AtomicBool>) -> Summary {
    let results: Vec<Outcome> = sources
        .par_iter()
        .enumerate()
        .map(|(i, src)| {
            if cancel.load(Ordering::Relaxed) {
                return Outcome::Cancelled;
            }
            match detect(src) {
                Ok(f) if !targets_for(f).contains(&target) => {
                    let _ = events.send(Event::Skipped(i, format!("A {f} file can't become {target}.")));
                    return Outcome::Skipped;
                }
                Err(e) => {
                    let _ = events.send(Event::Failed(i, e));
                    return Outcome::Failed;
                }
                Ok(_) => {}
            }
            let _ = events.send(Event::Started(i));
            match catch_unwind(AssertUnwindSafe(|| convert(src, target, &opts))) {
                Ok(Ok(outs)) => {
                    let _ = events.send(Event::Done(i, outs));
                    Outcome::Done
                }
                Ok(Err(e)) => {
                    let _ = events.send(Event::Failed(i, e));
                    Outcome::Failed
                }
                Err(panic) => {
                    let msg = panic.downcast_ref::<&str>().map(|s| s.to_string())
                        .or_else(|| panic.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "unknown".into());
                    let _ = events.send(Event::Failed(i, Error::Panicked(msg)));
                    Outcome::Failed
                }
            }
        })
        .collect();
    let mut s = Summary::default();
    for r in results {
        match r {
            Outcome::Done => s.converted += 1,
            Outcome::Failed => s.failed += 1,
            Outcome::Skipped => s.skipped += 1,
            Outcome::Cancelled => s.cancelled = true,
        }
    }
    let _ = events.send(Event::Finished(s));
    s
}

enum Outcome {
    Done,
    Failed,
    Skipped,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::raster::tests::sample;

    fn write_png(p: &Path) {
        sample(20, 10, false).save(p).unwrap();
    }

    #[test]
    fn batch_converts_skips_and_fails_without_stopping() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        write_png(&d.join("a.png"));
        write_png(&d.join("b.png"));
        std::fs::write(d.join("notes.md"), "# hi").unwrap(); // can't become WebP: skipped
        std::fs::write(d.join("broken.jpg"), [0xFF, 0xD8, 0xFF, 0x00, 0x01]).unwrap(); // corrupt: fails
        std::fs::write(d.join("mystery.xyz"), b"???").unwrap(); // unknown: fails
        let (tx, rx) = std::sync::mpsc::channel();
        let files = expand_sources(&[d.to_path_buf()]);
        let s = run_batch(files, Target::Image(Format::WebP), Options::default(), tx, Arc::default());
        assert_eq!(s, Summary { converted: 2, failed: 2, skipped: 1, cancelled: false });
        let events: Vec<Event> = rx.into_iter().collect();
        assert!(matches!(events.last(), Some(Event::Finished(_))));
        assert!(d.join("a.webp").exists() && d.join("b.webp").exists());
        assert!(!d.join("broken.webp").exists(), "failed Conversion must not leave a file behind");
    }

    #[test]
    fn plan_counts_how_many_files_fit_each_target() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        write_png(&d.join("a.png"));
        std::fs::create_dir(d.join("sub")).unwrap();
        write_png(&d.join("sub/b.png"));
        std::fs::write(d.join("sub/notes.md"), "# hi").unwrap();
        let p = plan(&[d.to_path_buf()]);
        assert_eq!(p.files.len(), 3);
        let count = |t| p.targets.iter().find(|(x, _)| *x == t).map(|(_, n)| *n);
        assert_eq!(count(Target::Image(Format::Jpeg)), Some(2));
        assert_eq!(count(Target::Pdf), Some(3));
        assert_eq!(count(Target::Html), Some(1));
        assert_eq!(count(Target::PdfSplit), None);
    }

    #[test]
    fn cancelled_batch_reports_it() {
        let dir = tempfile::tempdir().unwrap();
        write_png(&dir.path().join("a.png"));
        let (tx, _rx) = std::sync::mpsc::channel();
        let s = run_batch(vec![dir.path().join("a.png")], Target::Image(Format::Jpeg), Options::default(), tx, Arc::new(AtomicBool::new(true)));
        assert!(s.cancelled && s.converted == 0);
    }
}
