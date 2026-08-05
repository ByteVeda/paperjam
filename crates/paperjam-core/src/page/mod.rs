pub mod content;
pub mod resources;

use std::sync::Mutex;

use crate::error::{PdfError, Result};
use crate::table::{Table, TableExtractionOptions};
use crate::text::font::FontInfo;
use crate::text::layout::{TextLine, TextSpan};

/// A single page in a PDF document.
pub struct Page {
    pub number: u32,
    pub width: f64,
    pub height: f64,
    pub rotation: u32,
    content_bytes: Vec<u8>,
    fonts: Vec<FontInfo>,
    text_spans: Mutex<Option<Vec<TextSpan>>>,
    text_lines: Mutex<Option<Vec<TextLine>>>,
}

/// Helper: get f64 from a lopdf Object (handles both Integer and Real/f32).
pub(crate) fn obj_to_f64(obj: &lopdf::Object) -> std::result::Result<f64, lopdf::Error> {
    obj.as_float()
        .map(|f| f as f64)
        .or_else(|_| obj.as_i64().map(|i| i as f64))
}

impl Page {
    pub(crate) fn parse(
        doc: &lopdf::Document,
        number: u32,
        object_id: lopdf::ObjectId,
    ) -> Result<Self> {
        let page_obj = doc
            .get_object(object_id)
            .map_err(|_| PdfError::ObjectNotFound(object_id.0, object_id.1))?;
        let page_dict = page_obj
            .as_dict()
            .map_err(|_| PdfError::Structure("Page object is not a dictionary".into()))?;

        let media_box = Self::get_media_box(doc, page_dict)?;
        let width = media_box[2] - media_box[0];
        let height = media_box[3] - media_box[1];

        let rotation = page_dict
            .get(b"Rotate")
            .ok()
            .and_then(|v| {
                let (_, deref) = doc.dereference(v).ok()?;
                deref.as_i64().ok()
            })
            .unwrap_or(0) as u32;

        let fonts = resources::load_page_fonts(doc, page_dict)?;
        let content_bytes = content::get_page_content(doc, page_dict)?;

        Ok(Self {
            number,
            width,
            height,
            rotation,
            content_bytes,
            fonts,
            text_spans: Mutex::new(None),
            text_lines: Mutex::new(None),
        })
    }

    fn get_media_box(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> Result<[f64; 4]> {
        // Try to find MediaBox in this dict or by walking up the Parent chain.
        // We collect the ObjectId of the MediaBox array rather than a reference,
        // to avoid borrow-checker issues with local variables.
        let direct_mb = dict.get(b"MediaBox").ok();

        let mb_obj = if let Some(mb) = direct_mb {
            mb
        } else {
            // Walk parent chain to find inherited MediaBox
            let mut parent_ref = dict.get(b"Parent").ok();
            let mut found = None;
            while let Some(pref) = parent_ref {
                if let Ok((_, parent_obj)) = doc.dereference(pref) {
                    if let Ok(parent_dict) = parent_obj.as_dict() {
                        if let Ok(mb) = parent_dict.get(b"MediaBox") {
                            found = Some(mb);
                            break;
                        }
                        parent_ref = parent_dict.get(b"Parent").ok();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            found.ok_or_else(|| PdfError::Structure("No MediaBox found for page".into()))?
        };

        let (_, mb_deref) = doc.dereference(mb_obj).unwrap_or((None, mb_obj));
        let arr = mb_deref
            .as_array()
            .map_err(|_| PdfError::Structure("MediaBox is not an array".into()))?;

        if arr.len() < 4 {
            return Err(PdfError::Structure("MediaBox must have 4 elements".into()));
        }

        let nums: std::result::Result<Vec<f64>, _> = arr
            .iter()
            .take(4)
            .map(|v| {
                let (_, v) = doc.dereference(v).unwrap_or((None, v));
                obj_to_f64(v)
            })
            .collect();

        let nums =
            nums.map_err(|_| PdfError::Structure("MediaBox values must be numbers".into()))?;
        Ok([nums[0], nums[1], nums[2], nums[3]])
    }

    pub fn extract_text(&self) -> Result<String> {
        let lines = self.text_lines()?;
        Ok(lines
            .iter()
            .map(|l| l.text())
            .collect::<Vec<_>>()
            .join("\n"))
    }

    pub fn text_spans(&self) -> Result<Vec<TextSpan>> {
        {
            let cache = self.text_spans.lock().unwrap();
            if let Some(ref spans) = *cache {
                return Ok(spans.clone());
            }
        }
        let spans = self.compute_text_spans()?;
        let mut cache = self.text_spans.lock().unwrap();
        *cache = Some(spans.clone());
        Ok(spans)
    }

    pub fn text_lines(&self) -> Result<Vec<TextLine>> {
        {
            let cache = self.text_lines.lock().unwrap();
            if let Some(ref lines) = *cache {
                return Ok(lines.clone());
            }
        }
        let spans = self.text_spans()?;
        let lines = TextLine::group_from_spans(&spans);
        let mut cache = self.text_lines.lock().unwrap();
        *cache = Some(lines.clone());
        Ok(lines)
    }

    pub fn extract_tables(&self, options: &TableExtractionOptions) -> Result<Vec<Table>> {
        crate::table::extract_tables(self, options)
    }

    pub fn content_bytes(&self) -> &[u8] {
        &self.content_bytes
    }

    pub fn fonts(&self) -> &[FontInfo] {
        &self.fonts
    }

    fn compute_text_spans(&self) -> Result<Vec<TextSpan>> {
        crate::text::extract_spans(&self.content_bytes, &self.fonts)
    }
}
