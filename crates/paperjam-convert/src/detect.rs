use paperjam_model::format::DocumentFormat;
use std::path::Path;

/// Detect format from a file path (extension first, then magic bytes).
pub fn detect_format(path: &Path) -> DocumentFormat {
    let from_ext = DocumentFormat::detect(path);
    if from_ext != DocumentFormat::Unknown {
        return from_ext;
    }
    if let Ok(bytes) = std::fs::read(path) {
        return detect_format_bytes(&bytes);
    }
    DocumentFormat::Unknown
}

/// Detect format from raw bytes using magic bytes.
pub fn detect_format_bytes(bytes: &[u8]) -> DocumentFormat {
    // PDF: starts with %PDF
    if bytes.len() >= 4 && &bytes[..4] == b"%PDF" {
        return DocumentFormat::Pdf;
    }
    // ZIP-based formats: starts with PK\x03\x04
    if bytes.len() >= 4 && &bytes[..4] == b"PK\x03\x04" {
        return detect_zip_format(bytes);
    }
    // HTML detection: check for common HTML markers.
    if looks_like_html(bytes) {
        return DocumentFormat::Html;
    }
    DocumentFormat::Unknown
}

/// Distinguish between DOCX, XLSX, PPTX, and EPUB by inspecting ZIP contents.
fn detect_zip_format(bytes: &[u8]) -> DocumentFormat {
    let cursor = std::io::Cursor::new(bytes);
    if let Ok(archive) = zip::ZipArchive::new(cursor) {
        // Check for EPUB first (has "mimetype" entry with "application/epub+zip").
        if archive.index_for_name("mimetype").is_some() {
            // Read mimetype to confirm.
            let cursor2 = std::io::Cursor::new(bytes);
            if let Ok(mut archive2) = zip::ZipArchive::new(cursor2) {
                if let Ok(mut file) = archive2.by_name("mimetype") {
                    let mut buf = String::new();
                    if std::io::Read::read_to_string(&mut file, &mut buf).is_ok()
                        && buf.trim() == "application/epub+zip"
                    {
                        return DocumentFormat::Epub;
                    }
                }
            }
        }

        // OOXML detection by directory structure.
        for i in 0..archive.len() {
            if let Some(name) = archive.name_for_index(i) {
                let lower = name.to_lowercase();
                if lower.starts_with("word/") {
                    return DocumentFormat::Docx;
                }
                if lower.starts_with("xl/") {
                    return DocumentFormat::Xlsx;
                }
                if lower.starts_with("ppt/") {
                    return DocumentFormat::Pptx;
                }
            }
        }
    }
    DocumentFormat::Unknown
}

/// Heuristic to detect HTML content from raw bytes.
fn looks_like_html(bytes: &[u8]) -> bool {
    // Check first 1024 bytes for common HTML markers.
    let check_len = bytes.len().min(1024);
    let text = String::from_utf8_lossy(&bytes[..check_len]);
    let lower = text.to_ascii_lowercase();

    lower.contains("<!doctype html")
        || lower.contains("<html")
        || lower.contains("<head")
        || lower.contains("<body")
}
