//! JavaScript and action removal for PDF/A conversion.
//!
//! Reuses logic from the sanitize module.

use crate::document::Document;
use crate::error::Result;
use crate::sanitize::{SanitizeOptions, SanitizedItem};

/// Remove JavaScript and prohibited actions for PDF/A compliance.
///
/// Returns the list of items that were removed.
pub fn remove_prohibited_actions(doc: &Document) -> Result<(Document, Vec<String>)> {
    let options = SanitizeOptions {
        remove_javascript: true,
        remove_embedded_files: false,
        remove_actions: true,
        remove_links: false,
    };

    let (sanitized, result) = crate::sanitize::sanitize(doc, &options)?;
    let mut actions = Vec::new();

    for item in &result.items {
        actions.push(format_action(item));
    }

    Ok((sanitized, actions))
}

fn format_action(item: &SanitizedItem) -> String {
    if let Some(page) = item.page {
        format!(
            "Removed {} on page {}: {}",
            item.category, page, item.description
        )
    } else {
        format!("Removed {}: {}", item.category, item.description)
    }
}
