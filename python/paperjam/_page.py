"""Page class wrapping the Rust page object."""

from __future__ import annotations

from typing import Any

from paperjam import _paperjam  # noqa: TC001
from paperjam._enums import TableStrategy
from paperjam._types import Cell, PageInfo, Row, Table, TextLine, TextSpan


class Page:
    """A single page in a PDF document.

    Pages are lazily parsed -- content is only decoded when you call
    extraction methods like extract_text() or extract_tables().
    """

    __slots__ = ("_inner",)

    _inner: Any

    def __init__(self) -> None:
        raise TypeError(
            "Page objects cannot be created directly. Access via Document.pages."
        )

    @classmethod
    def _from_rust(cls, rust_page: _paperjam.RustPage) -> Page:
        obj = object.__new__(cls)
        obj._inner = rust_page
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
