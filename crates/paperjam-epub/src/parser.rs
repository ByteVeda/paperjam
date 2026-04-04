use std::collections::HashMap;
use std::io::Read;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::{ChapterData, EpubDocument, OpfMetadata, TocEntry};
use crate::error::{EpubError, Result};
use crate::toc;

/// Parse an EPUB document from raw bytes.
pub fn parse_epub(bytes: &[u8]) -> Result<EpubDocument> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // 1. Find the OPF path from container.xml.
    let container_xml = read_zip_entry_string(&mut archive, "META-INF/container.xml")?;
    let opf_path = parse_container_xml(&container_xml)?;
    let opf_base_dir = opf_path
        .rsplit_once('/')
        .map(|(d, _)| d.to_string())
        .unwrap_or_default();

    // 2. Parse OPF: metadata, manifest, spine.
    let opf_xml = read_zip_entry_string(&mut archive, &opf_path)?;
    let (opf_metadata, manifest, spine) = parse_opf(&opf_xml)?;

    // 3. Parse TOC.
    let toc_entries = parse_toc_from_manifest(&mut archive, &manifest, &opf_base_dir);

    // 4. Read chapters in spine order.
    let mut chapters = Vec::new();
    for (idx, spine_idref) in spine.iter().enumerate() {
        if let Some(href) = manifest.get(spine_idref) {
            let full_path = resolve_path(&opf_base_dir, href);
            match read_zip_entry_bytes(&mut archive, &full_path) {
                Ok(html_bytes) => {
                    let html_doc = paperjam_html::HtmlDocument::from_bytes(&html_bytes)?;
                    let title = find_toc_title(&toc_entries, href);
                    chapters.push(ChapterData {
                        index: idx,
                        title,
                        href: href.clone(),
                        html: html_doc,
                    });
                }
                Err(_) => {
                    // Skip chapters we can't read.
                }
            }
        }
    }

    // 5. Collect archive images.
    let archive_images = collect_images(&mut archive, &manifest, &opf_base_dir);

    Ok(EpubDocument {
        chapters,
        opf_metadata,
        toc_entries,
        raw_bytes: bytes.to_vec(),
        archive_images,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_zip_entry_string(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
) -> Result<String> {
    let mut file = archive
        .by_name(name)
        .map_err(|_| EpubError::MissingEntry(name.to_string()))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn read_zip_entry_bytes(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
) -> Result<Vec<u8>> {
    let mut file = archive
        .by_name(name)
        .map_err(|_| EpubError::MissingEntry(name.to_string()))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn resolve_path(base_dir: &str, href: &str) -> String {
    if base_dir.is_empty() {
        href.to_string()
    } else {
        format!("{}/{}", base_dir, href)
    }
}

/// Parse `META-INF/container.xml` to find the OPF path.
fn parse_container_xml(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref());
                if local == "rootfile" {
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        if key == "full-path" {
                            return Ok(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        buf.clear();
    }

    Err(EpubError::InvalidStructure(
        "no rootfile found in container.xml".to_string(),
    ))
}

/// Parse the OPF package file.
///
/// Returns (metadata, manifest {id -> href}, spine [idrefs]).
fn parse_opf(xml: &str) -> Result<(OpfMetadata, HashMap<String, String>, Vec<String>)> {
    let mut metadata = OpfMetadata::default();
    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut spine: Vec<String> = Vec::new();

    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    // State: which section are we in?
    #[derive(PartialEq)]
    enum Section {
        None,
        Metadata,
        Manifest,
        Spine,
    }
    let mut section = Section::None;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "metadata" => section = Section::Metadata,
                    "manifest" => section = Section::Manifest,
                    "spine" => section = Section::Spine,
                    _ => {}
                }
                if section == Section::Metadata {
                    current_tag = local.clone();
                }
                if section == Section::Manifest && local == "item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "id" => id = val,
                            "href" => href = val,
                            _ => {}
                        }
                    }
                    if !id.is_empty() && !href.is_empty() {
                        manifest.insert(id, href);
                    }
                }
                if section == Section::Spine && local == "itemref" {
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        if key == "idref" {
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            spine.push(val);
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = local_name(e.name().as_ref());
                if section == Section::Manifest && local == "item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "id" => id = val,
                            "href" => href = val,
                            _ => {}
                        }
                    }
                    if !id.is_empty() && !href.is_empty() {
                        manifest.insert(id, href);
                    }
                }
                if section == Section::Spine && local == "itemref" {
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        if key == "idref" {
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            spine.push(val);
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if section == Section::Metadata {
                    let text = e.unescape().unwrap_or_default().trim().to_string();
                    if !text.is_empty() {
                        match current_tag.as_str() {
                            "title" => metadata.title = Some(text),
                            "creator" => metadata.creator = Some(text),
                            "subject" => metadata.subject = Some(text),
                            "description" => metadata.description = Some(text),
                            "publisher" => metadata.publisher = Some(text),
                            "date" => metadata.date = Some(text),
                            "language" => metadata.language = Some(text),
                            "identifier" => metadata.identifier = Some(text),
                            "rights" => metadata.rights = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "metadata" | "manifest" | "spine" => section = Section::None,
                    _ => {}
                }
                if section == Section::Metadata {
                    current_tag.clear();
                }
                // Handle <itemref> as non-self-closing in some EPUBs.
                if section == Section::Spine && local == "itemref" {
                    // Already handled in Start/Empty.
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        buf.clear();
    }

    Ok((metadata, manifest, spine))
}

/// Find TOC title for a given href (matching by filename).
fn find_toc_title(entries: &[TocEntry], href: &str) -> Option<String> {
    let href_clean = href.split('#').next().unwrap_or(href);
    for entry in entries {
        let entry_href_clean = entry.href.split('#').next().unwrap_or(&entry.href);
        if entry_href_clean == href_clean || entry_href_clean.ends_with(href_clean) {
            return Some(entry.title.clone());
        }
        if let Some(t) = find_toc_title(&entry.children, href) {
            return Some(t);
        }
    }
    None
}

/// Parse TOC from the manifest, trying NCX first then nav.xhtml.
fn parse_toc_from_manifest(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    manifest: &HashMap<String, String>,
    opf_base_dir: &str,
) -> Vec<TocEntry> {
    // Look for NCX file (usually id="ncx" or ends with .ncx).
    for (id, href) in manifest {
        if id == "ncx" || href.ends_with(".ncx") {
            let full_path = resolve_path(opf_base_dir, href);
            if let Ok(xml) = read_zip_entry_string(archive, &full_path) {
                let entries = toc::parse_ncx(&xml);
                if !entries.is_empty() {
                    return entries;
                }
            }
        }
    }

    // Look for nav.xhtml (EPUB 3).
    for href in manifest.values() {
        if href.contains("nav") && (href.ends_with(".xhtml") || href.ends_with(".html")) {
            let full_path = resolve_path(opf_base_dir, href);
            if let Ok(html_bytes) = read_zip_entry_bytes(archive, &full_path) {
                let entries = toc::parse_nav_xhtml(&html_bytes);
                if !entries.is_empty() {
                    return entries;
                }
            }
        }
    }

    Vec::new()
}

/// Collect image files from the archive based on manifest media types.
fn collect_images(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    manifest: &HashMap<String, String>,
    opf_base_dir: &str,
) -> Vec<(String, Vec<u8>)> {
    let image_extensions = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"];
    let mut images = Vec::new();

    for href in manifest.values() {
        let lower = href.to_ascii_lowercase();
        if image_extensions.iter().any(|ext| lower.ends_with(ext)) {
            let full_path = resolve_path(opf_base_dir, href);
            if let Ok(data) = read_zip_entry_bytes(archive, &full_path) {
                images.push((href.clone(), data));
            }
        }
    }

    images
}

/// Strip namespace prefix from a tag name.
fn local_name(name: &[u8]) -> String {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rsplit_once(':')
        .map(|(_, local)| local)
        .unwrap_or(s)
        .to_string()
}
