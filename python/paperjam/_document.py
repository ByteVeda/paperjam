"""Document class wrapping the Rust PDF engine."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, overload

from paperjam import _paperjam
from paperjam._page import Page
from paperjam._types import Bookmark, Metadata

if TYPE_CHECKING:
    from collections.abc import Iterator

    from paperjam._enums import AnnotationType, WatermarkLayer, WatermarkPosition
    from paperjam._types import (
        ContentBlock,
        DiffResult,
        FillFormResult,
        FormField,
        OptimizeResult,
        RedactRegion,
        RedactResult,
        RenderedImage,
        SanitizeResult,
        SearchResult,
        SignatureInfo,
        SignatureValidity,
    )


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

    __slots__ = ("_closed", "_inner", "_raw_bytes")

    if TYPE_CHECKING:
        # -- Extraction (attached by _extraction.py) --

        def extract_structure(
            self,
            *,
            heading_size_ratio: float = ...,
            detect_lists: bool = ...,
            include_tables: bool = ...,
            layout_aware: bool = ...,
        ) -> list[ContentBlock]: ...

        def to_markdown(
            self,
            *,
            heading_offset: int = ...,
            page_separator: str = ...,
            include_page_numbers: bool = ...,
            page_number_format: str = ...,
            html_tables: bool = ...,
            table_header_first_row: bool = ...,
            normalize_list_markers: bool = ...,
            heading_size_ratio: float = ...,
            detect_lists: bool = ...,
            include_tables: bool = ...,
            layout_aware: bool = ...,
        ) -> str: ...

        def search(
            self,
            query: str,
            *,
            case_sensitive: bool = ...,
            max_results: int = ...,
        ) -> list[SearchResult]: ...

        # -- Manipulation (attached by _manipulation.py) --

        def split(self, ranges: list[tuple[int, int]]) -> list[Document]: ...

        def split_pages(self) -> list[Document]: ...

        def reorder(self, page_order: list[int]) -> Document: ...

        def optimize(
            self,
            *,
            compress_streams: bool = ...,
            remove_unused: bool = ...,
            remove_duplicates: bool = ...,
            strip_metadata: bool = ...,
        ) -> tuple[Document, OptimizeResult]: ...

        def add_annotation(
            self,
            page: int,
            annotation_type: AnnotationType | str,
            rect: tuple[float, float, float, float],
            *,
            contents: str | None = ...,
            author: str | None = ...,
            color: tuple[float, float, float] | None = ...,
            opacity: float | None = ...,
            quad_points: tuple[float, ...] | None = ...,
            url: str | None = ...,
        ) -> Document: ...

        def remove_annotations(self, page: int) -> Document: ...

        def add_watermark(
            self,
            text: str,
            *,
            font_size: float = ...,
            rotation: float = ...,
            opacity: float = ...,
            color: tuple[float, float, float] = ...,
            font: str = ...,
            position: WatermarkPosition | str = ...,
            layer: WatermarkLayer | str = ...,
            pages: list[int] | None = ...,
        ) -> Document: ...

        # -- Comparison (attached by _comparison.py) --

        def diff(self, other: Document) -> DiffResult: ...

        # -- Security (attached by _security.py) --

        def sanitize(
            self,
            *,
            remove_javascript: bool = ...,
            remove_embedded_files: bool = ...,
            remove_actions: bool = ...,
            remove_links: bool = ...,
        ) -> tuple[Document, SanitizeResult]: ...

        def redact(
            self,
            regions: list[RedactRegion],
            *,
            fill_color: tuple[float, float, float] | None = ...,
        ) -> tuple[Document, RedactResult]: ...

        def redact_text(
            self,
            query: str,
            *,
            case_sensitive: bool = ...,
            fill_color: tuple[float, float, float] | None = ...,
        ) -> tuple[Document, RedactResult]: ...

        # -- Forms (attached by _forms.py) --

        @property
        def has_form(self) -> bool: ...

        @property
        def form_fields(self) -> list[FormField]: ...

        def fill_form(
            self,
            values: dict[str, str],
            *,
            need_appearances: bool = ...,
        ) -> tuple[Document, FillFormResult]: ...

        # -- Rendering (attached by _render.py) --

        def render_page(
            self,
            page_number: int,
            *,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
        ) -> RenderedImage: ...

        def render_pages(
            self,
            *,
            pages: list[int] | None = ...,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
        ) -> list[RenderedImage]: ...

        # -- Signatures (attached by _signature.py) --

        @property
        def signatures(self) -> list[SignatureInfo]: ...

        def verify_signatures(self) -> list[SignatureValidity]: ...

        def sign(
            self,
            *,
            private_key: bytes,
            certificates: list[bytes],
            reason: str | None = ...,
            location: str | None = ...,
            contact_info: str | None = ...,
            field_name: str = ...,
        ) -> bytes: ...

    def __init__(
        self,
        path_or_bytes: str | os.PathLike[str] | bytes,
        *,
        password: str | None = None,
    ) -> None:
        if isinstance(path_or_bytes, (str, os.PathLike)):
            path = str(path_or_bytes)
            with open(path, "rb") as f:
                self._raw_bytes: bytes | None = f.read()
            if password is not None:
                self._inner = _paperjam.RustDocument.open_with_password(path, password)
            else:
                self._inner = _paperjam.RustDocument.open(path)
        elif isinstance(path_or_bytes, (bytes, bytearray, memoryview)):
            self._raw_bytes = bytes(path_or_bytes)
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

    @property
    def bookmarks(self) -> list[Bookmark]:
        """Document bookmarks/table of contents as a nested tree."""
        raw = self._ensure_open().bookmarks()
        return _build_bookmark_tree(raw)


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
