use lopdf::{dictionary, Object, ObjectId, Stream};

use crate::document::Document;
use crate::error::{PdfError, Result};

/// Whether the stamp goes over or under existing content.
#[derive(Debug, Clone)]
pub enum StampLayer {
    Over,
    Under,
}

impl StampLayer {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "under" | "underlay" => Self::Under,
            _ => Self::Over,
        }
    }
}

/// Options for page stamping/overlay.
pub struct StampOptions {
    /// 1-based page number in the stamp document to use as the stamp.
    pub source_page: u32,
    /// Target pages in the document. None = all pages.
    pub target_pages: Option<Vec<u32>>,
    /// X offset in points.
    pub x: f64,
    /// Y offset in points.
    pub y: f64,
    /// Scale factor (default 1.0).
    pub scale: f64,
    /// Opacity (0.0–1.0, default 1.0).
    pub opacity: f64,
    /// Whether stamp goes over or under existing content.
    pub layer: StampLayer,
}

impl Default for StampOptions {
    fn default() -> Self {
        Self {
            source_page: 1,
            target_pages: None,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            opacity: 1.0,
            layer: StampLayer::Over,
        }
    }
}

/// Stamp (overlay) a page from one PDF onto pages of another.
pub fn stamp_pages(
    doc: &Document,
    stamp_doc: &Document,
    options: &StampOptions,
) -> Result<Document> {
    let stamp_inner = stamp_doc.inner();
    let stamp_page_map = stamp_inner.get_pages();

    let stamp_page_id =
        stamp_page_map
            .get(&options.source_page)
            .ok_or(PdfError::PageOutOfRange {
                page: options.source_page as usize,
                total: stamp_page_map.len(),
            })?;

    // Get the stamp page's content stream bytes and MediaBox
    let stamp_page_obj = stamp_inner
        .get_object(*stamp_page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get stamp page: {}", e)))?;
    let stamp_page_dict = stamp_page_obj
        .as_dict()
        .map_err(|e| PdfError::Annotation(format!("Stamp page not a dict: {}", e)))?;

    let media_box = get_media_box(stamp_inner, stamp_page_dict);

    // Collect content stream bytes from the stamp page
    let stamp_content_bytes = collect_content_stream(stamp_inner, stamp_page_dict)?;

    // Collect resources from the stamp page
    let stamp_resources = get_resolved_resources(stamp_inner, stamp_page_dict);

    // Clone target document
    let mut new_doc = doc.inner().clone();
    let page_map = new_doc.get_pages();
    let total_pages = page_map.len() as u32;

    let target_pages: Vec<u32> = match &options.target_pages {
        Some(pages) => pages.clone(),
        None => (1..=total_pages).collect(),
    };

    // Validate target pages
    for &p in &target_pages {
        if !page_map.contains_key(&p) {
            return Err(PdfError::PageOutOfRange {
                page: p as usize,
                total: page_map.len(),
            });
        }
    }

    // Copy all objects from stamp document with remapped IDs
    let mut id_map = std::collections::BTreeMap::new();
    for (old_id, obj) in &stamp_inner.objects {
        let new_id = new_doc.new_object_id();
        id_map.insert(*old_id, new_id);
        new_doc.objects.insert(new_id, obj.clone());
    }

    // Remap references within copied objects
    for new_id in id_map.values() {
        if let Some(obj) = new_doc.objects.get_mut(new_id) {
            remap_refs_with_map(obj, &id_map);
        }
    }

    // Build Form XObject from stamp content
    let bbox = Object::Array(vec![
        Object::Real(media_box[0] as f32),
        Object::Real(media_box[1] as f32),
        Object::Real(media_box[2] as f32),
        Object::Real(media_box[3] as f32),
    ]);

    let mut xobj_dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Form",
        "BBox" => bbox,
    };

    // Add remapped resources to the XObject
    if let Some(res) = &stamp_resources {
        let mut obj = Object::Dictionary(res.clone());
        remap_refs_with_map(&mut obj, &id_map);
        if let Object::Dictionary(remapped_res) = obj {
            xobj_dict.set("Resources", Object::Dictionary(remapped_res));
        }
    }

    let xobj_stream = Stream::new(xobj_dict, stamp_content_bytes);
    let xobj_id = new_doc.new_object_id();
    new_doc.objects.insert(xobj_id, Object::Stream(xobj_stream));

    // Create ExtGState for opacity if needed
    let gs_id = if options.opacity < 1.0 {
        let gs_dict = dictionary! {
            "Type" => "ExtGState",
            "CA" => Object::Real(options.opacity as f32),
            "ca" => Object::Real(options.opacity as f32),
        };
        let id = new_doc.new_object_id();
        new_doc.objects.insert(id, Object::Dictionary(gs_dict));
        Some(id)
    } else {
        None
    };

    let xobj_name = "StampXO";
    let gs_name = "StampGS";

    // Apply stamp to each target page
    for &page_num in &target_pages {
        let page_id = page_map[&page_num];

        // Build content stream for the stamp invocation
        let mut content = String::new();
        content.push_str("q\n");
        if let Some(_gs) = gs_id {
            content.push_str(&format!("/{} gs\n", gs_name));
        }
        content.push_str(&format!(
            "{} 0 0 {} {} {} cm\n",
            options.scale, options.scale, options.x, options.y
        ));
        content.push_str(&format!("/{} Do\n", xobj_name));
        content.push_str("Q\n");

        let content_stream = Stream::new(dictionary! {}, content.into_bytes());
        let content_id = new_doc.new_object_id();
        new_doc
            .objects
            .insert(content_id, Object::Stream(content_stream));

        // Inject content stream into page
        let page_obj = new_doc
            .get_object_mut(page_id)
            .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
        let page_dict = page_obj
            .as_dict_mut()
            .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

        match page_dict.get(b"Contents") {
            Ok(Object::Reference(existing_ref)) => {
                let existing_ref = *existing_ref;
                match options.layer {
                    StampLayer::Over => {
                        page_dict.set(
                            "Contents",
                            Object::Array(vec![
                                Object::Reference(existing_ref),
                                Object::Reference(content_id),
                            ]),
                        );
                    }
                    StampLayer::Under => {
                        page_dict.set(
                            "Contents",
                            Object::Array(vec![
                                Object::Reference(content_id),
                                Object::Reference(existing_ref),
                            ]),
                        );
                    }
                }
            }
            Ok(Object::Array(arr)) => {
                let mut new_arr = arr.clone();
                match options.layer {
                    StampLayer::Over => new_arr.push(Object::Reference(content_id)),
                    StampLayer::Under => new_arr.insert(0, Object::Reference(content_id)),
                }
                page_dict.set("Contents", Object::Array(new_arr));
            }
            _ => {
                page_dict.set("Contents", Object::Reference(content_id));
            }
        }

        // Ensure page has resources and add XObject + ExtGState references
        ensure_page_resources(&mut new_doc, page_id)?;

        let page_obj = new_doc
            .get_object_mut(page_id)
            .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
        let page_dict = page_obj
            .as_dict_mut()
            .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

        let resources = page_dict
            .get_mut(b"Resources")
            .map_err(|e| PdfError::Annotation(format!("No resources: {}", e)))?;
        let resources_dict = resources
            .as_dict_mut()
            .map_err(|e| PdfError::Annotation(format!("Resources not a dict: {}", e)))?;

        // Add XObject
        match resources_dict.get_mut(b"XObject") {
            Ok(Object::Dictionary(xobj_resources)) => {
                xobj_resources.set(xobj_name, Object::Reference(xobj_id));
            }
            _ => {
                let mut xobj_resources = lopdf::Dictionary::new();
                xobj_resources.set(xobj_name, Object::Reference(xobj_id));
                resources_dict.set("XObject", Object::Dictionary(xobj_resources));
            }
        }

        // Add ExtGState if needed
        if let Some(gs) = gs_id {
            match resources_dict.get_mut(b"ExtGState") {
                Ok(Object::Dictionary(gs_resources)) => {
                    gs_resources.set(gs_name, Object::Reference(gs));
                }
                _ => {
                    let mut gs_resources = lopdf::Dictionary::new();
                    gs_resources.set(gs_name, Object::Reference(gs));
                    resources_dict.set("ExtGState", Object::Dictionary(gs_resources));
                }
            }
        }
    }

    Document::from_lopdf(new_doc)
}

/// Collect the content stream bytes from a page.
fn collect_content_stream(doc: &lopdf::Document, page_dict: &lopdf::Dictionary) -> Result<Vec<u8>> {
    match page_dict.get(b"Contents") {
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Stream(stream)) => Ok(stream.content.clone()),
            _ => Ok(Vec::new()),
        },
        Ok(Object::Array(arr)) => {
            let mut bytes = Vec::new();
            for item in arr {
                if let Ok(id) = item.as_reference() {
                    if let Ok(Object::Stream(stream)) = doc.get_object(id) {
                        bytes.extend_from_slice(&stream.content);
                        bytes.push(b'\n');
                    }
                }
            }
            Ok(bytes)
        }
        Ok(Object::Stream(stream)) => Ok(stream.content.clone()),
        _ => Ok(Vec::new()),
    }
}

/// Get the MediaBox from a page dict, walking up parents if needed.
fn get_media_box(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> [f64; 4] {
    if let Ok(Object::Array(arr)) = dict.get(b"MediaBox") {
        if arr.len() == 4 {
            let vals: Vec<f64> = arr.iter().filter_map(obj_to_f64).collect();
            if vals.len() == 4 {
                return [vals[0], vals[1], vals[2], vals[3]];
            }
        }
    }
    if let Ok(parent_ref) = dict.get(b"Parent") {
        if let Ok(parent_id) = parent_ref.as_reference() {
            if let Ok(parent_obj) = doc.get_object(parent_id) {
                if let Ok(parent_dict) = parent_obj.as_dict() {
                    return get_media_box(doc, parent_dict);
                }
            }
        }
    }
    [0.0, 0.0, 612.0, 792.0]
}

/// Get resources dictionary from a page, resolving references.
fn get_resolved_resources(
    doc: &lopdf::Document,
    dict: &lopdf::Dictionary,
) -> Option<lopdf::Dictionary> {
    match dict.get(b"Resources") {
        Ok(Object::Dictionary(d)) => Some(d.clone()),
        Ok(Object::Reference(id)) => {
            if let Ok(obj) = doc.get_object(*id) {
                if let Ok(d) = obj.as_dict() {
                    return Some(d.clone());
                }
            }
            None
        }
        _ => None,
    }
}

/// Remap references using an explicit ID map.
fn remap_refs_with_map(object: &mut Object, map: &std::collections::BTreeMap<ObjectId, ObjectId>) {
    match object {
        Object::Reference(id) => {
            if let Some(new_id) = map.get(id) {
                *id = *new_id;
            }
        }
        Object::Array(arr) => {
            for item in arr.iter_mut() {
                remap_refs_with_map(item, map);
            }
        }
        Object::Dictionary(dict) => {
            for (_, item) in dict.iter_mut() {
                remap_refs_with_map(item, map);
            }
        }
        Object::Stream(stream) => {
            for (_, item) in stream.dict.iter_mut() {
                remap_refs_with_map(item, map);
            }
        }
        _ => {}
    }
}

/// Ensure the page has its own /Resources dict (not inherited).
fn ensure_page_resources(doc: &mut lopdf::Document, page_id: ObjectId) -> Result<()> {
    let page_obj = doc
        .get_object(page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict()
        .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

    let has_resources = page_dict.get(b"Resources").is_ok();
    if has_resources {
        if let Ok(Object::Reference(res_id)) = page_dict.get(b"Resources") {
            let res_id = *res_id;
            if let Ok(res_obj) = doc.get_object(res_id) {
                let cloned = res_obj.clone();
                let page_obj = doc.get_object_mut(page_id).unwrap();
                let page_dict = page_obj.as_dict_mut().unwrap();
                page_dict.set("Resources", cloned);
            }
        }
        return Ok(());
    }

    // Try to inherit from parent
    let parent_resources = {
        let page_obj = doc
            .get_object(page_id)
            .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
        let page_dict = page_obj.as_dict().unwrap();

        if let Ok(parent_ref) = page_dict.get(b"Parent") {
            if let Ok(parent_id) = parent_ref.as_reference() {
                if let Ok(parent_obj) = doc.get_object(parent_id) {
                    if let Ok(parent_dict) = parent_obj.as_dict() {
                        if let Ok(res) = parent_dict.get(b"Resources") {
                            match res {
                                Object::Reference(rid) => doc.get_object(*rid).ok().cloned(),
                                other => Some(other.clone()),
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Annotation(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Annotation(format!("Page not a dict: {}", e)))?;

    match parent_resources {
        Some(res) => page_dict.set("Resources", res),
        None => page_dict.set("Resources", Object::Dictionary(lopdf::Dictionary::new())),
    }

    Ok(())
}

fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v as f64),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}
