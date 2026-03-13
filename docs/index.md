# paperjam

**Fast PDF processing powered by Rust.**

paperjam is a Python library with a Rust core for reading, manipulating, and analysing PDF documents.
It exposes text and table extraction, document manipulation, annotations, security tools, forms, and more —
all through a clean, Pythonic API.

## Install

```
pip install paperjam
```

## Feature highlights

- Text, table, image, and structured-content extraction
- Multi-column layout detection and correct reading order
- PDF-to-Markdown conversion (great for LLM/RAG pipelines)
- Split, merge, rotate, reorder, delete, and insert pages
- Annotations, watermarks, and page stamping
- True content-stream redaction (not cosmetic overlay)
- Sanitization and encryption
- Interactive form inspection, filling, and creation
- Text-level and visual (pixel) PDF diffing
- Page rendering to PNG/JPEG
- Digital signature verification and signing
- Async API (CPU-bound ops run in a thread pool)

```{toctree}
:maxdepth: 1
:caption: Getting started

installation
quickstart
architecture
```

```{toctree}
:maxdepth: 2
:caption: Guides

guides/index
```

```{toctree}
:maxdepth: 2
:caption: API Reference

api/index
```
