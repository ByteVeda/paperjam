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
from paperjam._functions import diff, merge, merge_files, open
from paperjam._page import Page
from paperjam._paperjam import (
    AnnotationError,
    InvalidPassword,
    OptimizationError,
    PageOutOfRange,
    ParseError,
    PasswordRequired,
    SanitizeError,
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
    ContentBlock,
    DiffOp,
    DiffResult,
    DiffSummary,
    Image,
    Metadata,
    OptimizeResult,
    PageDiff,
    PageInfo,
    Row,
    SanitizedItem,
    SanitizeResult,
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
    "ContentBlock",
    "DiffOp",
    "DiffResult",
    "DiffSummary",
    "Document",
    "Image",
    "InvalidPassword",
    "Metadata",
    "OptimizationError",
    "OptimizeResult",
    "Page",
    "PageDiff",
    "PageInfo",
    "PageOutOfRange",
    "ParseError",
    "PasswordRequired",
    "PdfError",
    "Rotation",
    "Row",
    "SanitizeError",
    "SanitizedItem",
    "SanitizeResult",
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
    "diff",
    "merge",
    "merge_files",
    "open",
]
