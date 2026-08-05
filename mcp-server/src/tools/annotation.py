"""Annotation tools: watermark, add/remove annotations, stamp."""

from __future__ import annotations

import json

from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def add_watermark(
    session_id: str,
    text: str,
    font_size: float = 60.0,
    rotation: float = 45.0,
    opacity: float = 0.3,
    color: list[float] | None = None,
    font: str = "Helvetica",
    position: str = "center",
    layer: str = "over",
    pages: list[int] | None = None,
) -> str:
    """Add a text watermark to PDF pages. Modifies the session in-place.

    Position: center, top_left, top_right, bottom_left, bottom_right.
    Layer: over (on top of content) or under (behind content).
    Color is [r, g, b] with values 0.0-1.0. Default: [0.5, 0.5, 0.5].
    """
    _session, doc = session_manager.get_pdf(session_id)
    c: tuple[float, float, float] = (color[0], color[1], color[2]) if color else (0.5, 0.5, 0.5)
    new_doc = doc.add_watermark(
        text,
        font_size=font_size,
        rotation=rotation,
        opacity=opacity,
        color=c,
        font=font,
        position=position,
        layer=layer,
        pages=pages,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"watermark_added": True, "text": text})


@mcp.tool()
@handle_errors
def add_annotation(
    session_id: str,
    page: int,
    annotation_type: str,
    rect: list[float],
    contents: str | None = None,
    author: str | None = None,
    color: list[float] | None = None,
    opacity: float | None = None,
    url: str | None = None,
) -> str:
    """Add an annotation to a PDF page. Modifies the session in-place.

    Types: text, link, free_text, highlight, underline, strike_out, square, circle, line, stamp.
    Rect is [x1, y1, x2, y2] in PDF points.
    """
    _session, doc = session_manager.get_pdf(session_id)
    c: tuple[float, float, float] | None = (color[0], color[1], color[2]) if color else None
    r: tuple[float, float, float, float] = (rect[0], rect[1], rect[2], rect[3])
    new_doc = doc.add_annotation(
        page,
        annotation_type,
        r,
        contents=contents,
        author=author,
        color=c,
        opacity=opacity,
        url=url,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"annotation_added": True, "page": page, "type": annotation_type})


@mcp.tool()
@handle_errors
def remove_annotations(
    session_id: str,
    page: int,
    annotation_types: list[str] | None = None,
    indices: list[int] | None = None,
) -> str:
    """Remove annotations from a PDF page. Modifies the session in-place.

    Optionally filter by annotation type or 0-based index.
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, count = doc.remove_annotations(page, annotation_types=annotation_types, indices=indices)  # type: ignore[arg-type]
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"removed": count, "page": page})


@mcp.tool()
@handle_errors
def stamp_pages(
    session_id: str,
    stamp_session_id: str,
    source_page: int = 1,
    target_pages: list[int] | None = None,
    x: float = 0.0,
    y: float = 0.0,
    scale: float = 1.0,
    opacity: float = 1.0,
    layer: str = "over",
) -> str:
    """Overlay a page from another PDF onto pages of this document. Modifies the session in-place.

    Opens stamp_session_id as the source for the overlay content.
    """
    _session, doc = session_manager.get_pdf(session_id)
    _stamp_session, stamp_doc = session_manager.get_pdf(stamp_session_id)
    new_doc = doc.stamp(
        stamp_doc,
        source_page=source_page,
        target_pages=target_pages,
        x=x,
        y=y,
        scale=scale,
        opacity=opacity,
        layer=layer,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps({"stamped": True, "source_page": source_page})
