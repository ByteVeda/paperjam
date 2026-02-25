"""Document class wrapping the Rust PDF engine."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, overload

from paperjam import _paperjam
from paperjam._enums import AnnotationType, WatermarkLayer, WatermarkPosition
from paperjam._page import Page
from paperjam._page import _raw_block_to_content_block
from paperjam._types import (
    Annotation,
    Bookmark,
    ContentBlock,
    DiffOp,
    DiffResult,
    DiffSummary,
    Metadata,
    OptimizeResult,
    PageDiff,
    SanitizedItem,
    SanitizeResult,
    SearchResult,
)

if TYPE_CHECKING:
    from collections.abc import Iterator


class Document:
    """A PDF document with lazy page loading.

    Use as a context manager for automatic resource cleanup:

        with paperjam.open("file.pdf") as doc:
            for page in doc.pages:
                print(page.extract_text())

    Or without a context manager (resources freed on garbage collection):

        doc = paperjam.open("file.pdf")
        text = doc.pages[0].extract_text()
    """

    __slots__ = ("_closed", "_inner")

    def __init__(
        self,
        path_or_bytes: str | os.PathLike[str] | bytes,
        *,
        password: str | None = None,
    ) -> None:
        if isinstance(path_or_bytes, (str, os.PathLike)):
            path = str(path_or_bytes)
            if password is not None:
                self._inner = _paperjam.RustDocument.open_with_password(path, password)
            else:
                self._inner = _paperjam.RustDocument.open(path)
        elif isinstance(path_or_bytes, (bytes, bytearray, memoryview)):
            if password is not None:
                self._inner = _paperjam.RustDocument.from_bytes_with_password(
                    bytes(path_or_bytes), password
                )
            else:
                self._inner = _paperjam.RustDocument.from_bytes(bytes(path_or_bytes))
        else:
            raise TypeError(
                f"Expected str, os.PathLike, or bytes, got {type(path_or_bytes).__name__}"
            )
        self._closed = False

    def __enter__(self) -> Document:
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self.close()

    def __repr__(self) -> str:
        state = "closed" if self._closed else f"{self.page_count} pages"
        return f"<paperjam.Document [{state}]>"

    def __len__(self) -> int:
        return self.page_count

    def close(self) -> None:
        """Release the underlying PDF resources."""
        if not self._closed:
            self._inner = None  # type: ignore[assignment]
            self._closed = True

    def _ensure_open(self) -> _paperjam.RustDocument:
        if self._closed:
            raise ValueError("I/O operation on closed document")
        return self._inner

    @property
    def page_count(self) -> int:
        """Total number of pages in the document."""
        return self._ensure_open().page_count()

    @property
    def pages(self) -> _PageAccessor:
        """Access pages by index or iterate over all pages lazily."""
        return _PageAccessor(self)

    @property
    def metadata(self) -> Metadata:
        """Document metadata (title, author, etc.)."""
        raw = self._ensure_open().metadata()
        return Metadata(**raw)

    def save(self, path: str | os.PathLike[str]) -> None:
        """Save the document to a file."""
        self._ensure_open().save(str(path))

    def save_bytes(self) -> bytes:
        """Serialize the document to bytes."""
        return self._ensure_open().save_bytes()

    def split(self, ranges: list[tuple[int, int]]) -> list[Document]:
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

    def split_pages(self) -> list[Document]:
        """Split into individual single-page documents."""
        return self.split([(i, i) for i in range(1, self.page_count + 1)])

    @property
    def bookmarks(self) -> list[Bookmark]:
        """Document bookmarks/table of contents as a nested tree."""
        raw = self._ensure_open().bookmarks()
        return _build_bookmark_tree(raw)

    def search(
        self,
        query: str,
        *,
        case_sensitive: bool = True,
        max_results: int = 0,
    ) -> list[SearchResult]:
        """Search for text across all pages.

        Args:
            query: The text to search for.
            case_sensitive: Whether the search is case-sensitive (default True).
            max_results: Maximum number of results to return (0 = unlimited).
        """
        results: list[SearchResult] = []
        for page in self.pages:
            matches = page.search(query, case_sensitive=case_sensitive)
            results.extend(matches)
            if max_results > 0 and len(results) >= max_results:
                return results[:max_results]
        return results

    def reorder(self, page_order: list[int]) -> Document:
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

    def optimize(
        self,
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
    ) -> Document:
        """Add an annotation to a page, returning a new Document."""
        inner = self._ensure_open()
        type_str = annotation_type.value if isinstance(annotation_type, AnnotationType) else str(annotation_type)
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

    def remove_annotations(self, page: int) -> Document:
        """Remove all annotations from a page, returning a new Document."""
        inner = self._ensure_open()
        result, _count = _paperjam.remove_annotations(inner, page)
        doc = object.__new__(Document)
        doc._inner = result
        doc._closed = False
        return doc

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

    def extract_structure(
        self,
        *,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
    ) -> list[ContentBlock]:
        """Extract structured content (headings, paragraphs, lists, tables) from all pages."""
        inner = self._ensure_open()
        raw_blocks = inner.extract_structure(
            heading_size_ratio=heading_size_ratio,
            detect_lists=detect_lists,
            include_tables=include_tables,
        )
        return [_raw_block_to_content_block(b) for b in raw_blocks]

    def diff(self, other: Document) -> DiffResult:
        """Compare this document with another at the text level.

        Returns a DiffResult with per-page changes and summary statistics.
        """
        inner_a = self._ensure_open()
        inner_b = other._ensure_open()
        raw = _paperjam.diff_documents(inner_a, inner_b)
        return DiffResult(
            page_diffs=tuple(
                PageDiff(
                    page=pd["page"],
                    ops=tuple(DiffOp(**op) for op in pd["ops"]),
                )
                for pd in raw["page_diffs"]
            ),
            summary=DiffSummary(**raw["summary"]),
        )

    def sanitize(
        self,
        *,
        remove_javascript: bool = True,
        remove_embedded_files: bool = True,
        remove_actions: bool = True,
        remove_links: bool = True,
    ) -> tuple[Document, SanitizeResult]:
        """Remove potentially dangerous objects from the PDF.

        Returns a tuple of (sanitized_document, result_stats).
        """
        inner = self._ensure_open()
        sanitized, stats = _paperjam.sanitize(
            inner, remove_javascript, remove_embedded_files,
            remove_actions, remove_links,
        )
        doc = object.__new__(Document)
        doc._inner = sanitized
        doc._closed = False
        return doc, SanitizeResult(
            javascript_removed=stats["javascript_removed"],
            embedded_files_removed=stats["embedded_files_removed"],
            actions_removed=stats["actions_removed"],
            links_removed=stats["links_removed"],
            items=tuple(SanitizedItem(**item) for item in stats["items"]),
        )


def _build_bookmark_tree(flat_items: list[dict]) -> list[Bookmark]:
    """Build a nested bookmark tree from a flat level-annotated list."""
    if not flat_items:
        return []

    result: list[Bookmark] = []
    i = 0
    while i < len(flat_items):
        item = flat_items[i]
        level = item["level"]

        # Collect all children (items with higher level immediately following)
        children_items: list[dict] = []
        j = i + 1
        while j < len(flat_items) and flat_items[j]["level"] > level:
            children_items.append(flat_items[j])
            j += 1

        children = _build_bookmark_tree(children_items)
        result.append(
            Bookmark(
                title=item["title"],
                page=item["page"],
                level=level,
                children=tuple(children),
            )
        )
        i = j

    return result


class _PageAccessor:
    """Provides both indexing and iteration over pages."""

    __slots__ = ("_doc",)

    def __init__(self, doc: Document) -> None:
        self._doc = doc

    def __len__(self) -> int:
        return self._doc.page_count

    @overload
    def __getitem__(self, index: int) -> Page: ...

    @overload
    def __getitem__(self, index: slice) -> list[Page]: ...

    def __getitem__(self, index: int | slice) -> Page | list[Page]:
        inner = self._doc._ensure_open()
        if isinstance(index, int):
            if index < 0:
                index += len(self)
            if index < 0 or index >= len(self):
                raise IndexError(f"page index {index} out of range")
            return Page._from_rust(inner.page(index + 1), inner)
        elif isinstance(index, slice):
            indices = range(*index.indices(len(self)))
            return [Page._from_rust(inner.page(i + 1), inner) for i in indices]
        else:
            raise TypeError(
                f"indices must be integers or slices, not {type(index).__name__}"
            )

    def __iter__(self) -> Iterator[Page]:
        inner = self._doc._ensure_open()
        for i in range(1, len(self) + 1):
            yield Page._from_rust(inner.page(i), inner)
