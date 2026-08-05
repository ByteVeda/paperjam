"""Validation tools: validate_pdf_a, validate_pdf_ua, convert_to_pdf_a."""

from __future__ import annotations

import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def validate_pdf_a(session_id: str, level: str = "1b") -> str:
    """Validate PDF/A compliance.

    level: 1b (default), 1a, or 2b.
    Returns compliance status and a list of issues.
    """
    _session, doc = session_manager.get_pdf(session_id)
    report = doc.validate_pdf_a(level=level)
    return json.dumps(serialize(report))


@mcp.tool()
@handle_errors
def validate_pdf_ua(session_id: str, level: str = "1") -> str:
    """Validate PDF/UA (accessibility) compliance.

    Checks: MarkInfo, language, structure tree, alt text, headings,
    tab order, and annotation accessibility.
    """
    _session, doc = session_manager.get_pdf(session_id)
    report = doc.validate_pdf_ua(level=level)
    return json.dumps(serialize(report))


@mcp.tool()
@handle_errors
def convert_to_pdf_a(
    session_id: str,
    level: str = "1b",
    force: bool = False,
) -> str:
    """Convert a PDF to PDF/A conformance. Modifies the session in-place.

    Performs: XMP metadata update, sRGB OutputIntent embedding,
    JavaScript/action removal, transparency removal (PDF/A-1),
    and encryption removal.
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, result = doc.convert_to_pdf_a(level=level, force=force)
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))
