"""Form field methods for Document: has_form, form_fields, fill_form."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import ChoiceOption, FillFormResult, FormField


def _has_form_getter(self: Document) -> bool:
    """Whether the document contains an interactive form (AcroForm)."""
    inner = self._ensure_open()
    return inner.has_form()


def _form_fields_getter(self: Document) -> list[FormField]:
    """Extract all form fields from the document's AcroForm."""
    inner = self._ensure_open()
    raw_fields = inner.form_fields()
    return [
        FormField(
            name=f["name"],
            field_type=f["field_type"],
            value=f["value"],
            default_value=f["default_value"],
            page=f["page"],
            rect=tuple(f["rect"]) if f["rect"] is not None else None,
            read_only=f["read_only"],
            required=f["required"],
            max_length=f["max_length"],
            options=tuple(
                ChoiceOption(display=o["display"], export_value=o["export_value"])
                for o in f["options"]
            ),
        )
        for f in raw_fields
    ]


def _fill_form(
    self: Document,
    values: dict[str, str],
    *,
    need_appearances: bool = True,
) -> tuple[Document, FillFormResult]:
    """Fill form fields by name.

    Args:
        values: Mapping of fully-qualified field names to string values.
        need_appearances: If True (default), sets /NeedAppearances so viewers
            regenerate field appearances automatically.

    Returns:
        A tuple of (new_document, fill_result).
    """
    inner = self._ensure_open()
    filled, stats = _paperjam.fill_form(inner, values, need_appearances)
    doc = object.__new__(Document)
    doc._inner = filled
    doc._closed = False
    return doc, FillFormResult(
        fields_filled=stats["fields_filled"],
        fields_not_found=stats["fields_not_found"],
        not_found_names=tuple(stats["not_found_names"]),
    )


Document.has_form = property(_has_form_getter)  # type: ignore[assignment, method-assign]
Document.form_fields = property(_form_fields_getter)  # type: ignore[assignment, method-assign]
Document.fill_form = _fill_form  # type: ignore[method-assign]
