"""Manipulation methods for Document: split, reorder, optimize, annotations, watermark."""

from __future__ import annotations

from typing import TYPE_CHECKING, Self

from paperjam import _paperjam
from paperjam._enums import AnnotationType, Rotation, WatermarkLayer, WatermarkPosition
from paperjam._types import OptimizeResult

if TYPE_CHECKING:
    from paperjam._protocols import DocumentBase

    _Base = DocumentBase
else:
    _Base = object

_UNSET = object()


class ManipulationMixin(_Base):
    __slots__ = ()

    def split(self, ranges: list[tuple[int, int]]) -> list[Self]:
        """Split into multiple documents by page ranges (1-indexed, inclusive)."""
        inner = self._ensure_open()
        parts = _paperjam.split(inner, [(s, e) for s, e in ranges])
        result = []
        for part in parts:
            doc = self._new_instance()
            doc._inner = part
            doc._closed = False
            result.append(doc)
        return result

    def split_pages(self) -> list[Self]:
        """Split into individual single-page documents."""
        return self.split([(i, i) for i in range(1, self.page_count + 1)])

    def reorder(self, page_order: list[int]) -> Self:
        """Reorder pages, returning a new Document.

        Args:
            page_order: List of 1-indexed page numbers in desired order.
                        Can subset (drop pages) or repeat (duplicate pages).
        """
        inner = self._ensure_open()
        result = _paperjam.reorder_pages(inner, page_order)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def optimize(
        self,
        *,
        compress_streams: bool = True,
        remove_unused: bool = True,
        remove_duplicates: bool = True,
        strip_metadata: bool = False,
    ) -> tuple[Self, OptimizeResult]:
        """Optimize the PDF to reduce file size.

        Returns a tuple of (optimized_document, result_stats).
        """
        inner = self._ensure_open()
        optimized, stats = _paperjam.optimize(inner, compress_streams, remove_unused, remove_duplicates, strip_metadata)
        doc = self._new_instance()
        doc._inner = optimized
        doc._closed = False
        return doc, OptimizeResult(**stats)

    def add_annotation(
        self,
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
    ) -> Self:
        """Add an annotation to a page, returning a new Document."""
        inner = self._ensure_open()
        type_str = annotation_type.value if isinstance(annotation_type, AnnotationType) else str(annotation_type)
        result = _paperjam.add_annotation(
            inner,
            page,
            type_str,
            list(rect),
            contents,
            author,
            list(color) if color else None,
            opacity,
            list(quad_points) if quad_points else None,
            url,
        )
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def remove_annotations(
        self,
        page: int,
        *,
        annotation_types: list[AnnotationType | str] | None = None,
        indices: list[int] | None = None,
    ) -> tuple[Self, int]:
        """Remove annotations from a page, returning a new Document and count removed.

        Args:
            page: 1-indexed page number.
            annotation_types: If provided, only remove annotations matching these types.
            indices: If provided, only remove annotations at these 0-based positions.
        """
        inner = self._ensure_open()
        type_strs = None
        if annotation_types is not None:
            type_strs = [t.value if isinstance(t, AnnotationType) else str(t) for t in annotation_types]
        result, count = _paperjam.remove_annotations(inner, page, type_strs, indices)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc, count

    def add_watermark(
        self,
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
        x: float | None = None,
        y: float | None = None,
    ) -> Self:
        """Add a text watermark to pages, returning a new Document.

        Args:
            x: Custom X position in points. When both x and y are provided,
               the position parameter is ignored.
            y: Custom Y position in points. When both x and y are provided,
               the position parameter is ignored.
        """
        inner = self._ensure_open()
        pos_str = position.value if isinstance(position, WatermarkPosition) else str(position)
        layer_str = layer.value if isinstance(layer, WatermarkLayer) else str(layer)
        result = _paperjam.add_watermark(
            inner,
            text,
            font_size,
            rotation,
            opacity,
            list(color),
            font,
            pos_str,
            layer_str,
            pages,
            custom_x=x,
            custom_y=y,
        )
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def rotate(
        self,
        page_rotations: list[tuple[int, Rotation | int]],
    ) -> Self:
        """Rotate pages by specified angles, returning a new Document.

        Args:
            page_rotations: List of (page_number, rotation) tuples.
                            page_number is 1-indexed. rotation is degrees (0, 90, 180, 270)
                            or a Rotation enum value.
        """
        inner = self._ensure_open()
        normalized = [(page, rot.value if isinstance(rot, Rotation) else int(rot)) for page, rot in page_rotations]
        result = _paperjam.rotate_pages(inner, normalized)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def delete_pages(self, page_numbers: list[int]) -> Self:
        """Delete specific pages from the document, returning a new Document.

        Args:
            page_numbers: List of 1-indexed page numbers to remove.
                          At least one page must remain.
        """
        inner = self._ensure_open()
        result = _paperjam.delete_pages(inner, page_numbers)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def insert_blank_pages(
        self,
        positions: list[tuple[int, float, float]],
    ) -> Self:
        """Insert blank pages at specified positions, returning a new Document.

        Args:
            positions: List of (after_page, width, height) tuples.
                       after_page=0 inserts at the beginning.
                       width and height are in PDF points (72 points = 1 inch).
        """
        inner = self._ensure_open()
        result = _paperjam.insert_blank_pages(inner, positions)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def set_metadata(
        self,
        *,
        title: str | None = _UNSET,  # type: ignore[assignment]
        author: str | None = _UNSET,  # type: ignore[assignment]
        subject: str | None = _UNSET,  # type: ignore[assignment]
        keywords: str | None = _UNSET,  # type: ignore[assignment]
        creator: str | None = _UNSET,  # type: ignore[assignment]
        producer: str | None = _UNSET,  # type: ignore[assignment]
    ) -> Self:
        """Update document metadata, returning a new Document.

        Pass a string value to set a field, None to remove it,
        or omit it to leave it unchanged.
        """
        inner = self._ensure_open()
        updates = {}
        for key, val in [
            ("title", title),
            ("author", author),
            ("subject", subject),
            ("keywords", keywords),
            ("creator", creator),
            ("producer", producer),
        ]:
            if val is not _UNSET:
                updates[key] = val
        result = _paperjam.set_metadata(inner, updates)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc

    def set_bookmarks(
        self,
        bookmarks: list,
    ) -> Self:
        """Replace the document's bookmarks/outlines, returning a new Document.

        Args:
            bookmarks: List of Bookmark objects defining the new outline tree.
                       Pass an empty list to remove all bookmarks.
        """
        from paperjam._types import Bookmark

        inner = self._ensure_open()

        def _to_dict(bm: Bookmark) -> dict:
            d: dict = {"title": bm.title, "page": bm.page}
            if bm.children:
                d["children"] = [_to_dict(c) for c in bm.children]
            return d

        bm_dicts = [_to_dict(bm) for bm in bookmarks]
        result = _paperjam.set_bookmarks(inner, bm_dicts)
        doc = self._new_instance()
        doc._inner = result
        doc._closed = False
        return doc
