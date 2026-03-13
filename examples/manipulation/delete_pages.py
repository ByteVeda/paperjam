"""Delete specific pages from a PDF."""

import argparse
import os
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(description="Delete pages from a PDF file.")
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
        help="Comma-separated 1-indexed page numbers to delete, e.g. '2,4,6'",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="Output filename (default: deleted_<input>.pdf)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    pages_to_delete = [int(p) for p in args.pages.split(",")]
    print(f"Deleting pages: {pages_to_delete}")

    result = doc.delete_pages(pages_to_delete)

    basename = os.path.splitext(os.path.basename(args.input))[0]
    name = args.name or f"deleted_{basename}.pdf"
    out = output / name
    result.save(str(out))

    print(f"\nResult: {result.page_count} pages (removed {doc.page_count - result.page_count})")
    for i in range(min(result.page_count, 5)):
        text = result.pages[i].extract_text()[:60].replace("\n", " ").strip()
        print(f"  Page {i + 1}: {text}...")

    print(f"\nSaved to {out} ({out.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
