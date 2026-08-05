"""Form tools: has_form, get_form_fields, fill_form, modify_form_field, add_form_field."""

from __future__ import annotations

import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, session_manager


@mcp.tool()
@handle_errors
def has_form(session_id: str) -> str:
    """Check if a PDF document contains an interactive form (AcroForm)."""
    _session, doc = session_manager.get_pdf(session_id)
    return json.dumps({"has_form": doc.has_form})


@mcp.tool()
@handle_errors
def get_form_fields(session_id: str) -> str:
    """Extract all form fields from a PDF's AcroForm.

    Returns field names, types, current values, and properties.
    """
    _session, doc = session_manager.get_pdf(session_id)
    fields = doc.form_fields
    return json.dumps({"fields": serialize(fields), "count": len(fields)})


@mcp.tool()
@handle_errors
def fill_form(
    session_id: str,
    values: dict[str, str],
    generate_appearances: bool = False,
) -> str:
    """Fill form fields by name/value pairs. Modifies the session in-place.

    For checkboxes, use "Yes" or "Off". For radio buttons, use the export value.
    Set generate_appearances=True for reliable rendering across all viewers.
    """
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, result = doc.fill_form(values, generate_appearances=generate_appearances)
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))


@mcp.tool()
@handle_errors
def modify_form_field(
    session_id: str,
    field_name: str,
    value: str | None = None,
    default_value: str | None = None,
    read_only: bool | None = None,
    required: bool | None = None,
    max_length: int | None = None,
) -> str:
    """Modify properties of a form field. Modifies the session in-place."""
    _session, doc = session_manager.get_pdf(session_id)
    new_doc, result = doc.modify_form_field(
        field_name,
        value=value,
        default_value=default_value,
        read_only=read_only,
        required=required,
        max_length=max_length,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))


@mcp.tool()
@handle_errors
def add_form_field(
    session_id: str,
    name: str,
    field_type: str,
    page: int = 1,
    rect: list[float] | None = None,
    value: str | None = None,
    default_value: str | None = None,
    read_only: bool = False,
    required: bool = False,
    max_length: int | None = None,
    font_size: float = 0.0,
    generate_appearance: bool = True,
) -> str:
    """Create a new form field on a page. Modifies the session in-place.

    field_type: text, checkbox, radio_button, combo_box, list_box, push_button, signature.
    rect is [x1, y1, x2, y2] in PDF points. Default: [0, 0, 100, 20].
    """
    _session, doc = session_manager.get_pdf(session_id)
    r: tuple[float, float, float, float] = (rect[0], rect[1], rect[2], rect[3]) if rect else (0.0, 0.0, 100.0, 20.0)
    new_doc, result = doc.add_form_field(
        name,
        field_type,
        page=page,
        rect=r,
        value=value,
        default_value=default_value,
        read_only=read_only,
        required=required,
        max_length=max_length,
        font_size=font_size,
        generate_appearance=generate_appearance,
    )
    session_manager.update_document(session_id, new_doc)
    return json.dumps(serialize(result))
