"""paperjam -- Fast PDF processing powered by Rust."""

from __future__ import annotations

__version__ = "0.1.0"

import paperjam._async
import paperjam._comparison
import paperjam._conversion

# Import feature modules to attach methods to Document
import paperjam._extraction
import paperjam._forms
import paperjam._manipulation
import paperjam._render
import paperjam._security
import paperjam._signature
import paperjam._stamp
import paperjam._toc
import paperjam._validation  # noqa: F401
from paperjam._async import (
    amerge,
    aopen,
    arender,
    ato_markdown,
)
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
from paperjam._render import render
from paperjam._types import (
    Annotation,
    Bookmark,
    Cell,
    CertificateInfo,
    ChoiceOption,
    ContentBlock,
    ConversionAction,
    ConversionResult,
    CreateFieldResult,
    DiffOp,
    DiffResult,
    DiffSummary,
    EncryptResult,
    FillFormResult,
    FormField,
    Image,
    LayoutRegion,
    Link,
    Metadata,
    ModifyFieldResult,
    OptimizeResult,
    PageDiff,
    PageInfo,
    PageLayout,
    PdfUaReport,
    Permissions,
    RedactedItem,
    RedactRegion,
    RedactResult,
    RenderedImage,
    Row,
    SanitizedItem,
    SanitizeResult,
    SearchResult,
    SignatureInfo,
    SignatureValidity,
    Table,
    TextLine,
    TextSpan,
    ValidationIssue,
    ValidationReport,
    VisualDiffPage,
    VisualDiffResult,
)

__all__ = [
    "Annotation",
    "AnnotationError",
    "AnnotationType",
    "Bookmark",
    "Cell",
    "CertificateInfo",
    "ChoiceOption",
    "ContentBlock",
    "ConversionAction",
    "ConversionResult",
    "CreateFieldResult",
    "DiffOp",
    "DiffResult",
    "DiffSummary",
    "Document",
    "EncryptResult",
    "EncryptionError",
    "FillFormResult",
    "FormError",
    "FormField",
    "FormFieldType",
    "Image",
    "ImageFormat",
    "InvalidPassword",
    "LayoutRegion",
    "Link",
    "Metadata",
    "ModifyFieldResult",
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
    "PdfUaReport",
    "Permissions",
    "RedactError",
    "RedactRegion",
    "RedactResult",
    "RedactedItem",
    "RenderError",
    "RenderedImage",
    "Rotation",
    "Row",
    "SanitizeError",
    "SanitizeResult",
    "SanitizedItem",
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
    "ValidationIssue",
    "ValidationReport",
    "VisualDiffPage",
    "VisualDiffResult",
    "WatermarkError",
    "WatermarkLayer",
    "WatermarkPosition",
    "amerge",
    "aopen",
    "arender",
    "ato_markdown",
    "diff",
    "merge",
    "merge_files",
    "open",
    "render",
    "to_markdown",
]
