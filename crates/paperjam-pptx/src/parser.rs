use crate::document::{PptxDocument, SlideData, TextBlock};
use crate::error::{PptxError, Result};
use crate::metadata;
use paperjam_model::table::{Cell, Row, Table, TableStrategy};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use zip::ZipArchive;

/// Parse a PPTX file from raw bytes into a `PptxDocument`.
pub fn parse_pptx(bytes: &[u8]) -> Result<PptxDocument> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let meta = metadata::parse_metadata(&mut archive)?;
    let slide_names = find_slide_files(&archive);

    let mut slides = Vec::with_capacity(slide_names.len());
    for (idx, name) in slide_names.iter().enumerate() {
        let slide_xml = read_zip_entry(&mut archive, name)?;
        let notes_path = format!("ppt/notesSlides/notesSlide{}.xml", idx + 1);
        let notes_xml = read_zip_entry(&mut archive, &notes_path).ok();
        let slide = parse_slide(&slide_xml, idx + 1, notes_xml.as_deref())?;
        slides.push(slide);
    }

    Ok(PptxDocument {
        slides,
        metadata: meta,
        raw_bytes: bytes.to_vec(),
    })
}

/// Discover slide file paths inside the archive, sorted by slide number.
fn find_slide_files<R: Read + std::io::Seek>(archive: &ZipArchive<R>) -> Vec<String> {
    let mut names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let name = archive.name_for_index(i).map(|s| s.to_string())?;
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    // Sort numerically by slide number.
    names.sort_by_key(|name| extract_slide_number(name));
    names
}

/// Extract the numeric part from a path like "ppt/slides/slide3.xml" -> 3.
fn extract_slide_number(name: &str) -> usize {
    let stem = name
        .strip_prefix("ppt/slides/slide")
        .unwrap_or("")
        .strip_suffix(".xml")
        .unwrap_or("");
    stem.parse::<usize>().unwrap_or(usize::MAX)
}

/// Read a ZIP entry by path and return its contents as a UTF-8 string.
fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<String> {
    let mut entry = archive
        .by_name(path)
        .map_err(|_| PptxError::MissingEntry(path.to_string()))?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Slide XML parsing
// ---------------------------------------------------------------------------

/// State tracked while walking a slide's XML event stream.
struct SlideParser {
    index: usize,
    title: Option<String>,
    text_blocks: Vec<TextBlock>,
    tables: Vec<Table>,

    // Shape-level tracking
    in_shape: bool,
    is_title_shape: bool,
    is_subtitle_shape: bool,

    // Text body / paragraph tracking
    in_text_body: bool,
    in_paragraph: bool,
    paragraph_text: String,
    paragraph_level: u8,
    paragraph_is_bullet: bool,

    // Table tracking
    in_table: bool,
    table_rows: Vec<Row>,
    current_row_cells: Vec<Cell>,
    in_table_cell: bool,
    cell_text: String,
}

impl SlideParser {
    fn new(index: usize) -> Self {
        Self {
            index,
            title: None,
            text_blocks: Vec::new(),
            tables: Vec::new(),
            in_shape: false,
            is_title_shape: false,
            is_subtitle_shape: false,
            in_text_body: false,
            in_paragraph: false,
            paragraph_text: String::new(),
            paragraph_level: 0,
            paragraph_is_bullet: false,
            in_table: false,
            table_rows: Vec::new(),
            current_row_cells: Vec::new(),
            in_table_cell: false,
            cell_text: String::new(),
        }
    }

    fn finish_paragraph(&mut self) {
        let text = self.paragraph_text.trim().to_string();
        if text.is_empty() {
            self.paragraph_text.clear();
            return;
        }

        if self.is_title_shape && self.title.is_none() {
            self.title = Some(text.clone());
        }

        let is_title = self.is_title_shape;
        let is_bullet = self.paragraph_is_bullet;
        let level = self.paragraph_level;

        self.text_blocks.push(TextBlock {
            text,
            is_title,
            is_bullet,
            level,
        });

        self.paragraph_text.clear();
    }

    fn finish_cell(&mut self) {
        let text = self.cell_text.trim().to_string();
        self.current_row_cells.push(Cell {
            text,
            bbox: (0.0, 0.0, 0.0, 0.0),
            col_span: 1,
            row_span: 1,
        });
        self.cell_text.clear();
    }

    fn finish_row(&mut self) {
        if !self.current_row_cells.is_empty() {
            self.table_rows.push(Row {
                cells: std::mem::take(&mut self.current_row_cells),
                y_min: 0.0,
                y_max: 0.0,
            });
        }
    }

    fn finish_table(&mut self) {
        if self.table_rows.is_empty() {
            return;
        }
        let col_count = self
            .table_rows
            .iter()
            .map(|r| r.cells.len())
            .max()
            .unwrap_or(0);
        self.tables.push(Table {
            bbox: (0.0, 0.0, 0.0, 0.0),
            rows: std::mem::take(&mut self.table_rows),
            col_count,
            strategy: TableStrategy::Auto,
        });
    }

    fn into_slide(self, notes: Option<String>) -> SlideData {
        SlideData {
            index: self.index,
            title: self.title,
            text_blocks: self.text_blocks,
            tables: self.tables,
            notes,
        }
    }
}

/// Extract the local part of a possibly namespaced tag name (e.g., `p:sp` -> `sp`).
///
/// Returns an owned `String` to avoid lifetime issues with quick-xml temporaries.
fn local_name_owned(full: &[u8]) -> String {
    let s = std::str::from_utf8(full).unwrap_or("");
    match s.rfind(':') {
        Some(pos) => s[pos + 1..].to_string(),
        None => s.to_string(),
    }
}

/// Check whether a tag belongs to the PresentationML namespace prefix `p:`.
fn is_p_ns(full: &[u8]) -> bool {
    let s = std::str::from_utf8(full).unwrap_or("");
    s.starts_with("p:")
}

/// Parse a single slide's XML into `SlideData`.
fn parse_slide(xml: &str, index: usize, notes_xml: Option<&str>) -> Result<SlideData> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut parser = SlideParser::new(index);

    // Track nesting depth for shapes so we correctly detect when we leave one.
    let mut shape_depth: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name();
                let name_ref = name_bytes.as_ref();
                let local = local_name_owned(name_ref);
                let p_ns = is_p_ns(name_ref);
                match local.as_str() {
                    // <p:sp> -- shape start
                    "sp" if p_ns => {
                        parser.in_shape = true;
                        parser.is_title_shape = false;
                        parser.is_subtitle_shape = false;
                        shape_depth = 1;
                    }
                    // <p:graphicFrame> may contain tables
                    "graphicFrame" if p_ns => {
                        parser.in_shape = true;
                        shape_depth = 1;
                    }
                    _ if parser.in_shape => {
                        shape_depth += 1;
                        handle_shape_start(&mut parser, &local);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) if parser.in_shape => {
                let local = local_name_owned(e.name().as_ref());
                handle_shape_empty(&mut parser, e, &local)?;
            }
            Ok(Event::Text(ref e)) => {
                if parser.in_table_cell {
                    let t = e.unescape().unwrap_or_default();
                    parser.cell_text.push_str(&t);
                } else if parser.in_paragraph {
                    let t = e.unescape().unwrap_or_default();
                    parser.paragraph_text.push_str(&t);
                }
            }
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name();
                let name_ref = name_bytes.as_ref();
                let local = local_name_owned(name_ref);
                let p_ns = is_p_ns(name_ref);
                if parser.in_shape {
                    match local.as_str() {
                        "p" if !parser.in_table => {
                            // End of <a:p> paragraph
                            parser.finish_paragraph();
                            parser.in_paragraph = false;
                        }
                        "txBody" => {
                            parser.in_text_body = false;
                        }
                        "tc" => {
                            parser.finish_cell();
                            parser.in_table_cell = false;
                        }
                        "tr" => {
                            parser.finish_row();
                        }
                        "tbl" => {
                            parser.finish_table();
                            parser.in_table = false;
                        }
                        "sp" if p_ns => {
                            parser.in_shape = false;
                            shape_depth = 0;
                        }
                        "graphicFrame" if p_ns => {
                            parser.in_shape = false;
                            shape_depth = 0;
                        }
                        _ => {
                            shape_depth = shape_depth.saturating_sub(1);
                            if shape_depth == 0 {
                                parser.in_shape = false;
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PptxError::Xml(format!("slide XML error: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    let notes = notes_xml.and_then(|xml| parse_notes_text(xml).ok());

    Ok(parser.into_slide(notes))
}

/// Handle a Start event inside a shape.
fn handle_shape_start(parser: &mut SlideParser, local: &str) {
    match local {
        "txBody" => {
            parser.in_text_body = true;
        }
        "p" if parser.in_text_body && !parser.in_table => {
            parser.in_paragraph = true;
            parser.paragraph_text.clear();
            parser.paragraph_level = 0;
            parser.paragraph_is_bullet = false;
        }
        "p" if parser.in_table_cell => {
            // paragraph inside a table cell -- just accumulate text
        }
        "tbl" => {
            parser.in_table = true;
            parser.table_rows.clear();
        }
        "tr" => {
            parser.current_row_cells.clear();
        }
        "tc" => {
            parser.in_table_cell = true;
            parser.cell_text.clear();
        }
        _ => {}
    }
}

/// Handle an Empty (self-closing) event inside a shape.
fn handle_shape_empty(
    parser: &mut SlideParser,
    e: &quick_xml::events::BytesStart,
    local: &str,
) -> Result<()> {
    match local {
        // <p:ph type="title"/> or <p:ph type="ctrTitle"/>
        "ph" => {
            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"type" {
                    let val = std::str::from_utf8(&attr.value).unwrap_or("");
                    match val {
                        "title" | "ctrTitle" => parser.is_title_shape = true,
                        "subTitle" => parser.is_subtitle_shape = true,
                        _ => {}
                    }
                }
            }
        }
        // <a:pPr lvl="1"> paragraph properties (may also be Start, handled elsewhere)
        "pPr" => {
            parse_paragraph_props(parser, e)?;
        }
        // <a:buNone/> means explicitly no bullet
        "buNone" => {
            parser.paragraph_is_bullet = false;
        }
        // <a:buChar/> or <a:buAutoNum/> means bulleted
        "buChar" | "buAutoNum" | "buBlip" => {
            parser.paragraph_is_bullet = true;
        }
        _ => {}
    }
    Ok(())
}

/// Extract paragraph-level attributes from `<a:pPr>`.
fn parse_paragraph_props(
    parser: &mut SlideParser,
    e: &quick_xml::events::BytesStart,
) -> Result<()> {
    for attr in e.attributes() {
        let attr = attr?;
        if attr.key.as_ref() == b"lvl" {
            let val = std::str::from_utf8(&attr.value).unwrap_or("0");
            parser.paragraph_level = val.parse::<u8>().unwrap_or(0);
            // A non-zero level usually implies a bullet list
            if parser.paragraph_level > 0 {
                parser.paragraph_is_bullet = true;
            }
        }
    }
    Ok(())
}

/// Parse speaker notes from a notesSlide XML.
fn parse_notes_text(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut in_text_body = false;
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name_owned(e.name().as_ref());
                if local == "txBody" {
                    in_text_body = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let local = local_name_owned(e.name().as_ref());
                if local == "txBody" {
                    in_text_body = false;
                }
                // Separate paragraphs with newlines
                if local == "p" && in_text_body && !text.is_empty() {
                    text.push('\n');
                }
            }
            Ok(Event::Text(ref e)) if in_text_body => {
                text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PptxError::Xml(format!("notes XML error: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(text.trim().to_string())
}
