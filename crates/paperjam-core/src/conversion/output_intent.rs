//! sRGB ICC profile embedding and OutputIntent creation for PDF/A.

use lopdf::{dictionary, Object};

use crate::error::{PdfError, Result};

/// Embedded sRGB IEC61966-2.1 ICC profile.
const SRGB_ICC_PROFILE: &[u8] = include_bytes!("../../data/sRGB.icc");

/// Ensure the document has an OutputIntent with an sRGB ICC profile.
pub fn ensure_output_intent(doc: &mut lopdf::Document) -> Result<Vec<String>> {
    let mut actions = Vec::new();

    let catalog_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Conversion("No /Root".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Conversion("/Root not ref".to_string()))?;

    // Check if a PDF/A OutputIntent already exists
    let has_pdfa_intent = {
        let catalog = doc
            .get_object(catalog_id)
            .map_err(|e| PdfError::Conversion(format!("catalog: {}", e)))?
            .as_dict()
            .map_err(|_| PdfError::Conversion("catalog not dict".to_string()))?;

        if let Ok(Object::Array(arr)) = catalog.get(b"OutputIntents") {
            arr.iter().any(|item| {
                let d = match item {
                    Object::Reference(id) => {
                        doc.get_object(*id).ok().and_then(|o| o.as_dict().ok())
                    }
                    Object::Dictionary(d) => Some(d),
                    _ => None,
                };
                d.is_some_and(|d| matches!(d.get(b"S"), Ok(Object::Name(n)) if n == b"GTS_PDFA1"))
            })
        } else {
            false
        }
    };

    if has_pdfa_intent {
        return Ok(actions);
    }

    // Create ICC profile stream
    let icc_stream = lopdf::Stream::new(
        dictionary! {
            "N" => Object::Integer(3),
            "Alternate" => Object::Name(b"DeviceRGB".to_vec()),
            "Length" => Object::Integer(SRGB_ICC_PROFILE.len() as i64)
        },
        SRGB_ICC_PROFILE.to_vec(),
    );
    let icc_id = doc.add_object(Object::Stream(icc_stream));

    // Create OutputIntent dictionary
    let output_intent = dictionary! {
        "Type" => Object::Name(b"OutputIntent".to_vec()),
        "S" => Object::Name(b"GTS_PDFA1".to_vec()),
        "OutputConditionIdentifier" => Object::String(
            b"sRGB IEC61966-2.1".to_vec(),
            lopdf::StringFormat::Literal,
        ),
        "RegistryName" => Object::String(
            b"http://www.color.org".to_vec(),
            lopdf::StringFormat::Literal,
        ),
        "DestOutputProfile" => Object::Reference(icc_id)
    };
    let oi_id = doc.add_object(Object::Dictionary(output_intent));

    // Add to catalog /OutputIntents
    let catalog = doc
        .get_object_mut(catalog_id)
        .map_err(|e| PdfError::Conversion(format!("catalog: {}", e)))?
        .as_dict_mut()
        .map_err(|_| PdfError::Conversion("catalog not dict".to_string()))?;

    match catalog.get_mut(b"OutputIntents") {
        Ok(Object::Array(arr)) => {
            arr.push(Object::Reference(oi_id));
        }
        _ => {
            catalog.set(
                "OutputIntents",
                Object::Array(vec![Object::Reference(oi_id)]),
            );
        }
    }

    actions.push("Added sRGB OutputIntent with ICC profile".to_string());
    Ok(actions)
}
