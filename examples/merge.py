"""Merge examples: merge documents, merge files."""

import time
from pathlib import Path

import paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "results" / "merge"


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

    # ── Split into parts first ───────────────────────────────────
    section("Split into 3 parts")
    with timed("split into 3 ranges"):
        parts = doc.split([(1, 5), (100, 110), (500, 505)])
    for i, part in enumerate(parts):
        print(f"  Part {i}: {part.page_count} pages")

    # ── Merge documents ──────────────────────────────────────────
    section("Merge 3 parts back together")
    with timed("merge 3 documents"):
        merged = paperjam.merge(parts)
    expected = 5 + 11 + 6  # 22 pages
    out = OUTPUT_DIR / "merged_3_parts.pdf"
    merged.save(str(out))
    print(f"  Pages: {merged.page_count} (expected {expected})")
    print(f"  Saved: {out}")
    print(f"  Size:  {out.stat().st_size:,} bytes")

    # ── Merge 2 documents ────────────────────────────────────────
    section("Merge first 5 + last 5 pages")
    with timed("split first+last"):
        first5 = doc.split([(1, 5)])[0]
        last5 = doc.split([(doc.page_count - 4, doc.page_count)])[0]
    print(f"  First: {first5.page_count} pages, Last: {last5.page_count} pages")

    with timed("merge 2 documents"):
        merged2 = paperjam.merge([first5, last5])
    out = OUTPUT_DIR / "first5_last5.pdf"
    merged2.save(str(out))
    print(f"  Pages: {merged2.page_count}")
    print(f"  Saved: {out}")
    print(f"  Size:  {out.stat().st_size:,} bytes")

    # Verify content
    first_text = merged2.pages[0].extract_text()[:80].replace("\n", " ")
    last_text = merged2.pages[-1].extract_text()[:80].replace("\n", " ")
    print(f"  Page 1 text:  {first_text}...")
    print(f"  Page {merged2.page_count} text: {last_text}...")

    # ── Merge files from paths ───────────────────────────────────
    section("Merge files from paths")
    # Save the parts as files first
    part_paths = []
    for i, part in enumerate(parts):
        p = OUTPUT_DIR / f"part_{i}.pdf"
        part.save(str(p))
        part_paths.append(str(p))
        print(f"  Saved part {i}: {p}")

    with timed("merge_files"):
        merged3 = paperjam.merge_files(part_paths)
    out = OUTPUT_DIR / "merged_from_files.pdf"
    merged3.save(str(out))
    print(f"  Pages: {merged3.page_count}")
    print(f"  Saved: {out}")
    print(f"  Size:  {out.stat().st_size:,} bytes")

    # ── Summary ──────────────────────────────────────────────────
    total_time = time.perf_counter() - t_start
    print(f"\n{'=' * 60}")
    print(f"  Total: {total_time:.3f}s")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
