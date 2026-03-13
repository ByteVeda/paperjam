pub mod encoder;
pub mod filter;

use std::collections::BTreeMap;

use lopdf::{dictionary, Object, Stream};

use crate::document::Document;
use crate::error::{PdfError, Result};
use crate::page::Page;
use crate::text::operators::parse_content_stream;

/// A rectangular region to redact on a specific page.
#[derive(Debug, Clone)]
pub struct RedactRegion {
    /// 1-indexed page number.
    pub page: u32,
    /// Bounding rectangle in PDF coordinates: [x1, y1, x2, y2].
    pub rect: [f64; 4],
}

/// Options for redaction.
#[derive(Debug, Clone)]
pub struct RedactOptions {
    /// Regions to redact.
    pub regions: Vec<RedactRegion>,
    /// Optional fill color [r, g, b] (0.0–1.0) for overlay rectangles.
    /// If `None`, no overlay is drawn (text is just removed).
    pub fill_color: Option<[f64; 3]>,
}

/// A single item that was redacted.
#[derive(Debug, Clone)]
pub struct RedactedItem {
    /// Page number where the item was found.
    pub page: u32,
    /// The decoded text that was removed.
    pub text: String,
    /// Bounding box of the removed text: [x1, y1, x2, y2].
    pub rect: [f64; 4],
}

/// Result statistics from a redaction operation.
#[derive(Debug, Clone)]
pub struct RedactResult {
    /// Number of pages that were modified.
    pub pages_modified: u32,
    /// Total number of text items redacted.
    pub items_redacted: u32,
    /// Detailed list of redacted items.
    pub items: Vec<RedactedItem>,
}

/// Redact text within specified regions, removing it from the content stream.
///
/// Returns a new document with the redacted content and statistics.
pub fn redact(doc: &Document, options: &RedactOptions) -> Result<(Document, RedactResult)> {
    let mut inner = doc.inner().clone();
    let page_map = inner.get_pages();

    // Group regions by page
    let mut regions_by_page: BTreeMap<u32, Vec<[f64; 4]>> = BTreeMap::new();
    for region in &options.regions {
        regions_by_page
            .entry(region.page)
            .or_default()
            .push(region.rect);
    }

    let mut total_redacted = 0u32;
    let mut all_items = Vec::new();
    let mut pages_modified = 0u32;

    for (&page_num, page_regions) in &regions_by_page {
        let page_id = page_map.get(&page_num).copied().ok_or(PdfError::PageOutOfRange {
            page: page_num as usize,
            total: page_map.len(),
        })?;

        // Parse the page to get content bytes and fonts
        let page = Page::parse(&inner, page_num, page_id)?;
        let content_bytes = page.content_bytes();
        let fonts = page.fonts();

        // Parse content stream into operators
        let ops = parse_content_stream(content_bytes)?;

        // Filter out text operators that overlap redaction regions
        let (filtered_ops, redacted_items) = filter::filter_ops(&ops, page_regions, fonts);

        if !redacted_items.is_empty() {
            let redacted_count = redacted_items.len() as u32;

            // Encode filtered operators back to content stream bytes
            let mut new_content = encoder::encode_content_stream(&filtered_ops);

            // Add overlay rectangles if fill_color is specified
            if let Some(color) = options.fill_color {
                for rect in page_regions {
                    let w = rect[2] - rect[0];
                    let h = rect[3] - rect[1];
                    let overlay = format!(
                        "q {} {} {} rg {} {} {} {} re f Q\n",
                        format_f64(color[0]),
                        format_f64(color[1]),
                        format_f64(color[2]),
                        format_f64(rect[0]),
                        format_f64(rect[1]),
                        format_f64(w),
                        format_f64(h),
                    );
                    new_content.extend_from_slice(overlay.as_bytes());
                }
            }

            // Replace page content stream
            replace_page_content(&mut inner, page_id, &new_content)?;

            for (text, rect) in &redacted_items {
                all_items.push(RedactedItem {
                    page: page_num,
                    text: text.clone(),
                    rect: *rect,
                });
            }

            total_redacted += redacted_count;
            pages_modified += 1;
        }
    }

    let result_doc = Document::from_lopdf(inner)?;
    Ok((
        result_doc,
        RedactResult {
            pages_modified,
            items_redacted: total_redacted,
            items: all_items,
        },
    ))
}

/// Redact all occurrences of a text query across the document.
///
/// Finds text matching the query, builds redaction regions from their positions,
/// then performs true content-stream redaction.
///
/// When `use_regex` is true, `query` is treated as a regular expression pattern.
pub fn redact_text(
    doc: &Document,
    query: &str,
    case_sensitive: bool,
    use_regex: bool,
    fill_color: Option<[f64; 3]>,
) -> Result<(Document, RedactResult)> {
    let mut regions = Vec::new();

    let regex_pattern = if use_regex {
        Some(
            regex::RegexBuilder::new(query)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|e| PdfError::Redact(format!("Invalid regex: {}", e)))?,
        )
    } else {
        None
    };
    let query_lower = query.to_lowercase();

    // Search phase: parallelized (read-only per-page span scanning)
    let count = doc.page_count() as u32;
    let per_page_regions = crate::parallel::par_map_pages(count, |page_num| {
        let page = doc.page(page_num)?;
        let spans = page.text_spans()?;
        let mut page_regions = Vec::new();

        for span in &spans {
            let matches = if let Some(ref pattern) = regex_pattern {
                pattern.is_match(&span.text)
            } else if case_sensitive {
                span.text.contains(query)
            } else {
                span.text.to_lowercase().contains(&query_lower)
            };

            if matches {
                page_regions.push(RedactRegion {
                    page: page_num,
                    rect: [
                        span.x,
                        span.y - span.font_size * 0.3,
                        span.x + span.width,
                        span.y + span.font_size * 0.8,
                    ],
                });
            }
        }
        Ok(page_regions)
    });
    let collected = crate::parallel::collect_par_results(per_page_regions)?;
    for page_regions in collected {
        regions.extend(page_regions);
    }

    if regions.is_empty() {
        // No matches found — return a cloned document unchanged
        let inner = doc.inner().clone();
        let result_doc = Document::from_lopdf(inner)?;
        return Ok((
            result_doc,
            RedactResult {
                pages_modified: 0,
                items_redacted: 0,
                items: vec![],
            },
        ));
    }

    redact(doc, &RedactOptions { regions, fill_color })
}

/// Replace a page's /Contents with a single new content stream.
fn replace_page_content(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    content: &[u8],
) -> Result<()> {
    let new_stream = Stream::new(dictionary! {}, content.to_vec());
    let stream_id = doc.new_object_id();
    doc.objects.insert(stream_id, Object::Stream(new_stream));

    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Redact(format!("Cannot access page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Redact(format!("Page is not a dictionary: {}", e)))?;

    page_dict.set("Contents", Object::Reference(stream_id));
    Ok(())
}

/// Format a float for PDF content stream output.
fn format_f64(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.6}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
