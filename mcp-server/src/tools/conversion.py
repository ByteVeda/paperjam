"""Conversion tools: convert between formats, detect format."""

from __future__ import annotations

import json

import paperjam
from paperjam_mcp.server import handle_errors, mcp, resolve_path, session_manager


@mcp.tool()
@handle_errors
def convert_document(session_id: str, target_format: str) -> str:
    """Convert a document to another format. Creates a new session with the result.

    Supported target formats: pdf, docx, xlsx, pptx, html, epub, markdown.
    """
    session = session_manager.get(session_id)
    doc = session.document

    if isinstance(doc, paperjam.Document):
        pdf_bytes = doc.save_bytes()
        output_bytes = paperjam.convert_bytes(pdf_bytes, from_format=session.format, to_format=target_format)
    else:
        output_bytes = doc.convert_to(target_format)

    new_doc = paperjam.Document(output_bytes) if target_format == "pdf" else paperjam.AnyDocument(output_bytes, format=target_format)

    new_id = session_manager.register(new_doc, fmt=target_format)
    return json.dumps(
        {
            "session_id": new_id,
            "format": target_format,
            "source_session_id": session_id,
            "source_format": session.format,
        }
    )


@mcp.tool()
@handle_errors
def convert_file(input_path: str, output_path: str) -> str:
    """Convert a file directly from one format to another (no session needed).

    Formats are auto-detected from file extensions.
    """
    resolved_in = resolve_path(input_path)
    resolved_out = resolve_path(output_path)
    resolved_out.parent.mkdir(parents=True, exist_ok=True)
    result = paperjam.convert(str(resolved_in), str(resolved_out))
    return json.dumps(
        {
            "input": str(resolved_in),
            "output": str(resolved_out),
            "from_format": result.get("from_format"),
            "to_format": result.get("to_format"),
        }
    )


@mcp.tool()
@handle_errors
def detect_format(path: str) -> str:
    """Detect the document format from a file path or extension.

    Returns: pdf, docx, xlsx, pptx, html, epub, or unknown.
    """
    resolved = resolve_path(path)
    fmt = paperjam.detect_format(str(resolved))
    return json.dumps({"path": str(resolved), "format": fmt})
