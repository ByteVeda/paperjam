use lopdf::{dictionary, Object, ObjectId, Stream};

use crate::error::{PdfError, Result};
use crate::forms::types::FormFieldType;

/// Parsed default appearance (/DA) string.
pub(crate) struct DefaultAppearance {
    pub font_name: String,
    pub font_size: f64,
    pub color: [f64; 3],
}

impl DefaultAppearance {
    /// Parse a /DA string like "/Helv 12 Tf 0 g" or "/Helv 12 Tf 0.5 0.5 0.5 rg".
    pub fn parse(da: &str) -> Self {
        let tokens: Vec<&str> = da.split_whitespace().collect();
        let mut font_name = "Helv".to_string();
        let mut font_size = 12.0;
        let mut color = [0.0, 0.0, 0.0];

        for (i, &tok) in tokens.iter().enumerate() {
            if tok == "Tf" && i >= 2 {
                font_name = tokens[i - 2].trim_start_matches('/').to_string();
                font_size = tokens[i - 1].parse().unwrap_or(12.0);
            } else if tok == "g" && i >= 1 {
                let gray = tokens[i - 1].parse().unwrap_or(0.0);
                color = [gray, gray, gray];
            } else if tok == "rg" && i >= 3 {
                color = [
                    tokens[i - 3].parse().unwrap_or(0.0),
                    tokens[i - 2].parse().unwrap_or(0.0),
                    tokens[i - 1].parse().unwrap_or(0.0),
                ];
            }
        }

        Self {
            font_name,
            font_size,
            color,
        }
    }
}

/// Generate an appearance stream for a field and set it on the widget.
pub(crate) fn generate_field_appearance(
    doc: &mut lopdf::Document,
    field_id: ObjectId,
    field_type: &FormFieldType,
    value: &str,
) -> Result<()> {
    match field_type {
        FormFieldType::Text | FormFieldType::ComboBox | FormFieldType::ListBox => {
            generate_text_appearance(doc, field_id, value)
        }
        FormFieldType::Checkbox => generate_checkbox_appearance(doc, field_id, value),
        FormFieldType::RadioButton => generate_radio_appearance(doc, field_id, value),
        _ => Ok(()), // PushButton, Signature, Unknown — skip
    }
}

/// Read the /DA string from a field or its AcroForm, with fallback.
fn get_da_string(doc: &lopdf::Document, field_id: ObjectId) -> String {
    // Try field itself
    if let Ok(field_obj) = doc.get_object(field_id) {
        if let Ok(dict) = field_obj.as_dict() {
            if let Ok(Object::String(bytes, _)) = dict.get(b"DA") {
                return String::from_utf8_lossy(bytes).to_string();
            }
        }
    }

    // Try AcroForm /DA
    if let Ok(root_ref) = doc.trailer.get(b"Root") {
        if let Ok(root_id) = root_ref.as_reference() {
            if let Ok(root_obj) = doc.get_object(root_id) {
                if let Ok(root_dict) = root_obj.as_dict() {
                    let af_dict = match root_dict.get(b"AcroForm") {
                        Ok(Object::Dictionary(d)) => Some(d),
                        Ok(Object::Reference(id)) => doc
                            .get_object(*id)
                            .ok()
                            .and_then(|o| o.as_dict().ok()),
                        _ => None,
                    };
                    if let Some(af) = af_dict {
                        if let Ok(Object::String(bytes, _)) = af.get(b"DA") {
                            return String::from_utf8_lossy(bytes).to_string();
                        }
                    }
                }
            }
        }
    }

    "/Helv 12 Tf 0 g".to_string()
}

/// Get the widget rect as [x1, y1, x2, y2].
fn get_widget_rect(doc: &lopdf::Document, widget_id: ObjectId) -> Option<[f64; 4]> {
    let obj = doc.get_object(widget_id).ok()?;
    let dict = obj.as_dict().ok()?;
    if let Ok(Object::Array(arr)) = dict.get(b"Rect") {
        if arr.len() == 4 {
            let mut r = [0.0f64; 4];
            for (i, v) in arr.iter().enumerate() {
                r[i] = match v {
                    Object::Real(f) => *f as f64,
                    Object::Integer(n) => *n as f64,
                    _ => 0.0,
                };
            }
            return Some(r);
        }
    }
    None
}

/// Resolve a font reference from AcroForm /DR/Font/{font_name}.
/// If not found, creates a basic Helvetica font.
fn resolve_font_ref(doc: &mut lopdf::Document, font_name: &str) -> ObjectId {
    // Try to find existing font in AcroForm DR
    if let Ok(root_ref) = doc.trailer.get(b"Root") {
        if let Ok(root_id) = root_ref.as_reference() {
            if let Ok(root_obj) = doc.get_object(root_id) {
                if let Ok(root_dict) = root_obj.as_dict() {
                    let af_dict = match root_dict.get(b"AcroForm") {
                        Ok(Object::Dictionary(d)) => Some(d.clone()),
                        Ok(Object::Reference(id)) => doc
                            .get_object(*id)
                            .ok()
                            .and_then(|o| o.as_dict().ok())
                            .cloned(),
                        _ => None,
                    };
                    if let Some(af) = af_dict {
                        if let Ok(dr) = af.get(b"DR") {
                            let dr_dict = match dr {
                                Object::Dictionary(d) => Some(d),
                                Object::Reference(id) => doc
                                    .get_object(*id)
                                    .ok()
                                    .and_then(|o| o.as_dict().ok()),
                                _ => None,
                            };
                            if let Some(dr) = dr_dict {
                                if let Ok(fonts) = dr.get(b"Font") {
                                    let fonts_dict = match fonts {
                                        Object::Dictionary(d) => Some(d),
                                        Object::Reference(id) => doc
                                            .get_object(*id)
                                            .ok()
                                            .and_then(|o| o.as_dict().ok()),
                                        _ => None,
                                    };
                                    if let Some(fd) = fonts_dict {
                                        if let Ok(Object::Reference(id)) =
                                            fd.get(font_name.as_bytes())
                                        {
                                            return *id;
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

    // Create a basic Type1 Helvetica font
    let font_dict = dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    };
    let font_id = doc.new_object_id();
    doc.objects
        .insert(font_id, Object::Dictionary(font_dict));
    font_id
}

/// Escape a string for use in a PDF content stream text operator.
fn escape_pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out
}

/// Generate a text field appearance stream.
fn generate_text_appearance(
    doc: &mut lopdf::Document,
    field_id: ObjectId,
    value: &str,
) -> Result<()> {
    // Get rect from the field itself or its first widget kid
    let widget_id = get_widget_id(doc, field_id);
    let rect = get_widget_rect(doc, widget_id).unwrap_or([0.0, 0.0, 100.0, 20.0]);

    let w = (rect[2] - rect[0]).abs();
    let h = (rect[3] - rect[1]).abs();

    let da_str = get_da_string(doc, field_id);
    let da = DefaultAppearance::parse(&da_str);

    let font_size = if da.font_size == 0.0 {
        (h * 0.7).max(1.0)
    } else {
        da.font_size
    };

    let font_ref = resolve_font_ref(doc, &da.font_name);
    let escaped = escape_pdf_string(value);

    // Position text with left margin and vertically centered
    let x = 2.0;
    let y = (h - font_size) / 2.0;

    let content = format!(
        "/Tx BMC q 1 1 {} {} re W n BT /{} {} Tf {} {} {} rg {} {} Td ({}) Tj ET Q EMC",
        w - 2.0,
        h - 2.0,
        da.font_name,
        font_size,
        da.color[0],
        da.color[1],
        da.color[2],
        x,
        y.max(1.0),
        escaped
    );

    let resources = dictionary! {
        "Font" => dictionary! {
            da.font_name.as_str() => Object::Reference(font_ref),
        },
    };

    let bbox = Object::Array(vec![
        Object::Real(0.0),
        Object::Real(0.0),
        Object::Real(w as f32),
        Object::Real(h as f32),
    ]);

    let stream_dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Form",
        "BBox" => bbox,
        "Resources" => resources,
    };

    let stream = Stream::new(stream_dict, content.into_bytes());
    let stream_id = doc.new_object_id();
    doc.objects.insert(stream_id, Object::Stream(stream));

    // Set /AP << /N stream_ref >> on the widget
    let widget_obj = doc
        .get_object_mut(widget_id)
        .map_err(|e| PdfError::Form(format!("Cannot get widget: {}", e)))?;
    let widget_dict = widget_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("Widget is not a dictionary".to_string()))?;
    widget_dict.set(
        "AP",
        Object::Dictionary(dictionary! {
            "N" => Object::Reference(stream_id),
        }),
    );

    Ok(())
}

/// Generate checkbox appearance streams (/Yes and /Off).
fn generate_checkbox_appearance(
    doc: &mut lopdf::Document,
    field_id: ObjectId,
    value: &str,
) -> Result<()> {
    let widget_id = get_widget_id(doc, field_id);
    let rect = get_widget_rect(doc, widget_id).unwrap_or([0.0, 0.0, 12.0, 12.0]);

    let w = (rect[2] - rect[0]).abs();
    let h = (rect[3] - rect[1]).abs();

    // "Yes" appearance — checkmark
    let yes_content = format!(
        "q 0 0 1 rg BT /ZaDb {} Tf 0.5 0.5 Td (4) Tj ET Q",
        h * 0.8
    );
    let yes_stream = create_form_xobject(doc, w, h, yes_content.into_bytes());

    // "Off" appearance — empty box
    let off_content = "q Q".to_string();
    let off_stream = create_form_xobject(doc, w, h, off_content.into_bytes());

    let export_name = if value == "Off" { "Off" } else { "Yes" };

    // Set /AP << /N << /Yes ref1 /Off ref2 >> >>
    let ap_n = dictionary! {
        export_name => Object::Reference(yes_stream),
        "Off" => Object::Reference(off_stream),
    };

    let widget_obj = doc
        .get_object_mut(widget_id)
        .map_err(|e| PdfError::Form(format!("Cannot get widget: {}", e)))?;
    let widget_dict = widget_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("Widget is not a dictionary".to_string()))?;
    widget_dict.set(
        "AP",
        Object::Dictionary(dictionary! { "N" => Object::Dictionary(ap_n) }),
    );

    Ok(())
}

/// Generate radio button appearance streams.
fn generate_radio_appearance(
    doc: &mut lopdf::Document,
    field_id: ObjectId,
    value: &str,
) -> Result<()> {
    // For radio buttons, we need to set appearances on each widget kid
    let dict = doc
        .get_object(field_id)
        .map_err(|e| PdfError::Form(format!("Cannot get field: {}", e)))?
        .as_dict()
        .map_err(|_| PdfError::Form("Field not dict".to_string()))?
        .clone();

    let widget_ids: Vec<ObjectId> = if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
        kids.iter()
            .filter_map(|k| k.as_reference().ok())
            .collect()
    } else {
        vec![field_id]
    };

    for wid in &widget_ids {
        let rect = get_widget_rect(doc, *wid).unwrap_or([0.0, 0.0, 12.0, 12.0]);
        let w = (rect[2] - rect[0]).abs();
        let h = (rect[3] - rect[1]).abs();

        // Determine the export value for this widget from its existing /AP/N keys
        let export_val = get_widget_export_value(doc, *wid).unwrap_or_else(|| value.to_string());

        // Selected: filled circle
        let r = w.min(h) / 2.0 - 1.0;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let k = 0.5523; // bezier approximation for circle
        let selected_content = format!(
            "q 0 0 0 rg {} {} m {} {} {} {} {} {} c {} {} {} {} {} {} c {} {} {} {} {} {} c {} {} {} {} {} {} c f Q",
            cx, cy + r,
            cx + r * k, cy + r, cx + r, cy + r * k, cx + r, cy,
            cx + r, cy - r * k, cx + r * k, cy - r, cx, cy - r,
            cx - r * k, cy - r, cx - r, cy - r * k, cx - r, cy,
            cx - r, cy + r * k, cx - r * k, cy + r, cx, cy + r,
        );
        let selected_id = create_form_xobject(doc, w, h, selected_content.into_bytes());

        // Off: empty
        let off_content = "q Q".to_string();
        let off_id = create_form_xobject(doc, w, h, off_content.into_bytes());

        let ap_n = dictionary! {
            export_val.as_str() => Object::Reference(selected_id),
            "Off" => Object::Reference(off_id),
        };

        let widget_obj = doc
            .get_object_mut(*wid)
            .map_err(|e| PdfError::Form(format!("Cannot get widget: {}", e)))?;
        let widget_dict = widget_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("Widget not dict".to_string()))?;
        widget_dict.set(
            "AP",
            Object::Dictionary(dictionary! { "N" => Object::Dictionary(ap_n) }),
        );

        // Set /AS
        if export_val == value {
            widget_dict.set("AS", Object::Name(value.as_bytes().to_vec()));
        } else {
            widget_dict.set("AS", Object::Name(b"Off".to_vec()));
        }
    }

    Ok(())
}

/// Create a Form XObject and return its ObjectId.
fn create_form_xobject(
    doc: &mut lopdf::Document,
    w: f64,
    h: f64,
    content: Vec<u8>,
) -> ObjectId {
    let bbox = Object::Array(vec![
        Object::Real(0.0),
        Object::Real(0.0),
        Object::Real(w as f32),
        Object::Real(h as f32),
    ]);

    let stream_dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Form",
        "BBox" => bbox,
    };

    let stream = Stream::new(stream_dict, content);
    let id = doc.new_object_id();
    doc.objects.insert(id, Object::Stream(stream));
    id
}

/// Get the widget ObjectId for a field (either the field itself or its first widget kid).
fn get_widget_id(doc: &lopdf::Document, field_id: ObjectId) -> ObjectId {
    if let Ok(obj) = doc.get_object(field_id) {
        if let Ok(dict) = obj.as_dict() {
            if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
                if let Some(first) = kids.first() {
                    if let Ok(kid_id) = first.as_reference() {
                        return kid_id;
                    }
                }
            }
        }
    }
    field_id
}

/// Get the export value of a radio widget from its /AP/N keys (the non-"Off" key).
fn get_widget_export_value(doc: &lopdf::Document, widget_id: ObjectId) -> Option<String> {
    let obj = doc.get_object(widget_id).ok()?;
    let dict = obj.as_dict().ok()?;
    let ap = dict.get(b"AP").ok()?;
    let ap_dict = match ap {
        Object::Dictionary(d) => d,
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok()?,
        _ => return None,
    };
    let n = ap_dict.get(b"N").ok()?;
    let n_dict = match n {
        Object::Dictionary(d) => d,
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok()?,
        _ => return None,
    };

    for (key, _) in n_dict.iter() {
        if key != b"Off" {
            return Some(String::from_utf8_lossy(key).to_string());
        }
    }
    None
}
