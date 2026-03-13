"""Extract bookmarks (table of contents) from a PDF."""

import argparse
from pathlib import Path

import paperjam


def print_tree(
    bookmarks: list[paperjam.Bookmark],
    indent: int = 0,
) -> list[str]:
    """Pretty-print bookmark tree, returning lines for file output."""
    lines: list[str] = []
    for bm in bookmarks:
        prefix = "  " * indent
        line = f"{prefix}{bm.title} (page {bm.page})"
        print(f"  {line}")
        lines.append(line)
        if bm.children:
            lines.extend(print_tree(list(bm.children), indent + 1))
    return lines


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Extract bookmarks / TOC from a PDF.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    bookmarks = doc.bookmarks

    if not bookmarks:
        print("  No bookmarks found in this document.")
        return

    def count_all(bms: list[paperjam.Bookmark]) -> int:
        return sum(1 + count_all(list(b.children)) for b in bms)

    total = count_all(bookmarks)
    print(f"  {len(bookmarks)} top-level, {total} total\n")

    lines = print_tree(bookmarks)

    out = output / "bookmarks.txt"
    out.write_text("\n".join(lines) + "\n")
    print(f"\nSaved to {out}")


if __name__ == "__main__":
    main()
