"""Async wrappers for CPU-bound paperjam operations.

All async methods delegate to the sync counterparts via
``loop.run_in_executor()``, keeping the event loop responsive.
"""

from __future__ import annotations

import asyncio
import os
from concurrent.futures import ThreadPoolExecutor
from functools import partial
from typing import TYPE_CHECKING

from paperjam._document import Document
from paperjam._page import Page

if TYPE_CHECKING:
    from collections.abc import Sequence

    from paperjam._enums import TableStrategy
    from paperjam._types import (
        DiffResult,
        RedactResult,
        RenderedImage,
        SearchResult,
        Table,
    )

_executor = ThreadPoolExecutor(max_workers=min(4, os.cpu_count() or 4))


def configure(*, max_workers: int | None = None) -> None:
    """Reconfigure the async thread pool.

    Args:
        max_workers: Maximum number of threads for async operations.
    """
    global _executor
    if max_workers is not None:
        _executor.shutdown(wait=False)
        _executor = ThreadPoolExecutor(max_workers=max_workers)


def _get_executor() -> ThreadPoolExecutor:
    return _executor


# ---------------------------------------------------------------------------
# Document async methods
# ---------------------------------------------------------------------------


async def _aopen(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
) -> Document:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(_get_executor(), partial(Document, path_or_bytes, password=password))


async def _asave(self: Document, path: str | os.PathLike[str]) -> None:
    loop = asyncio.get_running_loop()
    await loop.run_in_executor(_get_executor(), partial(self.save, path))


async def _asave_bytes(self: Document) -> bytes:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(_get_executor(), self.save_bytes)


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
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            self.render_page,
            page_number,
            dpi=dpi,
            format=format,
            quality=quality,
            background_color=background_color,
            scale_to_width=scale_to_width,
            scale_to_height=scale_to_height,
        ),
    )


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
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            self.render_pages,
            pages=pages,
            dpi=dpi,
            format=format,
            quality=quality,
            background_color=background_color,
            scale_to_width=scale_to_width,
            scale_to_height=scale_to_height,
        ),
    )


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
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            self.extract_tables,
            strategy=strategy,
            min_rows=min_rows,
            min_cols=min_cols,
            snap_tolerance=snap_tolerance,
            row_tolerance=row_tolerance,
            min_col_gap=min_col_gap,
        ),
    )


async def _ato_markdown(self: Document, **kwargs) -> str:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(_get_executor(), partial(self.to_markdown, **kwargs))


async def _asearch(
    self: Document,
    query: str,
    *,
    case_sensitive: bool = True,
    max_results: int = 0,
    use_regex: bool = False,
) -> list[SearchResult]:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            self.search,
            query,
            case_sensitive=case_sensitive,
            max_results=max_results,
            use_regex=use_regex,
        ),
    )


async def _adiff(self: Document, other: Document) -> DiffResult:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(_get_executor(), partial(self.diff, other))


async def _aredact_text(
    self: Document,
    query: str,
    *,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: tuple[float, float, float] | None = None,
) -> tuple[Document, RedactResult]:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            self.redact_text,
            query,
            case_sensitive=case_sensitive,
            use_regex=use_regex,
            fill_color=fill_color,
        ),
    )


# ---------------------------------------------------------------------------
# Page async methods
# ---------------------------------------------------------------------------


async def _page_aextract_text(self: Page) -> str:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(_get_executor(), self.extract_text)


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
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            self.extract_tables,
            strategy=strategy,
            min_rows=min_rows,
            min_cols=min_cols,
            snap_tolerance=snap_tolerance,
            row_tolerance=row_tolerance,
            min_col_gap=min_col_gap,
        ),
    )


async def _page_ato_markdown(self: Page, **kwargs) -> str:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(_get_executor(), partial(self.to_markdown, **kwargs))


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
    from paperjam._functions import merge

    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(merge, documents, deduplicate_resources=deduplicate_resources),
    )


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
    from paperjam._render import render

    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(
            render,
            path_or_bytes,
            page=page,
            dpi=dpi,
            format=format,
            quality=quality,
            background_color=background_color,
            scale_to_width=scale_to_width,
            scale_to_height=scale_to_height,
        ),
    )


async def ato_markdown(
    path_or_bytes: str | os.PathLike[str] | bytes,
    *,
    password: str | None = None,
    **kwargs,
) -> str:
    """Open a PDF and convert to Markdown asynchronously."""
    from paperjam._functions import to_markdown

    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        _get_executor(),
        partial(to_markdown, path_or_bytes, password=password, **kwargs),
    )


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
