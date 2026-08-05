# Quickstart

This page gives a brief end-to-end tour of paperjam's most common operations.

## Opening a document

```python
import paperjam

# From a file path
doc = paperjam.open("report.pdf")

# From bytes (e.g. from a web request or database)
pdf_bytes = open("report.pdf", "rb").read()
doc = paperjam.open(pdf_bytes)

# Encrypted PDF
doc = paperjam.open("locked.pdf", password="s3cr3t")

# Context manager — resources freed automatically on exit
with paperjam.open("report.pdf") as doc:
    text = doc.pages[0].extract_text()
```

## Extracting text

```python
doc = paperjam.open("report.pdf")

# All text from a page as a single string
text = doc.pages[0].extract_text()
print(text)

# Iterate every page
for page in doc.pages:
    print(f"=== Page {page.number} ===")
    print(page.extract_text())

# Structured lines with bounding boxes
for line in doc.pages[0].extract_text_lines():
    print(line.text, line.bbox)

# Individual positioned spans with font info
for span in doc.pages[0].extract_text_spans():
    print(span.text, span.font_name, span.font_size)
```

## Extracting tables

```python
from paperjam import TableStrategy

tables = doc.pages[0].extract_tables(strategy=TableStrategy.AUTO)
for table in tables:
    print(f"Table: {table.row_count} rows × {table.col_count} cols")
    print(table.to_list())        # list[list[str]]
    print(table.to_csv())
    # table.to_dataframe()        # requires pandas extra
```

## Converting to Markdown

paperjam can convert an entire PDF to Markdown, which is useful for feeding PDFs into LLM or RAG pipelines:

```python
markdown = doc.to_markdown(
    heading_offset=1,
    include_page_numbers=True,
    layout_aware=True,   # respects multi-column layout
)
print(markdown)

# Or use the convenience function
md = paperjam.to_markdown("report.pdf")
```

## Splitting and merging

```python
# Split into page ranges (1-indexed, inclusive)
parts = doc.split([(1, 5), (6, 10)])
parts[0].save("part1.pdf")

# One document per page
singles = doc.split_pages()

# Merge multiple documents
merged = paperjam.merge([doc_a, doc_b])
merged.save("combined.pdf")

# Merge files by path (no need to open them first)
merged = paperjam.merge_files(["a.pdf", "b.pdf", "c.pdf"])
```

## Searching

```python
results = doc.search("invoice", case_sensitive=False)
for r in results:
    print(f"Page {r.page}, line {r.line_number}: {r.text!r}")

# Regular expressions
results = doc.search(r"\d{3}-\d{2}-\d{4}", use_regex=True)
```

## Saving

```python
# All manipulation methods return a NEW document — immutable pattern
# The original doc is never modified

doc2 = doc.rotate([(1, 90)])   # new document with page 1 rotated
doc2.save("rotated.pdf")

pdf_bytes = doc2.save_bytes()  # in-memory bytes
```

## Async API

For async applications (FastAPI, asyncio) all expensive operations can run without blocking the event loop. Async is powered natively by Rust and tokio — no Python thread pool configuration needed:

```python
import paperjam

async def process():
    doc = await paperjam.aopen("report.pdf")
    text = await doc.pages[0].aextract_text()
    md = await doc.ato_markdown()
    await doc.asave("out.pdf")
```

## Opening any document format

paperjam auto-detects the format and returns the right document type:

```python
import paperjam

# Works with any format
doc = paperjam.open("report.docx")
text = doc.extract_text()
tables = doc.extract_tables()
md = doc.to_markdown()

# All formats supported
for path in ["data.xlsx", "slides.pptx", "page.html", "book.epub"]:
    with paperjam.open(path) as doc:
        print(f"{doc.format}: {doc.page_count} pages")
        print(doc.extract_text()[:200])
```

## Converting between formats

```python
# File-to-file conversion
paperjam.convert("report.docx", "report.pdf")
paperjam.convert("data.xlsx", "data.html")

# In-memory conversion
with open("report.docx", "rb") as f:
    pdf_bytes = paperjam.convert_bytes(
        f.read(),
        from_format="docx",
        to_format="pdf",
    )
```

## Running a pipeline

```python
result = paperjam.run_pipeline("""
name: Extract tables to Excel
input: "invoices/*.pdf"
parallel: true
steps:
  - type: extract_tables
  - type: convert
    format: xlsx
  - type: save
    path: "output/{stem}.xlsx"
""")
print(f"Processed {result['total_files']} files")
```

## Using the CLI

```bash
# Document info
pj info report.docx

# Extract text
pj extract text slides.pptx

# Convert formats
pj convert auto report.docx -o report.pdf

# Run a pipeline
pj pipeline run workflow.yaml --parallel
```

## What's next

- Read the [guides](../guides/extraction) for in-depth coverage of every feature
- Browse the [API reference](../api/document) for complete method signatures and parameter docs
