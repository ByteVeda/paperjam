"""Security tools: sanitize, redact_text, redact_regions, encrypt."""

from __future__ import annotations

import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def sanitize_document(
    session_id: str,
    remove_javascript: bool = True,
    remove_embedded_files: bool = True,
    remove_actions: bool = True,
    remove_links: bool = True,
) -> str:
    """Remove potentially dangerous objects from a PDF. Modifies the session in-place.

    Removes JavaScript, embedded files, actions, and/or links.
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, result = doc.sanitize(
        remove_javascript=remove_javascript,
        remove_embedded_files=remove_embedded_files,
        remove_actions=remove_actions,
        remove_links=remove_links,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))


@mcp.tool()
@handle_errors
def redact_text(
    session_id: str,
    query: str,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: list[float] | None = None,
) -> str:
    """Redact all occurrences of text matching a query/pattern. Modifies the session in-place.

    This is true redaction: text is removed from the content stream, not just hidden.
    fill_color is [r, g, b] with values 0.0-1.0 for the overlay rectangle color.
    """
    _session, doc = session_manager.get_pdf(session_id)
    c: tuple[float, float, float] | None = (fill_color[0], fill_color[1], fill_color[2]) if fill_color else None
    new_doc, result = doc.redact_text(query, case_sensitive=case_sensitive, use_regex=use_regex, fill_color=c)
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))


@mcp.tool()
@handle_errors
def redact_regions(
    session_id: str,
    regions: list[dict],
    fill_color: list[float] | None = None,
) -> str:
    """Redact specific rectangular areas from a PDF. Modifies the session in-place.

    Each region is {"page": int, "rect": [x1, y1, x2, y2]} in PDF points.
    """
    from paperjam import RedactRegion

    _session, doc = session_manager.get_pdf(session_id)
    c: tuple[float, float, float] | None = (fill_color[0], fill_color[1], fill_color[2]) if fill_color else None
    region_objects = [RedactRegion(page=r["page"], rect=(r["rect"][0], r["rect"][1], r["rect"][2], r["rect"][3])) for r in regions]
    new_doc, result = doc.redact(region_objects, fill_color=c)
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))


@mcp.tool()
@handle_errors
def encrypt_document(
    session_id: str,
    user_password: str,
    owner_password: str | None = None,
    algorithm: str = "aes128",
    permissions: dict | None = None,
) -> str:
    """Encrypt a PDF with user/owner passwords. Modifies the session in-place.

    algorithm: aes128 (default), aes256, or rc4.
    permissions: optional dict of {print, modify, copy, annotate, fill_forms, accessibility, assemble, print_high_quality} booleans.
    """
    from paperjam import Permissions

    _session, doc = session_manager.get_pdf(session_id)
    perms = Permissions(**permissions) if permissions else None
    encrypted_bytes, result = doc.encrypt(user_password=user_password, owner_password=owner_password, permissions=perms, algorithm=algorithm)
    # Re-open the encrypted document as a new session (needs password to read)
    import paperjam as pj

    new_doc = pj.Document(encrypted_bytes, password=user_password)
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))
