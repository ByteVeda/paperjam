use paperjam_model::format::DocumentFormat;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table as ModelTable;

use crate::error::ConvertError;
use crate::intermediate::IntermediateDoc;

/// Generate output bytes in the target format from an [`IntermediateDoc`].
pub fn generate(doc: &IntermediateDoc, format: DocumentFormat) -> Result<Vec<u8>, ConvertError> {
    match format {
        #[cfg(feature = "pdf")]
        DocumentFormat::Pdf => generate_pdf(doc),

        #[cfg(feature = "docx")]
        DocumentFormat::Docx => generate_docx(doc),

        #[cfg(feature = "xlsx")]
        DocumentFormat::Xlsx => generate_xlsx(doc),

        #[cfg(feature = "pptx")]
        DocumentFormat::Pptx => generate_pptx(doc),

        #[cfg(feature = "html")]
        DocumentFormat::Html => generate_html(doc),

        #[cfg(feature = "epub")]
        DocumentFormat::Epub => generate_epub(doc),

        DocumentFormat::Markdown => generate_markdown(doc),

        _ => Err(ConvertError::unsupported(format)),
    }
}

// ---------------------------------------------------------------------------
// Markdown generation (always available)
// ---------------------------------------------------------------------------

fn generate_markdown(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    let mut out = String::new();
    let mut in_list = false;

    for block in &doc.blocks {
        match block {
            ContentBlock::Heading { text, level, .. } => {
                in_list = false;
                ensure_blank_line(&mut out);
                let hashes = "#".repeat(*level as usize);
                out.push_str(&hashes);
                out.push(' ');
                out.push_str(text.trim());
                out.push('\n');
            }
            ContentBlock::Paragraph { text, .. } => {
                in_list = false;
                ensure_blank_line(&mut out);
                out.push_str(text.trim());
                out.push('\n');
            }
            ContentBlock::ListItem {
                text, indent_level, ..
            } => {
                if !in_list {
                    ensure_blank_line(&mut out);
                }
                in_list = true;
                let indent = "  ".repeat(*indent_level as usize);
                out.push_str(&indent);
                out.push_str("- ");
                out.push_str(text.trim());
                out.push('\n');
            }
            ContentBlock::Table { table, .. } => {
                in_list = false;
                ensure_blank_line(&mut out);
                render_markdown_table(&mut out, table);
            }
        }
    }

    // Append standalone tables that aren't already in content blocks.
    if !doc.tables.is_empty() {
        let has_inline_tables = doc
            .blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::Table { .. }));
        if !has_inline_tables {
            for table in &doc.tables {
                ensure_blank_line(&mut out);
                render_markdown_table(&mut out, table);
            }
        }
    }

    let trimmed = out.trim_end();
    let mut result = trimmed.to_string();
    if !result.is_empty() {
        result.push('\n');
    }

    Ok(result.into_bytes())
}

fn ensure_blank_line(out: &mut String) {
    if out.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
}

fn render_markdown_table(out: &mut String, table: &ModelTable) {
    if table.rows.is_empty() {
        return;
    }

    let col_count = table.col_count;

    // First row as header.
    out.push('|');
    if let Some(first_row) = table.rows.first() {
        for j in 0..col_count {
            let text = first_row
                .cells
                .get(j)
                .map(|c| escape_pipe(&c.text))
                .unwrap_or_default();
            out.push(' ');
            out.push_str(&text);
            out.push_str(" |");
        }
    }
    out.push('\n');

    // Separator row.
    out.push('|');
    for _ in 0..col_count {
        out.push_str(" --- |");
    }
    out.push('\n');

    // Data rows.
    for row in table.rows.iter().skip(1) {
        out.push('|');
        for j in 0..col_count {
            let text = row
                .cells
                .get(j)
                .map(|c| escape_pipe(&c.text))
                .unwrap_or_default();
            out.push(' ');
            out.push_str(&text);
            out.push_str(" |");
        }
        out.push('\n');
    }
}

fn escape_pipe(text: &str) -> String {
    text.trim().replace('|', "\\|").replace('\n', " ")
}

// ---------------------------------------------------------------------------
// PDF generation
// ---------------------------------------------------------------------------

#[cfg(feature = "pdf")]
fn generate_pdf(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    use lopdf::dictionary;
    use lopdf::{Document, Object, Stream};

    let mut pdf = Document::with_version("1.7");

    let pages_id = pdf.new_object_id();

    // Built-in font — Helvetica.
    let font_id = pdf.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let font_bold_id = pdf.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
    });

    // Collect text content, splitting into pages of roughly 50 lines each.
    let lines = collect_text_lines(doc);
    let lines_per_page = 50;
    let page_height = 842.0_f32; // A4
    let page_width = 595.0_f32;
    let margin = 72.0_f32;
    let line_height = 14.0_f32;

    let mut page_ids = Vec::new();

    let chunks: Vec<&[(bool, f32, String)]> = if lines.is_empty() {
        // Create at least one blank page.
        vec![&[]]
    } else {
        lines.chunks(lines_per_page).collect()
    };

    for chunk in chunks {
        let mut content = String::new();
        content.push_str("BT\n");

        for (i, (is_bold, font_size, text)) in chunk.iter().enumerate() {
            let font_name = if *is_bold { "/F2" } else { "/F1" };
            let y = page_height - margin - (i as f32 * line_height);
            content.push_str(&format!(
                "{} {} Tf\n{} {} Td\n({}) Tj\n",
                font_name,
                font_size,
                margin,
                y,
                pdf_escape_text(text),
            ));
        }

        content.push_str("ET\n");

        let content_stream = Stream::new(dictionary! {}, content.into_bytes());
        let content_id = pdf.add_object(content_stream);

        let resources = dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
                "F2" => font_bold_id,
            },
        };

        let page_id = pdf.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), Object::Real(page_width), Object::Real(page_height)],
            "Contents" => content_id,
            "Resources" => resources,
        });

        page_ids.push(page_id);
    }

    let page_refs: Vec<Object> = page_ids.iter().map(|id| Object::Reference(*id)).collect();
    let page_count = page_ids.len() as i64;

    pdf.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => page_refs,
            "Count" => page_count,
        }),
    );

    let catalog_id = pdf.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });

    pdf.trailer.set("Root", Object::Reference(catalog_id));

    let mut buf = Vec::new();
    pdf.save_to(&mut buf)
        .map_err(|e| ConvertError::Generation(e.to_string()))?;

    Ok(buf)
}

#[cfg(feature = "pdf")]
fn collect_text_lines(doc: &IntermediateDoc) -> Vec<(bool, f32, String)> {
    let mut lines = Vec::new();

    for block in &doc.blocks {
        match block {
            ContentBlock::Heading { text, level, .. } => {
                let font_size = match level {
                    1 => 20.0_f32,
                    2 => 16.0_f32,
                    3 => 14.0_f32,
                    _ => 12.0_f32,
                };
                lines.push((true, font_size, text.clone()));
            }
            ContentBlock::Paragraph { text, .. } => {
                // Wrap long paragraphs into multiple lines.
                for line in wrap_text(text, 80) {
                    lines.push((false, 11.0, line));
                }
            }
            ContentBlock::ListItem { text, .. } => {
                let bullet_text = format!("  - {}", text);
                lines.push((false, 11.0, bullet_text));
            }
            ContentBlock::Table { table, .. } => {
                for row in &table.rows {
                    let cells: Vec<String> = row.cells.iter().map(|c| c.text.clone()).collect();
                    lines.push((false, 10.0, cells.join("  |  ")));
                }
            }
        }
    }

    lines
}

#[cfg(feature = "pdf")]
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() > max_chars {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(feature = "pdf")]
fn pdf_escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

// ---------------------------------------------------------------------------
// DOCX generation
// ---------------------------------------------------------------------------

#[cfg(feature = "docx")]
fn generate_docx(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    use docx_rs::{Docx, Paragraph, Run, Table, TableCell, TableRow};

    let mut docx = Docx::new();

    for block in &doc.blocks {
        match block {
            ContentBlock::Heading { text, level, .. } => {
                let style = format!("Heading{}", level);
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(text))
                        .style(&style),
                );
            }
            ContentBlock::Paragraph { text, .. } => {
                docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)));
            }
            ContentBlock::ListItem { text, .. } => {
                let bullet_text = format!("\u{2022} {}", text);
                docx =
                    docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&bullet_text)));
            }
            ContentBlock::Table { table, .. } => {
                let mut rows = Vec::new();
                for row in &table.rows {
                    let cells: Vec<TableCell> = row
                        .cells
                        .iter()
                        .map(|c| {
                            TableCell::new().add_paragraph(
                                Paragraph::new().add_run(Run::new().add_text(&c.text)),
                            )
                        })
                        .collect();
                    rows.push(TableRow::new(cells));
                }
                docx = docx.add_table(Table::new(rows));
            }
        }
    }

    // Also add standalone tables not embedded in content blocks.
    if !doc.tables.is_empty() {
        let has_inline_tables = doc
            .blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::Table { .. }));
        if !has_inline_tables {
            for model_table in &doc.tables {
                let mut rows = Vec::new();
                for row in &model_table.rows {
                    let cells: Vec<TableCell> = row
                        .cells
                        .iter()
                        .map(|c| {
                            TableCell::new().add_paragraph(
                                Paragraph::new().add_run(Run::new().add_text(&c.text)),
                            )
                        })
                        .collect();
                    rows.push(TableRow::new(cells));
                }
                docx = docx.add_table(Table::new(rows));
            }
        }
    }

    let mut buf = Vec::new();
    docx.build()
        .pack(&mut std::io::Cursor::new(&mut buf))
        .map_err(|e| ConvertError::Generation(e.to_string()))?;

    Ok(buf)
}

// ---------------------------------------------------------------------------
// XLSX generation
// ---------------------------------------------------------------------------

#[cfg(feature = "xlsx")]
fn generate_xlsx(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    use rust_xlsxwriter::Workbook;

    // Collect all tables — from the tables field and from inline table blocks.
    let mut tables: Vec<&ModelTable> = doc.tables.iter().collect();
    for block in &doc.blocks {
        if let ContentBlock::Table { table, .. } = block {
            tables.push(table);
        }
    }

    // If there are no tables, create a single sheet with text content.
    if tables.is_empty() {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name("Content")
            .map_err(|e| ConvertError::Generation(e.to_string()))?;

        for (row_idx, block) in doc.blocks.iter().enumerate() {
            let text = block.text();
            worksheet
                .write_string(row_idx as u32, 0, text)
                .map_err(|e| ConvertError::Generation(e.to_string()))?;
        }

        let bytes = workbook
            .save_to_buffer()
            .map_err(|e| ConvertError::Generation(e.to_string()))?;
        return Ok(bytes);
    }

    let mut workbook = Workbook::new();

    for (i, table) in tables.iter().enumerate() {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(format!("Sheet{}", i + 1))
            .map_err(|e| ConvertError::Generation(e.to_string()))?;

        for (row_idx, row) in table.rows.iter().enumerate() {
            for (col_idx, cell) in row.cells.iter().enumerate() {
                worksheet
                    .write_string(row_idx as u32, col_idx as u16, &cell.text)
                    .map_err(|e| ConvertError::Generation(e.to_string()))?;
            }
        }
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| ConvertError::Generation(e.to_string()))?;

    Ok(bytes)
}

// ---------------------------------------------------------------------------
// PPTX generation
// ---------------------------------------------------------------------------

#[cfg(feature = "pptx")]
fn generate_pptx(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    // Build a minimal PPTX archive.
    // Each heading starts a new slide; paragraphs and list items become body text
    // on the current slide.
    let slides = build_slides(doc);

    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, content_types_xml(slides.len()).as_bytes())
            .map_err(ConvertError::Io)?;

        // _rels/.rels
        zip.start_file("_rels/.rels", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, RELS_DOT_RELS.as_bytes()).map_err(ConvertError::Io)?;

        // ppt/presentation.xml
        zip.start_file("ppt/presentation.xml", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, presentation_xml(slides.len()).as_bytes())
            .map_err(ConvertError::Io)?;

        // ppt/_rels/presentation.xml.rels
        zip.start_file("ppt/_rels/presentation.xml.rels", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, presentation_rels_xml(slides.len()).as_bytes())
            .map_err(ConvertError::Io)?;

        // ppt/slideLayouts/slideLayout1.xml
        zip.start_file("ppt/slideLayouts/slideLayout1.xml", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, SLIDE_LAYOUT_XML.as_bytes())
            .map_err(ConvertError::Io)?;

        // ppt/slideLayouts/_rels/slideLayout1.xml.rels
        zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, SLIDE_LAYOUT_RELS_XML.as_bytes())
            .map_err(ConvertError::Io)?;

        // ppt/slideMasters/slideMaster1.xml
        zip.start_file("ppt/slideMasters/slideMaster1.xml", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, slide_master_xml(slides.len()).as_bytes())
            .map_err(ConvertError::Io)?;

        // ppt/slideMasters/_rels/slideMaster1.xml.rels
        zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, SLIDE_MASTER_RELS_XML.as_bytes())
            .map_err(ConvertError::Io)?;

        // ppt/theme/theme1.xml
        zip.start_file("ppt/theme/theme1.xml", options)
            .map_err(ConvertError::Zip)?;
        std::io::Write::write_all(&mut zip, THEME_XML.as_bytes()).map_err(ConvertError::Io)?;

        // Individual slide files
        for (i, slide) in slides.iter().enumerate() {
            let idx = i + 1;
            zip.start_file(format!("ppt/slides/slide{}.xml", idx), options)
                .map_err(ConvertError::Zip)?;
            std::io::Write::write_all(&mut zip, slide_xml(&slide.title, &slide.body).as_bytes())
                .map_err(ConvertError::Io)?;

            zip.start_file(format!("ppt/slides/_rels/slide{}.xml.rels", idx), options)
                .map_err(ConvertError::Zip)?;
            std::io::Write::write_all(&mut zip, SLIDE_RELS_XML.as_bytes())
                .map_err(ConvertError::Io)?;
        }

        zip.finish().map_err(ConvertError::Zip)?;
    }

    Ok(buf)
}

#[cfg(feature = "pptx")]
struct PptxSlide {
    title: String,
    body: Vec<String>,
}

#[cfg(feature = "pptx")]
fn build_slides(doc: &IntermediateDoc) -> Vec<PptxSlide> {
    let mut slides = Vec::new();
    let mut current_title = String::new();
    let mut current_body: Vec<String> = Vec::new();

    for block in &doc.blocks {
        match block {
            ContentBlock::Heading { text, .. } => {
                // Flush current slide if it has content.
                if !current_title.is_empty() || !current_body.is_empty() {
                    slides.push(PptxSlide {
                        title: std::mem::take(&mut current_title),
                        body: std::mem::take(&mut current_body),
                    });
                }
                current_title = text.clone();
            }
            ContentBlock::Paragraph { text, .. } => {
                current_body.push(text.clone());
            }
            ContentBlock::ListItem { text, .. } => {
                current_body.push(format!("\u{2022} {}", text));
            }
            ContentBlock::Table { table, .. } => {
                // Render table as text lines in the slide body.
                for row in &table.rows {
                    let cells: Vec<String> = row.cells.iter().map(|c| c.text.clone()).collect();
                    current_body.push(cells.join(" | "));
                }
            }
        }
    }

    // Flush the last slide.
    if !current_title.is_empty() || !current_body.is_empty() {
        slides.push(PptxSlide {
            title: current_title,
            body: current_body,
        });
    }

    // Ensure at least one slide.
    if slides.is_empty() {
        slides.push(PptxSlide {
            title: String::new(),
            body: Vec::new(),
        });
    }

    slides
}

#[cfg(feature = "pptx")]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(feature = "pptx")]
fn content_types_xml(slide_count: usize) -> String {
    let mut s = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
"#,
    );
    for i in 1..=slide_count {
        s.push_str(&format!(
            "  <Override PartName=\"/ppt/slides/slide{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>\n",
            i
        ));
    }
    s.push_str("</Types>\n");
    s
}

#[cfg(feature = "pptx")]
const RELS_DOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>
"#;

#[cfg(feature = "pptx")]
fn presentation_xml(slide_count: usize) -> String {
    let mut slide_list = String::new();
    for i in 1..=slide_count {
        slide_list.push_str(&format!(
            "    <p:sldId id=\"{}\" r:id=\"rId{}\"/>\n",
            255 + i,
            i + 2
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
                xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>
  </p:sldMasterIdLst>
  <p:sldIdLst>
{slide_list}  </p:sldIdLst>
  <p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>
"#
    )
}

#[cfg(feature = "pptx")]
fn presentation_rels_xml(slide_count: usize) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
"#,
    );
    for i in 1..=slide_count {
        rels.push_str(&format!(
            "  <Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{}.xml\"/>\n",
            i + 2, i
        ));
    }
    rels.push_str("</Relationships>\n");
    rels
}

#[cfg(feature = "pptx")]
fn slide_master_xml(_slide_count: usize) -> String {
    String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst>
    <p:sldLayoutId id="2147483649" r:id="rId1"/>
  </p:sldLayoutIdLst>
</p:sldMaster>
"#,
    )
}

#[cfg(feature = "pptx")]
const SLIDE_MASTER_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>
"#;

#[cfg(feature = "pptx")]
const SLIDE_LAYOUT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             type="blank">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
</p:sldLayout>
"#;

#[cfg(feature = "pptx")]
const SLIDE_LAYOUT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>
"#;

#[cfg(feature = "pptx")]
const THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
  <a:themeElements>
    <a:clrScheme name="Office">
      <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
      <a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F497D"/></a:dk2>
      <a:lt2><a:srgbClr val="EEECE1"/></a:lt2>
      <a:accent1><a:srgbClr val="4F81BD"/></a:accent1>
      <a:accent2><a:srgbClr val="C0504D"/></a:accent2>
      <a:accent3><a:srgbClr val="9BBB59"/></a:accent3>
      <a:accent4><a:srgbClr val="8064A2"/></a:accent4>
      <a:accent5><a:srgbClr val="4BACC6"/></a:accent5>
      <a:accent6><a:srgbClr val="F79646"/></a:accent6>
      <a:hlink><a:srgbClr val="0000FF"/></a:hlink>
      <a:folHlink><a:srgbClr val="800080"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Office">
      <a:majorFont><a:latin typeface="Calibri"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="25400"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="38100"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>
"#;

#[cfg(feature = "pptx")]
const SLIDE_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>
"#;

#[cfg(feature = "pptx")]
fn slide_xml(title: &str, body: &[String]) -> String {
    let mut body_paras = String::new();
    for line in body {
        body_paras.push_str(&format!(
            r#"              <a:p><a:r><a:rPr lang="en-US" sz="1800" dirty="0"/><a:t>{}</a:t></a:r></a:p>
"#,
            xml_escape(line)
        ));
    }
    if body_paras.is_empty() {
        body_paras = "              <a:p><a:endParaRPr lang=\"en-US\"/></a:p>\n".to_string();
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="title"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="457200" y="274638"/>
            <a:ext cx="8229600" cy="1143000"/>
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US" sz="3200" b="1" dirty="0"/><a:t>{title}</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Body"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="457200" y="1600200"/>
            <a:ext cx="8229600" cy="4525963"/>
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
{body_paras}        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
        title = xml_escape(title),
    )
}

// ---------------------------------------------------------------------------
// HTML generation
// ---------------------------------------------------------------------------

#[cfg(feature = "html")]
fn generate_html(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    Ok(paperjam_html::writer::generate_html_bytes(
        &doc.blocks,
        &doc.tables,
        &doc.metadata,
    ))
}

// ---------------------------------------------------------------------------
// EPUB generation
// ---------------------------------------------------------------------------

#[cfg(feature = "epub")]
fn generate_epub(doc: &IntermediateDoc) -> Result<Vec<u8>, ConvertError> {
    paperjam_epub::writer::generate_epub_bytes(&doc.blocks, &doc.metadata, &doc.bookmarks)
        .map_err(|e| ConvertError::Generation(e.to_string()))
}
