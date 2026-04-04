"""Metadata tools: set_metadata, set_bookmarks, generate_toc."""

from __future__ import annotations

import json

import paperjam
from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def set_metadata(
    session_id: str,
    title: str | None = None,
    author: str | None = None,
    subject: str | None = None,
    keywords: str | None = None,
    creator: str | None = None,
    producer: str | None = None,
) -> str:
    """Update document metadata fields. Modifies the session in-place.

    Only provided fields are changed; omitted fields remain unchanged.
    """
    _session, doc = session_manager.get_pdf(session_id)
    kwargs = {}
    if title is not None:
        kwargs["title"] = title
    if author is not None:
        kwargs["author"] = author
    if subject is not None:
        kwargs["subject"] = subject
    if keywords is not None:
        kwargs["keywords"] = keywords
    if creator is not None:
        kwargs["creator"] = creator
    if producer is not None:
        kwargs["producer"] = producer

    new_doc = doc.set_metadata(**kwargs)
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"metadata_updated": True, "fields": list(kwargs.keys())})


@mcp.tool()
@handle_errors
def set_bookmarks(session_id: str, bookmarks: list[dict]) -> str:
    """Replace the document's bookmarks/TOC. Modifies the session in-place.

    Each bookmark is {"title": str, "page": int, "children": [...]}.
    Pass an empty list to remove all bookmarks.
    """
    _session, doc = session_manager.get_pdf(session_id)

    def _to_bookmark(d: dict) -> paperjam.Bookmark:
        children = tuple(_to_bookmark(c) for c in d.get("children", []))
        return paperjam.Bookmark(title=d["title"], page=d["page"], level=1, children=children)

    bm_objects = [_to_bookmark(b) for b in bookmarks]
    new_doc = doc.set_bookmarks(bm_objects)
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"bookmarks_set": True, "count": len(bookmarks)})


@mcp.tool()
@handle_errors
def generate_toc(
    session_id: str,
    max_depth: int = 6,
    heading_size_ratio: float = 1.2,
    layout_aware: bool = False,
    replace_existing: bool = True,
) -> str:
    """Auto-generate a table of contents from heading structure. Modifies the session in-place.

    Returns the generated bookmark entries.
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, bm_list = doc.generate_toc(
        max_depth=max_depth,
        heading_size_ratio=heading_size_ratio,
        layout_aware=layout_aware,
        replace_existing=replace_existing,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"bookmarks": serialize(bm_list), "count": len(bm_list)})
