"""Text extraction examples: plain text, lines, spans, bulk."""

import time
from pathlib import Path

import paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "results" / "text"


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

    # ── Plain text: first, middle, last ──────────────────────────
    mid = doc.page_count // 2
    targets = {"first": 0, "middle": mid, "last": doc.page_count - 1}

    for label, idx in targets.items():
        page_num = idx + 1
        section(f"Extract text — page {page_num} ({label})")
        with timed(f"extract_text page {page_num}"):
            text = doc.pages[idx].extract_text()

        out = OUTPUT_DIR / f"page_{page_num}_{label}.txt"
        out.write_text(text)

        preview = text[:200].replace("\n", "\\n")
        print(f"  Length:  {len(text)} chars")
        print(f"  Saved:   {out}")
        print(f"  Preview: {preview}...")

    # ── Text lines ───────────────────────────────────────────────
    section("Text lines — page 5")
    with timed("extract_text_lines page 5"):
        lines = doc.pages[4].extract_text_lines()
    print(f"  Lines: {len(lines)}")

    out = OUTPUT_DIR / "page_5_lines.txt"
    with open(out, "w") as f:
        for line in lines:
            bbox = f"[{line.bbox[0]:.0f},{line.bbox[1]:.0f},{line.bbox[2]:.0f},{line.bbox[3]:.0f}]"
            f.write(f"{bbox} ({len(line.spans)} spans) {line.text}\n")
    print(f"  Saved:  {out}")

    for line in lines[:5]:
        bbox = f"[{line.bbox[0]:.0f},{line.bbox[1]:.0f},{line.bbox[2]:.0f},{line.bbox[3]:.0f}]"
        print(f"    {bbox} ({len(line.spans)} spans) {line.text[:80]}")

    # ── Text spans ───────────────────────────────────────────────
    section("Text spans — page 5")
    with timed("extract_text_spans page 5"):
        spans = doc.pages[4].extract_text_spans()
    print(f"  Spans: {len(spans)}")

    out = OUTPUT_DIR / "page_5_spans.txt"
    with open(out, "w") as f:
        for span in spans:
            f.write(
                f"x={span.x:.1f} y={span.y:.1f} w={span.width:.1f} "
                f"font={span.font_name} size={span.font_size:.1f} "
                f'"{span.text}"\n'
            )
    print(f"  Saved:  {out}")

    for span in spans[:5]:
        print(
            f"    x={span.x:.1f} y={span.y:.1f} w={span.width:.1f} "
            f"font={span.font_name} size={span.font_size:.1f} "
            f'"{span.text[:60]}"'
        )

    # ── Bulk extraction: all pages ───────────────────────────────
    section(f"Bulk extraction — all {doc.page_count} pages")
    with timed(f"extract_text x{doc.page_count}"):
        all_text = []
        for page in doc.pages:
            all_text.append(page.extract_text())

    total_chars = sum(len(t) for t in all_text)
    out = OUTPUT_DIR / "all_pages.txt"
    with open(out, "w") as f:
        for i, text in enumerate(all_text, 1):
            f.write(f"--- Page {i} ---\n{text}\n\n")
    print(f"  Total chars: {total_chars:,}")
    print(f"  Saved:  {out} ({out.stat().st_size:,} bytes)")

    # ── Summary ──────────────────────────────────────────────────
    total_time = time.perf_counter() - t_start
    print(f"\n{'=' * 60}")
    print(f"  Total: {total_time:.3f}s")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
