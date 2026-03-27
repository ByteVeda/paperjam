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

Open a PDF document.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `path_or_bytes` | `str`, `PathLike`, or `bytes` | File path or raw PDF bytes |
| `password` | `str \| None` | Password for encrypted PDFs |

**Returns** a `Document` object.

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

## Async variants

Each of the above has an async counterpart. See the [Async guide](../guides/async) for full details.

```python
doc    = await paperjam.aopen("file.pdf")
merged = await paperjam.amerge([doc_a, doc_b])
img    = await paperjam.arender("slides.pdf", page=1)
md     = await paperjam.ato_markdown("report.pdf")
```
