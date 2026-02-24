"""Test script exercising paperjam against Learning_Python.pdf (1213 pages)."""

import time
from pathlib import Path

import paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"


def section(title: str) -> None:
    print(f"\n{'=' * 60}")
    print(f"  {title}")
    print(f"{'=' * 60}")


def timed(label: str):
    """Context manager that prints elapsed time."""

    class Timer:
        def __enter__(self):
            self.start = time.perf_counter()
            return self

        def __exit__(self, *_):
            elapsed = time.perf_counter() - self.start
            print(f"  [{label}: {elapsed:.4f}s]")

    return Timer()


def main() -> None:
    # --- Open ---
    section("Opening document")
    with timed("open"):
        doc = paperjam.open(str(PDF_PATH))
    print(f"  Pages: {doc.page_count}")
    print(f"  Repr:  {doc!r}")

    # --- Metadata ---
    section("Metadata")
    with timed("metadata"):
        meta = doc.metadata
    print(f"  Title:       {meta.title}")
    print(f"  Author:      {meta.author}")
    print(f"  Subject:     {meta.subject}")
    print(f"  Creator:     {meta.creator}")
    print(f"  Producer:    {meta.producer}")
    print(f"  PDF version: {meta.pdf_version}")
    print(f"  Encrypted:   {meta.is_encrypted}")
    print(f"  Page count:  {meta.page_count}")

    # --- Page info ---
    section("Page info (first page)")
    page = doc.pages[0]
    print(f"  Number:   {page.number}")
    print(f"  Size:     {page.width:.1f} x {page.height:.1f} pts")
    print(f"  Rotation: {page.rotation}")
    print(f"  Info:     {page.info}")

    # --- Text extraction: single page ---
    section("Text extraction — page 1")
    with timed("extract_text page 1"):
        text = doc.pages[0].extract_text()
    preview = text[:300].replace("\n", "\\n")
    print(f"  Length: {len(text)} chars")
    print(f"  Preview: {preview}...")

    # --- Text extraction: middle page ---
    mid = doc.page_count // 2
    section(f"Text extraction — page {mid + 1} (middle)")
    with timed(f"extract_text page {mid + 1}"):
        text = doc.pages[mid].extract_text()
    preview = text[:300].replace("\n", "\\n")
    print(f"  Length: {len(text)} chars")
    print(f"  Preview: {preview}...")

    # --- Text extraction: last page ---
    section(f"Text extraction — page {doc.page_count} (last)")
    with timed(f"extract_text page {doc.page_count}"):
        text = doc.pages[-1].extract_text()
    preview = text[:300].replace("\n", "\\n")
    print(f"  Length: {len(text)} chars")
    print(f"  Preview: {preview}...")

    # --- Bulk text extraction (first 20 pages) ---
    section("Bulk text extraction — first 20 pages")
    with timed("extract_text x20"):
        total_chars = 0
        for page in doc.pages[0:20]:
            total_chars += len(page.extract_text())
    print(f"  Total chars across 20 pages: {total_chars}")

    # --- Text lines ---
    section("Text lines — page 5")
    with timed("extract_text_lines page 5"):
        lines = doc.pages[4].extract_text_lines()
    print(f"  Line count: {len(lines)}")
    for line in lines[:5]:
        print(
            f"    [{line.bbox[0]:.0f},{line.bbox[1]:.0f},"
            f"{line.bbox[2]:.0f},{line.bbox[3]:.0f}]"
            f" ({len(line.spans)} spans) {line.text[:80]}"
        )

    # --- Text spans ---
    section("Text spans — page 5")
    with timed("extract_text_spans page 5"):
        spans = doc.pages[4].extract_text_spans()
    print(f"  Span count: {len(spans)}")
    for span in spans[:8]:
        print(
            f"    x={span.x:.1f} y={span.y:.1f} w={span.width:.1f}"
            f" font={span.font_name} size={span.font_size:.1f}"
            f' "{span.text[:50]}"'
        )

    # --- Table extraction ---
    section("Table extraction — page 5 (auto)")
    with timed("extract_tables page 5"):
        tables = doc.pages[4].extract_tables()
    print(f"  Tables found: {len(tables)}")
    for i, table in enumerate(tables):
        print(
            f"    Table {i}: {table.row_count} rows x {table.col_count} cols"
            f" strategy={table.strategy}"
        )
        if table.row_count > 0:
            first_row = [c.text for c in table.rows[0].cells]
            print(f"      Header: {first_row}")

    # --- Split ---
    section("Split — pages 1-5")
    with timed("split pages 1-5"):
        parts = doc.split([(1, 5)])
    print(f"  Result: {len(parts)} document(s)")
    print(f"  Part 0 page count: {parts[0].page_count}")

    # --- Save bytes round-trip ---
    section("Save bytes round-trip (split doc)")
    with timed("save_bytes"):
        data = parts[0].save_bytes()
    print(f"  Bytes: {len(data):,}")
    with timed("reopen from bytes"):
        doc2 = paperjam.open(data)
    print(f"  Reopened page count: {doc2.page_count}")
    print(f"  Page 1 text preview: {doc2.pages[0].extract_text()[:100]}...")

    # --- Context manager ---
    section("Context manager")
    with paperjam.open(str(PDF_PATH)) as ctx_doc:
        print(f"  Inside: {ctx_doc.page_count} pages")
    print(f"  Outside: closed={ctx_doc._closed}")

    # --- Indexing & slicing ---
    section("Page indexing & slicing")
    print(f"  doc.pages[0].number  = {doc.pages[0].number}")
    print(f"  doc.pages[-1].number = {doc.pages[-1].number}")
    subset = doc.pages[10:15]
    print(f"  doc.pages[10:15]     = {[p.number for p in subset]}")

    # --- Full extraction benchmark ---
    section(f"Full text extraction — all {doc.page_count} pages")
    with timed(f"extract_text x{doc.page_count}"):
        total = 0
        for page in doc.pages:
            total += len(page.extract_text())
    print(f"  Total chars: {total:,}")

    print(f"\n{'=' * 60}")
    print("  All tests passed!")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
