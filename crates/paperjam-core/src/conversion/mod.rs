//! PDF/A conversion: convert a document to PDF/A conformance.

mod actions;
mod output_intent;
mod transparency;
mod xmp;

use crate::document::Document;
use crate::error::{PdfError, Result};
use crate::validation::{PdfALevel, Severity};

pub use paperjam_model::conversion::*;

/// Convert a PDF document to PDF/A conformance.
///
/// Performs the following transformations:
/// 1. Removes encryption dictionary (if present)
/// 2. Writes/updates XMP metadata with PDF/A identification
/// 3. Adds sRGB OutputIntent with ICC profile (if missing)
/// 4. Strips JavaScript and prohibited actions
/// 5. Removes transparency (PDF/A-1 only)
///
/// Font embedding is NOT performed — documents with unembedded fonts will
/// have those reported as remaining issues.
pub fn convert_to_pdf_a(
    doc: &Document,
    options: &ConversionOptions,
) -> Result<(Document, ConversionResult)> {
    let mut inner = doc.inner().clone();
    let mut all_actions = Vec::new();

    // 1. Remove encryption
    let encryption_actions = remove_encryption(&mut inner);
    all_actions.extend(encryption_actions);

    // 2. XMP metadata
    let xmp_actions = xmp::ensure_xmp_metadata(&mut inner, options.level)?;
    all_actions.extend(xmp_actions);

    // 3. OutputIntent with ICC profile
    let oi_actions = output_intent::ensure_output_intent(&mut inner)?;
    all_actions.extend(oi_actions);

    // 4. Remove JS and actions (uses sanitize module, needs a Document wrapper)
    let temp_doc = Document::from_lopdf(inner)?;
    let (sanitized_doc, action_descriptions) = actions::remove_prohibited_actions(&temp_doc)?;
    inner = sanitized_doc.inner().clone();
    all_actions.extend(action_descriptions);

    // 5. Remove transparency (PDF/A-1 only)
    if matches!(options.level, PdfALevel::A1b | PdfALevel::A1a) {
        let page_map = inner.get_pages();
        let trans_actions = transparency::remove_transparency(&mut inner, &page_map);
        all_actions.extend(trans_actions);
    }

    // Build result document
    let result_doc = Document::from_lopdf(inner)?;

    // Validate the result to find remaining issues
    let report = crate::validation::validate_pdf_a(&result_doc, options.level)?;
    let remaining_issues = report.issues;

    // Check for unresolvable issues (like unembedded fonts)
    let has_errors = remaining_issues
        .iter()
        .any(|i| i.severity == Severity::Error);

    if has_errors && !options.force {
        return Err(PdfError::Conversion(format!(
            "PDF/A conversion has {} remaining error(s) that cannot be automatically fixed. \
                 Use force=true to proceed. Issues: {}",
            remaining_issues
                .iter()
                .filter(|i| i.severity == Severity::Error)
                .count(),
            remaining_issues
                .iter()
                .filter(|i| i.severity == Severity::Error)
                .map(|i| i.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
        )));
    }

    let actions_taken = all_actions
        .into_iter()
        .map(|desc| ConversionAction {
            category: categorize_action(&desc),
            description: desc,
            page: None,
        })
        .collect();

    Ok((
        result_doc,
        ConversionResult {
            level: options.level,
            success: !has_errors,
            actions_taken,
            remaining_issues,
        },
    ))
}

/// Remove encryption dictionary from the document.
fn remove_encryption(doc: &mut lopdf::Document) -> Vec<String> {
    let mut actions = Vec::new();

    if let Ok(encrypt_ref) = doc.trailer.get(b"Encrypt") {
        if let Ok(id) = encrypt_ref.as_reference() {
            doc.objects.remove(&id);
        }
        doc.trailer.remove(b"Encrypt");
        actions.push("Removed encryption dictionary".to_string());
    }

    actions
}

fn categorize_action(desc: &str) -> String {
    if desc.contains("XMP") || desc.contains("metadata") {
        "metadata".to_string()
    } else if desc.contains("OutputIntent") || desc.contains("ICC") {
        "color".to_string()
    } else if desc.contains("encryption") {
        "encryption".to_string()
    } else if desc.contains("transparency") || desc.contains("blend") || desc.contains("mask") {
        "transparency".to_string()
    } else if desc.contains("javascript") || desc.contains("action") || desc.contains("Removed") {
        "actions".to_string()
    } else {
        "other".to_string()
    }
}
