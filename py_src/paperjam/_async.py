"""Async methods for Document and Page, powered by Rust + tokio.

All async methods delegate to native Rust coroutines exposed via
``pyo3-async-runtimes``, using tokio's blocking thread pool under the hood.
"""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._page import Page
from paperjam._types import (
    DiffOp,
    DiffResult,
    DiffSummary,
    PageDiff,
    RedactedItem,
    RedactResult,
    RenderedImage,
)

if TYPE_CHECKING:
    from collections.abc import Sequence

    from paperjam._enums import TableStrategy
    from paperjam._types import (
        SearchResult,
        Table,
    )


# ---------------------------------------------------------------------------
# Document async methods
# ---------------------------------------------------------------------------


async def _aopen(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
) -> Document:
    if isinstance(path_or_bytes, (str, os.PathLike)):
        path = str(path_or_bytes)
        if password is not None:
            rust_doc = await _paperjam.aopen_with_password(path, password)
        else:
            rust_doc = await _paperjam.aopen(path)
        doc = object.__new__(Document)
        doc._inner = rust_doc
        doc._closed = False
        with open(path, "rb") as f:
            doc._raw_bytes = f.read()
        return doc
    elif isinstance(path_or_bytes, (bytes, bytearray, memoryview)):
        data = bytes(path_or_bytes)
        if password is not None:
            rust_doc = await _paperjam.aopen_bytes_with_password(data, password)
        else:
            rust_doc = await _paperjam.aopen_bytes(data)
        doc = object.__new__(Document)
        doc._inner = rust_doc
        doc._closed = False
        doc._raw_bytes = data
        return doc
    else:
        msg = f"Expected str, os.PathLike, or bytes, got {type(path_or_bytes).__name__}"
        raise TypeError(msg)


async def _asave(self: Document, path: str | os.PathLike[str]) -> None:
    await _paperjam.asave(self._ensure_open(), str(path))


async def _asave_bytes(self: Document) -> bytes:
    return await _paperjam.asave_bytes(self._ensure_open())


async def _arender_page(
    self: Document,
    page_number: int,
    *,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage:
    raw = await _paperjam.arender_page(
        self._ensure_open(),
        page_number,
        dpi=dpi,
        format=format,
        quality=quality,
        background_color=list(background_color) if background_color else None,
        scale_to_width=scale_to_width,
        scale_to_height=scale_to_height,
    )
    return RenderedImage(data=bytes(raw["data"]), width=raw["width"], height=raw["height"], format=raw["format"], page=raw["page"])


async def _arender_pages(
    self: Document,
    *,
    pages: list[int] | None = None,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> list[RenderedImage]:
    raw_list = await _paperjam.arender_pages(
        self._ensure_open(),
        pages=pages,
        dpi=dpi,
        format=format,
        quality=quality,
        background_color=list(background_color) if background_color else None,
        scale_to_width=scale_to_width,
        scale_to_height=scale_to_height,
    )
    return [RenderedImage(data=bytes(r["data"]), width=r["width"], height=r["height"], format=r["format"], page=r["page"]) for r in raw_list]


async def _aextract_tables(
    self: Document,
    *,
    strategy: TableStrategy | str = "auto",
    min_rows: int = 2,
    min_cols: int = 2,
    snap_tolerance: float = 3.0,
    row_tolerance: float = 0.5,
    min_col_gap: float = 10.0,
) -> list[Table]:
    """Extract tables from all pages asynchronously."""
    import asyncio
    from functools import partial

    return await asyncio.to_thread(
        partial(
            self.extract_tables,
            strategy=strategy,
            min_rows=min_rows,
            min_cols=min_cols,
            snap_tolerance=snap_tolerance,
            row_tolerance=row_tolerance,
            min_col_gap=min_col_gap,
        )
    )


async def _ato_markdown(self: Document, **kwargs) -> str:
    return await _paperjam.ato_markdown(self._ensure_open(), **kwargs)


async def _asearch(
    self: Document,
    query: str,
    *,
    case_sensitive: bool = True,
    max_results: int = 0,
    use_regex: bool = False,
) -> list[SearchResult]:
    """Search across all pages asynchronously."""
    import asyncio
    from functools import partial

    return await asyncio.to_thread(
        partial(
            self.search,
            query,
            case_sensitive=case_sensitive,
            max_results=max_results,
            use_regex=use_regex,
        )
    )


async def _adiff(self: Document, other: Document) -> DiffResult:
    raw = await _paperjam.adiff_documents(self._ensure_open(), other._ensure_open())
    page_diffs = []
    for pd in raw["page_diffs"]:
        ops = tuple(DiffOp(**op) for op in pd["ops"])
        page_diffs.append(PageDiff(page=pd["page"], ops=ops))
    summary = DiffSummary(**raw["summary"])
    return DiffResult(page_diffs=tuple(page_diffs), summary=summary)


async def _aredact_text(
    self: Document,
    query: str,
    *,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: tuple[float, float, float] | None = None,
) -> tuple[Document, RedactResult]:
    color_list = list(fill_color) if fill_color else None
    rust_doc, raw = await _paperjam.aredact_text(
        self._ensure_open(),
        query,
        case_sensitive=case_sensitive,
        use_regex=use_regex,
        fill_color=color_list,
    )
    new_doc = object.__new__(Document)
    new_doc._inner = rust_doc
    new_doc._closed = False
    items = tuple(RedactedItem(**item) for item in raw["items"])
    result = RedactResult(
        pages_modified=raw["pages_modified"],
        items_redacted=raw["items_redacted"],
        items=items,
    )
    return new_doc, result


# ---------------------------------------------------------------------------
# Page async methods
# ---------------------------------------------------------------------------


async def _page_aextract_text(self: Page) -> str:
    return await _paperjam.apage_extract_text(self._inner)


async def _page_aextract_tables(
    self: Page,
    *,
    strategy: TableStrategy | str = "auto",
    min_rows: int = 2,
    min_cols: int = 2,
    snap_tolerance: float = 3.0,
    row_tolerance: float = 0.5,
    min_col_gap: float = 10.0,
) -> list[Table]:
    import asyncio
    from functools import partial

    return await asyncio.to_thread(
        partial(
            self.extract_tables,
            strategy=strategy,
            min_rows=min_rows,
            min_cols=min_cols,
            snap_tolerance=snap_tolerance,
            row_tolerance=row_tolerance,
            min_col_gap=min_col_gap,
        )
    )


async def _page_ato_markdown(self: Page, **kwargs) -> str:
    return await _paperjam.apage_to_markdown(self._inner, **kwargs)


# ---------------------------------------------------------------------------
# Top-level async functions
# ---------------------------------------------------------------------------


async def aopen(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
) -> Document:
    """Open a PDF document asynchronously."""
    return await _aopen(path_or_bytes, password=password)


async def amerge(
    documents: Sequence[Document],
    *,
    deduplicate_resources: bool = False,
) -> Document:
    """Merge multiple documents asynchronously."""
    inners = [doc._ensure_open() for doc in documents]
    rust_doc = await _paperjam.amerge(inners, deduplicate_resources=deduplicate_resources)
    new_doc = object.__new__(Document)
    new_doc._inner = rust_doc
    new_doc._closed = False
    return new_doc


async def arender(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    page: int = 1,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage:
    """Render a page from a PDF asynchronously."""
    if isinstance(path_or_bytes, (str, os.PathLike)):
        with open(str(path_or_bytes), "rb") as f:
            data = f.read()
    else:
        data = bytes(path_or_bytes)
    raw = await _paperjam.arender_file(
        data,
        page_number=page,
        dpi=dpi,
        format=format,
        quality=quality,
        background_color=list(background_color) if background_color else None,
        scale_to_width=scale_to_width,
        scale_to_height=scale_to_height,
    )
    return RenderedImage(data=bytes(raw["data"]), width=raw["width"], height=raw["height"], format=raw["format"], page=raw["page"])


async def ato_markdown(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
    **kwargs,
) -> str:
    """Open a PDF and convert to Markdown asynchronously."""
    doc = await aopen(path_or_bytes, password=password)
    return await doc.ato_markdown(**kwargs)


# ---------------------------------------------------------------------------
# Attach async methods to Document and Page
# ---------------------------------------------------------------------------

Document.aopen = staticmethod(_aopen)  # type: ignore[method-assign]
Document.asave = _asave  # type: ignore[method-assign]
Document.asave_bytes = _asave_bytes  # type: ignore[method-assign]
Document.arender_page = _arender_page  # type: ignore[method-assign]
Document.arender_pages = _arender_pages  # type: ignore[method-assign]
Document.aextract_tables = _aextract_tables  # type: ignore[method-assign]
Document.ato_markdown = _ato_markdown  # type: ignore[method-assign]
Document.asearch = _asearch  # type: ignore[method-assign]
Document.adiff = _adiff  # type: ignore[method-assign]
Document.aredact_text = _aredact_text  # type: ignore[method-assign]

Page.aextract_text = _page_aextract_text  # type: ignore[method-assign]
Page.aextract_tables = _page_aextract_tables  # type: ignore[method-assign]
Page.ato_markdown = _page_ato_markdown  # type: ignore[method-assign]
