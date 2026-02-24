"""Metadata extraction example: document info, page info."""

import time
from pathlib import Path

import paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "results" / "metadata"


def timed(label: str):
    class Timer:
        def __enter__(self):
            self.t0 = time.perf_counter()
            return self

        def __exit__(self, *_):
            self.elapsed = time.perf_counter() - self.t0
            print(f"  [{label}: {self.elapsed:.4f}s]")

    return Timer()


def section(title: str) -> None:
    print(f"\n{'=' * 60}")
    print(f"  {title}")
    print(f"{'=' * 60}")


def main() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    t_start = time.perf_counter()

    with timed("open"):
        doc = paperjam.open(str(PDF_PATH))
    print(f"  Document: {PDF_PATH.name} ({doc.page_count} pages)")

    # ── Document metadata ────────────────────────────────────────
    section("Document metadata")
    with timed("metadata"):
        meta = doc.metadata

    fields = [
        ("Title", meta.title),
        ("Author", meta.author),
        ("Subject", meta.subject),
        ("Keywords", meta.keywords),
        ("Creator", meta.creator),
        ("Producer", meta.producer),
        ("Creation date", meta.creation_date),
        ("Modification date", meta.modification_date),
        ("PDF version", meta.pdf_version),
        ("Page count", meta.page_count),
        ("Encrypted", meta.is_encrypted),
    ]

    for label, value in fields:
        print(f"  {label:20s}: {value}")

    # Save metadata to file
    out = OUTPUT_DIR / "metadata.txt"
    with open(out, "w") as f:
        for label, value in fields:
            f.write(f"{label}: {value}\n")
        if meta.xmp_metadata:
            f.write(f"\nXMP Metadata:\n{meta.xmp_metadata}\n")
    print(f"\n  Saved: {out}")

    # ── Page info: first, middle, last ───────────────────────────
    section("Page info")
    targets = {
        "First": 0,
        "Middle": doc.page_count // 2,
        "Last": doc.page_count - 1,
    }

    out = OUTPUT_DIR / "page_info.txt"
    with open(out, "w") as f:
        for label, idx in targets.items():
            page = doc.pages[idx]
            info = page.info
            line = (
                f"{label} (page {page.number}): "
                f"{page.width:.1f} x {page.height:.1f} pts, "
                f"rotation={page.rotation}"
            )
            print(f"  {line}")
            f.write(f"{line}\n")
            f.write(f"  PageInfo: {info}\n\n")
    print(f"  Saved: {out}")

    # ── Context manager demo ─────────────────────────────────────
    section("Context manager")
    with paperjam.open(str(PDF_PATH)) as ctx_doc:
        print(f"  Inside:  {ctx_doc.page_count} pages, repr={ctx_doc!r}")
    print(f"  Outside: repr={ctx_doc!r}")

    # ── Indexing & slicing demo ──────────────────────────────────
    section("Page indexing & slicing")
    print(f"  doc.pages[0].number   = {doc.pages[0].number}")
    print(f"  doc.pages[-1].number  = {doc.pages[-1].number}")
    print(f"  len(doc.pages)        = {len(doc.pages)}")
    subset = doc.pages[10:15]
    print(f"  doc.pages[10:15]      = {[p.number for p in subset]}")

    # ── Summary ──────────────────────────────────────────────────
    total_time = time.perf_counter() - t_start
    print(f"\n{'=' * 60}")
    print(f"  Total: {total_time:.3f}s")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
