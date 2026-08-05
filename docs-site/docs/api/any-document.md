---
sidebar_position: 3
title: AnyDocument
---

# AnyDocument

```python
class paperjam.AnyDocument
```

A format-agnostic document for non-PDF files: DOCX, XLSX, PPTX, HTML, and EPUB. Provides text extraction, table extraction, structured content parsing, image extraction, Markdown conversion, and format conversion.

`AnyDocument` is returned automatically by `paperjam.open()` when the input is not a PDF. You can also instantiate it directly.

Use as a context manager for automatic resource cleanup:

```python
import paperjam

# Via paperjam.open() (recommended)
with paperjam.open("report.docx") as doc:
    print(doc.extract_text())

# Direct instantiation
from paperjam import AnyDocument

doc = AnyDocument("report.docx")
doc = AnyDocument(raw_bytes, format="pptx")
```

---

## Constructor

```python
AnyDocument(
    path_or_bytes: str | os.PathLike | bytes,
    *,
    format: str | None = None,
)
```

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `path_or_bytes` | `str`, `PathLike`, or `bytes` | File path or raw document bytes |
| `format` | `str \| None` | Format hint: `"docx"`, `"xlsx"`, `"pptx"`, `"html"`, `"epub"`. Required when passing bytes without an extension to infer from. |

**Raises** `ParseError` if the file cannot be parsed. `ValueError` if `format` is required but not provided.

---

## Properties

### `format`

```python
doc.format: str
```

The detected document format: `"docx"`, `"xlsx"`, `"pptx"`, `"html"`, or `"epub"`.

### `page_count`

```python
doc.page_count: int
```

Number of logical pages. For DOCX this is the estimated page count. For XLSX it is the number of sheets. For PPTX it is the number of slides. For HTML and EPUB it is based on section breaks.

### `metadata`

```python
doc.metadata: Metadata
```

Document metadata as a frozen `Metadata` dataclass. Available fields depend on the format. See [types reference](types).

### `bookmarks`

```python
doc.bookmarks: list[Bookmark]
```

Table of contents or bookmark tree extracted from the document. Returns an empty list if the format has no bookmark equivalent.

---

## Text extraction

### `extract_text`

```python
doc.extract_text(
    *,
    pages: list[int] | None = None,
) -> str
```

Extract plain text from the document. Pass `pages` (1-indexed) to limit extraction to specific pages/sheets/slides. `None` extracts from all pages.

### `extract_text_lines`

```python
doc.extract_text_lines(
    *,
    pages: list[int] | None = None,
) -> list[TextLine]
```

Extract text as a list of `TextLine` objects. Bounding box information is available when the format supports it (DOCX, PPTX) and `None` otherwise.

---

## Table extraction

### `extract_tables`

```python
doc.extract_tables(
    *,
    pages: list[int] | None = None,
    min_rows: int = 2,
    min_cols: int = 2,
) -> list[Table]
```

Extract tables. For XLSX, every sheet range with data is returned as a table. For DOCX and PPTX, embedded tables are extracted. For HTML, `<table>` elements are parsed.

```python
for table in doc.extract_tables():
    print(f"Headers: {table.headers}")
    print(f"Rows: {len(table.rows)}")
    print(table.to_csv())
```

---

## Structure extraction

### `extract_structure`

```python
doc.extract_structure(
    *,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
) -> list[ContentBlock]
```

Extract structured content blocks: headings, paragraphs, list items, and tables. Returns a list of `ContentBlock` objects.

---

## Image extraction

### `extract_images`

```python
doc.extract_images(
    *,
    pages: list[int] | None = None,
) -> list[Image]
```

Extract embedded images from the document.

---

## Conversion

### `to_markdown`

```python
doc.to_markdown(
    *,
    heading_offset: int = 0,
    include_page_numbers: bool = False,
    html_tables: bool = False,
) -> str
```

Convert the entire document to Markdown.

```python
md = doc.to_markdown(heading_offset=1, include_page_numbers=True)
```

### `convert_to`

```python
doc.convert_to(
    format: str,
    *,
    output: str | os.PathLike | None = None,
) -> bytes | None
```

Convert the document to another format. If `output` is provided, the result is written to disk and `None` is returned. Otherwise, the converted bytes are returned.

```python
# Get bytes
pdf_bytes = doc.convert_to("pdf")

# Write to file
doc.convert_to("pdf", output="report.pdf")
```

**Supported target formats:** `"pdf"`, `"docx"`, `"html"`, `"epub"`, `"markdown"`. Not all conversions are supported for all source formats. See the [Format Conversion guide](../guides/conversion) for the full matrix.

---

## Saving

### `save`

```python
doc.save(path: str | os.PathLike) -> None
```

Save the document to disk in its original format.

### `save_bytes`

```python
doc.save_bytes() -> bytes
```

Serialize the document to bytes in its original format.

### `close`

```python
doc.close() -> None
```

Release resources. Called automatically when used as a context manager.

---

## Comparison with Document

`AnyDocument` provides the extraction and conversion surface that works across all formats. PDF-specific features are only available on `Document`:

| Feature | `AnyDocument` | `Document` (PDF) |
|---------|:---:|:---:|
| `extract_text` | yes | yes |
| `extract_text_lines` | yes | yes |
| `extract_tables` | yes | yes |
| `extract_structure` | yes | yes |
| `extract_images` | yes | yes |
| `to_markdown` | yes | yes |
| `convert_to` | yes | yes |
| `metadata` / `bookmarks` | yes | yes |
| `search` | no | yes |
| `pages[]` accessor | no | yes |
| `redact` / `redact_text` | no | yes |
| `encrypt` | no | yes |
| `sign` / `verify_signatures` | no | yes |
| `fill_form` / `form_fields` | no | yes |
| `render_page` / `render_pages` | no | yes |
| `add_watermark` / `add_annotation` | no | yes |
| `split` / `reorder` / `rotate` | no | yes |
| `optimize` | no | yes |
| `validate_pdf_a` / `validate_pdf_ua` | no | yes |

If you need PDF-specific features, use `paperjam.open()` on a PDF file (which returns `Document`) or convert the document first with `convert_to("pdf")` and then open the result.
