"""Split examples: extract page ranges, individual pages."""

import time
from pathlib import Path

import paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "results" / "split"


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

    # ── Split by range ───────────────────────────────────────────
    section("Split by range — pages 1-5")
    with timed("split [(1, 5)]"):
        parts = doc.split([(1, 5)])
    part = parts[0]
    out = OUTPUT_DIR / "pages_1_to_5.pdf"
    part.save(str(out))
    print(f"  Pages: {part.page_count}")
    print(f"  Saved: {out}")
    print(f"  Size:  {out.stat().st_size:,} bytes")
    print(f"  Text preview: {part.pages[0].extract_text()[:100]}...")

    # ── Split multiple ranges at once ────────────────────────────
    section("Split multiple ranges — pages 1-10, 500-510, 1200-1213")
    ranges = [(1, 10), (500, 510), (1200, doc.page_count)]
    with timed(f"split {ranges}"):
        parts = doc.split(ranges)
    for i, (r, part) in enumerate(zip(ranges, parts, strict=True)):
        out = OUTPUT_DIR / f"range_{r[0]}_{r[1]}.pdf"
        part.save(str(out))
        print(f"  Part {i}: pages {r[0]}-{r[1]} → {part.page_count} pages")
        print(f"    Saved: {out} ({out.stat().st_size:,} bytes)")

    # ── Split single pages ───────────────────────────────────────
    section("Split single pages — pages 1, 100, 607, 1213")
    targets = [1, 100, 607, doc.page_count]
    with timed(f"split {len(targets)} single pages"):
        singles = doc.split([(p, p) for p in targets])
    for p, single in zip(targets, singles, strict=True):
        out = OUTPUT_DIR / f"single_page_{p}.pdf"
        single.save(str(out))
        text = single.pages[0].extract_text()
        print(f"  Page {p}: {single.page_count} page, {out.stat().st_size:,} bytes")
        print(f"    Text: {text[:80].replace(chr(10), ' ')}...")
        print(f"    Saved: {out}")

    # ── Save bytes round-trip ────────────────────────────────────
    section("Save bytes round-trip")
    with timed("save_bytes"):
        data = singles[0].save_bytes()
    print(f"  Bytes: {len(data):,}")
    with timed("reopen from bytes"):
        reopened = paperjam.open(data)
    print(f"  Reopened: {reopened.page_count} page(s)")
    print(f"  Text: {reopened.pages[0].extract_text()[:80]}...")

    # ── Summary ──────────────────────────────────────────────────
    total_time = time.perf_counter() - t_start
    print(f"\n{'=' * 60}")
    print(f"  Total: {total_time:.3f}s")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
