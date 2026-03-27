# Architecture

paperjam is a mixed Rust/Python library. Python provides the public API and ergonomics; Rust provides the PDF engine, performance, and safety.

```mermaid
flowchart LR
    User(["User Code"])

    subgraph PY["Python  —  py_src/paperjam/"]
        direction TB
        TopFns["open · merge · diff · render · to_markdown · aopen · amerge"]
        DocPage["Document  ·  Page"]
        Mods["feature modules — monkey-patched onto Document and Page at import time"]
        Types["_types.py  ·  _enums.py"]
    end

    subgraph RS["Rust Extension  —  _paperjam.abi3.so"]
        direction TB
        Bindings["PyO3 Bindings  —  crates/paperjam-py"]
        Async["Async Wrappers  —  crates/paperjam-async"]
        Core["Rust Core  —  crates/paperjam-core"]
    end

    subgraph OPT["Optional  —  Cargo feature flags"]
        direction TB
        F1["render  →  pdfium"]
        F2["parallel  →  rayon  (default on)"]
        F3["validation  →  roxmltree"]
        F4["signatures  →  rcgen + p12"]
        F5["mmap  →  memmap2"]
    end

    User --> TopFns
    TopFns --> DocPage
    Mods -.->|"monkey-patches"| DocPage
    Types -.- DocPage
    DocPage -->|"FFI via PyO3"| Bindings
    Bindings -->|"sync"| Core
    Bindings -->|"async"| Async
    Async -->|"spawn_blocking"| Core
    Core --> F1 & F2 & F3 & F4 & F5
```

## Layers

**Python layer** — The public API. `Document` and `Page` are pure-Python classes. Feature modules (`_extraction.py`, `_manipulation.py`, etc.) attach methods onto those classes at import time via simple assignment (`Document.method = _method`), keeping each feature self-contained without subclassing.

**PyO3 boundary** — The compiled extension (`_paperjam.abi3.so`) exposes `RustDocument` and `RustPage` as opaque Python objects. All PDF heavy lifting crosses this boundary via PyO3 FFI. The GIL is released for long-running operations.

**Rust core** — `crates/paperjam-core` owns the PDF object model, parser, text engine, table extractor, manipulation primitives, security operations, and diff algorithm. No Python dependencies; usable as a standalone Rust crate.

**Async layer** — `crates/paperjam-async` wraps `paperjam-core` operations with `tokio::task::spawn_blocking`. The PyO3 bindings expose these as native Python coroutines via `pyo3-async-runtimes::tokio::future_into_py()`. The Python `_async.py` module is a thin shim that imports the Rust async functions and attaches them to `Document` and `Page`.

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
