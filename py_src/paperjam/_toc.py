"""Auto TOC generation for Document."""

from __future__ import annotations

from typing import TYPE_CHECKING, Self

from paperjam import _paperjam
from paperjam._types import Bookmark

if TYPE_CHECKING:
    from paperjam._protocols import DocumentBase

    _Base = DocumentBase
else:
    _Base = object


def _spec_to_bookmark(raw: dict, level: int = 1) -> Bookmark:
    children = tuple(_spec_to_bookmark(c, level=level + 1) for c in raw.get("children", []))
    return Bookmark(
        title=raw["title"],
        page=raw["page"],
        level=level,
        children=children,
    )


class TocMixin(_Base):
    __slots__ = ()

    def generate_toc(
        self,
        *,
        max_depth: int = 6,
        heading_size_ratio: float = 1.2,
        layout_aware: bool = False,
        replace_existing: bool = True,
    ) -> tuple[Self, list[Bookmark]]:
        """Auto-generate a table of contents from heading structure.

        Returns a tuple of (new_document_with_bookmarks, list_of_bookmarks).
        """
        inner = self._ensure_open()
        new_inner, raw_specs = _paperjam.generate_toc(
            inner,
            max_depth=max_depth,
            heading_size_ratio=heading_size_ratio,
            layout_aware=layout_aware,
            replace_existing=replace_existing,
        )
        doc = self._new_instance()
        doc._inner = new_inner
        doc._closed = False
        doc._raw_bytes = self._raw_bytes
        bookmarks = [_spec_to_bookmark(s, level=1) for s in raw_specs]
        return doc, bookmarks
