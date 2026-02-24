use crate::document::Document;
use crate::error::{PdfError, Result};

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
}

impl Rotation {
    pub fn as_degrees(&self) -> i32 {
        match self {
            Self::Degrees0 => 0,
            Self::Degrees90 => 90,
            Self::Degrees180 => 180,
            Self::Degrees270 => 270,
        }
    }
}

/// Rotate specific pages in a document.
pub fn rotate_pages(doc: &mut Document, page_rotations: &[(u32, Rotation)]) -> Result<()> {
    let page_map = doc.inner().get_pages();

    for (page_number, rotation) in page_rotations {
        let object_id = page_map.get(page_number).ok_or(PdfError::PageOutOfRange {
            page: *page_number as usize,
            total: page_map.len(),
        })?;

        let page_obj = doc
            .inner_mut()
            .get_object_mut(*object_id)
            .map_err(|_| PdfError::ObjectNotFound(object_id.0, object_id.1))?;

        if let Ok(dict) = page_obj.as_dict_mut() {
            dict.set("Rotate", lopdf::Object::Integer(rotation.as_degrees() as i64));
        }
    }

    Ok(())
}

/// Rotate all pages in a document.
pub fn rotate_all(doc: &mut Document, rotation: Rotation) -> Result<()> {
    let pages: Vec<u32> = (1..=doc.page_count() as u32).collect();
    let rotations: Vec<(u32, Rotation)> = pages.into_iter().map(|p| (p, rotation)).collect();
    rotate_pages(doc, &rotations)
}
