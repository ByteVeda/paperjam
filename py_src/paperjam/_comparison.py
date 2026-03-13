"""Comparison methods for Document: diff."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import DiffOp, DiffResult, DiffSummary, PageDiff


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


Document.diff = _diff  # type: ignore[method-assign]
