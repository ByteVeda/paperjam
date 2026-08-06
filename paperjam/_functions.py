"""Top-level convenience functions."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Sequence

    from paperjam._types import DiffResult

from paperjam._any_document import AnyDocument
from paperjam._document import Document


def open(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
    format: str | None = None,
) -> Document | AnyDocument:
    """Open a document file. Auto-detects format.

    Returns a Document for PDFs (with full PDF-specific methods),
    or an AnyDocument for other formats (DOCX, XLSX, PPTX, HTML, EPUB).

    Args:
        path_or_bytes: File path, path-like object, or raw document bytes.
        password: Password for encrypted PDFs.
        format: Explicit format hint (e.g., 'pdf', 'docx', 'html').
            If None, format is detected automatically.

    Returns:
        Document for PDFs, AnyDocument for other formats.
    """
    from paperjam._convert import detect_format

    detected = (detect_format(str(path_or_bytes)) if isinstance(path_or_bytes, (str, os.PathLike)) else "pdf") if format is None else format

    if detected in ("pdf", "unknown"):
        return Document(path_or_bytes, password=password)

    if password is not None:
        raise ValueError("password is only supported for PDF documents")
    return AnyDocument(path_or_bytes, format=detected)


def open_pdf(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
) -> Document:
    """Open a PDF document specifically.

    Use this when you need the full PDF-specific API (signatures, forms,
    rendering, encryption, etc.) and want strict type checking.

    Args:
        path_or_bytes: File path, path-like object, or raw PDF bytes.
        password: Password for encrypted PDFs.

    Returns:
        A Document instance with full PDF capabilities.
    """
    return Document(path_or_bytes, password=password)


def merge(
    documents: Sequence[Document],
    *,
    deduplicate_resources: bool = False,
) -> Document:
    """Merge multiple open PDF documents into one."""
    from paperjam import _paperjam

    inners = [doc._ensure_open() for doc in documents]
    result = _paperjam.merge(inners, deduplicate_resources=deduplicate_resources)
    new_doc = object.__new__(Document)
    new_doc._inner = result
    new_doc._closed = False
    return new_doc


def merge_files(
    paths: Sequence[str | os.PathLike[str]],
    *,
    deduplicate_resources: bool = False,
) -> Document:
    """Merge PDF files from paths into one document."""
    docs = [Document(p) for p in paths]
    return merge(docs, deduplicate_resources=deduplicate_resources)


def diff(doc_a: Document, doc_b: Document) -> DiffResult:
    """Compare two PDF documents at the text level."""
    return doc_a.diff(doc_b)


def to_markdown(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
    **kwargs: Any,
) -> str:
    """Open any document and convert it to Markdown in one call."""
    doc = open(path_or_bytes, password=password)
    if isinstance(doc, Document):
        return str(doc.to_markdown(**kwargs))
    return str(doc.to_markdown())
