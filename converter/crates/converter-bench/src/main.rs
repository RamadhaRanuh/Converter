//! Benchmark harness for the v1 speed targets. Drives the core library directly (the same code path the
//! CLI and GUI use) on the fixed corpus from `make-corpus.sh`, and writes JSON shaped like the Node baseline.
//!
//! Usage: converter-bench <corpus-dir> [out.json]
//! Each case runs in its own child process (`--case <name>`) so peak memory is measured per case.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use converter_core::{Format, Options, Target, convert, run_batch};

const RUNS: usize = 5;

struct Case {
    name: &'static str,
    label: &'static str,
    target_s: Option<f64>,
}

const CASES: &[Case] = &[
    Case { name: "psd", label: "single: big.psd -> png", target_s: Some(3.0) },
    Case { name: "tiff", label: "single: big.tiff -> webp q80", target_s: Some(6.0) },
    Case { name: "batch", label: "batch parallel: 300 jpg -> webp q80", target_s: Some(20.0) },
    Case { name: "pdf", label: "single: pages.pdf (50 pages) -> png per page @150dpi", target_s: None },
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--case") {
        run_case(&args[2], Path::new(&args[3]));
        return;
    }
    let corpus = PathBuf::from(args.get(1).expect("usage: converter-bench <corpus-dir> [out.json]"));
    let out_json = args.get(2).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("baseline-rust.json"));
    let exe = std::env::current_exe().unwrap();
    let mut rows = Vec::new();
    let mut all_pass = true;
    for c in CASES {
        let out = Command::new(&exe).args(["--case", c.name]).arg(&corpus).output().expect("spawn case");
        let text = String::from_utf8_lossy(&out.stdout);
        let line = text.lines().last().unwrap_or_default().to_string();
        if !out.status.success() {
            eprintln!("{}: failed\n{}", c.label, String::from_utf8_lossy(&out.stderr));
            all_pass = false;
            continue;
        }
        // Child prints: "<median> <min> <peak_mb> <files>"
        let v: Vec<f64> = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
        let (median, min, peak, files) = (v[0], v[1], v[2], v[3]);
        let pass = c.target_s.map(|t| median <= t);
        all_pass &= pass.unwrap_or(true);
        let verdict = match (pass, c.target_s) {
            (Some(true), Some(t)) => format!("PASS (target ≤ {t} s)"),
            (Some(false), Some(t)) => format!("FAIL (target ≤ {t} s)"),
            _ => "recorded".into(),
        };
        println!("{:<55} median {median:>7.3} s  min {min:>7.3} s  peak {peak:>5.0} MB  {verdict}", c.label);
        let fps = if files > 1.0 { format!(", \"files_per_s\": {:.1}", files / median) } else { String::new() };
        rows.push(format!(
            "    {{ \"label\": \"{}\", \"runs\": {RUNS}, \"median_s\": {median:.3}, \"min_s\": {min:.3}, \"peak_rss_mb\": {peak:.0}{fps} }}",
            c.label
        ));
    }
    let json = format!(
        "{{\n  \"machine\": {{ \"logical_cores\": {} }},\n  \"quality\": 80,\n  \"results\": [\n{}\n  ]\n}}\n",
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        rows.join(",\n")
    );
    std::fs::write(&out_json, json).unwrap();
    println!("\nwrote {}", out_json.display());
    if !all_pass {
        std::process::exit(1);
    }
}

fn run_case(name: &str, corpus: &Path) {
    let scratch = std::env::temp_dir().join(format!("converter-bench-{}", std::process::id()));
    let opts = Options { quality: 80, dpi: 150, out_dir: Some(scratch.clone()) };
    let photos: Vec<PathBuf> = {
        let mut v: Vec<PathBuf> = std::fs::read_dir(corpus.join("photos")).unwrap().map(|e| e.unwrap().path()).collect();
        v.sort();
        v
    };
    let job: Box<dyn Fn() -> usize> = match name {
        "psd" => Box::new(|| convert(&corpus.join("big.psd"), Target::Image(Format::Png), &opts).unwrap().len()),
        "tiff" => Box::new(|| convert(&corpus.join("big.tiff"), Target::Image(Format::WebP), &opts).unwrap().len()),
        "pdf" => Box::new(|| convert(&corpus.join("pages.pdf"), Target::PdfPages(Format::Png), &opts).unwrap().len()),
        "batch" => Box::new(|| {
            let (tx, rx) = std::sync::mpsc::channel();
            let s = run_batch(photos.clone(), Target::Image(Format::WebP), opts.clone(), tx, Arc::default());
            drop(rx);
            assert_eq!(s.failed, 0);
            s.converted
        }),
        other => panic!("unknown case {other}"),
    };
    let mut times = Vec::new();
    let mut files = 0;
    for i in 0..=RUNS {
        let _ = std::fs::remove_dir_all(&scratch);
        let t = Instant::now();
        files = job();
        if i > 0 {
            times.push(t.elapsed().as_secs_f64()); // run 0 is the warm-up
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let files = if name == "batch" { files } else { 1 };
    println!("{:.4} {:.4} {:.0} {files}", times[times.len() / 2], times[0], peak_mb());
}

#[cfg(windows)]
fn peak_mb() -> f64 {
    use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let mut c: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        c.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb);
        c.PeakWorkingSetSize as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(not(windows))]
fn peak_mb() -> f64 {
    0.0
}
