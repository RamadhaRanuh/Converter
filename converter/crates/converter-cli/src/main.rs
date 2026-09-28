//! `converter`: the command-line face of the converter core.
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::mpsc::channel;

use clap::Parser;
use converter_core::{Event, Options, Target, combine, expand_sources, plan, run_batch};

/// Convert images, designs and documents, fast and entirely offline.
///
/// Folders are converted recursively. Output files are written next to each source file
/// (or into --out) and existing files are never overwritten.
#[derive(Parser)]
#[command(version, after_help = TARGETS_HELP)]
struct Cli {
    /// Files or folders to convert.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Target format (see list below). Not needed with --combine or --targets.
    #[arg(long, short = 't', value_parser = parse_target)]
    to: Option<Target>,

    /// Quality for JPG and WebP, 1-100 (100 = lossless WebP).
    #[arg(long, short = 'q', default_value_t = 85, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: u8,

    /// Resolution for rendering PDF pages, Illustrator files and SVGs.
    #[arg(long, default_value_t = 150, value_parser = clap::value_parser!(u16).range(10..=1200))]
    dpi: u16,

    /// Write output files into this folder instead of next to each source file.
    #[arg(long, short = 'o')]
    out: Option<PathBuf>,

    /// Combine all images and PDFs into one PDF, in the order given.
    #[arg(long, conflicts_with = "to")]
    combine: bool,

    /// Only list the target formats the inputs can become, with how many files fit each.
    #[arg(long, conflicts_with_all = ["to", "combine"])]
    targets: bool,
}

const TARGETS_HELP: &str = "Target formats:\n  jpg png webp bmp tiff gif   images\n  psd                         flattened Photoshop file\n  svg                         traced (from images) or optimized (from SVG)\n  pdf                         PDF\n  png-pages jpg-pages         one image per PDF page\n  pdf-split                   one PDF per page\n  html                        from Markdown";

fn parse_target(s: &str) -> Result<Target, String> {
    Target::parse(s).ok_or_else(|| format!("unknown target format '{s}'"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let opts = Options { quality: cli.quality, dpi: cli.dpi, out_dir: cli.out.clone() };

    if cli.targets {
        let p = plan(&cli.inputs);
        let total = p.files.len();
        for (t, n) in &p.targets {
            let fit = if *n == total { "all files".to_string() } else { format!("{n} of {total} files") };
            println!("{:<22} {fit}", t.label());
        }
        for f in &p.files {
            if let Err(e) = &f.format {
                eprintln!("  ! {}: {e}", f.path.display());
            }
        }
        return ExitCode::SUCCESS;
    }

    let files = expand_sources(&cli.inputs);
    if files.is_empty() {
        eprintln!("No files found.");
        return ExitCode::from(2);
    }

    if cli.combine {
        return match combine(&files, &opts) {
            Ok(out) => {
                println!("✓ {} files → {}", files.len(), out.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("✗ {e}");
                ExitCode::FAILURE
            }
        };
    }

    let Some(target) = cli.to else {
        eprintln!("Choose a target format with --to (or use --combine / --targets). Run with --help for the list.");
        return ExitCode::from(2);
    };

    let (tx, rx) = channel();
    let names = files.clone();
    let worker = std::thread::spawn(move || run_batch(files, target, opts, tx, Arc::default()));
    for ev in rx {
        match ev {
            Event::Done(i, outs) => {
                let outs: Vec<String> = outs.iter().map(|o| o.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect();
                let shown = if outs.len() > 3 { format!("{} … ({} files)", outs[0], outs.len()) } else { outs.join(", ") };
                println!("✓ {} → {shown}", names[i].display());
            }
            Event::Failed(i, e) => eprintln!("✗ {}: {e}", names[i].display()),
            Event::Skipped(i, why) => println!("- {}: skipped. {why}", names[i].display()),
            Event::Started(_) => {}
            Event::Finished(s) => println!("\n{} converted · {} failed · {} skipped", s.converted, s.failed, s.skipped),
        }
    }
    let summary = worker.join().expect("batch thread");
    if summary.failed > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
