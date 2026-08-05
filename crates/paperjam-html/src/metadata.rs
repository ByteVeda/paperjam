use scraper::Selector;

use paperjam_model::metadata::Metadata;

use crate::document::HtmlDocument;
use crate::error::HtmlError;

/// Extract metadata from a parsed HTML DOM's `<head>` section.
pub fn extract_metadata_from_html(dom: &scraper::Html) -> Metadata {
    let mut metadata = Metadata::default();

    // <title>
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = dom.select(&sel).next() {
            let title = el.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                metadata.title = Some(title);
            }
        }
    }

    // <meta> tags
    if let Ok(sel) = Selector::parse("meta[name]") {
        for el in dom.select(&sel) {
            let name = el.value().attr("name").unwrap_or("").to_ascii_lowercase();
            let content = el.value().attr("content").unwrap_or("").trim().to_string();
            if content.is_empty() {
                continue;
            }
            match name.as_str() {
                "author" | "dc.creator" => metadata.author = Some(content),
                "description" | "dc.description" => metadata.subject = Some(content),
                "keywords" => metadata.keywords = Some(content),
                "generator" => metadata.creator = Some(content),
                "dcterms.created" | "dc.date" => metadata.creation_date = Some(content),
                "dcterms.modified" => metadata.modification_date = Some(content),
                _ => {}
            }
        }
    }

    metadata.page_count = 1;
    metadata
}

impl HtmlDocument {
    pub fn extract_metadata(&self) -> Result<Metadata, HtmlError> {
        Ok(extract_metadata_from_html(&self.dom))
    }
}
