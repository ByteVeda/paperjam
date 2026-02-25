"""paperjam -- Fast PDF processing powered by Rust."""

from __future__ import annotations

__version__ = "0.1.0"

from paperjam._document import Document
from paperjam._enums import (
    AnnotationType,
    Rotation,
    TableStrategy,
    WatermarkLayer,
    WatermarkPosition,
)
from paperjam._functions import merge, merge_files, open
from paperjam._page import Page
from paperjam._paperjam import (
    AnnotationError,
    InvalidPassword,
    OptimizationError,
    PageOutOfRange,
    ParseError,
    PasswordRequired,
    TableExtractionError,
    UnsupportedFeature,
    WatermarkError,
)
from paperjam._paperjam import (
    PaperJamError as PdfError,
)
from paperjam._types import (
    Annotation,
    Bookmark,
    Cell,
    Image,
    Metadata,
    OptimizeResult,
    PageInfo,
    Row,
    SearchResult,
    Table,
    TextLine,
    TextSpan,
)

__all__ = [
    "Annotation",
    "AnnotationError",
    "AnnotationType",
    "Bookmark",
    "Cell",
    "Document",
    "Image",
    "InvalidPassword",
    "Metadata",
    "OptimizationError",
    "OptimizeResult",
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
    "WatermarkError",
    "WatermarkLayer",
    "WatermarkPosition",
    "merge",
    "merge_files",
    "open",
]
