"""Form field methods for Document: has_form, form_fields, fill_form, modify_form_field, add_form_field."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import ChoiceOption, CreateFieldResult, FillFormResult, FormField, ModifyFieldResult


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
            options=tuple(ChoiceOption(display=o["display"], export_value=o["export_value"]) for o in f["options"]),
        )
        for f in raw_fields
    ]


def _fill_form(
    self: Document,
    values: dict[str, str],
    *,
    need_appearances: bool = True,
    generate_appearances: bool = False,
) -> tuple[Document, FillFormResult]:
    """Fill form fields by name.

    Args:
        values: Mapping of fully-qualified field names to string values.
            For checkboxes, pass "Yes" or "Off". For radio buttons, pass the export value.
        need_appearances: If True (default), sets /NeedAppearances so viewers
            regenerate field appearances automatically. Ignored if generate_appearances is True.
        generate_appearances: If True, generates explicit /AP streams so forms
            render correctly even in viewers that ignore /NeedAppearances.

    Returns:
        A tuple of (new_document, fill_result).
    """
    inner = self._ensure_open()
    filled, stats = _paperjam.fill_form(inner, values, need_appearances, generate_appearances)  # type: ignore[call-arg]
    doc = object.__new__(Document)
    doc._inner = filled
    doc._closed = False
    return doc, FillFormResult(
        fields_filled=stats["fields_filled"],
        fields_not_found=stats["fields_not_found"],
        not_found_names=tuple(stats["not_found_names"]),
    )


def _modify_form_field(
    self: Document,
    field_name: str,
    *,
    value: str | None = None,
    default_value: str | None = None,
    read_only: bool | None = None,
    required: bool | None = None,
    max_length: int | None = None,
    options: list[ChoiceOption] | None = None,
) -> tuple[Document, ModifyFieldResult]:
    """Modify properties of a form field.

    Args:
        field_name: Fully-qualified field name.
        value: New value for the field.
        default_value: New default value.
        read_only: Set the read-only flag.
        required: Set the required flag.
        max_length: Set max length (text fields).
        options: Replace choice options (combo/list boxes).

    Returns:
        A tuple of (new_document, modify_result).
    """
    inner = self._ensure_open()
    kwargs: dict = {}
    if value is not None:
        kwargs["value"] = value
    if default_value is not None:
        kwargs["default_value"] = default_value
    if read_only is not None:
        kwargs["read_only"] = read_only
    if required is not None:
        kwargs["required"] = required
    if max_length is not None:
        kwargs["max_length"] = max_length
    if options is not None:
        kwargs["options"] = [{"display": o.display, "export_value": o.export_value} for o in options]

    modified, result = _paperjam.modify_form_field(inner, field_name, **kwargs)  # type: ignore[attr-defined]
    doc = object.__new__(Document)
    doc._inner = modified
    doc._closed = False
    return doc, ModifyFieldResult(
        field_name=result["field_name"],
        modified=result["modified"],
    )


def _add_form_field(
    self: Document,
    name: str,
    field_type: str,
    *,
    page: int = 1,
    rect: tuple[float, float, float, float] = (0.0, 0.0, 100.0, 20.0),
    value: str | None = None,
    default_value: str | None = None,
    read_only: bool = False,
    required: bool = False,
    max_length: int | None = None,
    options: list[ChoiceOption] | None = None,
    font_size: float = 0.0,
    generate_appearance: bool = True,
) -> tuple[Document, CreateFieldResult]:
    """Create a new form field on a page.

    Args:
        name: Fully-qualified field name.
        field_type: One of "text", "checkbox", "radio_button", "combo_box",
            "list_box", "push_button", "signature".
        page: 1-based page number (default 1).
        rect: Field rectangle (x1, y1, x2, y2) in PDF points.
        value: Initial value.
        default_value: Default value.
        read_only: Whether the field is read-only.
        required: Whether the field is required.
        max_length: Maximum length for text fields.
        options: Choice options for combo/list boxes.
        font_size: Font size (0 = auto).
        generate_appearance: Whether to generate an appearance stream (default True).

    Returns:
        A tuple of (new_document, create_result).
    """
    inner = self._ensure_open()
    opts_dicts = None
    if options is not None:
        opts_dicts = [{"display": o.display, "export_value": o.export_value} for o in options]

    created, result = _paperjam.add_form_field(  # type: ignore[attr-defined]
        inner,
        name,
        field_type,
        page,
        rect,
        value,
        default_value,
        read_only,
        required,
        max_length,
        opts_dicts,
        font_size,
        generate_appearance,
    )
    doc = object.__new__(Document)
    doc._inner = created
    doc._closed = False
    return doc, CreateFieldResult(
        field_name=result["field_name"],
        created=result["created"],
    )


Document.has_form = property(_has_form_getter)  # type: ignore[assignment, method-assign]
Document.form_fields = property(_form_fields_getter)  # type: ignore[assignment, method-assign]
Document.fill_form = _fill_form  # type: ignore[method-assign]
Document.modify_form_field = _modify_form_field  # type: ignore[method-assign]
Document.add_form_field = _add_form_field  # type: ignore[method-assign]
