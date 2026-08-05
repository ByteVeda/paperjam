use docx_rs::{
    DocumentChild, Paragraph, ParagraphChild, Run, RunChild, StructuredDataTagChild,
    TableCellContent, TableChild, TableRowChild,
};
use paperjam_model::text::{TextLine, TextSpan};

use crate::document::DocxDocument;
use crate::error::DocxError;

impl DocxDocument {
    /// Extract all text from the document as a single string, paragraphs
    /// separated by newlines.
    pub fn extract_text(&self) -> Result<String, DocxError> {
        let mut paragraphs = Vec::new();
        for child in &self.inner.document.children {
            collect_text_from_document_child(child, &mut paragraphs);
        }
        Ok(paragraphs.join("\n"))
    }

    /// Extract text as positioned lines (one `TextLine` per paragraph).
    pub fn extract_text_lines(&self) -> Result<Vec<TextLine>, DocxError> {
        let mut lines = Vec::new();
        for child in &self.inner.document.children {
            collect_text_lines_from_document_child(child, &mut lines);
        }
        Ok(lines)
    }
}

// ---------------------------------------------------------------------------
// Text string helpers
// ---------------------------------------------------------------------------

fn collect_text_from_document_child(child: &DocumentChild, out: &mut Vec<String>) {
    match child {
        DocumentChild::Paragraph(p) => {
            let text = extract_paragraph_text(p);
            if !text.is_empty() {
                out.push(text);
            }
        }
        DocumentChild::Table(t) => {
            for row_child in &t.rows {
                let TableChild::TableRow(row) = row_child;
                for cell_child in &row.cells {
                    let TableRowChild::TableCell(cell) = cell_child;
                    for content in &cell.children {
                        match content {
                            TableCellContent::Paragraph(p) => {
                                let text = extract_paragraph_text(p);
                                if !text.is_empty() {
                                    out.push(text);
                                }
                            }
                            TableCellContent::Table(nested) => {
                                collect_text_from_table(nested, out);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        DocumentChild::StructuredDataTag(sdt) => {
            collect_text_from_sdt(sdt, out);
        }
        DocumentChild::TableOfContents(toc) => {
            for item in &toc.items {
                if !item.text.is_empty() {
                    out.push(item.text.clone());
                }
            }
        }
        _ => {}
    }
}

fn collect_text_from_table(table: &docx_rs::Table, out: &mut Vec<String>) {
    for row_child in &table.rows {
        let TableChild::TableRow(row) = row_child;
        for cell_child in &row.cells {
            let TableRowChild::TableCell(cell) = cell_child;
            for content in &cell.children {
                match content {
                    TableCellContent::Paragraph(p) => {
                        let text = extract_paragraph_text(p);
                        if !text.is_empty() {
                            out.push(text);
                        }
                    }
                    TableCellContent::Table(nested) => {
                        collect_text_from_table(nested, out);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_text_from_sdt(sdt: &docx_rs::StructuredDataTag, out: &mut Vec<String>) {
    for child in &sdt.children {
        match child {
            StructuredDataTagChild::Paragraph(p) => {
                let text = extract_paragraph_text(p);
                if !text.is_empty() {
                    out.push(text);
                }
            }
            StructuredDataTagChild::Table(t) => {
                collect_text_from_table(t, out);
            }
            StructuredDataTagChild::Run(r) => {
                let text = extract_run_text(r);
                if !text.is_empty() {
                    out.push(text);
                }
            }
            StructuredDataTagChild::StructuredDataTag(nested) => {
                collect_text_from_sdt(nested, out);
            }
            _ => {}
        }
    }
}

/// Extract concatenated text from a single paragraph.
pub(crate) fn extract_paragraph_text(para: &Paragraph) -> String {
    let mut parts = Vec::new();
    for child in &para.children {
        collect_paragraph_child_text(child, &mut parts);
    }
    parts.join("")
}

fn collect_paragraph_child_text(child: &ParagraphChild, parts: &mut Vec<String>) {
    match child {
        ParagraphChild::Run(run) => {
            let t = extract_run_text(run);
            if !t.is_empty() {
                parts.push(t);
            }
        }
        ParagraphChild::Hyperlink(hl) => {
            for hc in &hl.children {
                collect_paragraph_child_text(hc, parts);
            }
        }
        ParagraphChild::Insert(ins) => {
            for ic in &ins.children {
                if let docx_rs::InsertChild::Run(run) = ic {
                    let t = extract_run_text(run);
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn extract_run_text(run: &Run) -> String {
    let mut s = String::new();
    for child in &run.children {
        match child {
            RunChild::Text(t) => s.push_str(&t.text),
            RunChild::Tab(_) => s.push('\t'),
            RunChild::Break(_) => s.push('\n'),
            _ => {}
        }
    }
    s
}

// ---------------------------------------------------------------------------
// TextLine helpers
// ---------------------------------------------------------------------------

fn collect_text_lines_from_document_child(child: &DocumentChild, out: &mut Vec<TextLine>) {
    match child {
        DocumentChild::Paragraph(p) => {
            if let Some(line) = paragraph_to_text_line(p) {
                out.push(line);
            }
        }
        DocumentChild::Table(t) => {
            for row_child in &t.rows {
                let TableChild::TableRow(row) = row_child;
                for cell_child in &row.cells {
                    let TableRowChild::TableCell(cell) = cell_child;
                    for content in &cell.children {
                        match content {
                            TableCellContent::Paragraph(p) => {
                                if let Some(line) = paragraph_to_text_line(p) {
                                    out.push(line);
                                }
                            }
                            TableCellContent::Table(nested) => {
                                collect_text_lines_from_table(nested, out);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        DocumentChild::StructuredDataTag(sdt) => {
            collect_text_lines_from_sdt(sdt, out);
        }
        _ => {}
    }
}

fn collect_text_lines_from_table(table: &docx_rs::Table, out: &mut Vec<TextLine>) {
    for row_child in &table.rows {
        let TableChild::TableRow(row) = row_child;
        for cell_child in &row.cells {
            let TableRowChild::TableCell(cell) = cell_child;
            for content in &cell.children {
                match content {
                    TableCellContent::Paragraph(p) => {
                        if let Some(line) = paragraph_to_text_line(p) {
                            out.push(line);
                        }
                    }
                    TableCellContent::Table(nested) => {
                        collect_text_lines_from_table(nested, out);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_text_lines_from_sdt(sdt: &docx_rs::StructuredDataTag, out: &mut Vec<TextLine>) {
    for child in &sdt.children {
        match child {
            StructuredDataTagChild::Paragraph(p) => {
                if let Some(line) = paragraph_to_text_line(p) {
                    out.push(line);
                }
            }
            StructuredDataTagChild::Table(t) => {
                collect_text_lines_from_table(t, out);
            }
            StructuredDataTagChild::StructuredDataTag(nested) => {
                collect_text_lines_from_sdt(nested, out);
            }
            _ => {}
        }
    }
}

fn paragraph_to_text_line(para: &Paragraph) -> Option<TextLine> {
    let text = extract_paragraph_text(para);
    if text.is_empty() {
        return None;
    }
    // Flow-based document: no coordinate info available.
    let span = TextSpan {
        text,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        font_size: 0.0,
        font_name: String::new(),
    };
    Some(TextLine {
        spans: vec![span],
        bbox: (0.0, 0.0, 0.0, 0.0),
    })
}
