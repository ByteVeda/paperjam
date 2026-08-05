use std::fmt;

use crate::error::HtmlError;

/// A parsed HTML document.
pub struct HtmlDocument {
    pub(crate) dom: scraper::Html,
    pub(crate) raw_bytes: Vec<u8>,
    /// Whether this was parsed as a fragment (no `<html>` wrapper).
    #[allow(dead_code)]
    pub(crate) is_fragment: bool,
}

impl fmt::Debug for HtmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlDocument")
            .field("is_fragment", &self.is_fragment)
            .field("raw_bytes_len", &self.raw_bytes.len())
            .finish()
    }
}

impl HtmlDocument {
    /// Parse an HTML document from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HtmlError> {
        let text = String::from_utf8(bytes.to_vec())
            .map_err(|e| HtmlError::Parse(format!("invalid UTF-8: {e}")))?;
        Self::from_string(text)
    }

    /// Parse from a string, consuming it.
    pub fn from_string(text: String) -> Result<Self, HtmlError> {
        let raw_bytes = text.as_bytes().to_vec();
        let is_fragment = !text.to_ascii_lowercase().contains("<html");
        let dom = if is_fragment {
            scraper::Html::parse_fragment(&text)
        } else {
            scraper::Html::parse_document(&text)
        };
        Ok(Self {
            dom,
            raw_bytes,
            is_fragment,
        })
    }

    /// Get a reference to the parsed DOM.
    pub fn dom(&self) -> &scraper::Html {
        &self.dom
    }
}
