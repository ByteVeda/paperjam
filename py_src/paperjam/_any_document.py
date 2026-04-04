"""Format-agnostic document wrapper for non-PDF formats."""

from __future__ import annotations

import builtins as _builtins
import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from paperjam._types import Bookmark, ContentBlock, Image, Metadata, Table, TextLine

from paperjam import _paperjam
from paperjam._types import (
    Bookmark as _Bookmark,
)
from paperjam._types import (
    Image as _Image,
)
from paperjam._types import (
    Metadata as _Metadata,
)
from paperjam._types import (
    TextLine as _TextLine,
)

builtins_open = _builtins.open


def _dict_to_metadata(d: dict) -> _Metadata:
    return _Metadata(
        title=d.get("title"),
        author=d.get("author"),
        subject=d.get("subject"),
        keywords=d.get("keywords"),
        creator=d.get("creator"),
        producer=d.get("producer"),
        creation_date=d.get("creation_date"),
        modification_date=d.get("modification_date"),
        pdf_version="",
        page_count=d.get("page_count", 0),
        is_encrypted=False,
        xmp_metadata=None,
    )


class AnyDocument:
    """A document in any supported format (DOCX, XLSX, PPTX, HTML, EPUB).

    Provides format-agnostic extraction methods. For PDF-specific operations
    (encryption, signatures, rendering, forms, etc.), use Document instead.
    """

    __slots__ = ("_closed", "_inner")

    def __init__(
        self,
        path_or_bytes: str | os.PathLike[str] | bytes,
        *,
        format: str | None = None,
    ) -> None:
        if isinstance(path_or_bytes, (bytes, bytearray, memoryview)):
            if format is None:
                raise ValueError("format is required when opening from bytes")
            self._inner = _paperjam.RustAnyDocument.from_bytes(bytes(path_or_bytes), format)
        else:
            path = str(path_or_bytes)
            if not os.path.exists(path):
                raise FileNotFoundError(f"No such file: '{path}'")
            self._inner = _paperjam.RustAnyDocument.open(path, format)
        self._closed = False

    def _ensure_open(self) -> _paperjam.RustAnyDocument:
        if self._closed:
            raise ValueError("Document is closed")
        return self._inner

    @property
    def format(self) -> str:
        """Document format (e.g., 'docx', 'html', 'epub')."""
        return str(self._ensure_open().format_name())

    @property
    def page_count(self) -> int:
        """Number of pages (or sheets/slides/chapters)."""
        return int(self._ensure_open().page_count())

    @property
    def metadata(self) -> Metadata:
        """Document metadata."""
        return _dict_to_metadata(self._ensure_open().metadata())

    @property
    def bookmarks(self) -> list[Bookmark]:
        """Document bookmarks/TOC entries."""
        return [_Bookmark(title=b["title"], page=b["page"], level=b.get("level", 0)) for b in self._ensure_open().bookmarks()]

    def extract_text(self) -> str:
        """Extract all text from the document."""
        return str(self._ensure_open().extract_text())

    def extract_text_lines(self) -> list[TextLine]:
        """Extract text lines with bounding boxes."""
        return [_TextLine(text=line["text"], spans=(), bbox=tuple(line["bbox"])) for line in self._ensure_open().extract_text_lines()]

    def extract_tables(self) -> list[Table]:
        """Extract all tables from the document."""
        return list(self._ensure_open().extract_tables())  # type: ignore[arg-type]

    def extract_structure(self) -> list[ContentBlock]:
        """Extract document structure (headings, paragraphs, lists)."""
        return list(self._ensure_open().extract_structure())  # type: ignore[arg-type]

    def extract_images(self) -> list[Image]:
        """Extract embedded images."""
        return [
            _Image(
                width=i["width"],
                height=i["height"],
                color_space=i.get("color_space"),
                bits_per_component=None,
                filters=[],
                data=b"",
            )
            for i in self._ensure_open().extract_images()
        ]

    def to_markdown(self) -> str:
        """Convert the document to Markdown."""
        return str(self._ensure_open().to_markdown())

    def convert_to(self, format: str) -> bytes:
        """Convert to another format. Returns the output bytes."""
        return bytes(self._ensure_open().convert_to(format))

    def save(self, path: str | os.PathLike[str]) -> None:
        """Save the document to a file."""
        data = self.save_bytes()
        with builtins_open(str(path), "wb") as f:
            f.write(data)

    def save_bytes(self) -> bytes:
        """Save the document to bytes."""
        return bytes(self._ensure_open().save_to_bytes())

    def close(self) -> None:
        """Close the document and release resources."""
        self._closed = True

    def __enter__(self) -> AnyDocument:
        return self

    def __exit__(self, *args: object) -> None:
        self.close()

    def __repr__(self) -> str:
        if self._closed:
            return "AnyDocument(<closed>)"
        return f"AnyDocument(format={self.format!r}, pages={self.page_count})"
