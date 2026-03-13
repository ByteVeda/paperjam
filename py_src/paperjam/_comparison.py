"""Comparison methods for Document: diff, visual_diff."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import DiffOp, DiffResult, DiffSummary, PageDiff, VisualDiffPage, VisualDiffResult


def _diff(self: Document, other: Document) -> DiffResult:
    """Compare this document with another at the text level.

    Returns a DiffResult with per-page changes and summary statistics.
    """
    inner_a = self._ensure_open()
    inner_b = other._ensure_open()
    raw = _paperjam.diff_documents(inner_a, inner_b)
    return DiffResult(
        page_diffs=tuple(
            PageDiff(
                page=pd["page"],
                ops=tuple(DiffOp(**op) for op in pd["ops"]),
            )
            for pd in raw["page_diffs"]
        ),
        summary=DiffSummary(**raw["summary"]),
    )


def _visual_diff(
    self: Document,
    other: Document,
    *,
    dpi: float = 150.0,
    highlight_color: tuple[int, int, int, int] | None = None,
    mode: str = "both",
    threshold: int = 10,
) -> VisualDiffResult:
    """Compare this document with another visually (pixel-level)."""
    inner_a = self._ensure_open()
    inner_b = other._ensure_open()
    bytes_a = self._raw_bytes or self.save_bytes()
    bytes_b = other._raw_bytes or other.save_bytes()
    color = list(highlight_color) if highlight_color else None
    raw = _paperjam.visual_diff(
        inner_a,
        inner_b,
        bytes_a,
        bytes_b,
        dpi=dpi,
        highlight_color=color,
        mode=mode,
        threshold=threshold,
    )
    return VisualDiffResult(
        pages=tuple(
            VisualDiffPage(
                page=p["page"],
                image_a=bytes(p["image_a"]),
                image_a_width=p["image_a_width"],
                image_a_height=p["image_a_height"],
                image_b=bytes(p["image_b"]),
                image_b_width=p["image_b_width"],
                image_b_height=p["image_b_height"],
                diff_image=bytes(p["diff_image"]),
                diff_image_width=p["diff_image_width"],
                diff_image_height=p["diff_image_height"],
                similarity=p["similarity"],
                changed_pixel_count=p["changed_pixel_count"],
            )
            for p in raw["pages"]
        ),
        overall_similarity=raw["overall_similarity"],
        text_diff_summary=DiffSummary(**raw["text_diff_summary"]),
    )


Document.diff = _diff  # type: ignore[method-assign]
Document.visual_diff = _visual_diff  # type: ignore[method-assign]
