"""Top-level convenience functions."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import os
    from collections.abc import Sequence

    from paperjam._types import DiffResult

from paperjam._document import Document


def open(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
) -> Document:
    """Open a PDF document.

    Args:
        path_or_bytes: File path, path-like object, or raw PDF bytes.
        password: Password for encrypted PDFs.

    Returns:
        A Document instance. Can be used as a context manager.

    Raises:
        FileNotFoundError: If the file does not exist.
        paperjam.PasswordRequired: If the PDF is encrypted and no password given.
        paperjam.InvalidPassword: If the password is incorrect.
        paperjam.ParseError: If the file is not a valid PDF.
    """
    return Document(path_or_bytes, password=password)


def merge(
    documents: Sequence[Document],
    *,
    deduplicate_resources: bool = False,
) -> Document:
    """Merge multiple open documents into one.

    Args:
        documents: Sequence of Document objects to merge.
        deduplicate_resources: If True, attempt to deduplicate shared fonts/images.

    Returns:
        A new Document containing all pages from all input documents.
    """
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
    """Merge PDF files from paths into one document.

    Args:
        paths: Sequence of file paths to merge.
        deduplicate_resources: If True, deduplicate shared resources.

    Returns:
        A new Document containing all pages from all files.
    """
    docs = [Document(p) for p in paths]
    return merge(docs, deduplicate_resources=deduplicate_resources)


def diff(doc_a: Document, doc_b: Document) -> DiffResult:
    """Compare two PDF documents at the text level.

    Returns a DiffResult with per-page changes and summary statistics.
    """
    return doc_a.diff(doc_b)
