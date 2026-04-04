"""MCP resource templates for browsing open document sessions."""

from __future__ import annotations

import json

import paperjam
from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.resource("document://{session_id}")
@handle_errors
def document_overview(session_id: str) -> str:
    """Overview of an open document: format, page count, metadata summary."""
    session = session_manager.get(session_id)
    doc = session.document
    return json.dumps(
        {
            "session_id": session_id,
            "format": session.format,
            "is_pdf": session.is_pdf,
            "page_count": doc.page_count,
            "path": session.path,
            "metadata": serialize(doc.metadata),
        }
    )


@mcp.resource("document://{session_id}/metadata")
@handle_errors
def document_metadata(session_id: str) -> str:
    """Full document metadata."""
    session = session_manager.get(session_id)
    return json.dumps(serialize(session.document.metadata))


@mcp.resource("document://{session_id}/text")
@handle_errors
def document_text(session_id: str) -> str:
    """Full text content of the document."""
    session = session_manager.get(session_id)
    doc = session.document
    if isinstance(doc, paperjam.Document):
        parts = [page.extract_text() for page in doc.pages]
        return "\n\n".join(parts)
    return doc.extract_text()


@mcp.resource("document://{session_id}/pages/{page_number}")
@handle_errors
def document_page(session_id: str, page_number: str) -> str:
    """Page info and text content for a specific page (PDF only)."""
    _session, doc = session_manager.get_pdf(session_id)
    page = doc.pages[int(page_number) - 1]
    return json.dumps(
        {
            "info": serialize(page.info),
            "text": page.extract_text(),
        }
    )


@mcp.resource("document://{session_id}/bookmarks")
@handle_errors
def document_bookmarks(session_id: str) -> str:
    """Bookmark/TOC tree of the document."""
    session = session_manager.get(session_id)
    bookmarks = session.document.bookmarks
    return json.dumps({"bookmarks": serialize(bookmarks), "count": len(bookmarks)})


@mcp.resource("document://{session_id}/form-fields")
@handle_errors
def document_form_fields(session_id: str) -> str:
    """Form fields in the document (PDF only)."""
    _session, doc = session_manager.get_pdf(session_id)
    fields = doc.form_fields
    return json.dumps({"fields": serialize(fields), "count": len(fields), "has_form": doc.has_form})


@mcp.resource("document://{session_id}/signatures")
@handle_errors
def document_signatures(session_id: str) -> str:
    """Digital signatures in the document (PDF only)."""
    _session, doc = session_manager.get_pdf(session_id)
    sigs = doc.signatures
    return json.dumps({"signatures": serialize(sigs), "count": len(sigs)})
