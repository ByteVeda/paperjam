# Async API

paperjam provides native async support powered by Rust and [tokio](https://tokio.rs/). All CPU-bound operations have async counterparts that run on tokio's blocking thread pool, keeping your asyncio event loop responsive.

Under the hood, the `paperjam-async` Rust crate wraps `paperjam-core` operations with `tokio::task::spawn_blocking`, and the `paperjam-py` bindings expose these as native Python coroutines via [`pyo3-async-runtimes`](https://github.com/PyO3/pyo3-async-runtimes). No Python-level thread pool management is needed.

## Top-level async functions

```python
import paperjam

# Open a document without blocking
doc = await paperjam.aopen("report.pdf")
doc = await paperjam.aopen(pdf_bytes)
doc = await paperjam.aopen("locked.pdf", password="secret")

# Merge without blocking
merged = await paperjam.amerge([doc_a, doc_b])

# Render without opening a Document first
img = await paperjam.arender("slides.pdf", page=1, dpi=150)

# Convert to Markdown without opening a Document first
md = await paperjam.ato_markdown("report.pdf")
```

## Document async methods

Every long-running document method has an async equivalent prefixed with `a`:

```python
doc = await paperjam.aopen("report.pdf")

# Saving
await doc.asave("output.pdf")
data = await doc.asave_bytes()

# Rendering (requires render feature)
img   = await doc.arender_page(1, dpi=150)
imgs  = await doc.arender_pages(pages=[1, 2, 3])

# Extraction
tables  = await doc.aextract_tables()
md      = await doc.ato_markdown()
results = await doc.asearch("keyword")

# Comparison
diff_result = await doc.adiff(other_doc)

# Redaction
redacted, result = await doc.aredact_text("SSN:")
```

## Page async methods

```python
page = doc.pages[0]

text   = await page.aextract_text()
tables = await page.aextract_tables()
md     = await page.ato_markdown()
```

## FastAPI example

```python
from fastapi import FastAPI, UploadFile
from fastapi.responses import Response
import paperjam

app = FastAPI()

@app.post("/extract-text")
async def extract_text(file: UploadFile):
    data = await file.read()
    doc = await paperjam.aopen(data)
    text = await doc.pages[0].aextract_text()
    return {"text": text}

@app.post("/render-page")
async def render_page(file: UploadFile, page: int = 1, dpi: int = 150):
    data = await file.read()
    doc = await paperjam.aopen(data)
    img = await doc.arender_page(page, dpi=dpi)
    return Response(content=img.data, media_type="image/png")

@app.post("/to-markdown")
async def to_markdown(file: UploadFile):
    data = await file.read()
    doc = await paperjam.aopen(data)
    md = await doc.ato_markdown(layout_aware=True)
    return {"markdown": md}
```

## Concurrency example

Because each async call runs on tokio's blocking thread pool, you can process multiple documents concurrently:

```python
import asyncio
import paperjam

async def process_file(path: str) -> str:
    doc = await paperjam.aopen(path)
    return await doc.ato_markdown()

async def main():
    paths = [f"report_{i}.pdf" for i in range(10)]
    results = await asyncio.gather(*[process_file(p) for p in paths])
    for path, md in zip(paths, results):
        print(f"{path}: {len(md)} chars")

asyncio.run(main())
```

## Naming convention

All async methods use the `a` prefix:

| Sync | Async |
|------|-------|
| `paperjam.open()` | `paperjam.aopen()` |
| `paperjam.merge()` | `paperjam.amerge()` |
| `paperjam.to_markdown()` | `paperjam.ato_markdown()` |
| `doc.save()` | `doc.asave()` |
| `doc.save_bytes()` | `doc.asave_bytes()` |
| `doc.render_page()` | `doc.arender_page()` |
| `doc.render_pages()` | `doc.arender_pages()` |
| `doc.extract_tables()` | `doc.aextract_tables()` |
| `doc.to_markdown()` | `doc.ato_markdown()` |
| `doc.search()` | `doc.asearch()` |
| `doc.diff()` | `doc.adiff()` |
| `doc.redact_text()` | `doc.aredact_text()` |
| `page.extract_text()` | `page.aextract_text()` |
| `page.extract_tables()` | `page.aextract_tables()` |
| `page.to_markdown()` | `page.ato_markdown()` |
