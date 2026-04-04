"""Session management tools: open, close, list, info, save."""

from __future__ import annotations

import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, resolve_path, session_manager


@mcp.tool()
@handle_errors
def open_document(path: str, password: str | None = None, format: str | None = None) -> str:
    """Open a document from a file path. Supports PDF, DOCX, XLSX, PPTX, HTML, EPUB.

    Returns a session_id for subsequent operations. Use format to override auto-detection.
    """
    resolved = resolve_path(path)
    session_id = session_manager.open_from_path(str(resolved), password=password, fmt=format)
    session = session_manager.get(session_id)
    return json.dumps(
        {
            "session_id": session_id,
            "format": session.format,
            "is_pdf": session.is_pdf,
            "page_count": session.document.page_count,
            "path": session.path,
        }
    )


@mcp.tool()
@handle_errors
def close_document(session_id: str) -> str:
    """Close an open document session and free resources."""
    closed = session_manager.close(session_id)
    return json.dumps({"closed": closed})


@mcp.tool()
@handle_errors
def list_sessions() -> str:
    """List all currently open document sessions."""
    sessions = session_manager.list_sessions()
    return json.dumps({"sessions": sessions, "count": len(sessions)})


@mcp.tool()
@handle_errors
def get_document_info(session_id: str) -> str:
    """Get document metadata, page count, and format information."""
    session = session_manager.get(session_id)
    doc = session.document
    info: dict = {
        "session_id": session_id,
        "format": session.format,
        "is_pdf": session.is_pdf,
        "page_count": doc.page_count,
        "path": session.path,
    }
    info["metadata"] = serialize(doc.metadata)
    return json.dumps(info)


@mcp.tool()
@handle_errors
def save_document(session_id: str, output_path: str) -> str:
    """Save an open document to a file path."""
    session = session_manager.get(session_id)
    resolved = resolve_path(output_path)
    resolved.parent.mkdir(parents=True, exist_ok=True)
    session.document.save(str(resolved))
    return json.dumps({"saved": True, "path": str(resolved)})
