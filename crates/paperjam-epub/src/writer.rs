use std::io::Write;

use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;

use crate::error::EpubError;

/// Generate a valid EPUB 3 archive from content blocks and metadata.
pub fn generate_epub_bytes(
    blocks: &[ContentBlock],
    metadata: &Metadata,
    bookmarks: &[BookmarkItem],
) -> Result<Vec<u8>, EpubError> {
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);

    // 1. mimetype — must be first entry, stored (not compressed).
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", options)?;
    zip.write_all(b"application/epub+zip")?;

    let deflate = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflate)?;
    zip.write_all(CONTAINER_XML)?;

    // 3. Split blocks into chapters (split at Heading level 1, or single chapter).
    let chapters = split_into_chapters(blocks);

    // 4. content.opf
    let opf = build_opf(metadata, chapters.len());
    zip.start_file("OEBPS/content.opf", deflate)?;
    zip.write_all(opf.as_bytes())?;

    // 5. nav.xhtml (EPUB 3 navigation)
    let nav = build_nav(&chapters, bookmarks);
    zip.start_file("OEBPS/nav.xhtml", deflate)?;
    zip.write_all(nav.as_bytes())?;

    // 6. Chapter XHTML files.
    for (i, chapter_blocks) in chapters.iter().enumerate() {
        let chapter_html = paperjam_html::writer::generate_html_bytes(
            chapter_blocks,
            &[], // Tables are embedded in blocks.
            metadata,
        );
        let path = format!("OEBPS/chapter_{}.xhtml", i + 1);
        zip.start_file(&path, deflate)?;
        zip.write_all(&chapter_html)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

/// Split blocks into chapters by H1 headings.
fn split_into_chapters(blocks: &[ContentBlock]) -> Vec<Vec<ContentBlock>> {
    let mut chapters: Vec<Vec<ContentBlock>> = Vec::new();
    let mut current = Vec::new();

    for block in blocks {
        if let ContentBlock::Heading { level: 1, .. } = block {
            if !current.is_empty() {
                chapters.push(current);
                current = Vec::new();
            }
        }
        current.push(block.clone());
    }

    if !current.is_empty() {
        chapters.push(current);
    }

    // If no chapters were created, wrap everything in one.
    if chapters.is_empty() && !blocks.is_empty() {
        chapters.push(blocks.to_vec());
    }

    chapters
}

fn build_opf(metadata: &Metadata, chapter_count: usize) -> String {
    let title = metadata.title.as_deref().unwrap_or("Document");
    let author = metadata.author.as_deref().unwrap_or("Unknown");

    let mut items = String::new();
    let mut spine = String::new();

    items.push_str("    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n");

    for i in 1..=chapter_count {
        items.push_str(&format!(
            "    <item id=\"chapter_{i}\" href=\"chapter_{i}.xhtml\" media-type=\"application/xhtml+xml\"/>\n"
        ));
        spine.push_str(&format!("    <itemref idref=\"chapter_{i}\"/>\n"));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">urn:uuid:00000000-0000-0000-0000-000000000000</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{author}</dc:creator>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">2024-01-01T00:00:00Z</meta>
  </metadata>
  <manifest>
{items}  </manifest>
  <spine>
{spine}  </spine>
</package>
"#,
        title = escape_xml(title),
        author = escape_xml(author),
        items = items,
        spine = spine,
    )
}

fn build_nav(chapters: &[Vec<ContentBlock>], bookmarks: &[BookmarkItem]) -> String {
    let mut toc_items = String::new();

    if !bookmarks.is_empty() {
        // Use bookmarks for navigation.
        for bm in bookmarks {
            let chapter_idx = bm.page.min(chapters.len());
            let href = if chapter_idx > 0 {
                format!("chapter_{}.xhtml", chapter_idx)
            } else {
                "chapter_1.xhtml".to_string()
            };
            toc_items.push_str(&format!(
                "      <li><a href=\"{}\">{}</a></li>\n",
                href,
                escape_xml(&bm.title)
            ));
        }
    } else {
        // Generate from chapter titles (first heading in each chapter).
        for (i, chapter_blocks) in chapters.iter().enumerate() {
            let title = chapter_blocks
                .iter()
                .find_map(|b| {
                    if let ContentBlock::Heading { text, .. } = b {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| format!("Chapter {}", i + 1));
            toc_items.push_str(&format!(
                "      <li><a href=\"chapter_{}.xhtml\">{}</a></li>\n",
                i + 1,
                escape_xml(&title)
            ));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
  <title>Table of Contents</title>
</head>
<body>
  <nav epub:type="toc">
    <h1>Table of Contents</h1>
    <ol>
{toc_items}    </ol>
  </nav>
</body>
</html>
"#,
        toc_items = toc_items,
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const CONTAINER_XML: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;
