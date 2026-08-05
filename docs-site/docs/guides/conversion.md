---
sidebar_position: 4
title: Format Conversion
---

# Format Conversion

paperjam can convert documents between PDF, DOCX, XLSX, PPTX, HTML, EPUB, and Markdown. Conversions are available as module-level functions, as methods on document objects, and through the CLI.

## File-to-file conversion

The simplest approach takes an input path and an output path. The formats are inferred from the file extensions:

```python
import paperjam

paperjam.convert("report.docx", "report.pdf")
paperjam.convert("spreadsheet.xlsx", "spreadsheet.html")
paperjam.convert("slides.pptx", "slides.pdf")
```

## In-memory conversion

When working with bytes (e.g. files received over HTTP), use `convert_bytes()`. Since there are no extensions to infer from, you must specify the formats explicitly:

```python
pdf_bytes = paperjam.convert_bytes(
    docx_bytes,
    from_format="docx",
    to_format="pdf",
)
```

## Converting from an open document

Any `AnyDocument` (or `Document`) instance can convert itself to another format:

```python
doc = paperjam.open("presentation.pptx")

# Convert and save to disk
doc.convert_to("pdf", output="presentation.pdf")

# Convert and get bytes back
pdf_bytes = doc.convert_to("pdf")
```

## Supported conversion paths

The matrix below shows which conversions are supported. Every format can be converted to PDF and Markdown. PDF itself can be converted to all other formats.

| From \ To | PDF | DOCX | XLSX | PPTX | HTML | EPUB | Markdown |
|-----------|-----|------|------|------|------|------|----------|
| **PDF**   | --  | yes  | yes  | yes  | yes  | yes  | yes      |
| **DOCX**  | yes | --   | no   | no   | yes  | yes  | yes      |
| **XLSX**  | yes | no   | --   | no   | yes  | no   | yes      |
| **PPTX**  | yes | no   | no   | --   | yes  | no   | yes      |
| **HTML**  | yes | yes  | no   | no   | --   | yes  | yes      |
| **EPUB**  | yes | yes  | no   | no   | yes  | --   | yes      |

Attempting an unsupported conversion raises `ConversionError`.

## Quality notes

Not all conversions are created equal. Some paths produce near-perfect results; others are lossy:

| Conversion | Fidelity | Notes |
|-----------|----------|-------|
| DOCX to PDF | High | Paragraph styles, tables, and images are preserved. Complex custom fonts may render differently. |
| PDF to DOCX | Medium | Layout is approximated. Tables are reconstructed; some formatting may shift. |
| XLSX to PDF | High | Cell formatting and borders are preserved. Charts are rasterized. |
| PDF to Markdown | Medium | Headings, paragraphs, and tables are extracted structurally. Images are not embedded. |
| HTML to PDF | High | CSS is applied. External resources must be accessible at conversion time. |
| PPTX to PDF | High | Slide layout is preserved. Animations are ignored. |
| PDF to XLSX | Low | Only tabular content is extracted. Non-table text is placed in cell A1 per page. |

## Conversion options

Some conversion paths accept extra parameters:

```python
# PDF to DOCX with options
paperjam.convert("report.pdf", "report.docx", layout_aware=True)

# DOCX to PDF with specific page size
paperjam.convert("letter.docx", "letter.pdf", page_size="a4")

# Markdown conversion with heading offset
paperjam.convert("doc.pdf", "doc.md", heading_offset=1, include_page_numbers=True)
```

## Common workflows

### Convert a batch of Word documents to PDF

```python
from pathlib import Path

for docx_path in Path("contracts/").glob("*.docx"):
    pdf_path = docx_path.with_suffix(".pdf")
    paperjam.convert(str(docx_path), str(pdf_path))
    print(f"Converted {docx_path.name}")
```

### Convert an uploaded file to Markdown for RAG

```python
def ingest_document(filename: str, data: bytes) -> str:
    fmt = paperjam.detect_format(filename)
    if fmt is None:
        raise ValueError(f"Unsupported format: {filename}")

    md_bytes = paperjam.convert_bytes(data, from_format=fmt, to_format="markdown")
    return md_bytes.decode("utf-8")
```

### Round-trip: PDF to DOCX for editing, then back to PDF

```python
# Extract to an editable format
paperjam.convert("original.pdf", "editable.docx")

# ... user edits the DOCX ...

# Convert the edited version back
paperjam.convert("editable.docx", "final.pdf")
```

## CLI usage

The `pj convert` command supports all the same conversion paths:

```bash
# Auto-detect formats from extensions
pj convert auto input.docx -o output.pdf

# Explicit format conversion
pj convert to-pdf report.docx -o report.pdf
pj convert to-docx report.pdf -o report.docx
pj convert to-html spreadsheet.xlsx -o spreadsheet.html
pj convert to-epub article.html -o article.epub
pj convert markdown report.pdf -o report.md

# With options
pj convert markdown report.pdf -o report.md --layout-aware --include-page-numbers

# JSON output for scripting
pj convert auto input.docx -o output.pdf --format json
```

See the [CLI Reference](cli) for the full list of convert subcommands and options.
