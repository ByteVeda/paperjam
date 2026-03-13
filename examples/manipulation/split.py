"""Split a PDF into parts by page ranges."""

import argparse
from pathlib import Path

import paperjam


def parse_range(spec: str) -> tuple[int, int]:
    """Parse '1-5' into (1, 5) or '3' into (3, 3)."""
    if "-" in spec:
        start, end = spec.split("-", 1)
        return int(start), int(end)
    n = int(spec)
    return n, n


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Split a PDF into parts by page ranges.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "-r",
        "--ranges",
        nargs="+",
        help="Page ranges to extract, e.g. '1-5 10-20 25'",
    )
    group.add_argument(
        "--each",
        action="store_true",
        help="Split into individual single-page PDFs",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    ranges = [(i, i) for i in range(1, doc.page_count + 1)] if args.each else [parse_range(r) for r in args.ranges]

    parts = doc.split(ranges)

    for (start, end), part in zip(ranges, parts, strict=True):
        name = f"page_{start}.pdf" if start == end else f"pages_{start}_{end}.pdf"
        out = output / name
        part.save(str(out))
        size = out.stat().st_size
        print(f"  Pages {start}-{end}: {part.page_count} page(s) -> {out.name} ({size:,} bytes)")

    print(f"\nSplit into {len(parts)} part(s) in {output}/")


if __name__ == "__main__":
    main()
