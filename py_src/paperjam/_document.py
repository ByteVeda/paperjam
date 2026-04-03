"""Document class wrapping the Rust PDF engine."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, overload

from paperjam import _paperjam
from paperjam._async import AsyncMixin
from paperjam._comparison import ComparisonMixin
from paperjam._conversion import ConversionMixin
from paperjam._extraction import ExtractionMixin
from paperjam._forms import FormsMixin
from paperjam._manipulation import ManipulationMixin
from paperjam._page import Page
from paperjam._render import RenderMixin
from paperjam._security import SecurityMixin
from paperjam._signature import SignatureMixin
from paperjam._stamp import StampMixin
from paperjam._toc import TocMixin
from paperjam._types import Bookmark, Metadata
from paperjam._validation import ValidationMixin

if TYPE_CHECKING:
    from collections.abc import Iterator


class Document(
    ExtractionMixin,
    ManipulationMixin,
    ComparisonMixin,
    SecurityMixin,
    FormsMixin,
    RenderMixin,
    SignatureMixin,
    StampMixin,
    TocMixin,
    ValidationMixin,
    ConversionMixin,
    AsyncMixin,
):
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

    def __init__(
        self,
        path_or_bytes: str | os.PathLike[str] | bytes,
        *,
        password: str | None = None,
    ) -> None:
        """Open a PDF document from a file path or raw bytes.

        Args:
            path_or_bytes: File path, path-like object, or raw PDF bytes.
            password: Password for encrypted PDFs.
        """
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
                self._inner = _paperjam.RustDocument.from_bytes_with_password(bytes(path_or_bytes), password)
            else:
                self._inner = _paperjam.RustDocument.from_bytes(bytes(path_or_bytes))
        else:
            raise TypeError(f"Expected str, os.PathLike, or bytes, got {type(path_or_bytes).__name__}")
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

    def _new_instance(self) -> Document:
        """Create a new uninitialised instance of this class."""
        return object.__new__(type(self))

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
            raise TypeError(f"indices must be integers or slices, not {type(index).__name__}")

    def __iter__(self) -> Iterator[Page]:
        inner = self._doc._ensure_open()
        for i in range(1, len(self) + 1):
            yield Page._from_rust(inner.page(i), inner)
