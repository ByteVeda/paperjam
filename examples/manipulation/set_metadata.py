"""Edit PDF document metadata fields."""

import argparse
import os
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Edit PDF metadata (title, author, subject, etc.).",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument("--title", default=None, help="Set the document title")
    parser.add_argument("--author", default=None, help="Set the document author")
    parser.add_argument("--subject", default=None, help="Set the document subject")
    parser.add_argument("--keywords", default=None, help="Set document keywords")
    parser.add_argument("--creator", default=None, help="Set the creator application")
    parser.add_argument("--producer", default=None, help="Set the PDF producer")
    parser.add_argument(
        "--remove",
        nargs="*",
        default=[],
        metavar="FIELD",
        help="Remove specific fields, e.g. --remove keywords producer",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="Output filename (default: metadata_<input>.pdf)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    # Show current metadata
    meta = doc.metadata
    print("\nCurrent metadata:")
    for field in ("title", "author", "subject", "keywords", "creator", "producer"):
        print(f"  {field:12s}: {getattr(meta, field)}")

    # Build kwargs — only pass fields the user explicitly set
    kwargs: dict[str, str | None] = {}
    for field in ("title", "author", "subject", "keywords", "creator", "producer"):
        value = getattr(args, field)
        if value is not None:
            kwargs[field] = value

    # Handle removals (set to None to delete the field)
    for field in args.remove:
        field = field.lower()
        if field not in ("title", "author", "subject", "keywords", "creator", "producer"):
            parser.error(f"Unknown field to remove: {field}")
        kwargs[field] = None

    if not kwargs:
        print("\nNo changes specified. Use --title, --author, etc. or --remove.")
        return

    result = doc.set_metadata(**kwargs)

    # Show updated metadata
    new_meta = result.metadata
    print("\nUpdated metadata:")
    for field in ("title", "author", "subject", "keywords", "creator", "producer"):
        old = getattr(meta, field)
        new = getattr(new_meta, field)
        marker = " *" if old != new else ""
        print(f"  {field:12s}: {new}{marker}")

    basename = os.path.splitext(os.path.basename(args.input))[0]
    name = args.name or f"metadata_{basename}.pdf"
    out = output / name
    result.save(str(out))

    print(f"\nSaved to {out} ({out.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
