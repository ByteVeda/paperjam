/// Extract XMP metadata from a PDF document, if present.
pub fn extract_xmp(doc: &lopdf::Document) -> Option<String> {
    let catalog = doc.catalog().ok()?;
    let meta_ref = catalog.get(b"Metadata").ok()?;
    let (_, meta_obj) = doc.dereference(meta_ref).ok()?;
    let stream = meta_obj.as_stream().ok()?;
    let mut stream = stream.clone();
    stream.decompress();
    String::from_utf8(stream.content).ok()
}
