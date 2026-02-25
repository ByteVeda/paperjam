"""Type stubs for the native Rust extension module."""

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
    def extract_structure(
        self,
        *,
        heading_size_ratio: float = 1.2,
        detect_lists: bool = True,
        include_tables: bool = True,
    ) -> list[dict[str, Any]]: ...

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
    ) -> list[dict[str, Any]]: ...

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
