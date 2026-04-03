//! XMP metadata writing for PDF/A conversion.

use lopdf::Object;

use crate::error::{PdfError, Result};
use crate::validation::PdfALevel;

/// Ensure the document has valid XMP metadata with PDF/A identification.
///
/// If XMP metadata already exists, it is replaced. Otherwise, a new metadata
/// stream is created and added to the catalog.
pub fn ensure_xmp_metadata(doc: &mut lopdf::Document, level: PdfALevel) -> Result<Vec<String>> {
    let mut actions = Vec::new();

    let (part, conformance) = match level {
        PdfALevel::A1b => ("1", "B"),
        PdfALevel::A1a => ("1", "A"),
        PdfALevel::A2b => ("2", "B"),
    };

    // Read existing Info dict for metadata fields
    let title = get_info_string(doc, b"Title").unwrap_or_default();
    let creator = get_info_string(doc, b"Creator").unwrap_or_else(|| "paperjam".to_string());
    let producer = get_info_string(doc, b"Producer").unwrap_or_else(|| "paperjam".to_string());

    let xmp = build_xmp_packet(part, conformance, &title, &creator, &producer);

    // Create XMP metadata stream
    let xmp_stream = lopdf::Stream::new(
        lopdf::dictionary! {
            "Type" => Object::Name(b"Metadata".to_vec()),
            "Subtype" => Object::Name(b"XML".to_vec()),
            "Length" => Object::Integer(xmp.len() as i64)
        },
        xmp.into_bytes(),
    );

    let xmp_id = doc.add_object(Object::Stream(xmp_stream));

    // Set /Metadata in catalog
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Conversion("No /Root".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Conversion("/Root not ref".to_string()))?;

    let catalog = doc
        .get_object_mut(catalog_id)
        .map_err(|e| PdfError::Conversion(format!("catalog get: {}", e)))?
        .as_dict_mut()
        .map_err(|_| PdfError::Conversion("catalog not dict".to_string()))?;

    catalog.set("Metadata", Object::Reference(xmp_id));
    actions.push(format!(
        "Set XMP metadata with pdfaid:part={}, pdfaid:conformance={}",
        part, conformance
    ));

    Ok(actions)
}

fn build_xmp_packet(
    part: &str,
    conformance: &str,
    title: &str,
    creator: &str,
    producer: &str,
) -> String {
    format!(
        r#"<?xpacket begin="{bom}" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:xmp="http://ns.adobe.com/xap/1.0/"
        xmlns:pdf="http://ns.adobe.com/pdf/1.3/"
        xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
      <pdfaid:part>{part}</pdfaid:part>
      <pdfaid:conformance>{conformance}</pdfaid:conformance>
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">{title}</rdf:li>
        </rdf:Alt>
      </dc:title>
      <xmp:CreatorTool>{creator}</xmp:CreatorTool>
      <pdf:Producer>{producer}</pdf:Producer>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
        bom = "\u{FEFF}",
        part = xml_escape(part),
        conformance = xml_escape(conformance),
        title = xml_escape(title),
        creator = xml_escape(creator),
        producer = xml_escape(producer),
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn get_info_string(doc: &lopdf::Document, key: &[u8]) -> Option<String> {
    let info_id = doc.trailer.get(b"Info").ok()?.as_reference().ok()?;
    let info_dict = doc.get_object(info_id).ok()?.as_dict().ok()?;
    match info_dict.get(key).ok()? {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        _ => None,
    }
}
