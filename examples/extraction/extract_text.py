"""Extract text from a PDF — plain text, lines with bboxes, and spans."""

import argparse
from pathlib import Path

import paperjam


def parse_pages(spec: str, total: int) -> list[int]:
    """Parse '1-5' or '1,3,5' into 1-indexed page numbers."""
    pages: list[int] = []
    for part in spec.split(","):
        if "-" in part:
            start, end = part.split("-", 1)
            pages.extend(range(int(start), int(end) + 1))
        else:
            pages.append(int(part))
    return [p for p in pages if 1 <= p <= total]


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Extract text from a PDF file.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "-p", "--pages",
        help="Pages to extract, e.g. '1-5' or '1,3,5' (default: all)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    page_nums = parse_pages(args.pages, doc.page_count) if args.pages else list(range(1, doc.page_count + 1))

    for num in page_nums:
        page = doc.pages[num - 1]

        # Plain text
        text = page.extract_text()
        out = output / f"page_{num}.txt"
        out.write_text(text)
        print(f"  Page {num}: {len(text):,} chars -> {out.name}")

        # Text lines (first 3)
        lines = page.extract_text_lines()
        if lines:
            print(f"    {len(lines)} lines:")
            for line in lines[:3]:
                b = line.bbox
                print(f"      [{b[0]:.0f},{b[1]:.0f},"
                      f"{b[2]:.0f},{b[3]:.0f}] "
                      f"{line.text[:60]}")

        # Text spans (first 3)
        spans = page.extract_text_spans()
        if spans:
            print(f"    {len(spans)} spans:")
            for span in spans[:3]:
                print(f"      ({span.font_name} "
                      f"{span.font_size}pt) "
                      f'"{span.text[:50]}"')

    print(f"\nSaved {len(page_nums)} page(s) to {output}/")


if __name__ == "__main__":
    main()
