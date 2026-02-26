"""Reorder, reverse, or subset pages in a PDF."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(description="Reorder pages in a PDF file.")
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "-p", "--pages",
        help="Comma-separated page numbers in desired order, e.g. '5,3,1,4,2'",
    )
    group.add_argument(
        "--reverse",
        action="store_true",
        help="Reverse the page order",
    )
    parser.add_argument(
        "--name", default="reordered.pdf",
        help="Output filename (default: reordered.pdf)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    if args.reverse:
        order = list(range(doc.page_count, 0, -1))
        print("Reversing page order")
    else:
        order = [int(p) for p in args.pages.split(",")]
        print(f"New page order: {order}")

    result = doc.reorder(order)

    out = output / args.name
    result.save(str(out))

    print(f"\nResult: {result.page_count} pages")
    for i in range(min(result.page_count, 5)):
        text = result.pages[i].extract_text()[:60].replace("\n", " ").strip()
        print(f"  Page {i + 1}: {text}...")

    print(f"\nSaved to {out} ({out.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
