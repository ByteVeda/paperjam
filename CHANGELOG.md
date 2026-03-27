# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
