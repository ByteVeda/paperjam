"""Type stubs for the native Rust extension module."""

from collections.abc import Coroutine
from typing import Any

# --- Exception classes ---

class PaperJamError(Exception): ...
class ParseError(PaperJamError): ...
class PasswordRequired(PaperJamError): ...  # noqa: N818
class InvalidPassword(PaperJamError): ...  # noqa: N818
class PageOutOfRange(PaperJamError): ...  # noqa: N818
class UnsupportedFeature(PaperJamError): ...  # noqa: N818
class TableExtractionError(PaperJamError): ...
class OptimizationError(PaperJamError): ...
class AnnotationError(PaperJamError): ...
class WatermarkError(PaperJamError): ...
class SanitizeError(PaperJamError): ...
class RedactError(PaperJamError): ...
class FormError(PaperJamError): ...
class RenderError(PaperJamError): ...
class SignatureError(PaperJamError): ...
class EncryptionError(PaperJamError): ...

# --- Classes ---

class RustDocument:
    @staticmethod
    def open(path: str) -> RustDocument: ...
    @staticmethod
    def open_with_password(path: str, password: str) -> RustDocument: ...
    @staticmethod
    def from_bytes(data: bytes) -> RustDocument: ...
    @staticmethod
    def from_bytes_with_password(data: bytes, password: str) -> RustDocument: ...
    def page_count(self) -> int: ...
    def page(self, number: int) -> RustPage: ...
    def metadata(self) -> dict[str, Any]: ...
    def save_bytes(self) -> bytes: ...
    def save(self, path: str) -> None: ...
    def extract_images(self, page_number: int) -> list[dict[str, Any]]: ...
    def bookmarks(self) -> list[dict[str, Any]]: ...
    def annotations(self, page_number: int) -> list[dict[str, Any]]: ...
    def has_form(self) -> bool: ...
    def form_fields(self) -> list[dict[str, Any]]: ...
    def extract_structure(
        self,
        *,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
        layout_aware: bool = False,
    ) -> list[dict[str, Any]]: ...
    def to_markdown(
        self,
        *,
        heading_offset: int = 0,
        page_separator: str = "---",
        include_page_numbers: bool = False,
        page_number_format: str = "<!-- page {n} -->",
        html_tables: bool = False,
        table_header_first_row: bool = True,
        normalize_list_markers: bool = True,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
        layout_aware: bool = False,
    ) -> str: ...

class RustPage:
    def number(self) -> int: ...
    def width(self) -> float: ...
    def height(self) -> float: ...
    def rotation(self) -> int: ...
    def extract_text(self) -> str: ...
    def extract_text_spans(self) -> list[dict[str, Any]]: ...
    def extract_text_lines(self) -> list[dict[str, Any]]: ...
    def extract_tables(
        self,
        *,
        strategy: str = "auto",
        min_rows: int = 2,
        min_cols: int = 2,
        snap_tolerance: float = 3.0,
        row_tolerance: float = 0.5,
        min_col_gap: float = 10.0,
    ) -> list[dict[str, Any]]: ...
    def extract_structure(
        self,
        *,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
        layout_aware: bool = False,
    ) -> list[dict[str, Any]]: ...
    def analyze_layout(
        self,
        *,
        min_gutter_width: float = 20.0,
        max_columns: int = 4,
        detect_headers_footers: bool = True,
        header_zone_fraction: float = 0.08,
        footer_zone_fraction: float = 0.08,
        min_column_line_fraction: float = 0.1,
    ) -> dict[str, Any]: ...
    def extract_text_layout(
        self,
        *,
        min_gutter_width: float = 20.0,
        max_columns: int = 4,
        detect_headers_footers: bool = True,
        header_zone_fraction: float = 0.08,
        footer_zone_fraction: float = 0.08,
        min_column_line_fraction: float = 0.1,
    ) -> str: ...
    def to_markdown(
        self,
        *,
        heading_offset: int = 0,
        page_separator: str = "---",
        include_page_numbers: bool = False,
        page_number_format: str = "<!-- page {n} -->",
        html_tables: bool = False,
        table_header_first_row: bool = True,
        normalize_list_markers: bool = True,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
        layout_aware: bool = False,
    ) -> str: ...

# --- Module-level functions ---

def merge(
    documents: list[RustDocument],
    deduplicate_resources: bool = False,
) -> RustDocument: ...
def split(
    document: RustDocument,
    ranges: list[tuple[int, int]],
) -> list[RustDocument]: ...
def rotate_pages(
    document: RustDocument,
    page_rotations: list[tuple[int, int]],
) -> RustDocument: ...
def reorder_pages(
    document: RustDocument,
    page_order: list[int],
) -> RustDocument: ...
def optimize(
    document: RustDocument,
    compress_streams: bool,
    remove_unused: bool,
    remove_duplicates: bool,
    strip_metadata: bool,
) -> tuple[RustDocument, dict[str, Any]]: ...
def add_annotation(
    document: RustDocument,
    page_number: int,
    annotation_type: str,
    rect: list[float],
    contents: str | None = None,
    author: str | None = None,
    color: list[float] | None = None,
    opacity: float | None = None,
    quad_points: list[float] | None = None,
    url: str | None = None,
) -> RustDocument: ...
def remove_annotations(
    document: RustDocument,
    page_number: int,
    annotation_types: list[str] | None = None,
    indices: list[int] | None = None,
) -> tuple[RustDocument, int]: ...
def add_watermark(
    document: RustDocument,
    text: str,
    font_size: float,
    rotation: float,
    opacity: float,
    color: list[float],
    font: str,
    position: str,
    layer: str,
    pages: list[int] | None = None,
    custom_x: float | None = None,
    custom_y: float | None = None,
) -> RustDocument: ...
def diff_documents(
    document_a: RustDocument,
    document_b: RustDocument,
) -> dict[str, Any]: ...
def sanitize(
    document: RustDocument,
    remove_javascript: bool = True,
    remove_embedded_files: bool = True,
    remove_actions: bool = True,
    remove_links: bool = True,
) -> tuple[RustDocument, dict[str, Any]]: ...
def redact(
    document: RustDocument,
    regions: list[dict[str, Any]],
    fill_color: list[float] | None = None,
) -> tuple[RustDocument, dict[str, Any]]: ...
def redact_text(
    document: RustDocument,
    query: str,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: list[float] | None = None,
) -> tuple[RustDocument, dict[str, Any]]: ...
def fill_form(
    document: RustDocument,
    values: dict[str, str],
    need_appearances: bool = True,
) -> tuple[RustDocument, dict[str, Any]]: ...
def render_page(
    document: RustDocument,
    page_number: int,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> dict[str, Any]: ...
def render_pages(
    document: RustDocument,
    pages: list[int] | None = None,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> list[dict[str, Any]]: ...
def render_file(
    data: bytes,
    page_number: int = 1,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> dict[str, Any]: ...
def render_pages_bytes(
    data: bytes,
    pages: list[int] | None = None,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> list[dict[str, Any]]: ...
def delete_pages(
    document: RustDocument,
    page_numbers: list[int],
) -> RustDocument: ...
def insert_blank_pages(
    document: RustDocument,
    positions: list[tuple[int, float, float]],
) -> RustDocument: ...
def set_metadata(
    document: RustDocument,
    updates: dict[str, str | None],
) -> RustDocument: ...
def set_bookmarks(
    document: RustDocument,
    bookmarks: list[dict[str, Any]],
) -> RustDocument: ...
def generate_toc(
    document: RustDocument,
    *,
    max_depth: int = 6,
    heading_size_ratio: float = 1.2,
    layout_aware: bool = False,
    replace_existing: bool = True,
) -> tuple[RustDocument, list[dict[str, Any]]]: ...
def stamp_pages(
    document: RustDocument,
    stamp_document: RustDocument,
    *,
    source_page: int = 1,
    target_pages: list[int] | None = None,
    x: float = 0.0,
    y: float = 0.0,
    scale: float = 1.0,
    opacity: float = 1.0,
    layer: str = "over",
) -> RustDocument: ...
def visual_diff(
    document_a: RustDocument,
    document_b: RustDocument,
    bytes_a: bytes,
    bytes_b: bytes,
    *,
    dpi: float = 150.0,
    highlight_color: list[int] | None = None,
    mode: str = "both",
    threshold: int = 10,
    library_path: str | None = None,
) -> dict[str, Any]: ...
def validate_pdf_a(
    document: RustDocument,
    *,
    level: str = "1b",
) -> dict[str, Any]: ...
def encrypt_document(
    document: RustDocument,
    user_password: str,
    owner_password: str | None = None,
    permissions: dict[str, bool] | None = None,
    algorithm: str = "aes128",
) -> tuple[bytes, dict[str, Any]]: ...
def sign_document(
    document: RustDocument,
    private_key: bytes,
    certificates: list[bytes],
    reason: str | None = None,
    location: str | None = None,
    contact_info: str | None = None,
    field_name: str = "Signature1",
    tsa_url: str | None = None,
    timestamp_token: bytes | None = None,
    ocsp_responses: list[bytes] | None = None,
    crls: list[bytes] | None = None,
) -> bytes: ...
def extract_signatures(
    document: RustDocument,
    raw_bytes: bytes,
) -> list[dict[str, Any]]: ...
def verify_signatures(
    document: RustDocument,
    raw_bytes: bytes,
) -> list[dict[str, Any]]: ...
def convert_to_pdf_a(
    document: RustDocument,
    *,
    level: str = "1b",
    force: bool = False,
) -> tuple[RustDocument, dict[str, Any]]: ...
def validate_pdf_ua(
    document: RustDocument,
    *,
    level: str = "1",
) -> dict[str, Any]: ...

# --- Async functions ---

def aopen(path: str) -> Coroutine[Any, Any, RustDocument]: ...
def aopen_with_password(path: str, password: str) -> Coroutine[Any, Any, RustDocument]: ...
def aopen_bytes(data: bytes) -> Coroutine[Any, Any, RustDocument]: ...
def aopen_bytes_with_password(data: bytes, password: str) -> Coroutine[Any, Any, RustDocument]: ...
def asave(document: RustDocument, path: str) -> Coroutine[Any, Any, None]: ...
def asave_bytes(document: RustDocument) -> Coroutine[Any, Any, bytes]: ...
def ato_markdown(
    document: RustDocument,
    *,
    heading_offset: int = 0,
    page_separator: str = "---",
    include_page_numbers: bool = False,
    page_number_format: str = "<!-- page {n} -->",
    html_tables: bool = False,
    table_header_first_row: bool = True,
    normalize_list_markers: bool = True,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> Coroutine[Any, Any, str]: ...
def arender_page(
    document: RustDocument,
    page_number: int,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> Coroutine[Any, Any, dict[str, Any]]: ...
def arender_pages(
    document: RustDocument,
    pages: list[int] | None = None,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> Coroutine[Any, Any, list[dict[str, Any]]]: ...
def arender_file(
    data: bytes,
    page_number: int = 1,
    dpi: float = 150.0,
    format: str = "png",
    quality: int = 85,
    background_color: list[int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
    library_path: str | None = None,
) -> Coroutine[Any, Any, dict[str, Any]]: ...
def adiff_documents(
    document_a: RustDocument,
    document_b: RustDocument,
) -> Coroutine[Any, Any, dict[str, Any]]: ...
def aredact_text(
    document: RustDocument,
    query: str,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: list[float] | None = None,
) -> Coroutine[Any, Any, tuple[RustDocument, dict[str, Any]]]: ...
def amerge(
    documents: list[RustDocument],
    deduplicate_resources: bool = False,
) -> Coroutine[Any, Any, RustDocument]: ...
def apage_extract_text(page: RustPage) -> Coroutine[Any, Any, str]: ...
def apage_extract_tables(
    page: RustPage,
    *,
    strategy: str = "auto",
    min_rows: int = 2,
    min_cols: int = 2,
    snap_tolerance: float = 3.0,
    row_tolerance: float = 0.5,
    min_col_gap: float = 10.0,
) -> Coroutine[Any, Any, list[dict[str, Any]]]: ...
def apage_to_markdown(
    page: RustPage,
    *,
    heading_offset: int = 0,
    page_separator: str = "---",
    include_page_numbers: bool = False,
    page_number_format: str = "<!-- page {n} -->",
    html_tables: bool = False,
    table_header_first_row: bool = True,
    normalize_list_markers: bool = True,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> Coroutine[Any, Any, str]: ...
