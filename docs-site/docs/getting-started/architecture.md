# Architecture

paperjam is a mixed Rust/Python library. Python provides the public API and ergonomics; Rust provides the PDF engine, performance, and safety.

```
paperjam-model (shared types, traits)
    ↑
    ├── paperjam-core (PDF)
    ├── paperjam-docx (Word)
    ├── paperjam-xlsx (Excel)
    ├── paperjam-pptx (PowerPoint)
    ├── paperjam-html (HTML)
    └── paperjam-epub (EPUB)
         ↑
    paperjam-convert (universal converter)
         ↑
    paperjam-pipeline (workflow engine)
         ↑
    ├── paperjam-cli (command line)
    ├── paperjam-mcp (AI agents via MCP)
    ├── paperjam-py (Python bindings)
    ├── paperjam-wasm (WebAssembly)
    ├── paperjam-async (async wrappers)
    └── paperjam-studio (web UI)
```

```mermaid
---
config:
  layout: elk
---
flowchart TB
 subgraph PY["Python  —  py_src/paperjam/"]
    direction TB
        TopFns["open · merge · diff · render · to_markdown · convert · run_pipeline"]
        DocPage["Document  ·  Page  ·  AnyDocument"]
        Mods["feature modules — monkey-patched onto Document and Page at import time"]
        Types["_types.py  ·  _enums.py"]
  end
 subgraph RS["Rust Extension  —  _paperjam.abi3.so"]
    direction TB
        Bindings["PyO3 Bindings  —  crates/paperjam-py"]
        Async["Async Wrappers  —  crates/paperjam-async"]
        Convert["Universal Converter  —  crates/paperjam-convert"]
        Pipeline["Pipeline Engine  —  crates/paperjam-pipeline"]
        Model["Shared Model  —  crates/paperjam-model"]
        Core["PDF Core  —  crates/paperjam-core"]
        Docx["Word  —  crates/paperjam-docx"]
        Xlsx["Excel  —  crates/paperjam-xlsx"]
        Pptx["PowerPoint  —  crates/paperjam-pptx"]
  end
 subgraph IFACE["Interfaces"]
    direction TB
        CLI["CLI  —  crates/paperjam-cli"]
        MCP["MCP Server  —  crates/paperjam-mcp"]
        WASM["WebAssembly  —  crates/paperjam-wasm"]
        Studio["Web UI  —  crates/paperjam-studio"]
  end
 subgraph OPT["Optional  —  Cargo feature flags"]
    direction TB
        F1["render  →  pdfium"]
        F2["parallel  →  rayon  (default on)"]
        F3["validation  →  roxmltree"]
        F4["signatures  →  rcgen + p12"]
        F5["mmap  →  memmap2"]
  end
    User(["User Code"]) --> TopFns
    TopFns --> DocPage
    Mods -. "monkey-patches" .-> DocPage
    Types -.- DocPage
    DocPage -- FFI via PyO3 --> Bindings
    Bindings -- sync --> Core & Docx & Xlsx & Pptx
    Bindings -- async --> Async
    Bindings -- convert --> Convert
    Bindings -- pipeline --> Pipeline
    Async -- spawn_blocking --> Core
    Convert --> Core & Docx & Xlsx & Pptx
    Pipeline --> Convert
    Core --> Model & F1 & F2 & F3 & F4 & F5
    Docx --> Model
    Xlsx --> Model
    Pptx --> Model
    CLI --> Convert & Pipeline
    MCP --> Convert & Pipeline
    WASM --> Convert & Pipeline
    Studio --> Convert & Pipeline
```

## Layers

**Python layer** — The public API. `Document` and `Page` are pure-Python classes for PDFs. `AnyDocument` is the format-agnostic wrapper returned by `open()` for non-PDF formats. Feature modules (`_extraction.py`, `_manipulation.py`, etc.) attach methods onto those classes at import time via simple assignment (`Document.method = _method`), keeping each feature self-contained without subclassing.

**PyO3 boundary** — The compiled extension (`_paperjam.abi3.so`) exposes `RustDocument` and `RustPage` as opaque Python objects. All document heavy lifting crosses this boundary via PyO3 FFI. The GIL is released for long-running operations.

**Shared model** — `crates/paperjam-model` defines the common traits and types shared across all format crates: `DocumentLike`, `PageLike`, `ContentBlock`, `Table`, etc. Each format crate implements these traits.

**Format crates** — Each document format has its own crate: `paperjam-core` (PDF), `paperjam-docx` (Word), `paperjam-xlsx` (Excel), `paperjam-pptx` (PowerPoint). They all implement the shared model traits, providing a uniform API regardless of format.

**Universal converter** — `crates/paperjam-convert` bridges between formats. It uses the format crates to read one format and write another, supporting conversions like DOCX to PDF, XLSX to HTML, etc.

**Pipeline engine** — `crates/paperjam-pipeline` provides a YAML/JSON-driven workflow system for batch processing. It orchestrates multi-step operations across files with parallel execution support.

**Async layer** — `crates/paperjam-async` wraps core operations with `tokio::task::spawn_blocking`. The PyO3 bindings expose these as native Python coroutines via `pyo3-async-runtimes::tokio::future_into_py()`. The Python `_async.py` module is a thin shim that imports the Rust async functions and attaches them to `Document` and `Page`.

**CLI** — `crates/paperjam-cli` provides the `pj` command-line tool for document operations: info, extract, convert, pipeline, and more.

**MCP server** — `crates/paperjam-mcp` exposes paperjam capabilities as an MCP server, allowing AI assistants like Claude Code and Cursor to process documents directly.

**Feature flags** — Optional capabilities gated behind Cargo features. `parallel` (rayon) is on by default. `render`, `signatures`, `validation`, and `mmap` must be enabled at compile time.

## Data flow

```mermaid
sequenceDiagram
    participant User
    participant PyAPI  as Python API
    participant PyO3   as PyO3 Bindings
    participant Rust   as Rust Core

    User->>PyAPI: paperjam.open("file.pdf")
    PyAPI->>PyO3: RustDocument.open(path)
    PyO3->>Rust: parse PDF bytes
    Rust-->>PyO3: RustDocument handle
    PyO3-->>PyAPI: opaque RustDocument
    PyAPI-->>User: Document(._inner = RustDocument)

    User->>PyAPI: page.extract_text()
    PyAPI->>PyO3: RustPage.extract_text()
    PyO3->>Rust: walk content stream
    Rust-->>PyO3: raw str
    PyO3-->>PyAPI: str
    PyAPI-->>User: "Hello world…"

    User->>PyAPI: doc.split([(1, 3)])
    PyAPI->>PyO3: RustDocument.split(ranges)
    PyO3->>Rust: clone + filter pages
    Rust-->>PyO3: new RustDocument
    PyO3-->>PyAPI: opaque RustDocument
    PyAPI-->>User: new Document
```
