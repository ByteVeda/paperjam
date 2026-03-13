"""Rotate specific pages in a PDF."""

import argparse
from pathlib import Path

import paperjam
from paperjam import _paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Rotate pages in a PDF file.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "-p",
        "--pages",
        required=True,
        help="Comma-separated page numbers, e.g. '1,3,5'",
    )
    parser.add_argument(
        "-d",
        "--degrees",
        type=int,
        default=90,
        choices=[90, 180, 270],
        help="Rotation degrees (default: 90)",
    )
    parser.add_argument(
        "--name",
        default="rotated.pdf",
        help="Output filename (default: rotated.pdf)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    page_nums = [int(p) for p in args.pages.split(",")]

    print(f"Opened: {args.input} ({doc.page_count} pages)")
    print(f"Rotating pages {page_nums} by {args.degrees} degrees")

    for num in page_nums:
        page = doc.pages[num - 1]
        print(f"  Page {num}: {page.width:.0f}x{page.height:.0f} rotation={page.rotation}")

    rotations = [(p, args.degrees) for p in page_nums]
    rotated_inner = _paperjam.rotate_pages(doc._inner, rotations)

    rotated = object.__new__(paperjam.Document)
    rotated._inner = rotated_inner
    rotated._closed = False

    out = output / args.name
    rotated.save(str(out))

    print("\nAfter rotation:")
    for num in page_nums:
        page = rotated.pages[num - 1]
        print(f"  Page {num}: rotation={page.rotation}")

    print(f"\nSaved to {out} ({out.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
