"""Page stamping/overlay for Document."""

from __future__ import annotations

from typing import TYPE_CHECKING, Self

from paperjam import _paperjam

if TYPE_CHECKING:
    from paperjam._document import Document
    from paperjam._protocols import DocumentBase

    _Base = DocumentBase
else:
    _Base = object


class StampMixin(_Base):
    __slots__ = ()

    def stamp(
        self,
        stamp_doc: Document,
        *,
        source_page: int = 1,
        target_pages: list[int] | None = None,
        x: float = 0.0,
        y: float = 0.0,
        scale: float = 1.0,
        opacity: float = 1.0,
        layer: str = "over",
    ) -> Self:
        """Overlay a page from another PDF onto pages of this document."""
        inner = self._ensure_open()
        stamp_inner = stamp_doc._ensure_open()
        new_inner = _paperjam.stamp_pages(
            inner,
            stamp_inner,
            source_page=source_page,
            target_pages=target_pages,
            x=x,
            y=y,
            scale=scale,
            opacity=opacity,
            layer=layer,
        )
        doc = self._new_instance()
        doc._inner = new_inner
        doc._closed = False
        doc._raw_bytes = self._raw_bytes
        return doc
