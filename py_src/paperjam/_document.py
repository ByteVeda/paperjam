"""Document class wrapping the Rust PDF engine."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, overload

from paperjam import _paperjam
from paperjam._page import Page
from paperjam._types import Bookmark, Metadata

if TYPE_CHECKING:
    from collections.abc import Iterator

    from paperjam._enums import AnnotationType, Rotation, TableStrategy, WatermarkLayer, WatermarkPosition
    from paperjam._types import (
        ChoiceOption,
        ContentBlock,
        CreateFieldResult,
        DiffResult,
        EncryptResult,
        FillFormResult,
        FormField,
        ModifyFieldResult,
        OptimizeResult,
        Permissions,
        RedactRegion,
        RedactResult,
        RenderedImage,
        SanitizeResult,
        SearchResult,
        SignatureInfo,
        SignatureValidity,
        Table,
    )


class Document:
    """A PDF document with lazy page loading.

    Use as a context manager for automatic resource cleanup:

        with paperjam.open("file.pdf") as doc:
            for page in doc.pages:
                print(page.extract_text())

    Or without a context manager (resources freed on garbage collection):

        doc = paperjam.open("file.pdf")
        text = doc.pages[0].extract_text()
    """

    __slots__ = ("_closed", "_inner", "_raw_bytes")

    if TYPE_CHECKING:
        # -- Extraction (attached by _extraction.py) --

        def extract_structure(
            self,
            *,
            heading_size_ratio: float = ...,
            detect_lists: bool = ...,
            include_tables: bool = ...,
            layout_aware: bool = ...,
        ) -> list[ContentBlock]:
            """Extract structured content (headings, paragraphs, lists, tables) from all pages."""
            ...

        def to_markdown(
            self,
            *,
            heading_offset: int = ...,
            page_separator: str = ...,
            include_page_numbers: bool = ...,
            page_number_format: str = ...,
            html_tables: bool = ...,
            table_header_first_row: bool = ...,
            normalize_list_markers: bool = ...,
            heading_size_ratio: float = ...,
            detect_lists: bool = ...,
            include_tables: bool = ...,
            layout_aware: bool = ...,
        ) -> str:
            """Convert the entire document to Markdown."""
            ...

        def search(
            self,
            query: str,
            *,
            case_sensitive: bool = ...,
            max_results: int = ...,
            use_regex: bool = ...,
        ) -> list[SearchResult]:
            """Search for text across all pages.

            Args:
                query: The text or regex pattern to search for.
                case_sensitive: Whether the search is case-sensitive (default True).
                max_results: Maximum number of results to return (0 = unlimited).
                use_regex: If True, treat query as a regular expression.
            """
            ...

        def extract_tables(
            self,
            *,
            strategy: TableStrategy | str = ...,
            min_rows: int = ...,
            min_cols: int = ...,
            snap_tolerance: float = ...,
            row_tolerance: float = ...,
            min_col_gap: float = ...,
        ) -> list[Table]:
            """Extract tables from all pages."""
            ...

        # -- Manipulation (attached by _manipulation.py) --

        def split(self, ranges: list[tuple[int, int]]) -> list[Document]:
            """Split into multiple documents by page ranges (1-indexed, inclusive)."""
            ...

        def split_pages(self) -> list[Document]:
            """Split into individual single-page documents."""
            ...

        def rotate(self, page_rotations: list[tuple[int, Rotation | int]]) -> Document:
            """Rotate pages by specified angles, returning a new Document.

            Args:
                page_rotations: List of (page_number, rotation) tuples.
                                page_number is 1-indexed. rotation is degrees (0, 90, 180, 270)
                                or a Rotation enum value.
            """
            ...

        def reorder(self, page_order: list[int]) -> Document:
            """Reorder pages, returning a new Document.

            Args:
                page_order: List of 1-indexed page numbers in desired order.
                            Can subset (drop pages) or repeat (duplicate pages).
            """
            ...

        def optimize(
            self,
            *,
            compress_streams: bool = ...,
            remove_unused: bool = ...,
            remove_duplicates: bool = ...,
            strip_metadata: bool = ...,
        ) -> tuple[Document, OptimizeResult]:
            """Optimize the PDF to reduce file size.

            Returns a tuple of (optimized_document, result_stats).
            """
            ...

        def add_annotation(
            self,
            page: int,
            annotation_type: AnnotationType | str,
            rect: tuple[float, float, float, float],
            *,
            contents: str | None = ...,
            author: str | None = ...,
            color: tuple[float, float, float] | None = ...,
            opacity: float | None = ...,
            quad_points: tuple[float, ...] | None = ...,
            url: str | None = ...,
        ) -> Document:
            """Add an annotation to a page, returning a new Document."""
            ...

        def remove_annotations(
            self,
            page: int,
            *,
            annotation_types: list[AnnotationType | str] | None = ...,
            indices: list[int] | None = ...,
        ) -> tuple[Document, int]:
            """Remove annotations from a page, returning a new Document and count removed.

            Args:
                page: 1-indexed page number.
                annotation_types: If provided, only remove annotations matching these types.
                indices: If provided, only remove annotations at these 0-based positions.
            """
            ...

        def add_watermark(
            self,
            text: str,
            *,
            font_size: float = ...,
            rotation: float = ...,
            opacity: float = ...,
            color: tuple[float, float, float] = ...,
            font: str = ...,
            position: WatermarkPosition | str = ...,
            layer: WatermarkLayer | str = ...,
            pages: list[int] | None = ...,
            x: float | None = ...,
            y: float | None = ...,
        ) -> Document:
            """Add a text watermark to pages, returning a new Document.

            Args:
                x: Custom X position in points. When both x and y are provided,
                   the position parameter is ignored.
                y: Custom Y position in points. When both x and y are provided,
                   the position parameter is ignored.
            """
            ...

        def delete_pages(self, page_numbers: list[int]) -> Document:
            """Delete specific pages from the document, returning a new Document.

            Args:
                page_numbers: List of 1-indexed page numbers to remove.
                              At least one page must remain.
            """
            ...

        def insert_blank_pages(
            self,
            positions: list[tuple[int, float, float]],
        ) -> Document:
            """Insert blank pages at specified positions, returning a new Document.

            Args:
                positions: List of (after_page, width, height) tuples.
                           after_page=0 inserts at the beginning.
                           width and height are in PDF points (72 points = 1 inch).
            """
            ...

        def set_metadata(
            self,
            *,
            title: str | None = ...,
            author: str | None = ...,
            subject: str | None = ...,
            keywords: str | None = ...,
            creator: str | None = ...,
            producer: str | None = ...,
        ) -> Document:
            """Update document metadata, returning a new Document.

            Pass a string value to set a field, None to remove it,
            or omit it to leave it unchanged.
            """
            ...

        def set_bookmarks(self, bookmarks: list[Bookmark]) -> Document:
            """Replace the document's bookmarks/outlines, returning a new Document.

            Args:
                bookmarks: List of Bookmark objects defining the new outline tree.
                           Pass an empty list to remove all bookmarks.
            """
            ...

        # -- Comparison (attached by _comparison.py) --

        def diff(self, other: Document) -> DiffResult:
            """Compare this document with another at the text level.

            Returns a DiffResult with per-page changes and summary statistics.
            """
            ...

        # -- Security (attached by _security.py) --

        def sanitize(
            self,
            *,
            remove_javascript: bool = ...,
            remove_embedded_files: bool = ...,
            remove_actions: bool = ...,
            remove_links: bool = ...,
        ) -> tuple[Document, SanitizeResult]:
            """Remove potentially dangerous objects from the PDF.

            Returns a tuple of (sanitized_document, result_stats).
            """
            ...

        def redact(
            self,
            regions: list[RedactRegion],
            *,
            fill_color: tuple[float, float, float] | None = ...,
        ) -> tuple[Document, RedactResult]:
            """Redact text within specified regions, removing it from the content stream.

            Args:
                regions: List of RedactRegion specifying areas to redact.
                fill_color: Optional (r, g, b) color for overlay rectangles (0.0-1.0).

            Returns a tuple of (redacted_document, result_stats).
            """
            ...

        def redact_text(
            self,
            query: str,
            *,
            case_sensitive: bool = ...,
            use_regex: bool = ...,
            fill_color: tuple[float, float, float] | None = ...,
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
            ...

        def encrypt(
            self,
            *,
            user_password: str,
            owner_password: str | None = ...,
            permissions: Permissions | None = ...,
            algorithm: str = ...,
        ) -> tuple[bytes, EncryptResult]:
            """Encrypt the document with user/owner passwords and permission flags.

            Args:
                user_password: Password required to open the document.
                owner_password: Password for full access. Defaults to user_password.
                permissions: Permission flags controlling what viewers can do.
                algorithm: Encryption algorithm — "aes128" (default) or "rc4".

            Returns a tuple of (encrypted_bytes, encrypt_result).
            """
            ...

        # -- Forms (attached by _forms.py) --

        @property
        def has_form(self) -> bool:
            """Whether the document contains an interactive form (AcroForm)."""
            ...

        @property
        def form_fields(self) -> list[FormField]:
            """Extract all form fields from the document's AcroForm."""
            ...

        def fill_form(
            self,
            values: dict[str, str],
            *,
            need_appearances: bool = ...,
            generate_appearances: bool = ...,
        ) -> tuple[Document, FillFormResult]:
            """Fill form fields by name.

            Args:
                values: Mapping of fully-qualified field names to string values.
                    For checkboxes, pass "Yes" or "Off". For radio buttons, pass the export value.
                need_appearances: If True (default), sets /NeedAppearances so viewers
                    regenerate field appearances automatically. Ignored if generate_appearances is True.
                generate_appearances: If True, generates explicit /AP streams so forms
                    render correctly even in viewers that ignore /NeedAppearances.

            Returns:
                A tuple of (new_document, fill_result).
            """
            ...

        def modify_form_field(
            self,
            field_name: str,
            *,
            value: str | None = ...,
            default_value: str | None = ...,
            read_only: bool | None = ...,
            required: bool | None = ...,
            max_length: int | None = ...,
            options: list[ChoiceOption] | None = ...,
        ) -> tuple[Document, ModifyFieldResult]:
            """Modify properties of a form field.

            Args:
                field_name: Fully-qualified field name.
                value: New value for the field.
                default_value: New default value.
                read_only: Set the read-only flag.
                required: Set the required flag.
                max_length: Set max length (text fields).
                options: Replace choice options (combo/list boxes).

            Returns:
                A tuple of (new_document, modify_result).
            """
            ...

        def add_form_field(
            self,
            name: str,
            field_type: str,
            *,
            page: int = ...,
            rect: tuple[float, float, float, float] = ...,
            value: str | None = ...,
            default_value: str | None = ...,
            read_only: bool = ...,
            required: bool = ...,
            max_length: int | None = ...,
            options: list[ChoiceOption] | None = ...,
            font_size: float = ...,
            generate_appearance: bool = ...,
        ) -> tuple[Document, CreateFieldResult]:
            """Create a new form field on a page.

            Args:
                name: Fully-qualified field name.
                field_type: One of "text", "checkbox", "radio_button", "combo_box",
                    "list_box", "push_button", "signature".
                page: 1-based page number (default 1).
                rect: Field rectangle (x1, y1, x2, y2) in PDF points.
                value: Initial value.
                default_value: Default value.
                read_only: Whether the field is read-only.
                required: Whether the field is required.
                max_length: Maximum length for text fields.
                options: Choice options for combo/list boxes.
                font_size: Font size (0 = auto).
                generate_appearance: Whether to generate an appearance stream (default True).

            Returns:
                A tuple of (new_document, create_result).
            """
            ...

        # -- Rendering (attached by _render.py) --

        def render_page(
            self,
            page_number: int,
            *,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
            background_color: tuple[int, int, int] | None = ...,
            scale_to_width: int | None = ...,
            scale_to_height: int | None = ...,
        ) -> RenderedImage:
            """Render a single page to an image.

            Args:
                page_number: 1-based page number to render.
                dpi: Resolution in dots per inch (default 150).
                format: Image format - "png", "jpeg", or "bmp" (default "png").
                quality: JPEG quality 1-100 (default 85, only used for JPEG).
                background_color: RGB tuple (0-255) for background color.
                scale_to_width: Target pixel width (overrides DPI).
                scale_to_height: Target pixel height (overrides DPI).

            Returns:
                A RenderedImage with the image data and dimensions.
            """
            ...

        def render_pages(
            self,
            *,
            pages: list[int] | None = ...,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
            background_color: tuple[int, int, int] | None = ...,
            scale_to_width: int | None = ...,
            scale_to_height: int | None = ...,
        ) -> list[RenderedImage]:
            """Render multiple pages to images.

            Args:
                pages: List of 1-based page numbers. None renders all pages.
                dpi: Resolution in dots per inch (default 150).
                format: Image format - "png", "jpeg", or "bmp" (default "png").
                quality: JPEG quality 1-100 (default 85, only used for JPEG).
                background_color: RGB tuple (0-255) for background color.
                scale_to_width: Target pixel width (overrides DPI).
                scale_to_height: Target pixel height (overrides DPI).

            Returns:
                List of RenderedImage objects.
            """
            ...

        # -- Signatures (attached by _signature.py) --

        @property
        def signatures(self) -> list[SignatureInfo]:
            """Extract all digital signatures from the document."""
            ...

        def verify_signatures(self) -> list[SignatureValidity]:
            """Verify all digital signatures in the document.

            For each signature, checks:
            - Integrity: the hash of the signed bytes matches the PKCS#7 signature
            - Certificate validity: basic date check

            Returns a list of SignatureValidity results.
            """
            ...

        def sign(
            self,
            *,
            private_key: bytes,
            certificates: list[bytes],
            reason: str | None = ...,
            location: str | None = ...,
            contact_info: str | None = ...,
            field_name: str = ...,
        ) -> bytes:
            """Sign the document with a digital signature.

            Args:
                private_key: DER-encoded private key (PKCS#8 format).
                certificates: List of DER-encoded X.509 certificates.
                    The first certificate should be the signing certificate.
                reason: Reason for signing.
                location: Location of signing.
                contact_info: Contact information.
                field_name: Signature field name (default: "Signature1").

            Returns:
                The finalized signed PDF as bytes.
            """
            ...

        # -- Async wrappers (attached by _async.py) --

        @staticmethod
        async def aopen(
            path_or_bytes: str | os.PathLike[str] | bytes,
            *,
            password: str | None = ...,
        ) -> Document: ...

        async def asave(self, path: str | os.PathLike[str]) -> None: ...
        async def asave_bytes(self) -> bytes: ...

        async def arender_page(
            self,
            page_number: int,
            *,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
            background_color: tuple[int, int, int] | None = ...,
            scale_to_width: int | None = ...,
            scale_to_height: int | None = ...,
        ) -> RenderedImage: ...

        async def arender_pages(
            self,
            *,
            pages: list[int] | None = ...,
            dpi: float = ...,
            format: str = ...,
            quality: int = ...,
            background_color: tuple[int, int, int] | None = ...,
            scale_to_width: int | None = ...,
            scale_to_height: int | None = ...,
        ) -> list[RenderedImage]: ...

        async def aextract_tables(
            self,
            *,
            strategy: TableStrategy | str = ...,
            min_rows: int = ...,
            min_cols: int = ...,
            snap_tolerance: float = ...,
            row_tolerance: float = ...,
            min_col_gap: float = ...,
        ) -> list[Table]: ...

        async def ato_markdown(self, **kwargs) -> str: ...

        async def asearch(
            self,
            query: str,
            *,
            case_sensitive: bool = ...,
            max_results: int = ...,
            use_regex: bool = ...,
        ) -> list[SearchResult]: ...

        async def adiff(self, other: Document) -> DiffResult: ...

        async def aredact_text(
            self,
            query: str,
            *,
            case_sensitive: bool = ...,
            use_regex: bool = ...,
            fill_color: tuple[float, float, float] | None = ...,
        ) -> tuple[Document, RedactResult]: ...

    def __init__(
        self,
        path_or_bytes: str | os.PathLike[str] | bytes,
        *,
        password: str | None = None,
    ) -> None:
        """Open a PDF document from a file path or raw bytes.

        Args:
            path_or_bytes: File path, path-like object, or raw PDF bytes.
            password: Password for encrypted PDFs.
        """
        if isinstance(path_or_bytes, (str, os.PathLike)):
            path = str(path_or_bytes)
            with open(path, "rb") as f:
                self._raw_bytes: bytes | None = f.read()
            if password is not None:
                self._inner = _paperjam.RustDocument.open_with_password(path, password)
            else:
                self._inner = _paperjam.RustDocument.open(path)
        elif isinstance(path_or_bytes, (bytes, bytearray, memoryview)):
            self._raw_bytes = bytes(path_or_bytes)
            if password is not None:
                self._inner = _paperjam.RustDocument.from_bytes_with_password(bytes(path_or_bytes), password)
            else:
                self._inner = _paperjam.RustDocument.from_bytes(bytes(path_or_bytes))
        else:
            raise TypeError(f"Expected str, os.PathLike, or bytes, got {type(path_or_bytes).__name__}")
        self._closed = False

    def __enter__(self) -> Document:
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self.close()

    def __repr__(self) -> str:
        state = "closed" if self._closed else f"{self.page_count} pages"
        return f"<paperjam.Document [{state}]>"

    def __len__(self) -> int:
        return self.page_count

    def close(self) -> None:
        """Release the underlying PDF resources."""
        if not self._closed:
            self._inner = None  # type: ignore[assignment]
            self._closed = True

    def _ensure_open(self) -> _paperjam.RustDocument:
        if self._closed:
            raise ValueError("I/O operation on closed document")
        return self._inner

    @property
    def page_count(self) -> int:
        """Total number of pages in the document."""
        return self._ensure_open().page_count()

    @property
    def pages(self) -> _PageAccessor:
        """Access pages by index or iterate over all pages lazily."""
        return _PageAccessor(self)

    @property
    def metadata(self) -> Metadata:
        """Document metadata (title, author, etc.)."""
        raw = self._ensure_open().metadata()
        return Metadata(**raw)

    def save(self, path: str | os.PathLike[str]) -> None:
        """Save the document to a file."""
        self._ensure_open().save(str(path))

    def save_bytes(self) -> bytes:
        """Serialize the document to bytes."""
        return self._ensure_open().save_bytes()

    @property
    def bookmarks(self) -> list[Bookmark]:
        """Document bookmarks/table of contents as a nested tree."""
        raw = self._ensure_open().bookmarks()
        return _build_bookmark_tree(raw)


def _build_bookmark_tree(flat_items: list[dict]) -> list[Bookmark]:
    """Build a nested bookmark tree from a flat level-annotated list."""
    if not flat_items:
        return []

    result: list[Bookmark] = []
    i = 0
    while i < len(flat_items):
        item = flat_items[i]
        level = item["level"]

        # Collect all children (items with higher level immediately following)
        children_items: list[dict] = []
        j = i + 1
        while j < len(flat_items) and flat_items[j]["level"] > level:
            children_items.append(flat_items[j])
            j += 1

        children = _build_bookmark_tree(children_items)
        result.append(
            Bookmark(
                title=item["title"],
                page=item["page"],
                level=level,
                children=tuple(children),
            )
        )
        i = j

    return result


class _PageAccessor:
    """Provides both indexing and iteration over pages."""

    __slots__ = ("_doc",)

    def __init__(self, doc: Document) -> None:
        self._doc = doc

    def __len__(self) -> int:
        return self._doc.page_count

    @overload
    def __getitem__(self, index: int) -> Page: ...

    @overload
    def __getitem__(self, index: slice) -> list[Page]: ...

    def __getitem__(self, index: int | slice) -> Page | list[Page]:
        inner = self._doc._ensure_open()
        if isinstance(index, int):
            if index < 0:
                index += len(self)
            if index < 0 or index >= len(self):
                raise IndexError(f"page index {index} out of range")
            return Page._from_rust(inner.page(index + 1), inner)
        elif isinstance(index, slice):
            indices = range(*index.indices(len(self)))
            return [Page._from_rust(inner.page(i + 1), inner) for i in indices]
        else:
            raise TypeError(f"indices must be integers or slices, not {type(index).__name__}")

    def __iter__(self) -> Iterator[Page]:
        inner = self._doc._ensure_open()
        for i in range(1, len(self) + 1):
            yield Page._from_rust(inner.page(i), inner)
