"""Page-level tools: extract text, tables, structure, layout, markdown from specific pages."""

from __future__ import annotations

import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def page_get_info(session_id: str, page_number: int) -> str:
    """Get information about a specific page: number, width, height, rotation."""
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    return json.dumps(serialize(page.info))


@mcp.tool()
@handle_errors
def page_extract_text(session_id: str, page_number: int) -> str:
    """Extract all text from a specific page."""
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    return str(page.extract_text())


@mcp.tool()
@handle_errors
def page_extract_tables(
    session_id: str,
    page_number: int,
    strategy: str = "auto",
    min_rows: int = 2,
    min_cols: int = 2,
) -> str:
    """Extract tables from a specific page.

    strategy: auto, lattice, or stream.
    """
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    tables = page.extract_tables(strategy=strategy, min_rows=min_rows, min_cols=min_cols)
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
    return json.dumps({"tables": result, "count": len(result), "page": page_number})


@mcp.tool()
@handle_errors
def page_extract_structure(
    session_id: str,
    page_number: int,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> str:
    """Extract structured content (headings, paragraphs, lists, tables) from a specific page."""
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    blocks = page.extract_structure(
        heading_size_ratio=heading_size_ratio,
        detect_lists=detect_lists,
        include_tables=include_tables,
        layout_aware=layout_aware,
    )
    return json.dumps({"blocks": serialize(blocks), "count": len(blocks), "page": page_number})


@mcp.tool()
@handle_errors
def page_analyze_layout(
    session_id: str,
    page_number: int,
    min_gutter_width: float = 20.0,
    max_columns: int = 4,
    detect_headers_footers: bool = True,
) -> str:
    """Analyze the layout of a specific page: detect columns, headers, and footers."""
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    layout = page.analyze_layout(
        min_gutter_width=min_gutter_width,
        max_columns=max_columns,
        detect_headers_footers=detect_headers_footers,
    )
    return json.dumps(serialize(layout))


@mcp.tool()
@handle_errors
def page_to_markdown(
    session_id: str,
    page_number: int,
    heading_offset: int = 0,
    html_tables: bool = False,
    layout_aware: bool = False,
) -> str:
    """Convert a specific page to Markdown text."""
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[page_number - 1]
    return str(
        page.to_markdown(
            heading_offset=heading_offset,
            html_tables=html_tables,
            layout_aware=layout_aware,
        )
    )
