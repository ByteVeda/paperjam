"""Extraction methods for Document: extract_structure, to_markdown, search, extract_tables."""

from __future__ import annotations

from typing import TYPE_CHECKING

from paperjam._document import Document
from paperjam._enums import TableStrategy
from paperjam._page import _raw_block_to_content_block

if TYPE_CHECKING:
    from paperjam._types import ContentBlock, SearchResult, Table


def _extract_structure(
    self: Document,
    *,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> list[ContentBlock]:
    """Extract structured content (headings, paragraphs, lists, tables) from all pages."""
    inner = self._ensure_open()
    raw_blocks = inner.extract_structure(
        heading_size_ratio=heading_size_ratio,
        detect_lists=detect_lists,
        include_tables=include_tables,
        layout_aware=layout_aware,
    )
    return [_raw_block_to_content_block(b) for b in raw_blocks]


def _to_markdown(
    self: Document,
    *,
    heading_offset: int = 0,
    page_separator: str = "---",
    include_page_numbers: bool = False,
    page_number_format: str = "<!-- page {n} -->",
    html_tables: bool = False,
    table_header_first_row: bool = True,
    normalize_list_markers: bool = True,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> str:
    """Convert the entire document to Markdown."""
    inner = self._ensure_open()
    return inner.to_markdown(
        heading_offset=heading_offset,
        page_separator=page_separator,
        include_page_numbers=include_page_numbers,
        page_number_format=page_number_format,
        html_tables=html_tables,
        table_header_first_row=table_header_first_row,
        normalize_list_markers=normalize_list_markers,
        heading_size_ratio=heading_size_ratio,
        detect_lists=detect_lists,
        include_tables=include_tables,
        layout_aware=layout_aware,
    )


def _search(
    self: Document,
    query: str,
    *,
    case_sensitive: bool = True,
    max_results: int = 0,
    use_regex: bool = False,
) -> list[SearchResult]:
    """Search for text across all pages.

    Args:
        query: The text or regex pattern to search for.
        case_sensitive: Whether the search is case-sensitive (default True).
        max_results: Maximum number of results to return (0 = unlimited).
        use_regex: If True, treat query as a regular expression.
    """
    results: list[SearchResult] = []
    for page in self.pages:
        matches = page.search(query, case_sensitive=case_sensitive, use_regex=use_regex)
        results.extend(matches)
        if max_results > 0 and len(results) >= max_results:
            return results[:max_results]
    return results


def _extract_tables(
    self: Document,
    *,
    strategy: TableStrategy | str = TableStrategy.AUTO,
    min_rows: int = 2,
    min_cols: int = 2,
    snap_tolerance: float = 3.0,
    row_tolerance: float = 0.5,
    min_col_gap: float = 10.0,
) -> list[Table]:
    """Extract tables from all pages."""
    tables: list[Table] = []
    for page in self.pages:
        tables.extend(page.extract_tables(
            strategy=strategy, min_rows=min_rows, min_cols=min_cols,
            snap_tolerance=snap_tolerance, row_tolerance=row_tolerance,
            min_col_gap=min_col_gap,
        ))
    return tables


Document.extract_structure = _extract_structure  # type: ignore[method-assign]
Document.to_markdown = _to_markdown  # type: ignore[method-assign]
Document.search = _search  # type: ignore[method-assign]
Document.extract_tables = _extract_tables  # type: ignore[method-assign]
