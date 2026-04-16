use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::TocEntry;

/// Parse an NCX (EPUB 2) table of contents.
pub fn parse_ncx(xml: &str) -> Vec<TocEntry> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut entries = Vec::new();

    // Find <navMap> then parse <navPoint> recursively.
    let mut in_nav_map = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref());
                if local == "navMap" {
                    in_nav_map = true;
                }
                if in_nav_map && local == "navPoint" {
                    if let Some(entry) = parse_nav_point(&mut reader) {
                        entries.push(entry);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = local_name(e.name().as_ref());
                if local == "navMap" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    flatten_toc(&entries, 0)
}

fn parse_nav_point(reader: &mut Reader<&[u8]>) -> Option<TocEntry> {
    let mut buf = Vec::new();
    let mut title = String::new();
    let mut href = String::new();
    let mut children = Vec::new();
    let mut in_text = false;
    let mut _depth = 1u32; // Track nesting depth.

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "navPoint" => {
                        _depth += 1;
                        if let Some(child) = parse_nav_point(reader) {
                            children.push(child);
                        }
                        _depth -= 1;
                    }
                    "text" => in_text = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = local_name(e.name().as_ref());
                if local == "content" {
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        if key == "src" {
                            href = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                title = e.unescape().unwrap_or_default().trim().to_string();
            }
            Ok(Event::End(ref e)) => {
                let local = local_name(e.name().as_ref());
                if local == "text" {
                    in_text = false;
                }
                if local == "navPoint" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if title.is_empty() && href.is_empty() {
        return None;
    }

    Some(TocEntry {
        title,
        href,
        level: 0, // Will be set during flattening.
        children,
    })
}

/// Parse a nav.xhtml (EPUB 3) table of contents.
pub fn parse_nav_xhtml(html_bytes: &[u8]) -> Vec<TocEntry> {
    let text = match std::str::from_utf8(html_bytes) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let dom = paperjam_html::scraper::Html::parse_document(text);
    let nav_sel = match paperjam_html::scraper::Selector::parse("nav") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // Find the nav element (preferably with epub:type="toc").
    let nav_el = dom
        .select(&nav_sel)
        .find(|el| {
            el.value()
                .attr("epub:type")
                .map(|t| t == "toc")
                .unwrap_or(false)
        })
        .or_else(|| dom.select(&nav_sel).next());

    let Some(nav_el) = nav_el else {
        return Vec::new();
    };

    // Find the <ol> inside nav.
    let ol_sel = match paperjam_html::scraper::Selector::parse("ol") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let Some(ol_el) = nav_el.select(&ol_sel).next() else {
        return Vec::new();
    };

    let entries = parse_nav_ol(ol_el);
    flatten_toc(&entries, 0)
}

fn parse_nav_ol(ol: paperjam_html::scraper::ElementRef) -> Vec<TocEntry> {
    let mut entries = Vec::new();

    for child in ol.children() {
        let Some(li) = paperjam_html::scraper::ElementRef::wrap(child) else {
            continue;
        };
        if !li.value().name().eq_ignore_ascii_case("li") {
            continue;
        }

        let mut title = String::new();
        let mut href = String::new();
        let mut children = Vec::new();

        for li_child in li.children() {
            let Some(el) = paperjam_html::scraper::ElementRef::wrap(li_child) else {
                continue;
            };
            let tag = el.value().name().to_ascii_lowercase();
            match tag.as_str() {
                "a" => {
                    title = el.text().collect::<String>().trim().to_string();
                    href = el.value().attr("href").unwrap_or("").to_string();
                }
                "ol" => {
                    children = parse_nav_ol(el);
                }
                _ => {}
            }
        }

        if !title.is_empty() || !href.is_empty() {
            entries.push(TocEntry {
                title,
                href,
                level: 0,
                children,
            });
        }
    }

    entries
}

/// Flatten a tree of TocEntry into a list with proper levels.
fn flatten_toc(entries: &[TocEntry], level: usize) -> Vec<TocEntry> {
    let mut result = Vec::new();
    for entry in entries {
        result.push(TocEntry {
            title: entry.title.clone(),
            href: entry.href.clone(),
            level,
            children: Vec::new(), // Flattened — no children.
        });
        result.extend(flatten_toc(&entry.children, level + 1));
    }
    result
}

fn local_name(name: &[u8]) -> String {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rsplit_once(':')
        .map(|(_, local)| local)
        .unwrap_or(s)
        .to_string()
}
