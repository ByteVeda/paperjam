use std::collections::HashMap;

use crate::document::Document;
use crate::error::Result;
use crate::page::Page;
use crate::table::{Table, TableExtractionOptions};
use crate::text::layout::{TextLine, TextSpan};

/// A block of structured content extracted from a page.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    /// A heading with an inferred level (1 = largest, up to 6).
    Heading {
        text: String,
        level: u8,
        bbox: (f64, f64, f64, f64),
        page: u32,
    },
    /// A paragraph of body text.
    Paragraph {
        text: String,
        bbox: (f64, f64, f64, f64),
        page: u32,
    },
    /// An item in a bulleted or numbered list.
    ListItem {
        text: String,
        indent_level: u8,
        bbox: (f64, f64, f64, f64),
        page: u32,
    },
    /// A table detected on the page.
    Table { table: Table, page: u32 },
}

impl ContentBlock {
    /// Get the block type as a string identifier.
    pub fn block_type(&self) -> &str {
        match self {
            ContentBlock::Heading { .. } => "heading",
            ContentBlock::Paragraph { .. } => "paragraph",
            ContentBlock::ListItem { .. } => "list_item",
            ContentBlock::Table { .. } => "table",
        }
    }

    /// Get the text content (empty string for tables).
    pub fn text(&self) -> &str {
        match self {
            ContentBlock::Heading { text, .. }
            | ContentBlock::Paragraph { text, .. }
            | ContentBlock::ListItem { text, .. } => text,
            ContentBlock::Table { .. } => "",
        }
    }

    /// Get the bounding box.
    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        match self {
            ContentBlock::Heading { bbox, .. }
            | ContentBlock::Paragraph { bbox, .. }
            | ContentBlock::ListItem { bbox, .. } => *bbox,
            ContentBlock::Table { table, .. } => table.bbox,
        }
    }

    /// Get the page number.
    pub fn page(&self) -> u32 {
        match self {
            ContentBlock::Heading { page, .. }
            | ContentBlock::Paragraph { page, .. }
            | ContentBlock::ListItem { page, .. }
            | ContentBlock::Table { page, .. } => *page,
        }
    }
}

/// Options controlling structure extraction heuristics.
pub struct StructureOptions {
    /// Minimum font size ratio vs body text to consider a heading.
    /// Default: 1.2 (20% larger).
    pub heading_size_ratio: f64,
    /// Whether to detect list items by bullet/number prefixes.
    pub detect_lists: bool,
    /// Whether to include tables from table extraction.
    pub include_tables: bool,
}

impl Default for StructureOptions {
    fn default() -> Self {
        Self {
            heading_size_ratio: 1.2,
            detect_lists: true,
            include_tables: true,
        }
    }
}

/// Extract structured content blocks from a single page.
pub fn extract_structure(page: &Page, options: &StructureOptions) -> Result<Vec<ContentBlock>> {
    let lines = page.text_lines()?;
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let spans = page.text_spans()?;
    let body_font_size = find_body_font_size(&spans);
    let heading_threshold = body_font_size * options.heading_size_ratio;

    // Build heading level map: distinct font sizes >= threshold, sorted largest→smallest
    let heading_map = build_heading_map(&spans, heading_threshold);

    // Find the left margin (minimum x across all body-sized lines)
    let left_margin = find_left_margin(&lines, body_font_size);

    // Classify each line
    let classified: Vec<ClassifiedLine> = lines
        .iter()
        .map(|line| classify_line(line, body_font_size, &heading_map, left_margin, options))
        .collect();

    // Group consecutive same-type lines into content blocks
    let blocks = group_into_blocks(&classified, body_font_size, page.number);

    // Optionally interleave tables
    if options.include_tables {
        let table_opts = TableExtractionOptions::default();
        match page.extract_tables(&table_opts) {
            Ok(tables) if !tables.is_empty() => {
                return Ok(interleave_tables(blocks, tables, page.number));
            }
            _ => {}
        }
    }

    Ok(blocks)
}

/// Extract structured content from all pages of a document.
pub fn extract_document_structure(
    doc: &Document,
    options: &StructureOptions,
) -> Result<Vec<ContentBlock>> {
    let mut all_blocks = Vec::new();
    for i in 1..=doc.page_count() as u32 {
        let page = doc.page(i)?;
        let mut page_blocks = extract_structure(&page, options)?;
        all_blocks.append(&mut page_blocks);
    }
    Ok(all_blocks)
}

// --- Internal helpers ---

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineKind {
    Heading(u8),
    Paragraph,
    ListItem(u8),
}

#[derive(Debug, Clone)]
struct ClassifiedLine {
    kind: LineKind,
    text: String,
    bbox: (f64, f64, f64, f64),
    y_center: f64,
}

/// Find the most common font size (the "body" font size).
fn find_body_font_size(spans: &[TextSpan]) -> f64 {
    if spans.is_empty() {
        return 12.0;
    }

    // Bucket font sizes to 0.5pt increments
    let mut counts: HashMap<i32, (usize, f64)> = HashMap::new();
    for span in spans {
        if span.text.trim().is_empty() {
            continue;
        }
        let bucket = (span.font_size * 2.0).round() as i32;
        let entry = counts.entry(bucket).or_insert((0, span.font_size));
        entry.0 += span.text.len(); // Weight by text length
    }

    counts
        .into_iter()
        .max_by_key(|(_, (count, _))| *count)
        .map(|(_, (_, size))| size)
        .unwrap_or(12.0)
}

/// Build a map of font sizes that qualify as headings → heading level.
fn build_heading_map(spans: &[TextSpan], threshold: f64) -> HashMap<i32, u8> {
    // Collect distinct font sizes above threshold
    let mut sizes: Vec<f64> = spans
        .iter()
        .filter(|s| s.font_size >= threshold && !s.text.trim().is_empty())
        .map(|s| s.font_size)
        .collect();

    // Deduplicate by bucket
    let mut seen = std::collections::HashSet::new();
    sizes.retain(|&s| seen.insert((s * 2.0).round() as i32));
    sizes.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let mut map = HashMap::new();
    for (i, size) in sizes.iter().enumerate() {
        let level = (i as u8 + 1).min(6);
        let bucket = (*size * 2.0).round() as i32;
        map.insert(bucket, level);
    }
    map
}

/// Find the typical left margin (minimum x of body-text lines).
fn find_left_margin(lines: &[TextLine], body_font_size: f64) -> f64 {
    lines
        .iter()
        .filter(|l| {
            let dominant = dominant_font_size(l);
            (dominant - body_font_size).abs() < 1.0
        })
        .filter_map(|l| l.spans.first().map(|s| s.x))
        .fold(f64::INFINITY, f64::min)
        .max(0.0)
}

/// Get the dominant font size in a line (font size of the span with most text).
fn dominant_font_size(line: &TextLine) -> f64 {
    line.spans
        .iter()
        .max_by_key(|s| s.text.len())
        .map(|s| s.font_size)
        .unwrap_or(12.0)
}

/// Check if font name suggests bold.
fn is_bold_font(font_name: &str) -> bool {
    let lower = font_name.to_lowercase();
    lower.contains("bold") || lower.contains("-bd") || lower.contains("heavy")
}

/// Check if a line's text starts with a list marker.
fn detect_list_marker(text: &str) -> bool {
    let trimmed = text.trim_start();
    // Bullet markers
    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("\u{2022} ")  // bullet
        || trimmed.starts_with("\u{2023} ")  // triangular bullet
        || trimmed.starts_with("\u{25E6} ")  // white bullet
        || trimmed.starts_with("\u{2043} ")  // hyphen bullet
        || trimmed.starts_with("\u{2013} ")  // en dash
        || trimmed.starts_with("\u{2014} ")  // em dash
    {
        return true;
    }

    // Number markers: 1. 2) a. iv)
    let bytes = trimmed.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    // Check for digit-based markers: "1." "12)" "3:"
    let mut i = 0;
    if bytes[0].is_ascii_digit() {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')') {
            if i + 1 < bytes.len() && bytes[i + 1] == b' ' {
                return true;
            }
        }
    }

    // Check for letter-based markers: "a." "b)" "A."
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && (bytes[1] == b'.' || bytes[1] == b')')
        && bytes[2] == b' '
    {
        return true;
    }

    false
}

/// Classify a single text line.
fn classify_line(
    line: &TextLine,
    body_font_size: f64,
    heading_map: &HashMap<i32, u8>,
    left_margin: f64,
    options: &StructureOptions,
) -> ClassifiedLine {
    let text = line.text();
    let dominant_size = dominant_font_size(line);
    let bucket = (dominant_size * 2.0).round() as i32;

    let kind = if let Some(&level) = heading_map.get(&bucket) {
        LineKind::Heading(level)
    } else {
        // Check for bold short lines as possible headings
        let dominant_font = line
            .spans
            .iter()
            .max_by_key(|s| s.text.len())
            .map(|s| s.font_name.as_str())
            .unwrap_or("");

        if is_bold_font(dominant_font)
            && (dominant_size - body_font_size).abs() < 1.0
            && text.len() < 100
            && !text.trim().is_empty()
        {
            // Bold text at body size, short line → treat as heading (lowest level)
            let max_existing_level = heading_map.values().copied().max().unwrap_or(0);
            LineKind::Heading((max_existing_level + 1).min(6))
        } else if options.detect_lists && detect_list_marker(&text) {
            // Calculate indent level from x-offset
            let x = line.spans.first().map(|s| s.x).unwrap_or(0.0);
            let indent_pts = (x - left_margin).max(0.0);
            let indent_level = (indent_pts / 36.0).round() as u8;
            LineKind::ListItem(indent_level)
        } else {
            LineKind::Paragraph
        }
    };

    let y_center = (line.bbox.1 + line.bbox.3) / 2.0;

    ClassifiedLine {
        kind,
        text,
        bbox: line.bbox,
        y_center,
    }
}

/// Merge a bounding box with another.
fn union_bbox(
    a: (f64, f64, f64, f64),
    b: (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    (
        a.0.min(b.0),
        a.1.min(b.1),
        a.2.max(b.2),
        a.3.max(b.3),
    )
}

/// Group classified lines into content blocks.
fn group_into_blocks(
    lines: &[ClassifiedLine],
    body_font_size: f64,
    page: u32,
) -> Vec<ContentBlock> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let current = &lines[i];

        match current.kind {
            LineKind::Heading(level) => {
                // Merge consecutive heading lines at the same level
                let mut text = current.text.clone();
                let mut bbox = current.bbox;
                let mut j = i + 1;
                while j < lines.len() {
                    if let LineKind::Heading(l) = lines[j].kind {
                        if l == level {
                            text.push(' ');
                            text.push_str(&lines[j].text);
                            bbox = union_bbox(bbox, lines[j].bbox);
                            j += 1;
                            continue;
                        }
                    }
                    break;
                }
                blocks.push(ContentBlock::Heading {
                    text,
                    level,
                    bbox,
                    page,
                });
                i = j;
            }
            LineKind::Paragraph => {
                // Merge consecutive paragraph lines
                let mut text = current.text.clone();
                let mut bbox = current.bbox;
                let mut j = i + 1;
                while j < lines.len() {
                    if lines[j].kind != LineKind::Paragraph {
                        break;
                    }
                    // Check vertical gap — if large gap, start new paragraph
                    let gap = (lines[j - 1].y_center - lines[j].y_center).abs();
                    if gap > body_font_size * 2.0 {
                        break;
                    }
                    text.push(' ');
                    text.push_str(&lines[j].text);
                    bbox = union_bbox(bbox, lines[j].bbox);
                    j += 1;
                }
                blocks.push(ContentBlock::Paragraph { text, bbox, page });
                i = j;
            }
            LineKind::ListItem(indent) => {
                // Each list item is its own block
                blocks.push(ContentBlock::ListItem {
                    text: current.text.clone(),
                    indent_level: indent,
                    bbox: current.bbox,
                    page,
                });
                i += 1;
            }
        }
    }

    blocks
}

/// Interleave table blocks with text blocks by vertical position.
fn interleave_tables(
    mut text_blocks: Vec<ContentBlock>,
    tables: Vec<Table>,
    page: u32,
) -> Vec<ContentBlock> {
    // Create table blocks
    let table_blocks: Vec<ContentBlock> = tables
        .into_iter()
        .map(|table| ContentBlock::Table { table, page })
        .collect();

    // Merge and sort by y-position (top of page = highest y value in PDF coords, sort descending)
    text_blocks.extend(table_blocks);
    text_blocks.sort_by(|a, b| {
        b.bbox()
            .3
            .partial_cmp(&a.bbox().3)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    text_blocks
}
