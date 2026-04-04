"""Render tools: render pages to images."""

from __future__ import annotations

import base64
import json

from paperjam_mcp.server import handle_errors, mcp, resolve_path, session_manager

MAX_RENDER_DPI = 600.0
MAX_RENDER_PAGES = 50


@mcp.tool()
@handle_errors
def render_page(
    session_id: str,
    page_number: int,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    save_path: str | None = None,
) -> str:
    """Render a single PDF page to an image. Returns base64-encoded image data.

    format: png, jpeg, or bmp. quality: JPEG quality 1-100. Max DPI: 600.
    Optionally save to save_path.
    """
    dpi = min(dpi, MAX_RENDER_DPI)
    _session, doc = session_manager.get_pdf(session_id)
    result = doc.render_page(page_number, dpi=dpi, format=format, quality=quality)

    if save_path:
        resolved = resolve_path(save_path)
        resolved.parent.mkdir(parents=True, exist_ok=True)
        result.save(str(resolved))

    encoded = base64.b64encode(result.data).decode()
    return json.dumps(
        {
            "width": result.width,
            "height": result.height,
            "format": result.format,
            "page": result.page,
            "data_base64": encoded,
            "data_size": len(result.data),
            "saved_to": str(resolve_path(save_path)) if save_path else None,
        }
    )


@mcp.tool()
@handle_errors
def render_pages(
    session_id: str,
    pages: list[int] | None = None,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    save_dir: str | None = None,
) -> str:
    """Render multiple PDF pages to images. Returns metadata and base64 data for each page.

    pages: list of 1-based page numbers. None renders first 50 pages. Max DPI: 600.
    save_dir: if provided, saves each image as page_N.{format} in that directory.
    """
    dpi = min(dpi, MAX_RENDER_DPI)
    _session, doc = session_manager.get_pdf(session_id)
    if pages is None:
        pages = list(range(1, min(doc.page_count, MAX_RENDER_PAGES) + 1))
    elif len(pages) > MAX_RENDER_PAGES:
        pages = pages[:MAX_RENDER_PAGES]
    results = doc.render_pages(pages=pages, dpi=dpi, format=format, quality=quality)

    output = []
    for r in results:
        entry: dict = {
            "width": r.width,
            "height": r.height,
            "format": r.format,
            "page": r.page,
            "data_base64": base64.b64encode(r.data).decode(),
            "data_size": len(r.data),
        }
        if save_dir:
            resolved_dir = resolve_path(save_dir)
            resolved_dir.mkdir(parents=True, exist_ok=True)
            path = resolved_dir / f"page_{r.page}.{r.format}"
            r.save(str(path))
            entry["saved_to"] = str(path)
        output.append(entry)

    return json.dumps({"pages": output, "count": len(output)})
