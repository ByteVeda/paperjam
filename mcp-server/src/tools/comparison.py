"""Comparison tools: diff and visual_diff between documents."""

from __future__ import annotations

import base64
import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def diff_documents(session_id_a: str, session_id_b: str) -> str:
    """Compare two PDF documents at the text level.

    Returns per-page changes (additions, removals, modifications) and summary statistics.
    """
    _sa, doc_a = session_manager.get_pdf(session_id_a)
    _sb, doc_b = session_manager.get_pdf(session_id_b)
    result = doc_a.diff(doc_b)
    return json.dumps(serialize(result))


@mcp.tool()
@handle_errors
def visual_diff(
    session_id_a: str,
    session_id_b: str,
    dpi: float = 150.0,
    threshold: int = 10,
    mode: str = "both",
) -> str:
    """Compare two PDF documents visually (pixel-level).

    Returns similarity scores and diff images as base64-encoded PNGs.
    mode: 'both' (side-by-side + diff), 'diff_only', or 'overlay'.
    """
    _sa, doc_a = session_manager.get_pdf(session_id_a)
    _sb, doc_b = session_manager.get_pdf(session_id_b)
    result = doc_a.visual_diff(doc_b, dpi=dpi, threshold=threshold, mode=mode)

    pages_data = []
    for p in result.pages:
        pages_data.append(
            {
                "page": p.page,
                "similarity": p.similarity,
                "changed_pixel_count": p.changed_pixel_count,
                "diff_image_base64": base64.b64encode(p.diff_image).decode(),
                "diff_image_width": p.diff_image_width,
                "diff_image_height": p.diff_image_height,
            }
        )

    return json.dumps(
        {
            "overall_similarity": result.overall_similarity,
            "text_diff_summary": serialize(result.text_diff_summary),
            "pages": pages_data,
        }
    )
