//! Converter's window: drop files, pick a Target format, convert. All conversion work runs on worker
//! threads; the UI thread only draws and drains progress events.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};

use converter_core::{Error, Event, Format, Options, Plan, Summary, Target, can_combine, combine, plan, run_batch};
use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Sense, Stroke, Vec2};

fn main() -> eframe::Result {
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Converter")
            .with_inner_size([680.0, 540.0])
            .with_min_inner_size([420.0, 360.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    // Files passed on the command line (Explorer's "Open with", or dropping onto the exe) start loaded.
    let initial: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    eframe::run_native(
        "Converter",
        native,
        Box::new(move |_cc| {
            let mut app = App::default();
            app.add_paths(initial);
            Ok(Box::new(app))
        }),
    )
}

/// What one row of the file list shows.
enum Status {
    Waiting,
    Running,
    Done(Vec<PathBuf>),
    Failed(String),
    Skipped(String),
}

struct Row {
    path: PathBuf,
    format: Result<Format, String>,
    status: Status,
}

enum Phase {
    Empty,
    Ready,
    Running { rx: Receiver<Msg>, cancel: Arc<AtomicBool> },
    Done(Summary),
}

/// Messages from worker threads.
enum Msg {
    Batch(Event),
    Combined(Result<PathBuf, Error>),
}

struct App {
    phase: Phase,
    rows: Vec<Row>,
    targets: Vec<(Target, usize)>,
    target: Option<Target>,
    opts: Options,
    last_output: Option<PathBuf>,
    notice: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        App {
            phase: Phase::Empty,
            rows: Vec::new(),
            targets: Vec::new(),
            target: None,
            opts: Options::default(),
            last_output: None,
            notice: None,
        }
    }
}

impl App {
    fn add_paths(&mut self, paths: Vec<PathBuf>) {
        if paths.is_empty() || matches!(self.phase, Phase::Running { .. }) {
            return;
        }
        let mut all: Vec<PathBuf> =
            if matches!(self.phase, Phase::Done(_)) { Vec::new() } else { self.rows.iter().map(|r| r.path.clone()).collect() };
        all.extend(paths);
        let Plan { files, targets } = plan(&all);
        let mut seen = std::collections::HashSet::new();
        self.rows = files
            .into_iter()
            .filter(|f| seen.insert(f.path.clone()))
            .map(|f| Row { path: f.path, format: f.format.map_err(|e| e.to_string()), status: Status::Waiting })
            .collect();
        self.targets = targets;
        if self.target.is_some_and(|t| !self.targets.iter().any(|(x, _)| *x == t)) {
            self.target = None;
        }
        self.notice = None;
        self.phase = if self.rows.is_empty() { Phase::Empty } else { Phase::Ready };
    }

    fn clear(&mut self) {
        *self = App { opts: self.opts.clone(), ..Default::default() };
    }

    fn start_batch(&mut self, ctx: &egui::Context, target: Target) {
        for r in &mut self.rows {
            r.status = Status::Waiting;
        }
        let files: Vec<PathBuf> = self.rows.iter().map(|r| r.path.clone()).collect();
        let (tx, rx) = channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let (opts, c2, ctx2) = (self.opts.clone(), cancel.clone(), ctx.clone());
        std::thread::spawn(move || {
            let (etx, erx) = channel();
            let worker = std::thread::spawn(move || run_batch(files, target, opts, etx, c2));
            for ev in erx {
                let _ = tx.send(Msg::Batch(ev));
                ctx2.request_repaint();
            }
            let _ = worker.join();
        });
        self.last_output = None;
        self.phase = Phase::Running { rx, cancel };
    }

    fn start_combine(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> =
            self.rows.iter().filter(|r| r.format.as_ref().is_ok_and(|f| can_combine(*f))).map(|r| r.path.clone()).collect();
        let (tx, rx) = channel();
        let (opts, ctx2) = (self.opts.clone(), ctx.clone());
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Combined(combine(&files, &opts)));
            ctx2.request_repaint();
        });
        for r in &mut self.rows {
            r.status = if r.format.as_ref().is_ok_and(|f| can_combine(*f)) {
                Status::Running
            } else {
                Status::Skipped("Only images and PDFs can be combined.".into())
            };
        }
        self.phase = Phase::Running { rx, cancel: Arc::default() };
    }

    fn poll(&mut self) {
        let Phase::Running { rx, .. } = &self.phase else { return };
        let mut finished = None;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Batch(Event::Started(i)) => self.rows[i].status = Status::Running,
                Msg::Batch(Event::Done(i, outs)) => {
                    self.last_output = outs.first().cloned().or(self.last_output.take());
                    self.rows[i].status = Status::Done(outs);
                }
                Msg::Batch(Event::Failed(i, e)) => self.rows[i].status = Status::Failed(e.to_string()),
                Msg::Batch(Event::Skipped(i, why)) => self.rows[i].status = Status::Skipped(why),
                Msg::Batch(Event::Finished(s)) => finished = Some(s),
                Msg::Combined(res) => {
                    let mut s = Summary::default();
                    for r in &mut self.rows {
                        if matches!(r.status, Status::Running) {
                            r.status = match &res {
                                Ok(p) => Status::Done(vec![p.clone()]),
                                Err(e) => Status::Failed(e.to_string()),
                            };
                        }
                        match r.status {
                            Status::Done(_) => s.converted += 1,
                            Status::Failed(_) => s.failed += 1,
                            _ => s.skipped += 1,
                        }
                    }
                    if let Ok(p) = &res {
                        self.last_output = Some(p.clone());
                        self.notice = Some(format!("Combined into {}", p.file_name().unwrap_or_default().to_string_lossy()));
                    }
                    finished = Some(s);
                }
            }
        }
        if let Some(s) = finished {
            for r in &mut self.rows {
                if matches!(r.status, Status::Waiting | Status::Running) {
                    r.status = Status::Skipped("Cancelled.".into());
                }
            }
            self.phase = Phase::Done(s);
        }
    }

    fn combinable(&self) -> usize {
        self.rows.iter().filter(|r| r.format.as_ref().is_ok_and(|f| can_combine(*f))).count()
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll();
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        self.add_paths(dropped);
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        if !matches!(self.phase, Phase::Empty) {
            egui::Panel::bottom("actions").exact_size(52.0).show(ui, |ui| action_bar(ui, &ctx, self));
        }
        egui::CentralPanel::default().show(ui, |ui| {
            if matches!(self.phase, Phase::Empty) {
                empty_state(ui, hovering, self);
            } else {
                file_view(ui, self, hovering);
            }
        });
    }
}

fn pick_files() -> Vec<PathBuf> {
    rfd::FileDialog::new().set_title("Choose files to convert").pick_files().unwrap_or_default()
}

fn empty_state(ui: &mut egui::Ui, hovering: bool, app: &mut App) {
    let avail = ui.available_size();
    let (rect, resp) = ui.allocate_exact_size(avail, Sense::click());
    let accent = ui.visuals().selection.bg_fill;
    let stroke = if hovering || resp.hovered() { Stroke::new(2.5, accent) } else { Stroke::new(2.0, ui.visuals().weak_text_color()) };
    let rect = rect.shrink(12.0);
    ui.painter().rect_stroke(rect, CornerRadius::same(14), stroke, egui::StrokeKind::Inside);
    let c = rect.center();
    let strong = ui.visuals().strong_text_color();
    let weak = ui.visuals().weak_text_color();
    ui.painter().text(
        c - Vec2::new(0.0, 18.0),
        egui::Align2::CENTER_CENTER,
        "Drop files or folders here",
        egui::FontId::proportional(22.0),
        strong,
    );
    ui.painter().text(
        c + Vec2::new(0.0, 14.0),
        egui::Align2::CENTER_CENTER,
        "or click to choose · photos, HEIC, PSD, AI, SVG, PDF, Word, PowerPoint, Excel, Markdown",
        egui::FontId::proportional(13.0),
        weak,
    );
    if resp.clicked() {
        app.add_paths(pick_files());
    }
}

fn file_view(ui: &mut egui::Ui, app: &mut App, hovering: bool) {
    let running = matches!(app.phase, Phase::Running { .. });
    let n = app.rows.len();

    // Header: count, add, clear.
    ui.horizontal(|ui| {
        ui.heading(format!("{n} file{}", if n == 1 { "" } else { "s" }));
        if hovering {
            ui.label(RichText::new("  drop to add").color(ui.visuals().selection.bg_fill));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.add_enabled(!running, egui::Button::new("Clear")).clicked() {
                app.clear();
            }
            if ui.add_enabled(!running, egui::Button::new("Add files…")).clicked() {
                let picked = pick_files();
                app.add_paths(picked);
            }
        });
    });
    if matches!(app.phase, Phase::Empty) {
        return;
    }
    ui.add_space(4.0);

    // Target format chips.
    ui.label(RichText::new("Convert all to:").weak());
    ui.horizontal_wrapped(|ui| {
        for (t, count) in app.targets.clone() {
            let text = if count == n { t.label() } else { format!("{}  {count}/{n}", t.label()) };
            let chip = ui.add_enabled(!running, egui::Button::selectable(app.target == Some(t), text));
            let chip =
                if count == n { chip } else { chip.on_hover_text(format!("{count} of {n} files can become {t}; the rest are skipped.")) };
            if chip.clicked() {
                app.target = Some(t);
            }
        }
    });

    // Options, collapsed by default; only knobs that matter for the chosen target are shown.
    ui.add_space(2.0);
    ui.add_enabled_ui(!running, |ui| {
        egui::CollapsingHeader::new("Options").default_open(false).show(ui, |ui| {
            let lossy = matches!(
                app.target,
                Some(Target::Image(Format::Jpeg | Format::WebP)) | Some(Target::PdfPages(Format::Jpeg)) | Some(Target::Pdf)
            );
            let renders = matches!(app.target, Some(Target::PdfPages(_)))
                || app.rows.iter().any(|r| matches!(r.format, Ok(Format::Ai | Format::Svg)));
            if lossy || app.target.is_none() {
                ui.add(egui::Slider::new(&mut app.opts.quality, 1..=100).text("Quality (100 = lossless WebP)"));
            }
            if renders || app.target.is_none() {
                ui.add(egui::Slider::new(&mut app.opts.dpi, 36..=600).text("DPI for pages, AI and SVG"));
            }
            ui.horizontal(|ui| {
                let where_ = match &app.opts.out_dir {
                    Some(d) => d.display().to_string(),
                    None => "Next to each file".into(),
                };
                ui.label(format!("Save to: {where_}"));
                if ui.button("Choose folder…").clicked()
                    && let Some(d) = rfd::FileDialog::new().set_title("Save converted files to").pick_folder()
                {
                    app.opts.out_dir = Some(d);
                }
                if app.opts.out_dir.is_some() && ui.button("Reset").clicked() {
                    app.opts.out_dir = None;
                }
            });
        });
    });
    ui.add_space(4.0);
    ui.separator();

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Grid::new("files").num_columns(2).striped(true).spacing([12.0, 6.0]).show(ui, |ui| {
            for r in &app.rows {
                let name = r.path.file_name().unwrap_or_default().to_string_lossy();
                ui.label(name).on_hover_text(r.path.display().to_string());
                status_cell(ui, r);
                ui.end_row();
            }
        });
    });
}

fn action_bar(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App) {
    let n = app.rows.len();
    ui.add_space(10.0);
    ui.horizontal(|ui| match &app.phase {
        Phase::Running { cancel, .. } => {
            ui.spinner();
            let done = app.rows.iter().filter(|r| !matches!(r.status, Status::Waiting | Status::Running)).count();
            ui.label(format!("Converting… {done}/{n}"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    cancel.store(true, Ordering::Relaxed);
                }
            });
        }
        Phase::Done(s) => {
            let s = *s;
            ui.label(format!("{} converted · {} failed · {} skipped", s.converted, s.failed, s.skipped));
            if let Some(msg) = &app.notice {
                ui.label(RichText::new(msg).weak());
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(out) = app.last_output.clone()
                    && ui.add(egui::Button::new(RichText::new("Open folder").strong())).clicked()
                {
                    open_folder(out.parent().unwrap_or(Path::new(".")));
                }
                if ui.button("Convert again").clicked() {
                    app.phase = Phase::Ready;
                    for r in &mut app.rows {
                        r.status = Status::Waiting;
                    }
                }
            });
        }
        _ => {
            ui.label(
                RichText::new(match &app.opts.out_dir {
                    Some(_) => "Saves to the chosen folder",
                    None => "Saves next to each file",
                })
                .weak(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = match app.target {
                    Some(t) => format!("Convert to {}", t.label()),
                    None => "Choose a format".into(),
                };
                let convert =
                    ui.add_enabled(app.target.is_some(), egui::Button::new(RichText::new(label).strong()).min_size(Vec2::new(150.0, 30.0)));
                if convert.clicked()
                    && let Some(t) = app.target
                {
                    app.start_batch(ctx, t);
                }
                if app.combinable() >= 2 && ui.button("Combine into one PDF").clicked() {
                    app.start_combine(ctx);
                }
            });
        }
    });
}

fn status_cell(ui: &mut egui::Ui, r: &Row) {
    let red = Color32::from_rgb(200, 60, 50);
    let green = Color32::from_rgb(30, 142, 62);
    match (&r.format, &r.status) {
        (Err(e), _) => {
            ui.label(RichText::new(format!("✖ {e}")).color(red));
        }
        (Ok(f), Status::Waiting) => {
            ui.label(RichText::new(f.to_string()).weak());
        }
        (_, Status::Running) => {
            ui.spinner();
        }
        (_, Status::Done(outs)) => {
            let first = outs.first().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let text = if outs.len() > 1 { format!("✔ {first} + {} more", outs.len() - 1) } else { format!("✔ {first}") };
            ui.label(RichText::new(text).color(green));
        }
        (_, Status::Failed(e)) => {
            ui.label(RichText::new(format!("✖ {e}")).color(red));
        }
        (_, Status::Skipped(why)) => {
            ui.label(RichText::new(format!("skipped: {why}")).weak());
        }
    }
}

fn open_folder(dir: &Path) {
    #[cfg(windows)]
    let cmd = "explorer";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let cmd = "xdg-open";
    let _ = std::process::Command::new(cmd).arg(dir).spawn();
}
