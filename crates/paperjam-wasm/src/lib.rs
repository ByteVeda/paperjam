use std::sync::Arc;

use paperjam_core::document::Document;
use paperjam_core::markdown::MarkdownOptions;
use paperjam_core::structure::StructureOptions;
use paperjam_core::table::TableExtractionOptions;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A PDF document handle for use in JavaScript.
#[wasm_bindgen]
pub struct WasmDocument {
    inner: Arc<Document>,
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

#[wasm_bindgen]
impl WasmDocument {
    /// Open a PDF from bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<WasmDocument, JsValue> {
        let doc = Document::open_bytes(data).map_err(to_js_err)?;
        Ok(WasmDocument {
            inner: Arc::new(doc),
        })
    }

    /// Open a password-protected PDF from bytes.
    #[wasm_bindgen(js_name = "openWithPassword")]
    pub fn open_with_password(data: &[u8], password: &str) -> Result<WasmDocument, JsValue> {
        let doc = Document::open_bytes_with_password(data, password).map_err(to_js_err)?;
        Ok(WasmDocument {
            inner: Arc::new(doc),
        })
    }

    /// Number of pages in the document.
    #[wasm_bindgen(js_name = "pageCount")]
    pub fn page_count(&self) -> usize {
        self.inner.page_count()
    }

    /// Get page info (dimensions, rotation) as JSON.
    #[wasm_bindgen(js_name = "pageInfo")]
    pub fn page_info(&self, page_number: u32) -> Result<JsValue, JsValue> {
        let page = self.inner.page(page_number).map_err(to_js_err)?;
        let info = PageInfo {
            number: page.number,
            width: page.width,
            height: page.height,
            rotation: page.rotation,
        };
        serde_wasm_bindgen::to_value(&info).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Extract plain text from a page.
    #[wasm_bindgen(js_name = "extractText")]
    pub fn extract_text(&self, page_number: u32) -> Result<String, JsValue> {
        let page = self.inner.page(page_number).map_err(to_js_err)?;
        page.extract_text().map_err(to_js_err)
    }

    /// Extract text from all pages.
    #[wasm_bindgen(js_name = "extractAllText")]
    pub fn extract_all_text(&self) -> Result<String, JsValue> {
        let mut result = String::new();
        for i in 1..=self.inner.page_count() as u32 {
            let page = self.inner.page(i).map_err(to_js_err)?;
            let text = page.extract_text().map_err(to_js_err)?;
            if !result.is_empty() {
                result.push_str("\n\n---\n\n");
            }
            result.push_str(&text);
        }
        Ok(result)
    }

    /// Extract text lines with bounding boxes from a page (returns JSON).
    #[wasm_bindgen(js_name = "extractTextLines")]
    pub fn extract_text_lines(&self, page_number: u32) -> Result<JsValue, JsValue> {
        let page = self.inner.page(page_number).map_err(to_js_err)?;
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

    /// Extract tables from a page (returns JSON).
    #[wasm_bindgen(js_name = "extractTables")]
    pub fn extract_tables(&self, page_number: u32) -> Result<JsValue, JsValue> {
        let page = self.inner.page(page_number).map_err(to_js_err)?;
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
        let options = MarkdownOptions {
            include_page_numbers: include_page_numbers.unwrap_or(false),
            html_tables: html_tables.unwrap_or(false),
            structure_options: StructureOptions {
                layout_aware: layout_aware.unwrap_or(false),
                ..Default::default()
            },
            ..Default::default()
        };
        paperjam_core::markdown::document_to_markdown(&self.inner, &options).map_err(to_js_err)
    }

    /// Get document metadata as JSON.
    #[wasm_bindgen]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        let meta = self.inner.metadata().map_err(to_js_err)?;
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
        let opts = StructureOptions::default();
        let blocks = paperjam_core::structure::extract_document_structure(&self.inner, &opts)
            .map_err(to_js_err)?;
        let results: Vec<StructureBlock> = blocks
            .iter()
            .map(|b| {
                let level = match b {
                    paperjam_core::structure::ContentBlock::Heading { level, .. } => Some(*level),
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

    /// Search for text across all pages (returns JSON array of matches).
    #[wasm_bindgen(js_name = "searchText")]
    pub fn search_text(
        &self,
        query: &str,
        case_sensitive: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let case_sensitive = case_sensitive.unwrap_or(true);
        let mut matches = Vec::new();

        for i in 1..=self.inner.page_count() as u32 {
            let page = self.inner.page(i).map_err(to_js_err)?;
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

    /// Convert a single page to Markdown.
    #[wasm_bindgen(js_name = "pageToMarkdown")]
    pub fn page_to_markdown(&self, page_number: u32) -> Result<String, JsValue> {
        let page = self.inner.page(page_number).map_err(to_js_err)?;
        let options = MarkdownOptions::default();
        paperjam_core::markdown::page_to_markdown(&page, &options).map_err(to_js_err)
    }

    /// Save the document to bytes.
    #[wasm_bindgen(js_name = "saveBytes")]
    pub fn save_bytes(&self) -> Result<Vec<u8>, JsValue> {
        let mut inner = self.inner.inner().clone();
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
        let ranges: Vec<(u32, u32)> = serde_wasm_bindgen::from_value(ranges)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let docs = paperjam_core::manipulation::split(&self.inner, &ranges).map_err(to_js_err)?;
        let mut byte_arrays: Vec<Vec<u8>> = Vec::new();
        for doc in docs {
            let mut inner = doc.into_inner();
            let mut buf = Vec::new();
            inner
                .save_to(&mut buf)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            byte_arrays.push(buf);
        }
        serde_wasm_bindgen::to_value(&byte_arrays).map_err(|e| JsValue::from_str(&e.to_string()))
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
        let (doc, result) = sanitize(&self.inner, &options).map_err(to_js_err)?;
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
    pub fn redact_text(&self, query: &str, case_sensitive: bool) -> Result<JsValue, JsValue> {
        let (doc, result) =
            paperjam_core::redact::redact_text(&self.inner, query, case_sensitive, false, None)
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
    #[wasm_bindgen]
    pub fn encrypt(&self, user_password: &str, owner_password: &str) -> Result<Vec<u8>, JsValue> {
        use paperjam_core::encryption::{
            encrypt, EncryptionAlgorithm, EncryptionOptions, Permissions,
        };
        let options = EncryptionOptions {
            user_password: user_password.to_string(),
            owner_password: owner_password.to_string(),
            permissions: Permissions::default(),
            algorithm: EncryptionAlgorithm::default(),
        };
        encrypt(&self.inner, &options).map_err(to_js_err)
    }

    /// Analyze the layout of a single page (columns, headers, footers, regions).
    #[wasm_bindgen(js_name = "analyzeLayout")]
    pub fn analyze_layout(&self, page_number: u32) -> Result<JsValue, JsValue> {
        use paperjam_core::layout::{analyze_layout, LayoutOptions, RegionKind};
        let page = self.inner.page(page_number).map_err(to_js_err)?;
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
}
