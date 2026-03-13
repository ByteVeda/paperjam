use crate::error::{PdfError, Result};

/// Extract and decompress the content stream(s) from a page dictionary.
pub fn get_page_content(doc: &lopdf::Document, page_dict: &lopdf::Dictionary) -> Result<Vec<u8>> {
    let contents = match page_dict.get(b"Contents") {
        Ok(contents) => contents,
        Err(_) => return Ok(Vec::new()),
    };

    let (_, contents) = doc.dereference(contents).unwrap_or((None, contents));

    match contents {
        lopdf::Object::Array(arr) => {
            let mut all_bytes = Vec::new();
            for item in arr {
                let (_, obj) = doc
                    .dereference(item)
                    .map_err(|_| {
                        PdfError::Structure("Failed to dereference content stream".into())
                    })?;
                if let Ok(stream) = obj.as_stream() {
                    let bytes = get_stream_bytes(stream)?;
                    all_bytes.extend_from_slice(&bytes);
                    all_bytes.push(b'\n');
                }
            }
            Ok(all_bytes)
        }
        lopdf::Object::Stream(ref stream) => get_stream_bytes(stream),
        lopdf::Object::Reference(id) => {
            let obj = doc
                .get_object(*id)
                .map_err(|_| PdfError::ObjectNotFound(id.0, id.1))?;
            if let Ok(stream) = obj.as_stream() {
                get_stream_bytes(stream)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn get_stream_bytes(stream: &lopdf::Stream) -> Result<Vec<u8>> {
    let mut stream = stream.clone();
    stream.decompress();
    Ok(stream.content)
}
