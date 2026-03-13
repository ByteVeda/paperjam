use std::collections::BTreeMap;

use lopdf::{dictionary, Object, ObjectId};

use crate::error::{PdfError, Result};

/// Where a link annotation points to.
#[derive(Debug, Clone)]
pub enum LinkDestination {
    /// External URI (e.g. "https://example.com").
    Uri(String),
    /// Go to a specific page within the document.
    GoTo { page: u32 },
    /// A named destination string.
    Named(String),
}

/// Type of PDF annotation.
#[derive(Debug, Clone)]
pub enum AnnotationType {
    Text,
    Link,
    FreeText,
    Highlight,
    Underline,
    StrikeOut,
    Square,
    Circle,
    Line,
    Stamp,
    Unknown(String),
}

impl AnnotationType {
    fn from_name(name: &[u8]) -> Self {
        match name {
            b"Text" => Self::Text,
            b"Link" => Self::Link,
            b"FreeText" => Self::FreeText,
            b"Highlight" => Self::Highlight,
            b"Underline" => Self::Underline,
            b"StrikeOut" => Self::StrikeOut,
            b"Square" => Self::Square,
            b"Circle" => Self::Circle,
            b"Line" => Self::Line,
            b"Stamp" => Self::Stamp,
            other => Self::Unknown(String::from_utf8_lossy(other).to_string()),
        }
    }

    fn to_name(&self) -> &[u8] {
        match self {
            Self::Text => b"Text",
            Self::Link => b"Link",
            Self::FreeText => b"FreeText",
            Self::Highlight => b"Highlight",
            Self::Underline => b"Underline",
            Self::StrikeOut => b"StrikeOut",
            Self::Square => b"Square",
            Self::Circle => b"Circle",
            Self::Line => b"Line",
            Self::Stamp => b"Stamp",
            Self::Unknown(s) => s.as_bytes(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Link => "link",
            Self::FreeText => "free_text",
            Self::Highlight => "highlight",
            Self::Underline => "underline",
            Self::StrikeOut => "strike_out",
            Self::Square => "square",
            Self::Circle => "circle",
            Self::Line => "line",
            Self::Stamp => "stamp",
            Self::Unknown(s) => s.as_str(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "text" => Self::Text,
            "link" => Self::Link,
            "free_text" => Self::FreeText,
            "highlight" => Self::Highlight,
            "underline" => Self::Underline,
            "strike_out" => Self::StrikeOut,
            "square" => Self::Square,
            "circle" => Self::Circle,
            "line" => Self::Line,
            "stamp" => Self::Stamp,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// A parsed PDF annotation.
#[derive(Debug, Clone)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub rect: [f64; 4],
    pub contents: Option<String>,
    pub author: Option<String>,
    pub color: Option<[f64; 3]>,
    pub creation_date: Option<String>,
    pub opacity: Option<f64>,
    pub url: Option<String>,
    pub destination: Option<LinkDestination>,
}

/// Options for adding a new annotation.
pub struct AddAnnotationOptions {
    pub annotation_type: AnnotationType,
    pub rect: [f64; 4],
    pub contents: Option<String>,
    pub author: Option<String>,
    pub color: Option<[f64; 3]>,
    pub opacity: Option<f64>,
    pub quad_points: Option<Vec<f64>>,
    pub url: Option<String>,
}

/// Extract all annotations from a specific page.
pub fn extract_annotations(
    doc: &lopdf::Document,
    page_number: u32,
    page_map: &BTreeMap<u32, ObjectId>,
) -> Result<Vec<Annotation>> {
    let page_id = page_map.get(&page_number).ok_or(PdfError::PageOutOfRange {
        page: page_number as usize,
        total: page_map.len(),
    })?;

    let page_obj = doc
        .get_object(*page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get page object: {}", e)))?;
    let page_dict = page_obj
        .as_dict()
        .map_err(|e| PdfError::Annotation(format!("Page is not a dictionary: {}", e)))?;

    let annots_obj = match page_dict.get(b"Annots") {
        Ok(obj) => obj,
        Err(_) => return Ok(Vec::new()), // No annotations
    };

    let annots_array = match annots_obj {
        Object::Array(arr) => arr.clone(),
        Object::Reference(id) => {
            let obj = doc
                .get_object(*id)
                .map_err(|e| PdfError::Annotation(format!("Cannot dereference Annots: {}", e)))?;
            match obj {
                Object::Array(arr) => arr.clone(),
                _ => return Ok(Vec::new()),
            }
        }
        _ => return Ok(Vec::new()),
    };

    let mut annotations = Vec::new();
    for annot_ref in &annots_array {
        let annot_obj = match annot_ref {
            Object::Reference(id) => match doc.get_object(*id) {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            obj => obj,
        };

        let dict = match annot_obj.as_dict() {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Parse annotation type
        let annot_type = match dict.get(b"Subtype") {
            Ok(Object::Name(name)) => AnnotationType::from_name(name),
            _ => AnnotationType::Unknown("unknown".to_string()),
        };

        // Parse rect
        let rect = match dict.get(b"Rect") {
            Ok(Object::Array(arr)) if arr.len() == 4 => {
                let mut r = [0.0f64; 4];
                for (i, v) in arr.iter().enumerate() {
                    r[i] = obj_to_f64(v).unwrap_or(0.0);
                }
                r
            }
            _ => [0.0; 4],
        };

        // Parse contents
        let contents = dict
            .get(b"Contents")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        // Parse author
        let author = dict.get(b"T").ok().and_then(|o| obj_to_string(o, doc));

        // Parse color
        let color = match dict.get(b"C") {
            Ok(Object::Array(arr)) if arr.len() == 3 => {
                let r = obj_to_f64(&arr[0]).unwrap_or(0.0);
                let g = obj_to_f64(&arr[1]).unwrap_or(0.0);
                let b = obj_to_f64(&arr[2]).unwrap_or(0.0);
                Some([r, g, b])
            }
            _ => None,
        };

        // Parse creation date
        let creation_date = dict
            .get(b"CreationDate")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        // Parse opacity
        let opacity = dict.get(b"CA").ok().and_then(obj_to_f64);

        // Parse link destination from /A action or /Dest key
        let (url, destination) = parse_link_destination(dict, doc);

        annotations.push(Annotation {
            annotation_type: annot_type,
            rect,
            contents,
            author,
            color,
            creation_date,
            opacity,
            url,
            destination,
        });
    }

    Ok(annotations)
}

/// Parse link destination from an annotation dictionary.
///
/// Checks the `/A` (action) dict for `/URI` or `/GoTo` actions,
/// and falls back to the `/Dest` key for direct destinations.
fn parse_link_destination(
    dict: &lopdf::Dictionary,
    doc: &lopdf::Document,
) -> (Option<String>, Option<LinkDestination>) {
    // Try /A action dictionary first
    if let Ok(action_obj) = dict.get(b"A") {
        let action_dict = match action_obj {
            Object::Dictionary(d) => Some(d),
            Object::Reference(id) => doc
                .get_object(*id)
                .ok()
                .and_then(|o| o.as_dict().ok()),
            _ => None,
        };
        if let Some(ad) = action_dict {
            // Check action type /S
            if let Ok(Object::Name(s_type)) = ad.get(b"S") {
                match s_type.as_slice() {
                    b"URI" => {
                        if let Ok(uri_obj) = ad.get(b"URI") {
                            let uri = obj_to_string(uri_obj, doc);
                            if let Some(ref u) = uri {
                                return (
                                    Some(u.clone()),
                                    Some(LinkDestination::Uri(u.clone())),
                                );
                            }
                        }
                    }
                    b"GoTo" => {
                        if let Ok(dest_obj) = ad.get(b"D") {
                            if let Some(dest) = parse_dest_value(dest_obj, doc) {
                                return (None, Some(dest));
                            }
                        }
                    }
                    b"GoToR" => {
                        // Remote GoTo — extract the file/URI if present
                        if let Ok(f_obj) = ad.get(b"F") {
                            let uri = obj_to_string(f_obj, doc);
                            if let Some(u) = uri {
                                return (Some(u.clone()), Some(LinkDestination::Uri(u)));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Fallback: check /Dest key directly
    if let Ok(dest_obj) = dict.get(b"Dest") {
        if let Some(dest) = parse_dest_value(dest_obj, doc) {
            return (None, Some(dest));
        }
    }

    (None, None)
}

/// Parse a destination value (array or named string).
fn parse_dest_value(obj: &Object, doc: &lopdf::Document) -> Option<LinkDestination> {
    match obj {
        Object::Array(arr) => {
            // First element is page reference or page number
            if let Some(first) = arr.first() {
                match first {
                    Object::Reference(id) => {
                        // Find page number from page reference
                        let pages = doc.get_pages();
                        for (&page_num, &page_id) in &pages {
                            if page_id == *id {
                                return Some(LinkDestination::GoTo { page: page_num });
                            }
                        }
                        None
                    }
                    Object::Integer(n) => {
                        Some(LinkDestination::GoTo { page: (*n as u32) + 1 })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Object::String(bytes, _) => {
            Some(LinkDestination::Named(String::from_utf8_lossy(bytes).to_string()))
        }
        Object::Name(bytes) => {
            Some(LinkDestination::Named(String::from_utf8_lossy(bytes).to_string()))
        }
        Object::Reference(id) => {
            doc.get_object(*id).ok().and_then(|o| parse_dest_value(o, doc))
        }
        _ => None,
    }
}

/// Extract only link annotations from a specific page.
pub fn extract_links(
    doc: &lopdf::Document,
    page_number: u32,
    page_map: &BTreeMap<u32, ObjectId>,
) -> Result<Vec<Annotation>> {
    let all = extract_annotations(doc, page_number, page_map)?;
    Ok(all
        .into_iter()
        .filter(|a| matches!(a.annotation_type, AnnotationType::Link))
        .collect())
}

/// Add an annotation to a specific page.
pub fn add_annotation(
    doc: &mut lopdf::Document,
    page_number: u32,
    page_map: &BTreeMap<u32, ObjectId>,
    options: &AddAnnotationOptions,
) -> Result<()> {
    let page_id = *page_map.get(&page_number).ok_or(PdfError::PageOutOfRange {
        page: page_number as usize,
        total: page_map.len(),
    })?;

    // Build the annotation dictionary
    let mut annot_dict = dictionary! {
        "Type" => "Annot",
        "Subtype" => Object::Name(options.annotation_type.to_name().to_vec()),
        "Rect" => Object::Array(options.rect.iter().map(|&v| Object::Real(v as f32)).collect()),
        "F" => Object::Integer(4), // Print flag
    };

    if let Some(ref contents) = options.contents {
        annot_dict.set("Contents", Object::String(contents.as_bytes().to_vec(), lopdf::StringFormat::Literal));
    }

    if let Some(ref author) = options.author {
        annot_dict.set("T", Object::String(author.as_bytes().to_vec(), lopdf::StringFormat::Literal));
    }

    if let Some(color) = options.color {
        annot_dict.set(
            "C",
            Object::Array(color.iter().map(|&v| Object::Real(v as f32)).collect()),
        );
    }

    if let Some(opacity) = options.opacity {
        annot_dict.set("CA", Object::Real(opacity as f32));
    }

    if let Some(ref quad_points) = options.quad_points {
        annot_dict.set(
            "QuadPoints",
            Object::Array(quad_points.iter().map(|&v| Object::Real(v as f32)).collect()),
        );
    }

    // Handle Link annotation with URL
    if let Some(ref url) = options.url {
        let action = dictionary! {
            "S" => "URI",
            "URI" => Object::String(url.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        };
        annot_dict.set("A", Object::Dictionary(action));
    }

    // Add the annotation as a new object
    let annot_id = doc.new_object_id();
    doc.objects
        .insert(annot_id, Object::Dictionary(annot_dict));

    // Append to the page's /Annots array
    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

    match page_dict.get_mut(b"Annots") {
        Ok(Object::Array(arr)) => {
            arr.push(Object::Reference(annot_id));
        }
        _ => {
            page_dict.set(
                "Annots",
                Object::Array(vec![Object::Reference(annot_id)]),
            );
        }
    }

    Ok(())
}

/// Remove annotations from a specific page. Returns count removed.
///
/// If `annotation_types` is `Some`, only annotations whose `/Subtype` matches
/// one of the given type strings are removed. If `indices` is `Some`, only
/// annotations at those 0-based positions are removed. Both filters can be
/// combined (AND logic). Pass `None` for both to remove all annotations.
pub fn remove_annotations(
    doc: &mut lopdf::Document,
    page_number: u32,
    page_map: &BTreeMap<u32, ObjectId>,
    annotation_types: Option<&[&str]>,
    indices: Option<&[usize]>,
) -> Result<usize> {
    let page_id = *page_map.get(&page_number).ok_or(PdfError::PageOutOfRange {
        page: page_number as usize,
        total: page_map.len(),
    })?;

    let page_obj = doc
        .get_object(page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict()
        .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

    // Collect all annotation references
    let annots_array = match page_dict.get(b"Annots") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(arr)) => arr.clone(),
            _ => return Ok(0),
        },
        _ => return Ok(0),
    };

    let mut to_remove = Vec::new();
    let mut survivors = Vec::new();

    for (i, annot_ref) in annots_array.iter().enumerate() {
        let annot_obj = match annot_ref {
            Object::Reference(id) => match doc.get_object(*id) {
                Ok(obj) => obj,
                Err(_) => {
                    survivors.push(annot_ref.clone());
                    continue;
                }
            },
            obj => obj,
        };

        let should_remove = {
            // Check index filter
            let index_match = match indices {
                Some(idx_list) => idx_list.contains(&i),
                None => true,
            };

            // Check type filter
            let type_match = match annotation_types {
                Some(types) => {
                    if let Ok(dict) = annot_obj.as_dict() {
                        if let Ok(Object::Name(name)) = dict.get(b"Subtype") {
                            let annot_type = AnnotationType::from_name(name);
                            types.iter().any(|t| annot_type.as_str() == *t)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                None => true,
            };

            index_match && type_match
        };

        if should_remove {
            if let Object::Reference(id) = annot_ref {
                to_remove.push(*id);
            }
        } else {
            survivors.push(annot_ref.clone());
        }
    }

    let removed_count = to_remove.len();

    // Remove annotation objects
    for annot_id in &to_remove {
        doc.objects.remove(annot_id);
    }

    // Update or remove /Annots on the page
    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

    if survivors.is_empty() {
        page_dict.remove(b"Annots");
    } else {
        page_dict.set("Annots", Object::Array(survivors));
    }

    Ok(removed_count)
}

/// Helper: extract a float from a PDF object.
fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v as f64),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

/// Helper: extract a string from a PDF object.
fn obj_to_string(obj: &Object, doc: &lopdf::Document) -> Option<String> {
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Reference(id) => doc
            .get_object(*id)
            .ok()
            .and_then(|o| obj_to_string(o, doc)),
        _ => None,
    }
}
