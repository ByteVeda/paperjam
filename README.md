# paperjam

Fast PDF processing powered by Rust.

paperjam is a Python library with a Rust core for reading, manipulating, and analyzing PDF documents. It provides text extraction, table extraction, structured content parsing, page manipulation, annotations, watermarks, optimization, sanitization, and document diffing — all through a clean Pythonic API.

## Features

- **Text extraction** — plain text, positioned lines, or individual spans with font info
- **Table extraction** — lattice and stream strategies with CSV/DataFrame export
- **Structured content** — headings, paragraphs, lists, and tables detected via font heuristics
- **Image extraction** — extract embedded images with metadata
- **Search** — full-text search across pages with bounding box locations
- **Bookmarks** — extract the document outline as a nested tree
- **Metadata** — title, author, dates, PDF version, encryption status, XMP
- **Page manipulation** — split, merge, reorder, rotate
- **Annotations** — read, add, and remove (highlight, underline, link, stamp, etc.)
- **Watermarks** — add text watermarks with configurable position, rotation, opacity, layer
- **Optimization** — compress streams, remove unused objects, deduplicate, strip metadata
- **Sanitization** — remove JavaScript, embedded files, auto-launch actions, link annotations
- **PDF diff** — text-level comparison of two documents using LCS algorithm
- **Password-protected PDFs** — open encrypted documents with password

## Installation

```bash
pip install paperjam
```

### From source

```bash
git clone https://github.com/user/paperjam.git
cd paperjam
pip install maturin
maturin develop --release
```

## Quick Start

```python
import paperjam

# Open a PDF
doc = paperjam.open("document.pdf")

# Extract text from all pages
for page in doc.pages:
    print(page.extract_text())

# Save a modified copy
doc.save("output.pdf")
```

Use as a context manager for automatic cleanup:

```python
with paperjam.open("document.pdf") as doc:
    text = doc.pages[0].extract_text()
```

## API Overview

### Opening Documents

```python
# From file path
doc = paperjam.open("file.pdf")

# From bytes
doc = paperjam.open(pdf_bytes)

# Password-protected
doc = paperjam.open("encrypted.pdf", password="secret")
```

### Text Extraction

```python
page = doc.pages[0]

# Plain text
text = page.extract_text()

# Lines with bounding boxes
for line in page.extract_text_lines():
    print(f"{line.text}  bbox={line.bbox}")
    for span in line.spans:
        print(f"  '{span.text}' font={span.font_name} size={span.font_size}")

# Individual spans
spans = page.extract_text_spans()
```

### Table Extraction

```python
from paperjam import TableStrategy

tables = page.extract_tables(strategy=TableStrategy.AUTO)

for table in tables:
    print(f"{table.row_count} x {table.col_count}")

    # Access cells
    cell = table.cell(0, 0)

    # Export to CSV
    csv_text = table.to_csv()

    # Export to pandas DataFrame
    df = table.to_dataframe()

    # Export to 2D list
    rows = table.to_list()
```

Table extraction parameters:

```python
tables = page.extract_tables(
    strategy="lattice",     # "auto", "lattice", or "stream"
    min_rows=2,
    min_cols=2,
    snap_tolerance=3.0,
    row_tolerance=0.5,
    min_col_gap=10.0,
)
```

### Structured Content Extraction

Extract headings, paragraphs, lists, and tables based on font size heuristics:

```python
blocks = doc.extract_structure(
    heading_size_ratio=1.2,  # fonts 1.2x larger than body → heading
    detect_lists=True,
    include_tables=True,
)

for block in blocks:
    if block.type == "heading":
        print(f"{'#' * block.level} {block.text}")
    elif block.type == "paragraph":
        print(block.text)
    elif block.type == "list_item":
        print(f"{'  ' * block.indent_level}- {block.text}")
    elif block.type == "table":
        print(f"Table: {block.table.row_count}x{block.table.col_count}")
```

Also available per-page via `page.extract_structure()`.

### Image Extraction

```python
for page in doc.pages:
    for img in page.extract_images():
        print(f"{img.width}x{img.height} {img.color_space}")
        img.save("image.png")
```

### Search

```python
# Search across all pages
results = doc.search("keyword", case_sensitive=False, max_results=10)
for r in results:
    print(f"Page {r.page}, line {r.line_number}: {r.text}")

# Search a single page
matches = page.search("keyword")
```

### Metadata

```python
meta = doc.metadata
print(f"Title: {meta.title}")
print(f"Author: {meta.author}")
print(f"Pages: {meta.page_count}")
print(f"PDF version: {meta.pdf_version}")
print(f"Encrypted: {meta.is_encrypted}")
```

### Bookmarks

```python
for bookmark in doc.bookmarks:
    indent = "  " * bookmark.level
    print(f"{indent}{bookmark.title} → page {bookmark.page}")
    for child in bookmark.children:
        print(f"  {indent}{child.title} → page {child.page}")
```

### Page Information

```python
page = doc.pages[0]
print(f"Page {page.number}: {page.width}x{page.height}pt, rotation={page.rotation}°")

info = page.info  # PageInfo dataclass
```

### Annotations

```python
# Read annotations
for annot in page.annotations:
    print(f"{annot.type}: {annot.contents}")

# Add a highlight annotation
from paperjam import AnnotationType

doc = doc.add_annotation(
    page=1,
    annotation_type=AnnotationType.HIGHLIGHT,
    rect=(100, 700, 300, 720),
    contents="Important section",
    color=(1.0, 1.0, 0.0),
    opacity=0.5,
)

# Remove all annotations from a page
doc = doc.remove_annotations(page=1)
```

### Watermarks

```python
from paperjam import WatermarkPosition, WatermarkLayer

doc = doc.add_watermark(
    "CONFIDENTIAL",
    font_size=60.0,
    rotation=45.0,
    opacity=0.3,
    color=(0.5, 0.5, 0.5),
    position=WatermarkPosition.CENTER,
    layer=WatermarkLayer.OVER,
    pages=[1, 2, 3],  # None = all pages
)
doc.save("watermarked.pdf")
```

### Splitting and Merging

```python
# Split by page ranges (1-indexed, inclusive)
parts = doc.split([(1, 5), (6, 10)])
parts[0].save("pages_1_to_5.pdf")

# Split into individual pages
single_pages = doc.split_pages()

# Merge documents
merged = paperjam.merge([doc_a, doc_b, doc_c])

# Merge from file paths
merged = paperjam.merge_files(["a.pdf", "b.pdf", "c.pdf"])
```

### Reordering Pages

```python
# Reverse page order
doc = doc.reorder([3, 2, 1])

# Duplicate pages
doc = doc.reorder([1, 1, 2, 2, 3, 3])

# Drop pages (subset)
doc = doc.reorder([1, 3, 5])
```

### Optimization

```python
optimized, result = doc.optimize(
    compress_streams=True,
    remove_unused=True,
    remove_duplicates=True,
    strip_metadata=False,
)

print(f"Size: {result.original_size} → {result.optimized_size}")
print(f"Reduction: {result.reduction_percent:.1f}%")
print(f"Objects removed: {result.objects_removed}")
optimized.save("optimized.pdf")
```

### Sanitization

Strip potentially dangerous content from a PDF:

```python
sanitized, result = doc.sanitize(
    remove_javascript=True,
    remove_embedded_files=True,
    remove_actions=True,
    remove_links=True,
)

print(f"Removed {result.total_removed} items")
print(f"  JavaScript: {result.javascript_removed}")
print(f"  Embedded files: {result.embedded_files_removed}")
print(f"  Actions: {result.actions_removed}")
print(f"  Links: {result.links_removed}")

for item in result.items:
    loc = f" (page {item.page})" if item.page else ""
    print(f"  [{item.category}]{loc} {item.description}")

sanitized.save("clean.pdf")
```

### PDF Diff

Compare two documents at the text level:

```python
result = paperjam.diff(doc_a, doc_b)

print(f"Pages changed: {result.summary.pages_changed}")
print(f"Additions: {result.summary.total_additions}")
print(f"Removals: {result.summary.total_removals}")

for page_diff in result.page_diffs:
    print(f"--- Page {page_diff.page} ---")
    for op in page_diff.ops:
        if op.kind == "added":
            print(f"  + {op.text_b}")
        elif op.kind == "removed":
            print(f"  - {op.text_a}")
        elif op.kind == "changed":
            print(f"  ~ {op.text_a} → {op.text_b}")
```

### Saving

```python
# Save to file
doc.save("output.pdf")

# Serialize to bytes
pdf_bytes = doc.save_bytes()
```

## Types Reference

All types are frozen dataclasses with `__slots__`.

| Type | Description |
|------|-------------|
| `TextSpan` | Positioned text with font name, size, and coordinates |
| `TextLine` | Line of text with spans and bounding box |
| `Cell` | Table cell with text, bbox, col_span, row_span |
| `Row` | Table row with cells and y-range |
| `Table` | Extracted table with `to_csv()`, `to_dataframe()`, `to_list()` |
| `Metadata` | Document metadata (title, author, dates, version, encryption) |
| `PageInfo` | Page number, dimensions, rotation |
| `Image` | Extracted image with dimensions, color space, and raw data |
| `Bookmark` | TOC entry with title, page, level, nested children |
| `SearchResult` | Search match with page, text, line number, bbox |
| `OptimizeResult` | Optimization stats with `reduction_percent` property |
| `Annotation` | Page annotation with type, rect, contents, color, opacity |
| `ContentBlock` | Structured block: heading, paragraph, list_item, or table |
| `DiffOp` | Single diff change (added, removed, changed) |
| `PageDiff` | Per-page diff operations |
| `DiffSummary` | Diff statistics (pages changed/added/removed, totals) |
| `DiffResult` | Complete diff with page_diffs and summary |
| `SanitizedItem` | Item removed during sanitization (category, description, page) |
| `SanitizeResult` | Sanitization stats with `total_removed` property |

## Enums

| Enum | Values |
|------|--------|
| `TableStrategy` | `AUTO`, `LATTICE`, `STREAM` |
| `Rotation` | `NONE`, `CW_90`, `CW_180`, `CW_270` |
| `AnnotationType` | `TEXT`, `LINK`, `FREE_TEXT`, `HIGHLIGHT`, `UNDERLINE`, `STRIKE_OUT`, `SQUARE`, `CIRCLE`, `LINE`, `STAMP` |
| `WatermarkPosition` | `CENTER`, `TOP_LEFT`, `TOP_RIGHT`, `BOTTOM_LEFT`, `BOTTOM_RIGHT` |
| `WatermarkLayer` | `OVER`, `UNDER` |

## Exceptions

All exceptions inherit from `PdfError`.

| Exception | Description |
|-----------|-------------|
| `PdfError` | Base exception for all paperjam errors |
| `ParseError` | Invalid or corrupt PDF structure |
| `PasswordRequired` | PDF is encrypted and no password was provided |
| `InvalidPassword` | The provided password is incorrect |
| `PageOutOfRange` | Page number is out of bounds (also an `IndexError`) |
| `UnsupportedFeature` | PDF uses a feature not supported by paperjam |
| `TableExtractionError` | Error during table extraction |
| `AnnotationError` | Error adding or removing annotations |
| `WatermarkError` | Error applying a watermark |
| `OptimizationError` | Error during PDF optimization |
| `SanitizeError` | Error during PDF sanitization |

## License

MIT OR Apache-2.0
