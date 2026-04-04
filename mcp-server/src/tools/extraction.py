"""Extraction tools: text, tables, structure, markdown, search, links, images, bookmarks."""

from __future__ import annotations

import json

import paperjam
from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def extract_text(session_id: str) -> str:
    """Extract all text from a document.

    Works with both PDF and non-PDF formats.
    """
    session = session_manager.get(session_id)
    doc = session.document
    if isinstance(doc, paperjam.Document):
        parts = [page.extract_text() for page in doc.pages]
        return "\n\n".join(parts)
    return doc.extract_text()


@mcp.tool()
@handle_errors
def extract_tables(
    session_id: str,
    strategy: str = "auto",
    min_rows: int = 2,
    min_cols: int = 2,
) -> str:
    """Extract tables from a document as structured data.

    For PDFs, strategy can be 'auto', 'lattice', or 'stream'.
    """
    session = session_manager.get(session_id)
    doc = session.document
    if isinstance(doc, paperjam.Document):
        tables = doc.extract_tables(strategy=strategy, min_rows=min_rows, min_cols=min_cols)
    else:
        tables = doc.extract_tables()
    result = []
    for t in tables:
        result.append(
            {
                "rows": t.to_list(),
                "row_count": t.row_count,
                "col_count": t.col_count,
                "strategy": t.strategy,
            }
        )
    return json.dumps({"tables": result, "count": len(result)})


@mcp.tool()
@handle_errors
def extract_structure(
    session_id: str,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> str:
    """Extract document structure (headings, paragraphs, lists, tables).

    For PDFs, supports advanced options. For other formats, uses default extraction.
    """
    session = session_manager.get(session_id)
    doc = session.document
    if isinstance(doc, paperjam.Document):
        blocks = doc.extract_structure(
            heading_size_ratio=heading_size_ratio,
            detect_lists=detect_lists,
            include_tables=include_tables,
            layout_aware=layout_aware,
        )
    else:
        blocks = doc.extract_structure()
    return json.dumps({"blocks": serialize(blocks), "count": len(blocks)})


@mcp.tool()
@handle_errors
def to_markdown(
    session_id: str,
    heading_offset: int = 0,
    include_page_numbers: bool = False,
    html_tables: bool = False,
    layout_aware: bool = False,
) -> str:
    """Convert a document to Markdown text.

    Works with both PDF and non-PDF formats. PDFs support additional options.
    """
    session = session_manager.get(session_id)
    doc = session.document
    if isinstance(doc, paperjam.Document):
        return doc.to_markdown(
            heading_offset=heading_offset,
            include_page_numbers=include_page_numbers,
            html_tables=html_tables,
            layout_aware=layout_aware,
        )
    return doc.to_markdown()


@mcp.tool()
@handle_errors
def search_document(
    session_id: str,
    query: str,
    case_sensitive: bool = True,
    max_results: int = 0,
    use_regex: bool = False,
) -> str:
    """Search for text across all pages. Supports regex. PDF only.

    Set max_results=0 for unlimited results.
    """
    _session, doc = session_manager.get_pdf(session_id)
    results = doc.search(query, case_sensitive=case_sensitive, max_results=max_results, use_regex=use_regex)
    return json.dumps({"results": serialize(results), "count": len(results)})


@mcp.tool()
@handle_errors
def extract_links(session_id: str) -> str:
    """Extract all hyperlinks from a PDF document."""
    _session, doc = session_manager.get_pdf(session_id)
    links = doc.extract_links()
    return json.dumps({"links": serialize(links), "count": len(links)})


@mcp.tool()
@handle_errors
def extract_images(session_id: str, page_number: int) -> str:
    """Extract image metadata from a specific page. PDF only.

    Returns image dimensions and properties (not raw image bytes).
    """
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    images = page.extract_images()
    result = []
    for img in images:
        result.append(
            {
                "width": img.width,
                "height": img.height,
                "color_space": img.color_space,
                "bits_per_component": img.bits_per_component,
                "filters": img.filters,
                "data_size": len(img.data),
            }
        )
    return json.dumps({"images": result, "count": len(result), "page": page_number})


@mcp.tool()
@handle_errors
def extract_bookmarks(session_id: str) -> str:
    """Extract the bookmark/table of contents tree from a document."""
    session = session_manager.get(session_id)
    bookmarks = session.document.bookmarks
    return json.dumps({"bookmarks": serialize(bookmarks), "count": len(bookmarks)})
