# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] — 2026-04-04

### Added

- **Multi-format ecosystem**: new crates for DOCX (`paperjam-docx`), XLSX (`paperjam-xlsx`), PPTX (`paperjam-pptx`), HTML (`paperjam-html`), and EPUB (`paperjam-epub`)
- **Shared model layer**: `paperjam-model` crate with format-agnostic types shared across all crates
- **Format conversion engine**: `paperjam-convert` crate for converting between document formats
- **Processing pipelines**: `paperjam-pipeline` crate for YAML-driven multi-step document workflows
- **CLI tool**: `paperjam-cli` crate with commands for all core operations
- **MCP server**: `paperjam-mcp` crate exposing document operations over the Model Context Protocol
- **Studio UI**: `paperjam-studio` — web-based document viewer, converter, and pipeline builder
- **AnyDocument API**: format-agnostic Python wrapper for non-PDF documents
- `open()` auto-detects format and returns `Document` (PDF) or `AnyDocument` (other formats)
- `open_pdf()` for explicit PDF-only usage with strict typing
- `convert()`, `convert_bytes()`, and `detect_format()` Python functions
- `Pipeline` Python class for building and running document processing pipelines
- Async conversion helpers in `paperjam-async`
- AES-256 encryption support
- LTV (Long-Term Validation) signature embedding
- PDF/A conversion (`convert_to_pdf_a`)
- PDF/UA accessibility validation (`validate_pdf_ua`)
- WASM bindings for multi-format operations, conversion, and format detection

### Fixed

- WASM: draw black rectangles over redacted text instead of just removing it
- WASM: serialize `doc_bytes` as `Uint8Array` instead of `Array<number>`
- WASM: make `owner_password` optional in encrypt binding
- WASM: cache-bust module URL to prevent stale JS/WASM mismatch
- WASM: use pure-Rust compression to avoid `libz-sys` failure on `wasm32-unknown-unknown`

### Changed

- Core types (annotations, bookmarks, layout, metadata, etc.) moved from `paperjam-core` to `paperjam-model` and re-exported
- Validation report `level` field now returns full format string (e.g. `"PDF/A-1b"` instead of `"1b"`)
- Examples updated to use `open_pdf()` for type-safe PDF operations

## [0.1.3] — 2026-03-28

### Fixed

- Include LICENSE file in sdist (PyPI rejected upload without it)

## [0.1.2] — 2026-03-27

### Fixed

- Fix publish workflow: move `id-token: write` to job-level permissions
- Upgrade artifact actions to v7/v8
- Add `verbose` and `skip-existing` to PyPI publish step
- Pin Python 3.12/3.13 interpreters (PyO3 0.23.5 doesn't support 3.14)
- Fix license to MIT in pyproject.toml, Cargo.toml, and README

## [0.1.0] — 2026-03-27

### Added

- Text, line, and span extraction with font info and positions
- Table detection and extraction (lattice, stream, auto strategies)
- PDF-to-Markdown conversion with layout awareness
- Document manipulation: merge, split, rotate, reorder, delete, insert blank pages
- Annotation support: add, remove, and inspect annotations
- Link extraction with URL and destination parsing
- Bookmark get/set and automatic TOC generation from headings
- Watermark overlay and page stamping via Form XObject injection
- Region-based and text-based redaction
- Document sanitization
- Form field support: inspect, fill, create, modify, and form-aware merging
- AES-128/256 and RC4 encryption
- Digital signing and verification (`signatures` feature)
- Page rendering and visual diff (`render` feature, requires pdfium)
- PDF/A compliance validation (`validation` feature)
- Parallel processing via rayon (`parallel` feature, default on)
- Memory-mapped file access (`mmap` feature)
- Native async Python API via Rust tokio
- WASM bindings for browser usage
- Docusaurus documentation site with interactive WASM playground
- GitHub Actions CI and PyPI publish workflows
