"""Base classes for mixin type checking.

Mixins reference attributes like _ensure_open(), _raw_bytes, _inner, etc.
that are defined on Document/Page. These base classes tell mypy what mixins
expect from the class they'll be composed into.

At runtime, mixins inherit from ``object`` (via the conditional base pattern
in each mixin file). Only during type-checking do they inherit from these
bases, so there is zero runtime cost.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Self

if TYPE_CHECKING:
    from paperjam import _paperjam


class DocumentBase:
    """Base describing attributes that Document mixins can use."""

    _closed: bool
    _inner: _paperjam.RustDocument
    _raw_bytes: bytes | None

    def _ensure_open(self) -> _paperjam.RustDocument:
        raise NotImplementedError

    def _new_instance(self) -> Self:
        """Create a new uninitialised instance of the concrete class."""
        return object.__new__(type(self))  # type: ignore[return-value]

    def save_bytes(self) -> bytes:
        raise NotImplementedError

    @property
    def page_count(self) -> int:
        raise NotImplementedError

    @property
    def pages(self) -> Any:
        raise NotImplementedError


class PageBase:
    """Base describing attributes that Page mixins can use."""

    _inner: _paperjam.RustPage
    _doc: _paperjam.RustDocument | None

    @property
    def number(self) -> int:
        raise NotImplementedError
