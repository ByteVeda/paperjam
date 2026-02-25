"""Manipulation methods for Document: split, reorder, optimize, annotations, watermark."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._enums import AnnotationType, WatermarkLayer, WatermarkPosition
from paperjam._types import OptimizeResult


def _split(self: Document, ranges: list[tuple[int, int]]) -> list[Document]:
    """Split into multiple documents by page ranges (1-indexed, inclusive)."""
    inner = self._ensure_open()
    parts = _paperjam.split(inner, [(s, e) for s, e in ranges])
    result = []
    for part in parts:
        doc = object.__new__(Document)
        doc._inner = part
        doc._closed = False
        result.append(doc)
    return result


def _split_pages(self: Document) -> list[Document]:
    """Split into individual single-page documents."""
    return self.split([(i, i) for i in range(1, self.page_count + 1)])


def _reorder(self: Document, page_order: list[int]) -> Document:
    """Reorder pages, returning a new Document.

    Args:
        page_order: List of 1-indexed page numbers in desired order.
                    Can subset (drop pages) or repeat (duplicate pages).
    """
    inner = self._ensure_open()
    result = _paperjam.reorder_pages(inner, page_order)
    doc = object.__new__(Document)
    doc._inner = result
    doc._closed = False
    return doc


def _optimize(
    self: Document,
    *,
    compress_streams: bool = True,
    remove_unused: bool = True,
    remove_duplicates: bool = True,
    strip_metadata: bool = False,
) -> tuple[Document, OptimizeResult]:
    """Optimize the PDF to reduce file size.

    Returns a tuple of (optimized_document, result_stats).
    """
    inner = self._ensure_open()
    optimized, stats = _paperjam.optimize(
        inner, compress_streams, remove_unused, remove_duplicates, strip_metadata
    )
    doc = object.__new__(Document)
    doc._inner = optimized
    doc._closed = False
    return doc, OptimizeResult(**stats)


def _add_annotation(
    self: Document,
    page: int,
    annotation_type: AnnotationType | str,
    rect: tuple[float, float, float, float],
    *,
    contents: str | None = None,
    author: str | None = None,
    color: tuple[float, float, float] | None = None,
    opacity: float | None = None,
    quad_points: tuple[float, ...] | None = None,
    url: str | None = None,
) -> Document:
    """Add an annotation to a page, returning a new Document."""
    inner = self._ensure_open()
    type_str = (
        annotation_type.value
        if isinstance(annotation_type, AnnotationType)
        else str(annotation_type)
    )
    result = _paperjam.add_annotation(
        inner, page, type_str, list(rect),
        contents, author,
        list(color) if color else None,
        opacity,
        list(quad_points) if quad_points else None,
        url,
    )
    doc = object.__new__(Document)
    doc._inner = result
    doc._closed = False
    return doc


def _remove_annotations(self: Document, page: int) -> Document:
    """Remove all annotations from a page, returning a new Document."""
    inner = self._ensure_open()
    result, _count = _paperjam.remove_annotations(inner, page)
    doc = object.__new__(Document)
    doc._inner = result
    doc._closed = False
    return doc


def _add_watermark(
    self: Document,
    text: str,
    *,
    font_size: float = 60.0,
    rotation: float = 45.0,
    opacity: float = 0.3,
    color: tuple[float, float, float] = (0.5, 0.5, 0.5),
    font: str = "Helvetica",
    position: WatermarkPosition | str = WatermarkPosition.CENTER,
    layer: WatermarkLayer | str = WatermarkLayer.OVER,
    pages: list[int] | None = None,
) -> Document:
    """Add a text watermark to pages, returning a new Document."""
    inner = self._ensure_open()
    pos_str = position.value if isinstance(position, WatermarkPosition) else str(position)
    layer_str = layer.value if isinstance(layer, WatermarkLayer) else str(layer)
    result = _paperjam.add_watermark(
        inner, text, font_size, rotation, opacity,
        list(color), font, pos_str, layer_str, pages,
    )
    doc = object.__new__(Document)
    doc._inner = result
    doc._closed = False
    return doc


Document.split = _split  # type: ignore[method-assign]
Document.split_pages = _split_pages  # type: ignore[method-assign]
Document.reorder = _reorder  # type: ignore[method-assign]
Document.optimize = _optimize  # type: ignore[method-assign]
Document.add_annotation = _add_annotation  # type: ignore[method-assign]
Document.remove_annotations = _remove_annotations  # type: ignore[method-assign]
Document.add_watermark = _add_watermark  # type: ignore[method-assign]
