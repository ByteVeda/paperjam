"""PDF/A conversion methods for Document."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import ConversionAction, ConversionResult, ValidationIssue


def _convert_to_pdf_a(
    self: Document,
    *,
    level: str = "1b",
    force: bool = False,
) -> tuple[Document, ConversionResult]:
    """Convert the document to PDF/A conformance.

    Performs: XMP metadata update, sRGB OutputIntent embedding,
    JavaScript/action removal, transparency removal (PDF/A-1),
    and encryption removal.

    Font embedding is not performed — documents with unembedded fonts
    will have those reported as remaining issues.

    Args:
        level: Target conformance level — "1b" (default), "1a", or "2b".
        force: If True, proceed even when some issues cannot be fixed.

    Returns:
        A tuple of (converted_document, conversion_result).
    """
    inner = self._ensure_open()
    new_inner, result_dict = _paperjam.convert_to_pdf_a(inner, level=level, force=force)

    new_doc = Document.__new__(Document)
    new_doc._inner = new_inner
    new_doc._raw_bytes = None
    new_doc._closed = False

    actions = tuple(
        ConversionAction(
            category=a["category"],
            description=a["description"],
            page=a.get("page"),
        )
        for a in result_dict["actions_taken"]
    )

    issues = tuple(
        ValidationIssue(
            severity=i["severity"],
            rule=i["rule"],
            message=i["message"],
            page=i.get("page"),
        )
        for i in result_dict["remaining_issues"]
    )

    result = ConversionResult(
        level=result_dict["level"],
        success=result_dict["success"],
        actions_taken=actions,
        remaining_issues=issues,
    )

    return new_doc, result


Document.convert_to_pdf_a = _convert_to_pdf_a  # type: ignore[attr-defined]
