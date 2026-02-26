"""paperjam -- Fast PDF processing powered by Rust."""

from __future__ import annotations

__version__ = "0.1.0"

from paperjam._document import Document
from paperjam._enums import (
    AnnotationType,
    FormFieldType,
    ImageFormat,
    Rotation,
    TableStrategy,
    WatermarkLayer,
    WatermarkPosition,
)
from paperjam._functions import diff, merge, merge_files, open, to_markdown
from paperjam._render import render
from paperjam._page import Page
from paperjam._paperjam import (
    AnnotationError,
    EncryptionError,
    FormError,
    InvalidPassword,
    OptimizationError,
    PageOutOfRange,
    ParseError,
    PasswordRequired,
    RedactError,
    RenderError,
    SanitizeError,
    SignatureError,
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
    CertificateInfo,
    ChoiceOption,
    ContentBlock,
    DiffOp,
    DiffResult,
    DiffSummary,
    EncryptResult,
    FillFormResult,
    FormField,
    Image,
    LayoutRegion,
    RenderedImage,
    Metadata,
    OptimizeResult,
    PageDiff,
    PageInfo,
    PageLayout,
    Permissions,
    RedactedItem,
    RedactRegion,
    RedactResult,
    Row,
    SanitizedItem,
    SanitizeResult,
    SearchResult,
    SignatureInfo,
    SignatureValidity,
    Table,
    TextLine,
    TextSpan,
)

# Import feature modules to attach methods to Document
import paperjam._extraction  # noqa: F401
import paperjam._manipulation  # noqa: F401
import paperjam._comparison  # noqa: F401
import paperjam._security  # noqa: F401
import paperjam._forms  # noqa: F401
import paperjam._render  # noqa: F401
import paperjam._signature  # noqa: F401

__all__ = [
    "Annotation",
    "AnnotationError",
    "AnnotationType",
    "Bookmark",
    "Cell",
    "CertificateInfo",
    "ChoiceOption",
    "ContentBlock",
    "DiffOp",
    "DiffResult",
    "DiffSummary",
    "Document",
    "EncryptionError",
    "EncryptResult",
    "FillFormResult",
    "FormError",
    "FormField",
    "FormFieldType",
    "Image",
    "ImageFormat",
    "InvalidPassword",
    "LayoutRegion",
    "Metadata",
    "OptimizationError",
    "OptimizeResult",
    "Page",
    "PageDiff",
    "PageInfo",
    "PageLayout",
    "PageOutOfRange",
    "ParseError",
    "PasswordRequired",
    "PdfError",
    "Permissions",
    "RedactedItem",
    "RedactError",
    "RenderedImage",
    "RenderError",
    "RedactRegion",
    "RedactResult",
    "Rotation",
    "Row",
    "SanitizeError",
    "SanitizedItem",
    "SanitizeResult",
    "SearchResult",
    "SignatureError",
    "SignatureInfo",
    "SignatureValidity",
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
    "render",
    "to_markdown",
]
