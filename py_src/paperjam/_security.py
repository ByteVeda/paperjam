"""Security methods for Document: sanitize, redact, redact_text."""

from __future__ import annotations

from paperjam import _paperjam
from paperjam._document import Document
from paperjam._types import (
    EncryptResult,
    Permissions,
    RedactedItem,
    RedactRegion,
    RedactResult,
    SanitizedItem,
    SanitizeResult,
)


def _sanitize(
    self: Document,
    *,
    remove_javascript: bool = True,
    remove_embedded_files: bool = True,
    remove_actions: bool = True,
    remove_links: bool = True,
) -> tuple[Document, SanitizeResult]:
    """Remove potentially dangerous objects from the PDF.

    Returns a tuple of (sanitized_document, result_stats).
    """
    inner = self._ensure_open()
    sanitized, stats = _paperjam.sanitize(
        inner,
        remove_javascript,
        remove_embedded_files,
        remove_actions,
        remove_links,
    )
    doc = object.__new__(Document)
    doc._inner = sanitized
    doc._closed = False
    return doc, SanitizeResult(
        javascript_removed=stats["javascript_removed"],
        embedded_files_removed=stats["embedded_files_removed"],
        actions_removed=stats["actions_removed"],
        links_removed=stats["links_removed"],
        items=tuple(SanitizedItem(**item) for item in stats["items"]),
    )


def _redact(
    self: Document,
    regions: list[RedactRegion],
    *,
    fill_color: tuple[float, float, float] | None = None,
) -> tuple[Document, RedactResult]:
    """Redact text within specified regions, removing it from the content stream.

    Args:
        regions: List of RedactRegion specifying areas to redact.
        fill_color: Optional (r, g, b) color for overlay rectangles (0.0-1.0).

    Returns a tuple of (redacted_document, result_stats).
    """
    inner = self._ensure_open()
    region_dicts = [{"page": r.page, "rect": list(r.rect)} for r in regions]
    redacted, stats = _paperjam.redact(
        inner,
        region_dicts,
        list(fill_color) if fill_color else None,
    )
    doc = object.__new__(Document)
    doc._inner = redacted
    doc._closed = False
    return doc, RedactResult(
        pages_modified=stats["pages_modified"],
        items_redacted=stats["items_redacted"],
        items=tuple(
            RedactedItem(
                page=item["page"],
                text=item["text"],
                rect=tuple(item["rect"]),
            )
            for item in stats["items"]
        ),
    )


def _redact_text(
    self: Document,
    query: str,
    *,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: tuple[float, float, float] | None = None,
) -> tuple[Document, RedactResult]:
    """Redact all occurrences of a text query from the document.

    Finds text matching the query, then removes the underlying text
    operators from the content stream (true redaction, not cosmetic).

    Args:
        query: The text or regex pattern to search for and redact.
        case_sensitive: Whether the search is case-sensitive (default True).
        use_regex: If True, treat query as a regular expression.
        fill_color: Optional (r, g, b) color for overlay rectangles (0.0-1.0).

    Returns a tuple of (redacted_document, result_stats).
    """
    inner = self._ensure_open()
    redacted, stats = _paperjam.redact_text(
        inner,
        query,
        case_sensitive,
        use_regex,
        list(fill_color) if fill_color else None,
    )
    doc = object.__new__(Document)
    doc._inner = redacted
    doc._closed = False
    return doc, RedactResult(
        pages_modified=stats["pages_modified"],
        items_redacted=stats["items_redacted"],
        items=tuple(
            RedactedItem(
                page=item["page"],
                text=item["text"],
                rect=tuple(item["rect"]),
            )
            for item in stats["items"]
        ),
    )


def _encrypt(
    self: Document,
    *,
    user_password: str,
    owner_password: str | None = None,
    permissions: Permissions | None = None,
    algorithm: str = "aes128",
) -> tuple[bytes, EncryptResult]:
    """Encrypt the document with user/owner passwords and permission flags.

    Args:
        user_password: Password required to open the document.
        owner_password: Password for full access. Defaults to user_password.
        permissions: Permission flags controlling what viewers can do.
        algorithm: Encryption algorithm — "aes128" (default) or "rc4".

    Returns a tuple of (encrypted_bytes, encrypt_result).
    """
    inner = self._ensure_open()
    perms = permissions or Permissions()
    perms_dict = {
        "print": perms.print,
        "modify": perms.modify,
        "copy": perms.copy,
        "annotate": perms.annotate,
        "fill_forms": perms.fill_forms,
        "accessibility": perms.accessibility,
        "assemble": perms.assemble,
        "print_high_quality": perms.print_high_quality,
    }
    data, stats = _paperjam.encrypt_document(
        inner,
        user_password,
        owner_password,
        perms_dict,
        algorithm,
    )
    return data, EncryptResult(
        algorithm=stats["algorithm"],
        key_length=stats["key_length"],
    )


Document.sanitize = _sanitize  # type: ignore[method-assign]
Document.redact = _redact  # type: ignore[method-assign]
Document.redact_text = _redact_text  # type: ignore[method-assign]
Document.encrypt = _encrypt  # type: ignore[method-assign]
