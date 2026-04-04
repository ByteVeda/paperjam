"""Manipulation tools: split, merge, reorder, delete, insert, rotate, optimize."""

from __future__ import annotations

import json

import paperjam
from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def split_document(session_id: str, ranges: list[list[int]]) -> str:
    """Split a PDF into multiple documents by page ranges (1-indexed, inclusive).

    Each range is [start, end]. Example: [[1, 3], [4, 6]] splits into two docs.
    Creates a new session for each resulting part.
    """
    _session, doc = session_manager.get_pdf(session_id)
    range_tuples = [(r[0], r[1]) for r in ranges]
    parts = doc.split(range_tuples)

    new_sessions = []
    for i, part in enumerate(parts):
        new_id = session_manager.register(part, fmt="pdf")
        new_sessions.append(
            {
                "session_id": new_id,
                "page_range": ranges[i],
                "page_count": part.page_count,
            }
        )
    return json.dumps({"parts": new_sessions, "count": len(new_sessions)})


@mcp.tool()
@handle_errors
def merge_documents(session_ids: list[str], deduplicate_resources: bool = False) -> str:
    """Merge multiple PDF documents into one. Creates a new session.

    Documents are merged in the order provided.
    """
    docs = []
    for sid in session_ids:
        _session, doc = session_manager.get_pdf(sid)
        docs.append(doc)

    merged = paperjam.merge(docs, deduplicate_resources=deduplicate_resources)
    new_id = session_manager.register(merged, fmt="pdf")
    return json.dumps(
        {
            "session_id": new_id,
            "page_count": merged.page_count,
            "source_count": len(session_ids),
        }
    )


@mcp.tool()
@handle_errors
def reorder_pages(session_id: str, page_order: list[int]) -> str:
    """Reorder, subset, or duplicate pages in a PDF. Modifies the session in-place.

    page_order is a list of 1-indexed page numbers in desired order.
    Pages can be repeated (duplicated) or omitted (removed).
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc = doc.reorder(page_order)
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"page_count": new_doc.page_count, "page_order": page_order})


@mcp.tool()
@handle_errors
def delete_pages(session_id: str, page_numbers: list[int]) -> str:
    """Delete specific pages from a PDF (1-indexed). Modifies the session in-place."""
    _session, doc = session_manager.get_pdf(session_id)
    new_doc = doc.delete_pages(page_numbers)
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"deleted": page_numbers, "page_count": new_doc.page_count})


@mcp.tool()
@handle_errors
def insert_blank_pages(session_id: str, positions: list[list[float]]) -> str:
    """Insert blank pages at specified positions. Modifies the session in-place.

    Each position is [after_page, width, height] where after_page=0 means the beginning.
    Width and height are in PDF points (72 points = 1 inch).
    Example: [[0, 612, 792]] inserts a US Letter page at the beginning.
    """
    _session, doc = session_manager.get_pdf(session_id)
    pos_tuples = [(int(p[0]), p[1], p[2]) for p in positions]
    new_doc = doc.insert_blank_pages(pos_tuples)
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"page_count": new_doc.page_count, "inserted": len(positions)})


@mcp.tool()
@handle_errors
def rotate_pages(session_id: str, page_rotations: list[list[int]]) -> str:
    """Rotate specific pages in a PDF. Modifies the session in-place.

    Each entry is [page_number, degrees] where degrees is 0, 90, 180, or 270.
    """
    _session, doc = session_manager.get_pdf(session_id)
    rot_tuples: list[tuple[int, int]] = [(r[0], r[1]) for r in page_rotations]
    new_doc = doc.rotate(rot_tuples)  # type: ignore[arg-type]
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"rotated": len(page_rotations)})


@mcp.tool()
@handle_errors
def optimize_document(
    session_id: str,
    compress_streams: bool = True,
    remove_unused: bool = True,
    remove_duplicates: bool = True,
    strip_metadata: bool = False,
) -> str:
    """Optimize a PDF to reduce file size. Modifies the session in-place.

    Returns size reduction statistics.
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, result = doc.optimize(
        compress_streams=compress_streams,
        remove_unused=remove_unused,
        remove_duplicates=remove_duplicates,
        strip_metadata=strip_metadata,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))
