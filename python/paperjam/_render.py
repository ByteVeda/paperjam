"""Rendering methods for Document and Page, plus standalone render function."""

from __future__ import annotations

import os
import pathlib

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._page import Page
from paperjam._types import RenderedImage


def _pdfium_library_path() -> str | None:
    """Return the path to the bundled libpdfium.so, or None if not found."""
    lib = pathlib.Path(__file__).parent / "libpdfium.so"
    return str(lib) if lib.exists() else None


def _render_page(
    self: Document,
    page_number: int,
    *,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage:
    """Render a single page to an image.

    Args:
        page_number: 1-based page number to render.
        dpi: Resolution in dots per inch (default 150).
        format: Image format - "png", "jpeg", or "bmp" (default "png").
        quality: JPEG quality 1-100 (default 85, only used for JPEG).
        background_color: RGB tuple (0-255) for background color.
        scale_to_width: Target pixel width (overrides DPI).
        scale_to_height: Target pixel height (overrides DPI).

    Returns:
        A RenderedImage with the image data and dimensions.
    """
    bg = list(background_color) if background_color else None
    raw_bytes = getattr(self, "_raw_bytes", None)
    if raw_bytes is not None:
        # Fast path: pass original bytes directly, skip serialization
        raw = _paperjam.render_file(
            raw_bytes, page_number, dpi, format, quality,
            bg, scale_to_width, scale_to_height,
            library_path=_pdfium_library_path(),
        )
    else:
        inner = self._ensure_open()
        raw = _paperjam.render_page(
            inner, page_number, dpi, format, quality,
            bg, scale_to_width, scale_to_height,
            library_path=_pdfium_library_path(),
        )
    return RenderedImage(
        data=bytes(raw["data"]),
        width=raw["width"],
        height=raw["height"],
        format=raw["format"],
        page=raw["page"],
    )


def _render_pages(
    self: Document,
    *,
    pages: list[int] | None = None,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> list[RenderedImage]:
    """Render multiple pages to images.

    Args:
        pages: List of 1-based page numbers. None renders all pages.
        dpi: Resolution in dots per inch (default 150).
        format: Image format - "png", "jpeg", or "bmp" (default "png").
        quality: JPEG quality 1-100 (default 85, only used for JPEG).
        background_color: RGB tuple (0-255) for background color.
        scale_to_width: Target pixel width (overrides DPI).
        scale_to_height: Target pixel height (overrides DPI).

    Returns:
        List of RenderedImage objects.
    """
    bg = list(background_color) if background_color else None
    raw_bytes = getattr(self, "_raw_bytes", None)
    if raw_bytes is not None:
        # Fast path: pass original bytes directly, skip serialization
        raw_list = _paperjam.render_pages_bytes(
            raw_bytes, pages, dpi, format, quality,
            bg, scale_to_width, scale_to_height,
            library_path=_pdfium_library_path(),
        )
    else:
        inner = self._ensure_open()
        raw_list = _paperjam.render_pages(
            inner, pages, dpi, format, quality,
            bg, scale_to_width, scale_to_height,
            library_path=_pdfium_library_path(),
        )
    return [
        RenderedImage(
            data=bytes(r["data"]),
            width=r["width"],
            height=r["height"],
            format=r["format"],
            page=r["page"],
        )
        for r in raw_list
    ]


def _page_render(
    self: Page,
    *,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage:
    """Render this page to an image.

    Args:
        dpi: Resolution in dots per inch (default 150).
        format: Image format - "png", "jpeg", or "bmp" (default "png").
        quality: JPEG quality 1-100 (default 85, only used for JPEG).
        background_color: RGB tuple (0-255) for background color.
        scale_to_width: Target pixel width (overrides DPI).
        scale_to_height: Target pixel height (overrides DPI).

    Returns:
        A RenderedImage with the image data and dimensions.
    """
    if self._doc is None:
        raise RuntimeError("Page has no document reference; cannot render")
    bg = list(background_color) if background_color else None
    raw = _paperjam.render_page(
        self._doc, self.number, dpi, format, quality,
        bg, scale_to_width, scale_to_height,
        library_path=_pdfium_library_path(),
    )
    return RenderedImage(
        data=bytes(raw["data"]),
        width=raw["width"],
        height=raw["height"],
        format=raw["format"],
        page=raw["page"],
    )


def render(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    page: int = 1,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage:
    """Render a page from a PDF file or bytes directly.

    This is the fastest path for rendering - it passes bytes directly to
    the renderer without fully parsing the document.

    Args:
        path_or_bytes: PDF file path or raw PDF bytes.
        page: 1-based page number to render (default 1).
        dpi: Resolution in dots per inch (default 150).
        format: Image format - "png", "jpeg", or "bmp" (default "png").
        quality: JPEG quality 1-100 (default 85, only used for JPEG).
        background_color: RGB tuple (0-255) for background color.
        scale_to_width: Target pixel width (overrides DPI).
        scale_to_height: Target pixel height (overrides DPI).

    Returns:
        A RenderedImage with the image data and dimensions.
    """
    if isinstance(path_or_bytes, (str, os.PathLike)):
        with open(str(path_or_bytes), "rb") as f:
            data = f.read()
    elif isinstance(path_or_bytes, (bytes, bytearray, memoryview)):
        data = bytes(path_or_bytes)
    else:
        raise TypeError(
            f"Expected str, os.PathLike, or bytes, got {type(path_or_bytes).__name__}"
        )

    bg = list(background_color) if background_color else None
    raw = _paperjam.render_file(
        data, page, dpi, format, quality,
        bg, scale_to_width, scale_to_height,
        library_path=_pdfium_library_path(),
    )
    return RenderedImage(
        data=bytes(raw["data"]),
        width=raw["width"],
        height=raw["height"],
        format=raw["format"],
        page=raw["page"],
    )


Document.render_page = _render_page  # type: ignore[method-assign]
Document.render_pages = _render_pages  # type: ignore[method-assign]
Page.render = _page_render  # type: ignore[method-assign]
