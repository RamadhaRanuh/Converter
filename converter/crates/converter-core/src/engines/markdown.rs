//! Markdown → HTML (comrak) and Markdown → PDF (comrak's tree mapped onto office2pdf's document model,
//! which lays out with Typst). GitHub-flavoured extensions: tables, strikethrough, autolinks, task lists.
use std::io::Write;
use std::path::Path;

use comrak::nodes::{ListType, NodeValue};
use comrak::{Arena, Node, Options as MdOptions};
use office2pdf::ir::{
    Block, Document, FlowPage, List, ListItem, ListKind, Margins, Metadata, Page, PageSize, Paragraph, ParagraphStyle, Run, StyleSheet,
    Table, TableCell, TableRow, TextStyle,
};

use crate::Error;

fn md_options() -> MdOptions<'static> {
    let mut o = MdOptions::default();
    o.extension.table = true;
    o.extension.strikethrough = true;
    o.extension.autolink = true;
    o.extension.tasklist = true;
    o
}

fn read(src: &Path) -> Result<String, Error> {
    let bytes = std::fs::read(src).map_err(|e| Error::io(src, e))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Markdown → a standalone HTML page with light, readable styling.
pub(crate) fn to_html(src: &Path, out: &mut dyn Write) -> Result<(), Error> {
    let body = comrak::markdown_to_html(&read(src)?, &md_options());
    let title = src.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let title = title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let page = format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n<style>body{{max-width:46rem;margin:2rem auto;padding:0 1rem;font:16px/1.6 system-ui,sans-serif;color:#1c1c1e}}pre,code{{background:#f4f4f5;border-radius:4px}}pre{{padding:.75rem;overflow:auto}}code{{padding:.1rem .3rem}}pre code{{padding:0}}table{{border-collapse:collapse}}td,th{{border:1px solid #ccc;padding:.3rem .6rem}}blockquote{{margin-left:0;padding-left:1rem;border-left:3px solid #ccc;color:#555}}img{{max-width:100%}}</style>\n</head><body>\n{body}</body></html>\n"
    );
    out.write_all(page.as_bytes()).map_err(|e| Error::encode("HTML", e))
}

/// Markdown → PDF on A4.
pub(crate) fn to_pdf(src: &Path, out: &mut dyn Write) -> Result<(), Error> {
    let text = read(src)?;
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &text, &md_options());
    let margins = Margins::default();
    let size = PageSize::default();
    let mut ctx = Ctx { text_width: size.width - margins.left - margins.right, blocks: Vec::new() };
    for child in root.children() {
        ctx.block(child, 0);
    }
    let doc = Document {
        metadata: Metadata { title: src.file_stem().map(|s| s.to_string_lossy().into_owned()), ..Default::default() },
        pages: vec![Page::Flow(FlowPage {
            size,
            margins,
            content: ctx.blocks,
            header: None,
            footer: None,
            first_header: None,
            first_footer: None,
            columns: None,
            line_grid_pitch: None,
            line_grid_snaps_lines: false,
            page_numbering: None,
        })],
        styles: StyleSheet::default(),
    };
    let pdf = office2pdf::render_document(&doc).map_err(|e| Error::encode("PDF", e))?;
    out.write_all(&pdf).map_err(|e| Error::encode("PDF", e))
}

const MONO: &str = "DejaVu Sans Mono";
const BODY_PT: f64 = 11.0;

struct Ctx {
    text_width: f64,
    blocks: Vec<Block>,
}

fn para(runs: Vec<Run>, style: ParagraphStyle) -> Paragraph {
    Paragraph { style, runs }
}

fn run(text: impl Into<String>, style: TextStyle, href: Option<String>) -> Run {
    Run { text: text.into(), style, href, footnote: None }
}

fn spaced(before: f64, after: f64) -> ParagraphStyle {
    ParagraphStyle { space_before: Some(before), space_after: Some(after), ..Default::default() }
}

impl Ctx {
    fn block<'a>(&mut self, node: Node<'a>, depth: u32) {
        let value = node.data().value.clone();
        match value {
            NodeValue::Heading(h) => {
                let size = [22.0, 18.0, 15.0, 13.0, 12.0, 11.0][(h.level.clamp(1, 6) - 1) as usize];
                let base = TextStyle { bold: Some(true), font_size: Some(size), ..Default::default() };
                let mut style = spaced(size * 0.8, size * 0.4);
                style.heading_level = Some(h.level);
                self.blocks.push(Block::Paragraph(para(inlines(node, &base), style)));
            }
            NodeValue::Paragraph => {
                let base = TextStyle { font_size: Some(BODY_PT), ..Default::default() };
                self.blocks.push(Block::Paragraph(para(inlines(node, &base), spaced(0.0, 6.0))));
            }
            NodeValue::CodeBlock(cb) => {
                let style = TextStyle { font_family: Some(MONO.into()), font_size: Some(9.5), ..Default::default() };
                let lines: Vec<&str> = cb.literal.trim_end_matches('\n').split('\n').collect();
                let last = lines.len().saturating_sub(1);
                for (i, line) in lines.into_iter().enumerate() {
                    let mut ps = spaced(if i == 0 { 4.0 } else { 0.0 }, if i == last { 8.0 } else { 0.0 });
                    ps.background = Some(office2pdf::ir::Color { r: 244, g: 244, b: 245 });
                    ps.indent_left = Some(6.0);
                    // A lone space keeps blank lines from collapsing.
                    let text = if line.is_empty() { " ".to_string() } else { line.replace('\t', "    ") };
                    self.blocks.push(Block::Paragraph(para(vec![run(text, style.clone(), None)], ps)));
                }
            }
            NodeValue::BlockQuote => {
                let start = self.blocks.len();
                for c in node.children() {
                    self.block(c, depth);
                }
                for b in &mut self.blocks[start..] {
                    if let Block::Paragraph(p) = b {
                        p.style.indent_left = Some(p.style.indent_left.unwrap_or(0.0) + 18.0);
                        for r in &mut p.runs {
                            r.style.italic = Some(true);
                            r.style.color = Some(office2pdf::ir::Color { r: 85, g: 85, b: 85 });
                        }
                    }
                }
            }
            NodeValue::List(l) => {
                let kind = if l.list_type == ListType::Ordered { ListKind::Ordered } else { ListKind::Unordered };
                let mut items = Vec::new();
                self.list_items(node, depth, &mut items, l.start as u32);
                self.blocks.push(Block::List(List { kind, items, level_styles: Default::default() }));
            }
            NodeValue::Table(_) => self.table(node),
            NodeValue::ThematicBreak => {
                let mut ps = spaced(6.0, 6.0);
                ps.alignment = Some(office2pdf::ir::Alignment::Center);
                self.blocks.push(Block::Paragraph(para(vec![run("⁂", TextStyle::default(), None)], ps)));
            }
            NodeValue::HtmlBlock(h) => {
                let t = h.literal.trim();
                if !t.is_empty() {
                    let style = TextStyle { font_family: Some(MONO.into()), font_size: Some(9.5), ..Default::default() };
                    self.blocks.push(Block::Paragraph(para(vec![run(t, style, None)], spaced(0.0, 6.0))));
                }
            }
            _ => {
                for c in node.children() {
                    self.block(c, depth);
                }
            }
        }
    }

    /// Flattens a (possibly nested) list into items with indent levels, as the IR expects.
    fn list_items<'a>(&mut self, list: Node<'a>, level: u32, items: &mut Vec<ListItem>, start: u32) {
        for (i, item) in list.children().enumerate() {
            let mut content = Vec::new();
            let mut nested = Vec::new();
            let task = match &item.data().value {
                NodeValue::TaskItem(t) => Some(t.symbol.is_some()),
                _ => None,
            };
            for c in item.children() {
                match &c.data().value {
                    NodeValue::List(_) => nested.push(c),
                    NodeValue::Paragraph => {
                        let base = TextStyle { font_size: Some(BODY_PT), ..Default::default() };
                        let mut runs = inlines(c, &base);
                        if let (Some(done), true) = (task, content.is_empty()) {
                            runs.insert(0, run(if done { "☑ " } else { "☐ " }, base.clone(), None));
                        }
                        content.push(para(runs, spaced(0.0, 2.0)));
                    }
                    _ => {
                        let mut sub = Ctx { text_width: self.text_width, blocks: Vec::new() };
                        sub.block(c, level);
                        content.extend(sub.blocks.into_iter().filter_map(|b| if let Block::Paragraph(p) = b { Some(p) } else { None }));
                    }
                }
            }
            if content.is_empty() {
                content.push(para(vec![], ParagraphStyle::default()));
            }
            items.push(ListItem { content, level, start_at: (i == 0 && start > 1).then_some(start) });
            for n in nested {
                let s = if let NodeValue::List(l) = &n.data().value { l.start as u32 } else { 1 };
                self.list_items(n, level + 1, items, s);
            }
        }
    }

    fn table<'a>(&mut self, node: Node<'a>) {
        let mut rows = Vec::new();
        let mut header_rows = 0;
        let mut cols = 0;
        for row in node.children() {
            let header = matches!(row.data().value, NodeValue::TableRow(true));
            header_rows += header as usize;
            let base = TextStyle { font_size: Some(10.0), bold: header.then_some(true), ..Default::default() };
            let cells: Vec<TableCell> = row
                .children()
                .map(|cell| TableCell {
                    content: vec![Block::Paragraph(para(inlines(cell, &base), ParagraphStyle::default()))],
                    ..Default::default()
                })
                .collect();
            cols = cols.max(cells.len());
            rows.push(TableRow { cells, height: None, minimum_height: None });
        }
        if cols == 0 {
            return;
        }
        self.blocks.push(Block::Table(Table {
            rows,
            column_widths: vec![self.text_width / cols as f64; cols],
            header_row_count: header_rows,
            ..Default::default()
        }));
        self.blocks.push(Block::Paragraph(para(vec![], spaced(0.0, 6.0))));
    }
}

/// Collects a block's inline children into styled runs.
fn inlines<'a>(node: Node<'a>, base: &TextStyle) -> Vec<Run> {
    let mut runs = Vec::new();
    collect(node, base, None, &mut runs);
    runs
}

fn collect<'a>(node: Node<'a>, style: &TextStyle, href: Option<&str>, runs: &mut Vec<Run>) {
    for c in node.children() {
        let value = c.data().value.clone();
        let link = href.map(str::to_string);
        match value {
            NodeValue::Text(t) => runs.push(run(t.to_string(), style.clone(), link)),
            NodeValue::Code(code) => {
                let s = TextStyle { font_family: Some(MONO.into()), ..style.clone() };
                runs.push(run(code.literal, s, link));
            }
            NodeValue::SoftBreak => runs.push(run(" ", style.clone(), link)),
            NodeValue::LineBreak => runs.push(run("\n", style.clone(), link)),
            NodeValue::HtmlInline(h) => runs.push(run(h, style.clone(), link)),
            NodeValue::Emph => collect(c, &TextStyle { italic: Some(true), ..style.clone() }, href, runs),
            NodeValue::Strong => collect(c, &TextStyle { bold: Some(true), ..style.clone() }, href, runs),
            NodeValue::Strikethrough => collect(c, &TextStyle { strikethrough: Some(true), ..style.clone() }, href, runs),
            NodeValue::Link(l) => {
                let s = TextStyle { color: Some(office2pdf::ir::Color { r: 31, g: 95, b: 191 }), underline: Some(true), ..style.clone() };
                collect(c, &s, Some(&l.url), runs);
            }
            NodeValue::Image(img) => {
                let alt: String = c
                    .descendants()
                    .filter_map(|d| if let NodeValue::Text(t) = &d.data().value { Some(t.to_string()) } else { None })
                    .collect();
                let label = if alt.is_empty() { img.url.clone() } else { alt };
                runs.push(run(format!("[image: {label}]"), TextStyle { italic: Some(true), ..style.clone() }, link));
            }
            _ => collect(c, style, href, runs),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Format, Options, Target, convert};

    const MD: &str = "# Title\n\nSome *emphasis*, **bold**, `code` and a [link](https://example.com).\n\n\
        - one\n- two\n  1. nested\n  2. again\n- [x] done\n\n> quoted text\n\n```rust\nfn main() {}\n```\n\n\
        | A | B |\n|---|---|\n| 1 | 2 |\n\n---\n\nEnd.\n";

    #[test]
    fn markdown_to_html_renders_gfm() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("notes.md");
        std::fs::write(&p, MD).unwrap();
        let out = convert(&p, Target::Html, &Options::default()).unwrap();
        let html = std::fs::read_to_string(&out[0]).unwrap();
        for needle in ["<h1>Title</h1>", "<em>emphasis</em>", "<table>", "<blockquote>", "checked", "<title>notes</title>"] {
            assert!(html.contains(needle), "missing {needle}");
        }
    }

    #[test]
    fn markdown_to_pdf_lays_out_every_block() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("notes.md");
        std::fs::write(&p, MD.repeat(8)).unwrap();
        let pdf = convert(&p, Target::Pdf, &Options::default()).unwrap();
        let pages = convert(&pdf[0], Target::PdfPages(Format::Png), &Options { dpi: 40, ..Default::default() }).unwrap();
        assert!(pages.len() >= 2, "eight copies should flow onto several pages, got {}", pages.len());
        let img = image::open(&pages[0]).unwrap().to_luma8();
        assert!(img.pixels().filter(|p| p.0[0] < 128).count() > 500, "text should be drawn");
    }
}
