"""Page class wrapping the Rust page object."""

from __future__ import annotations

from typing import Any

from paperjam import _paperjam  # noqa: TC001
from paperjam._enums import TableStrategy
from paperjam._types import Annotation, Cell, ContentBlock, Image, PageInfo, Row, SearchResult, Table, TextLine, TextSpan


class Page:
    """A single page in a PDF document.

    Pages are lazily parsed -- content is only decoded when you call
    extraction methods like extract_text() or extract_tables().
    """

    __slots__ = ("_doc", "_inner")

    _inner: Any
    _doc: Any  # RustDocument reference for document-level operations

    def __init__(self) -> None:
        raise TypeError(
            "Page objects cannot be created directly. Access via Document.pages."
        )

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
    ) -> list[SearchResult]:
        """Search for text in this page, returning matches with line info."""
        lines = self.extract_text_lines()
        results: list[SearchResult] = []
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
    ) -> list[ContentBlock]:
        """Extract structured content (headings, paragraphs, lists, tables)."""
        raw_blocks = self._inner.extract_structure(
            heading_size_ratio=heading_size_ratio,
            detect_lists=detect_lists,
            include_tables=include_tables,
        )
        return [_raw_block_to_content_block(b) for b in raw_blocks]

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
