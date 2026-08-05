//! WebAssembly bindings for paperjam, exposed via `wasm-bindgen`.
//!
//! Builds with `wasm-pack build --target web`. The generated JS + WASM
//! pair powers the interactive playground on the docs site.
//! Functionality is a subset of the native engine — rendering and
//! signatures are omitted on wasm, and compression is pure-Rust to avoid
//! `libz-sys` on `wasm32-unknown-unknown`.

use std::sync::Arc;

use paperjam_core::document::Document;
use paperjam_core::markdown::MarkdownOptions;
use paperjam_core::structure::StructureOptions;
use paperjam_core::table::TableExtractionOptions;
use paperjam_model::format::DocumentFormat;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Internal storage: PDF uses native paperjam-core, other formats use IntermediateDoc.
#[allow(clippy::large_enum_variant)]
enum DocumentInner {
    Pdf(Arc<Document>),
    Generic {
        format: DocumentFormat,
        intermediate: paperjam_convert::IntermediateDoc,
        raw_bytes: Vec<u8>,
    },
}

/// A document handle for use in JavaScript. Supports PDF and other formats.
#[wasm_bindgen]
pub struct WasmDocument {
    inner: DocumentInner,
}

#[derive(Serialize)]
struct TextLineResult {
    text: String,
    bbox: (f64, f64, f64, f64),
    spans: Vec<TextSpanResult>,
}

#[derive(Serialize)]
struct TextSpanResult {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    font_size: f64,
    font_name: String,
}

#[derive(Serialize)]
struct TableResult {
    rows: Vec<Vec<String>>,
    row_count: usize,
    col_count: usize,
    bbox: (f64, f64, f64, f64),
    strategy: String,
}

#[derive(Serialize)]
struct MetadataResult {
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    keywords: Option<String>,
    creator: Option<String>,
    producer: Option<String>,
    creation_date: Option<String>,
    modification_date: Option<String>,
    pdf_version: String,
    page_count: usize,
    is_encrypted: bool,
}

#[derive(Serialize)]
struct StructureBlock {
    block_type: String,
    text: String,
    page: u32,
    bbox: (f64, f64, f64, f64),
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<u8>,
}

#[derive(Serialize)]
struct SearchMatch {
    page: u32,
    line_number: usize,
    text: String,
    bbox: (f64, f64, f64, f64),
}

#[derive(Serialize)]
struct PageInfo {
    number: u32,
    width: f64,
    height: f64,
    rotation: u32,
}

fn to_js_err(e: paperjam_core::error::PdfError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

#[derive(Serialize)]
struct SanitizeResultJs {
    javascript_removed: usize,
    embedded_files_removed: usize,
    actions_removed: usize,
    links_removed: usize,
}

#[derive(Serialize)]
struct RedactResultJs {
    pages_modified: u32,
    items_redacted: u32,
    items: Vec<RedactedItemJs>,
}

#[derive(Serialize)]
struct RedactedItemJs {
    page: u32,
    text: String,
    rect: [f64; 4],
}

#[derive(Serialize)]
struct LayoutResultJs {
    num_columns: usize,
    regions: Vec<LayoutRegionJs>,
    has_header: bool,
    has_footer: bool,
}

#[derive(Serialize)]
struct LayoutRegionJs {
    region_type: String,
    bbox: (f64, f64, f64, f64),
    text: String,
}

#[derive(Serialize)]
struct ValidationIssueJs {
    severity: String,
    rule: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

#[derive(Serialize)]
struct ValidationReportJs {
    level: String,
    is_compliant: bool,
    fonts_checked: usize,
    pages_checked: usize,
    issues: Vec<ValidationIssueJs>,
}

#[derive(Serialize)]
struct PdfUaReportJs {
    level: String,
    is_compliant: bool,
    pages_checked: usize,
    structure_elements_checked: usize,
    issues: Vec<ValidationIssueJs>,
}

#[derive(Serialize)]
struct ConversionActionJs {
    category: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

#[derive(Serialize)]
struct ConversionResultJs {
    level: String,
    success: bool,
    actions_taken: Vec<ConversionActionJs>,
    remaining_issues: Vec<ValidationIssueJs>,
}

impl WasmDocument {
    /// Get the PDF document reference, or error if not PDF.
    fn pdf_inner(&self) -> Result<&Arc<Document>, JsValue> {
        match &self.inner {
            DocumentInner::Pdf(doc) => Ok(doc),
            DocumentInner::Generic { format, .. } => Err(JsValue::from_str(&format!(
                "Operation only supported for PDF documents (this is {})",
                format.display_name()
            ))),
        }
    }
}

#[wasm_bindgen]
impl WasmDocument {
    /// Open a document from bytes. Format is auto-detected.
    /// For PDF, uses native paperjam-core. For other formats, extracts to intermediate.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<WasmDocument, JsValue> {
        let format = paperjam_convert::detect_format_bytes(data);
        match format {
            DocumentFormat::Pdf => {
                let doc = Document::open_bytes(data).map_err(to_js_err)?;
                Ok(WasmDocument {
                    inner: DocumentInner::Pdf(Arc::new(doc)),
                })
            }
            DocumentFormat::Unknown => {
                // Fall back to PDF for backward compatibility.
                let doc = Document::open_bytes(data).map_err(to_js_err)?;
                Ok(WasmDocument {
                    inner: DocumentInner::Pdf(Arc::new(doc)),
                })
            }
            _ => {
                let intermediate = paperjam_convert::extract::extract(data, format)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                Ok(WasmDocument {
                    inner: DocumentInner::Generic {
                        format,
                        intermediate,
                        raw_bytes: data.to_vec(),
                    },
                })
            }
        }
    }

    /// Open a document with an explicit format hint.
    #[wasm_bindgen(js_name = "openWithFormat")]
    pub fn open_with_format(data: &[u8], format_str: &str) -> Result<WasmDocument, JsValue> {
        let format = DocumentFormat::from_extension(format_str);
        match format {
            DocumentFormat::Pdf => {
                let doc = Document::open_bytes(data).map_err(to_js_err)?;
                Ok(WasmDocument {
                    inner: DocumentInner::Pdf(Arc::new(doc)),
                })
            }
            _ => {
                let intermediate = paperjam_convert::extract::extract(data, format)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                Ok(WasmDocument {
                    inner: DocumentInner::Generic {
                        format,
                        intermediate,
                        raw_bytes: data.to_vec(),
                    },
                })
            }
        }
    }

    /// Open a password-protected PDF from bytes.
    #[wasm_bindgen(js_name = "openWithPassword")]
    pub fn open_with_password(data: &[u8], password: &str) -> Result<WasmDocument, JsValue> {
        let doc = Document::open_bytes_with_password(data, password).map_err(to_js_err)?;
        Ok(WasmDocument {
            inner: DocumentInner::Pdf(Arc::new(doc)),
        })
    }

    /// Get the document format as a string ("pdf", "docx", etc.).
    #[wasm_bindgen(js_name = "documentFormat")]
    pub fn document_format(&self) -> String {
        match &self.inner {
            DocumentInner::Pdf(_) => "pdf".to_string(),
            DocumentInner::Generic { format, .. } => format.extension().to_string(),
        }
    }

    /// Number of pages in the document.
    #[wasm_bindgen(js_name = "pageCount")]
    pub fn page_count(&self) -> usize {
        match &self.inner {
            DocumentInner::Pdf(doc) => doc.page_count(),
            DocumentInner::Generic { intermediate, .. } => {
                // Count distinct pages from content blocks.
                let max_page = intermediate
                    .blocks
                    .iter()
                    .map(|b| b.page())
                    .max()
                    .unwrap_or(1);
                max_page as usize
            }
        }
    }

    /// Get page info (dimensions, rotation) as JSON. PDF only.
    #[wasm_bindgen(js_name = "pageInfo")]
    pub fn page_info(&self, page_number: u32) -> Result<JsValue, JsValue> {
        let pdf = self.pdf_inner()?;
        let page = pdf.page(page_number).map_err(to_js_err)?;
        let info = PageInfo {
            number: page.number,
            width: page.width,
            height: page.height,
            rotation: page.rotation,
        };
        serde_wasm_bindgen::to_value(&info).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Extract plain text from a page. PDF only.
    #[wasm_bindgen(js_name = "extractText")]
    pub fn extract_text(&self, page_number: u32) -> Result<String, JsValue> {
        let pdf = self.pdf_inner()?;
        let page = pdf.page(page_number).map_err(to_js_err)?;
        page.extract_text().map_err(to_js_err)
    }

    /// Extract text from all pages / the entire document.
    #[wasm_bindgen(js_name = "extractAllText")]
    pub fn extract_all_text(&self) -> Result<String, JsValue> {
        match &self.inner {
            DocumentInner::Pdf(doc) => {
                let mut result = String::new();
                for i in 1..=doc.page_count() as u32 {
                    let page = doc.page(i).map_err(to_js_err)?;
                    let text = page.extract_text().map_err(to_js_err)?;
                    if !result.is_empty() {
                        result.push_str("\n\n---\n\n");
                    }
                    result.push_str(&text);
                }
                Ok(result)
            }
            DocumentInner::Generic { intermediate, .. } => Ok(intermediate
                .blocks
                .iter()
                .map(|b| b.text().to_string())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n")),
        }
    }

    /// Extract text lines with bounding boxes from a page (returns JSON). PDF only.
    #[wasm_bindgen(js_name = "extractTextLines")]
    pub fn extract_text_lines(&self, page_number: u32) -> Result<JsValue, JsValue> {
        let pdf = self.pdf_inner()?;
        let page = pdf.page(page_number).map_err(to_js_err)?;
        let lines = page.text_lines().map_err(to_js_err)?;
        let results: Vec<TextLineResult> = lines
            .iter()
            .map(|line| TextLineResult {
                text: line.text(),
                bbox: line.bbox,
                spans: line
                    .spans
                    .iter()
                    .map(|s| TextSpanResult {
                        text: s.text.clone(),
                        x: s.x,
                        y: s.y,
                        width: s.width,
                        font_size: s.font_size,
                        font_name: s.font_name.clone(),
                    })
                    .collect(),
            })
            .collect();
        serde_wasm_bindgen::to_value(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Extract tables from a page (returns JSON). PDF only for per-page; use extractAllTables for other formats.
    #[wasm_bindgen(js_name = "extractTables")]
    pub fn extract_tables(&self, page_number: u32) -> Result<JsValue, JsValue> {
        let pdf = self.pdf_inner()?;
        let page = pdf.page(page_number).map_err(to_js_err)?;
        let opts = TableExtractionOptions::default();
        let tables = page.extract_tables(&opts).map_err(to_js_err)?;
        let results: Vec<TableResult> = tables
            .iter()
            .map(|t| TableResult {
                rows: t.to_vec(),
                row_count: t.row_count(),
                col_count: t.col_count,
                bbox: t.bbox,
                strategy: format!("{:?}", t.strategy),
            })
            .collect();
        serde_wasm_bindgen::to_value(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert the entire document to Markdown.
    #[wasm_bindgen(js_name = "toMarkdown")]
    pub fn to_markdown(
        &self,
        layout_aware: Option<bool>,
        include_page_numbers: Option<bool>,
        html_tables: Option<bool>,
    ) -> Result<String, JsValue> {
        match &self.inner {
            DocumentInner::Pdf(doc) => {
                let options = MarkdownOptions {
                    include_page_numbers: include_page_numbers.unwrap_or(false),
                    html_tables: html_tables.unwrap_or(false),
                    structure_options: StructureOptions {
                        layout_aware: layout_aware.unwrap_or(false),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                paperjam_core::markdown::document_to_markdown(doc, &options).map_err(to_js_err)
            }
            DocumentInner::Generic { intermediate, .. } => {
                // Generate markdown from intermediate blocks.
                let md_bytes =
                    paperjam_convert::generate::generate(intermediate, DocumentFormat::Markdown)
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                String::from_utf8(md_bytes).map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }

    /// Get document metadata as JSON.
    #[wasm_bindgen]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        let meta = match &self.inner {
            DocumentInner::Pdf(doc) => {
                let m = doc.metadata().map_err(to_js_err)?;
                (*m).clone()
            }
            DocumentInner::Generic { intermediate, .. } => intermediate.metadata.clone(),
        };
        let result = MetadataResult {
            title: meta.title.clone(),
            author: meta.author.clone(),
            subject: meta.subject.clone(),
            keywords: meta.keywords.clone(),
            creator: meta.creator.clone(),
            producer: meta.producer.clone(),
            creation_date: meta.creation_date.clone(),
            modification_date: meta.modification_date.clone(),
            pdf_version: meta.pdf_version.clone(),
            page_count: meta.page_count,
            is_encrypted: meta.is_encrypted,
        };
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Extract document structure (headings, paragraphs, lists) as JSON.
    #[wasm_bindgen(js_name = "extractStructure")]
    pub fn extract_structure(&self) -> Result<JsValue, JsValue> {
        let blocks = match &self.inner {
            DocumentInner::Pdf(doc) => {
                let opts = StructureOptions::default();
                paperjam_core::structure::extract_document_structure(doc, &opts)
                    .map_err(to_js_err)?
            }
            DocumentInner::Generic { intermediate, .. } => intermediate.blocks.clone(),
        };
        let results: Vec<StructureBlock> = blocks
            .iter()
            .map(|b| {
                let level = match b {
                    paperjam_model::structure::ContentBlock::Heading { level, .. } => Some(*level),
                    _ => None,
                };
                StructureBlock {
                    block_type: b.block_type().to_string(),
                    text: b.text().to_string(),
                    page: b.page(),
                    bbox: b.bbox(),
                    level,
                }
            })
            .collect();
        serde_wasm_bindgen::to_value(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Search for text across all pages (returns JSON array of matches). PDF only.
    #[wasm_bindgen(js_name = "searchText")]
    pub fn search_text(
        &self,
        query: &str,
        case_sensitive: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let pdf = self.pdf_inner()?;
        let case_sensitive = case_sensitive.unwrap_or(true);
        let mut matches = Vec::new();

        for i in 1..=pdf.page_count() as u32 {
            let page = pdf.page(i).map_err(to_js_err)?;
            let lines = page.text_lines().map_err(to_js_err)?;

            for (line_idx, line) in lines.iter().enumerate() {
                let line_text = line.text();
                let found = if case_sensitive {
                    line_text.contains(query)
                } else {
                    line_text.to_lowercase().contains(&query.to_lowercase())
                };
                if found {
                    matches.push(SearchMatch {
                        page: i,
                        line_number: line_idx + 1,
                        text: line_text,
                        bbox: line.bbox,
                    });
                }
            }
        }

        serde_wasm_bindgen::to_value(&matches).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert a single page to Markdown. PDF only.
    #[wasm_bindgen(js_name = "pageToMarkdown")]
    pub fn page_to_markdown(&self, page_number: u32) -> Result<String, JsValue> {
        let pdf = self.pdf_inner()?;
        let page = pdf.page(page_number).map_err(to_js_err)?;
        let options = MarkdownOptions::default();
        paperjam_core::markdown::page_to_markdown(&page, &options).map_err(to_js_err)
    }

    /// Save the document to bytes.
    #[wasm_bindgen(js_name = "saveBytes")]
    pub fn save_bytes(&self) -> Result<Vec<u8>, JsValue> {
        match &self.inner {
            DocumentInner::Generic { raw_bytes, .. } => return Ok(raw_bytes.clone()),
            DocumentInner::Pdf(_) => {}
        }
        let pdf = self.pdf_inner()?;
        let mut inner = pdf.inner().clone();
        let mut buf = Vec::new();
        inner
            .save_to(&mut buf)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(buf)
    }

    /// Split the document into multiple documents by page ranges.
    /// `ranges` should be an array of [start, end] tuples (1-indexed, inclusive).
    #[wasm_bindgen]
    pub fn split(&self, ranges: JsValue) -> Result<JsValue, JsValue> {
        let pdf = self.pdf_inner()?;
        let ranges: Vec<(u32, u32)> = serde_wasm_bindgen::from_value(ranges)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let docs = paperjam_core::manipulation::split(pdf, &ranges).map_err(to_js_err)?;
        let result = js_sys::Array::new();
        for doc in docs {
            let mut inner = doc.into_inner();
            let mut buf = Vec::new();
            inner
                .save_to(&mut buf)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            result.push(&js_sys::Uint8Array::from(buf.as_slice()));
        }
        Ok(result.into())
    }

    /// Sanitize the document by removing potentially dangerous content.
    #[wasm_bindgen]
    pub fn sanitize(
        &self,
        remove_js: bool,
        remove_files: bool,
        remove_actions: bool,
        remove_links: bool,
    ) -> Result<JsValue, JsValue> {
        use paperjam_core::sanitize::{sanitize, SanitizeOptions};
        let options = SanitizeOptions {
            remove_javascript: remove_js,
            remove_embedded_files: remove_files,
            remove_actions,
            remove_links,
        };
        let pdf = self.pdf_inner()?;
        let (doc, result) = sanitize(pdf, &options).map_err(to_js_err)?;
        let mut inner = doc.into_inner();
        let mut buf = Vec::new();
        inner
            .save_to(&mut buf)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let js_result = SanitizeResultJs {
            javascript_removed: result.javascript_removed,
            embedded_files_removed: result.embedded_files_removed,
            actions_removed: result.actions_removed,
            links_removed: result.links_removed,
        };
        #[derive(Serialize)]
        struct SanitizeResponse {
            #[serde(with = "serde_bytes")]
            doc_bytes: Vec<u8>,
            result: SanitizeResultJs,
        }
        serde_wasm_bindgen::to_value(&SanitizeResponse {
            doc_bytes: buf,
            result: js_result,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Redact all occurrences of a text query across the document.
    #[wasm_bindgen(js_name = "redactText")]
    pub fn redact_text(
        &self,
        query: &str,
        case_sensitive: bool,
        fill_color: Option<Vec<f64>>,
    ) -> Result<JsValue, JsValue> {
        let color = fill_color.and_then(|c| {
            if c.len() == 3 {
                Some([c[0], c[1], c[2]])
            } else {
                None
            }
        });
        let pdf = self.pdf_inner()?;
        let (doc, result) =
            paperjam_core::redact::redact_text(pdf, query, case_sensitive, false, color)
                .map_err(to_js_err)?;
        let mut inner = doc.into_inner();
        let mut buf = Vec::new();
        inner
            .save_to(&mut buf)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let js_result = RedactResultJs {
            pages_modified: result.pages_modified,
            items_redacted: result.items_redacted,
            items: result
                .items
                .iter()
                .map(|item| RedactedItemJs {
                    page: item.page,
                    text: item.text.clone(),
                    rect: item.rect,
                })
                .collect(),
        };
        #[derive(Serialize)]
        struct RedactResponse {
            #[serde(with = "serde_bytes")]
            doc_bytes: Vec<u8>,
            result: RedactResultJs,
        }
        serde_wasm_bindgen::to_value(&RedactResponse {
            doc_bytes: buf,
            result: js_result,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Encrypt the document with user and owner passwords.
    /// `algorithm` can be "aes128" (default), "aes256", or "rc4".
    #[wasm_bindgen]
    pub fn encrypt(
        &self,
        user_password: &str,
        owner_password: Option<String>,
        algorithm: Option<String>,
    ) -> Result<Vec<u8>, JsValue> {
        use paperjam_core::encryption::{
            encrypt, EncryptionAlgorithm, EncryptionOptions, Permissions,
        };
        let algo = match algorithm.as_deref() {
            None | Some("aes128") | Some("AES-128") => EncryptionAlgorithm::Aes128,
            Some("aes256") | Some("AES-256") => EncryptionAlgorithm::Aes256,
            Some("rc4") | Some("RC4") => EncryptionAlgorithm::Rc4,
            Some(other) => {
                return Err(JsValue::from_str(&format!(
                    "Unknown algorithm: '{}'. Use 'aes128', 'aes256', or 'rc4'.",
                    other
                )))
            }
        };
        let options = EncryptionOptions {
            user_password: user_password.to_string(),
            owner_password: owner_password.unwrap_or_default(),
            permissions: Permissions::default(),
            algorithm: algo,
        };
        let pdf = self.pdf_inner()?;
        encrypt(pdf, &options).map_err(to_js_err)
    }

    /// Analyze the layout of a single page (columns, headers, footers, regions).
    #[wasm_bindgen(js_name = "analyzeLayout")]
    pub fn analyze_layout(&self, page_number: u32) -> Result<JsValue, JsValue> {
        use paperjam_core::layout::{analyze_layout, LayoutOptions, RegionKind};
        let pdf = self.pdf_inner()?;
        let page = pdf.page(page_number).map_err(to_js_err)?;
        let options = LayoutOptions::default();
        let layout = analyze_layout(&page, &options).map_err(to_js_err)?;
        let mut has_header = false;
        let mut has_footer = false;
        let regions: Vec<LayoutRegionJs> = layout
            .regions
            .iter()
            .map(|r| {
                let region_type = match &r.kind {
                    RegionKind::Header => {
                        has_header = true;
                        "header".to_string()
                    }
                    RegionKind::Footer => {
                        has_footer = true;
                        "footer".to_string()
                    }
                    RegionKind::BodyColumn { index } => format!("body_column_{}", index),
                    RegionKind::FullWidth => "full_width".to_string(),
                };
                let text = r
                    .lines
                    .iter()
                    .map(|l| l.text())
                    .collect::<Vec<_>>()
                    .join("\n");
                LayoutRegionJs {
                    region_type,
                    bbox: r.bbox,
                    text,
                }
            })
            .collect();
        let result = LayoutResultJs {
            num_columns: layout.column_count,
            regions,
            has_header,
            has_footer,
        };
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Validate PDF/A compliance. Returns JSON with level, is_compliant, issues, etc.
    #[wasm_bindgen(js_name = "validatePdfA")]
    pub fn validate_pdf_a(&self, level: Option<String>) -> Result<JsValue, JsValue> {
        let pdf_a_level =
            paperjam_core::validation::PdfALevel::from_str(level.as_deref().unwrap_or("1b"));
        let pdf = self.pdf_inner()?;
        let report =
            paperjam_core::validation::validate_pdf_a(pdf, pdf_a_level).map_err(to_js_err)?;
        let result = ValidationReportJs {
            level: report.level.as_str().to_string(),
            is_compliant: report.is_compliant,
            fonts_checked: report.fonts_checked,
            pages_checked: report.pages_checked,
            issues: report
                .issues
                .iter()
                .map(|i| ValidationIssueJs {
                    severity: i.severity.as_str().to_string(),
                    rule: i.rule.clone(),
                    message: i.message.clone(),
                    page: i.page,
                })
                .collect(),
        };
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Validate PDF/UA (accessibility) compliance. Returns JSON report.
    #[wasm_bindgen(js_name = "validatePdfUa")]
    pub fn validate_pdf_ua(&self, level: Option<String>) -> Result<JsValue, JsValue> {
        let pdf_ua_level =
            paperjam_core::validation::PdfUaLevel::from_str(level.as_deref().unwrap_or("1"));
        let pdf = self.pdf_inner()?;
        let report =
            paperjam_core::validation::validate_pdf_ua(pdf, pdf_ua_level).map_err(to_js_err)?;
        let result = PdfUaReportJs {
            level: report.level.as_str().to_string(),
            is_compliant: report.is_compliant,
            pages_checked: report.pages_checked,
            structure_elements_checked: report.structure_elements_checked,
            issues: report
                .issues
                .iter()
                .map(|i| ValidationIssueJs {
                    severity: i.severity.as_str().to_string(),
                    rule: i.rule.clone(),
                    message: i.message.clone(),
                    page: i.page,
                })
                .collect(),
        };
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert the document to PDF/A conformance. Returns { doc_bytes, result }.
    #[wasm_bindgen(js_name = "convertToPdfA")]
    pub fn convert_to_pdf_a(
        &self,
        level: Option<String>,
        force: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let pdf_a_level =
            paperjam_core::validation::PdfALevel::from_str(level.as_deref().unwrap_or("1b"));
        let options = paperjam_core::conversion::ConversionOptions {
            level: pdf_a_level,
            force: force.unwrap_or(false),
        };
        let pdf = self.pdf_inner()?;
        let (doc, result) =
            paperjam_core::conversion::convert_to_pdf_a(pdf, &options).map_err(to_js_err)?;
        let mut inner = doc.into_inner();
        let mut buf = Vec::new();
        inner
            .save_to(&mut buf)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let js_result = ConversionResultJs {
            level: result.level.as_str().to_string(),
            success: result.success,
            actions_taken: result
                .actions_taken
                .iter()
                .map(|a| ConversionActionJs {
                    category: a.category.clone(),
                    description: a.description.clone(),
                    page: a.page,
                })
                .collect(),
            remaining_issues: result
                .remaining_issues
                .iter()
                .map(|i| ValidationIssueJs {
                    severity: i.severity.as_str().to_string(),
                    rule: i.rule.clone(),
                    message: i.message.clone(),
                    page: i.page,
                })
                .collect(),
        };

        #[derive(Serialize)]
        struct ConvertResponse {
            #[serde(with = "serde_bytes")]
            doc_bytes: Vec<u8>,
            result: ConversionResultJs,
        }
        serde_wasm_bindgen::to_value(&ConvertResponse {
            doc_bytes: buf,
            result: js_result,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    /// Convert this document to another format. Returns the output bytes.
    #[wasm_bindgen(js_name = "convertTo")]
    pub fn convert_to(&self, target_format: &str) -> Result<Vec<u8>, JsValue> {
        let target = DocumentFormat::from_extension(target_format);
        if target == DocumentFormat::Unknown {
            return Err(JsValue::from_str(&format!(
                "Unknown target format: '{}'",
                target_format
            )));
        }

        match &self.inner {
            DocumentInner::Pdf(doc) => {
                // Extract to intermediate first.
                let mut buf = Vec::new();
                let mut inner = doc.inner().clone();
                inner
                    .save_to(&mut buf)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                paperjam_convert::convert_bytes(&buf, DocumentFormat::Pdf, target)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            DocumentInner::Generic { intermediate, .. } => {
                paperjam_convert::generate::generate(intermediate, target)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }
}

/// Convert bytes from one format to another.
#[wasm_bindgen(js_name = "convertDocument")]
pub fn convert_document(
    data: &[u8],
    from_format: &str,
    to_format: &str,
) -> Result<Vec<u8>, JsValue> {
    let from = DocumentFormat::from_extension(from_format);
    let to = DocumentFormat::from_extension(to_format);
    paperjam_convert::convert_bytes(data, from, to).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Merge multiple PDFs (given as byte arrays) into one. Returns the merged PDF bytes.
#[wasm_bindgen(js_name = "mergePdfs")]
pub fn merge_pdfs(pdf_arrays: JsValue) -> Result<Vec<u8>, JsValue> {
    let arrays: Vec<Vec<u8>> = serde_wasm_bindgen::from_value(pdf_arrays)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let docs: paperjam_core::error::Result<Vec<Document>> = arrays
        .iter()
        .map(|data| Document::open_bytes(data))
        .collect();
    let docs = docs.map_err(to_js_err)?;
    let options = paperjam_core::manipulation::MergeOptions {
        deduplicate_resources: false,
    };
    let merged = paperjam_core::manipulation::merge(docs, &options).map_err(to_js_err)?;
    let mut inner = merged.into_inner();
    let mut buf = Vec::new();
    inner
        .save_to(&mut buf)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(buf)
}
