# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Security

- Bound ZIP entry reads in EPUB, PPTX, and DOCX parsers. A crafted archive
  declaring a tiny compressed size could previously expand to multi-GB on
  decompression; entries are now rejected when the declared or observed
  decompressed size exceeds a per-entry cap.
- Cap `Vec::with_capacity` preallocations in XLSX sheet parsing and PPTX
  slide parsing at reasonable ceilings so attacker-controlled counts can
  no longer trigger large allocations up front.
- `paperjam-mcp`: resolved paths are now sandboxed to the configured
  working directory by default. Absolute paths and `..` traversal that
  escape the working dir are rejected with a structured error. Operators
  can opt out with `--allow-absolute-paths` (or
  `ServerConfig::allow_absolute_paths`).

### Fixed

- Replace panic-prone `f64::partial_cmp(..).unwrap()` in table detection
  (`table/{grid,lattice,stream}.rs`) with `total_cmp`, so malformed PDFs
  producing NaN coordinates no longer crash the parser.
- Replace `get_object_mut().unwrap()` / `as_dict_mut().unwrap()` /
  `from_utf8().unwrap()` across the stamp, watermark, bookmarks, and
  PDF/UA validation modules with structured `PdfError` returns. Malformed
  PDFs now surface typed errors instead of panicking the process.
- Stub drift: add `modify_form_field`, `add_form_field`, and the
  `fill_form.generate_appearances` parameter to `_paperjam.pyi` so mypy
  sees the full PyO3 surface.

### Added

- Crate-level `//!` rustdoc summaries on every workspace crate.
- `rust-toolchain.toml` pins the contributor toolchain to stable with
  `rustfmt`, `clippy`, and the `wasm32-unknown-unknown` target.
- `justfile` with shortcuts for common build / test / lint tasks.
- `[profile.release]` with thin LTO, `codegen-units = 1`, and symbol
  strip. Adds a `release-with-debug` profile for profiling.

### Changed

- `paperjam-async` no longer force-enables `signatures` and `validation`
  on `paperjam-core`. Consumers that need them (e.g. `paperjam-py`)
  continue to enable them explicitly; lightweight async users no longer
  drag in the full signing / validation stack.
- Docs site CI now builds on pull requests (without deploying) so docs
  regressions are caught pre-merge. Binaryen's `wasm-opt` is installed
  so release WASM bundles are size-optimized.

### Docs

- README: CLI examples now use the correct `pj` binary name and accurate
  flags; removed the nonexistent `extract tables --format csv` flag.
- `docs-site/docs/getting-started/installation.md`: replace leftover
  Sphinx build instructions with the Docusaurus workflow, fix the
  clone org, expand the feature-flag table.
- `pyproject.toml`: fill in multi-format description, `readme`,
  `project.urls`, and extra classifiers/keywords so the PyPI page is
  populated. Drop the stale Sphinx `[docs]` extra.

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
