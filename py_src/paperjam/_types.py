"""Structured result types and exceptions for paperjam."""

from __future__ import annotations

import dataclasses

# --- Exceptions (kept for backwards compatibility, re-exported from _paperjam in __init__) ---


class PdfError(Exception):
    """Base exception for all paperjam errors."""


class ParseError(PdfError):
    """Error parsing the PDF structure."""


class PasswordRequired(PdfError):  # noqa: N818
    """The PDF is encrypted and requires a password."""


class InvalidPassword(PdfError):  # noqa: N818
    """The provided password is incorrect."""


class PageOutOfRange(PdfError, IndexError):  # noqa: N818
    """Page number is out of range."""


class UnsupportedFeature(PdfError):  # noqa: N818
    """The PDF uses a feature not supported by paperjam."""


class TableExtractionError(PdfError):
    """Error during table extraction."""


# --- Data types ---


@dataclasses.dataclass(frozen=True, slots=True)
class TextSpan:
    """A positioned piece of text on a page."""

    text: str
    x: float
    y: float
    width: float
    font_size: float
    font_name: str


@dataclasses.dataclass(frozen=True, slots=True)
class TextLine:
    """A line of text composed of multiple spans."""

    text: str
    spans: tuple[TextSpan, ...]
    bbox: tuple[float, float, float, float]


@dataclasses.dataclass(frozen=True, slots=True)
class Cell:
    """A single cell in a table."""

    text: str
    bbox: tuple[float, float, float, float]
    col_span: int = 1
    row_span: int = 1


@dataclasses.dataclass(frozen=True, slots=True)
class Row:
    """A row in a table."""

    cells: tuple[Cell, ...]
    y_min: float
    y_max: float


@dataclasses.dataclass(frozen=True, slots=True)
class Table:
    """An extracted table from a PDF page."""

    rows: tuple[Row, ...]
    col_count: int
    bbox: tuple[float, float, float, float]
    strategy: str

    @property
    def row_count(self) -> int:
        """Number of rows in the table."""
        return len(self.rows)

    def cell(self, row: int, col: int) -> Cell | None:
        """Get cell at (row, col), both 0-indexed."""
        if 0 <= row < len(self.rows) and 0 <= col < len(self.rows[row].cells):
            return self.rows[row].cells[col]
        return None

    def to_list(self) -> list[list[str]]:
        """Convert to 2D list of strings."""
        return [[c.text for c in r.cells] for r in self.rows]

    def to_csv(self, delimiter: str = ",") -> str:
        """Convert to CSV string."""
        import csv
        import io

        buf = io.StringIO()
        writer = csv.writer(buf, delimiter=delimiter)
        for row in self.rows:
            writer.writerow([c.text for c in row.cells])
        return buf.getvalue()

    def to_dataframe(self):
        """Convert to pandas DataFrame. First row used as headers."""
        import pandas as pd  # type: ignore[import-untyped]

        data = self.to_list()
        if len(data) > 1:
            return pd.DataFrame(data[1:], columns=data[0])
        return pd.DataFrame(data)


@dataclasses.dataclass(frozen=True, slots=True)
class Metadata:
    """PDF document metadata."""

    title: str | None
    author: str | None
    subject: str | None
    keywords: str | None
    creator: str | None
    producer: str | None
    creation_date: str | None
    modification_date: str | None
    pdf_version: str
    page_count: int
    is_encrypted: bool
    xmp_metadata: str | None


@dataclasses.dataclass(frozen=True, slots=True)
class PageInfo:
    """Basic information about a page."""

    number: int
    width: float
    height: float
    rotation: int


@dataclasses.dataclass(frozen=True, slots=True)
class Image:
    """An extracted image from a PDF page."""

    width: int
    height: int
    color_space: str | None
    bits_per_component: int | None
    filters: list[str]
    data: bytes

    def save(self, path: str) -> None:
        """Write the raw image bytes to a file."""
        with open(path, "wb") as f:
            f.write(self.data)


@dataclasses.dataclass(frozen=True, slots=True)
class Bookmark:
    """A bookmark/outline entry from the document's TOC."""

    title: str
    page: int
    level: int
    children: tuple[Bookmark, ...] = ()


@dataclasses.dataclass(frozen=True, slots=True)
class SearchResult:
    """A text search match within the document."""

    page: int
    text: str
    line_number: int
    bbox: tuple[float, float, float, float] | None


@dataclasses.dataclass(frozen=True, slots=True)
class OptimizeResult:
    """Statistics from PDF optimization."""

    original_size: int
    optimized_size: int
    objects_removed: int
    streams_compressed: int

    @property
    def reduction_percent(self) -> float:
        """Percentage size reduction achieved."""
        if self.original_size == 0:
            return 0.0
        return (1 - self.optimized_size / self.original_size) * 100


@dataclasses.dataclass(frozen=True, slots=True)
class Annotation:
    """A PDF annotation extracted from a page."""

    type: str
    rect: tuple[float, float, float, float]
    contents: str | None = None
    author: str | None = None
    color: tuple[float, float, float] | None = None
    creation_date: str | None = None
    opacity: float | None = None
    url: str | None = None
    destination: dict[str, object] | None = None


@dataclasses.dataclass(frozen=True, slots=True)
class Link:
    """A hyperlink extracted from a PDF page."""

    page: int
    rect: tuple[float, float, float, float]
    url: str | None = None
    destination: dict[str, object] | None = None
    contents: str | None = None


# --- Structure extraction types ---


@dataclasses.dataclass(frozen=True, slots=True)
class ContentBlock:
    """A block of structured content extracted from a page."""

    type: str  # "heading", "paragraph", "list_item", "table"
    page: int
    text: str | None = None
    level: int | None = None  # heading level (1-6), only for headings
    indent_level: int | None = None  # only for list items
    bbox: tuple[float, float, float, float] | None = None
    table: Table | None = None  # only for type="table"


# --- Diff types ---


@dataclasses.dataclass(frozen=True, slots=True)
class DiffOp:
    """A single change detected between two documents."""

    kind: str  # "added", "removed", "changed"
    page: int
    text_a: str | None = None
    text_b: str | None = None
    bbox_a: tuple[float, float, float, float] | None = None
    bbox_b: tuple[float, float, float, float] | None = None
    line_index_a: int | None = None
    line_index_b: int | None = None


@dataclasses.dataclass(frozen=True, slots=True)
class PageDiff:
    """Diff results for a single page."""

    page: int
    ops: tuple[DiffOp, ...]


@dataclasses.dataclass(frozen=True, slots=True)
class DiffSummary:
    """Summary statistics from a document diff."""

    pages_changed: int
    pages_added: int
    pages_removed: int
    total_additions: int
    total_removals: int
    total_changes: int


@dataclasses.dataclass(frozen=True, slots=True)
class DiffResult:
    """Complete diff result between two documents."""

    page_diffs: tuple[PageDiff, ...]
    summary: DiffSummary


# --- Sanitize types ---


@dataclasses.dataclass(frozen=True, slots=True)
class SanitizedItem:
    """A single item found and removed during sanitization."""

    category: str
    description: str
    page: int | None = None


@dataclasses.dataclass(frozen=True, slots=True)
class SanitizeResult:
    """Statistics from PDF sanitization."""

    javascript_removed: int
    embedded_files_removed: int
    actions_removed: int
    links_removed: int
    items: tuple[SanitizedItem, ...]

    @property
    def total_removed(self) -> int:
        """Total number of items removed."""
        return self.javascript_removed + self.embedded_files_removed + self.actions_removed + self.links_removed


# --- Redaction types ---


@dataclasses.dataclass(frozen=True, slots=True)
class RedactRegion:
    """A rectangular region to redact on a specific page."""

    page: int
    rect: tuple[float, float, float, float]  # (x1, y1, x2, y2) in PDF coordinates


@dataclasses.dataclass(frozen=True, slots=True)
class RedactedItem:
    """A single item that was redacted."""

    page: int
    text: str
    rect: tuple[float, float, float, float]


@dataclasses.dataclass(frozen=True, slots=True)
class RedactResult:
    """Statistics from PDF redaction."""

    pages_modified: int
    items_redacted: int
    items: tuple[RedactedItem, ...]


# --- Render types ---


@dataclasses.dataclass(frozen=True, slots=True)
class RenderedImage:
    """A rendered page image."""

    data: bytes
    width: int
    height: int
    format: str  # "png", "jpeg", "bmp"
    page: int

    def save(self, path: str) -> None:
        """Write the image data to a file."""
        with open(path, "wb") as f:
            f.write(self.data)


# --- Form types ---


@dataclasses.dataclass(frozen=True, slots=True)
class ChoiceOption:
    """An option in a choice field (combo box or list box)."""

    display: str
    export_value: str


@dataclasses.dataclass(frozen=True, slots=True)
class FormField:
    """A form field extracted from the PDF's AcroForm."""

    name: str
    field_type: str  # "text", "checkbox", "radio_button", "combo_box", "list_box", etc.
    value: str | None = None
    default_value: str | None = None
    page: int | None = None
    rect: tuple[float, float, float, float] | None = None
    read_only: bool = False
    required: bool = False
    max_length: int = 0
    options: tuple[ChoiceOption, ...] = ()


@dataclasses.dataclass(frozen=True, slots=True)
class FillFormResult:
    """Result of a form fill operation."""

    fields_filled: int
    fields_not_found: int
    not_found_names: tuple[str, ...] = ()


@dataclasses.dataclass(frozen=True, slots=True)
class ModifyFieldResult:
    """Result of a field modification operation."""

    field_name: str
    modified: bool


@dataclasses.dataclass(frozen=True, slots=True)
class CreateFieldResult:
    """Result of a field creation operation."""

    field_name: str
    created: bool


# --- Signature types ---


@dataclasses.dataclass(frozen=True, slots=True)
class CertificateInfo:
    """Basic X.509 certificate information."""

    subject: str
    issuer: str
    serial_number: str
    not_before: str
    not_after: str
    is_self_signed: bool


@dataclasses.dataclass(frozen=True, slots=True)
class SignatureInfo:
    """Information about a digital signature found in the PDF."""

    name: str
    signer: str | None = None
    reason: str | None = None
    location: str | None = None
    date: str | None = None
    contact_info: str | None = None
    byte_range: tuple[int, int, int, int] | None = None
    certificate: CertificateInfo | None = None
    covers_whole_document: bool = False


@dataclasses.dataclass(frozen=True, slots=True)
class SignatureValidity:
    """Result of signature verification."""

    name: str
    integrity_ok: bool
    certificate_valid: bool
    message: str
    signer: str | None = None


# --- Layout types ---


@dataclasses.dataclass(frozen=True, slots=True)
class LayoutRegion:
    """A rectangular region of the page with classified content."""

    kind: str  # "header", "footer", "body_column", "full_width"
    column_index: int | None
    bbox: tuple[float, float, float, float]
    lines: tuple[TextLine, ...]


@dataclasses.dataclass(frozen=True, slots=True)
class Permissions:
    """Permission flags for an encrypted PDF."""

    print: bool = True
    modify: bool = True
    copy: bool = True
    annotate: bool = True
    fill_forms: bool = True
    accessibility: bool = True
    assemble: bool = True
    print_high_quality: bool = True

    @classmethod
    def none(cls) -> Permissions:
        """Create a Permissions instance with everything denied."""
        return cls(
            print=False,
            modify=False,
            copy=False,
            annotate=False,
            fill_forms=False,
            accessibility=False,
            assemble=False,
            print_high_quality=False,
        )


@dataclasses.dataclass(frozen=True, slots=True)
class EncryptResult:
    """Statistics from PDF encryption."""

    algorithm: str
    key_length: int


@dataclasses.dataclass(frozen=True, slots=True)
class PageLayout:
    """Layout analysis result for a single page."""

    page_width: float
    page_height: float
    column_count: int
    gutters: tuple[float, ...]
    regions: tuple[LayoutRegion, ...]

    @property
    def is_multi_column(self) -> bool:
        """True if page has more than one column."""
        return self.column_count > 1

    def text(self) -> str:
        """Get all text in reading order."""
        lines = []
        for region in self.regions:
            for line in region.lines:
                lines.append(line.text)
        return "\n".join(lines)


# --- Visual diff types ---


@dataclasses.dataclass(frozen=True, slots=True)
class VisualDiffPage:
    """Visual diff result for a single page."""

    page: int
    image_a: bytes
    image_a_width: int
    image_a_height: int
    image_b: bytes
    image_b_width: int
    image_b_height: int
    diff_image: bytes
    diff_image_width: int
    diff_image_height: int
    similarity: float
    changed_pixel_count: int


@dataclasses.dataclass(frozen=True, slots=True)
class VisualDiffResult:
    """Complete visual diff result between two documents."""

    pages: tuple[VisualDiffPage, ...]
    overall_similarity: float
    text_diff_summary: DiffSummary


# --- Validation types ---


@dataclasses.dataclass(frozen=True, slots=True)
class ValidationIssue:
    """A single PDF/A validation issue."""

    severity: str  # "error", "warning", "info"
    rule: str
    message: str
    page: int | None = None


@dataclasses.dataclass(frozen=True, slots=True)
class ValidationReport:
    """PDF/A validation report."""

    level: str
    is_compliant: bool
    issues: tuple[ValidationIssue, ...]
    fonts_checked: int
    pages_checked: int
