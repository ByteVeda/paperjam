"""PDF/A validation for Document."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import ValidationIssue, ValidationReport


def _validate_pdf_a(
    self: Document,
    level: str = "1b",
) -> ValidationReport:
    """Validate PDF/A compliance.

    Args:
        level: PDF/A level to validate against — "1b", "1a", or "2b" (default "1b").
    """
    inner = self._ensure_open()
    raw = _paperjam.validate_pdf_a(inner, level=level)
    return ValidationReport(
        level=raw["level"],
        is_compliant=raw["is_compliant"],
        issues=tuple(
            ValidationIssue(
                severity=i["severity"],
                rule=i["rule"],
                message=i["message"],
                page=i.get("page"),
            )
            for i in raw["issues"]
        ),
        fonts_checked=raw["fonts_checked"],
        pages_checked=raw["pages_checked"],
    )


Document.validate_pdf_a = _validate_pdf_a  # type: ignore[method-assign]
