"""Create or replace bookmarks (table of contents) in a PDF."""

import argparse
import json
import os
from pathlib import Path

import paperjam


def print_tree(bookmarks: list[paperjam.Bookmark], indent: int = 0) -> None:
    """Pretty-print a bookmark tree."""
    for bm in bookmarks:
        prefix = "  " * indent
        print(f"  {prefix}{bm.title} -> page {bm.page}")
        if bm.children:
            print_tree(list(bm.children), indent + 1)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Create or replace bookmarks in a PDF.",
        epilog=(
            "Bookmark specs can be given inline or via a JSON file.\n\n"
            "Inline:  -b 'Chapter 1:1' 'Chapter 2:5' 'Chapter 3:10'\n"
            "JSON:    --json bookmarks.json\n\n"
            "JSON format:\n"
            '  [{"title": "Ch 1", "page": 1, "children": [\n'
            '      {"title": "Section 1.1", "page": 2}\n'
            "  ]}]"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
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
        "-b",
        "--bookmarks",
        nargs="+",
        metavar="TITLE:PAGE",
        help="Flat list of bookmarks as 'title:page' pairs",
    )
    group.add_argument(
        "--json",
        metavar="FILE",
        help="JSON file with bookmark tree (supports nesting)",
    )
    group.add_argument(
        "--clear",
        action="store_true",
        help="Remove all bookmarks from the document",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="Output filename (default: bookmarked_<input>.pdf)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    try:
        existing = doc.bookmarks
    except Exception:
        existing = []
    if existing:
        print(f"\nExisting bookmarks ({len(existing)} top-level):")
        print_tree(existing)
    else:
        print("\nNo existing bookmarks.")

    # Build bookmark list
    if args.clear:
        bookmarks: list[paperjam.Bookmark] = []
        print("\nClearing all bookmarks.")
    elif args.json:
        raw = json.loads(Path(args.json).read_text())
        bookmarks = _parse_json_bookmarks(raw)
        print(f"\nLoaded {len(bookmarks)} top-level bookmarks from {args.json}")
    else:
        bookmarks = []
        for spec in args.bookmarks:
            title, page_str = spec.rsplit(":", 1)
            bookmarks.append(paperjam.Bookmark(title=title, page=int(page_str), level=0))
        print(f"\nCreating {len(bookmarks)} bookmarks")

    result = doc.set_bookmarks(bookmarks)

    # Show result
    new_bookmarks = result.bookmarks
    if new_bookmarks:
        print(f"\nNew bookmarks ({len(new_bookmarks)} top-level):")
        print_tree(new_bookmarks)
    else:
        print("\nBookmarks removed.")

    basename = os.path.splitext(os.path.basename(args.input))[0]
    name = args.name or f"bookmarked_{basename}.pdf"
    out = output / name
    result.save(str(out))

    print(f"\nSaved to {out} ({out.stat().st_size:,} bytes)")


def _parse_json_bookmarks(items: list[dict]) -> list[paperjam.Bookmark]:
    """Recursively parse JSON bookmark dicts into Bookmark objects."""
    result = []
    for item in items:
        children = _parse_json_bookmarks(item.get("children", []))
        result.append(
            paperjam.Bookmark(
                title=item["title"],
                page=item["page"],
                level=0,
                children=tuple(children),
            )
        )
    return result


if __name__ == "__main__":
    main()
