# paperjam

Fast PDF processing powered by Rust.

## Installation

```bash
pip install paperjam
```

## Quick Start

```python
import paperjam

doc = paperjam.open("report.pdf")

# Extract text
text = doc.pages[0].extract_text()

# Extract tables
tables = doc.pages[0].extract_tables()

# Convert to Markdown
md = doc.to_markdown(layout_aware=True)

# Async support
doc = await paperjam.aopen("report.pdf")
md = await doc.ato_markdown()
```

## Features

- **Text extraction** — plain text, positioned lines, spans with font info
- **Table extraction** — lattice and stream strategies with CSV/DataFrame export
- **PDF to Markdown** — layout-aware conversion for LLM/RAG pipelines
- **Page manipulation** — split, merge, reorder, rotate, delete, insert blank pages
- **Search** — full-text search across pages with bounding boxes
- **Metadata & bookmarks** — read and edit document properties and outline
- **Annotations & watermarks** — add, read, remove annotations; text watermarks
- **Forms** — inspect, fill, create, and modify form fields
- **Security** — encryption (AES-128/RC4), sanitization, true content-stream redaction
- **PDF diff** — text-level comparison of two documents
- **Layout analysis** — multi-column detection, header/footer identification
- **Native async** — powered by Rust and tokio, no Python thread pools
- **WASM playground** — try it in the browser at [docs.byteveda.org/paperjam](https://docs.byteveda.org/paperjam/)

## Documentation

Full docs, API reference, and interactive playground at **[docs.byteveda.org/paperjam](https://docs.byteveda.org/paperjam/)**.

## License

MIT OR Apache-2.0
