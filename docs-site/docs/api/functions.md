# Module-level functions

These functions are importable directly from `paperjam`.

---

## `open`

```python
paperjam.open(
    path_or_bytes: str | os.PathLike | bytes,
    *,
    password: str | None = None,
) -> Document
```

Open a document. The format is auto-detected from the file extension or content. For PDF files, returns a `Document` object. For other formats (DOCX, XLSX, PPTX, HTML, EPUB), returns an `AnyDocument` with the same extraction interface.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `path_or_bytes` | `str`, `PathLike`, or `bytes` | File path or raw document bytes |
| `password` | `str \| None` | Password for encrypted PDFs |

**Returns** a `Document` (for PDFs) or `AnyDocument` (for other formats).

**Raises** `PasswordRequired` if the PDF is encrypted and no password is given. `InvalidPassword` if the password is wrong. `ParseError` if the file is not a valid PDF.

**Example**

```python
import paperjam

doc = paperjam.open("report.pdf")
doc = paperjam.open(pdf_bytes)
doc = paperjam.open("locked.pdf", password="secret")

with paperjam.open("report.pdf") as doc:
    text = doc.pages[0].extract_text()
```

---

## `merge`

```python
paperjam.merge(documents: list[Document]) -> Document
```

Merge a list of `Document` objects into a single new document.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `documents` | `list[Document]` | Documents to merge, in order |

**Returns** a new `Document` containing all pages.

**Example**

```python
merged = paperjam.merge([cover, body, appendix])
merged.save("complete.pdf")
```

---

## `merge_files`

```python
paperjam.merge_files(paths: list[str]) -> Document
```

Open PDF files by path and merge them into a single new document. Equivalent to `merge([open(p) for p in paths])` but more efficient.

**Example**

```python
merged = paperjam.merge_files(["cover.pdf", "body.pdf", "appendix.pdf"])
```

---

## `diff`

```python
paperjam.diff(doc_a: Document, doc_b: Document) -> DiffResult
```

Compare two documents at the text level and return a `DiffResult`.

Also available as `doc_a.diff(doc_b)`.

**Example**

```python
result = paperjam.diff(old, new)
print(result.summary.pages_changed)
```

---

## `to_markdown`

```python
paperjam.to_markdown(
    path: str,
    *,
    heading_offset: int = 0,
    include_page_numbers: bool = False,
    html_tables: bool = False,
    layout_aware: bool = False,
) -> str
```

Open a PDF file and convert its entire content to Markdown in one call. Useful when you just need the Markdown string and do not need to interact with the `Document` object.

**Example**

```python
md = paperjam.to_markdown("report.pdf", layout_aware=True)
```

---

## `render`

```python
paperjam.render(
    path: str,
    *,
    page: int = 1,
    dpi: float = 150,
    format: str = "png",
    quality: int = 85,
) -> RenderedImage
```

Open a PDF and render a single page to an image. Requires the `render` feature.

**Example**

```python
img = paperjam.render("slides.pdf", page=3, dpi=300)
img.save("slide3.png")
```

---

---

## `detect_format`

```python
paperjam.detect_format(path: str) -> str
```

Detect the document format from a file path. Returns a format string: `"pdf"`, `"docx"`, `"xlsx"`, `"pptx"`, `"html"`, `"epub"`, or `"unknown"`.

---

## `convert`

```python
paperjam.convert(input_path: str, output_path: str) -> dict
```

Convert a file from one format to another. Formats are auto-detected from extensions. Returns a dict with conversion statistics.

**Example**

```python
paperjam.convert("report.docx", "report.pdf")
paperjam.convert("data.xlsx", "data.html")
```

---

## `convert_bytes`

```python
paperjam.convert_bytes(data: bytes, *, from_format: str, to_format: str) -> bytes
```

Convert in-memory bytes between formats. Returns the converted document bytes.

**Example**

```python
with open("report.docx", "rb") as f:
    pdf_bytes = paperjam.convert_bytes(f.read(), from_format="docx", to_format="pdf")
```

---

## `run_pipeline`

```python
paperjam.run_pipeline(yaml_or_json: str) -> dict
```

Run a document processing pipeline from a YAML or JSON definition string. Returns a dict with `total_files`, `succeeded`, `failed`, `skipped`, and `file_results`.

**Example**

```python
result = paperjam.run_pipeline("""
name: Extract tables to Excel
input: "invoices/*.pdf"
steps:
  - type: extract_tables
  - type: convert
    format: xlsx
""")
print(f"Processed {result['total_files']} files")
```

---

## `validate_pipeline`

```python
paperjam.validate_pipeline(yaml_or_json: str) -> None
```

Validate a pipeline definition without running it. Raises `PipelineError` if invalid.

---

## Async variants

Each of the above has an async counterpart. See the [Async guide](../guides/async) for full details.

```python
doc    = await paperjam.aopen("file.pdf")
merged = await paperjam.amerge([doc_a, doc_b])
img    = await paperjam.arender("slides.pdf", page=1)
md     = await paperjam.ato_markdown("report.pdf")
```
