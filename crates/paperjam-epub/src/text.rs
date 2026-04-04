use paperjam_model::text::TextLine;

use crate::document::EpubDocument;
use crate::error::EpubError;

impl EpubDocument {
    pub fn extract_text(&self) -> Result<String, EpubError> {
        let mut parts = Vec::new();
        for ch in &self.chapters {
            let text = paperjam_html::text::extract_text_from_html(ch.html.dom());
            if !text.is_empty() {
                if let Some(title) = &ch.title {
                    parts.push(format!("--- {} ---\n{}", title, text));
                } else {
                    parts.push(text);
                }
            }
        }
        Ok(parts.join("\n\n"))
    }

    pub fn extract_text_lines(&self) -> Result<Vec<TextLine>, EpubError> {
        let mut all_lines = Vec::new();
        for ch in &self.chapters {
            let lines = paperjam_html::text::extract_text_lines_from_html(ch.html.dom());
            all_lines.extend(lines);
        }
        Ok(all_lines)
    }
}
