//! PDF/A compliance validation.

use lopdf::Object;

use crate::document::Document;
use crate::error::Result;

use super::{get_resources, PdfALevel, Severity, ValidationIssue, ValidationReport};

/// Validate a PDF against PDF/A compliance rules.
pub fn validate_pdf_a(doc: &Document, level: PdfALevel) -> Result<ValidationReport> {
    let inner = doc.inner();
    let page_map = inner.get_pages();
    let page_count = page_map.len();

    let mut issues = Vec::new();
    check_encryption(inner, &mut issues);
    check_xmp_metadata(inner, level, &mut issues);
    let fonts_checked = check_font_embedding(inner, &page_map, &mut issues);
    check_output_intents(inner, &mut issues);

    if matches!(level, PdfALevel::A1b | PdfALevel::A1a) {
        check_transparency(inner, &page_map, &mut issues);
    }

    check_javascript(inner, &page_map, &mut issues);

    let has_errors = issues.iter().any(|i| i.severity == Severity::Error);

    Ok(ValidationReport {
        level,
        is_compliant: !has_errors,
        issues,
        fonts_checked,
        pages_checked: page_count,
    })
}

fn check_encryption(doc: &lopdf::Document, issues: &mut Vec<ValidationIssue>) {
    if doc.is_encrypted() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            rule: "encryption".to_string(),
            message: "PDF/A documents must not be encrypted".to_string(),
            page: None,
        });
    }
}

fn check_xmp_metadata(doc: &lopdf::Document, level: PdfALevel, issues: &mut Vec<ValidationIssue>) {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "xmp.catalog".to_string(),
                message: "Cannot access document catalog".to_string(),
                page: None,
            });
            return;
        }
    };

    let metadata_stream = match catalog.get(b"Metadata") {
        Ok(Object::Reference(id)) => doc.get_object(*id).ok(),
        Ok(obj) => Some(obj),
        Err(_) => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "xmp.missing".to_string(),
                message:
                    "Document catalog has no /Metadata entry; XMP metadata is required for PDF/A"
                        .to_string(),
                page: None,
            });
            return;
        }
    };

    let xmp_bytes = match metadata_stream {
        Some(Object::Stream(stream)) => {
            let mut s = stream.clone();
            s.decompress();
            s.content.clone()
        }
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "xmp.not_stream".to_string(),
                message: "XMP metadata is not a valid stream".to_string(),
                page: None,
            });
            return;
        }
    };

    let xmp_str = String::from_utf8_lossy(&xmp_bytes);

    let xmp_doc = match roxmltree::Document::parse(&xmp_str) {
        Ok(d) => d,
        Err(_) => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "xmp.parse".to_string(),
                message: "XMP metadata is not valid XML".to_string(),
                page: None,
            });
            return;
        }
    };

    let mut found_part = false;
    let mut found_conformance = false;

    for node in xmp_doc.descendants() {
        let tag = node.tag_name().name();
        if tag == "part" {
            found_part = true;
            if let Some(text) = node.text() {
                let expected_part = match level {
                    PdfALevel::A1b | PdfALevel::A1a => "1",
                    PdfALevel::A2b => "2",
                };
                if text.trim() != expected_part {
                    issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        rule: "xmp.part_mismatch".to_string(),
                        message: format!(
                            "XMP pdfaid:part is '{}', expected '{}'",
                            text.trim(),
                            expected_part
                        ),
                        page: None,
                    });
                }
            }
        }
        if tag == "conformance" {
            found_conformance = true;
        }
    }

    if !found_part {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            rule: "xmp.no_pdfaid_part".to_string(),
            message: "XMP metadata missing pdfaid:part element".to_string(),
            page: None,
        });
    }
    if !found_conformance {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            rule: "xmp.no_pdfaid_conformance".to_string(),
            message: "XMP metadata missing pdfaid:conformance element".to_string(),
            page: None,
        });
    }
}

fn check_font_embedding(
    doc: &lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
    issues: &mut Vec<ValidationIssue>,
) -> usize {
    let mut fonts_checked = 0;
    let mut checked_ids = std::collections::HashSet::new();

    for (&page_num, &page_id) in page_map {
        let page_obj = match doc.get_object(page_id) {
            Ok(obj) => obj,
            Err(_) => continue,
        };
        let page_dict = match page_obj.as_dict() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let resources = get_resources(doc, page_dict);
        let resources_dict = match &resources {
            Some(d) => d,
            None => continue,
        };

        let fonts = match resources_dict.get(b"Font") {
            Ok(Object::Dictionary(d)) => Some(d.clone()),
            Ok(Object::Reference(id)) => doc
                .get_object(*id)
                .ok()
                .and_then(|o| o.as_dict().ok())
                .cloned(),
            _ => None,
        };

        if let Some(font_dict) = fonts {
            for (_, font_ref) in font_dict.iter() {
                let font_obj = match font_ref {
                    Object::Reference(id) => {
                        if !checked_ids.insert(*id) {
                            continue;
                        }
                        match doc.get_object(*id) {
                            Ok(o) => o,
                            Err(_) => continue,
                        }
                    }
                    _ => continue,
                };

                let fd = match font_obj.as_dict() {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                fonts_checked += 1;

                let font_descriptor = match fd.get(b"FontDescriptor") {
                    Ok(Object::Reference(id)) => doc.get_object(*id).ok(),
                    Ok(obj) => Some(obj),
                    Err(_) => {
                        let subtype = fd.get(b"Subtype").ok().and_then(|o| {
                            if let Object::Name(n) = o {
                                Some(String::from_utf8_lossy(n).to_string())
                            } else {
                                None
                            }
                        });
                        let base_font = fd.get(b"BaseFont").ok().and_then(|o| {
                            if let Object::Name(n) = o {
                                Some(String::from_utf8_lossy(n).to_string())
                            } else {
                                None
                            }
                        });

                        if subtype.as_deref() == Some("Type1") {
                            if let Some(ref name) = base_font {
                                issues.push(ValidationIssue {
                                    severity: Severity::Error,
                                    rule: "font.not_embedded".to_string(),
                                    message: format!(
                                        "Font '{}' has no FontDescriptor (not embedded)",
                                        name
                                    ),
                                    page: Some(page_num),
                                });
                            }
                        }
                        continue;
                    }
                };

                if let Some(desc_obj) = font_descriptor {
                    if let Ok(desc_dict) = desc_obj.as_dict() {
                        let has_font_file = desc_dict.get(b"FontFile").is_ok()
                            || desc_dict.get(b"FontFile2").is_ok()
                            || desc_dict.get(b"FontFile3").is_ok();

                        if !has_font_file {
                            let name = fd
                                .get(b"BaseFont")
                                .ok()
                                .and_then(|o| {
                                    if let Object::Name(n) = o {
                                        Some(String::from_utf8_lossy(n).to_string())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_else(|| "unknown".to_string());

                            issues.push(ValidationIssue {
                                severity: Severity::Error,
                                rule: "font.not_embedded".to_string(),
                                message: format!(
                                    "Font '{}' is not embedded (no FontFile/FontFile2/FontFile3 in descriptor)",
                                    name
                                ),
                                page: Some(page_num),
                            });
                        }
                    }
                }
            }
        }
    }

    fonts_checked
}

fn check_output_intents(doc: &lopdf::Document, issues: &mut Vec<ValidationIssue>) {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => return,
    };

    match catalog.get(b"OutputIntents") {
        Ok(Object::Array(arr)) => {
            let has_pdfa_intent = arr.iter().any(|item| {
                let dict = match item {
                    Object::Reference(id) => {
                        doc.get_object(*id).ok().and_then(|o| o.as_dict().ok())
                    }
                    Object::Dictionary(d) => Some(d),
                    _ => None,
                };
                if let Some(d) = dict {
                    if let Ok(Object::Name(s)) = d.get(b"S") {
                        return s == b"GTS_PDFA1";
                    }
                }
                false
            });
            if !has_pdfa_intent {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    rule: "output_intent.no_pdfa".to_string(),
                    message: "No OutputIntent with /S /GTS_PDFA1 found".to_string(),
                    page: None,
                });
            }
        }
        Ok(Object::Reference(id)) => {
            if let Ok(Object::Array(arr)) = doc.get_object(*id) {
                let has_pdfa_intent = arr.iter().any(|item| {
                    let dict = match item {
                        Object::Reference(id) => {
                            doc.get_object(*id).ok().and_then(|o| o.as_dict().ok())
                        }
                        Object::Dictionary(d) => Some(d),
                        _ => None,
                    };
                    if let Some(d) = dict {
                        if let Ok(Object::Name(s)) = d.get(b"S") {
                            return s == b"GTS_PDFA1";
                        }
                    }
                    false
                });
                if !has_pdfa_intent {
                    issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        rule: "output_intent.no_pdfa".to_string(),
                        message: "No OutputIntent with /S /GTS_PDFA1 found".to_string(),
                        page: None,
                    });
                }
            }
        }
        _ => {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                rule: "output_intent.missing".to_string(),
                message: "No /OutputIntents found in catalog".to_string(),
                page: None,
            });
        }
    }
}

fn check_transparency(
    doc: &lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
    issues: &mut Vec<ValidationIssue>,
) {
    for (&page_num, &page_id) in page_map {
        let page_obj = match doc.get_object(page_id) {
            Ok(obj) => obj,
            Err(_) => continue,
        };
        let page_dict = match page_obj.as_dict() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if let Ok(group_obj) = page_dict.get(b"Group") {
            let group_dict = match group_obj {
                Object::Dictionary(d) => Some(d),
                Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_dict().ok()),
                _ => None,
            };
            if let Some(gd) = group_dict {
                if let Ok(Object::Name(s)) = gd.get(b"S") {
                    if s == b"Transparency" {
                        issues.push(ValidationIssue {
                            severity: Severity::Error,
                            rule: "transparency.group".to_string(),
                            message: "Page has transparency group (forbidden in PDF/A-1)"
                                .to_string(),
                            page: Some(page_num),
                        });
                    }
                }
            }
        }

        let resources = get_resources(doc, page_dict);
        if let Some(res) = &resources {
            if let Ok(gs_obj) = res.get(b"ExtGState") {
                let gs_dict = match gs_obj {
                    Object::Dictionary(d) => Some(d),
                    Object::Reference(id) => {
                        doc.get_object(*id).ok().and_then(|o| o.as_dict().ok())
                    }
                    _ => None,
                };
                if let Some(gsd) = gs_dict {
                    for (_, gs_ref) in gsd.iter() {
                        let gs = match gs_ref {
                            Object::Reference(id) => doc.get_object(*id).ok(),
                            _ => Some(gs_ref),
                        };
                        if let Some(gs_obj) = gs {
                            if let Ok(gs_d) = gs_obj.as_dict() {
                                if let Ok(Object::Name(bm)) = gs_d.get(b"BM") {
                                    if bm != b"Normal" && bm != b"Compatible" {
                                        issues.push(ValidationIssue {
                                            severity: Severity::Error,
                                            rule: "transparency.blend_mode".to_string(),
                                            message: format!(
                                                "ExtGState uses blend mode '{}' (forbidden in PDF/A-1)",
                                                String::from_utf8_lossy(bm)
                                            ),
                                            page: Some(page_num),
                                        });
                                    }
                                }
                                if let Ok(smask) = gs_d.get(b"SMask") {
                                    if !matches!(smask, Object::Name(n) if n == b"None") {
                                        issues.push(ValidationIssue {
                                            severity: Severity::Error,
                                            rule: "transparency.smask".to_string(),
                                            message: "ExtGState has /SMask (forbidden in PDF/A-1)"
                                                .to_string(),
                                            page: Some(page_num),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_javascript(
    doc: &lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
    issues: &mut Vec<ValidationIssue>,
) {
    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => return,
    };

    check_action_dict_for_js(doc, catalog, b"AA", None, issues);

    if let Ok(open_action) = catalog.get(b"OpenAction") {
        if is_js_action(doc, open_action) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "javascript.open_action".to_string(),
                message: "Document has JavaScript OpenAction (forbidden in PDF/A)".to_string(),
                page: None,
            });
        }
    }

    if let Ok(names_obj) = catalog.get(b"Names") {
        let names_dict = match names_obj {
            Object::Dictionary(d) => Some(d),
            Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_dict().ok()),
            _ => None,
        };
        if let Some(nd) = names_dict {
            if nd.get(b"JavaScript").is_ok() {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    rule: "javascript.names_tree".to_string(),
                    message: "Document has /Names/JavaScript tree (forbidden in PDF/A)".to_string(),
                    page: None,
                });
            }
        }
    }

    for (&page_num, &page_id) in page_map {
        let page_obj = match doc.get_object(page_id) {
            Ok(o) => o,
            Err(_) => continue,
        };
        let page_dict = match page_obj.as_dict() {
            Ok(d) => d,
            Err(_) => continue,
        };
        check_action_dict_for_js(doc, page_dict, b"AA", Some(page_num), issues);
    }
}

fn check_action_dict_for_js(
    doc: &lopdf::Document,
    dict: &lopdf::Dictionary,
    key: &[u8],
    page: Option<u32>,
    issues: &mut Vec<ValidationIssue>,
) {
    let aa_obj = match dict.get(key) {
        Ok(obj) => obj,
        Err(_) => return,
    };

    let aa_dict = match aa_obj {
        Object::Dictionary(d) => d,
        Object::Reference(id) => match doc.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d,
                Err(_) => return,
            },
            Err(_) => return,
        },
        _ => return,
    };

    for (_, action_ref) in aa_dict.iter() {
        if is_js_action(doc, action_ref) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                rule: "javascript.action".to_string(),
                message: "JavaScript action found (forbidden in PDF/A)".to_string(),
                page,
            });
        }
    }
}

fn is_js_action(doc: &lopdf::Document, obj: &Object) -> bool {
    let dict = match obj {
        Object::Dictionary(d) => d,
        Object::Reference(id) => match doc.get_object(*id) {
            Ok(o) => match o.as_dict() {
                Ok(d) => d,
                Err(_) => return false,
            },
            Err(_) => return false,
        },
        _ => return false,
    };

    if let Ok(Object::Name(s)) = dict.get(b"S") {
        return s == b"JavaScript";
    }
    false
}
