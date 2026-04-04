//! PDF/UA (ISO 14289-1) accessibility validation.

use lopdf::Object;
use std::collections::HashSet;

use crate::document::Document;
use crate::error::Result;

use super::{PdfUaLevel, PdfUaReport, Severity, ValidationIssue};

/// Validate a PDF against PDF/UA accessibility requirements.
pub fn validate_pdf_ua(doc: &Document, level: PdfUaLevel) -> Result<PdfUaReport> {
    let inner = doc.inner();
    let page_map = inner.get_pages();
    let page_count = page_map.len();

    let mut issues = Vec::new();

    // Catalog-level checks
    check_mark_info(inner, &mut issues);
    check_language(inner, &mut issues);
    check_display_doc_title(inner, &mut issues);

    // Structure tree
    let struct_count = check_struct_tree(inner, &mut issues);

    // Page-level checks
    check_tab_order(inner, &page_map, &mut issues);
    check_annotation_accessibility(inner, &page_map, &mut issues);

    // Content stream checks
    check_tagged_content(inner, &page_map, &mut issues);

    let has_errors = issues.iter().any(|i| i.severity == Severity::Error);

    Ok(PdfUaReport {
        level,
        is_compliant: !has_errors,
        issues,
        pages_checked: page_count,
        structure_elements_checked: struct_count,
    })
}

/// Check that /MarkInfo exists with /Marked true.
fn check_mark_info(doc: &lopdf::Document, issues: &mut Vec<ValidationIssue>) {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "ua.catalog".to_string(),
                message: "Cannot access document catalog".to_string(),
                page: None,
            });
            return;
        }
    };

    let mark_info = match catalog.get(b"MarkInfo") {
        Ok(Object::Dictionary(d)) => d,
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d,
                Err(_) => {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        rule: "ua.mark_info.invalid".to_string(),
                        message: "/MarkInfo is not a dictionary".to_string(),
                        page: None,
                    });
                    return;
                }
            },
            Err(_) => return,
        },
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "ua.mark_info.missing".to_string(),
                message: "Document catalog has no /MarkInfo dictionary".to_string(),
                page: None,
            });
            return;
        }
    };

    match mark_info.get(b"Marked") {
        Ok(Object::Boolean(true)) => {}
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "ua.mark_info.not_marked".to_string(),
                message: "/MarkInfo/Marked is not true".to_string(),
                page: None,
            });
        }
    }
}

/// Check that /Lang is set in the catalog.
fn check_language(doc: &lopdf::Document, issues: &mut Vec<ValidationIssue>) {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => return,
    };

    match catalog.get(b"Lang") {
        Ok(Object::String(bytes, _)) if !bytes.is_empty() => {}
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "ua.lang.missing".to_string(),
                message: "Document catalog has no /Lang entry (language must be specified)"
                    .to_string(),
                page: None,
            });
        }
    }
}

/// Check /ViewerPreferences/DisplayDocTitle is true.
fn check_display_doc_title(doc: &lopdf::Document, issues: &mut Vec<ValidationIssue>) {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => return,
    };

    let vp = match catalog.get(b"ViewerPreferences") {
        Ok(Object::Dictionary(d)) => d,
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d,
                Err(_) => {
                    issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        rule: "ua.display_doc_title".to_string(),
                        message: "/ViewerPreferences is not a dictionary".to_string(),
                        page: None,
                    });
                    return;
                }
            },
            Err(_) => return,
        },
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                rule: "ua.display_doc_title".to_string(),
                message: "No /ViewerPreferences dictionary (DisplayDocTitle should be true)"
                    .to_string(),
                page: None,
            });
            return;
        }
    };

    match vp.get(b"DisplayDocTitle") {
        Ok(Object::Boolean(true)) => {}
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                rule: "ua.display_doc_title".to_string(),
                message: "/ViewerPreferences/DisplayDocTitle is not true".to_string(),
                page: None,
            });
        }
    }
}

/// Check /StructTreeRoot exists and validate its structure.
/// Returns the number of structure elements checked.
fn check_struct_tree(doc: &lopdf::Document, issues: &mut Vec<ValidationIssue>) -> usize {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let str_id = match catalog.get(b"StructTreeRoot") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "ua.struct_tree.missing".to_string(),
                message: "Document has no /StructTreeRoot (structure tree is required)".to_string(),
                page: None,
            });
            return 0;
        }
    };

    let str_dict = match doc.get_object(str_id) {
        Ok(obj) => match obj.as_dict() {
            Ok(d) => d.clone(),
            Err(_) => return 0,
        },
        Err(_) => return 0,
    };

    // Walk the structure tree
    let mut visited = HashSet::new();
    let mut elem_count = 0;
    let mut headings_found: Vec<u8> = Vec::new();

    walk_struct_tree(
        doc,
        &str_dict,
        &mut visited,
        &mut elem_count,
        &mut headings_found,
        issues,
    );

    // Check heading hierarchy
    check_heading_hierarchy(&headings_found, issues);

    elem_count
}

/// Recursively walk the structure tree.
fn walk_struct_tree(
    doc: &lopdf::Document,
    elem_dict: &lopdf::Dictionary,
    visited: &mut HashSet<lopdf::ObjectId>,
    count: &mut usize,
    headings: &mut Vec<u8>,
    issues: &mut Vec<ValidationIssue>,
) {
    *count += 1;

    // Get structure type
    let struct_type = elem_dict
        .get(b"S")
        .ok()
        .and_then(|o| match o {
            Object::Name(n) => Some(String::from_utf8_lossy(n).to_string()),
            _ => None,
        })
        .unwrap_or_default();

    // Check /Figure elements for /Alt text
    if struct_type == "Figure" && elem_dict.get(b"Alt").is_err() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            rule: "ua.figure.no_alt".to_string(),
            message: "Figure structure element has no /Alt text".to_string(),
            page: None,
        });
    }

    // Track heading levels
    if struct_type.starts_with('H') && struct_type.len() == 2 {
        if let Ok(level) = struct_type[1..].parse::<u8>() {
            if (1..=6).contains(&level) {
                headings.push(level);
            }
        }
    }

    // Recurse into /K (kids)
    let kids = match elem_dict.get(b"K") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => vec![Object::Reference(*id)],
        Ok(Object::Dictionary(_)) => vec![elem_dict.get(b"K").unwrap().clone()],
        _ => return,
    };

    for kid in &kids {
        let kid_dict = match kid {
            Object::Reference(id) => {
                if !visited.insert(*id) {
                    continue; // Cycle detection
                }
                match doc.get_object(*id) {
                    Ok(obj) => match obj.as_dict() {
                        Ok(d) => d.clone(),
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                }
            }
            Object::Dictionary(d) => d.clone(),
            _ => continue,
        };

        // Only recurse into structure elements (those with /S)
        if kid_dict.get(b"S").is_ok() {
            walk_struct_tree(doc, &kid_dict, visited, count, headings, issues);
        }
    }
}

/// Check that headings don't skip levels (e.g., H1 -> H3 without H2).
fn check_heading_hierarchy(headings: &[u8], issues: &mut Vec<ValidationIssue>) {
    if headings.is_empty() {
        return;
    }

    for window in headings.windows(2) {
        let prev = window[0];
        let next = window[1];
        if next > prev + 1 {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                rule: "ua.heading.skip".to_string(),
                message: format!("Heading level skipped: H{} followed by H{}", prev, next),
                page: None,
            });
        }
    }
}

/// Check that each page has /Tabs /S for structure-based tab order.
fn check_tab_order(
    doc: &lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
    issues: &mut Vec<ValidationIssue>,
) {
    for (&page_num, &page_id) in page_map {
        let page_dict = match doc.get_object(page_id).and_then(|o| o.as_dict()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        match page_dict.get(b"Tabs") {
            Ok(Object::Name(n)) if n == b"S" => {}
            _ => {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    rule: "ua.tab_order".to_string(),
                    message: format!(
                        "Page {} does not have /Tabs /S (structure-based tab order)",
                        page_num
                    ),
                    page: Some(page_num),
                });
            }
        }
    }
}

/// Check that annotations have /Contents or /Alt.
fn check_annotation_accessibility(
    doc: &lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
    issues: &mut Vec<ValidationIssue>,
) {
    for (&page_num, &page_id) in page_map {
        let page_dict = match doc.get_object(page_id).and_then(|o| o.as_dict()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let annots = match page_dict.get(b"Annots") {
            Ok(Object::Array(arr)) => arr.clone(),
            Ok(Object::Reference(id)) => match doc.get_object(*id) {
                Ok(Object::Array(arr)) => arr.clone(),
                _ => continue,
            },
            _ => continue,
        };

        for annot_ref in &annots {
            let annot_dict = match annot_ref {
                Object::Reference(id) => match doc.get_object(*id) {
                    Ok(obj) => match obj.as_dict() {
                        Ok(d) => d,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                },
                Object::Dictionary(d) => d,
                _ => continue,
            };

            // Skip Widget annotations (form fields)
            if let Ok(Object::Name(subtype)) = annot_dict.get(b"Subtype") {
                if subtype == b"Widget" {
                    continue;
                }
            }

            let has_contents = annot_dict.get(b"Contents").is_ok();
            let has_alt = annot_dict.get(b"Alt").is_ok();

            if !has_contents && !has_alt {
                let subtype = annot_dict
                    .get(b"Subtype")
                    .ok()
                    .and_then(|o| match o {
                        Object::Name(n) => Some(String::from_utf8_lossy(n).to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    rule: "ua.annotation.no_alt".to_string(),
                    message: format!(
                        "Annotation of type '{}' on page {} has no /Contents or /Alt",
                        subtype, page_num
                    ),
                    page: Some(page_num),
                });
            }
        }
    }
}

/// Check that content streams contain marked content operators (BDC/BMC/EMC).
fn check_tagged_content(
    doc: &lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
    issues: &mut Vec<ValidationIssue>,
) {
    for (&page_num, &page_id) in page_map {
        let page_dict = match doc.get_object(page_id).and_then(|o| o.as_dict()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Get content stream(s)
        let content_streams = match page_dict.get(b"Contents") {
            Ok(Object::Reference(id)) => match doc.get_object(*id) {
                Ok(Object::Stream(s)) => {
                    let mut stream = s.clone();
                    stream.decompress();
                    vec![stream.content.clone()]
                }
                Ok(Object::Array(arr)) => {
                    let mut streams = Vec::new();
                    for item in arr {
                        if let Object::Reference(ref_id) = item {
                            if let Ok(Object::Stream(s)) = doc.get_object(*ref_id) {
                                let mut stream = s.clone();
                                stream.decompress();
                                streams.push(stream.content.clone());
                            }
                        }
                    }
                    streams
                }
                _ => continue,
            },
            Ok(Object::Stream(s)) => {
                let mut stream = s.clone();
                stream.decompress();
                vec![stream.content.clone()]
            }
            _ => continue,
        };

        let has_marked_content = content_streams
            .iter()
            .any(|content| content_has_marked_content(content));

        if !has_marked_content {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                rule: "ua.tagged_content".to_string(),
                message: format!(
                    "Page {} has no marked content operators (BDC/BMC)",
                    page_num
                ),
                page: Some(page_num),
            });
        }
    }
}

/// Simple heuristic check for marked content operators in a content stream.
///
/// Scans for "BMC" or "BDC" as standalone operators (not inside strings).
fn content_has_marked_content(content: &[u8]) -> bool {
    let content_str = String::from_utf8_lossy(content);
    // Look for BMC or BDC as operators (preceded by whitespace or start)
    for line in content_str.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with("BMC") || trimmed.ends_with("BDC") {
            return true;
        }
    }
    false
}
