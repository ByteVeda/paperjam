# Document

```python
class paperjam.Document
```

A PDF document with lazy page loading. All manipulation methods return a **new** `Document` — the original is never modified.

Use as a context manager for automatic resource cleanup, or let the garbage collector free resources:

```python
# Context manager (recommended)
with paperjam.open("file.pdf") as doc:
    print(doc.page_count)

# Without context manager
doc = paperjam.open("file.pdf")
print(doc.page_count)
doc.close()
```

---

## Properties

### `page_count`

```python
doc.page_count: int
```

Total number of pages in the document.

### `pages`

```python
doc.pages  # _PageAccessor
```

Access pages by 0-based index (like a list) or iterate. Supports slices.

```python
first  = doc.pages[0]
last   = doc.pages[-1]
subset = doc.pages[1:5]
for page in doc.pages:
    ...
```

### `metadata`

```python
doc.metadata: Metadata
```

Document metadata as a frozen `Metadata` dataclass. See [types reference](types.md).

### `bookmarks`

```python
doc.bookmarks: list[Bookmark]
```

Nested bookmark tree. Top-level entries are returned; each `Bookmark` has a `children` tuple.

### `has_form`

```python
doc.has_form: bool
```

`True` if the document contains an AcroForm dictionary.

### `form_fields`

```python
doc.form_fields: list[FormField]
```

All form fields extracted from the AcroForm.

### `signatures`

```python
doc.signatures: list[SignatureInfo]
```

All digital signatures found in the document. Requires the `signatures` feature.

---

## Saving

### `save`

```python
doc.save(path: str | os.PathLike) -> None
```

Save the document to a file.

### `save_bytes`

```python
doc.save_bytes() -> bytes
```

Serialize the document to bytes in memory.

### `close`

```python
doc.close() -> None
```

Explicitly release resources. Called automatically when used as a context manager.

---

## Text and content extraction

### `extract_structure`

```python
doc.extract_structure(
    *,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> list[ContentBlock]
```

Extract structured content (headings, paragraphs, list items, tables) from all pages.

### `extract_tables`

```python
doc.extract_tables(
    *,
    strategy: TableStrategy | str = TableStrategy.AUTO,
    min_rows: int = 2,
    min_cols: int = 2,
    snap_tolerance: float = 3.0,
    row_tolerance: float = 0.5,
    min_col_gap: float = 10.0,
) -> list[Table]
```

Extract tables from all pages.

### `extract_links`

```python
doc.extract_links() -> list[Link]
```

Extract all hyperlinks from all pages.

### `search`

```python
doc.search(
    query: str,
    *,
    case_sensitive: bool = True,
    max_results: int = 0,
    use_regex: bool = False,
) -> list[SearchResult]
```

Search for text across all pages. `max_results=0` means unlimited.

### `to_markdown`

```python
doc.to_markdown(
    *,
    heading_offset: int = 0,
    include_page_numbers: bool = False,
    html_tables: bool = False,
    layout_aware: bool = False,
    # ... additional formatting options
) -> str
```

Convert the entire document to Markdown.

---

## Metadata and bookmarks

### `set_metadata`

```python
doc.set_metadata(
    *,
    title: str | None = ...,
    author: str | None = ...,
    subject: str | None = ...,
    keywords: str | None = ...,
    creator: str | None = ...,
    producer: str | None = ...,
) -> Document
```

Update document metadata. Pass a string to set a field, `None` to remove it, or omit it to leave it unchanged. Returns a new `Document`.

### `set_bookmarks`

```python
doc.set_bookmarks(bookmarks: list[Bookmark]) -> Document
```

Replace the entire bookmark tree. Pass an empty list to remove all bookmarks. Returns a new `Document`.

### `generate_toc`

```python
doc.generate_toc(
    *,
    max_depth: int = 3,
    heading_size_ratio: float = 1.2,
    layout_aware: bool = False,
    replace_existing: bool = True,
) -> tuple[Document, list[Bookmark]]
```

Auto-generate a table of contents from the document's heading structure. Returns `(new_document, bookmarks)`.

---

## Page manipulation

### `split`

```python
doc.split(ranges: list[tuple[int, int]]) -> list[Document]
```

Split into multiple documents by page ranges. Ranges are 1-indexed and inclusive on both ends.

### `split_pages`

```python
doc.split_pages() -> list[Document]
```

Split into individual single-page documents.

### `reorder`

```python
doc.reorder(page_order: list[int]) -> Document
```

Reorder pages. `page_order` is a list of 1-indexed page numbers in the desired output order. You may repeat page numbers (to duplicate) or omit them (to delete). Returns a new `Document`.

### `rotate`

```python
doc.rotate(page_rotations: list[tuple[int, Rotation | int]]) -> Document
```

Rotate pages. Each tuple is `(page_number, angle)` where page numbers are 1-indexed and angle is degrees or a `Rotation` enum value. Returns a new `Document`.

### `delete_pages`

```python
doc.delete_pages(page_numbers: list[int]) -> Document
```

Delete pages by 1-indexed page number. At least one page must remain. Returns a new `Document`.

### `insert_blank_pages`

```python
doc.insert_blank_pages(
    positions: list[tuple[int, float, float]],
) -> Document
```

Insert blank pages. Each tuple is `(after_page, width_pt, height_pt)`. `after_page=0` inserts before page 1. Returns a new `Document`.

### `stamp`

```python
doc.stamp(
    stamp_doc: Document,
    *,
    source_page: int = 1,
    target_pages: list[int] | None = None,
    x: float = 0.0,
    y: float = 0.0,
    scale: float = 1.0,
    opacity: float = 1.0,
    layer: str = "over",
) -> Document
```

Overlay a page from another PDF onto pages of this document. `layer` is `"over"` or `"under"`. Returns a new `Document`.

---

## Annotations

### `add_annotation`

```python
doc.add_annotation(
    page: int,
    annotation_type: AnnotationType | str,
    rect: tuple[float, float, float, float],
    *,
    contents: str | None = None,
    author: str | None = None,
    color: tuple[float, float, float] | None = None,
    opacity: float | None = None,
    url: str | None = None,
) -> Document
```

Add an annotation to a page. Returns a new `Document`.

### `add_watermark`

```python
doc.add_watermark(
    text: str,
    *,
    font_size: float = 60.0,
    rotation: float = 45.0,
    opacity: float = 0.3,
    color: tuple[float, float, float] = (0.5, 0.5, 0.5),
    position: WatermarkPosition | str = WatermarkPosition.CENTER,
    layer: WatermarkLayer | str = WatermarkLayer.OVER,
    pages: list[int] | None = None,
    x: float | None = None,
    y: float | None = None,
) -> Document
```

Add a text watermark to pages. Returns a new `Document`.

### `remove_annotations`

```python
doc.remove_annotations(
    page: int,
    *,
    annotation_types: list[AnnotationType | str] | None = None,
    indices: list[int] | None = None,
) -> tuple[Document, int]
```

Remove annotations from a page. Returns `(new_document, count_removed)`.

---

## Security

### `sanitize`

```python
doc.sanitize(
    *,
    remove_javascript: bool = True,
    remove_embedded_files: bool = True,
    remove_actions: bool = True,
    remove_links: bool = True,
) -> tuple[Document, SanitizeResult]
```

Remove potentially dangerous content. Returns `(sanitized_document, result_stats)`.

### `redact`

```python
doc.redact(
    regions: list[RedactRegion],
    *,
    fill_color: tuple[float, float, float] | None = None,
) -> tuple[Document, RedactResult]
```

Redact specific rectangular regions from the content stream. Returns `(redacted_document, result_stats)`.

### `redact_text`

```python
doc.redact_text(
    query: str,
    *,
    case_sensitive: bool = True,
    use_regex: bool = False,
    fill_color: tuple[float, float, float] | None = None,
) -> tuple[Document, RedactResult]
```

Find and redact all occurrences of a text query. Returns `(redacted_document, result_stats)`.

### `encrypt`

```python
doc.encrypt(
    *,
    user_password: str,
    owner_password: str | None = None,
    permissions: Permissions | None = None,
    algorithm: str = "aes128",
) -> tuple[bytes, EncryptResult]
```

Encrypt the document. Returns `(encrypted_bytes, result)` — note the first element is `bytes`, not a `Document`.

### `validate_pdf_a`

```python
doc.validate_pdf_a(level: str = "1b") -> ValidationReport
```

Validate PDF/A compliance. `level` is `"1b"`, `"1a"`, or `"2b"`.

---

## Forms

### `fill_form`

```python
doc.fill_form(
    values: dict[str, str],
    *,
    generate_appearances: bool = False,
) -> tuple[Document, FillFormResult]
```

Fill form fields by name. Returns `(new_document, result)`.

### `add_form_field`

```python
doc.add_form_field(
    name: str,
    field_type: str,
    *,
    page: int = 1,
    rect: tuple[float, float, float, float],
    # ... many optional kwargs
) -> tuple[Document, CreateFieldResult]
```

Create a new form field. Returns `(new_document, result)`.

### `modify_form_field`

```python
doc.modify_form_field(
    field_name: str,
    *,
    value: str | None = None,
    read_only: bool | None = None,
    required: bool | None = None,
    max_length: int | None = None,
    options: list[ChoiceOption] | None = None,
) -> tuple[Document, ModifyFieldResult]
```

Modify an existing form field. Returns `(new_document, result)`.

---

## Rendering

### `render_page`

```python
doc.render_page(
    page_number: int,
    *,
    dpi: float = 150,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage
```

Render a single page to an image. Requires the `render` feature.

### `render_pages`

```python
doc.render_pages(
    *,
    pages: list[int] | None = None,
    dpi: float = 150,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
) -> list[RenderedImage]
```

Render multiple (or all) pages. `pages=None` renders all pages.

---

## Comparison

### `diff`

```python
doc.diff(other: Document) -> DiffResult
```

Compare this document to another at the text level.

### `visual_diff`

```python
doc.visual_diff(
    other: Document,
    *,
    dpi: float = 150,
    highlight_color: tuple[int, int, int, int] | None = None,
    mode: str = "both",
    threshold: int = 10,
) -> VisualDiffResult
```

Compare this document to another visually (pixel-level). Requires the `render` feature.

---

## Signatures

### `verify_signatures`

```python
doc.verify_signatures() -> list[SignatureValidity]
```

Verify all digital signatures. Returns a list of `SignatureValidity` results.

### `sign`

```python
doc.sign(
    *,
    private_key: bytes,
    certificates: list[bytes],
    reason: str | None = None,
    location: str | None = None,
    contact_info: str | None = None,
    field_name: str = "Signature1",
) -> bytes
```

Sign the document. Returns the signed PDF as bytes.

---

## Async methods

See the [Async guide](../guides/async.md) for usage. Async versions use the `a` prefix: `asave`, `asave_bytes`, `arender_page`, `arender_pages`, `aextract_tables`, `ato_markdown`, `asearch`, `adiff`, `aredact_text`.
