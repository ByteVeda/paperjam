use lopdf::{dictionary, Object};

use crate::document::Document;
use crate::error::{PdfError, Result};
use crate::forms::types::{CreateFieldOptions, CreateFieldResult, FormFieldType};
use crate::forms::{appearance, traverse};

/// Create a new form field and add it to the document.
pub fn create_form_field(
    doc: &Document,
    options: &CreateFieldOptions,
) -> Result<(Document, CreateFieldResult)> {
    let mut inner = doc.inner().clone();

    // Validate page exists
    let pages = inner.get_pages();
    let page_id = pages.get(&options.page).copied().ok_or_else(|| {
        PdfError::Form(format!(
            "Page {} does not exist (document has {} pages)",
            options.page,
            pages.len()
        ))
    })?;

    // Check name doesn't already exist
    if traverse::find_field_id(&inner, &options.name)?.is_some() {
        return Err(PdfError::Form(format!(
            "Field '{}' already exists",
            options.name
        )));
    }

    // Map field type to /FT name
    let ft_name = match &options.field_type {
        FormFieldType::Text => "Tx",
        FormFieldType::Checkbox | FormFieldType::RadioButton | FormFieldType::PushButton => "Btn",
        FormFieldType::ComboBox | FormFieldType::ListBox => "Ch",
        FormFieldType::Signature => "Sig",
        FormFieldType::Unknown(_) => {
            return Err(PdfError::Form(
                "Cannot create Unknown field type".to_string(),
            ))
        }
    };

    // Compute /Ff flags
    let mut ff: u32 = 0;
    if options.read_only {
        ff |= 1;
    }
    if options.required {
        ff |= 2;
    }
    // Type-specific flags
    match &options.field_type {
        FormFieldType::PushButton => ff |= 1 << 16,
        FormFieldType::RadioButton => ff |= 1 << 15,
        FormFieldType::ComboBox => ff |= 1 << 17,
        _ => {}
    }

    // Build field+widget dictionary
    let rect = Object::Array(vec![
        Object::Real(options.rect[0] as f32),
        Object::Real(options.rect[1] as f32),
        Object::Real(options.rect[2] as f32),
        Object::Real(options.rect[3] as f32),
    ]);

    // Build DA string
    let da_string = format!(
        "/Helv {} Tf 0 g",
        if options.font_size == 0.0 {
            12.0
        } else {
            options.font_size
        }
    );

    let mut field_dict = dictionary! {
        "Type" => "Annot",
        "Subtype" => "Widget",
        "FT" => Object::Name(ft_name.as_bytes().to_vec()),
        "T" => Object::String(options.name.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        "Ff" => Object::Integer(ff as i64),
        "Rect" => rect,
        "P" => Object::Reference(page_id),
        "DA" => Object::String(da_string.as_bytes().to_vec(), lopdf::StringFormat::Literal),
    };

    // Set value
    if let Some(ref value) = options.value {
        match &options.field_type {
            FormFieldType::Checkbox | FormFieldType::RadioButton => {
                field_dict.set("V", Object::Name(value.as_bytes().to_vec()));
                field_dict.set("AS", Object::Name(value.as_bytes().to_vec()));
            }
            _ => {
                field_dict.set(
                    "V",
                    Object::String(value.as_bytes().to_vec(), lopdf::StringFormat::Literal),
                );
            }
        }
    } else if matches!(
        options.field_type,
        FormFieldType::Checkbox | FormFieldType::RadioButton
    ) {
        field_dict.set("AS", Object::Name(b"Off".to_vec()));
    }

    // Set default value
    if let Some(ref dv) = options.default_value {
        field_dict.set(
            "DV",
            Object::String(dv.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }

    // Set max length
    if let Some(max_len) = options.max_length {
        field_dict.set("MaxLen", Object::Integer(max_len as i64));
    }

    // Set options for choice fields
    if !options.options.is_empty() {
        let opt_array: Vec<Object> = options
            .options
            .iter()
            .map(|o| {
                Object::Array(vec![
                    Object::String(
                        o.export_value.as_bytes().to_vec(),
                        lopdf::StringFormat::Literal,
                    ),
                    Object::String(o.display.as_bytes().to_vec(), lopdf::StringFormat::Literal),
                ])
            })
            .collect();
        field_dict.set("Opt", Object::Array(opt_array));
    }

    // Insert as new object
    let field_id = inner.new_object_id();
    inner
        .objects
        .insert(field_id, Object::Dictionary(field_dict));

    // Add to page's /Annots
    let page_obj = inner
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Form(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("Page not dict".to_string()))?;
    match page_dict.get_mut(b"Annots") {
        Ok(Object::Array(annots)) => {
            annots.push(Object::Reference(field_id));
        }
        _ => {
            page_dict.set("Annots", Object::Array(vec![Object::Reference(field_id)]));
        }
    }

    // Ensure AcroForm and add field
    super::add_field_to_acroform(&mut inner, field_id)?;

    // Ensure AcroForm has default resources with Helvetica
    ensure_acroform_font(&mut inner)?;

    // Generate appearance if requested
    if options.generate_appearance {
        let value_str = options.value.as_deref().unwrap_or("");
        if !value_str.is_empty()
            || matches!(
                options.field_type,
                FormFieldType::Checkbox | FormFieldType::RadioButton
            )
        {
            appearance::generate_field_appearance(
                &mut inner,
                field_id,
                &options.field_type,
                value_str,
            )?;
        }
    }

    Ok((
        Document::from_lopdf(inner)?,
        CreateFieldResult {
            field_name: options.name.clone(),
            created: true,
        },
    ))
}

/// Ensure AcroForm /DR has a Helvetica font reference.
fn ensure_acroform_font(doc: &mut lopdf::Document) -> Result<()> {
    let af_id = super::ensure_acroform(doc)?;

    // Check if DR/Font/Helv already exists
    let has_helv = {
        let af_obj = doc
            .get_object(af_id)
            .map_err(|e| PdfError::Form(format!("Cannot get AcroForm: {}", e)))?;
        let af_dict = af_obj
            .as_dict()
            .map_err(|_| PdfError::Form("AcroForm not dict".to_string()))?;
        if let Ok(dr) = af_dict.get(b"DR") {
            let dr_dict = match dr {
                Object::Dictionary(d) => Some(d),
                _ => None,
            };
            if let Some(dr) = dr_dict {
                if let Ok(Object::Dictionary(fd)) = dr.get(b"Font") {
                    fd.get(b"Helv").is_ok()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    };

    if !has_helv {
        // Create Helvetica font
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
            "Encoding" => "WinAnsiEncoding",
        };
        let font_id = doc.new_object_id();
        doc.objects.insert(font_id, Object::Dictionary(font_dict));

        // Set DR
        let af_obj = doc
            .get_object_mut(af_id)
            .map_err(|e| PdfError::Form(format!("Cannot get AcroForm: {}", e)))?;
        let af_dict = af_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("AcroForm not dict".to_string()))?;

        // Try to add to existing DR, or create new one
        if let Ok(Object::Dictionary(dr)) = af_dict.get_mut(b"DR") {
            if let Ok(Object::Dictionary(fonts)) = dr.get_mut(b"Font") {
                fonts.set("Helv", Object::Reference(font_id));
            } else {
                dr.set(
                    "Font",
                    Object::Dictionary(dictionary! {
                        "Helv" => Object::Reference(font_id),
                    }),
                );
            }
        } else {
            af_dict.set(
                "DR",
                Object::Dictionary(dictionary! {
                    "Font" => Object::Dictionary(dictionary! {
                        "Helv" => Object::Reference(font_id),
                    }),
                }),
            );
        }
    }

    Ok(())
}
