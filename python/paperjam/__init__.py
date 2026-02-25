"""paperjam -- Fast PDF processing powered by Rust."""

from __future__ import annotations

__version__ = "0.1.0"

from paperjam._document import Document
from paperjam._enums import Rotation, TableStrategy
from paperjam._functions import merge, merge_files, open
from paperjam._page import Page
from paperjam._paperjam import (
    InvalidPassword,
    PageOutOfRange,
    ParseError,
    PasswordRequired,
    TableExtractionError,
    UnsupportedFeature,
)
from paperjam._paperjam import (
    PaperJamError as PdfError,
)
from paperjam._types import (
    Bookmark,
    Cell,
    Image,
    Metadata,
    PageInfo,
    Row,
    SearchResult,
    Table,
    TextLine,
    TextSpan,
)

__all__ = [
    "Bookmark",
    "Cell",
    "Document",
    "Image",
    "InvalidPassword",
    "Metadata",
    "Page",
    "PageInfo",
    "PageOutOfRange",
    "ParseError",
    "PasswordRequired",
    "PdfError",
    "Rotation",
    "Row",
    "SearchResult",
    "Table",
    "TableExtractionError",
    "TableStrategy",
    "TextLine",
    "TextSpan",
    "UnsupportedFeature",
    "merge",
    "merge_files",
    "open",
]
