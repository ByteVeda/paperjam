# Text & Content Extraction

paperjam provides several levels of text extraction: raw strings, line-level objects with bounding boxes, and individual positioned spans. It also supports structured extraction that identifies headings, paragraphs, list items, and tables.

:::tip Multi-Format Support
All extraction methods work across formats. Use `paperjam.open()` with any supported file:

```python
# Works with PDF, DOCX, XLSX, PPTX, HTML, EPUB
doc = paperjam.open("report.docx")
text = doc.extract_text()
tables = doc.extract_tables()
structure = doc.extract_structure()
```
:::

## Plain text

The simplest way to get text out of a page is `extract_text()`. It returns a plain string with newlines preserved:

```python
import paperjam

doc = paperjam.open("report.pdf")
text = doc.pages[0].extract_text()
print(text)
```

## Text lines

`extract_text_lines()` returns a list of `TextLine` objects. Each `TextLine` carries the full text of the line, its bounding box `(x1, y1, x2, y2)` in PDF points (origin at bottom-left), and a tuple of `TextSpan` objects that make up the line:

```python
for line in doc.pages[0].extract_text_lines():
    x1, y1, x2, y2 = line.bbox
    print(f"[{x1:.1f},{y1:.1f},{x2:.1f},{y2:.1f}] {line.text}")
    for span in line.spans:
        print(f"  span: {span.text!r}  font={span.font_name} size={span.font_size}")
```

## Text spans

`extract_text_spans()` returns every individual positioned text fragment without grouping them into lines. This is useful when you need the exact `(x, y)` position, width, font name, and font size of each fragment:

```python
for span in doc.pages[0].extract_text_spans():
    print(span.text, span.x, span.y, span.font_name, span.font_size, span.width)
```

`TextSpan` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `text` | `str` | The text content |
| `x` | `float` | Left edge in PDF points |
| `y` | `float` | Baseline y in PDF points |
| `width` | `float` | Width of the span |
| `font_size` | `float` | Font size in points |
| `font_name` | `str` | Font name as embedded in the PDF |

## Structured content

`extract_structure()` recognises headings (by relative font size), paragraphs, list items, and tables. Each block is returned as a `ContentBlock`:

```python
blocks = doc.extract_structure(
    heading_size_ratio=1.2,   # font must be 1.2× median to count as heading
    detect_lists=True,
    include_tables=True,
    layout_aware=False,       # set True for multi-column docs
)

for block in blocks:
    if block.type == "heading":
        print(f"{'#' * (block.level or 1)} {block.text}  (page {block.page})")
    elif block.type == "paragraph":
        print(block.text)
    elif block.type == "list_item":
        indent = "  " * (block.indent_level or 0)
        print(f"{indent}- {block.text}")
    elif block.type == "table":
        print(block.table.to_csv())
```

`ContentBlock` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `type` | `str` | `"heading"`, `"paragraph"`, `"list_item"`, `"table"` |
| `page` | `int` | 1-based page number |
| `text` | `str \| None` | Text content (None for tables) |
| `level` | `int \| None` | Heading level 1–6 (headings only) |
| `indent_level` | `int \| None` | Nesting depth (list items only) |
| `bbox` | `tuple \| None` | Bounding box `(x1, y1, x2, y2)` |
| `table` | `Table \| None` | The `Table` object (tables only) |

## Image extraction

Embedded images can be extracted from any page:

```python
for img in doc.pages[0].extract_images():
    print(f"{img.width}×{img.height} {img.color_space} {img.bits_per_component}bpc")
    print(f"  filters: {img.filters}")
    img.save("extracted.png")   # writes raw bytes to disk
```

`Image` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `width` | `int` | Width in pixels |
| `height` | `int` | Height in pixels |
| `color_space` | `str \| None` | e.g. `"DeviceRGB"`, `"DeviceCMYK"` |
| `bits_per_component` | `int \| None` | Bit depth |
| `filters` | `list[str]` | PDF stream filters applied, e.g. `["DCTDecode"]` |
| `data` | `bytes` | Raw encoded image data |

## Searching

`search()` is available on both `Document` and `Page`:

```python
# Search the whole document
results = doc.search("invoice", case_sensitive=False, max_results=50)

# Search a single page
results = doc.pages[2].search("total", case_sensitive=False)

# Regex search
results = doc.search(r"\$[\d,]+\.\d{2}", use_regex=True)

for r in results:
    print(f"Page {r.page}, line {r.line_number}: {r.text!r}")
    if r.bbox:
        print(f"  at {r.bbox}")
```

`SearchResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `text` | `str` | Full line text containing the match |
| `line_number` | `int` | 1-based line index on the page |
| `bbox` | `tuple \| None` | Bounding box of the matching line |

## Link extraction

```python
# All links across all pages
links = doc.extract_links()

# Links on a single page
links = doc.pages[0].extract_links()

for link in links:
    print(f"Page {link.page}  url={link.url}  dest={link.destination}")
```

`Link` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `rect` | `tuple` | Clickable area `(x1, y1, x2, y2)` |
| `url` | `str \| None` | External URL, if any |
| `destination` | `dict \| None` | Internal destination (page jump), if any |
| `contents` | `str \| None` | Alternative text |

## PDF to Markdown

For LLM and RAG pipelines, converting a PDF to Markdown is often more useful than plain text:

```python
# From a Document
md = doc.to_markdown(
    heading_offset=1,            # shift heading levels by N
    include_page_numbers=True,   # insert <!-- page N --> comments
    html_tables=False,           # use pipe-style tables by default
    layout_aware=True,           # respect multi-column layout
)

# From a single page
md = doc.pages[0].to_markdown()

# Convenience function — no need to open the document first
md = paperjam.to_markdown("report.pdf")
```
