"""Insert blank pages into a PDF at specified positions."""

import argparse
import os
from pathlib import Path

import paperjam

# Common page sizes in points (72 points = 1 inch)
PAGE_SIZES = {
    "letter": (612.0, 792.0),
    "a4": (595.28, 841.89),
    "a5": (419.53, 595.28),
    "legal": (612.0, 1008.0),
}


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Insert blank pages into a PDF.",
        epilog="Example: %(prog)s input.pdf -i '0:letter' '3:a4'   (inserts letter-size at start, a4 after page 3)",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "-i",
        "--insert",
        nargs="+",
        required=True,
        metavar="POS:SIZE",
        help=f"Insertions as 'after_page:size' pairs. after_page=0 inserts at the beginning. Size can be {', '.join(PAGE_SIZES)} or WxH in points.",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="Output filename (default: expanded_<input>.pdf)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    positions: list[tuple[int, float, float]] = []
    for spec in args.insert:
        after_str, size_str = spec.split(":", 1)
        after_page = int(after_str)
        if size_str.lower() in PAGE_SIZES:
            w, h = PAGE_SIZES[size_str.lower()]
        else:
            w_str, h_str = size_str.split("x", 1)
            w, h = float(w_str), float(h_str)
        positions.append((after_page, w, h))

    for after_page, w, h in positions:
        where = "beginning" if after_page == 0 else f"after page {after_page}"
        print(f"  Insert blank {w:.0f}x{h:.0f} pts at {where}")

    result = doc.insert_blank_pages(positions)

    basename = os.path.splitext(os.path.basename(args.input))[0]
    name = args.name or f"expanded_{basename}.pdf"
    out = output / name
    result.save(str(out))

    added = result.page_count - doc.page_count
    print(f"\nResult: {result.page_count} pages (+{added} blank)")
    print(f"Saved to {out} ({out.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
