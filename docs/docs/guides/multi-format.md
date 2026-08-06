---
sidebar_position: 1
title: Multi-Format Documents
---

# Multi-Format Documents

paperjam is not limited to PDF. It can open, extract content from, and convert between six document formats: PDF, DOCX, XLSX, PPTX, HTML, and EPUB.

## Opening any document

`paperjam.open()` detects the format automatically from the file extension or magic bytes. For PDF files it returns a `Document`; for everything else it returns an `AnyDocument`:

```python
import paperjam

# PDF — returns Document
pdf = paperjam.open("report.pdf")

# DOCX — returns AnyDocument
docx = paperjam.open("proposal.docx")

# XLSX — returns AnyDocument
xlsx = paperjam.open("budget.xlsx")
```

This works with in-memory bytes too. When the format cannot be inferred from a file extension, pass it explicitly:

```python
doc = paperjam.open(raw_bytes, format="docx")
```

## Format detection

If you need to know the format before opening, use `detect_format()`:

```python
fmt = paperjam.detect_format("mystery_file")
print(fmt)  # "pdf", "docx", "xlsx", "pptx", "html", "epub", or None
```

It inspects magic bytes first, then falls back to the file extension. Returns `None` when the format is unrecognised.

## Document vs AnyDocument

paperjam has two document classes:

| Class | Returned for | Capabilities |
|-------|-------------|--------------|
| `Document` | PDF | Full feature set: forms, signatures, rendering, annotations, redaction, encryption, page manipulation, and everything in `AnyDocument` |
| `AnyDocument` | DOCX, XLSX, PPTX, HTML, EPUB | Text extraction, table extraction, structure extraction, image extraction, Markdown conversion, format conversion |

PDF-specific operations like `redact_text()`, `encrypt()`, `fill_form()`, `sign()`, `render_page()`, and `add_watermark()` are only available on `Document`. The extraction and conversion methods are shared.

## Opening each format

### PDF

```python
doc = paperjam.open("report.pdf")
print(doc.page_count)
print(doc.pages[0].extract_text())
```

### DOCX (Word)

```python
doc = paperjam.open("proposal.docx")
print(doc.page_count)
print(doc.extract_text())
```

### XLSX (Excel)

Each sheet is treated as a page. Tables are extracted natively from the spreadsheet structure:

```python
doc = paperjam.open("budget.xlsx")
print(f"{doc.page_count} sheets")

tables = doc.extract_tables()
for table in tables:
    print(table.to_csv())
```

### PPTX (PowerPoint)

Each slide is a page:

```python
doc = paperjam.open("deck.pptx")
for i in range(doc.page_count):
    text = doc.extract_text(pages=[i + 1])
    print(f"Slide {i + 1}: {text[:80]}...")
```

### HTML

```python
doc = paperjam.open("article.html")
md = doc.to_markdown()
```

### EPUB

```python
doc = paperjam.open("book.epub")
print(doc.metadata)
text = doc.extract_text()
```

## Common extraction methods

All document types (both `Document` and `AnyDocument`) support these extraction methods:

```python
doc = paperjam.open("any_file.docx")

# Plain text
text = doc.extract_text()

# Text with line positions
lines = doc.extract_text_lines()

# Tables
tables = doc.extract_tables()
for table in tables:
    print(table.headers)
    for row in table.rows:
        print(row)

# Structured content (headings, paragraphs, list items, tables)
blocks = doc.extract_structure()

# Embedded images
images = doc.extract_images()

# Markdown conversion
md = doc.to_markdown()
```

## Context manager support

Both `Document` and `AnyDocument` support the context manager protocol for automatic cleanup:

```python
with paperjam.open("data.xlsx") as doc:
    tables = doc.extract_tables()
    # resources are released when the block exits
```

## When to use AnyDocument directly

In most cases, `paperjam.open()` is all you need. If you want to be explicit, you can instantiate `AnyDocument` directly:

```python
from paperjam import AnyDocument

doc = AnyDocument("report.docx")
doc = AnyDocument(raw_bytes, format="pptx")
```

See the [AnyDocument API reference](../api/any-document) for the full method list and the [Format Conversion guide](conversion) for converting between formats.
