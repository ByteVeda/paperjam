"""Rotate the middle 4 pages of Learning_Python.pdf — two use cases."""

import time
from pathlib import Path

import paperjam
from paperjam import _paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "results" / "rotate"


def timed(label: str):
    class Timer:
        def __enter__(self):
            self.t0 = time.perf_counter()
            return self

        def __exit__(self, *_):
            self.elapsed = time.perf_counter() - self.t0
            print(f"  [{label}: {self.elapsed:.4f}s]")

    return Timer()


def main() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    t_start = time.perf_counter()

    with timed("open"):
        doc = paperjam.open(str(PDF_PATH))

    total = doc.page_count
    mid = total // 2
    mid_pages = [mid - 1, mid, mid + 1, mid + 2]

    print(f"\nDocument: {PDF_PATH.name} ({total} pages)")
    print(f"Target pages: {mid_pages}")

    # Before state
    for p in mid_pages:
        page = doc.pages[p - 1]
        print(f"  Page {p}: {page.width:.0f}x{page.height:.0f} rotation={page.rotation}")

    # ── Use case 1: Whole file with 4 pages rotated ──────────────
    print(f"\n{'=' * 60}")
    print("  Use case 1: Whole file, 4 pages rotated")
    print(f"{'=' * 60}")

    rotations = [(p, 90) for p in mid_pages]
    with timed("rotate"):
        rotated_inner = _paperjam.rotate_pages(doc._inner, rotations)

    rotated_full = object.__new__(paperjam.Document)
    rotated_full._inner = rotated_inner
    rotated_full._closed = False

    out_full = OUTPUT_DIR / "full_with_rotated.pdf"
    with timed("save whole file"):
        rotated_full.save(str(out_full))

    print(f"  Pages: {rotated_full.page_count}")
    print(f"  Saved: {out_full}")
    print(f"  Size:  {out_full.stat().st_size:,} bytes")
    for p in mid_pages:
        page = rotated_full.pages[p - 1]
        print(f"  Page {p}: rotation={page.rotation}")

    # ── Use case 2: Extract only the rotated pages ────────────────
    print(f"\n{'=' * 60}")
    print("  Use case 2: Extract only the 4 rotated pages")
    print(f"{'=' * 60}")

    with timed("split (extract 4 pages from 1213)"):
        parts = rotated_full.split([(mid_pages[0], mid_pages[-1])])
    extracted = parts[0]

    out_extract = OUTPUT_DIR / "rotated_mid4.pdf"
    with timed("save extracted"):
        extracted.save(str(out_extract))

    print(f"  Pages: {extracted.page_count}")
    print(f"  Saved: {out_extract}")
    print(f"  Size:  {out_extract.stat().st_size:,} bytes")
    for i in range(extracted.page_count):
        page = extracted.pages[i]
        print(f"  Page {i + 1}: rotation={page.rotation}")

    # ── Summary ───────────────────────────────────────────────────
    total_time = time.perf_counter() - t_start
    print(f"\n{'=' * 60}")
    print(f"  Total: {total_time:.3f}s")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
