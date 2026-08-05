use scraper::node::Node;
use scraper::ElementRef;

use paperjam_model::text::{TextLine, TextSpan};

use crate::document::HtmlDocument;
use crate::error::HtmlError;

/// Tags whose text content should be skipped during extraction.
const SKIP_TAGS: &[&str] = &["script", "style", "noscript", "svg"];

/// Block-level tags that warrant a newline separator.
const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "tr",
    "blockquote",
    "pre",
    "article",
    "section",
    "header",
    "footer",
    "nav",
    "aside",
    "main",
    "figure",
    "figcaption",
    "details",
    "summary",
    "dt",
    "dd",
];

/// Extract plain text from a parsed HTML DOM.
pub fn extract_text_from_html(dom: &scraper::Html) -> String {
    let mut parts: Vec<String> = Vec::new();
    collect_text(dom.root_element(), &mut parts);

    // Join parts, collapse excessive blank lines.
    let joined = parts.join("");
    let mut result = String::with_capacity(joined.len());
    let mut blank_count = 0u32;
    for line in joined.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
        } else {
            blank_count = 0;
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result.trim().to_string()
}

fn collect_text(element: ElementRef, parts: &mut Vec<String>) {
    let tag = element.value().name().to_ascii_lowercase();
    if SKIP_TAGS.contains(&tag.as_str()) {
        return;
    }

    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let t = text.trim();
                if !t.is_empty() {
                    parts.push(t.to_string());
                    parts.push(" ".to_string());
                }
            }
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    let child_tag = child_el.value().name().to_ascii_lowercase();
                    if child_tag == "br" {
                        parts.push("\n".to_string());
                    } else if BLOCK_TAGS.contains(&child_tag.as_str()) {
                        parts.push("\n".to_string());
                        collect_text(child_el, parts);
                        parts.push("\n".to_string());
                    } else {
                        collect_text(child_el, parts);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Extract text lines from a parsed HTML DOM.
pub fn extract_text_lines_from_html(dom: &scraper::Html) -> Vec<TextLine> {
    let text = extract_text_from_html(dom);
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            let span = TextSpan {
                text: line.to_string(),
                x: 0.0,
                y: i as f64 * 14.0,
                width: line.len() as f64 * 7.0,
                font_size: 12.0,
                font_name: String::new(),
            };
            TextLine {
                spans: vec![span],
                bbox: (0.0, 0.0, 0.0, 0.0),
            }
        })
        .collect()
}

impl HtmlDocument {
    pub fn extract_text(&self) -> Result<String, HtmlError> {
        Ok(extract_text_from_html(&self.dom))
    }

    pub fn extract_text_lines(&self) -> Result<Vec<TextLine>, HtmlError> {
        Ok(extract_text_lines_from_html(&self.dom))
    }
}
