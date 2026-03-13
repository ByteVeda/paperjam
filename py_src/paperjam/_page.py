"""Page class wrapping the Rust page object."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

from paperjam import _paperjam  # noqa: TC001
from paperjam._enums import TableStrategy
from paperjam._types import Annotation, Cell, ContentBlock, Image, LayoutRegion, PageInfo, PageLayout, Row, SearchResult, Table, TextLine, TextSpan

if TYPE_CHECKING:
    from paperjam._types import RenderedImage


class Page:
    """A single page in a PDF document.

    Pages are lazily parsed -- content is only decoded when you call
    extraction methods like extract_text() or extract_tables().
    """

    __slots__ = ("_doc", "_inner")

    _inner: _paperjam.RustPage
    _doc: _paperjam.RustDocument | None  # RustDocument reference for document-level operations

    def __init__(self) -> None:
        raise TypeError("Page objects cannot be created directly. Access via Document.pages.")

    @classmethod
    def _from_rust(
        cls,
        rust_page: _paperjam.RustPage,
        rust_doc: _paperjam.RustDocument | None = None,
    ) -> Page:
        obj = object.__new__(cls)
        obj._inner = rust_page
        obj._doc = rust_doc
        return obj

    def __repr__(self) -> str:
        return f"<paperjam.Page number={self.number} size=({self.width:.1f} x {self.height:.1f})>"

    @property
    def number(self) -> int:
        """1-based page number."""
        return self._inner.number()

    @property
    def width(self) -> float:
        """Page width in points (1 point = 1/72 inch)."""
        return self._inner.width()

    @property
    def height(self) -> float:
        """Page height in points."""
        return self._inner.height()

    @property
    def rotation(self) -> int:
        """Page rotation in degrees (0, 90, 180, 270)."""
        return self._inner.rotation()

    @property
    def info(self) -> PageInfo:
        """Basic page information as a frozen dataclass."""
        return PageInfo(
            number=self.number,
            width=self.width,
            height=self.height,
            rotation=self.rotation,
        )

    def extract_text(self) -> str:
        """Extract all text from the page as a single string."""
        return self._inner.extract_text()

    def extract_text_lines(self) -> list[TextLine]:
        """Extract text grouped into lines with position information."""
        raw_lines = self._inner.extract_text_lines()
        return [
            TextLine(
                text=line["text"],
                spans=tuple(TextSpan(**s) for s in line["spans"]),
                bbox=tuple(line["bbox"]),
            )
            for line in raw_lines
        ]

    def extract_text_spans(self) -> list[TextSpan]:
        """Extract text as individually positioned spans."""
        raw_spans = self._inner.extract_text_spans()
        return [TextSpan(**s) for s in raw_spans]

    @property
    def annotations(self) -> list[Annotation]:
        """Get all annotations on this page."""
        if self._doc is None:
            raise RuntimeError("Page has no document reference; cannot get annotations")
        raw = self._doc.annotations(self.number)
        return [
            Annotation(
                type=a["type"],
                rect=tuple(a["rect"]),
                contents=a["contents"],
                author=a["author"],
                color=tuple(a["color"]) if a["color"] else None,
                creation_date=a["creation_date"],
                opacity=a["opacity"],
            )
            for a in raw
        ]

    def extract_images(self) -> list[Image]:
        """Extract all images embedded in this page."""
        if self._doc is None:
            raise RuntimeError("Page has no document reference; cannot extract images")
        raw = self._doc.extract_images(self.number)
        return [
            Image(
                width=img["width"],
                height=img["height"],
                color_space=img["color_space"],
                bits_per_component=img["bits_per_component"],
                filters=img["filters"],
                data=bytes(img["data"]),
            )
            for img in raw
        ]

    def search(
        self,
        query: str,
        *,
        case_sensitive: bool = True,
        use_regex: bool = False,
    ) -> list[SearchResult]:
        """Search for text in this page, returning matches with line info.

        Args:
            query: Text or regex pattern to search for.
            case_sensitive: Whether the search is case-sensitive (default True).
            use_regex: If True, treat query as a regular expression.
        """
        lines = self.extract_text_lines()
        results: list[SearchResult] = []
        if use_regex:
            flags = 0 if case_sensitive else re.IGNORECASE
            pattern = re.compile(query, flags)
            for i, line in enumerate(lines):
                if pattern.search(line.text):
                    results.append(
                        SearchResult(
                            page=self.number,
                            text=line.text,
                            line_number=i + 1,
                            bbox=line.bbox,
                        )
                    )
        else:
            q = query if case_sensitive else query.lower()
            for i, line in enumerate(lines):
                text = line.text if case_sensitive else line.text.lower()
                if q in text:
                    results.append(
                        SearchResult(
                            page=self.number,
                            text=line.text,
                            line_number=i + 1,
                            bbox=line.bbox,
                        )
                    )
        return results

    def extract_structure(
        self,
        *,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
        layout_aware: bool = False,
    ) -> list[ContentBlock]:
        """Extract structured content (headings, paragraphs, lists, tables)."""
        raw_blocks = self._inner.extract_structure(
            heading_size_ratio=heading_size_ratio,
            detect_lists=detect_lists,
            include_tables=include_tables,
            layout_aware=layout_aware,
        )
        return [_raw_block_to_content_block(b) for b in raw_blocks]

    def analyze_layout(
        self,
        *,
        min_gutter_width: float = 20.0,
        max_columns: int = 4,
        detect_headers_footers: bool = True,
        header_zone_fraction: float = 0.08,
        footer_zone_fraction: float = 0.08,
        min_column_line_fraction: float = 0.1,
    ) -> PageLayout:
        """Analyze the page layout to detect columns, headers, and footers."""
        raw = self._inner.analyze_layout(
            min_gutter_width=min_gutter_width,
            max_columns=max_columns,
            detect_headers_footers=detect_headers_footers,
            header_zone_fraction=header_zone_fraction,
            footer_zone_fraction=footer_zone_fraction,
            min_column_line_fraction=min_column_line_fraction,
        )
        return _raw_to_page_layout(raw)

    def extract_text_layout(
        self,
        *,
        min_gutter_width: float = 20.0,
        max_columns: int = 4,
        detect_headers_footers: bool = True,
        header_zone_fraction: float = 0.08,
        footer_zone_fraction: float = 0.08,
        min_column_line_fraction: float = 0.1,
    ) -> str:
        """Extract text in layout-aware reading order."""
        return self._inner.extract_text_layout(
            min_gutter_width=min_gutter_width,
            max_columns=max_columns,
            detect_headers_footers=detect_headers_footers,
            header_zone_fraction=header_zone_fraction,
            footer_zone_fraction=footer_zone_fraction,
            min_column_line_fraction=min_column_line_fraction,
        )

    def to_markdown(
        self,
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
        """Convert page content to Markdown."""
        return self._inner.to_markdown(
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

    def extract_tables(
        self,
        *,
        strategy: TableStrategy | str = TableStrategy.AUTO,
        min_rows: int = 2,
        min_cols: int = 2,
        snap_tolerance: float = 3.0,
        row_tolerance: float = 0.5,
        min_col_gap: float = 10.0,
    ) -> list[Table]:
        """Extract tables from this page."""
        strategy_str = strategy.value if isinstance(strategy, TableStrategy) else str(strategy)
        raw_tables = self._inner.extract_tables(
            strategy=strategy_str,
            min_rows=min_rows,
            min_cols=min_cols,
            snap_tolerance=snap_tolerance,
            row_tolerance=row_tolerance,
            min_col_gap=min_col_gap,
        )
        tables = []
        for rt in raw_tables:
            rows = tuple(
                Row(
                    cells=tuple(
                        Cell(
                            text=c["text"],
                            bbox=tuple(c["bbox"]),
                            col_span=c.get("col_span", 1),
                            row_span=c.get("row_span", 1),
                        )
                        for c in r["cells"]
                    ),
                    y_min=r["y_min"],
                    y_max=r["y_max"],
                )
                for r in rt["rows"]
            )
            tables.append(
                Table(
                    rows=rows,
                    col_count=rt["col_count"],
                    bbox=tuple(rt["bbox"]),
                    strategy=rt["strategy"],
                )
            )
        return tables

    if TYPE_CHECKING:
        # -- Rendering (attached by _render.py) --

        def render(
            self,
            *,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
            background_color: tuple[int, int, int] | None = ...,
            scale_to_width: int | None = ...,
            scale_to_height: int | None = ...,
        ) -> RenderedImage:
            """Render this page to an image.

            Args:
                dpi: Resolution in dots per inch (default 150).
                format: Image format - "png", "jpeg", or "bmp" (default "png").
                quality: JPEG quality 1-100 (default 85, only used for JPEG).
                background_color: RGB tuple (0-255) for background color.
                scale_to_width: Target pixel width (overrides DPI).
                scale_to_height: Target pixel height (overrides DPI).

            Returns:
                A RenderedImage with the image data and dimensions.
            """
            ...

        # -- Async wrappers (attached by _async.py) --

        async def aextract_text(self) -> str: ...

        async def aextract_tables(
            self,
            *,
            strategy: TableStrategy | str = ...,
            min_rows: int = ...,
            min_cols: int = ...,
            snap_tolerance: float = ...,
            row_tolerance: float = ...,
            min_col_gap: float = ...,
        ) -> list[Table]: ...

        async def ato_markdown(self, **kwargs) -> str: ...


def _raw_block_to_content_block(raw: dict) -> ContentBlock:
    """Convert a raw dict from Rust into a ContentBlock dataclass."""
    block_type = raw["type"]
    page = raw.get("page", 0)

    if block_type == "table":
        raw_table = raw["table"]
        rows = tuple(
            Row(
                cells=tuple(
                    Cell(
                        text=c["text"],
                        bbox=tuple(c["bbox"]),
                        col_span=c.get("col_span", 1),
                        row_span=c.get("row_span", 1),
                    )
                    for c in r["cells"]
                ),
                y_min=r["y_min"],
                y_max=r["y_max"],
            )
            for r in raw_table["rows"]
        )
        table = Table(
            rows=rows,
            col_count=raw_table["col_count"],
            bbox=tuple(raw_table["bbox"]),
            strategy=raw_table["strategy"],
        )
        return ContentBlock(type="table", page=page, table=table, bbox=table.bbox)

    return ContentBlock(
        type=block_type,
        page=page,
        text=raw.get("text"),
        level=raw.get("level"),
        indent_level=raw.get("indent_level"),
        bbox=tuple(raw["bbox"]) if "bbox" in raw else None,
    )


def _raw_to_page_layout(raw: dict) -> PageLayout:
    """Convert a raw dict from Rust into a PageLayout dataclass."""
    regions = []
    for r in raw["regions"]:
        lines = tuple(
            TextLine(
                text=ln["text"],
                spans=tuple(TextSpan(**s) for s in ln["spans"]),
                bbox=tuple(ln["bbox"]),
            )
            for ln in r["lines"]
        )
        regions.append(
            LayoutRegion(
                kind=r["kind"],
                column_index=r.get("column_index"),
                bbox=tuple(r["bbox"]),
                lines=lines,
            )
        )
    return PageLayout(
        page_width=raw["page_width"],
        page_height=raw["page_height"],
        column_count=raw["column_count"],
        gutters=tuple(raw["gutters"]),
        regions=tuple(regions),
    )
