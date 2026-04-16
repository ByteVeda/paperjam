#!/usr/bin/env python3
"""Generate deterministic synthetic PDF fixtures for the table-extraction accuracy harness.

Each fixture is described by a ``TableSpec`` that is the single source of truth for BOTH
the rendered PDF and the ground-truth JSON sidecar — by construction they cannot drift.

Usage:
    uv run python scripts/generate_table_fixtures.py
"""

from __future__ import annotations

import json
import pathlib
from dataclasses import dataclass

import reportlab.rl_config

reportlab.rl_config.invariant = 1  # deterministic PDF metadata (fixed /CreationDate, /ID, etc.)

from reportlab.lib.pagesizes import landscape, letter  # noqa: E402
from reportlab.pdfgen import canvas  # noqa: E402

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "tests" / "fixtures" / "tables"

FONT_REGULAR = "Helvetica"
FONT_BOLD = "Helvetica-Bold"
FONT_SIZE = 10


@dataclass(frozen=True)
class CellSpec:
    row: int
    col: int
    text: str
    row_span: int = 1
    col_span: int = 1


@dataclass
class TableSpec:
    """One source of truth for a single-table single-page fixture."""

    name: str
    page_size: tuple[float, float]
    bordered: bool
    origin: tuple[float, float]  # (x_left, y_top) in PDF coords (bottom-left origin)
    col_widths: list[float]
    row_heights: list[float]
    cells: list[CellSpec]
    notes: str
    bold_header: bool = False


def _simple_cells(data: list[list[str]]) -> list[CellSpec]:
    return [CellSpec(row=r, col=c, text=t) for r, row in enumerate(data) for c, t in enumerate(row)]


def _render_table(c: canvas.Canvas, spec: TableSpec, *, page_y_top: float | None = None) -> None:
    """Render one table onto the current canvas page, using spec.origin (or override y)."""
    x_left, y_top = spec.origin
    if page_y_top is not None:
        y_top = page_y_top
    xs = [x_left]
    for w in spec.col_widths:
        xs.append(xs[-1] + w)
    ys = [y_top]
    for h in spec.row_heights:
        ys.append(ys[-1] - h)

    for cell in spec.cells:
        x1 = xs[cell.col]
        x2 = xs[cell.col + cell.col_span]
        y1 = ys[cell.row + cell.row_span]
        y2 = ys[cell.row]
        if spec.bordered:
            c.setStrokeColorRGB(0, 0, 0)
            c.setLineWidth(0.5)
            c.rect(x1, y1, x2 - x1, y2 - y1)
        font = FONT_BOLD if (spec.bold_header and cell.row == 0) else FONT_REGULAR
        c.setFont(font, FONT_SIZE)
        text_w = c.stringWidth(cell.text, font, FONT_SIZE)
        tx = (x1 + x2) / 2 - text_w / 2
        ty = (y1 + y2) / 2 - FONT_SIZE / 3  # rough vertical centering
        c.drawString(tx, ty, cell.text)


def _table_bbox(spec: TableSpec) -> list[float]:
    x_left, y_top = spec.origin
    return [
        x_left,
        y_top - sum(spec.row_heights),
        x_left + sum(spec.col_widths),
        y_top,
    ]


def _gt_for_spec(spec: TableSpec, *, page: int = 1) -> dict:
    return {
        "page": page,
        "bbox": _table_bbox(spec),
        "col_count": len(spec.col_widths),
        "row_count": len(spec.row_heights),
        "cells": [
            {
                "row": c.row,
                "col": c.col,
                "row_span": c.row_span,
                "col_span": c.col_span,
                "text": c.text,
            }
            for c in spec.cells
        ],
    }


def _canvas_for(name: str, page_size: tuple[float, float]) -> canvas.Canvas:
    pdf_path = OUT_DIR / f"{name}.pdf"
    c = canvas.Canvas(str(pdf_path), pagesize=page_size, pageCompression=0)
    c.setCreator("paperjam-test-fixtures")
    c.setTitle(name)
    c.setSubject("Table extraction accuracy fixture")
    return c


def _write_gt(name: str, notes: str, tables: list[dict]) -> None:
    data = {
        "source": "synthetic",
        "license": "generated",
        "notes": notes,
        "tables": tables,
    }
    (OUT_DIR / f"{name}.gt.json").write_text(json.dumps(data, indent=2) + "\n")


def emit_single_table_fixture(spec: TableSpec) -> None:
    c = _canvas_for(spec.name, spec.page_size)
    _render_table(c, spec)
    c.showPage()
    c.save()
    _write_gt(spec.name, spec.notes, [_gt_for_spec(spec)])


def emit_multipage_continuation() -> None:
    name = "multipage_continuation"
    header = ["Year", "Metric", "Value"]
    page1_rows = [[f"201{i}", "Revenue", str(100 * (i + 1))] for i in range(6)]
    page2_rows = [[f"202{i}", "Revenue", str(200 * (i + 1))] for i in range(6)]

    col_widths = [100.0, 120.0, 100.0]
    row_heights = [24.0] * 7  # header + 6 data rows
    origin = (72.0, 720.0)

    c = _canvas_for(name, letter)
    for page_data in (page1_rows, page2_rows):
        spec = TableSpec(
            name=name,
            page_size=letter,
            bordered=True,
            origin=origin,
            col_widths=col_widths,
            row_heights=row_heights,
            cells=_simple_cells([header, *page_data]),
            notes="",
        )
        _render_table(c, spec)
        c.showPage()
    c.save()

    def _page_gt(page_idx: int, rows: list[list[str]]) -> dict:
        cells = [{"row": r, "col": cc, "row_span": 1, "col_span": 1, "text": t} for r, row in enumerate([header, *rows]) for cc, t in enumerate(row)]
        return {
            "page": page_idx,
            "bbox": [origin[0], origin[1] - sum(row_heights), origin[0] + sum(col_widths), origin[1]],
            "col_count": len(col_widths),
            "row_count": len(row_heights),
            "cells": cells,
        }

    _write_gt(
        name,
        "2-page table, header repeats on page 2; Phase 0 scores per-page tables",
        [_page_gt(1, page1_rows), _page_gt(2, page2_rows)],
    )


def build_fixtures() -> list[TableSpec]:
    specs: list[TableSpec] = [
        TableSpec(
            name="bordered_simple",
            page_size=letter,
            bordered=True,
            origin=(72.0, 700.0),
            col_widths=[100.0, 100.0, 100.0, 100.0],
            row_heights=[24.0, 24.0, 24.0],
            cells=_simple_cells(
                [
                    ["Header A", "Header B", "Header C", "Header D"],
                    ["a1", "b1", "c1", "d1"],
                    ["a2", "b2", "c2", "d2"],
                ]
            ),
            notes="3x4 fully bordered, single-line cells",
        ),
        TableSpec(
            name="bordered_dense",
            page_size=letter,
            bordered=True,
            origin=(54.0, 720.0),
            col_widths=[60.0] * 6,
            row_heights=[22.0] * 8,
            cells=_simple_cells(
                [
                    ["Year", "Q1", "Q2", "Q3", "Q4", "Total"],
                    ["2016", "100", "110", "120", "130", "460"],
                    ["2017", "105", "115", "125", "135", "480"],
                    ["2018", "110", "120", "130", "140", "500"],
                    ["2019", "115", "125", "135", "145", "520"],
                    ["2020", "120", "130", "140", "150", "540"],
                    ["2021", "125", "135", "145", "155", "560"],
                    ["2022", "130", "140", "150", "160", "580"],
                ]
            ),
            notes="8x6 bordered with numeric data, tight spacing",
        ),
        TableSpec(
            name="bordered_merged",
            page_size=letter,
            bordered=True,
            origin=(72.0, 700.0),
            col_widths=[80.0, 80.0, 80.0, 80.0],
            row_heights=[24.0] * 5,
            cells=[
                CellSpec(row=0, col=0, text="Year", row_span=2, col_span=1),
                CellSpec(row=0, col=1, text="Revenue", row_span=1, col_span=2),
                CellSpec(row=0, col=3, text="Notes", row_span=2, col_span=1),
                CellSpec(row=1, col=1, text="Q1"),
                CellSpec(row=1, col=2, text="Q2"),
                CellSpec(row=2, col=0, text="2020"),
                CellSpec(row=2, col=1, text="100"),
                CellSpec(row=2, col=2, text="120"),
                CellSpec(row=2, col=3, text="stable"),
                CellSpec(row=3, col=0, text="2021"),
                CellSpec(row=3, col=1, text="110"),
                CellSpec(row=3, col=2, text="130"),
                CellSpec(row=3, col=3, text="up"),
                CellSpec(row=4, col=0, text="2022"),
                CellSpec(row=4, col=1, text="120"),
                CellSpec(row=4, col=2, text="140"),
                CellSpec(row=4, col=3, text="up"),
            ],
            notes="5x4 with row-span merges (Year, Notes) and col-span merge (Revenue)",
        ),
        TableSpec(
            name="borderless_financial",
            page_size=letter,
            bordered=False,
            origin=(72.0, 700.0),
            col_widths=[120.0, 80.0, 80.0, 80.0],
            row_heights=[24.0] * 6,
            cells=_simple_cells(
                [
                    ["Metric", "2021", "2022", "2023"],
                    ["Revenue", "1,000", "1,200", "1,500"],
                    ["COGS", "600", "720", "900"],
                    ["Gross", "400", "480", "600"],
                    ["OpEx", "200", "240", "300"],
                    ["Net", "200", "240", "300"],
                ]
            ),
            notes="6x4 borderless financial statement",
        ),
        TableSpec(
            name="borderless_invoice",
            page_size=letter,
            bordered=False,
            origin=(72.0, 700.0),
            col_widths=[160.0, 60.0, 80.0],
            row_heights=[24.0] * 5,
            cells=_simple_cells(
                [
                    ["Item", "Qty", "Price"],
                    ["Widget A", "2", "10.00"],
                    ["Widget B", "1", "25.00"],
                    ["Gadget", "3", "15.00"],
                    ["Gizmo", "5", "5.00"],
                ]
            ),
            notes="5x3 borderless invoice with bold header row",
            bold_header=True,
        ),
        TableSpec(
            name="rotated_landscape",
            page_size=landscape(letter),
            bordered=True,
            origin=(72.0, 500.0),
            col_widths=[100.0] * 5,
            row_heights=[24.0] * 4,
            cells=_simple_cells(
                [
                    ["Col 1", "Col 2", "Col 3", "Col 4", "Col 5"],
                    ["a", "b", "c", "d", "e"],
                    ["f", "g", "h", "i", "j"],
                    ["k", "l", "m", "n", "o"],
                ]
            ),
            notes="4x5 bordered on a landscape-oriented page",
        ),
        TableSpec(
            name="sparse_cells",
            page_size=letter,
            bordered=False,
            origin=(72.0, 700.0),
            col_widths=[90.0, 60.0, 70.0, 70.0, 70.0],
            row_heights=[24.0] * 7,
            cells=_simple_cells(
                [
                    ["Item", "Qty", "Price", "Total", "Note"],
                    ["Apple", "2", "1.00", "2.00", ""],
                    ["Banana", "5", "0.50", "2.50", ""],
                    ["Subtotal", "", "", "4.50", ""],
                    ["Orange", "3", "1.50", "4.50", ""],
                    ["Grape", "1", "3.00", "3.00", ""],
                    ["Total", "", "", "12.00", ""],
                ]
            ),
            notes="7x5 borderless with blank cells on subtotal/total rows",
        ),
    ]
    return specs


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for spec in build_fixtures():
        emit_single_table_fixture(spec)
        print(f"  wrote {spec.name}.pdf + .gt.json")
    emit_multipage_continuation()
    print("  wrote multipage_continuation.pdf + .gt.json")
    print(f"Done. Fixtures in {OUT_DIR.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
